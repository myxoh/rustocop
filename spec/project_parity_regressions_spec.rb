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
    Lint/RequireParentheses
    Style/EmptyCaseCondition
    Lint/MissingSuper
    Style/EmptyLambdaParameter
    Style/PartitionInsteadOfDoubleSelect
    Style/SymbolProc
    Lint/ScriptPermission
    Lint/UriRegexp
    Layout/HeredocArgumentClosingParenthesis
    Style/ClassCheck
    Style/PercentQLiterals
    Lint/DeprecatedOpenSSLConstant
    Style/KeywordParametersOrder
    Style/Proc
    Style/RedundantRegexpCharacterClass
    Gemspec/RequireMFA
    Layout/ConditionPosition
    Lint/ParenthesesAsGroupedExpression
    Lint/RedundantRegexpQuantifiers
    Style/Alias
    Style/ClassMethodsDefinitions
    Lint/SelfAssignment
    Style/BitwisePredicate
    Style/FileRead
    Style/AmbiguousEndlessMethodDefinition
    Style/HashLookupMethod
    Style/IfWithBooleanLiteralBranches
    Style/RedundantConditional
    Lint/EmptyConditionalBody
    Style/DateTime
    Layout/TrailingEmptyLines
    Style/ClassEqualityComparison
    Style/NumericLiterals
    Style/RequireOrder
    Lint/SuppressedException
    Style/RedundantBegin
    Style/SafeNavigationChainLength
    Lint/UnreachableCode
    Lint/DuplicateMethods
    Gemspec/AddRuntimeDependency
    Layout/IndentationStyle
    Layout/LeadingCommentSpace
    Layout/LineContinuationSpacing
    Layout/SpaceAroundKeyword
    Layout/SpaceAroundMethodCallOperator
    Layout/SpaceInsideParens
    Lint/AmbiguousAssignment
    Layout/LeadingCommentSpace
    Layout/SpaceAroundKeyword
    Layout/SpaceAroundMethodCallOperator
    Layout/SpaceInsideParens
    Lint/AmbiguousAssignment
    Layout/SpaceAroundKeyword
    Layout/SpaceAroundKeyword
    Layout/SpaceAroundMethodCallOperator
    Layout/SpaceInsideParens
    Layout/SpaceAroundMethodCallOperator
    Layout/SpaceAroundKeyword
    Layout/SpaceAroundKeyword
    Lint/UnusedBlockArgument
    Naming/AccessorMethodName
    Naming/BlockForwarding
    Naming/HeredocDelimiterCase
    Naming/MethodParameterName
    Naming/RescuedExceptionsVariableName
    Layout/FirstHashElementIndentation
    Style/CombinableLoops
    Style/HashLikeCase
    Style/NumberedParameters
    Style/MultilineMethodSignature
    Style/EndlessMethod
    Layout/SpaceAfterMethodName
    Lint/RequireRelativeSelfPath
    Naming/PredicatePrefix
    Gemspec/RubyVersionGlobalsUsage
    Bundler/GemVersion
    Gemspec/DuplicatedAssignment
    Gemspec/RequiredRubyVersion
    Layout/ArrayAlignment
    Layout/ArrayAlignment
    Layout/ClassStructure
    Layout/ClassStructure
    Layout/HashAlignment
    Layout/HashAlignment
    Layout/SpaceAroundOperators
    Layout/SpaceAroundOperators
    Layout/SpaceInsideArrayLiteralBrackets
    Layout/SpaceInsideArrayLiteralBrackets
    Layout/SpaceInsideReferenceBrackets
    Layout/SpaceInsideReferenceBrackets
    Lint/ConstantReassignment
    Lint/ConstantReassignment
    Lint/DuplicateBranch
    Lint/DuplicateMethods
    Lint/FormatParameterMismatch
    Lint/NonAtomicFileOperation
    Lint/OutOfRangeRegexpRef
    Lint/RedundantSplatExpansion
    Lint/RedundantSplatExpansion
    Lint/SafeNavigationChain
    Lint/SafeNavigationChain
    Lint/SafeNavigationConsistency
    Lint/SafeNavigationConsistency
    Lint/ShadowedArgument
    Lint/ShadowedArgument
    Lint/ShadowingOuterLocalVariable
    Lint/ShadowingOuterLocalVariable
    Lint/UselessAccessModifier
    Metrics/ModuleLength
    Metrics/ModuleLength
    Naming/InclusiveLanguage
    Naming/MethodName
    Naming/MethodName
    Style/AndOr
    Style/ArgumentsForwarding
    Style/ArgumentsForwarding
    Style/ArrayIntersect
    Style/ArrayIntersect
    Style/BlockDelimiters
    Style/BlockDelimiters
    Style/ClassAndModuleChildren
    Style/ClassAndModuleChildren
    Style/EmptyElse
    Style/EmptyElse
    Style/KeywordArgumentsMerging
    Style/ArrayFirstLast
    Style/ArrayFirstLast
    Style/EmptyStringInsideInterpolation
    Style/CaseLikeIf
    Style/CaseLikeIf
    Style/CombinableLoops
    Style/CombinableLoops
    Layout/EmptyLineAfterMultilineCondition
    Lint/PercentStringArray
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
