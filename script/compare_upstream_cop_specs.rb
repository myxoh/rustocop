# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "thread"
require "tmpdir"
require "time"
require "yaml"
require_relative "../lib/rustocop/compatibility_baseline"
require_relative "../lib/rustocop/config_serialization"

root = File.expand_path("..", __dir__)
options = {
  corpus: File.join(root, "tmp/rubocop-1.87.0-cop-cases.jsonl"),
  jobs: 8,
  limit: nil,
  only: nil,
  corrections: false,
  baseline: nil,
  report: File.join(root, "tmp/rubocop-1.87.0-compatibility.json")
}

OptionParser.new do |parser|
  parser.on("--corpus PATH") { |path| options[:corpus] = File.expand_path(path) }
  parser.on("--jobs COUNT", Integer) { |count| options[:jobs] = count }
  parser.on("--limit-per-cop COUNT", Integer) { |count| options[:limit] = count }
  parser.on("--only COPS", "comma-separated cop names") { |cops| options[:only] = cops.split(",") }
  parser.on("--corrections", "also verify asserted corrected source") { options[:corrections] = true }
  parser.on("--baseline PATH", "reject regressions below an approved compatibility baseline") do |path|
    options[:baseline] = File.expand_path(path)
  end
  parser.on("--report PATH") { |path| options[:report] = File.expand_path(path) }
end.parse!

if options[:baseline] && (options[:only] || options[:limit] || options[:corrections])
  abort "--baseline requires a complete diagnostic run without --only, --limit-per-cop, or --corrections"
end

native = ENV.fetch("RUSTOCOP_NATIVE_PATH", File.join(root, "libexec/rustocop-native"))
abort "native Rustocop executable not found at #{native}" unless File.executable?(native)
abort "captured corpus not found; run script/extract_upstream_cop_specs.rb" unless File.file?(options[:corpus])

cases = []
per_cop = Hash.new(0)
File.foreach(options[:corpus]) do |line|
  test_case = JSON.parse(line)
  cop = test_case.fetch("cop")
  next if options[:only] && !options[:only].include?(cop)
  next if options[:limit] && per_cop[cop] >= options[:limit]

  per_cop[cop] += 1
  cases << test_case
end
abort "no captured upstream cases matched the requested cops" if cases.empty?

# RuboCop's callback order is an implementation detail and can differ between
# Parser and Prism for nested nodes. Compatibility is about the diagnostics,
# not the order in which the two engines discovered identical offenses.
offense_order = lambda do |offense|
  [
    offense.fetch("line"), offense.fetch("column"), offense.fetch("last_line"),
    offense.fetch("last_column"), offense.fetch("message"), offense.fetch("severity"),
    offense.fetch("correctable") ? 1 : 0
  ]
end

config_root = File.join(root, "tmp/upstream-rubocop-configs")
FileUtils.mkdir_p(config_root)
config_paths = {}
cases.each do |test_case|
  config = test_case.fetch("config")
  all_cops = config.fetch("AllCops", {}).merge(
    "TargetRubyVersion" => test_case.fetch("ruby_version")
  )
  config = config.merge("AllCops" => all_cops)
  rendered_config = Rustocop::ConfigSerialization.rubocop_yaml(config)
  digest = Digest::SHA256.hexdigest(rendered_config)
  config_paths[digest] ||= begin
    path = File.join(config_root, "#{digest}.yml")
    File.write(path, rendered_config) unless File.file?(path)
    path
  end
  test_case["config_path"] = config_paths.fetch(digest)
end

# Some upstream examples vary process-global state (for example,
# Encoding.default_external) that is not represented in the executable input.
# If otherwise identical captured inputs consequently have multiple asserted
# corrections, any of those upstream-produced corrections is a valid match.
correction_alternatives = cases.each_with_object(Hash.new { |hash, key| hash[key] = [] }) do |test_case, grouped|
  next unless test_case.key?("correction") && !test_case.fetch("asserts_no_correction", false)

  key = JSON.generate([
    test_case.fetch("cop"), test_case.fetch("source"), test_case.fetch("path"),
    test_case.fetch("ruby_version"), test_case.fetch("config")
  ])
  grouped[key] << test_case.fetch("correction")
end

queue = Queue.new
cases.each { |test_case| queue << test_case }
results = []
lock = Mutex.new

workers = Array.new(options[:jobs]) do
  Thread.new do
    loop do
      test_case = queue.pop(true)
      source = test_case.fetch("source")
      source = [source.fetch("$hex")].pack("H*") if source.is_a?(Hash) && source.key?("$hex")
      command = [
        native, "--format", "json", "--only", test_case.fetch("cop"),
        "--config", test_case.fetch("config_path"), "--stdin", test_case.fetch("path")
      ]
      stdout, stderr, status = Open3.capture3(*command, stdin_data: source)

      actual = if stdout.empty?
                 []
               else
                 JSON.parse(stdout).fetch("files").first.fetch("offenses").map do |offense|
                   location = offense.fetch("location")
                   {
                     "message" => offense.fetch("message"),
                     "severity" => offense.fetch("severity"),
                     "correctable" => offense.fetch("correctable"),
                     "line" => location.fetch("start_line"),
                     "column" => location.fetch("start_column"),
                     "last_line" => location.fetch("last_line"),
                     "last_column" => location.fetch("last_column")
                   }
                 end
               end
      expected = test_case["offenses"]&.map do |offense|
        captured = offense.slice(
          "message", "severity", "correctable", "line", "column", "last_line", "last_column"
        )
        # Captured Parser ranges use column zero when the range ends exactly
        # at a newline; RuboCop's public JSON formatter serializes that point
        # as column one.
        if captured["last_line"] > captured["line"] && captured["last_column"].zero?
          captured["last_column"] = 1
        end
        captured
      end
      diagnostics_match = expected.nil? || actual.sort_by(&offense_order) == expected.sort_by(&offense_order)
      passed = expected.nil? ||
               (status.exitstatus == (expected.empty? ? 0 : 1) && diagnostics_match)
      correction_passed = nil
      expected_correction = nil
      actual_correction = nil
      if options[:corrections] && test_case.key?("correction")
        expected_correction = if test_case.fetch("asserts_no_correction", false)
                                source
                              else
                                correction = test_case["correction"]
                                if correction.is_a?(Hash) && correction.key?("$hex")
                                  correction = [correction.fetch("$hex")].pack("H*")
                                end
                                correction.b
                              end

        Dir.mktmpdir("rustocop-upstream-case") do |directory|
          relative_path = test_case.fetch("path").sub(%r{\A/+}, "").tr("\\", "/")
          relative_path = "example.rb" if relative_path.empty? || relative_path.start_with?("../")
          source_path = File.join(directory, relative_path)
          FileUtils.mkdir_p(File.dirname(source_path))
          File.binwrite(source_path, source)
          _correction_stdout, correction_stderr, correction_status = Open3.capture3(
            native, "-A", "--format", "json", "--only", test_case.fetch("cop"),
            "--config", test_case.fetch("config_path"), source_path
          )
          acceptable_status = correction_status.success? ||
                              (test_case.fetch("asserts_no_correction", false) &&
                               correction_status.exitstatus == 1)
          actual_correction = File.binread(source_path)
          correction_key = JSON.generate([
            test_case.fetch("cop"), test_case.fetch("source"), test_case.fetch("path"),
            test_case.fetch("ruby_version"), test_case.fetch("config")
          ])
          alternatives = correction_alternatives.fetch(correction_key, []).map do |correction|
            if correction.is_a?(Hash) && correction.key?("$hex")
              [correction.fetch("$hex")].pack("H*")
            else
              correction.b
            end
          end
          correction_passed = acceptable_status && correction_stderr.empty? &&
                              ([expected_correction] + alternatives).include?(actual_correction)
        end
        passed &&= correction_passed
      end

      lock.synchronize do
        results << {
          "cop" => test_case.fetch("cop"),
          "example" => test_case.fetch("example"),
          "passed" => passed,
          "correction_passed" => correction_passed,
          "expected_correction" => correction_passed == false ? expected_correction : nil,
          "actual_correction" => correction_passed == false ? actual_correction : nil,
          "expected" => expected,
          "actual" => actual,
          "stderr" => stderr,
          "exit_status" => status.exitstatus
        }
      end
    rescue ThreadError
      break
    rescue StandardError => e
      lock.synchronize do
        results << {
          "cop" => test_case&.fetch("cop", "unknown"),
          "example" => test_case&.fetch("example", {}),
          "passed" => false,
          "error" => "#{e.class}: #{e.message}"
        }
      end
    end
  end
end
workers.each(&:join)

by_cop = results.group_by { |result| result.fetch("cop") }.sort.to_h.transform_values do |cop_results|
  passed = cop_results.count { |result| result.fetch("passed") }
  {
    "passed" => passed,
    "total" => cop_results.length,
    "status" => passed == cop_results.length ? "passing" : "failing",
    "first_failure" => cop_results.find { |result| !result.fetch("passed") },
    "failures" => cop_results.reject { |result| result.fetch("passed") }
  }
end
rust_commit, _git_error, git_status = Open3.capture3(
  "git", "log", "-1", "--format=%H", "--", "crates/rustocop", chdir: root
)
summary = {
  "generated_at" => Time.now.iso8601,
  "rust_commit" => git_status.success? ? rust_commit.strip : nil,
  "native_sha256" => Digest::SHA256.file(native).hexdigest,
  "fixture_corpus_sha256" => Digest::SHA256.file(options[:corpus]).hexdigest,
  "rubocop_version" => "1.87.0",
  "cases" => results.length,
  "passed_cases" => results.count { |result| result.fetch("passed") },
  "cops" => by_cop.length,
  "passing_cops" => by_cop.count { |_cop, result| result.fetch("status") == "passing" },
  "results" => by_cop
}

FileUtils.mkdir_p(File.dirname(options[:report]))
File.write(options[:report], JSON.pretty_generate(summary))
puts "#{summary.fetch("passed_cases")}/#{summary.fetch("cases")} cases pass; " \
     "#{summary.fetch("passing_cops")}/#{summary.fetch("cops")} cops pass every selected case"
puts "Report: #{options[:report]}"
if options[:baseline]
  baseline_errors = Rustocop::CompatibilityBaseline.errors(
    summary,
    YAML.safe_load(File.read(options[:baseline]))
  )
  if baseline_errors.empty?
    puts "Compatibility baseline preserved."
    exit 0
  end
  warn "Compatibility baseline regression:"
  baseline_errors.each { |error| warn "  - #{error}" }
  exit 1
end
exit(summary.fetch("passed_cases") == summary.fetch("cases") ? 0 : 1)
