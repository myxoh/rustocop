# frozen_string_literal: true

require "json"
require "time"
require "tmpdir"

RSpec.describe "compatibility evidence report" do
  fixture_snapshot = File.join(ROOT, "spec", "compatibility_evidence", "fixtures.json")
  project_snapshot = File.join(ROOT, "spec", "compatibility_evidence", "projects.json")
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
