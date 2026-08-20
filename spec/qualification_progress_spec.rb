# frozen_string_literal: true

require_relative "../lib/rustocop/qualification_progress"

RSpec.describe Rustocop::QualificationProgress do
  it "distinguishes complete records from records whose Rust source changed" do
    Dir.mktmpdir do |root|
      rubocop_root = File.join(root, "rubocop")
      FileUtils.mkdir_p(File.join(root, "qualification/work"))
      FileUtils.mkdir_p(File.join(root, "crates/rustocop"))
      FileUtils.mkdir_p(File.join(rubocop_root, "lib/rubocop/cop/style"))
      File.write(File.join(root, "crates/rustocop/example.rs"), "example")
      File.write(File.join(rubocop_root, "lib/rubocop/cop/style/example.rb"), "example")
      File.write(File.join(root, "qualification/work/example.yml"), YAML.dump(document))

      ledger = described_class.new(
        root: root,
        rubocop_root: rubocop_root,
        source_current: ->(record) { record.fetch("cop") == "Style/Current" }
      )

      expect(ledger.evidence_complete_count).to eq(2)
      expect(ledger.fully_qualified_count).to eq(1)
      expect(ledger.stale_records.map { |record| record.fetch("cop") }).to eq(["Style/Stale"])
      expect(ledger.recorded_count(4)).to eq(2)
      expect(ledger.current_count(4)).to eq(1)
    end
  end

  def document
    record = {
      "sources" => {
        "rubocop" => "lib/rubocop/cop/style/example.rb",
        "rustocop" => ["crates/rustocop/example.rs"]
      },
      "manual_review" => { "status" => "passed", "notes" => %w[first second] },
      "upstream_tests" => { "status" => "passed", "passed" => 2, "total" => 2, "corrections" => true },
      "edge_cases" => 4.times.map { |index| { "id" => "edge_#{index}" } },
      "real_world" => {
        "positives" => real_examples("positive"),
        "negatives" => real_examples("negative")
      }
    }
    {
      "schema" => 1,
      "batch" => "example",
      "rubocop_commit" => "r" * 40,
      "rustocop_commit" => "a" * 40,
      "cops" => {
        "Style/Current" => record.merge("matrix_position" => 2),
        "Style/Stale" => Marshal.load(Marshal.dump(record)).merge("matrix_position" => 1)
      }
    }
  end

  def real_examples(prefix)
    2.times.map do |index|
      {
        "repository" => "owner/project",
        "revision" => index.to_s * 40,
        "path" => "#{prefix}_#{index}.rb",
        "line" => index + 1,
        "source" => "example\n"
      }
    end
  end
end
