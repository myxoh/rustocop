# frozen_string_literal: true

require "json"
require_relative "../lib/rustocop/compatibility_status"

RSpec.describe Rustocop::CompatibilityStatus do
  subject(:status) do
    described_class.new(
      root: ROOT,
      version: "test",
      data: { "fully_compatible_cops" => ["Style/Verified"] }
    )
  end

  it "accepts verified-only contracts" do
    expect { status.validate_verified!(["Style/Verified"], label: "fixture") }.not_to raise_error
  end

  it "identifies every non-verified cop in a contract" do
    expect do
      status.validate_verified!(["Style/Verified", "Style/Heuristic"], label: "fixture")
    end.to raise_error(
      ArgumentError,
      "fixture contains non-verified cops: Style/Heuristic"
    )
  end

  it "requires hardened cops to be verified and backed by complete evidence" do
    hardening_data = {
      "version" => 1,
      "rubocop_version" => "test",
      "required_categories" => ["comments", "strings"],
      "cops" => {
        "Style/Verified" => {
          "fixture" => "spec/compatibility_status_spec.rb",
          "categories" => ["comments", "strings"],
          "evidence" => ["spec/compatibility_status_spec.rb"]
        }
      }
    }
    hardened = described_class.new(
      root: ROOT,
      version: "test",
      data: { "fully_compatible_cops" => ["Style/Verified"] },
      hardening_data:
    )

    expect(hardened.validate_hardening!).to be(true)
    expect(hardened).to be_hardened("Style/Verified")
  end

  it "keeps intentionally-pending cops out of every active corpus" do
    actual = described_class.load(root: ROOT)
    pending = actual.intentionally_pending_cops
    expect(pending.length).to eq(73)
    expect(pending & actual.built_in_cops).to be_empty

    rust_source = File.read(File.join(ROOT, "crates/rustocop/src/cops/mod.rs"))
    rust_pending = rust_source
      .match(/INTENTIONALLY_PENDING_COP_NAMES:.*?= &\[(.*?)\];/m)[1]
      .scan(/"([A-Za-z]+\/[A-Za-z0-9]+)"/)
      .flatten
    expect(rust_pending).to eq(pending)

    %w[fixtures projects].each do |name|
      snapshot = JSON.parse(
        File.read(File.join(ROOT, "spec/compatibility_evidence/#{name}.json"))
      )
      expect(snapshot.fetch("results").keys & pending).to be_empty
    end
  end
end
