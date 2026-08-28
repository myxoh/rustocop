# frozen_string_literal: true

require "fileutils"
require "json"
require "tmpdir"
require "rustocop/structural_parity"

RSpec.describe Rustocop::StructuralParity do
  around do |example|
    Dir.mktmpdir do |dir|
      @root = dir
      %w[compatibility/structural/dossiers compatibility/structural/attestations crates/rustocop/src/cops/prism/framework].each { |path| FileUtils.mkdir_p(File.join(dir, path)) }
      FileUtils.mkdir_p(File.join(dir, "crates/rustocop/src/cops/prism"))
      File.write(File.join(dir, "compatibility/structural/standard.md"), "standard")
      File.write(File.join(dir, "crates/rustocop/src/cops/prism/framework/adapter.rs"), "adapter")
      File.write(File.join(dir, "crates/rustocop/src/cops/prism/example.rs"), "rule")
      manifest = {"cops" => [{
        "cop" => "Style/Example", "upstream_source" => "lib/example.rb", "upstream_sha256" => "pinned",
        "upstream_callbacks" => ["on_send"], "upstream_mixins" => [],
        "upstream_dependencies" => [{"gem" => "ast", "version" => "2.4.3", "sha256" => "dependency"}],
        "implementations" => ["src/cops/prism/example.rs"],
        "fixtures" => {"status" => "compatible"}, "projects" => {"classification" => "dormant"},
        "similarity_score" => 5, "structural_status" => "near_source_shaped",
        "migration_status" => "migrated", "structural_gaps" => [], "documented_adaptations" => ["legacy"]
      }]}
      path = File.join(dir, "manifest.json")
      File.write(path, JSON.generate(manifest))
      @workflow = described_class.new(root: dir, legacy_manifest: path)
      example.run
    end
  end

  it "treats legacy migration declarations as unverified" do
    expect(@workflow.state("Style/Example")).to eq("legacy_unverified")
    expect(@workflow.next_cop).to eq(["prepare", "Style/Example"])
  end

  it "does not accept a dossier without an attestation" do
    path = @workflow.init_dossier("Style/Example")
    expect(@workflow.state("Style/Example")).to eq("obligations_extracted")
    expect(@workflow.next_cop).to eq(["prepare", "Style/Example"])
    expect(JSON.parse(File.read(path)).dig("fingerprints", "upstream_dependencies_sha256")).not_to be_nil
  end

  it "rejects an incomplete dossier before ready" do
    @workflow.init_dossier("Style/Example")
    expect { @workflow.transition("Style/Example", "dossier_ready") }.to raise_error(ArgumentError, /facet callbacks/)
  end

  it "invalidates acceptance after implementation changes" do
    path = @workflow.init_dossier("Style/Example")
    dossier = JSON.parse(File.read(path))
    dossier["facets"].each_value { |facet| facet.replace("status" => "not_applicable", "notes" => "none in minimal test") }
    File.write(path, "#{JSON.pretty_generate(dossier)}\n")
    @workflow.transition("Style/Example", "dossier_ready")
    @workflow.transition("Style/Example", "implementation_submitted")
    attestation = @workflow.attestation_template("Style/Example", "fresh-reviewer")
    attestation["statement"] = "Independently verified this minimal inventory."
    File.write(@workflow.attestation_path("Style/Example"), "#{JSON.pretty_generate(attestation)}\n")
    expect(@workflow.state("Style/Example")).to eq("accepted")
    File.write(File.join(@root, "crates/rustocop/src/cops/prism/example.rs"), "changed")
    expect(@workflow.state("Style/Example")).to eq("invalidated")
  end
end
