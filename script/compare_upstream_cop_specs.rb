# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "thread"
require "tmpdir"
require "yaml"

root = File.expand_path("..", __dir__)
options = {
  corpus: File.join(root, "tmp/rubocop-1.87.0-cop-cases.jsonl"),
  jobs: 8,
  limit: nil,
  only: nil,
  corrections: false,
  report: File.join(root, "tmp/rubocop-1.87.0-compatibility.json")
}

OptionParser.new do |parser|
  parser.on("--corpus PATH") { |path| options[:corpus] = File.expand_path(path) }
  parser.on("--jobs COUNT", Integer) { |count| options[:jobs] = count }
  parser.on("--limit-per-cop COUNT", Integer) { |count| options[:limit] = count }
  parser.on("--only COPS", "comma-separated cop names") { |cops| options[:only] = cops.split(",") }
  parser.on("--corrections", "also verify asserted corrected source") { options[:corrections] = true }
  parser.on("--report PATH") { |path| options[:report] = File.expand_path(path) }
end.parse!

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

config_root = File.join(root, "tmp/upstream-rubocop-configs")
FileUtils.mkdir_p(config_root)
config_paths = {}
cases.each do |test_case|
  config = test_case.fetch("config")
  digest = Digest::SHA256.hexdigest(JSON.generate(config))
  config_paths[digest] ||= begin
    path = File.join(config_root, "#{digest}.yml")
    File.write(path, YAML.dump(config)) unless File.file?(path)
    path
  end
  test_case["config_path"] = config_paths.fetch(digest)
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
        offense.slice(
          "message", "severity", "correctable", "line", "column", "last_line", "last_column"
        )
      end
      passed = expected.nil? || (status.exitstatus == (expected.empty? ? 0 : 1) && actual == expected)
      correction_passed = nil
      if options[:corrections] && test_case.key?("correction")
        expected_correction = test_case["correction"]
        if expected_correction.is_a?(Hash) && expected_correction.key?("$hex")
          expected_correction = [expected_correction.fetch("$hex")].pack("H*")
        end
        expected_correction = expected_correction.b
        expected_correction = source if test_case.fetch("asserts_no_correction", false)

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
          correction_passed = acceptable_status && correction_stderr.empty? &&
                              File.binread(source_path) == expected_correction
        end
        passed &&= correction_passed
      end

      lock.synchronize do
        results << {
          "cop" => test_case.fetch("cop"),
          "example" => test_case.fetch("example"),
          "passed" => passed,
          "correction_passed" => correction_passed,
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
    "first_failure" => cop_results.find { |result| !result.fetch("passed") }
  }
end
summary = {
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
exit(summary.fetch("passed_cases") == summary.fetch("cases") ? 0 : 1)
