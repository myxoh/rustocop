# frozen_string_literal: true

require "json"
require_relative "../lib/rustocop/batched_native_reports"

RSpec.describe Rustocop::BatchedNativeReports do
  def report(files, offense_count:)
    {
      "metadata" => { "rubocop_version" => "test" },
      "files" => files,
      "summary" => {
        "offense_count" => offense_count,
        "target_file_count" => 2,
        "inspected_file_count" => 2
      }
    }
  end

  def successful_result(value, seconds: 0.5)
    {
      "stdout" => JSON.generate(value),
      "stderr" => "",
      "exitstatus" => value.fetch("summary").fetch("offense_count").zero? ? 0 : 1,
      "seconds" => seconds
    }
  end

  it "merges offenses by file while preserving report order and summary counts" do
    first = report([
      { "path" => "a.rb", "offenses" => [{ "cop_name" => "Layout/A" }] },
      { "path" => "b.rb", "offenses" => [] }
    ], offense_count: 1)
    second = report([
      { "path" => "a.rb", "offenses" => [{ "cop_name" => "Lint/B" }] },
      { "path" => "b.rb", "offenses" => [{ "cop_name" => "Lint/B" }] }
    ], offense_count: 2)

    merged = described_class.merge([first, second])

    expect(merged.fetch("files").map { |file| file.fetch("path") }).to eq(%w[a.rb b.rb])
    expect(merged.fetch("files").first.fetch("offenses").map { |offense| offense.fetch("cop_name") })
      .to eq(%w[Layout/A Lint/B])
    expect(merged.dig("summary", "offense_count")).to eq(3)
    expect(merged.dig("summary", "target_file_count")).to eq(2)
  end

  it "runs bounded cop batches and accumulates their reports and timings" do
    batches = []
    result = described_class.capture(cops: %w[A B C], batch_size: 2, run: lambda do |batch|
      batches << batch
      files = [{ "path" => "a.rb", "offenses" => batch.map { |cop| { "cop_name" => cop } } }]
      successful_result(report(files, offense_count: batch.length))
    end)

    expect(batches).to eq([%w[A B], %w[C]])
    expect(result.fetch("seconds")).to eq(1.0)
    expect(result.fetch("cache_hits")).to eq(0)
    expect(result.fetch("cache_misses")).to eq(0)
    expect(result.dig("report", "summary", "offense_count")).to eq(3)
    expect(result.fetch("exitstatus")).to eq(1)
  end

  it "keeps compact diagnostic batches compact" do
    result = described_class.capture(cops: %w[A B], batch_size: 1, run: lambda do |batch|
      {
        "stdout" => "captured Rustocop report",
        "stderr" => "",
        "exitstatus" => 1,
        "seconds" => 0.25,
        "cache_hit" => true,
        "encoded_offenses" => {
          "paths" => ["a.rb"],
          "cops" => batch,
          "messages" => ["Example"],
          "offenses" => [[0, 0, "convention", 0, 1, 1, 1, 2]]
        }
      }
    end)

    expect(result.fetch("encoded_offenses").length).to eq(2)
    expect(result.fetch("cache_hits")).to eq(2)
    expect(result).not_to have_key("report")
    expect(result.fetch("exitstatus")).to eq(1)
  end

  it "returns the failing batch so crash isolation stays bounded" do
    result = described_class.capture(cops: %w[A B C], batch_size: 2, run: lambda do |batch|
      next successful_result(report([], offense_count: 0)) if batch == %w[A B]

      { "stdout" => "", "stderr" => "crash", "exitstatus" => 2, "seconds" => 0.25 }
    end)

    expect(result.fetch("failed_cops")).to eq(%w[C])
    expect(result.fetch("exitstatus")).to eq(2)
  end
end
