# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "optparse"
require "set"
require "tempfile"
require "tmpdir"
require "yaml"

ROOT = File.expand_path("..", __dir__)
RUBOCOP_ROOT = Gem::Specification.find_by_name("rubocop", "1.87.0").full_gem_path
NATIVE = ENV.fetch("RUSTOCOP_NATIVE_PATH", File.join(ROOT, "libexec/rustocop-native"))
RUBOCOP_COMMIT = "e5b788dba181ad94de30cfbad661c5d6aa08a4e5"

options = {
  upstream: true,
  require_complete: false,
  only: nil,
  checks: (1..5).to_a,
  corpus: ENV["RUSTOCOP_UPSTREAM_CORPUS"]
}
OptionParser.new do |parser|
  parser.on("--[no-]upstream") { |value| options[:upstream] = value }
  parser.on("--require-complete") { options[:require_complete] = true }
  parser.on("--only COPS", "Comma-separated qualification records to verify") do |value|
    options[:only] = value.split(",").map(&:strip).reject(&:empty?).to_set
  end
  parser.on("--checks NUMBERS", "Comma-separated qualification checks to verify (1-5)") do |value|
    options[:checks] = value.split(",").map { |item| Integer(item, 10) }.uniq
    invalid = options[:checks] - (1..5).to_a
    raise OptionParser::InvalidArgument, "unknown checks: #{invalid.join(', ')}" unless invalid.empty?
  end
  parser.on("--corpus PATH", "Explicit extracted upstream corpus") do |value|
    options[:corpus] = File.expand_path(value)
  end
end.parse!

def capture_json(command, source: nil)
  stdout, stderr, status = Open3.capture3(*command, stdin_data: source)
  unless [0, 1].include?(status.exitstatus) && stderr.empty?
    raise "#{command.join(" ")} failed (#{status.exitstatus}): #{stderr}"
  end

  JSON.parse(stdout)
end

def normalized_offenses(report)
  report.fetch("files").flat_map do |file|
    file.fetch("offenses").map do |offense|
      offense.slice("cop_name", "severity", "message", "correctable", "corrected", "location")
    end
  end.sort_by do |offense|
    location = offense.fetch("location")
    [
      location.fetch("start_line"), location.fetch("start_column"),
      location.fetch("last_line"), location.fetch("last_column"),
      offense.fetch("cop_name"), offense.fetch("message")
    ]
  end
end

def case_config(cop, values)
  values ||= {}
  base = { "AllCops" => { "NewCops" => "disable", "TargetRubyVersion" => 3.4 } }
  if values.keys.any? { |key| key == "AllCops" || key.include?("/") }
    config = base.merge(values)
    config["AllCops"] = base.fetch("AllCops").merge(values.fetch("AllCops", {}))
    config[cop] = { "Enabled" => true }.merge(config.fetch(cop, {}))
    config
  else
    base.merge(cop => { "Enabled" => true }.merge(values))
  end
end

def corrected_source(command, cop, config_path, test_case)
  Dir.mktmpdir("rustocop-qualified-correction") do |directory|
    relative = test_case.fetch("path").sub(%r{\A/+}, "")
    relative = "example.rb" if relative.empty? || relative.start_with?("../")
    source_path = File.join(directory, relative)
    FileUtils.mkdir_p(File.dirname(source_path))
    File.binwrite(source_path, test_case.fetch("source"))
    stdout, stderr, status = Open3.capture3(
      *command, "-A", "--format", "json", "--only", cop,
      "--config", config_path, source_path
    )
    unless [0, 1].include?(status.exitstatus) && stderr.empty? && !stdout.empty?
      raise "autocorrection failed (#{status.exitstatus}): #{stderr}"
    end
    JSON.parse(stdout)
    File.binread(source_path)
  end
end

def verify_case(cop, test_case)
  Tempfile.create(["rustocop-qualification", ".yml"]) do |config|
    config.write(YAML.dump(case_config(cop, test_case["config"])))
    config.flush
    common = ["--format", "json", "--only", cop, "--config", config.path,
              "--stdin", test_case.fetch("path")]
    rubocop_command = ["bundle", "exec", "rubocop", "--cache", "false", "--no-server"]
    rubocop = capture_json([*rubocop_command, *common], source: test_case.fetch("source"))
    rustocop = capture_json([NATIVE, *common], source: test_case.fetch("source"))
    expected_correction = corrected_source(rubocop_command, cop, config.path, test_case)
    actual_correction = corrected_source([NATIVE], cop, config.path, test_case)
    [
      normalized_offenses(rubocop), normalized_offenses(rustocop),
      expected_correction, actual_correction
    ]
  end
end

record_files = Dir[File.join(ROOT, "qualification/work/*.yml")].sort
abort "no qualification work records found" if record_files.empty?

records = {}
record_files.each do |path|
  document = YAML.safe_load_file(path)
  abort "unsupported qualification schema in #{path}" unless document["schema"] == 1
  abort "wrong RuboCop version in #{path}" unless document["rubocop_version"] == "1.87.0"
  abort "wrong RuboCop commit in #{path}" unless document["rubocop_commit"] == RUBOCOP_COMMIT

  document.fetch("cops").each do |cop, record|
    abort "duplicate qualification record for #{cop}" if records.key?(cop)

    records[cop] = record.merge(
      "record_file" => path,
      "rustocop_commit" => document["rustocop_commit"]
    )
  end
end

if options[:only]
  missing = options[:only] - records.keys.to_set
  abort "unknown qualification records: #{missing.to_a.sort.join(", ")}" unless missing.empty?

  records.select! { |cop, _record| options[:only].include?(cop) }
end

errors = []
passed = Hash.new { |hash, cop| hash[cop] = {} }

records.each do |cop, record|
  if options[:checks].include?(1)
    manual = record.fetch("manual_review", {})
    ruby_source = File.join(RUBOC_ROOT, record.dig("sources", "rubocop").to_s)
    rust_sources = Array(record.dig("sources", "rustocop"))
    manual_ok = manual["status"] == "passed" && Array(manual["notes"]).length >= 2 &&
                File.file?(ruby_source) && rust_sources.any? &&
                rust_sources.all? { |path| File.file?(File.join(ROOT, path)) }
    passed[cop][1] = manual_ok
    errors << "#{cop}: incomplete manual source review" unless manual_ok
  end

  if options[:checks].include?(2)
    upstream = record.fetch("upstream_tests", {})
    upstream_ok = upstream["status"] == "passed" && upstream["corrections"] == true &&
                  upstream["total"].to_i.positive? && upstream["passed"] == upstream["total"]
    passed[cop][2] = upstream_ok
    errors << "#{cop}: incomplete upstream diagnostic/correction result" unless upstream_ok
  end

  if options[:checks].include?(3)
    edge_cases = Array(record["edge_cases"])
    edge_ok = edge_cases.length >= 4 && edge_cases.map { |item| item["id"] }.uniq.length == edge_cases.length
    edge_cases.each do |test_case|
      expected, actual, expected_correction, actual_correction = verify_case(cop, test_case)
      next if expected == actual && expected_correction == actual_correction

      edge_ok = false
      errors << "#{cop}: edge case #{test_case["id"]} diagnostics or correction differs"
    rescue StandardError => e
      edge_ok = false
      errors << "#{cop}: edge case #{test_case["id"]} failed: #{e.message}"
    end
    passed[cop][3] = edge_ok
    errors << "#{cop}: requires at least four distinct passing edge cases" unless edge_ok
  end

  real = record.fetch("real_world", {})
  { "positives" => [4, true], "negatives" => [5, false] }.each do |kind, (check, should_raise)|
    next unless options[:checks].include?(check)

    examples = Array(real[kind])
    unique_origins = examples.map { |item| [item["repository"], item["revision"], item["path"], item["line"]] }.uniq
    ok = examples.length >= 2 && unique_origins.length == examples.length
    examples.each do |test_case|
      provenance_ok = test_case["repository"].to_s.match?(%r{\A[^/]+/[^/]+\z}) &&
                      test_case["revision"].to_s.match?(/\A[0-9a-f]{40}\z/) &&
                      !test_case["path"].to_s.empty? && test_case["line"].to_i.positive?
      expected, actual, expected_correction, actual_correction = verify_case(cop, test_case)
      polarity_ok = should_raise ? !expected.empty? : expected.empty?
      case_ok = provenance_ok && polarity_ok && expected == actual &&
                expected_correction == actual_correction
      next if case_ok

      ok = false
      errors << "#{cop}: real-world #{kind} #{test_case["path"]}:#{test_case["line"]} is invalid or differs"
    rescue StandardError => e
      ok = false
      errors << "#{cop}: real-world #{kind} case failed: #{e.message}"
    end
    passed[cop][check] = ok
    errors << "#{cop}: requires two distinct passing real-world #{kind}" unless ok
  end
end

if options[:upstream] && options[:checks].include?(2)
  cops = records.keys.sort.join(",")
  Tempfile.create(["rustocop-qualification-upstream", ".json"]) do |report|
    command = [
      "bundle", "exec", "ruby", File.join(ROOT, "script/compare_upstream_cop_specs.rb"),
      "--only", cops, "--corrections", "--jobs", "15", "--report", report.path
    ]
    command.push("--corpus", options[:corpus]) if options[:corpus]
    _stdout, stderr, status = Open3.capture3(*command)
    result = JSON.parse(File.read(report.path))
    result.fetch("results").each do |cop, cop_result|
      next if cop_result["status"] == "passing"

      passed[cop][2] = false
      errors << "#{cop}: live upstream suite passes #{cop_result["passed"]}/#{cop_result["total"]}"
    end
    errors << "upstream verifier failed: #{stderr}" unless status.success? || stderr.empty?
  end
end

commits = records.values.map { |record| record["rustocop_commit"] }.uniq
if options[:require_complete]
  errors << "--require-complete cannot be combined with --only" if options[:only]
  positions = records.values.map { |record| record["matrix_position"] }
  errors << "qualification batch must contain matrix positions 1 through 60" unless positions.sort == (1..60).to_a
  errors << "qualification records do not share one Rust commit" unless commits.one?
  baseline = commits.first.to_s
  if baseline == "pending" || !baseline.match?(/\A[0-9a-f]{40}\z/)
    errors << "qualification Rust commit is not finalized"
  else
    _output, _stderr, exists = Open3.capture3("git", "cat-file", "-e", "#{baseline}^{commit}")
    errors << "qualification Rust commit does not exist: #{baseline}" unless exists.success?
    _output, _stderr, unchanged = Open3.capture3(
      "git", "diff", "--quiet", "#{baseline}..HEAD", "--", "crates/rustocop"
    )
    errors << "native Rust source changed after qualification commit #{baseline}" unless unchanged.success?
  end
end

puts "Qualification records: #{records.length} cops"
options[:checks].each do |check|
  count = passed.count { |_cop, checks| checks[check] }
  puts "Check #{check}: #{count}/606"
end
fully_qualified = passed.count { |_cop, checks| options[:checks].all? { |check| checks[check] } }
puts "Passed selected checks: #{fully_qualified}/606"

unless errors.empty?
  warn "Qualification failed with #{errors.length} problem(s):"
  errors.each { |error| warn "  - #{error}" }
  exit 1
end

puts "All recorded qualification evidence passed."
