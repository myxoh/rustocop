# frozen_string_literal: true

require "yaml"

ROOT = File.expand_path("..", __dir__)
OUTPUT = File.join(ROOT, "spec/source_cop_inventory.yml")
CHECK = ARGV.delete("--check")
abort "usage: generate_source_cop_inventory.rb [--check]" unless ARGV.empty?

allowed_reviews = %w[lexical syntax_aware_migrate temporary_text unreviewed].freeze
existing = File.file?(OUTPUT) ? YAML.safe_load_file(OUTPUT) : { "entries" => [] }
reviews = existing.fetch("entries", []).to_h do |entry|
  [[entry.fetch("pipeline"), entry.fetch("cop")], entry.fetch("review")]
end

entries = []
prism_root = File.join(ROOT, "crates/rustocop/src/cops/prism")
Dir[File.join(prism_root, "**/*.rs")].sort.each do |path|
  source = File.read(path)
  cops = source.scan(/=>\s*"([A-Za-z]+\/[A-Za-z0-9]+)"\s*=>\s*source\(/).flatten
  source.scan(/declare_source_cops!\s*\{(.*?)^\}/m).each do |body|
    cops.concat(body.first.scan(/=>\s*"([A-Za-z]+\/[A-Za-z0-9]+)"\s*=>/).flatten)
  end
  cops.uniq.sort.each do |cop|
    key = ["prism_source", cop]
    entries << {
      "cop" => cop,
      "pipeline" => key.first,
      "source" => path.delete_prefix("#{ROOT}/"),
      "review" => reviews.fetch(key, "unreviewed")
    }
  end
end

legacy_path = File.join(ROOT, "crates/rustocop/src/cops/text/mod.rs")
legacy_source = File.read(legacy_path)
legacy_body = legacy_source.match(/LEGACY_COP_NAMES:.*?=\s*&\[(.*?)\];/m)&.captures&.first
abort "could not find LEGACY_COP_NAMES" unless legacy_body
legacy_body.scan(/"([A-Za-z]+\/[A-Za-z0-9]+)"/).flatten.sort.each do |cop|
  key = ["legacy_text", cop]
  entries << {
    "cop" => cop,
    "pipeline" => key.first,
    "source" => legacy_path.delete_prefix("#{ROOT}/"),
    "review" => reviews.fetch(key, "temporary_text")
  }
end

entries.sort_by! { |entry| [entry.fetch("pipeline"), entry.fetch("cop")] }
invalid = entries.reject { |entry| allowed_reviews.include?(entry.fetch("review")) }
abort "invalid review values: #{invalid.map { |entry| entry.fetch("review") }.uniq.join(", ")}" unless invalid.empty?

document = {
  "version" => 1,
  "review_values" => allowed_reviews,
  "entries" => entries
}
rendered = YAML.dump(document)

if CHECK
  abort "source cop inventory is stale; run script/generate_source_cop_inventory.rb" unless File.file?(OUTPUT)
  abort "source cop inventory is stale; run script/generate_source_cop_inventory.rb" unless File.read(OUTPUT) == rendered

  counts = entries.map { |entry| entry.fetch("review") }.tally
  puts "Source cop inventory passed: #{entries.length} entries (#{counts.sort.to_h})."
  exit
end

File.write(OUTPUT, rendered)
puts "wrote #{OUTPUT}: #{entries.length} source/text cop entries"
