# frozen_string_literal: true

require_relative "../lib/rustocop/benchmark_documentation"

RSpec.describe Rustocop::BenchmarkDocumentation do
  it "generates memory documentation from benchmark measurements" do
    Dir.mktmpdir do |root|
      docs = File.join(root, "docs")
      FileUtils.mkdir_p(docs)
      path = File.join(docs, "performance.md")
      File.write(path, "<!-- generated:memory-results:start -->\nold\n<!-- generated:memory-results:end -->\n")
      report = {
        "results" => [{
          "files" => 500,
          "rustocop" => memory(4),
          "rustocop_parallel" => memory(5),
          "rubocop_prism" => memory(90)
        }]
      }

      2.times { described_class.update_memory(root, report) }

      expect(File.read(path)).to include("| 500 | 4.00 / 4.50 MiB | 5.00 / 5.50 MiB | 90.00 / 90.50 MiB |")
      expect(File.read(path)).to include("1.00 MiB over sequential rustocop")
      expect(File.read(path)).to include("18.0 times as much peak memory")
    end
  end

  def memory(mib)
    {
      "median_peak_rss_bytes" => mib * 1024**2,
      "p95_peak_rss_bytes" => (mib + 0.5) * 1024**2
    }
  end
end
