# Current project-parity gaps

Generated from the same complete audit as [the evidence matrix](cop-support.md).
Evidence updated at `2026-08-24T15:25:53-04:00`.
This queue contains only failures from that 50-project audit; it does not use the old
Verified/Heuristic qualification labels or captured-case pass counts.

The gap is unmatched complete diagnostic signatures, not the difference
between aggregate offense counts. A cop can have equal counts and still have a
nonzero gap because its message, severity, path, or source range differs.

- Rust source: `commit ddb32ffcc4aaa97d560add1482e3c33863409004`
- Unresolved cops: 236

| Cop | Status | Rustocop | RuboCop | Exact | Signature gap | Project regression evidence |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `Lint/ConstantResolution` | Mismatch | 464 | 849583 | 83 | 849881 | Pending unit |
| `Style/InlineComment` | Mismatch | 229984 | 18651 | 18651 | 211333 | Project-derived |
| `Metrics/AbcSize` | Mismatch | 60904 | 55118 | 1448 | 113126 | Pending unit |
| `Layout/MultilineMethodCallIndentation` | Mismatch | 21838 | 15130 | 9318 | 18332 | Project-derived + pending |
| `Style/DocumentationMethod` | Mismatch | 184728 | 186456 | 176574 | 18036 | Pending unit |
| `Layout/LineLength` | Mismatch | 187509 | 181525 | 178339 | 12356 | Project-derived + pending |
| `Style/Copyright` | Mismatch | 74188 | 85417 | 74171 | 11263 | Pending unit |
| `Layout/MultilineArrayLineBreaks` | Mismatch | 13345 | 22550 | 13010 | 9875 | Project-derived + pending |
| `Style/MissingElse` | Mismatch | 32260 | 40883 | 32220 | 8703 | Pending unit |
| `Metrics/MethodLength` | Mismatch | 36630 | 36690 | 32344 | 8632 | Pending unit |
| `Lint/UnusedMethodArgument` | Mismatch | 11152 | 3598 | 3284 | 8182 | No |
| `Style/MethodCallWithArgsParentheses` | Mismatch | 844894 | 838160 | 837975 | 7104 | Pending unit |
| `Style/Documentation` | Mismatch | 54391 | 49775 | 48846 | 6474 | No |
| `Naming/BlockParameterName` | Mismatch | 5996 | 14 | 7 | 5996 | No |
| `Metrics/PerceivedComplexity` | Mismatch | 6079 | 6450 | 3380 | 5769 | No |
| `Metrics/CyclomaticComplexity` | Mismatch | 7318 | 8047 | 5292 | 4781 | No |
| `Naming/VariableNumber` | Mismatch | 13082 | 9963 | 9545 | 3955 | No |
| `Lint/DuplicateRegexpCharacterClassElement` | Mismatch | 3770 | 103 | 99 | 3675 | No |
| `Layout/MultilineOperationIndentation` | Mismatch | 4362 | 2263 | 1629 | 3367 | Project-derived |
| `Lint/AmbiguousRegexpLiteral` | Mismatch | 3450 | 722 | 722 | 2728 | No |
| `Metrics/BlockLength` | Mismatch | 86055 | 86297 | 84997 | 2358 | No |
| `Layout/ExtraSpacing` | Mismatch | 1329 | 1905 | 471 | 2292 | Project-derived |
| `Metrics/ClassLength` | Mismatch | 5030 | 5020 | 4038 | 1974 | No |
| `Lint/MissingCopEnableDirective` | Mismatch | 2024 | 98 | 75 | 1972 | Project-derived |
| `Layout/LineEndStringConcatenationIndentation` | Mismatch | 3171 | 2646 | 1979 | 1859 | No |
| `Layout/IndentationWidth` | Mismatch | 8652 | 8652 | 7902 | 1500 | Project-derived |
| `Lint/UnderscorePrefixedVariableName` | Mismatch | 1036 | 2109 | 897 | 1351 | No |
| `Lint/UselessAssignment` | Mismatch | 1349 | 1313 | 696 | 1270 | No |
| `Naming/VariableName` | Mismatch | 1513 | 530 | 530 | 983 | No |
| `Lint/UselessConstantScoping` | Mismatch | 958 | 529 | 318 | 851 | No |
| `Style/IfUnlessModifier` | Mismatch | 11199 | 11656 | 11179 | 497 | Project-derived |
| `Style/AutoResourceCleanup` | Mismatch | 548 | 86 | 86 | 462 | Project-derived |
| `Lint/AmbiguousRange` | Mismatch | 456 | 164 | 98 | 424 | Project-derived |
| `Layout/RedundantLineBreak` | Mismatch | 31715 | 31577 | 31471 | 350 | No |
| `Style/RedundantParentheses` | Mismatch | 712 | 522 | 480 | 274 | Project-derived |
| `Metrics/BlockNesting` | Mismatch | 631 | 663 | 538 | 218 | No |
| `Style/FetchEnvVar` | Mismatch | 1497 | 1386 | 1340 | 203 | Project-derived |
| `Naming/FileName` | Mismatch | 469 | 469 | 369 | 200 | Project-derived |
| `Naming/HeredocDelimiterNaming` | Mismatch | 2356 | 2184 | 2184 | 172 | Project-derived |
| `Layout/MultilineAssignmentLayout` | Mismatch | 17262 | 17367 | 17234 | 161 | Project-derived |
| `Lint/RedundantCopEnableDirective` | Mismatch | 176 | 20 | 20 | 156 | Project-derived |
| `Style/ExplicitBlockArgument` | Mismatch | 282 | 357 | 248 | 143 | Project-derived |
| `Layout/SpaceAroundOperators` | Mismatch | 7021 | 7084 | 6988 | 129 | Project-derived |
| `Style/EmptyLiteral` | Mismatch | 240 | 118 | 116 | 126 | Project-derived |
| `Style/RedundantAssignment` | Mismatch | 249 | 201 | 176 | 98 | Project-derived |
| `Style/StringConcatenation` | Mismatch | 3175 | 3080 | 3080 | 95 | Project-derived |
| `Layout/RescueEnsureAlignment` | Mismatch | 455 | 372 | 372 | 83 | Project-derived |
| `Style/EvalWithLocation` | Mismatch | 529 | 504 | 483 | 67 | Project-derived |
| `Style/AccessorGrouping` | Mismatch | 2024 | 1978 | 1968 | 66 | Project-derived |
| `Lint/OrAssignmentToConstant` | Mismatch | 159 | 172 | 136 | 59 | No |
| `Layout/SpaceInsideParens` | Mismatch | 959 | 1012 | 958 | 55 | Project-derived |
| `Lint/RescueException` | Mismatch | 240 | 191 | 191 | 49 | Project-derived |
| `Gemspec/DependencyVersion` | Mismatch | 45 | 0 | 0 | 45 | No |
| `Style/FormatStringToken` | Mismatch | 7662 | 7619 | 7618 | 45 | Project-derived |
| `Lint/ErbNewArguments` | Mismatch | 46 | 8 | 8 | 38 | No |
| `Gemspec/AttributeAssignment` | Mismatch | 37 | 0 | 0 | 37 | No |
| `Style/Semicolon` | Mismatch | 2726 | 2692 | 2692 | 34 | No |
| `Layout/ClosingParenthesisIndentation` | Mismatch | 341 | 372 | 340 | 33 | No |
| `Style/ItBlockParameter` | Mismatch | 209 | 240 | 208 | 33 | No |
| `Style/StringHashKeys` | Mismatch | 131615 | 131591 | 131587 | 32 | Project-derived |
| `Style/ClassMethods` | Mismatch | 36 | 45 | 25 | 31 | Project-derived |
| `Lint/SelfAssignment` | Mismatch | 48 | 18 | 18 | 30 | Project-derived |
| `Lint/SuppressedExceptionInNumberConversion` | Mismatch | 34 | 4 | 4 | 30 | Project-derived |
| `Style/CombinableLoops` | Mismatch | 118 | 94 | 91 | 30 | Project-derived |
| `Style/RedundantFreeze` | Mismatch | 159 | 187 | 159 | 28 | No |
| `Layout/FirstParameterIndentation` | Mismatch | 24 | 5 | 1 | 27 | No |
| `Style/EmptyHeredoc` | Mismatch | 21 | 18 | 6 | 27 | Project-derived |
| `Style/EndlessMethod` | Mismatch | 29 | 3 | 3 | 26 | Project-derived |
| `Metrics/ParameterLists` | Mismatch | 1569 | 1558 | 1551 | 25 | Project-derived |
| `Style/EnvHome` | Mismatch | 16 | 33 | 12 | 25 | Project-derived |
| `Layout/HeredocArgumentClosingParenthesis` | Mismatch | 58 | 34 | 34 | 24 | Project-derived |
| `Style/CommentAnnotation` | Mismatch | 564 | 544 | 543 | 22 | No |
| `Style/RedundantCapitalW` | Mismatch | 55 | 35 | 34 | 22 | Project-derived |
| `Metrics/ModuleLength` | Mismatch | 1210 | 1201 | 1195 | 21 | Project-derived |
| `Style/MultilineBlockChain` | Mismatch | 296 | 275 | 275 | 21 | Project-derived |
| `Style/CaseLikeIf` | Mismatch | 60 | 52 | 46 | 20 | Project-derived |
| `Style/SpecialGlobalVars` | Mismatch | 539 | 519 | 519 | 20 | No |
| `Layout/EmptyLinesAroundExceptionHandlingKeywords` | Mismatch | 207 | 188 | 188 | 19 | Project-derived |
| `Lint/NonAtomicFileOperation` | Mismatch | 258 | 259 | 249 | 19 | Project-derived |
| `Migration/DepartmentName` | Mismatch | 18 | 1 | 0 | 19 | No |
| `Naming/RescuedExceptionsVariableName` | Mismatch | 1178 | 1195 | 1177 | 19 | Project-derived |
| `Lint/SharedMutableDefault` | Mismatch | 23 | 11 | 8 | 18 | No |
| `Lint/ShadowedException` | Mismatch | 22 | 39 | 22 | 17 | Project-derived |
| `Layout/IndentationStyle` | Mismatch | 7 | 22 | 7 | 15 | Project-derived |
| `Lint/UnreachableLoop` | Mismatch | 622 | 629 | 618 | 15 | Project-derived |
| `Layout/SpaceAfterMethodName` | Mismatch | 15 | 1 | 1 | 14 | Project-derived |
| `Style/MapIntoArray` | Mismatch | 119 | 129 | 117 | 14 | Project-derived |
| `Layout/SpaceInsideArrayPercentLiteral` | Mismatch | 121 | 112 | 110 | 13 | Project-derived |
| `Lint/DuplicateElsifCondition` | Mismatch | 13 | 0 | 0 | 13 | Project-derived |
| `Lint/UnreachablePatternBranch` | Mismatch | 13 | 0 | 0 | 13 | Project-derived |
| `Style/EachForSimpleLoop` | Mismatch | 13 | 0 | 0 | 13 | Project-derived |
| `Style/EmptyBlockParameter` | Mismatch | 15 | 4 | 3 | 13 | Project-derived |
| `Style/IdenticalConditionalBranches` | Mismatch | 159 | 172 | 159 | 13 | Project-derived |
| `Style/MultilineIfModifier` | Mismatch | 419 | 406 | 406 | 13 | Project-derived |
| `Layout/FirstArrayElementIndentation` | Mismatch | 3443 | 3431 | 3431 | 12 | Project-derived |
| `Lint/NumericOperationWithConstantResult` | Mismatch | 12 | 0 | 0 | 12 | No |
| `Style/ArgumentsForwarding` | Mismatch | 7602 | 7614 | 7602 | 12 | Project-derived |
| `Style/BlockDelimiters` | Mismatch | 7806 | 7814 | 7804 | 12 | Project-derived |
| `Style/MutableConstant` | Mismatch | 2214 | 2202 | 2202 | 12 | Project-derived |
| `Lint/UselessDefined` | Mismatch | 14 | 3 | 3 | 11 | Project-derived |
| `Style/DirEmpty` | Mismatch | 12 | 1 | 1 | 11 | Project-derived |
| `Style/RedundantStringEscape` | Mismatch | 1864 | 1875 | 1864 | 11 | No |
| `Lint/BigDecimalNew` | Mismatch | 10 | 0 | 0 | 10 | No |
| `Lint/ConstantOverwrittenInRescue` | Mismatch | 10 | 0 | 0 | 10 | Project-derived |
| `Style/ColonMethodDefinition` | Mismatch | 11 | 1 | 1 | 10 | Project-derived |
| `Style/SlicingWithRange` | Mismatch | 309 | 309 | 304 | 10 | No |
| `Layout/EmptyLineAfterGuardClause` | Mismatch | 4959 | 4964 | 4957 | 9 | Project-derived |
| `Layout/SpaceAroundKeyword` | Mismatch | 237 | 232 | 230 | 9 | Project-derived |
| `Style/SafeNavigation` | Mismatch | 1176 | 1167 | 1167 | 9 | Project-derived |
| `Layout/EmptyComment` | Mismatch | 105 | 99 | 98 | 8 | Project-derived |
| `Layout/EmptyLinesAfterModuleInclusion` | Mismatch | 1032 | 1024 | 1024 | 8 | Project-derived |
| `Layout/SpaceBeforeFirstArg` | Mismatch | 61 | 61 | 57 | 8 | No |
| `Lint/EmptyBlock` | Mismatch | 1633 | 1639 | 1632 | 8 | Project-derived |
| `Lint/EnsureReturn` | Mismatch | 7 | 1 | 0 | 8 | No |
| `Naming/InclusiveLanguage` | Mismatch | 476 | 480 | 474 | 8 | Project-derived |
| `Style/CombinableDefined` | Mismatch | 11 | 3 | 3 | 8 | Project-derived |
| `Style/PercentQLiterals` | Mismatch | 74 | 82 | 74 | 8 | Project-derived |
| `Layout/BlockAlignment` | Mismatch | 218 | 215 | 213 | 7 | Project-derived |
| `Lint/DuplicateSetElement` | Mismatch | 9 | 2 | 2 | 7 | Project-derived |
| `Lint/LambdaWithoutLiteralBlock` | Mismatch | 7 | 0 | 0 | 7 | No |
| `Style/OptionalArguments` | Mismatch | 23 | 16 | 16 | 7 | No |
| `Style/SymbolProc` | Mismatch | 1420 | 1413 | 1413 | 7 | Project-derived |
| `Style/TrivialAccessors` | Mismatch | 151 | 152 | 148 | 7 | No |
| `Lint/EmptyEnsure` | Mismatch | 6 | 0 | 0 | 6 | Project-derived |
| `Lint/RefinementImportMethods` | Mismatch | 6 | 0 | 0 | 6 | Project-derived |
| `Style/FrozenStringLiteralComment` | Mismatch | 14123 | 14117 | 14117 | 6 | No |
| `Style/MissingRespondToMissing` | Mismatch | 99 | 93 | 93 | 6 | Project-derived |
| `Style/RedundantLineContinuation` | Mismatch | 84 | 88 | 83 | 6 | Project-derived |
| `Layout/FirstArgumentIndentation` | Mismatch | 878 | 883 | 878 | 5 | Project-derived |
| `Lint/AmbiguousOperator` | Mismatch | 337 | 332 | 332 | 5 | Project-derived |
| `Lint/EachWithObjectArgument` | Mismatch | 5 | 0 | 0 | 5 | No |
| `Lint/RequireRangeParentheses` | Mismatch | 5 | 0 | 0 | 5 | Project-derived |
| `Lint/ShadowingOuterLocalVariable` | Mismatch | 385 | 386 | 383 | 5 | Project-derived |
| `Lint/UselessMethodDefinition` | Mismatch | 77 | 76 | 74 | 5 | Project-derived |
| `Style/ClassEqualityComparison` | Mismatch | 73 | 72 | 70 | 5 | Project-derived |
| `Style/DoubleCopDisableDirective` | Mismatch | 5 | 0 | 0 | 5 | No |
| `Style/GuardClause` | Mismatch | 4326 | 4327 | 4324 | 5 | Project-derived |
| `Style/InPatternThen` | Mismatch | 5 | 0 | 0 | 5 | No |
| `Style/InfiniteLoop` | Mismatch | 545 | 546 | 543 | 5 | Project-derived |
| `Style/ReturnNil` | Mismatch | 1976 | 1971 | 1971 | 5 | Project-derived |
| `Style/WordArray` | Mismatch | 3510 | 3513 | 3509 | 5 | No |
| `Layout/AssignmentIndentation` | Mismatch | 18 | 22 | 18 | 4 | Project-derived |
| `Lint/AmbiguousOperatorPrecedence` | Mismatch | 1011 | 1007 | 1007 | 4 | Project-derived |
| `Lint/DuplicateMagicComment` | Mismatch | 4 | 0 | 0 | 4 | No |
| `Lint/NestedMethodDefinition` | Mismatch | 117 | 113 | 113 | 4 | No |
| `Lint/UselessRescue` | Mismatch | 6 | 10 | 6 | 4 | No |
| `Naming/BinaryOperatorParameterName` | Mismatch | 184 | 186 | 183 | 4 | No |
| `Style/Alias` | Mismatch | 1715 | 1719 | 1715 | 4 | Project-derived |
| `Style/ArrayIntersectWithSingleElement` | Mismatch | 5 | 1 | 1 | 4 | No |
| `Style/DisableCopsWithinSourceCodeDirective` | Mismatch | 11439 | 11435 | 11435 | 4 | Project-derived |
| `Style/NumberedParameters` | Mismatch | 6 | 8 | 5 | 4 | Project-derived |
| `Style/SuperArguments` | Mismatch | 447 | 443 | 443 | 4 | No |
| `Bundler/InsecureProtocolSource` | Mismatch | 3 | 0 | 0 | 3 | No |
| `Layout/EmptyLinesAroundAccessModifier` | Mismatch | 1767 | 1764 | 1764 | 3 | Project-derived |
| `Layout/LineContinuationLeadingSpace` | Mismatch | 114 | 117 | 114 | 3 | No |
| `Layout/SpaceInsideArrayLiteralBrackets` | Mismatch | 3115 | 3118 | 3115 | 3 | Project-derived |
| `Lint/ConstantDefinitionInBlock` | Mismatch | 886 | 883 | 883 | 3 | Project-derived |
| `Lint/DisjunctiveAssignmentInConstructor` | Mismatch | 8 | 5 | 5 | 3 | Project-derived |
| `Lint/DuplicateBranch` | Mismatch | 396 | 397 | 395 | 3 | Project-derived |
| `Lint/HeredocMethodCallPosition` | Mismatch | 0 | 3 | 0 | 3 | Project-derived |
| `Lint/LiteralInInterpolation` | Mismatch | 48 | 45 | 45 | 3 | Project-derived |
| `Lint/MissingSuper` | Mismatch | 842 | 839 | 839 | 3 | Project-derived |
| `Lint/ToEnumArguments` | Mismatch | 7 | 10 | 7 | 3 | Project-derived |
| `Lint/UriEscapeUnescape` | Mismatch | 0 | 3 | 0 | 3 | Project-derived |
| `Lint/UselessNumericOperation` | Mismatch | 4 | 1 | 1 | 3 | No |
| `Naming/BlockForwarding` | Mismatch | 4855 | 4852 | 4852 | 3 | Project-derived |
| `Security/CompoundHash` | Mismatch | 29 | 26 | 26 | 3 | Project-derived |
| `Style/AsciiComments` | Mismatch | 4699 | 4696 | 4696 | 3 | Project-derived |
| `Style/BlockComments` | Mismatch | 38 | 41 | 38 | 3 | Project-derived |
| `Style/HashSyntax` | Mismatch | 19083 | 19082 | 19081 | 3 | No |
| `Style/LineEndConcatenation` | Mismatch | 655 | 658 | 655 | 3 | Project-derived |
| `Style/MultipleComparison` | Mismatch | 390 | 391 | 389 | 3 | Project-derived |
| `Style/OperatorMethodCall` | Mismatch | 3 | 6 | 3 | 3 | Project-derived |
| `Style/RedundantBegin` | Mismatch | 459 | 458 | 457 | 3 | Project-derived |
| `Style/RedundantInitialize` | Mismatch | 16 | 17 | 15 | 3 | Project-derived |
| `Layout/ClassStructure` | Mismatch | 2880 | 2878 | 2878 | 2 | Project-derived |
| `Layout/EmptyLineAfterMagicComment` | Mismatch | 1192 | 1194 | 1192 | 2 | Project-derived |
| `Layout/EmptyLinesAroundAttributeAccessor` | Mismatch | 242 | 240 | 240 | 2 | Project-derived |
| `Lint/DuplicateMethods` | Mismatch | 112 | 114 | 112 | 2 | Project-derived |
| `Lint/FloatComparison` | Mismatch | 121 | 119 | 119 | 2 | No |
| `Lint/NumberConversion` | Mismatch | 10806 | 10808 | 10806 | 2 | Project-derived |
| `Lint/RedundantRegexpQuantifiers` | Mismatch | 2 | 0 | 0 | 2 | Project-derived |
| `Lint/UnexpectedBlockArity` | Mismatch | 13 | 11 | 11 | 2 | No |
| `Lint/Void` | Mismatch | 64 | 62 | 62 | 2 | Project-derived |
| `Style/AccessModifierDeclarations` | Mismatch | 42 | 42 | 41 | 2 | No |
| `Style/DateTime` | Mismatch | 1632 | 1634 | 1632 | 2 | Project-derived |
| `Style/ExpandPathArguments` | Mismatch | 258 | 260 | 258 | 2 | Project-derived |
| `Style/IfWithBooleanLiteralBranches` | Mismatch | 23 | 21 | 21 | 2 | Project-derived |
| `Style/Lambda` | Mismatch | 3494 | 3492 | 3492 | 2 | No |
| `Style/LambdaCall` | Mismatch | 1645 | 1645 | 1644 | 2 | No |
| `Style/MagicCommentFormat` | Mismatch | 8 | 6 | 6 | 2 | No |
| `Style/NumericLiteralPrefix` | Mismatch | 1134 | 1136 | 1134 | 2 | No |
| `Style/RedundantFormat` | Mismatch | 9 | 11 | 9 | 2 | No |
| `Style/RedundantSelf` | Mismatch | 3581 | 3583 | 3581 | 2 | Project-derived |
| `Layout/CommentIndentation` | Mismatch | 171 | 170 | 170 | 1 | Project-derived |
| `Layout/LineContinuationSpacing` | Mismatch | 350 | 349 | 349 | 1 | Project-derived |
| `Lint/AmbiguousAssignment` | Mismatch | 1 | 0 | 0 | 1 | Project-derived |
| `Lint/AssignmentInCondition` | Mismatch | 1944 | 1945 | 1944 | 1 | No |
| `Lint/ConstantReassignment` | Mismatch | 2 | 1 | 1 | 1 | Project-derived |
| `Lint/CopDirectiveSyntax` | Mismatch | 224 | 225 | 224 | 1 | Project-derived |
| `Lint/EmptyClass` | Mismatch | 78 | 77 | 77 | 1 | Project-derived |
| `Lint/EmptyExpression` | Mismatch | 0 | 1 | 0 | 1 | No |
| `Lint/EmptyInterpolation` | Mismatch | 2 | 3 | 2 | 1 | No |
| `Lint/FormatParameterMismatch` | Mismatch | 1 | 0 | 0 | 1 | Project-derived |
| `Lint/LiteralAsCondition` | Mismatch | 17 | 18 | 17 | 1 | No |
| `Lint/ParenthesesAsGroupedExpression` | Mismatch | 229 | 230 | 229 | 1 | Project-derived |
| `Lint/RedundantSafeNavigation` | Mismatch | 42 | 43 | 42 | 1 | No |
| `Lint/Syntax` | Mismatch | 61 | 60 | 60 | 1 | Project-derived |
| `Lint/UselessAccessModifier` | Mismatch | 126 | 127 | 126 | 1 | Project-derived |
| `Lint/UselessDefaultValueArgument` | Mismatch | 18 | 19 | 18 | 1 | No |
| `Naming/ClassAndModuleCamelCase` | Mismatch | 235 | 234 | 234 | 1 | Project-derived |
| `Naming/MemoizedInstanceVariableName` | Mismatch | 555 | 554 | 554 | 1 | Project-derived |
| `Security/MarshalLoad` | Mismatch | 56 | 55 | 55 | 1 | No |
| `Style/ArrayFirstLast` | Mismatch | 7092 | 7093 | 7092 | 1 | Project-derived |
| `Style/CaseEquality` | Mismatch | 870 | 869 | 869 | 1 | Project-derived |
| `Style/EndBlock` | Mismatch | 1 | 0 | 0 | 1 | Project-derived |
| `Style/HashAsLastArrayItem` | Mismatch | 1630 | 1629 | 1629 | 1 | No |
| `Style/HashConversion` | Mismatch | 157 | 156 | 156 | 1 | No |
| `Style/HashLikeCase` | Mismatch | 41 | 42 | 41 | 1 | Project-derived |
| `Style/MixinUsage` | Mismatch | 48 | 47 | 47 | 1 | No |
| `Style/MultilineWhenThen` | Mismatch | 204 | 203 | 203 | 1 | No |
| `Style/NegatedIfElseCondition` | Mismatch | 280 | 279 | 279 | 1 | No |
| `Style/NestedParenthesizedCalls` | Mismatch | 154 | 153 | 153 | 1 | No |
| `Style/ParallelAssignment` | Mismatch | 722 | 721 | 721 | 1 | No |
| `Style/PartitionInsteadOfDoubleSelect` | Mismatch | 12 | 11 | 11 | 1 | Project-derived |
| `Style/QuotedSymbols` | Mismatch | 3777 | 3776 | 3776 | 1 | No |
| `Style/ReduceToHash` | Mismatch | 163 | 164 | 163 | 1 | No |
| `Style/RedundantCondition` | Mismatch | 98 | 97 | 97 | 1 | No |
| `Style/RedundantRegexpEscape` | Mismatch | 1132 | 1133 | 1132 | 1 | No |
| `Style/RedundantSortBy` | Mismatch | 1 | 2 | 1 | 1 | No |
| `Style/SoleNestedConditional` | Mismatch | 574 | 573 | 573 | 1 | No |
| `Style/StructInheritance` | Mismatch | 36 | 35 | 35 | 1 | No |
| `Style/UnlessLogicalOperators` | Mismatch | 93 | 94 | 93 | 1 | Project-derived |
| `Lint/RedundantCopDisableDirective` | RuboCop gate error | — | — | — | — | No |
| `Style/ClassAndModuleChildren` | RuboCop gate error | — | — | — | — | Project-derived |
| `Style/FileWrite` | RuboCop gate error | — | — | — | — | No |
