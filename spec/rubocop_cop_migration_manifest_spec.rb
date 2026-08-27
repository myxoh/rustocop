# frozen_string_literal: true

require "digest"
require "json"
require "time"
require "rustocop/compatibility_status"
require "rustocop/project_corpus"

RSpec.describe "RuboCop cop source-shape migration manifest" do
  manifest_path = File.join(ROOT, "crates", "rustocop", "rubocop-cop-migrations.json")
  fixture_path = File.join(ROOT, "spec", "compatibility_evidence", "fixtures.json")
  project_path = File.join(ROOT, "spec", "compatibility_evidence", "projects.json")
  crate_root = File.join(ROOT, "crates", "rustocop")

  let(:manifest) { JSON.parse(File.read(manifest_path)) }
  let(:fixture_results) { JSON.parse(File.read(fixture_path)).fetch("results") }
  let(:project_results) { JSON.parse(File.read(project_path)).fetch("results") }

  it "pins every audited cop to its upstream source and current evidence" do
    gem "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}"
    upstream_root = Gem::Specification.find_by_name("rubocop").full_gem_path
    active = Rustocop::CompatibilityStatus.load(root: ROOT).built_in_cops
    rows = manifest.fetch("cops")

    expect(manifest.fetch("format_version")).to eq(2)
    expect(manifest.fetch("rubocop_version")).to eq(Rustocop::ProjectCorpus::RUBOCOP_VERSION)
    expect(manifest.fetch("rubocop_commit")).to eq(Rustocop::ProjectCorpus::RUBOCOP_COMMIT)
    expect(manifest.fetch("target_cops")).to eq(active.length)
    expect(manifest.fetch("inventory_cops")).to eq(rows.length)
    expect(rows.length).to eq(active.length)
    expect(manifest.fetch("audited_cops")).to eq(
      rows.count { |row| row.fetch("structural_status") != "unaudited" }
    )
    expect(manifest.fetch("migrated_cops")).to eq(rows.count { |row| row.fetch("migration_status") == "migrated" })
    expect(rows.map { |row| row.fetch("cop") }).to contain_exactly(*rows.map { |row| row.fetch("cop") }.uniq)
    expect(rows.map { |row| row.fetch("cop") } - active).to be_empty

    rows.each do |row|
      upstream = File.join(upstream_root, row.fetch("upstream_source"))
      expect(File).to exist(upstream), row.fetch("cop")
      expect(Digest::SHA256.file(upstream).hexdigest).to eq(row.fetch("upstream_sha256")), row.fetch("cop")
      implementations = row.fetch("implementations").map { |path| File.join(crate_root, path) }
      expect(implementations).not_to be_empty
      expect(implementations.length).to eq(1), "#{row.fetch('cop')} must have one canonical implementation"
      expect(implementations).to all(satisfy { |path| File.file?(path) })
      expect(implementations.any? { |path| File.read(path).include?(row.fetch("cop")) }).to be(true)
      expect(row.fetch("structural_gaps")).not_to be_empty unless row.fetch("migration_status") == "migrated"
      expect(row.fetch("fixtures")).to eq(fixture_results.fetch(row.fetch("cop")))
      expect(row.fetch("projects")).to eq(project_results.fetch(row.fetch("cop")))

      next if row.fetch("structural_status") == "unaudited"

      expect(row.fetch("upstream_callbacks")).not_to be_empty
      expect(row.fetch("rust_callbacks")).not_to be_empty
      expect(row.fetch("similarity_score")).to be_between(1, 5)
      if row.fetch("migration_status") == "migrated"
        expect(row.fetch("documented_adaptations", [])).not_to be_empty
      end
    end
  end

  it "uses a consistent score classification and ISO 8601 timestamp" do
    statuses = {
      1 => %w[divergent],
      2 => %w[divergent],
      3 => %w[conceptually_aligned source_shaped_with_source_adapter],
      4 => %w[source_shaped_with_parser_adaptation source_shaped_with_prism_adaptation],
      5 => %w[near_source_shaped]
    }
    manifest.fetch("cops").reject { |row| row.fetch("structural_status") == "unaudited" }.each do |row|
      expect(statuses.fetch(row.fetch("similarity_score"))).to include(row.fetch("structural_status"))
    end

    updated_at = manifest.fetch("updated_at")
    expect { Time.iso8601(updated_at) }.not_to raise_error
    expect(updated_at).to match(/T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})\z/)
  end
end
