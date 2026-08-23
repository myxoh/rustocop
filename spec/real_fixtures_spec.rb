# frozen_string_literal: true

require "yaml"

RSpec.describe "real fixtures" do
  fixture_root = File.join(ROOT, "real_fixtures")
  fixture_directories = Dir.glob(File.join(fixture_root, "[0-9][0-9]_*"))
  rubocop_environment = { "RUBOCOP_CACHE_ROOT" => File.join(Dir.tmpdir, "rustocop-rubocop-cache") }

  it "contains the remaining initial examples and sourced real-world fixtures" do
    expect(fixture_directories.length).to be >= 17
  end

  fixture_directories.sort.each do |fixture_directory|
    fixture_name = File.basename(fixture_directory)

    it "matches RuboCop for #{fixture_name}" do
      expected_files = %w[input.rb rubocop.yml output.rb output.out]
      expect(expected_files).to all(satisfy { |name| File.file?(File.join(fixture_directory, name)) })

      diagnostics = run_rubocop(
        "--no-server",
        "--config",
        "rubocop.yml",
        "--format",
        "simple",
        "--no-color",
        "input.rb",
        chdir: fixture_directory,
        env: rubocop_environment
      )
      expect([0, 1]).to include(diagnostics.status.exitstatus)
      expect(diagnostics.stderr).to eq("")
      expect(diagnostics.stdout).to eq(File.read(File.join(fixture_directory, "output.out")))

      Dir.mktmpdir("rustocop-real-fixture-") do |temporary_directory|
        FileUtils.cp(File.join(fixture_directory, "input.rb"), File.join(temporary_directory, "input.rb"))
        FileUtils.cp(File.join(fixture_directory, "rubocop.yml"), File.join(temporary_directory, "rubocop.yml"))

        corrected = run_rubocop(
          "--no-server",
          "--config",
          "rubocop.yml",
          "--autocorrect-all",
          "--format",
          "quiet",
          "input.rb",
          chdir: temporary_directory,
          env: rubocop_environment
        )
        expect([0, 1]).to include(corrected.status.exitstatus)
        expect(corrected.stderr).to eq("")
        expect(File.read(File.join(temporary_directory, "input.rb"))).to(
          eq(File.read(File.join(fixture_directory, "output.rb")))
        )
      end
    end
  end

  Dir.glob(File.join(fixture_root, "[0-9][0-9]_*/source.yml")).sort.each do |source_path|
    fixture_directory = File.dirname(source_path)
    fixture_name = File.basename(fixture_directory)

    it "matches RuboCop diagnostics and corrections for sourced fixture #{fixture_name}" do
      metadata = YAML.safe_load_file(source_path)
      expect(metadata.fetch("license")).to eq("MIT")
      expect(metadata.fetch("custom_cops")).to be(false)
      cops = metadata.fetch("cops").join(",")

      rubocop = run_rubocop(
        "--no-server", "--config", "rubocop.yml", "--format", "json",
        "--only", cops, "input.rb", chdir: fixture_directory, env: rubocop_environment
      )
      rustocop = run_rustocop(
        "--config", "rubocop.yml", "--format", "json", "--only", cops,
        "input.rb", chdir: fixture_directory
      )

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
        normalize_rubocop_report(parsed_json(rubocop))
      )

      Dir.mktmpdir("rustocop-sourced-real-fixture-") do |temporary_directory|
        %w[input.rb rubocop.yml].each do |name|
          FileUtils.cp(File.join(fixture_directory, name), File.join(temporary_directory, name))
        end
        corrected = run_rustocop(
          "--config", "rubocop.yml", "--autocorrect-all", "--format", "json",
          "--only", cops, "input.rb", chdir: temporary_directory
        )

        expect(corrected.stderr).to eq("")
        expect(corrected.status.exitstatus).to eq(0)
        expect(File.read(File.join(temporary_directory, "input.rb"))).to(
          eq(File.read(File.join(fixture_directory, "output.rb")))
        )
      end
    end
  end
end
