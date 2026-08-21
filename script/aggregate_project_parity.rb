# frozen_string_literal: true

require "fileutils"
require "json"
require "optparse"
require "rubocop"
require "time"

ROOT = File.expand_path("..", __dir__)
options = {
  input: File.join(ROOT, "tmp/project-parity/batches/*.json"),
  report: File.join(ROOT, "tmp/project-parity/all-cops.json"),
  markdown: File.join(ROOT, "tmp/project-parity/all-cops.md")
}

OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/aggregate_project_parity.rb [options]"
  parser.on("--input GLOB") { |value| options[:input] = File.expand_path(value) }
  parser.on("--report PATH") { |value| options[:report] = File.expand_path(value) }
  parser.on("--markdown PATH") { |value| options[:markdown] = File.expand_path(value) }
end.parse!

paths = Dir[options[:input]].sort
abort "no project-gate reports matched #{options[:input]}" if paths.empty?
reports = paths.map { |path| JSON.parse(File.read(path)).merge("report_path" => path) }
commits = reports.map { |report| report.fetch("rust_commit") }.uniq
versions = reports.map { |report| report.fetch("rubocop_version") }.uniq
abort "project-gate reports use multiple Rust commits" unless commits.one?
abort "project-gate reports use multiple RuboCop versions" unless versions.one?

combined = {}
reports.each do |report|
  report.fetch("combined_by_cop").each do |cop, row|
    abort "duplicate project-gate result for #{cop}" if combined.key?(cop)

    combined[cop] = row
  end
end

matrix = RuboCop::Cop::Registry.global.map(&:cop_name).sort
positions = combined.keys.to_h { |cop| [cop, matrix.index(cop).to_i + 1] }
ordered = combined.sort_by { |cop, _row| -positions.fetch(cop) }
expected = (positions.values.min..positions.values.max).to_a
abort "project-gate reports do not cover a contiguous matrix range" unless positions.values.sort == expected

summary = combined.values.group_by { |row| row.fetch("classification") }.transform_values(&:length)
totals = %w[rustocop rubocop exact].to_h do |key|
  [key, combined.values.sum { |row| row[key].to_i }]
end
timing = reports.flat_map { |report| report.fetch("projects").values }
                .each_with_object({ "rustocop" => 0.0, "rubocop" => 0.0 }) do |project, output|
  output.each_key { |engine| output[engine] += project.dig("timing_seconds", engine) }
end

payload = {
  "generated_at" => Time.now.iso8601,
  "rust_commit" => commits.fetch(0),
  "rubocop_version" => versions.fetch(0),
  "matrix_start" => positions.values.min,
  "matrix_end" => positions.values.max,
  "summary" => summary,
  "diagnostic_totals" => totals,
  "timing_seconds" => timing,
  "combined_by_cop" => ordered.to_h
}
FileUtils.mkdir_p(File.dirname(options[:report]))
File.write(options[:report], JSON.pretty_generate(payload))

exact = ordered.select { |_cop, row| row.fetch("classification") == "project_exact" }
failures = ordered.select { |_cop, row| %w[crash rubocop_error].include?(row.fetch("classification")) }
failure_details = reports.flat_map do |report|
  report.fetch("crashes", []).map { |item| item.merge("classification" => "crash") } +
    report.fetch("rubocop_errors", []).map { |item| item.merge("classification" => "rubocop_error") }
end.to_h { |item| [item.fetch("cop"), item] }
scope_note = if positions.values.min == 1
               "This completes the project-first gate for the remaining reverse-order range."
             else
               "Positions 1–#{positions.values.min - 1} are outside this aggregate."
             end
rows = ordered.map do |cop, row|
  counts = %w[rustocop rubocop exact].map { |key| row[key].nil? ? "—" : row.fetch(key).to_s }
  "| #{positions.fetch(cop)} | `#{cop}` | #{counts.join(' | ')} | #{row.fetch('classification')} |"
end

markdown = <<~MARKDOWN
  # Remaining cop project gate: positions #{positions.values.max}–#{positions.values.min}

  Generated from #{reports.length} batched project-gate reports against Rust source
  `#{payload.fetch('rust_commit')}` and RuboCop #{payload.fetch('rubocop_version')}.
  #{scope_note}

  ## Summary

  | Classification | Cops |
  | --- | ---: |
  | Project-exact | #{summary.fetch('project_exact', 0)} |
  | Exact but dormant | #{summary.fetch('dormant', 0)} |
  | Diagnostic mismatch | #{summary.fetch('mismatch', 0)} |
  | Rustocop crash | #{summary.fetch('crash', 0)} |
  | RuboCop gate error | #{summary.fetch('rubocop_error', 0)} |
  | **Total** | **#{combined.length}** |

  The #{reports.length} final comparison runs took #{format('%.1f', timing.fetch('rustocop'))}
  seconds in Rustocop and #{format('%.1f', timing.fetch('rubocop'))} seconds in
  RuboCop. Timings exclude crash/error isolation probes.

  ## Project-exact cops

  These #{exact.length} cops match complete diagnostic signatures across every
  pinned project. Fixture and autocorrection coverage remain separate requirements.

  #{exact.map { |cop, _row| "- `#{cop}`" }.join("\n")}

  ## Engine failures

  #{failures.map do |cop, row|
    detail = failure_details.fetch(cop)
    message = detail.fetch("stderr").lines.map(&:strip).reject(&:empty?).first(3).join(" ")
    "- `#{cop}`: `#{row.fetch('classification')}` on #{detail.fetch('project')}: #{message}"
  end.join("\n")}

  ## Complete classification

  | Position | Cop | Rustocop | RuboCop | Exact | Classification |
  | ---: | --- | ---: | ---: | ---: | --- |
  #{rows.join("\n")}
MARKDOWN
FileUtils.mkdir_p(File.dirname(options[:markdown]))
File.write(options[:markdown], markdown)

puts "Aggregated #{combined.length} cops from #{reports.length} reports."
puts "Summary: #{summary.sort.map { |key, value| "#{key}=#{value}" }.join(', ')}"
puts "Report: #{options[:report]}"
puts "Markdown: #{options[:markdown]}"
