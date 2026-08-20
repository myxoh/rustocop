# frozen_string_literal: true

require_relative "../lib/rustocop/qualification_batch"

RSpec.describe Rustocop::QualificationBatch do
  describe Rustocop::QualificationBatch::Corpus do
    subject(:corpus) { described_class.allocate }

    it "selects a diverse set of reviewable upstream cases" do
      cases = [
        captured_case("plain offense", "items.length == 0\n", offenses: [{}]),
        captured_case("clean form", "items.empty?\n"),
        captured_case("correctable form", "items.size > 0\n", offenses: [{}], correction: "!items.empty?\n"),
        captured_case("multiline form", "items\n  .length == 0\n", offenses: [{}]),
        captured_case("duplicate shape", "other.length == 0\n", offenses: [{}])
      ]

      selected = corpus.select_edges(cases)

      expect(selected.length).to eq(4)
      expect(selected.map { |item| item.fetch("id") }).to all(start_with("upstream_"))
      expect(selected.map { |item| item.fetch("description") }).to include("clean form", "correctable form")
      expect(selected).to all(include("config" => include("AllCops")))
    end

    def captured_case(description, source, offenses: [], correction: nil)
      value = {
        "cop" => "Style/Example",
        "source" => source,
        "path" => "example.rb",
        "ruby_version" => 3.4,
        "config" => {
          "Style/Example" => {
            "Description" => "metadata",
            "Enabled" => true,
            "EnforcedStyle" => "example"
          }
        },
        "offenses" => offenses,
        "example" => { "description" => description }
      }
      value["correction"] = correction if correction
      value
    end
  end

  describe Rustocop::QualificationBatch::SnippetExtractor do
    it "extracts the complete statement containing an offense" do
      source = <<~RUBY
        def example
          if records.length == 0
            work
          end
        end
      RUBY

      snippet = described_class.new.extract(source, 2)

      expect(snippet).to include("if records.length == 0", "work", "end")
      expect(Prism.parse(snippet)).to be_success
    end
  end

  describe Rustocop::QualificationBatch::ReviewPacket do
    it "makes the Ruby and Rust internal shapes directly comparable" do
      document = {
        "rubocop_version" => "1.87.0",
        "rubocop_commit" => "ruby-sha",
        "rustocop_commit" => "rust-sha",
        "cops" => {
          "Style/Example" => {
            "sources" => { "rubocop" => "example.rb", "rustocop" => ["example.rs"] },
            "upstream_tests" => { "passed" => 4, "total" => 4 },
            "real_world" => { "positives" => [{}, {}], "negatives" => [{}, {}] },
            "preparation" => {
              "action" => "audit existing implementation",
              "rust_source_state" => "unchanged",
              "internals" => {
                "ruby" => { "callbacks" => ["on_send"], "helpers" => [], "configuration" => [], "offense_api" => ["add_offense"] },
                "rust" => { "callbacks" => ["on_send"], "helpers" => [], "configuration" => [], "offense_api" => ["report_selector"] }
              }
            }
          }
        }
      }

      packet = described_class.new.render(document)

      expect(packet).to include("| Callbacks | `on_send` | `on_send` |")
      expect(packet).to include("Human review remains required")
    end
  end
end
