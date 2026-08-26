# frozen_string_literal: true

RSpec.describe "rustocop executable" do
  it "uses the native binary by default" do
    result = run_rustocop("-V")

    expect(result.stderr).to eq("")
    expect(result.stdout).to include("rustocop native")
    expect(result.status.exitstatus).to eq(0)
  end

  it "uses the rustocop name and version" do
    result = run_rustocop("--version")

    expect(result.stderr).to eq("")
    expect(result.stdout).to eq("#{Rustocop::VERSION}\n")
    expect(result.status.exitstatus).to eq(0)
  end

  it "keeps the Ruby fallback explicit for development" do
    result = run_rustocop("-V", env: { "RUSTOCOP_DISABLE_NATIVE" => "1" })

    expect(result.stderr).to eq("")
    expect(result.stdout).to include("rustocop ruby fallback")
    expect(result.status.exitstatus).to eq(0)
  end

  it "advertises every requested non-Singulate cop" do
    expected_cops = %w[
      Bundler/OrderedGems
      Rails/DefaultScope
      Rails/FilePath
      Rails/ApplicationJob
      Rails/ReversibleMigration
      Layout/FirstHashElementIndentation
      Style/HashSyntax
      Style/KeywordParametersOrder
      Style/RedundantBegin
      Style/CaseLikeIf
      Style/ConditionalAssignment
      Style/EmptyCaseCondition
      Style/EmptyElse
      Style/GuardClause
      Style/HashLikeCase
      Style/ClassMethodsDefinitions
      Style/EndlessMethod
      Style/FrozenStringLiteralComment
      Style/TrailingCommaInArrayLiteral
      Style/TrailingCommaInArguments
      Style/TrailingCommaInHashLiteral
      Style/ItAssignment
      Style/NumberedParameters
      Style/StringLiterals
      Naming/PredicatePrefix
      Naming/AccessorMethodName
      Lint/MissingSuper
      Lint/EmptyBlock
      Lint/Debugger
      Security/Eval
      Security/JSONLoad
      Security/MarshalLoad
      Security/Open
      Security/IoMethods
      RSpec/NestedGroups
      RSpec/EmptyExampleGroup
      RSpec/MessageChain
      RSpec/MultipleExpectations
      RSpec/ExampleLength
      RSpec/VariableName
      RSpec/MultipleMemoizedHelpers
      RSpec/Focus
      RSpec/PendingWithoutReason
      RSpec/ScatteredSetup
      RSpec/SpecFilePathSuffix
      RSpec/SpecFilePathFormat
    ]

    result = run_rustocop("--show-cops")
    advertised_cops = result.stdout.lines.map(&:strip)

    expect(result.stderr).to eq("")
    expect(result.status.exitstatus).to eq(0)
    expect(advertised_cops).to include(*expected_cops)
    expect(advertised_cops.grep(%r{\ASingulateCops/})).to be_empty
  end

  it "preserves output order when inspecting files in parallel" do
    Dir.mktmpdir("rustocop-parallel-order-") do |directory|
      files = 10.times.map do |index|
        File.join(directory, format("%02d.rb", index)).tap { |path| File.write(path, "value_#{index} = ()\n") }
      end
      arguments = ["--format", "json", "--only", "Lint/EmptyExpression", *files]

      sequential = run_rustocop(*arguments)
      parallel = run_rustocop("--jobs", "4", *arguments)

      expect(parallel.stderr).to eq("")
      expect(parallel.status.exitstatus).to eq(sequential.status.exitstatus)
      expect(parallel.stdout).to eq(sequential.stdout)
    end
  end

  it "discovers RuboCop configuration and runs the same enabled base cops" do
    Dir.mktmpdir("rustocop-effective-config") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          DisabledByDefault: true
          NewCops: disable

        Layout/TrailingWhitespace:
          Enabled: true

        Style/StringLiterals:
          Enabled: false
      YAML
      path = File.join(directory, "example.rb")
      File.write(path, "'example'  \n")

      rustocop = run_rustocop("--format", "json", path, chdir: directory)
      fallback = run_rustocop(
        "--format", "json", path, chdir: directory, env: { "RUSTOCOP_DISABLE_NATIVE" => "1" }
      )
      rubocop = run_rubocop("--format", "json", path, chdir: directory)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
      expect(normalize_rubocop_report(parsed_json(fallback))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
    end
  end

  it "uses inherited effective cop settings for explicit native selections" do
    Dir.mktmpdir("rustocop-inherited-config") do |directory|
      File.write(File.join(directory, "base.yml"), <<~YAML)
        AllCops:
          DisabledByDefault: true
          NewCops: disable

        Style/StringLiterals:
          Enabled: true
          EnforcedStyle: double_quotes
      YAML
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        inherit_from: base.yml
      YAML
      path = File.join(directory, "example.rb")
      File.write(path, "'example'\n")

      rustocop = run_rustocop(
        "--format", "json", "--only", "Style/StringLiterals", path, chdir: directory
      )
      rubocop = run_rubocop(
        "--format", "json", "--only", "Style/StringLiterals", path, chdir: directory
      )

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(parsed_json(rustocop).dig("summary", "offense_count")).to eq(1)
      expect(parsed_json(rubocop).dig("summary", "offense_count")).to eq(1)
      expect(parsed_json(rustocop).fetch("files").flat_map { |file| file.fetch("offenses") })
        .to contain_exactly(include("cop_name" => "Style/StringLiterals"))
    end
  end

  it "keeps resolved path exclusions relative to the discovered config" do
    Dir.mktmpdir("rustocop-resolved-paths") do |directory|
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        AllCops:
          DisabledByDefault: true
          NewCops: disable
          Exclude:
            - excluded.rb

        Layout/TrailingWhitespace:
          Enabled: true
      YAML
      File.write(File.join(directory, "included.rb"), "puts :included  \n")
      File.write(File.join(directory, "excluded.rb"), "puts :excluded  \n")

      rustocop = run_rustocop("--format", "json", directory, chdir: directory)
      rubocop = run_rubocop("--format", "json", directory, chdir: directory)

      expect(rustocop.stderr).to eq("")
      expect(rustocop.status.exitstatus).to eq(rubocop.status.exitstatus)
      expect(normalize_rubocop_report(parsed_json(rustocop)))
        .to eq(normalize_rubocop_report(parsed_json(rubocop)))
      expect(parsed_json(rustocop).fetch("files").map { |file| File.basename(file.fetch("path")) })
        .to eq(["included.rb"])
    end
  end

  it "warns about configured non-native cops and delegates them only when requested" do
    Dir.mktmpdir("rustocop-non-native-config") do |directory|
      custom_cop = File.join(ROOT, "benchmark/custom_cops/synthetic_file_header.rb")
      File.write(File.join(directory, ".rubocop.yml"), <<~YAML)
        require:
          - #{custom_cop.inspect}

        AllCops:
          DisabledByDefault: true
          NewCops: disable

        Custom/SyntheticFileHeader:
          Enabled: true
      YAML
      path = File.join(directory, "example.rb")
      File.write(path, "puts :ok\n")

      ignored = run_rustocop("--format", "json", path, chdir: directory)
      included = run_rustocop(
        "--included-non-native-cops", "--format", "json", path, chdir: directory
      )

      expect(ignored.stderr).to eq("#{Rustocop::RubocopConfiguration::WARNING}\n")
      expect(parsed_json(ignored).dig("summary", "offense_count")).to eq(0)
      expect(included.stderr).to eq("")
      expect(parsed_json(included).fetch("files").flat_map { |file| file.fetch("offenses") })
        .to contain_exactly(include("cop_name" => "Custom/SyntheticFileHeader"))
    end
  end

  it "rejects an invalid parallel worker count" do
    result = run_rustocop("--jobs", "0", "--stdin", "example.rb", stdin: "puts :ok\n")

    expect(result.stderr).to eq("rustocop: invalid worker count 0\n")
    expect(result.status.exitstatus).to eq(2)
  end

  it "inspects endless methods without panicking in the legacy lint pass" do
    result = run_rustocop(
      "--format", "json", "--only", "Lint/UnusedMethodArgument", "--stdin", "example.rb",
      stdin: "def example(unused) = :ok\n"
    )

    expect(result.status.exitstatus).to eq(1)
    expect(parsed_json(result).fetch("files").flat_map { |file| file.fetch("offenses") })
      .to contain_exactly(include("cop_name" => "Lint/UnusedMethodArgument"))
  end

  it "delegates required custom cops while keeping built-in cops native" do
    Dir.mktmpdir("rustocop-mixed-cops-") do |directory|
      files = 3.times.map do |index|
        File.join(directory, format("%02d.rb", index)).tap { |path| File.write(path, "value_#{index} = ()\n") }
      end
      custom_cop = File.join(ROOT, "benchmark/custom_cops/synthetic_file_header.rb")
      config = File.join(ROOT, "benchmark/custom-cop-rubocop.yml")
      arguments = [
        "--no-parallel", "--format", "json", "--config", config,
        "--require", custom_cop, "--only", "Lint/EmptyExpression,Custom/SyntheticFileHeader", *files
      ]

      mixed = run_rustocop(*arguments)
      rubocop = run_rubocop(*arguments.reject { |argument| argument == "--no-parallel" }, "--no-server")

      expect(mixed.stderr).to eq("")
      expect(mixed.status.exitstatus).to eq(1)
      expect(normalize_rubocop_report(parsed_json(mixed))).to eq(normalize_rubocop_report(parsed_json(rubocop)))
    end
  end

  it "rejects autocorrection when native and custom cops are mixed" do
    result = run_rustocop(
      "-A", "--require", File.join(ROOT, "benchmark/custom_cops/synthetic_file_header.rb"),
      "--only", "Lint/EmptyExpression,Custom/SyntheticFileHeader", __FILE__
    )

    expect(result.stderr).to eq("rustocop: mixed custom-cop runs do not yet support autocorrection\n")
    expect(result.status.exitstatus).to eq(2)
  end

  it "autocorrects distinct files safely in parallel" do
    Dir.mktmpdir("rustocop-parallel-correction") do |directory|
      files = 6.times.map do |index|
        File.join(directory, "example_#{index}.rb").tap { |path| File.write(path, "{key:1}\n") }
      end

      result = run_rustocop("-A", "--jobs", "4", "--only", "Layout/SpaceAfterColon", *files)

      expect(result.stderr).to eq("")
      expect(result.status.exitstatus).to eq(0)
      expect(files.map { |path| File.read(path) }).to all(eq("{key: 1}\n"))
    end
  end
end
