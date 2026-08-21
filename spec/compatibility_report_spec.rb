# frozen_string_literal: true

require "json"

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
end
