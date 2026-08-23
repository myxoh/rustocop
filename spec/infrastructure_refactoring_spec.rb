# frozen_string_literal: true

RSpec.describe "shared infrastructure" do
  describe Rustocop::RepositoryLayout do
    subject(:layout) { described_class.new("/tmp/example-rustocop") }

    it "owns repository path conventions" do
      expect(layout.benchmark_config).to eq("/tmp/example-rustocop/benchmark/project-rubocop.yml")
      expect(layout.native_binary).to eq("/tmp/example-rustocop/crates/rustocop/target/release/rustocop")
      expect(layout.project_corpus("name" => "rails", "revision" => "abc123")).to(
        eq("/tmp/example-rustocop/tmp/project-benchmarks/corpora/rails-abc123")
      )
    end
  end

  describe Rustocop::ArtifactStore do
    it "round-trips pretty JSON and deterministic compressed JSON atomically" do
      Dir.mktmpdir("rustocop-artifacts-") do |directory|
        json = File.join(directory, "nested", "report.json")
        gzip = File.join(directory, "nested", "reference.json.gz")
        value = { "answer" => 42, "items" => %w[a b] }

        described_class.write_json(json, value, trailing_newline: true)
        described_class.write_gzip_json(gzip, value)
        first_compressed = File.binread(gzip)
        described_class.write_gzip_json(gzip, value)

        expect(described_class.read_json(json)).to eq(value)
        expect(described_class.read_gzip_json(gzip)).to eq(value)
        expect(File.binread(gzip)).to eq(first_compressed)
        expect(File.read(json)).to end_with("\n")
      end
    end
  end

  describe Rustocop::ProcessRunner do
    it "returns one normalized timed result for successful and accepted commands" do
      result = described_class.capture(RbConfig.ruby, "-e", "STDOUT.write('ok'); exit 1")

      expect(result.stdout).to eq("ok")
      expect(result).not_to be_success
      expect(result).to be_accepted(0, 1)
      expect(result.seconds).to be >= 0
      expect(result.to_h).to include("exitstatus" => 1, "stdout" => "ok")
    end
  end

  describe Rustocop::DiagnosticSignatures do
    it "normalizes paths and complete offense locations once" do
      report = {
        "files" => [{
          "path" => "/tmp/corpus/app/model.rb",
          "offenses" => [{
            "cop_name" => "Style/Example",
            "severity" => "convention",
            "message" => "Use the example.",
            "location" => {
              "start_line" => 2, "start_column" => 3,
              "last_line" => 2, "last_column" => 7
            }
          }]
        }]
      }

      signature = described_class.from_report(report, corpus: "/tmp/corpus").fetch(0)

      expect(signature.path).to eq("app/model.rb")
      expect(signature.cop).to eq("Style/Example")
      expect(signature.location_tuple).to eq(["convention", "Use the example.", 2, 3, 2, 7])
      expect(signature.to_h).to include("path" => "app/model.rb", "last_column" => 7)
    end
  end
end
