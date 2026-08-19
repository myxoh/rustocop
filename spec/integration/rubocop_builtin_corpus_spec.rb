# frozen_string_literal: true

require_relative "../../lib/rustocop/compatibility_status"

RSpec.describe "RuboCop built-in compatibility corpus" do
  fixture_root = File.join(ROOT, "spec", "fixtures", "rubocop_builtin_examples")
  manifest = File.readlines(File.join(fixture_root, "manifest.tsv"), chomp: true).drop(1).to_h do |line|
    directory, cop = line.split("\t", 2)
    [cop, Dir[File.join(fixture_root, directory, "*.rb")].sort]
  end
  status = Rustocop::CompatibilityStatus.load(root: ROOT)

  raise "expected 20 verified built-in cops, got #{manifest.length}" unless manifest.length == 20
  status.validate_verified!(manifest.keys, label: "compatibility corpus")
  raise "expected 500 Ruby examples, got #{manifest.values.flatten.length}" unless manifest.values.flatten.length == 500
  raise "expected 25 examples per cop" unless manifest.values.all? { |paths| paths.length == 25 }

  manifest.each do |cop, paths|
    it "matches RuboCop for #{cop} across 25 examples" do
      rubocop = run_rubocop("--cache", "false", "--format", "json", "--only", cop, *paths)
      rustocop = run_rustocop("--cache", "false", "--format", "json", "--only", cop, *paths)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
        normalize_rubocop_report(parsed_json(rubocop))
      )
    end
  end

  it "matches RuboCop across all 500 examples in one run" do
    cops = manifest.keys.join(",")
    paths = manifest.values.flatten.sort
    rubocop = run_rubocop("--cache", "false", "--format", "json", "--only", cops, *paths)
    rustocop = run_rustocop("--cache", "false", "--format", "json", "--only", cops, *paths)

    expect(rustocop.stderr).to eq("")
    expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
    expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
      normalize_rubocop_report(parsed_json(rubocop))
    )
  end

  it "matches RuboCop autocorrection for the new correctable Prism cops" do
    cops = %w[
      Lint/BooleanSymbol
      Style/CharacterLiteral
      Style/DefWithParentheses
      Style/MethodCallWithoutArgsParentheses
      Style/NilComparison
      Style/Not
      Style/RedundantArrayConstructor
      Style/RedundantFreeze
      Style/Semicolon
      Style/StringChars
      Style/UnlessElse
    ].join(",")
    source = <<~RUBY
      :true
      ?a
      def value()
        1
      end
      action()
      item == nil
      not ready
      Array([1])
      1.freeze
      first; second
      text.split("")
      unless ready
        work
      else
        wait
      end
    RUBY

    Dir.mktmpdir do |dir|
      rubocop_dir = File.join(dir, "rubocop")
      rustocop_dir = File.join(dir, "rustocop")
      FileUtils.mkdir_p([rubocop_dir, rustocop_dir])
      rubocop_path = File.join(rubocop_dir, "sample.rb")
      rustocop_path = File.join(rustocop_dir, "sample.rb")
      File.write(rubocop_path, source)
      File.write(rustocop_path, source)

      rubocop = run_rubocop("--cache", "false", "-A", "--format", "json", "--only", cops, rubocop_path)
      rustocop = run_rustocop("--cache", "false", "-A", "--format", "json", "--only", cops, rustocop_path)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(File.read(rustocop_path)).to eq(File.read(rubocop_path))
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(
        normalize_rubocop_report(parsed_json(rubocop))
      )
    end
  end
end
