# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "optparse"
require "rubocop"
require "stringio"
require "time"

require_relative "../lib/rustocop/config_serialization"

ROOT = File.expand_path("..", __dir__)
FIXTURE_ROOT = File.join(ROOT, "spec", "fixtures")
COP_ROOT = File.join(FIXTURE_ROOT, "cops")
MANIFEST_PATH = File.join(FIXTURE_ROOT, "unit_manifest.json")
DEFAULT_CAPTURE = File.join(ROOT, "tmp", "rubocop-1.87.0-cop-cases.jsonl")
FORMAT_VERSION = 1
MAX_CORRECTION_ITERATIONS = 200

options = { check: false, capture: DEFAULT_CAPTURE }
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/generate_unit_fixtures.rb [--check] [--capture PATH]"
  parser.on("--check", "validate the committed unit fixture cache without running RuboCop") do
    options[:check] = true
  end
  parser.on("--capture PATH", "captured upstream JSONL source") do |path|
    options[:capture] = File.expand_path(path)
  end
end.parse!

def encoded_source(value)
  return value unless value.is_a?(Hash) && value.key?("$hex")

  { "hex" => value.fetch("$hex") }
end

def decoded_source(value)
  return value unless value.is_a?(Hash) && value.key?("$hex")

  [value.fetch("$hex")].pack("H*")
end

def captured_value(value)
  utf8 = value.dup.force_encoding(Encoding::UTF_8)
  return utf8 if utf8.valid_encoding?

  { "hex" => value.b.unpack1("H*") }
end

def rubocop_config_value(value)
  case value
  when Hash
    return Regexp.new(value.fetch("$regexp"), value.fetch("options", 0)) if value.key?("$regexp")
    return value.fetch("$float").to_f if value.key?("$float")

    value.to_h { |key, child| [key, rubocop_config_value(child)] }
  when Array
    value.map { |child| rubocop_config_value(child) }
  else
    value
  end
end

def rubocop_config(test_case)
  config = RuboCop::Config.new(rubocop_config_value(test_case.fetch("config")))
  RuboCop::ConfigLoader.merge_with_default(config, test_case.fetch("path"))
end

def normalized_offenses(test_case)
  source = decoded_source(test_case.fetch("source"))
  Array(test_case["offenses"]).map do |offense|
    captured = offense.slice(
      "message", "severity", "correctable", "line", "column", "last_line", "last_column"
    )
    eof_line = source.count("\n") + 1
    eof_column = source.rpartition("\n").last.each_char.count + 1
    insertion_at_nonterminated_eof = !source.empty? && !source.end_with?("\n") &&
                                      captured["line"] == eof_line &&
                                      captured["column"] == eof_column
    if !insertion_at_nonterminated_eof && captured["last_line"] == captured["line"] &&
       captured["last_column"] + 1 == captured["column"]
      captured["last_column"] = captured["column"]
    end
    captured["last_column"] = 1 if captured["last_line"] > captured["line"] && captured["last_column"].zero?
    captured
  end.sort_by do |offense|
    offense.values_at("line", "column", "last_line", "last_column", "message", "severity")
  end
end

def with_encodings(test_case)
  external = Encoding.find(test_case.fetch("default_external_encoding"))
  internal_name = test_case["default_internal_encoding"]
  internal = internal_name && Encoding.find(internal_name)
  previous_external = Encoding.default_external
  previous_internal = Encoding.default_internal
  Encoding.default_external = external
  Encoding.default_internal = internal
  yield
ensure
  Encoding.default_external = previous_external
  Encoding.default_internal = previous_internal
end

def rubocop_correction(test_case)
  source = decoded_source(test_case.fetch("source"))
  config = rubocop_config(test_case)
  cop_class = RuboCop::Cop::Registry.global.find_by_cop_name(test_case.fetch("cop"))
  raise "unknown RuboCop cop #{test_case.fetch('cop')}" unless cop_class

  registry = RuboCop::Cop::Registry.new([cop_class])
  path = test_case.fetch("path")
  parser_engine = test_case.fetch("parser_engine").to_sym
  ruby_version = test_case.fetch("ruby_version").to_f
  options = { autocorrect: true, safe_autocorrect: false, raise_error: true, stdin: source }
  seen = { source => true }

  with_encodings(test_case) do
    MAX_CORRECTION_ITERATIONS.times do
      options[:stdin] = source
      processed = RuboCop::ProcessedSource.new(
        source, ruby_version, path, parser_engine: parser_engine
      )
      processed.registry = registry
      processed.config = config
      cop = cop_class.new(config, options)
      $stderr = StringIO.new
      report = RuboCop::Cop::Team.new([cop], config, options).investigate(processed)
      $stderr = STDERR
      corrector = report.correctors.first
      corrected = corrector ? corrector.rewrite : source
      return { "source" => source } if corrected == source
      return { "source" => corrected, "error" => "infinite_loop" } if seen[corrected]

      source = corrected
      seen[source] = true
    end
  ensure
    $stderr = STDERR
  end
  { "source" => source, "error" => "maximum_iterations" }
rescue SystemExit => error
  { "source" => source, "error" => "exit_#{error.status}" }
end

def safe_autocorrect?(test_case)
  config = rubocop_config(test_case)
  cop_class = RuboCop::Cop::Registry.global.find_by_cop_name(test_case.fetch("cop"))
  cop_class.new(config).safe_autocorrect?
end

def input_key(test_case)
  JSON.generate(test_case.slice(
    "cop", "source", "path", "ruby_version", "parser_engine",
    "default_external_encoding", "default_internal_encoding", "config", "file_mode", "lsp"
  ))
end

def validate_cache!
  abort "unit fixture manifest not found: #{MANIFEST_PATH}" unless File.file?(MANIFEST_PATH)

  manifest = JSON.parse(File.read(MANIFEST_PATH))
  problems = []
  problems << "format version" unless manifest["version"] == FORMAT_VERSION
  case_count = 0
  ids = []
  manifest.fetch("cops").each do |cop, entry|
    cases_path = File.join(FIXTURE_ROOT, entry.fetch("cases"))
    configs_path = File.join(FIXTURE_ROOT, entry.fetch("configs"))
    problems << "missing #{entry.fetch('cases')}" unless File.file?(cases_path)
    problems << "missing #{entry.fetch('configs')}" unless File.file?(configs_path)
    next unless File.file?(cases_path) && File.file?(configs_path)

    digest = Digest::SHA256.file(cases_path).hexdigest
    problems << "changed #{entry.fetch('cases')}" unless digest == entry.fetch("sha256")
    configs_digest = Digest::SHA256.file(configs_path).hexdigest
    problems << "changed #{entry.fetch('configs')}" unless configs_digest == entry.fetch("configs_sha256")
    cases = File.foreach(cases_path).map { |line| JSON.parse(line) }
    problems << "wrong owner in #{entry.fetch('cases')}" unless cases.all? { |item| item.fetch("cop") == cop }
    problems << "wrong count in #{entry.fetch('cases')}" unless cases.length == entry.fetch("count")
    configs = JSON.parse(File.read(configs_path))
    missing_configs = cases.map { |item| item.fetch("config") }.uniq - configs.keys
    problems << "missing configs in #{entry.fetch('configs')}: #{missing_configs.join(', ')}" unless missing_configs.empty?
    case_count += cases.length
    ids.concat(cases.map { |item| item.fetch("id") })
  end
  problems << "total case count" unless case_count == manifest.fetch("controlled_cases")
  problems << "duplicate controlled ids" unless ids.uniq.length == ids.length
  abort "unit fixture cache errors:\n  - #{problems.join("\n  - ")}" unless problems.empty?

  puts "unit fixture cache is valid: #{case_count} controlled cases across #{manifest.fetch('cops').length} cops"
end

if options[:check]
  validate_cache!
  exit
end

abort "captured corpus not found: #{options[:capture]}" unless File.file?(options[:capture])
abort "loaded RuboCop #{RuboCop::Version::STRING}, expected 1.87.0" unless RuboCop::Version::STRING == "1.87.0"

raw_sha256 = Digest::SHA256.file(options[:capture]).hexdigest
raw_cases = File.foreach(options[:capture]).map { |line| JSON.parse(line) }
comparable = raw_cases.reject { |test_case| test_case.fetch("lsp", false) }
grouped = comparable.group_by { |test_case| input_key(test_case) }

conflicts = grouped.values.filter_map do |cases|
  offenses = cases.map { |test_case| normalized_offenses(test_case) }.uniq
  [cases.first.fetch("cop"), cases.map { |item| item.dig("example", "id") }] if offenses.length > 1
end
abort "captured inputs have conflicting diagnostics: #{conflicts.inspect}" unless conflicts.empty?

controlled = grouped.values.map.with_index do |cases, index|
  test_case = cases.first
  source = decoded_source(test_case.fetch("source"))
  should_check_correction = cases.any? { |item| item.key?("correction") } ||
                            normalized_offenses(test_case).any? { |offense| offense.fetch("correctable") }
  all_result = should_check_correction ? rubocop_correction(test_case) : { "source" => source }
  all_output = all_result.fetch("source")
  safe = safe_autocorrect?(test_case)
  safe_output = safe ? all_output : source
  config_yaml = Rustocop::ConfigSerialization.rubocop_yaml(test_case.fetch("config"))
  config_id = Digest::SHA256.hexdigest(config_yaml).slice(0, 16)
  stable_input = input_key(test_case)
  id = Digest::SHA256.hexdigest(stable_input).slice(0, 20)
  warn "prepared #{index + 1}/#{grouped.length}" if ((index + 1) % 2_000).zero?
  {
    "id" => id,
    "cop" => test_case.fetch("cop"),
    "source" => encoded_source(test_case.fetch("source")),
    "path" => test_case.fetch("path"),
    "ruby_version" => test_case.fetch("ruby_version"),
    "parser_engine" => test_case.fetch("parser_engine"),
    "external_encoding" => test_case.fetch("default_external_encoding"),
    "internal_encoding" => test_case["default_internal_encoding"],
    "file_mode" => test_case["file_mode"],
    "config" => config_id,
    "config_yaml" => config_yaml,
    "diagnostics" => normalized_offenses(test_case),
    "autocorrect_checked" => should_check_correction,
    "autocorrect_all" => all_output == source ? nil : captured_value(all_output),
    "autocorrect_all_error" => all_result["error"],
    "autocorrect_safe" => if safe_output == source
                            nil
                          elsif safe_output == all_output
                            "$all"
                          else
                            captured_value(safe_output)
                          end,
    "autocorrect_safe_error" => safe ? all_result["error"] : nil,
    "origins" => cases.map { |item| item.fetch("example") }.uniq
  }
end

updated_at = Time.now.iso8601
entries = {}
controlled.group_by { |test_case| test_case.fetch("cop") }.sort.each do |cop, cases|
  department, name = cop.split("/", 2)
  unit_root = File.join(COP_ROOT, department, name, "unit")
  FileUtils.rm_rf(unit_root)
  FileUtils.mkdir_p(unit_root)
  configs = cases.to_h { |test_case| [test_case.fetch("config"), test_case.delete("config_yaml")] }
  cases_path = File.join(unit_root, "cases.jsonl")
  configs_path = File.join(unit_root, "configs.json")
  File.write(cases_path, cases.map { |test_case| JSON.generate(test_case) }.join("\n") + "\n")
  File.write(configs_path, JSON.pretty_generate(configs.sort.to_h) + "\n")
  entries[cop] = {
    "cases" => cases_path.delete_prefix("#{FIXTURE_ROOT}/"),
    "configs" => configs_path.delete_prefix("#{FIXTURE_ROOT}/"),
    "count" => cases.length,
    "sha256" => Digest::SHA256.file(cases_path).hexdigest,
    "configs_sha256" => Digest::SHA256.file(configs_path).hexdigest
  }
end

counts = entries.values.map { |entry| entry.fetch("count") }
manifest = {
  "version" => FORMAT_VERSION,
  "rubocop_version" => RuboCop::Version::STRING,
  "updated_at" => updated_at,
  "raw_capture_sha256" => raw_sha256,
  "raw_cases" => raw_cases.length,
  "lsp_exclusions" => raw_cases.length - comparable.length,
  "controlled_cases" => controlled.length,
  "exact_duplicates_removed" => comparable.length - controlled.length,
  "distribution" => {
    "minimum" => counts.min,
    "median" => counts.sort.fetch(counts.length / 2),
    "maximum" => counts.max
  },
  "cops" => entries
}
File.write(MANIFEST_PATH, JSON.pretty_generate(manifest) + "\n")
validate_cache!
puts "generated #{controlled.length} controlled cases from #{comparable.length} comparable captures"
