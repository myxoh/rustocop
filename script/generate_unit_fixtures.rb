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

options = { check: false, capture: DEFAULT_CAPTURE, supplements: [] }
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/generate_unit_fixtures.rb [--check] [--capture PATH] [--supplement PATH]"
  parser.on("--check", "validate the committed unit fixture cache without running RuboCop") do
    options[:check] = true
  end
  parser.on("--capture PATH", "captured upstream JSONL source") do |path|
    options[:capture] = File.expand_path(path)
  end
  parser.on("--supplement PATH", "additional controlled input JSONL to capture and retain") do |path|
    options[:supplements] << File.expand_path(path)
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
  values = rubocop_config_value(test_case.fetch("config"))
  values = Marshal.load(Marshal.dump(values))
  Array(test_case["selection"] || test_case.fetch("cop")).flat_map { |value| value.split(",") }.each do |cop|
    values[cop] = (values[cop] || {}).merge("Enabled" => true)
  end
  config = RuboCop::Config.new(values)
  RuboCop::ConfigLoader.merge_with_default(config, test_case.fetch("path"))
end

def selected_cop_classes(test_case)
  names = Array(test_case["selection"] || test_case.fetch("cop")).flat_map { |value| value.split(",") }
  names.map do |name|
    RuboCop::Cop::Registry.global.find_by_cop_name(name).tap do |cop_class|
      raise "unknown RuboCop cop #{name}" unless cop_class
    end
  end
end

def rubocop_investigation(test_case, autocorrect: false, safe_autocorrect: false, source: nil)
  source ||= decoded_source(test_case.fetch("source"))
  config = rubocop_config(test_case)
  cop_classes = selected_cop_classes(test_case)
  registry = RuboCop::Cop::Registry.new(cop_classes)
  options = {
    autocorrect:, safe_autocorrect:, raise_error: true, stdin: source,
    display_cop_names: false
  }
  processed = RuboCop::ProcessedSource.new(
    source,
    test_case.fetch("ruby_version").to_f,
    test_case.fetch("path"),
    parser_engine: test_case.fetch("parser_engine").to_sym
  )
  processed.registry = registry
  processed.config = config
  cops = cop_classes.map { |cop_class| cop_class.new(config, options) }
  report = RuboCop::Cop::Team.new(cops, config, options).investigate(processed)
  [report, source]
end

def captured_offenses(test_case)
  report, = with_encodings(test_case) { rubocop_investigation(test_case) }
  report.offenses.map do |offense|
    location = offense.location
    {
      "message" => offense.message,
      "severity" => offense.severity.name.to_s,
      "correctable" => offense.correctable?,
      "line" => location.line,
      "column" => location.column + 1,
      "last_line" => location.last_line,
      "last_column" => location.last_line > location.line && location.last_column.zero? ? 1 : location.last_column
    }
  end
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

def rubocop_correction(test_case, safe: false)
  source = decoded_source(test_case.fetch("source"))
  seen = { source => true }

  with_encodings(test_case) do
    MAX_CORRECTION_ITERATIONS.times do
      $stderr = StringIO.new
      report, = rubocop_investigation(test_case, autocorrect: true, safe_autocorrect: safe, source:)
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

def input_key(test_case)
  JSON.generate(test_case.slice(
    "cop", "selection", "source", "path", "ruby_version", "parser_engine",
    "default_external_encoding", "default_internal_encoding", "config", "file_mode", "lsp"
  ))
end

def preserved_imported_contracts
  return [] unless File.file?(MANIFEST_PATH)

  manifest = JSON.parse(File.read(MANIFEST_PATH))
  manifest.fetch("cops").values.flat_map do |entry|
    configs = JSON.parse(File.read(File.join(FIXTURE_ROOT, entry.fetch("configs"))))
    File.foreach(File.join(FIXTURE_ROOT, entry.fetch("cases"))).filter_map do |line|
      item = JSON.parse(line)
      next unless item.fetch("origins", []).any? { |origin| origin.key?("kind") }

      item.merge("config_yaml" => configs.fetch(item.fetch("config")))
    end
  end
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
base_raw_cases = raw_cases.length
options.fetch(:supplements).each do |path|
  abort "supplement not found: #{path}" unless File.file?(path)

  raw_cases.concat(File.foreach(path).map { |line| JSON.parse(line) })
end
preserved = preserved_imported_contracts
comparable = raw_cases.reject { |test_case| test_case.fetch("lsp", false) }
base_comparable = raw_cases.first(base_raw_cases).reject { |test_case| test_case.fetch("lsp", false) }
grouped = comparable.group_by { |test_case| input_key(test_case) }

conflicts = grouped.values.filter_map do |cases|
  offenses = cases.map { |test_case| normalized_offenses(test_case) }.uniq
  [cases.first.fetch("cop"), cases.map { |item| item.dig("example", "id") }] if offenses.length > 1
end
abort "captured inputs have conflicting diagnostics: #{conflicts.inspect}" unless conflicts.empty?

controlled = grouped.values.map.with_index do |cases, index|
  test_case = cases.first
  source = decoded_source(test_case.fetch("source"))
  should_check_correction = cases.any? { |item| item.key?("correction") || item["check_autocorrect"] } ||
                            normalized_offenses(test_case).any? { |offense| offense.fetch("correctable") }
  all_result = should_check_correction ? rubocop_correction(test_case, safe: false) : { "source" => source }
  safe_result = should_check_correction ? rubocop_correction(test_case, safe: true) : { "source" => source }
  all_output = all_result.fetch("source")
  safe_output = safe_result.fetch("source")
  config_yaml = Rustocop::ConfigSerialization.rubocop_yaml(test_case.fetch("config"))
  config_id = Digest::SHA256.hexdigest(config_yaml).slice(0, 16)
  stable_input = input_key(test_case)
  id = Digest::SHA256.hexdigest(stable_input).slice(0, 20)
  warn "prepared #{index + 1}/#{grouped.length}" if ((index + 1) % 2_000).zero?
  controlled_case = {
    "id" => id,
    "cop" => test_case.fetch("cop"),
    "selection" => test_case["selection"],
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
    "autocorrect_safe_error" => safe_result["error"],
    "origins" => cases.map { |item| item.fetch("example") }.uniq
  }
  controlled_case.delete("selection") unless controlled_case["selection"]
  controlled_case
end

controlled_by_id = controlled.to_h { |test_case| [test_case.fetch("id"), test_case] }
preserved.each do |test_case|
  existing = controlled_by_id[test_case.fetch("id")]
  if existing
    existing["origins"] = (existing.fetch("origins") + test_case.fetch("origins")).uniq
  else
    controlled << test_case
    controlled_by_id[test_case.fetch("id")] = test_case
  end
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
imported_case_count = controlled.count do |test_case|
  test_case.fetch("origins", []).any? { |origin| origin.key?("kind") }
end
manifest = {
  "version" => FORMAT_VERSION,
  "rubocop_version" => RuboCop::Version::STRING,
  "updated_at" => updated_at,
  "raw_capture_sha256" => raw_sha256,
  "raw_cases" => base_raw_cases,
  "lsp_exclusions" => base_raw_cases - base_comparable.length,
  "controlled_cases" => controlled.length,
  "imported_cases" => imported_case_count,
  "exact_duplicates_removed" => base_comparable.length - base_comparable.group_by { |item| input_key(item) }.length,
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
