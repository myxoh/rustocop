# frozen_string_literal: true

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
end
