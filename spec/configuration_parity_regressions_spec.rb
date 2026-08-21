# frozen_string_literal: true

RSpec.describe "configuration mutation parity regressions" do
  fixture_root = File.join(ROOT, "spec", "fixtures", "configuration_parity_regressions")
  native = File.join(ROOT, "crates", "rustocop", "target", "release", "rustocop")
  rows = File.readlines(File.join(fixture_root, "manifest.tsv"), chomp: true).drop(1).map do |line|
    line.split("\t", 5)
  end

  rows.each do |cop, file, config, repository, source_path|
    it "matches RuboCop for #{cop} with #{config} from #{repository}:#{source_path}" do
      arguments = [
        "--config", File.join(fixture_root, config),
        "--format", "json",
        "--only", cop,
        File.join(fixture_root, file)
      ]
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
