# frozen_string_literal: true

require "json"

ROOT = File.expand_path("..", __dir__)
FIXTURE_ROOT = File.join(ROOT, "spec", "fixtures")
COP_ROOT = File.join(FIXTURE_ROOT, "cops")
ALLOWED_ROOT_FILES = %w[README.md unit_manifest.json].freeze
ALLOWED_UNIT_FILES = %w[cases.jsonl configs.json].freeze

manifest = JSON.parse(File.read(File.join(FIXTURE_ROOT, "unit_manifest.json")))
problems = []
files = Dir[File.join(FIXTURE_ROOT, "**", "*")].select { |path| File.file?(path) }

files.each do |path|
  relative = path.delete_prefix("#{FIXTURE_ROOT}/")
  next if ALLOWED_ROOT_FILES.include?(relative)

  parts = relative.split(File::SEPARATOR)
  valid = parts.length == 5 && parts[0] == "cops" && parts[3] == "unit" &&
          ALLOWED_UNIT_FILES.include?(parts[4])
  problems << "non-unit fixture file: #{relative}" unless valid
end

manifest.fetch("cops").each do |cop, entry|
  department, name = cop.split("/", 2)
  expected = {
    "cases" => "cops/#{department}/#{name}/unit/cases.jsonl",
    "configs" => "cops/#{department}/#{name}/unit/configs.json"
  }
  expected.each do |key, path|
    problems << "#{cop} has noncanonical #{key}: #{entry.fetch(key)}" unless entry.fetch(key) == path
    problems << "#{cop} is missing #{path}" unless File.file?(File.join(FIXTURE_ROOT, path))
  end
end

owned_cops = files.filter_map do |path|
  relative = path.delete_prefix("#{COP_ROOT}/")
  parts = relative.split(File::SEPARATOR)
  parts.first(2).join("/") if parts.length == 4 && parts[2] == "unit"
end.uniq
unmanifested = owned_cops - manifest.fetch("cops").keys
problems << "unmanifested unit cops: #{unmanifested.sort.join(', ')}" unless unmanifested.empty?

abort "fixture ownership errors:\n  - #{problems.join("\n  - ")}" unless problems.empty?

puts "fixture ownership is valid: #{manifest.fetch('controlled_cases')} unit cases across #{owned_cops.length} cops"
