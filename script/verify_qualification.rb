# frozen_string_literal: true

require "json"
require "open3"
require "optparse"
require "tempfile"
require "yaml"

ROOT = File.expand_path("..", __dir__)
RUBOCOP_ROOT = Gem::Specification.find_by_name("rubocop", "1.87.0").full_gem_path
NATIVE = File.join(ROOT, "libexec/rustocop-native")
RUBOCOP_COMMIT = "e5b788dba181ad94de30cfbad661c5d6aa08a4e5"

options = { upstream: true, require_complete: false }
OptionParser.new do |parser|
  parser.on("--[no-]upstream") { |value| options[:upstream] = value }
  parser.on("--require-complete") { options[:require_complete] = true }
end.parse!

def command_output(command, source: nil)
  stdout, stderr, status = Open3.capture3(*command, stdin_data: source)
  unless [0, 1].include?(status.exitstatus) && stderr.empty?
    raise "#{command.join(" ")} failed (#{status.exitstatus}): #{stderr}"
  end

  JSON.parse(stdout)
end

def offenses(report)
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
  {
    "AllCops" => { "NewCops" => "disable", "TargetRubyVersion" => 3.4 },
    cop => { "Enabled" => true }.merge(values || {})
  }
end

def verify_case(cop, test_case)
  Tempfile.create(["rustocop-qualification", ".yml"]) do |config|
    config.write(YAML.dump(case_config(cop, test_case["config"])))
    config.flush
    common = ["--format", "json", "--only", cop, "--config", config.path, "--stdin", test_case.fetch("path")]
    rubocop = command_output(
      ["bundle", "exec", "rubocop", "--cache", "false", "--no-server", *common],
      source: test_case.fetch("source")
    )
    rustocop = command_output([NATIVE, *common], source: test_case.fetch("source"))
    [offenses(rubocop), offenses(rustocop)]
  end
end

records = Dir[File.join(ROOT, "qualification/work/*.yml")].sort.flat_map do |path|
  document = YAML.safe_load_file(path)
  abort "unsupported qualification schema in #{path}" unless document["schema"] == 1
  abort "wrong RuboCop version in #{path}" unless document["rubocop_version"] == "1.87.0"
  abort "wrong RuboCop commit in #{path}" unless document["rubocop_commit"] == RUBOCOP_COMMIT

  document.fetch("cops").map { |cop, record| [cop, record.merge("record_file" => path, "rustocop_commit" => document["rustocop_commit"])] }
end.to_h
abort "no qualification work records found" if records.empty?

errors = []
checks = Hash.new(0)

records.each do |cop, record|
  manual = record.fetch("manual_review")
  ruby_source = File.join(RUBOCOP_ROOT, record.dig("sources", "rubocop").to_s)
  rust_sources = Array(record.dig("sources", "rustocop"))
  manual_ok = manual["status"] == "passed" && Array(manual["notes"]).any? &&
              File.file?(ruby_source) && rust_sources.any? &&
              rust_sources.all? { |path| File.file?(File.join(ROOT, path)) }
  checks[1] += 1 if manual_ok
  errors << "#{cop}: incomplete manual source review" unless manual_ok

  upstream = record.fetch("upstream_tests")
  upstream_ok = upstream["status"] == "passed" && upstream["corrections"] == true &&
                upstream["total"].to_i.positive? && upstream["passed"] == upstream["total"]
  checks[2] += 1 if upstream_ok
  errors << "#{cop}: incomplete upstream diagnostic/correction result" unless upstream_ok

  edge_cases = Array(record["edge_cases"])
  edge_ok = edge_cases.length >= 4
  edge_cases.each do |test_case|
    expected, actual = verify_case(cop, test_case)
    next if expected == actual

    edge_ok = false
    errors << "#{cop}: edge case #{test_case["id"]} differs from RuboCop"
  rescue StandardError => e
    edge_ok = false
    errors << "#{cop}: edge case #{test_case["id"]} failed: #{e.message}"
  end
  checks[3] += 1 if edge_ok
  errors << "#{cop}: requires at least four passing edge cases" unless edge_ok

  real = record.fetch("real_world")
  real_results = {}
  { "positives" => true, "negatives" => false }.each do |kind, should_raise|
    examples = Array(real[kind])
    ok = examples.length >= 2
    examples.each do |test_case|
      provenance_ok = test_case["repository"].to_s.include?("/") &&
                      test_case["revision"].to_s.match?(/\A[0-9a-f]{40}\z/) &&
                      test_case["path"].to_s != "" && test_case["line"].to_i.positive?
      expected, actual = verify_case(cop, test_case)
      case_ok = provenance_ok && expected == actual && expected.empty? != should_raise
      next if case_ok

      ok = false
      errors << "#{cop}: real-world #{kind} case #{test_case["path"]}:#{test_case["line"]} is invalid or differs"
    rescue StandardError => e
      ok = false
      errors << "#{cop}: real-world #{kind} case failed: #{e.message}"
    end
    real_results[kind] = ok
  end
  checks[4] += 1 if real_results["positives"]
  checks[5] += 1 if real_results["negatives"]
  errors << "#{cop}: requires two passing real-world positives" unless real_results["positives"]
  errors << "#{cop}: requires two passing real-world negatives" unless real_results["negatives"]
end

if options[:upstream]
  cops = records.keys.sort.join(",")
  Tempfile.create(["rustocop-qualification-upstream", ".json"]) do |report|
    command = [
      "bundle", "exec", "ruby", File.join(ROOT, "script/compare_upstream_cop_specs.rb"),
      "--only", cops, "--corrections", "--jobs", "15", "--report", report.path
    ]
    _stdout, stderr, status = Open3.capture3(*command)
    result = JSON.parse(File.read(report.path))
    result.fetch("results").each do |cop, cop_result|
      next if cop_result["status"] == "passing"

      checks[2] -= 1 if records.dig(cop, "upstream_tests", "status") == "passed"
      errors << "#{cop}: live upstream suite passes #{cop_result["passed"]}/#{cop_result["total"]}"
    end
    errors << "upstream verifier failed: #{stderr}" unless status.success?
  end
end

commits = records.values.map { |record| record["rustocop_commit"] }.uniq
if options[:require_complete]
  errors << "qualification records do not share one Rust commit" unless commits.one?
  baseline = commits.first.to_s
  if baseline == "pending" || !baseline.match?(/\A[0-9a-f]{40}\z/)
    errors << "qualification Rust commit is not finalized"
  else
    _output, _stderr, status = Open3.capture3(
      "git", "diff", "--quiet", "#{baseline}..HEAD", "--", "crates/rustocop"
    )
    errors << "native Rust source changed after qualification commit #{baseline}" unless status.success?
  end
end

puts "Qualification records: #{records.length} cops"
(1..5).each { |check| puts "Check #{check}: #{checks[check]}/606" }
fully_qualified = records.count do |_cop, record|
  record.dig("manual_review", "status") == "passed" &&
    record.dig("upstream_tests", "status") == "passed" &&
    Array(record["edge_cases"]).length >= 4 &&
    Array(record.dig("real_world", "positives")).length >= 2 &&
    Array(record.dig("real_world", "negatives")).length >= 2
end
puts "Fully evidenced records (before live failures): #{fully_qualified}/606"

unless errors.empty?
  warn "Qualification failed with #{errors.length} problem(s):"
  errors.each { |error| warn "  - #{error}" }
  exit 1
end

puts "All recorded qualification evidence passed."
