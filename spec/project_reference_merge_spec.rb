# frozen_string_literal: true

require_relative "../lib/rustocop/project_reference_merge"

RSpec.describe Rustocop::ProjectReferenceMerge do
  def reference(cop, offense)
    {
      "version" => 2,
      "kind" => "rubocop_project_reference",
      "generated_at" => "old",
      "rubocop_version" => "1.87.0",
      "config_sha256" => "config",
      "project_revisions" => [{ "name" => "sample", "revision" => "abc" }],
      "cops" => [cop],
      "rubocop_errors" => [],
      "projects" => {
        "sample" => {
          "files" => 1,
          "seconds" => 1.0,
          "warning_count" => 0,
          "paths" => ["sample.rb"],
          "messages" => [offense.fetch(:message)],
          "offenses" => [[0, 0, "warning", 0, *offense.fetch(:position)]]
        }
      }
    }
  end

  it "reindexes disjoint cop references without changing diagnostic signatures" do
    first = reference("Lint/First", message: "first", position: [2, 1, 2, 2])
    second = reference("Style/Second", message: "second", position: [1, 1, 1, 2])

    merged = described_class.merge([first, second])

    expect(merged.fetch("cops")).to eq(%w[Lint/First Style/Second])
    project = merged.fetch("projects").fetch("sample")
    expect(project.fetch("offenses")).to eq([
      [0, 1, "warning", 0, 1, 1, 1, 2],
      [0, 0, "warning", 1, 2, 1, 2, 2]
    ])
    expect(project.fetch("seconds")).to eq(2.0)
  end

  it "rejects overlapping cop caches" do
    one = reference("Lint/Same", message: "one", position: [1, 1, 1, 2])
    two = reference("Lint/Same", message: "two", position: [2, 1, 2, 2])

    expect { described_class.merge([one, two]) }
      .to raise_error(described_class::Error, /overlap/)
  end
end
