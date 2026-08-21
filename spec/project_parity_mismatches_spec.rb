# frozen_string_literal: true

RSpec.describe "isolated real-project parity mismatches" do
  fixture_root = File.join(ROOT, "spec", "fixtures", "project_parity_regressions")
  config = File.join(fixture_root, "rubocop.yml")
  native = File.join(ROOT, "crates", "rustocop", "target", "release", "rustocop")
  rows = File.readlines(File.join(fixture_root, "mismatches.tsv"), chomp: true).drop(1).map do |line|
    line.split("\t", 6)
  end

  rows.each do |cop, file, repository, revision, source_path, kind|
    it "isolates #{kind} for #{cop} from #{repository}@#{revision}:#{source_path}" do
      pending "known project-parity mismatch; promote to manifest.tsv after fixing"

      path = File.join(fixture_root, file)
      arguments = ["--config", config, "--format", "json", "--only", cop, path]
      rubocop = run_rubocop("--no-server", *arguments)
      rustocop = run_rustocop(*arguments, env: { "RUSTOCOP_NATIVE_PATH" => native })

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
        normalize_rubocop_report(parsed_json(rubocop))
      )
    end
  end
end
