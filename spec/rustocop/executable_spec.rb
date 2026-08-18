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
end
