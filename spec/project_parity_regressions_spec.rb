# frozen_string_literal: true

RSpec.describe "real-project parity regressions" do
  expected_cops = %w[
    Layout/EmptyLineAfterMagicComment
    Lint/DuplicateSetElement
    Lint/MixedRegexpCaptureTypes
    Lint/OrderedMagicComments
    Metrics/CollectionLiteralLength
    Style/DirEmpty
    Style/EnvHome
    Style/FileEmpty
    Style/FileOpen
    Style/MapCompactWithConditionalBlock
    Style/MapToSet
    Style/MultilineIfModifier
    Style/MutableConstant
    Style/RedundantCapitalW
    Style/RedundantSelfAssignmentBranch
    Layout/SpaceAroundMethodCallOperator
    Lint/ConstantOverwrittenInRescue
    Lint/RefinementImportMethods
    Security/CompoundHash
    Style/InfiniteLoop
    Style/CombinableDefined
    Style/EmptyHeredoc
    Style/TrailingCommaInBlockArgs
    Lint/UnmodifiedReduceAccumulator
    Style/InverseMethods
    Style/NegatedWhile
    Style/UnlessLogicalOperators
    Gemspec/AddRuntimeDependency
    Lint/DuplicateElsifCondition
    Lint/DuplicateMatchPattern
    Style/EmptyBlockParameter
    Lint/SafeNavigationWithEmpty
    Lint/SuppressedExceptionInNumberConversion
    Lint/ToEnumArguments
    Lint/UnescapedBracketInRegexp
    Style/NegatedUnless
    Style/NestedTernaryOperator
    Style/OpenStructUse
    Style/ParenthesesAroundCondition
    Style/RedundantInterpolationUnfreeze
    Style/RedundantPercentQ
    Style/RedundantStructKeywordInit
    Style/StringHashKeys
    Gemspec/DeprecatedAttributeAssignment
    Style/ClassMethods
    Style/MapToHash
    Style/OperatorMethodCall
    Style/OrAssignment
    Style/RedundantInitialize
    Style/StringLiteralsInInterpolation
    Lint/BinaryOperatorWithIdenticalOperands
    Lint/NestedPercentLiteral
    Lint/NumberConversion
    Lint/UnreachableLoop
    Style/Dir
    Style/LineEndConcatenation
    Style/MissingRespondToMissing
    Style/CaseEquality
    Style/BlockComments
  ]
  fixture_root = File.join(ROOT, "spec", "fixtures", "project_parity_regressions")
  config = File.join(fixture_root, "rubocop.yml")
  native = File.join(ROOT, "crates", "rustocop", "target", "release", "rustocop")
  rows = File.readlines(File.join(fixture_root, "manifest.tsv"), chomp: true).drop(1).map do |line|
    line.split("\t", 5)
  end

  raise "unexpected project parity regression manifest" unless rows.map(&:first) == expected_cops

  rows.each do |cop, file, repository, revision, source_path|
    it "matches RuboCop for #{cop} from #{repository}@#{revision}:#{source_path}" do
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
