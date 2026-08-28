# frozen_string_literal: true

require "json"
require "pathname"
require "set"

ROOT = Pathname(__dir__).join("..").expand_path
GRAPH = ROOT.join("compatibility", "dependencies", "rubocop_dependency_graph.json")
EQUIVALENTS = ROOT.join("compatibility", "dependencies", "rubocop_dependency_rust_equivalents.json")
TRANSLATIONS = ROOT.join("crates", "rustocop", "rubocop-translation.json")

PACKAGE_PREFIX = {
  "rubocop" => "rubocop-1.87.0",
  "rubocop-ast" => "rubocop-ast-1.49.1"
}.freeze

def replacement_paths(document)
  document.fetch("rows").filter_map do |row|
    value = row["rust_equivalent"]
    next unless value.is_a?(Hash)
    next if value.dig("verification", "api_identity_evidence").to_s.strip.empty?
    next if value.dig("verification", "behavior_identity_evidence").to_s.strip.empty?

    row.fetch("path")
  end.to_set
end

def compatibility_manifest_candidates
  JSON.parse(TRANSLATIONS.read).fetch("components").filter_map do |component|
    next unless component["rust"]
    next unless %w[translated native].include?(component["status"])

    prefix = PACKAGE_PREFIX.fetch(component.fetch("package"))
    "#{prefix}/#{component.fetch('source')}"
  end.to_set
end

def not_necessary_closure(graph, replacements)
  nodes = graph.fetch("nodes")
  component_by_path = nodes.to_h do |node|
    [node.fetch("path"), node.fetch("strongly_connected_component")]
  end
  members = nodes.map { |node| node.fetch("path") }.group_by { |path| component_by_path.fetch(path) }
  consumers = Hash.new { |hash, key| hash[key] = Set.new }
  graph.fetch("edges").each do |edge|
    consumer = component_by_path.fetch(edge.fetch("source"))
    dependency = component_by_path.fetch(edge.fetch("target"))
    consumers[dependency] << consumer unless consumer == dependency
  end

  replacement_components = replacements.filter_map { |path| component_by_path[path] }.to_set
  unnecessary_components = Set.new
  loop do
    additions = members.keys.reject do |component|
      replacement_components.include?(component) || unnecessary_components.include?(component)
    end.select do |component|
      direct_consumers = consumers[component]
      direct_consumers.any? && direct_consumers.all? do |consumer|
        replacement_components.include?(consumer) || unnecessary_components.include?(consumer)
      end
    end
    break if additions.empty?

    unnecessary_components.merge(additions)
  end

  unnecessary_components.flat_map { |component| members.fetch(component) }.to_set
end

graph = JSON.parse(GRAPH.read)
document = JSON.parse(EQUIVALENTS.read)
trusted_replacements = replacement_paths(document)
not_necessary = not_necessary_closure(graph, trusted_replacements)
untrusted_candidates = not_necessary_closure(graph, compatibility_manifest_candidates)

consumers = Hash.new { |hash, key| hash[key] = Set.new }
graph.fetch("edges").each do |edge|
  consumers[edge.fetch("target")] << edge.fetch("source")
end

document["rows"] = document.fetch("rows").map do |row|
  path = row.fetch("path")
  if not_necessary.include?(path)
    row.merge(
      "rust_equivalent" => "not_necessary",
      "classification_basis" => "Every known consumer is an exact Rust replacement or is itself unnecessary by the same closed dependency proof.",
      "necessity_evidence" => {
        "direct_consumers" => consumers[path].to_a.sort,
        "trusted_replacement_frontier" => trusted_replacements.to_a.sort
      }
    )
  elsif row["rust_equivalent"] == "not_necessary"
    row.except("necessity_evidence").merge(
      "rust_equivalent" => "N/A",
      "classification_basis" => "No currently trusted replacement frontier proves this file unnecessary."
    )
  else
    row
  end
end

document["necessity_review"] = {
  "status" => "complete",
  "files_reviewed" => document.fetch("rows").length,
  "criterion" => "A file must have at least one known consumer, and every reverse dependency path must terminate at a whole-file Rust equivalent with explicit API-identity and behavior-identity evidence.",
  "trusted_replacement_files" => trusted_replacements.to_a.sort,
  "not_necessary_files" => not_necessary.to_a.sort,
  "untrusted_manifest_candidates_not_marked" => untrusted_candidates.to_a.sort,
  "untrusted_candidate_reason" => "The legacy compatibility manifest supplies candidate discovery but no candidate passed the strict whole-file equivalence audit; it cannot serve as a trusted replacement frontier."
}

EQUIVALENTS.binwrite(JSON.pretty_generate(document) << "\n")
puts "wrote #{EQUIVALENTS.relative_path_from(ROOT)}"
puts JSON.pretty_generate(document.fetch("necessity_review"))
