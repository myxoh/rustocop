# frozen_string_literal: true

require "json"
require "pathname"

ROOT = Pathname(__dir__).join("..").expand_path
EQUIVALENTS = ROOT.join("compatibility", "dependencies", "rubocop_dependency_rust_equivalents.json")
TRANSLATIONS = ROOT.join("crates", "rustocop", "rubocop-translation.json")

PACKAGE_PREFIX = {
  "rubocop" => "rubocop-1.87.0",
  "rubocop-ast" => "rubocop-ast-1.49.1"
}.freeze

MANUAL_FINDINGS = {
  "rubocop-1.87.0/lib/rubocop/cop/force.rb" => <<~TEXT.strip,
    Source-level review found material differences despite the nominal 2/2 test count: Rust `Force::initialize` leaves `name` empty instead of deriving `Force` from the runtime class, its registry is instance-owned rather than Ruby's inherited class registry, `investigate` is not independently represented, and hook failures model only string errors rather than arbitrary StandardError instances. The files are not functionally identical.
  TEXT
  "rubocop-1.87.0/lib/rubocop/cop/variable_force.rb" => <<~TEXT.strip,
    `VariableForce` inherits and relies on `Force`, whose initialization, registry, hook-dispatch, and error semantics are not functionally identical in Rust. A file depending on that non-equivalent superclass cannot be certified as an implementation-exact equivalent; the existing contracts do not repair or independently prove the inherited behavior.
  TEXT
  "rubocop-ast-1.49.1/lib/rubocop/ast/node/forward_args_node.rb" => <<~TEXT.strip,
    Rust reproduces the local `to_a` result for a `forward_args` node, but folds it into a generic NodeRef method inside a shared specialized-node file. It does not reproduce the upstream ForwardArgsNode class, its Node inheritance and CollectionNode inclusion, or the builder/version-dependent object topology exercised by the upstream spec. This is a useful behavioral port, not a functionally identical file implementation.
  TEXT
  "rubocop-ast-1.49.1/lib/rubocop/ast/node_pattern/compiler/sequence_subcompiler.rb" => <<~TEXT.strip
    Independent source extraction finds the private upstream `compile` API, but the compatibility manifest omits it and maps every other compiler operation to the single Rust `new` entrypoint. Rust deliberately replaces Ruby source generation with a native interpreter. The API and implementation are therefore not functionally identical.
  TEXT
}.freeze

translations = JSON.parse(TRANSLATIONS.read).fetch("components").to_h do |component|
  prefix = PACKAGE_PREFIX.fetch(component.fetch("package"))
  ["#{prefix}/#{component.fetch('source')}", component]
end

document = JSON.parse(EQUIVALENTS.read)
rows = document.fetch("rows").map do |row|
  unless row.fetch("review_status", "complete") == "candidate_pending" || row.key?("review_evidence")
    normalized = row.key?("review_status") ? row : row.merge("review_status" => "complete")
    if normalized.fetch("rust_equivalent", "N/A") == "N/A" && !normalized.key?("classification_basis")
      next normalized.merge(
        "classification_basis" => "No compatibility-layer source provenance, translation-manifest entry, or existing Rust file candidate."
      )
    end
    next normalized
  end

  path = row.fetch("path")
  component = translations.fetch(path)
  candidate_paths = if row.key?("candidates")
                      row.fetch("candidates").flat_map { |candidate| candidate.fetch("paths") }.uniq
                    else
                      row.dig("review_evidence", "candidate_paths") || []
                    end
  specs = component.fetch("specs", [])
  unresolved = component.dig("api_coverage", "unresolved") || []

  basis = MANUAL_FINDINGS[path]
  basis ||= if component.fetch("status") == "native"
              "The compatibility manifest itself classifies this as a native Rust implementation. The candidate intentionally uses Rust/Prism-native representation or control flow rather than the upstream file's implementation topology; no assertion-preserving whole-file identity proof exists."
            elsif unresolved.any?
              "The candidate's own compatibility record leaves upstream API members unresolved (#{unresolved.join(', ')}), so API identity is incomplete and the files cannot be equivalent."
            elsif specs.empty?
              "A provenance-linked Rust candidate exists, but there is no registered assertion-preserving port of the upstream file's specs. Source/API name correspondence alone cannot demonstrate functional identity, so the candidate remains uncertain and is N/A under the audit standard."
            else
              upstream_examples = specs.sum { |spec| spec.fetch("upstream_examples", 0).to_i }
              rust_tests = specs.sum { |spec| spec.fetch("rust_tests", 0).to_i }
              inferred = specs.any? do |spec|
                spec.fetch("example_contracts", []).any? do |contract|
                  contract.fetch("mapping_basis", "") == "semantic_terms"
                end
              end
              evidence_description = if inferred
                                       "The registered spec links use semantic-term inference"
                                     else
                                       "The registered compatibility evidence"
                                     end
              "#{evidence_description} (#{upstream_examples} upstream examples represented by #{rust_tests} Rust tests), not a reviewed one-to-one translation of assertions and control-flow branches. It does not establish whole-file API and behavioral identity, so the candidate remains uncertain and is N/A under the audit standard."
            end

  row.merge(
    "review_status" => "complete",
    "rust_equivalent" => "N/A",
    "classification_basis" => basis,
    "review_evidence" => {
      "candidate_paths" => candidate_paths,
      "manifest_status" => component.fetch("status"),
      "upstream_api_total" => component.dig("api_coverage", "total"),
      "unresolved_upstream_api" => unresolved,
      "registered_upstream_examples" => specs.sum { |spec| spec.fetch("upstream_examples", 0).to_i },
      "registered_rust_tests" => specs.sum { |spec| spec.fetch("rust_tests", 0).to_i },
      "assertion_preserving_review" => false
    }
  ).tap { |completed| completed.delete("candidates") }
end

document["rows"] = rows
document["review_summary"] = {
  "source_files" => rows.length,
  "definite_no_candidate" => rows.count do |row|
    row.fetch("classification_basis", "").start_with?("No compatibility-layer source provenance")
  end,
  "candidates_reviewed" => rows.count { |row| row.key?("review_evidence") },
  "candidate_review_categories" => {
    "source_level_findings" => rows.count { |row| MANUAL_FINDINGS.key?(row.fetch("path")) },
    "native_topology_without_identity_proof" => rows.count do |row|
      row.dig("review_evidence", "manifest_status") == "native" && !MANUAL_FINDINGS.key?(row.fetch("path"))
    end,
    "no_assertion_preserving_specs" => rows.count do |row|
      row.key?("review_evidence") && row.dig("review_evidence", "registered_upstream_examples").to_i.zero? &&
        row.dig("review_evidence", "manifest_status") != "native" && !MANUAL_FINDINGS.key?(row.fetch("path"))
    end,
    "non_assertion_preserving_contract_mapping" => rows.count do |row|
      row.dig("review_evidence", "registered_upstream_examples").to_i.positive? &&
        row.dig("review_evidence", "manifest_status") != "native" && !MANUAL_FINDINGS.key?(row.fetch("path"))
    end
  },
  "exact_equivalents" => rows.count do |row|
    !%w[N/A not_necessary].include?(row.fetch("rust_equivalent"))
  end,
  "standard" => "Uncertain, partial, analogous, native-topology, or non-assertion-preserving candidates are N/A."
}

EQUIVALENTS.binwrite(JSON.pretty_generate(document) << "\n")
puts "wrote #{EQUIVALENTS.relative_path_from(ROOT)}"
puts JSON.pretty_generate(document.fetch("review_summary"))
