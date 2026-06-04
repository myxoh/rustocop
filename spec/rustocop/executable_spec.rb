# frozen_string_literal: true

RSpec.describe "rustocop executable" do
  it "uses the native binary by default" do
    result = run_rustocop("-V")

    expect(result.stderr).to eq("")
    expect(result.stdout).to include("rustocop native")
    expect(result.status.exitstatus).to eq(0)
  end

  it "uses the rustocop name and version" do
    result = run_rustocop("--version")

    expect(result.stderr).to eq("")
    expect(result.stdout).to eq("#{Rustocop::VERSION}\n")
    expect(result.status.exitstatus).to eq(0)
  end

  it "keeps the Ruby fallback explicit for development" do
    result = run_rustocop("-V", env: { "RUSTOCOP_DISABLE_NATIVE" => "1" })

    expect(result.stderr).to eq("")
    expect(result.stdout).to include("rustocop ruby fallback")
    expect(result.status.exitstatus).to eq(0)
  end
end
