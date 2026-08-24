# frozen_string_literal: true

require_relative "../lib/rustocop/compatibility_status"
require "yaml"

ROOT = File.expand_path("..", __dir__)
FIXTURE_ROOT = File.join(ROOT, "spec", "fixtures")
COP_ROOT = File.join(FIXTURE_ROOT, "cops")
KNOWN_KINDS = %w[configuration end_to_end hardening native prism project unit].freeze

native_source = File.read(File.join(ROOT, "crates", "rustocop", "src", "engine", "fixture_tests.rs"))
native_registrations = native_source.scan(
  /fixture_test!\(\s*\w+,\s*"([^"]+)",\s*"[^"]+",\s*"([^"]+)"/m
)
native_cops = native_registrations.flat_map { |_path, selection| selection.split(",") }
known_cops = (Rustocop::CompatibilityStatus.load(root: ROOT).built_in_cops + native_cops).to_h { |cop| [cop, true] }
problems = []
owned_files = Dir[File.join(COP_ROOT, "**", "*")].select { |path| File.file?(path) }

owned_files.each do |path|
  department, name, kind = path.delete_prefix("#{COP_ROOT}/").split(File::SEPARATOR, 4)
  cop = [department, name].join("/")
  problems << "unknown cop path: #{path}" unless known_cops[cop]
  problems << "unknown fixture kind: #{path}" unless KNOWN_KINDS.include?(kind)
end

Dir[File.join(COP_ROOT, "*", "*", "end_to_end", "*", "rubocop.yml")].each do |config_path|
  relative = config_path.delete_prefix("#{COP_ROOT}/")
  department, name = relative.split(File::SEPARATOR, 3)
  owner = [department, name].join("/")
  configured_cops = YAML.safe_load_file(config_path).keys.grep(%r{\A[A-Z][A-Za-z]+/[A-Za-z0-9]+\z})
  problems << "end-to-end fixture #{relative} configures #{configured_cops.inspect}, expected #{owner}" unless configured_cops == [owner]
end

indexes = {
  "cop_project_cases.tsv" => [[1], "project"],
  "cop_project_mismatches.tsv" => [[1], "project"],
  "cop_configuration_cases.tsv" => [[1, 2], "configuration"]
}
indexed_paths = Hash.new { |hash, key| hash[key] = [] }
indexes.each do |name, (path_columns, kind)|
  File.readlines(File.join(FIXTURE_ROOT, name), chomp: true).drop(1).each do |line|
    columns = line.split("\t")
    cop = columns.fetch(0)
    path_columns.each do |path_column|
      relative_path = columns.fetch(path_column)
      path = File.join(FIXTURE_ROOT, relative_path)
      expected_prefix = File.join(COP_ROOT, *cop.split("/"), kind)
      problems << "missing indexed fixture: #{relative_path}" unless File.file?(path)
      problems << "fixture indexed under the wrong cop: #{relative_path} (#{cop})" unless path.start_with?("#{expected_prefix}/")
      indexed_paths[kind] << path
    end
  end
end

%w[configuration project].each do |kind|
  actual = owned_files.select { |path| path.include?("/#{kind}/") }.sort
  indexed = indexed_paths.fetch(kind).uniq.sort
  (actual - indexed).each { |path| problems << "unindexed #{kind} fixture: #{path.delete_prefix("#{ROOT}/")}" }
end

native_registrations.each do |relative_path, selection|
  directory = File.join(FIXTURE_ROOT, relative_path)
  cops = selection.split(",")
  problems << "missing registered native fixture: #{relative_path}" unless File.directory?(directory)
  next unless relative_path.start_with?("cops/")

  owner = relative_path.split(File::SEPARATOR).slice(1, 2).join("/")
  problems << "native fixture #{relative_path} is registered for #{selection}" unless cops == [owner]
end
registered_native = native_registrations.map { |relative_path, _selection| File.join(FIXTURE_ROOT, relative_path) }.sort
actual_native = Dir[File.join(COP_ROOT, "*", "*", "native", "*")] +
                Dir[File.join(FIXTURE_ROOT, "shared", "native", "*")] +
                Dir[File.join(FIXTURE_ROOT, "projects", "*", "native", "*")]
(actual_native.select { |path| File.directory?(path) }.sort - registered_native).each do |path|
  problems << "unregistered native fixture: #{path.delete_prefix("#{FIXTURE_ROOT}/")}"
end

abort "fixture ownership errors:\n  - #{problems.join("\n  - ")}" unless problems.empty?

cops = owned_files.map do |path|
  path.delete_prefix("#{COP_ROOT}/").split(File::SEPARATOR, 3).first(2).join("/")
end.uniq
puts "fixture ownership is valid: #{owned_files.length} files across #{cops.length} cops"
