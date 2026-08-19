# frozen_string_literal: true

require_relative "../lib/rustocop/generated_section"

RSpec.describe Rustocop::GeneratedSection do
  it "replaces only the named generated region and is idempotent" do
    Dir.mktmpdir do |directory|
      path = File.join(directory, "document.md")
      File.write(path, "before\n<!-- generated:example:start -->\nold\n<!-- generated:example:end -->\nafter\n")

      2.times { described_class.replace(path, "example", "new\ncontent") }

      expect(File.read(path)).to eq(
        "before\n<!-- generated:example:start -->\nnew\ncontent\n<!-- generated:example:end -->\nafter\n"
      )
    end
  end
end
