# frozen_string_literal: true

require_relative "../lib/rustocop/compatibility_status"

RSpec.describe "real-project parity regressions", project_audit: true do
  fixture_root = File.join(ROOT, "spec", "fixtures")
  config = File.join(fixture_root, "support", "project_regressions.yml")
  native = ENV.fetch("RUSTOCOP_NATIVE_PATH", File.join(ROOT, "crates", "rustocop", "target", "release", "rustocop"))
  rows = File.readlines(File.join(fixture_root, "cop_project_cases.tsv"), chomp: true).drop(1).map do |line|
    line.split("\t", 5)
  end

  Rustocop::CompatibilityStatus.load(root: ROOT).validate_verified!(
    rows.map(&:first).uniq,
    label: "project parity regression manifest"
  )

  rows.each do |cop, file, repository, revision, source_path|
    it "matches RuboCop for #{cop} from #{repository}@#{revision}:#{source_path}", cop: cop do
      Dir.mktmpdir("rustocop-project-parity-") do |directory|
        path = File.join(directory, source_path)
        FileUtils.mkdir_p(File.dirname(path))
        FileUtils.cp(File.join(fixture_root, file), path)
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
end
