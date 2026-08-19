# frozen_string_literal: true

require_relative "../lib/rustocop/compatibility_status"

RSpec.describe "hardened cop compatibility" do
  status = Rustocop::CompatibilityStatus.load(root: ROOT)

  status.hardening_entries.each do |cop, evidence|
    fixture = File.join(ROOT, evidence.fetch("fixture"))

    it "matches RuboCop diagnostics for adversarial #{cop} input" do
      rubocop = run_rubocop("--format", "json", "--only", cop, fixture)
      rustocop = run_rustocop("--format", "json", "--only", cop, fixture)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
        normalize_rubocop_report(parsed_json(rubocop))
      )
    end

    it "matches RuboCop correction for adversarial #{cop} input" do
      Dir.mktmpdir("rustocop-hardening") do |directory|
        rubocop_path = copy_fixture(fixture, directory, "rubocop")
        rustocop_path = copy_fixture(fixture, directory, "rustocop")
        rubocop = run_rubocop("-A", "--format", "json", "--only", cop, rubocop_path)
        rustocop = run_rustocop("-A", "--format", "json", "--only", cop, rustocop_path)

        expect(rustocop.stderr).to eq("")
        expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
        expect(File.binread(rustocop_path)).to eq(File.binread(rubocop_path))
        expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
          normalize_rubocop_report(parsed_json(rubocop))
        )
      end
    end
  end

  def copy_fixture(fixture, directory, implementation)
    destination = File.join(directory, implementation, File.basename(fixture))
    FileUtils.mkdir_p(File.dirname(destination))
    FileUtils.cp(fixture, destination)
    destination
  end
end
