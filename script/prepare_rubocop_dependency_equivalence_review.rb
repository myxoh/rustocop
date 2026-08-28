# frozen_string_literal: true

require "digest"
require "json"
require "pathname"

ROOT = Pathname(__dir__).join("..").expand_path
ANNOTATIONS = ROOT.join("compatibility", "dependencies", "rubocop_dependency_annotations.json")
EQUIVALENTS = ROOT.join("compatibility", "dependencies", "rubocop_dependency_rust_equivalents.json")
TRANSLATIONS = ROOT.join("crates", "rustocop", "rubocop-translation.json")

PACKAGE_PREFIX = {
  "rubocop" => "rubocop-1.87.0",
  "rubocop-ast" => "rubocop-ast-1.49.1"
}.freeze

annotations = JSON.parse(ANNOTATIONS.read)
existing = JSON.parse(EQUIVALENTS.read)
completed = existing.fetch("rows").select do |row|
  row.fetch("review_status", "complete") == "complete"
end.to_h { |row| [row.fetch("path"), row] }

candidates = Hash.new { |hash, key| hash[key] = [] }
JSON.parse(TRANSLATIONS.read).fetch("components").each do |component|
  package = PACKAGE_PREFIX.fetch(component.fetch("package"))
  logical_path = "#{package}/#{component.fetch('source')}"
  rust = component["rust"]
  next unless rust

  rust_path = "crates/rustocop/#{rust}"
  next unless ROOT.join(rust_path).file?

  candidates[logical_path] << {
    "paths" => [rust_path],
    "discovery" => "version-pinned compatibility manifest",
    "claimed_status" => component.fetch("status"),
    "upstream_sha256" => component.fetch("source_sha256"),
    "claimed_api_total" => component.dig("api_coverage", "total"),
    "claimed_api_unresolved" => component.dig("api_coverage", "unresolved") || [],
    "claimed_specs" => component.fetch("specs", []).map { |spec| spec.fetch("source") }
  }
end

rows = annotations.fetch("rows").map do |annotation|
  path = annotation.fetch("path")
  next completed.fetch(path) if completed.key?(path)

  path_candidates = candidates.fetch(path, []).uniq
  if path_candidates.empty?
    {
      "path" => path,
      "md5" => annotation.fetch("md5"),
      "review_status" => "complete",
      "rust_equivalent" => "N/A",
      "classification_basis" => "No compatibility-layer source provenance, translation-manifest entry, or existing Rust file candidate."
    }
  else
    {
      "path" => path,
      "md5" => annotation.fetch("md5"),
      "review_status" => "candidate_pending",
      "candidates" => path_candidates
    }
  end
end

document = {
  "schema_version" => 1,
  "equivalence_standard" => "Record a Rust path when a realistic Rust consumer can express materially the same upstream library logic and obtain RuboCop-compatible behavior for existing and reasonably foreseeable use cases. Accept necessary language adaptations that preserve consumer capability; do not require incidental Ruby runtime affordances unless a credible consumer depends on them. Use N/A for absent or partial capabilities, material semantic gaps, or cases without a pragmatic faithful interface.",
  "rows" => rows
}
EQUIVALENTS.binwrite(JSON.pretty_generate(document) << "\n")

pending = rows.count { |row| row["review_status"] == "candidate_pending" }
complete = rows.length - pending
puts "wrote #{EQUIVALENTS.relative_path_from(ROOT)}"
puts "definite classifications: #{complete}; candidates requiring careful review: #{pending}"
