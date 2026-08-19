# frozen_string_literal: true

require_relative "../lib/rustocop/compatibility_drift"
require_relative "../lib/rustocop/compatibility_status"

RSpec.describe Rustocop::CompatibilityDrift do
  let(:status) do
    Rustocop::CompatibilityStatus.new(
      root: ROOT,
      version: "test",
      data: { "fully_compatible_cops" => ["Style/Verified"] }
    )
  end
  let(:report) do
    {
      "results" => {
        "Style/Verified" => { "status" => "failing" },
        "Style/Ready" => { "status" => "passing" },
        "Style/Pending" => { "status" => "failing" }
      }
    }
  end

  it "reports promotion, regression, and correction-coverage drift" do
    contracts = {
      "Style/Ready" => { "correctable_cases" => 2, "assertions" => 0 }
    }

    expect(described_class.analyze(report, status, correction_contracts: contracts)).to eq(
      "passing_not_promoted" => ["Style/Ready"],
      "verified_regressions" => ["Style/Verified"],
      "passing_without_correction_assertions" => ["Style/Ready"]
    )
  end
end
