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
      Metrics/BlockLength
      Metrics/MethodLength
      Metrics/AbcSize
      Layout/LineLength
      Layout/ExtraSpacing
      Layout/EndAlignment
      Layout/FirstHashElementIndentation
      Layout/IndentationConsistency
      Layout/IndentationWidth
      Style/HashSyntax
      Style/KeywordParametersOrder
      Style/RedundantBegin
      Style/IfUnlessModifier
      Style/CaseLikeIf
      Style/ConditionalAssignment
      Style/EmptyCaseCondition
      Style/EmptyElse
      Style/GuardClause
      Style/Documentation
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
      Lint/UnusedMethodArgument
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
    files = Dir[File.join(ROOT, "spec/fixtures/rubocop_builtin_examples/lint_empty_expression/*.rb")].first(10)
    arguments = ["--format", "json", "--only", "Lint/EmptyExpression", *files]

    sequential = run_rustocop(*arguments)
    parallel = run_rustocop("--jobs", "4", *arguments)

    expect(parallel.stderr).to eq("")
    expect(parallel.status.exitstatus).to eq(sequential.status.exitstatus)
    expect(parallel.stdout).to eq(sequential.stdout)
  end

  it "rejects an invalid parallel worker count" do
    result = run_rustocop("--jobs", "0", "--stdin", "example.rb", stdin: "puts :ok\n")

    expect(result.stderr).to eq("rustocop: invalid worker count 0\n")
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
