# frozen_string_literal: true

require_relative "../script/support/benchmark"

RSpec.describe BenchmarkSupport do
  subject(:support) { Object.new.extend(described_class) }

  it "materializes the independent pinned benchmark corpus" do
    cops, paths = support.benchmark_corpus(ROOT)

    expect(cops.length).to eq(20)
    expect(paths.length).to eq(500)
    expect(paths.sum { |path| File.size(path) }).to eq(9_110)
    expect(paths).to all(start_with(File.join(ROOT, "tmp/performance-verification/benchmark-corpus")))
  end
end
