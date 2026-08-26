# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "pathname"
require "time"
require_relative "../lib/rustocop/artifact_store"
require_relative "../lib/rustocop/compatibility_status"
require_relative "../lib/rustocop/cop_implementation_inventory"
require_relative "../lib/rustocop/project_corpus"
require_relative "../lib/rustocop/repository_layout"

gem "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}"
require "rubocop"

layout = Rustocop::RepositoryLayout.default
root = layout.root
crate_root = layout.path("crates", "rustocop")
manifest_path = File.join(crate_root, "rubocop-cop-migrations.json")
options = {manifest: manifest_path, check: false}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/generate_cop_migration_inventory.rb [--check]"
  parser.on("--manifest PATH") { |path| options[:manifest] = File.expand_path(path) }
  parser.on("--check") { options[:check] = true }
end.parse!

manifest = if File.file?(options[:manifest])
             Rustocop::ArtifactStore.read_json(options[:manifest], label: "cop migration manifest")
           else
             {}
           end
prior = manifest.fetch("cops", []).to_h { |row| [row.fetch("cop"), row] }
fixtures = Rustocop::ArtifactStore.read_json(
  layout.compatibility_evidence("fixtures.json"), label: "fixture evidence"
).fetch("results")
projects = Rustocop::ArtifactStore.read_json(
  layout.compatibility_evidence("projects.json"), label: "project evidence"
).fetch("results")
active = Rustocop::CompatibilityStatus.load(root: root).built_in_cops.sort
rust_sources = Rustocop::CopImplementationInventory.sources(root: root)
rubocop_root = Gem::Specification.find_by_name("rubocop").full_gem_path
registry = RuboCop::Cop::Registry.global

rows = active.map do |cop|
  cop_class = registry.find { |candidate| candidate.cop_name == cop }
  abort "RuboCop class not found for #{cop}" unless cop_class
  source, = Object.const_source_location(cop_class.name)
  abort "RuboCop source not found for #{cop}" unless source&.start_with?(rubocop_root)
  upstream_source = Pathname(source).relative_path_from(Pathname(rubocop_root)).to_s
  source_text = File.read(source)
  callbacks = cop_class.instance_methods(false).grep(/^on_/).map(&:to_s).sort
  mixins = source_text.scan(/^\s+(?:include|extend)\s+([A-Z][A-Za-z0-9_:]*)\s*$/).flatten.uniq.sort
  implementations = Rustocop::CopImplementationInventory.registration_paths(
    cop, sources: rust_sources
  ).map { |path| Pathname(path).relative_path_from(Pathname(crate_root)).to_s }
  abort "Rust implementation not found for #{cop}" if implementations.empty?

  mechanical = {
    "cop" => cop,
    "upstream_source" => upstream_source,
    "upstream_sha256" => Digest::SHA256.file(source).hexdigest,
    "implementations" => implementations,
    "upstream_callbacks" => callbacks,
    "upstream_mixins" => mixins,
    "fixtures" => fixtures.fetch(cop),
    "projects" => projects.fetch(cop)
  }
  reviewed = prior.fetch(cop, {}).slice(
    "related_inactive_implementations", "rust_callbacks", "compatibility_components",
    "dsl_features", "similarity_score", "structural_status", "migration_status",
    "structural_gaps", "documented_adaptations"
  )
  mechanical.merge(
    {
      "rust_callbacks" => [],
      "compatibility_components" => [],
      "dsl_features" => [],
      "similarity_score" => nil,
      "structural_status" => "unaudited",
      "migration_status" => "unaudited",
      "structural_gaps" => ["Structural review pending."],
      "documented_adaptations" => []
    }.merge(reviewed)
  )
end

generated = {
  "format_version" => 2,
  "updated_at" => manifest.fetch("updated_at", Time.now.iso8601),
  "rubocop_version" => Rustocop::ProjectCorpus::RUBOCOP_VERSION,
  "rubocop_commit" => Rustocop::ProjectCorpus::RUBOCOP_COMMIT,
  "sampling" => manifest["sampling"],
  "rubric" => manifest.fetch("rubric"),
  "target_cops" => active.length,
  "inventory_cops" => rows.length,
  "audited_cops" => rows.count { |row| row.fetch("structural_status") != "unaudited" },
  "migrated_cops" => rows.count { |row| row.fetch("migration_status") == "migrated" },
  "cops" => rows
}.compact

content = Rustocop::ArtifactStore.serialize_json(generated, trailing_newline: true)
if options[:check]
  abort "cop migration inventory is stale" unless File.read(options[:manifest]) == content
else
  generated["updated_at"] = Time.now.iso8601
  Rustocop::ArtifactStore.atomic_write(
    options[:manifest], Rustocop::ArtifactStore.serialize_json(generated, trailing_newline: true)
  )
end
puts "Cop migration inventory: #{generated.fetch('audited_cops')}/#{generated.fetch('inventory_cops')} audited, " \
     "#{generated.fetch('migrated_cops')} migrated"
