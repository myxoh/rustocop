# frozen_string_literal: true

RSpec.describe Rustocop::SourceFingerprint do
  it "fingerprints cop source deterministically and ignores unrelated files" do
    Dir.mktmpdir("rustocop-source-fingerprint") do |root|
      cops = File.join(root, "crates", "rustocop", "src", "cops")
      FileUtils.mkdir_p(cops)
      File.write(File.join(cops, "b.rs"), "second\n")
      File.write(File.join(cops, "a.rs"), "first\n")

      initial = described_class.cops(root:)
      File.write(File.join(root, "README.md"), "ignored\n")
      expect(described_class.cops(root:)).to eq(initial)

      File.write(File.join(cops, "a.rs"), "changed\n")
      expect(described_class.cops(root:)).not_to eq(initial)
    end
  end
end
