# frozen_string_literal: true

require "spec_helper"
require "rustocop/project_corpus"

RSpec.describe Rustocop::ProjectCorpus do
  subject(:projects) { described_class::PROJECTS }

  it "contains the ten-project baseline and forty-project expansion" do
    expect(described_class::BASELINE_PROJECTS.length).to eq(10)
    expect(described_class::EXPANSION_PROJECTS.length).to eq(40)
    expect(projects.length).to eq(50)
  end

  it "uses unique names, repositories, and immutable Git revisions" do
    expect(projects.map { |project| project.fetch("name") }.uniq.length).to eq(50)
    expect(projects.map { |project| project.fetch("repository") }.uniq.length).to eq(50)
    expect(projects).to all(include("revision" => match(/\A[0-9a-f]{40}\z/)))
  end

  it "records a license for every project" do
    expect(projects).to all(include("license" => be_a(String)))
    expect(projects).to all(satisfy { |project| !project.fetch("license").empty? })
  end

  it "freezes the corpus and every normalized project entry" do
    expect(projects).to be_frozen
    expect(projects).to all(be_frozen)
    expect(described_class::BASELINE_PROJECTS).to all(be_frozen)
    expect(described_class::EXPANSION_PROJECTS).to all(be_frozen)
  end
end
