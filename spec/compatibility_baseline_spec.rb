# frozen_string_literal: true

require_relative "../lib/rustocop/compatibility_baseline"

RSpec.describe Rustocop::CompatibilityBaseline do
  let(:status) do
    {
      "captured_cases" => 3,
      "diagnostic_baseline" => {
        "passed_cases" => 2,
        "passing_cops" => 1,
        "total_cops" => 2
      },
      "fully_compatible_cops" => ["Style/Good"]
    }
  end
  let(:summary) do
    {
      "cases" => 3,
      "passed_cases" => 2,
      "cops" => 2,
      "passing_cops" => 1,
      "results" => {
        "Style/Good" => { "status" => "passing", "passed" => 1, "total" => 1 },
        "Style/Pending" => { "status" => "failing", "passed" => 1, "total" => 2 }
      }
    }
  end

  it "accepts the approved partial baseline" do
    expect(described_class.errors(summary, status)).to be_empty
  end

  it "accepts compatibility improvements without requiring an immediate baseline update" do
    summary["passed_cases"] = 3
    summary["passing_cops"] = 2
    summary["results"]["Style/Pending"] = {
      "status" => "passing", "passed" => 2, "total" => 2
    }

    expect(described_class.errors(summary, status)).to be_empty
  end

  it "reports aggregate drift and verified-cop regressions" do
    summary["passed_cases"] = 1
    summary["results"]["Style/Good"]["status"] = "failing"
    summary["results"]["Style/Good"]["passed"] = 0

    expect(described_class.errors(summary, status)).to contain_exactly(
      "passing cases: expected at least 2, got 1",
      "verified cop regressed: Style/Good (0/1)"
    )
  end
end
