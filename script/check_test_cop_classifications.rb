# frozen_string_literal: true

require "yaml"
require_relative "../lib/rustocop/compatibility_status"

ROOT = File.expand_path("..", __dir__)
CLASSIFICATIONS = YAML.safe_load_file(File.join(ROOT, "spec/cop_test_classifications.yml"))
STATUS = Rustocop::CompatibilityStatus.load(
  root: ROOT,
  version: CLASSIFICATIONS.fetch("version").to_s
)

test_files = [
  *Dir[File.join(ROOT, "spec/**/*.rb")],
  *Dir[File.join(ROOT, "crates/rustocop/src/**/*test*.rs")],
  *Dir[File.join(ROOT, "crates/rustocop/src/**/tests/**/*.rs")],
  *Dir[File.join(ROOT, "crates/rustocop/tests/fixtures/**/*")],
  *Dir[File.join(ROOT, "real_fixtures/**/source.yml")]
].uniq.select { |path| File.file?(path) }
test_files.reject! { |path| path.include?("/spec/upstream/") }

relative = ->(path) { path.delete_prefix("#{ROOT}/") }
non_behavioral = CLASSIFICATIONS.fetch("non_behavioral_files", [])
allowed = CLASSIFICATIONS.fetch("heuristic_regressions", {}).each_with_object({}) do |(cop, paths), index|
  paths.each { |path| index[[cop, path]] = true }
end
seen = {}
problems = []

test_files.each do |path|
  file = relative.call(path)
  next if non_behavioral.include?(file)

  File.read(path).scan(/\b[A-Z][A-Za-z]+\/[A-Z][A-Za-z0-9]+/) do |cop|
    next unless STATUS.heuristic?(cop)

    key = [cop, file]
    seen[key] = true
    problems << "unclassified heuristic cop #{cop} in #{file}" unless allowed[key]
  end
end

(allowed.keys - seen.keys).each do |cop, file|
  problems << "stale heuristic classification #{cop} in #{file}"
end

unless problems.empty?
  warn "Cop test classification errors:"
  problems.sort.each { |problem| warn "  - #{problem}" }
  exit 1
end

puts "Test cop classifications passed: #{seen.length} explicit heuristic regression references."
