# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "rubocop"
require "yaml"
require_relative "../lib/rustocop/compatibility_baseline"

ROOT = File.expand_path("..", __dir__)
STATUS_PATH = File.join(ROOT, "spec/upstream/rubocop-1.87.0/status.yml")
YAML_OUTPUT = File.join(ROOT, "spec/upstream/rubocop-1.87.0/remaining_cops.yml")
MARKDOWN_OUTPUT = File.join(ROOT, "docs/remaining-cops.md")

report_path = File.expand_path(ARGV.shift || File.join(ROOT, "tmp/rubocop-1.87.0-compatibility.json"))
native = ENV.fetch(
  "RUSTOCOP_NATIVE_PATH",
  File.join(ROOT, "crates/rustocop/target/debug/rustocop")
)
abort "compatibility report not found: #{report_path}" unless File.file?(report_path)
abort "native Rustocop executable not found: #{native}" unless File.executable?(native)

status = YAML.safe_load(File.read(STATUS_PATH))
report = JSON.parse(File.read(report_path))
verified = status.fetch("fully_compatible_cops")
registry = RuboCop::Cop::Registry.global.to_a
registry_names = registry.map(&:cop_name).sort
implemented_output, implemented_status = Open3.capture2(native, "--show-cops")
abort "could not read native cop registry" unless implemented_status.success?
implemented = implemented_output.lines.map(&:strip).reject(&:empty?)

expected_cases = status.fetch("captured_cases")
abort "report is for RuboCop #{report.fetch("rubocop_version")}, expected #{status.fetch("rubocop_version")}" unless report.fetch("rubocop_version") == status.fetch("rubocop_version")
abort "report is partial: #{report.fetch("cases")}/#{expected_cases} cases" unless report.fetch("cases") == expected_cases
abort "report is partial: #{report.fetch("cops")}/#{registry_names.length} cops" unless report.fetch("cops") == registry_names.length
reported_names = report.fetch("results").keys.sort
unless reported_names == registry_names
  missing = registry_names - reported_names
  extra = reported_names - registry_names
  abort "report cop registry differs (missing: #{missing.join(", ")}; extra: #{extra.join(", ")})"
end
baseline_errors = Rustocop::CompatibilityBaseline.errors(report, status)
unless baseline_errors.empty?
  abort "report regresses the approved baseline:\n  - #{baseline_errors.join("\n  - ")}"
end

capability_for = lambda do |name|
  department, short_name = name.split("/", 2)
  case department
  when "Layout" then "layout_engine"
  when "Metrics" then "metrics_engine"
  when "Bundler", "Gemspec" then "project_context"
  when "Naming" then "scope_and_symbols"
  else
    if short_name.match?(/PatternBranch|MatchPattern/)
      "control_flow"
    elsif short_name.match?(/Regexp|Regex/)
      "regexp_semantics"
    elsif short_name.match?(/Assignment|Constant|Variable|MethodDefinition|MissingSuper|Shadow/)
      "scope_and_symbols"
    elsif short_name.match?(/ControlFlow|Unreachable|Rescue|Return|Branch|Condition|Loop/)
      "control_flow"
    elsif short_name.match?(/Directive|MagicComment|Encoding|Permission|EndOfLine/)
      "file_metadata_and_lexing"
    else
      "ast_structural"
    end
  end
end

entries = registry.filter_map do |cop|
  name = cop.cop_name
  next if verified.include?(name)

  result = report.fetch("results").fetch(name)
  state = implemented.include?(name) ? "heuristic" : "missing"
  failures = result.fetch("total") - result.fetch("passed")
  capability = capability_for.call(name)
  lane = if state == "heuristic"
           "finish_existing"
         elsif failures <= 5 && capability == "ast_structural"
           "near_parity"
         elsif capability == "ast_structural"
           "structural_batch"
         else
           "engine_capability"
         end
  problem_path = File.join(ROOT, "missing/cops", *name.split("/"), "problem.md")
  {
    "cop" => name,
    "state" => state,
    "lane" => lane,
    "capability" => capability,
    "passing_cases" => result.fetch("passed"),
    "total_cases" => result.fetch("total"),
    "failing_cases" => failures,
    "autocorrect" => cop.support_autocorrect?,
    "problem_documented" => File.file?(problem_path)
  }
end

entries.sort_by! do |entry|
  lane_order = %w[finish_existing near_parity structural_batch engine_capability]
  [lane_order.index(entry.fetch("lane")), entry.fetch("failing_cases"), entry.fetch("cop")]
end

document = {
  "rubocop_version" => status.fetch("rubocop_version"),
  "generated_from" => report_path.delete_prefix("#{ROOT}/"),
  "remaining_cops" => entries.length,
  "states" => entries.group_by { |entry| entry.fetch("state") }.transform_values(&:length),
  "lanes" => entries.group_by { |entry| entry.fetch("lane") }.transform_values(&:length),
  "capabilities" => entries.group_by { |entry| entry.fetch("capability") }.transform_values(&:length),
  "cops" => entries
}
File.write(YAML_OUTPUT, YAML.dump(document))

lane_rows = document.fetch("lanes").map { |lane, count| "| `#{lane}` | #{count} |" }
capability_rows = document.fetch("capabilities").sort.map do |capability, count|
  "| `#{capability}` | #{count} |"
end
cop_rows = entries.map do |entry|
  "| `#{entry.fetch("cop")}` | #{entry.fetch("state")} | `#{entry.fetch("lane")}` | " \
    "`#{entry.fetch("capability")}` | #{entry.fetch("passing_cases")}/#{entry.fetch("total_cases")} | " \
    "#{entry.fetch("autocorrect") ? "Yes" : "No"} | #{entry.fetch("problem_documented") ? "Yes" : "No"} |"
end

markdown = <<~MARKDOWN
  # Remaining built-in cops

  Generated by `bundle exec ruby script/generate_remaining_cop_plan.rb REPORT`.
  This is a work queue, not a compatibility claim; [the support matrix](cop-support.md)
  remains authoritative for user-facing support.

  Of the #{entries.length} cops that are not yet Verified, #{document.dig("states", "heuristic")} have
  partial native implementations and #{document.dig("states", "missing")} have no native implementation.
  The queue is generated only from a complete #{expected_cases}-case report and refuses partial input.

  ## Delivery lanes

  | Lane | Cops |
  | --- | ---: |
  #{lane_rows.join("\n")}

  `finish_existing` closes gaps in native cops first. `near_parity` contains
  missing structural cops with at most five failing captured cases.
  `structural_batch` uses the current AST authoring APIs and is ordered by its
  observed compatibility gap. `engine_capability` is reserved for work that
  should improve the named shared capability before adding cop implementations.

  ## Work areas

  | Capability | Cops |
  | --- | ---: |
  #{capability_rows.join("\n")}

  ## Queue

  | Cop | State | Lane | Capability | Cases passing | Autocorrect | Problem documented |
  | --- | --- | --- | --- | ---: | --- | --- |
  #{cop_rows.join("\n")}
MARKDOWN
File.write(MARKDOWN_OUTPUT, markdown)

puts "wrote #{MARKDOWN_OUTPUT.delete_prefix("#{ROOT}/")} and #{YAML_OUTPUT.delete_prefix("#{ROOT}/")}: #{entries.length} cops"
