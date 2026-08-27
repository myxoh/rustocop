# frozen_string_literal: true

RSpec.describe Rustocop::ConfigurationCompiler do
  def run_compiler(*arguments, chdir:)
    Rustocop::ProcessRunner.capture(
      RbConfig.ruby, File.join(ROOT, "exe", "rustocop-config"), *arguments, chdir:
    )
  end

  it "flattens RuboCop inheritance and compiles enabled states" do
    Dir.mktmpdir("rustocop-config-compiler") do |directory|
      File.write(File.join(directory, "base.yml"), <<~YAML)
        AllCops:
          DisabledByDefault: true

        Style/StringLiterals:
          Enabled: true
          EnforcedStyle: double_quotes
      YAML
      File.write(File.join(directory, ".rubocop.yml"), "inherit_from: base.yml\n")

      result = run_compiler(chdir: directory)
      output = File.join(directory, ".rustocop.yml")
      compiled = YAML.safe_load_file(output)

      expect(result.status.exitstatus).to eq(0)
      expect(result.stderr).to eq("")
      expect(File.realpath(result.stdout.strip)).to eq(File.realpath(output))
      expect(compiled.dig("Rustocop", "SchemaVersion")).to eq(1)
      expect(compiled.dig("Rustocop", "SourceConfig")).to eq(".rubocop.yml")
      expect(compiled.dig("Rustocop", "ProjectRoot")).to eq(".")
      expect(compiled.dig("Rustocop", "BuiltInCops")).to include("Style/StringLiterals")
      expect(compiled.dig("Style/StringLiterals", "Enabled")).to be(true)
      expect(compiled.dig("Style/StringLiterals", "EnforcedStyle")).to eq("double_quotes")
      expect(compiled.dig("Layout/LineLength", "Enabled")).to be(false)
      expect(File.read(output)).not_to include("inherit_from")
    end
  end

  it "requires --force before replacing a compiled configuration" do
    Dir.mktmpdir("rustocop-config-compiler") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), "AllCops:\n  NewCops: disable\n")
      File.write(File.join(directory, ".rustocop.yml"), "existing\n")

      refused = run_compiler(chdir: directory)
      replaced = run_compiler("--force", chdir: directory)

      expect(refused.status.exitstatus).to eq(2)
      expect(refused.stderr).to include("refusing to overwrite")
      expect(replaced.status.exitstatus).to eq(0)
      expect(File.read(File.join(directory, ".rustocop.yml"))).to include("SchemaVersion")
    end
  end

  it "checks whether the compiled configuration is current" do
    Dir.mktmpdir("rustocop-config-compiler") do |directory|
      source = File.join(directory, ".rubocop.yml")
      File.write(source, "AllCops:\n  NewCops: disable\n")
      expect(run_compiler(chdir: directory).status.exitstatus).to eq(0)

      current = run_compiler("--check", chdir: directory)
      File.write(source, "AllCops:\n  NewCops: enable\n")
      stale = run_compiler("--check", chdir: directory)

      expect(current.status.exitstatus).to eq(0)
      expect(stale.status.exitstatus).to eq(1)
      expect(stale.stderr).to include("is missing or stale")
    end
  end
end
