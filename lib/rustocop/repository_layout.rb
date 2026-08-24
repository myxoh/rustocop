# frozen_string_literal: true

module Rustocop
  class RepositoryLayout
    attr_reader :root

    def self.default
      @default ||= new(File.expand_path("../..", __dir__))
    end

    def initialize(root)
      @root = File.expand_path(root)
    end

    def path(*parts)
      File.join(root, *parts)
    end

    def benchmark_config
      path("benchmark", "project-rubocop.yml")
    end

    def native_binary(profile: "release")
      path("crates", "rustocop", "target", profile, "rustocop")
    end

    def rust_manifest
      path("crates", "rustocop", "Cargo.toml")
    end

    def compatibility_evidence(name)
      path("spec", "compatibility_evidence", name)
    end

    def fixture_root
      path("spec", "fixtures")
    end

    def project_regression_manifest(pending: false)
      name = pending ? "cop_project_mismatches.tsv" : "cop_project_cases.tsv"
      path("spec", "fixtures", name)
    end

    def project_regression_config
      path("spec", "fixtures", "support", "project_regressions.yml")
    end

    def project_corpus(project)
      path(
        "tmp", "project-benchmarks", "corpora",
        "#{project.fetch('name')}-#{project.fetch('revision')}"
      )
    end

    def upstream(version, *parts)
      path("spec", "upstream", "rubocop-#{version}", *parts)
    end
  end
end
