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
MANIFEST = File.join(ROOT, "crates/rustocop/Cargo.toml")
NATIVE = File.join(ROOT, "crates/rustocop/target/debug/rustocop")
RUBOCOP_ROOT = Gem::Specification.find_by_name("rubocop", "1.87.0").full_gem_path
RUBOCOP_COMMIT = "e5b788dba181ad94de30cfbad661c5d6aa08a4e5"

options = { upstream: true, require_complete: false, only: nil, corpus: nil }
OptionParser.new do |parser|
  parser.on("--[no-]upstream") { |value| options[:upstream] = value }
  parser.on("--require-complete") { options[:require_complete] = true }
  parser.on("--only COPS") { |value| options[:only] = value.split(",").map(&:strip).to_set }
  parser.on("--corpus PATH") { |value| options[:corpus] = File.expand_path(value) }
end.parse!

_output, build_error, built = Open3.capture3("cargo", "build", "--manifest-path", MANIFEST)
abort "Rust build failed: #{build_error}" unless built.success?

def capture_json(command, source: nil)
  stdout, stderr, status = Open3.capture3(*command, stdin_data: source)
  unless [0, 1].include?(status.exitstatus) && stderr.empty?
    raise "#{command.join(' ')} failed (#{status.exitstatus}): #{stderr}"
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
    [location.fetch("start_line"), location.fetch("start_column"),
     location.fetch("last_line"), location.fetch("last_column"),
     offense.fetch("cop_name"), offense.fetch("message")]
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
    [normalized_offenses(rubocop), normalized_offenses(rustocop),
     expected_correction, actual_correction]
  end
end

record_files = Dir[File.join(ROOT, "qualification/work/*.yml")].sort
abort "no qualification work records found" if record_files.empty?

records = {}
documents = record_files.to_h do |path|
  document = YAML.safe_load_file(path)
  abort "unsupported qualification schema in #{path}" unless document["schema"] == 1
  abort "wrong RuboCop version in #{path}" unless document["rubocop_version"] == "1.87.0"
  abort "wrong RuboCop commit in #{path}" unless document["rubocop_commit"] == RUBOCOP_COMMIT
  document.fetch("cops").each do |cop, record|
    abort "duplicate qualification record for #{cop}" if records.key?(cop)

    records[cop] = record.merge("record_file" => path,
                                "rustocop_commit" => document["rustocop_commit"])
  end
  [path, document]
end

if options[:only]
  missing = options[:only] - records.keys.to_set
  abort "unknown qualification records: #{missing.to_a.sort.join(', ')}" unless missing.empty?

  records.select! { |cop, _record| options[:only].include?(cop) }
end

errors = []
passed = Hash.new { |hash, cop| hash[cop] = {} }

records.each do |cop, record|
  manual = record.fetch("manual_review")
  ruby_source = File.join(RUBOCOP_ROOT, record.dig("sources", "rubocop").to_s)
  rust_sources = Array(record.dig("sources", "rustocop"))
  manual_ok = manual["status"] == "passed" && Array(manual["notes"]).length >= 2 &&
              File.file?(ruby_source) && rust_sources.any? &&
              rust_sources.all? { |path| File.file?(File.join(ROOT, path)) }
  passed[cop][1] = manual_ok
  errors << "#{cop}: incomplete manual source review" unless manual_ok

  baseline = record.fetch("rustocop_commit").to_s
  source_current = baseline.match?(/\A[0-9a-f]{40}\z/) && rust_sources.any?
  if source_current
    _output, _stderr, exists = Open3.capture3("git", "cat-file", "-e", "#{baseline}^{commit}")
    source_current &&= exists.success?
  end
  if source_current
    _output, _stderr, unchanged = Open3.capture3(
      "git", "diff", "--quiet", baseline, "--", *rust_sources
    )
    source_current &&= unchanged.success?
  end
  passed[cop][:source_current] = source_current
  errors << "#{cop}: native Rust source differs from qualification commit #{baseline}" unless source_current

  upstream = record.fetch("upstream_tests")
  upstream_ok = upstream["status"] == "passed" && upstream["corrections"] == true &&
                upstream["total"].to_i.positive? && upstream["passed"] == upstream["total"]
  passed[cop][2] = upstream_ok
  errors << "#{cop}: incomplete upstream diagnostic/correction result" unless upstream_ok

  edge_cases = Array(record["edge_cases"])
  edge_ok = edge_cases.length >= 4 && edge_cases.map { |item| item["id"] }.uniq.length == edge_cases.length
  edge_cases.each do |test_case|
    expected, actual, expected_correction, actual_correction = verify_case(cop, test_case)
    next if expected == actual && expected_correction == actual_correction

    edge_ok = false
    errors << "#{cop}: edge case #{test_case['id']} differs: " \
              "diagnostics #{expected.inspect} != #{actual.inspect}; " \
              "correction #{expected_correction.inspect} != #{actual_correction.inspect}"
  rescue StandardError => e
    edge_ok = false
    errors << "#{cop}: edge case #{test_case['id']} failed: #{e.message}"
  end
  passed[cop][3] = edge_ok
  errors << "#{cop}: requires at least four distinct passing edge cases" unless edge_ok

  real = record.fetch("real_world")
  { "positives" => [4, true], "negatives" => [5, false] }.each do |kind, (check, should_raise)|
    examples = Array(real[kind])
    origins = examples.map { |item| [item["repository"], item["revision"], item["path"], item["line"]] }
    ok = examples.length >= 2 && origins.uniq.length == examples.length
    examples.each do |test_case|
      provenance_ok = test_case["repository"].to_s.match?(%r{\A[^/]+/[^/]+\z}) &&
                      test_case["revision"].to_s.match?(/\A[0-9a-f]{40}\z/) &&
                      !test_case["path"].to_s.empty? && test_case["line"].to_i.positive?
      expected, actual, expected_correction, actual_correction = verify_case(cop, test_case)
      polarity_ok = should_raise ? !expected.empty? : expected.empty?
      next if provenance_ok && polarity_ok && expected == actual &&
              expected_correction == actual_correction

      ok = false
      errors << "#{cop}: real-world #{kind} #{test_case['path']}:#{test_case['line']} is invalid or differs: " \
                "diagnostics #{expected.inspect} != #{actual.inspect}; " \
                "correction #{expected_correction.inspect} != #{actual_correction.inspect}"
    rescue StandardError => e
      ok = false
      errors << "#{cop}: real-world #{kind} case failed: #{e.message}"
    end
    passed[cop][check] = ok
    errors << "#{cop}: requires two distinct passing real-world #{kind}" unless ok
  end
end

if options[:upstream]
  Tempfile.create(["rustocop-qualification-upstream", ".json"]) do |report|
    command = ["bundle", "exec", "ruby", File.join(ROOT, "script/compare_upstream_cop_specs.rb"),
               "--only", records.keys.join(","), "--corrections", "--jobs", "15",
               "--report", report.path]
    command.push("--corpus", options[:corpus]) if options[:corpus]
    _stdout, stderr, status = Open3.capture3({ "RUSTOCOP_NATIVE_PATH" => NATIVE }, *command)
    result = JSON.parse(File.read(report.path))
    result.fetch("results").each do |cop, cop_result|
      next if cop_result["status"] == "passing"

      passed[cop][2] = false
      errors << "#{cop}: live upstream suite passes #{cop_result['passed']}/#{cop_result['total']}"
    end
    errors << "upstream verifier failed: #{stderr}" unless status.success? || stderr.empty?
  end
end

if options[:require_complete]
  errors << "--require-complete cannot be combined with --only" if options[:only]
  documents.each_value do |document|
    start = document.fetch("matrix_start")
    finish = document.fetch("matrix_end")
    expected = (start..finish).to_a.reverse
    positions = document.fetch("cops").values.map { |record| record["matrix_position"] }
    errors << "#{document['batch']}: records must cover #{finish} down through #{start}" unless positions == expected
  end
end

puts "Qualification records: #{records.length} cops"
(1..5).each do |check|
  count = passed.count { |_cop, checks| checks[check] }
  puts "Check #{check}: #{count}/606"
end
source_current = passed.count { |_cop, checks| checks[:source_current] }
puts "Current Rust source: #{source_current}/606"
fully_qualified = passed.count do |_cop, checks|
  checks[:source_current] && (1..5).all? { |check| checks[check] }
end
puts "Fully qualified: #{fully_qualified}/606"

unless errors.empty?
  warn "Qualification failed with #{errors.length} problem(s):"
  errors.each { |error| warn "  - #{error}" }
  exit 1
end

puts "All recorded qualification evidence passed."
