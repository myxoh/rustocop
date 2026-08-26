# frozen_string_literal: true

require "tmpdir"
require_relative "../lib/rustocop/native_result_cache"

RSpec.describe Rustocop::NativeResultCache do
  let(:metadata) do
    {
      "native_sha256" => "native",
      "config_sha256" => "config",
      "project" => { "name" => "example", "revision" => "abc" },
      "files" => 2,
      "cops" => %w[Layout/A Lint/B]
    }
  end
  let(:report) do
    {
      "metadata" => { "rubocop_version" => "test" },
      "files" => [
        { "path" => "empty.rb", "offenses" => [] },
        { "path" => "offense.rb", "offenses" => [{ "cop_name" => "Layout/A" }] }
      ],
      "summary" => {
        "offense_count" => 1,
        "target_file_count" => 2,
        "inspected_file_count" => 2
      }
    }
  end
  let(:result) do
    {
      "stdout" => "captured Rustocop report",
      "stderr" => "warning\n",
      "exitstatus" => 1,
      "seconds" => 4.5,
      "encoded_offenses" => {
        "paths" => ["offense.rb"],
        "cops" => %w[Layout/A Lint/B],
        "messages" => ["Example"],
        "offenses" => [[0, 0, "convention", 0, 1, 1, 1, 2]]
      }
    }
  end

  it "round trips only the compact diagnostic payload" do
    Dir.mktmpdir("rustocop-native-cache") do |root|
      cache = described_class.new(root:)
      expect(cache.cached?(metadata)).to be(false)
      stored = cache.store(metadata, result)
      expect(cache.cached?(metadata)).to be(true)
      cached = cache.fetch(metadata)

      expect(stored.fetch("cache_hit")).to be(false)
      expect(cached.fetch("cache_hit")).to be(true)
      expect(cached.fetch("stderr")).to eq("warning\n")
      expect(cached.fetch("encoded_offenses")).to eq(result.fetch("encoded_offenses"))
      expect(cached).not_to have_key("report")
    end
  end

  it "does not reuse a result when any cache-key metadata changes" do
    Dir.mktmpdir("rustocop-native-cache") do |root|
      cache = described_class.new(root:)
      cache.store(metadata, result)

      expect(cache.fetch(metadata.merge("native_sha256" => "changed"))).to be_nil
      expect(cache.fetch(metadata.merge("cops" => ["Layout/A"]))).to be_nil
    end
  end
end
