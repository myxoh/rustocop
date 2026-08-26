# frozen_string_literal: true

require "json"
require "digest"
require "time"
require "tmpdir"
require "rustocop/project_corpus"

RSpec.describe "compatibility evidence report" do
  fixture_snapshot = File.join(ROOT, "spec", "compatibility_evidence", "fixtures.json")
  project_snapshot = File.join(ROOT, "spec", "compatibility_evidence", "projects.json")
  consumer_manifest = File.join(ROOT, "crates", "rustocop", "rubocop-consumers.json")
  adoption_report = File.join(ROOT, "docs", "rubocop-compatibility-adoption.md")
  report = File.join(ROOT, "docs", "compatibility.md")

  it "keeps complete compact evidence snapshots" do
    fixtures = JSON.parse(File.read(fixture_snapshot))
    projects = JSON.parse(File.read(project_snapshot))

    expect(fixtures.fetch("kind")).to eq("fixture_compatibility")
    expect(projects.fetch("kind")).to eq("project_compatibility")
    expect(fixtures.fetch("results").length).to eq(606)
    expect(projects.fetch("results").length).to eq(606)
    expect(fixtures.fetch("results").keys).to match_array(projects.fetch("results").keys)

    [fixtures, projects].each do |snapshot|
      updated_at = snapshot.fetch("updated_at")
      expect { Time.iso8601(updated_at) }.not_to raise_error
      expect(updated_at).to match(/T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})\z/)
      commit = snapshot["rust_commit"]
      source = snapshot["cop_source_sha256"]
      expect(commit&.match?(/\A[0-9a-f]{40}\z/) || source&.match?(/\A[0-9a-f]{64}\z/)).to be(true)
    end
  end

  it "tracks compatibility-layer consumers by project" do
    projects = JSON.parse(File.read(project_snapshot))
    manifest = JSON.parse(File.read(consumer_manifest))
    adoption = projects.fetch("compatibility_layer")
    consumer_cops = manifest.fetch("consumers").map { |consumer| consumer.fetch("cop") }

    expect(projects.fetch("version")).to eq(2)
    expect(adoption.fetch("consumer_manifest_sha256")).to eq(Digest::SHA256.file(consumer_manifest).hexdigest)
    expect(adoption.fetch("consumer_cops")).to eq(consumer_cops)
    expect(adoption.fetch("projects").keys).to match_array(
      Rustocop::ProjectCorpus::PROJECTS.map { |project| project.fetch("name") }
    )

    adoption.fetch("projects").each_value do |project|
      expect(project.fetch("by_cop").keys).to eq(consumer_cops)
      expected_exercised = project.fetch("by_cop").filter_map do |cop, row|
        cop if row.fetch("rubocop").positive?
      end
      expect(project.fetch("exercised_cops")).to eq(expected_exercised)
    end

    updated_at = manifest.fetch("updated_at")
    expect { Time.iso8601(updated_at) }.not_to raise_error
    expect(updated_at).to match(/T\d{2}:\d{2}:\d{2}(?:\.\d+)?(?:Z|[+-]\d{2}:\d{2})\z/)
    expect(File.read(adoption_report)).to include("Projects exercising at least one consumer")
  end

  it "keeps the generated table current" do
    _stdout, stderr, status = Open3.capture3(
      RbConfig.ruby,
      File.join(ROOT, "script", "generate_compatibility_report.rb"),
      "--check"
    )

    expect(stderr).to eq("")
    expect(status).to be_success
    rows = File.readlines(report).grep(/^\| `[A-Za-z]+\/[A-Za-z0-9]+` \|/)
    expect(rows.length).to eq(606)
  end

  it "rejects date-only updated_at evidence" do
    Dir.mktmpdir("compatibility-report") do |directory|
      invalid_fixtures = JSON.parse(File.read(fixture_snapshot)).merge("updated_at" => "2026-08-21")
      invalid_path = File.join(directory, "fixtures.json")
      File.write(invalid_path, JSON.pretty_generate(invalid_fixtures))

      _stdout, stderr, status = Open3.capture3(
        RbConfig.ruby,
        File.join(ROOT, "script", "generate_compatibility_report.rb"),
        "--fixture-snapshot", invalid_path,
        "--project-snapshot", project_snapshot,
        "--output", File.join(directory, "compatibility.md")
      )

      expect(status).not_to be_success
      expect(stderr).to include("fixture snapshot updated_at must be ISO 8601")
    end
  end
end
