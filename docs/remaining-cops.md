# Current project-parity gaps

Generated from the same complete audit as [the evidence matrix](cop-support.md).
Evidence updated at `2026-08-22T14:36:12-04:00`.
This queue contains only current ten-project failures; it does not use the old
Verified/Heuristic qualification labels or captured-case pass counts.

The gap is unmatched complete diagnostic signatures, not the difference
between aggregate offense counts. A cop can have equal counts and still have a
nonzero gap because its message, severity, path, or source range differs.

- Rust source: `cdb4527ddb40dddd9ac779167e1f3bbf08acd557`
- Unresolved cops: 188

| Cop | Status | Rustocop | RuboCop | Exact | Signature gap | Project regression evidence |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `Style/MethodCallWithArgsParentheses` | Mismatch | 1041796 | 524118 | 495680 | 574554 | Pending |
| `Layout/LineLength` | Mismatch | 132326 | 111445 | 109137 | 25497 | Pending |
| `Layout/HeredocIndentation` | Mismatch | 21865 | 2276 | 0 | 24141 | Pending |
| `Layout/SpaceAroundOperators` | Mismatch | 18803 | 3859 | 0 | 22662 | Pending |
| `Style/RedundantConstantBase` | Mismatch | 29506 | 7630 | 7630 | 21876 | Passing |
| `Layout/HashAlignment` | Mismatch | 2641 | 15158 | 0 | 17799 | Pending |
| `Naming/HeredocDelimiterNaming` | Mismatch | 15249 | 1206 | 5 | 16445 | Pending |
| `Style/MultilineTernaryOperator` | Mismatch | 15376 | 79 | 51 | 15353 | Pending |
| `Style/MethodCallWithoutArgsParentheses` | Mismatch | 12872 | 158 | 131 | 12768 | Pending |
| `Style/ConstantVisibility` | Mismatch | 22332 | 20182 | 15361 | 11792 | Pending |
| `Layout/MultilineMethodCallIndentation` | Mismatch | 601 | 10247 | 0 | 10848 | Pending |
| `Style/DocumentationMethod` | Mismatch | 113418 | 118690 | 110753 | 10602 | Pending |
| `Layout/MultilineAssignmentLayout` | Mismatch | 0 | 10081 | 0 | 10081 | Pending |
| `Layout/SpaceBeforeBlockBraces` | Mismatch | 8021 | 38 | 34 | 7991 | Pending |
| `Naming/ConstantName` | Mismatch | 7963 | 221 | 195 | 7794 | Pending |
| `Layout/MultilineArrayLineBreaks` | Mismatch | 1758 | 7730 | 1593 | 6302 | Pending |
| `Metrics/MethodLength` | Mismatch | 22183 | 22292 | 19242 | 5991 | Pending |
| `Lint/SymbolConversion` | Mismatch | 7096 | 1335 | 1265 | 5901 | Pending |
| `Layout/ClassStructure` | Mismatch | 3671 | 1540 | 6 | 5199 | Pending |
| `Style/ArgumentsForwarding` | Mismatch | 231 | 4834 | 10 | 5045 | Pending |
| `Layout/BeginEndAlignment` | Mismatch | 4533 | 8 | 0 | 4541 | Pending |
| `Style/ClassAndModuleChildren` | Mismatch | 1102 | 3247 | 0 | 4349 | Pending |
| `Style/Documentation` | Mismatch | 35169 | 32450 | 31653 | 4313 | Pending |
| `Naming/InclusiveLanguage` | Mismatch | 4069 | 193 | 0 | 4262 | Pending |
| `Layout/SpaceBeforeBrackets` | Mismatch | 4069 | 0 | 0 | 4069 | Pending |
| `Lint/InterpolationCheck` | Mismatch | 3819 | 26 | 25 | 3795 | Pending |
| `Style/MissingElse` | Mismatch | 19352 | 22846 | 19327 | 3544 | Pending |
| `Naming/BlockParameterName` | Mismatch | 3309 | 2 | 1 | 3309 | Pending |
| `Layout/SpaceInsideArrayLiteralBrackets` | Mismatch | 850 | 2069 | 0 | 2919 | Pending |
| `Style/BlockDelimiters` | Mismatch | 20 | 2831 | 1 | 2849 | Pending |
| `Naming/PredicatePrefix` | Mismatch | 143 | 2837 | 143 | 2694 | Passing |
| `Layout/ArrayAlignment` | Mismatch | 583 | 1685 | 0 | 2268 | Pending |
| `Layout/MultilineOperationIndentation` | Mismatch | 601 | 1505 | 0 | 2106 | Pending |
| `Layout/SpaceInsideRangeLiteral` | Mismatch | 1974 | 4 | 0 | 1978 | Pending |
| `Gemspec/DuplicatedAssignment` | Mismatch | 1923 | 0 | 0 | 1923 | Passing |
| `Style/ImplicitRuntimeError` | Mismatch | 1570 | 1841 | 758 | 1895 | Pending |
| `Lint/CopDirectiveSyntax` | Mismatch | 1827 | 32 | 6 | 1847 | Pending |
| `Style/FetchEnvVar` | Mismatch | 2109 | 749 | 721 | 1416 | Pending |
| `Lint/MissingCopEnableDirective` | Mismatch | 1217 | 77 | 34 | 1226 | Pending |
| `Metrics/ModuleLength` | Mismatch | 464 | 685 | 0 | 1149 | Pending |
| `Lint/AmbiguousOperator` | Mismatch | 1098 | 7 | 7 | 1091 | Pending |
| `Lint/RedundantCopEnableDirective` | Mismatch | 1062 | 5 | 5 | 1057 | Pending |
| `Metrics/BlockLength` | Mismatch | 56135 | 56241 | 55663 | 1050 | Pending |
| `Layout/CommentIndentation` | Mismatch | 1053 | 32 | 32 | 1021 | Pending |
| `Naming/AsciiIdentifiers` | Mismatch | 960 | 10 | 10 | 950 | Pending |
| `Style/HashLikeCase` | Mismatch | 979 | 32 | 32 | 947 | Passing |
| `Layout/SpaceAroundEqualsInParameterDefault` | Mismatch | 1000 | 88 | 88 | 912 | Pending |
| `Naming/FileName` | Mismatch | 393 | 393 | 0 | 786 | Passing |
| `Lint/SafeNavigationConsistency` | Mismatch | 766 | 1 | 0 | 767 | Pending |
| `Metrics/ParameterLists` | Mismatch | 887 | 896 | 533 | 717 | Pending |
| `Layout/ClosingHeredocIndentation` | Mismatch | 750 | 701 | 376 | 699 | Pending |
| `Lint/AmbiguousRange` | Mismatch | 614 | 80 | 29 | 636 | Pending |
| `Lint/AmbiguousOperatorPrecedence` | Mismatch | 995 | 388 | 388 | 607 | Pending |
| `Layout/EndAlignment` | Mismatch | 1561 | 957 | 957 | 604 | Pending |
| `Lint/SafeNavigationChain` | Mismatch | 550 | 1 | 0 | 551 | Pending |
| `Lint/RequireRangeParentheses` | Mismatch | 539 | 0 | 0 | 539 | Pending |
| `Lint/UselessConstantScoping` | Mismatch | 603 | 330 | 199 | 535 | Pending |
| `Lint/AmbiguousBlockAssociation` | Mismatch | 4344 | 4612 | 4231 | 494 | Pending |
| `Layout/EmptyLinesAroundExceptionHandlingKeywords` | Mismatch | 588 | 123 | 123 | 465 | Pending |
| `Style/NumberedParameters` | Mismatch | 461 | 1 | 1 | 460 | Passing |
| `Style/AccessorGrouping` | Mismatch | 952 | 505 | 500 | 457 | Pending |
| `Layout/SpaceInsideReferenceBrackets` | Mismatch | 451 | 1 | 0 | 452 | Pending |
| `Lint/UselessDefined` | Mismatch | 442 | 0 | 0 | 442 | Pending |
| `Naming/MemoizedInstanceVariableName` | Mismatch | 368 | 426 | 189 | 416 | Pending |
| `Lint/DuplicateRequire` | Mismatch | 414 | 1 | 1 | 413 | Pending |
| `Style/IfWithSemicolon` | Mismatch | 404 | 0 | 0 | 404 | Pending |
| `Style/RedundantParentheses` | Mismatch | 558 | 226 | 193 | 398 | Pending |
| `Layout/ExtraSpacing` | Mismatch | 205 | 339 | 75 | 394 | Pending |
| `Style/AutoResourceCleanup` | Mismatch | 374 | 36 | 15 | 380 | Pending |
| `Layout/EmptyLinesAfterModuleInclusion` | Mismatch | 938 | 581 | 577 | 365 | Pending |
| `Style/EvalWithLocation` | Mismatch | 348 | 73 | 35 | 351 | Pending |
| `Style/ColonMethodDefinition` | Mismatch | 336 | 0 | 0 | 336 | Pending |
| `Style/MultilineMemoization` | Mismatch | 328 | 2 | 0 | 330 | Pending |
| `Style/IfUnlessModifier` | Mismatch | 5313 | 5405 | 5207 | 304 | Pending |
| `Style/IpAddresses` | Mismatch | 2025 | 1724 | 1724 | 301 | Pending |
| `Layout/EmptyLineAfterGuardClause` | Mismatch | 2665 | 2877 | 2622 | 298 | Pending |
| `Lint/ShadowingOuterLocalVariable` | Mismatch | 29 | 263 | 0 | 292 | Pending |
| `Lint/DeprecatedConstants` | Mismatch | 297 | 12 | 12 | 285 | Pending |
| `Lint/LiteralInInterpolation` | Mismatch | 309 | 26 | 25 | 285 | Pending |
| `Layout/RescueEnsureAlignment` | Mismatch | 6 | 245 | 0 | 251 | Pending |
| `Style/AsciiComments` | Mismatch | 857 | 609 | 609 | 248 | Pending |
| `Lint/EmptyBlock` | Mismatch | 986 | 988 | 864 | 246 | Pending |
| `Layout/AssignmentIndentation` | Mismatch | 240 | 7 | 2 | 243 | Pending |
| `Style/RedundantAssignment` | Mismatch | 214 | 64 | 22 | 234 | Pending |
| `Lint/IneffectiveAccessModifier` | Mismatch | 436 | 360 | 287 | 222 | Pending |
| `Style/IfInsideElse` | Mismatch | 311 | 107 | 104 | 210 | Pending |
| `Style/ExplicitBlockArgument` | Mismatch | 28 | 206 | 16 | 202 | Pending |
| `Style/DoubleNegation` | Mismatch | 367 | 212 | 190 | 199 | Pending |
| `Style/StringConcatenation` | Mismatch | 1490 | 1320 | 1313 | 184 | Pending |
| `Style/EndBlock` | Mismatch | 179 | 0 | 0 | 179 | Pending |
| `Lint/ElseLayout` | Mismatch | 169 | 0 | 0 | 169 | Pending |
| `Lint/DuplicateBranch` | Mismatch | 0 | 168 | 0 | 168 | Pending |
| `Layout/SpaceInsidePercentLiteralDelimiters` | Mismatch | 982 | 815 | 815 | 167 | Pending |
| `Style/ArrayCoercion` | Mismatch | 169 | 45 | 26 | 162 | Pending |
| `Lint/ShadowedArgument` | Mismatch | 156 | 4 | 0 | 160 | Pending |
| `Style/GuardClause` | Mismatch | 1977 | 2111 | 1967 | 154 | Pending |
| `Layout/SpaceInsideStringInterpolation` | Mismatch | 218 | 73 | 71 | 149 | Pending |
| `Style/MultipleComparison` | Mismatch | 134 | 202 | 95 | 146 | Pending |
| `Lint/RescueException` | Mismatch | 76 | 78 | 8 | 138 | Pending |
| `Lint/TrailingCommaInAttributeDeclaration` | Mismatch | 137 | 0 | 0 | 137 | Pending |
| `Lint/Void` | Mismatch | 143 | 7 | 7 | 136 | Pending |
| `Gemspec/RubyVersionGlobalsUsage` | Mismatch | 133 | 0 | 0 | 133 | Passing |
| `Style/RedundantSelf` | Mismatch | 1161 | 1033 | 1033 | 128 | Pending |
| `Style/ExpandPathArguments` | Mismatch | 69 | 69 | 7 | 124 | Pending |
| `Style/SafeNavigation` | Mismatch | 707 | 586 | 585 | 123 | Pending |
| `Lint/Syntax` | Mismatch | 73 | 44 | 0 | 117 | Pending |
| `Style/DisableCopsWithinSourceCodeDirective` | Mismatch | 8428 | 8468 | 8390 | 116 | Pending |
| `Style/ModuleFunction` | Mismatch | 0 | 114 | 0 | 114 | Pending |
| `Style/ClassVars` | Mismatch | 207 | 298 | 197 | 111 | Pending |
| `Lint/UselessElseWithoutRescue` | Mismatch | 109 | 0 | 0 | 109 | Pending |
| `Lint/UselessAccessModifier` | Mismatch | 0 | 100 | 0 | 100 | Pending |
| `Style/ArrayIntersect` | Mismatch | 54 | 76 | 16 | 98 | Pending |
| `Layout/SpaceInLambdaLiteral` | Mismatch | 90 | 125 | 60 | 95 | Pending |
| `Lint/NonAtomicFileOperation` | Mismatch | 9 | 82 | 0 | 91 | Pending |
| `Style/DocumentDynamicEvalDefinition` | Mismatch | 0 | 90 | 0 | 90 | Pending |
| `Bundler/GemVersion` | Mismatch | 88 | 0 | 0 | 88 | Passing |
| `Naming/PredicateMethod` | Mismatch | 1600 | 1560 | 1536 | 88 | Pending |
| `Lint/EmptyEnsure` | Mismatch | 87 | 0 | 0 | 87 | Pending |
| `Lint/HashNewWithKeywordArgumentsAsDefault` | Mismatch | 87 | 0 | 0 | 87 | Pending |
| `Style/RedundantLineContinuation` | Mismatch | 144 | 61 | 59 | 87 | Pending |
| `Style/IdenticalConditionalBranches` | Mismatch | 158 | 75 | 74 | 85 | Pending |
| `Style/InvertibleUnlessCondition` | Mismatch | 453 | 376 | 375 | 79 | Pending |
| `Style/MethodCalledOnDoEndBlock` | Mismatch | 6071 | 6149 | 6071 | 78 | Pending |
| `Lint/RedundantSplatExpansion` | Mismatch | 52 | 22 | 0 | 74 | Pending |
| `Style/MultilineMethodSignature` | Mismatch | 79 | 6 | 6 | 73 | Passing |
| `Lint/OutOfRangeRegexpRef` | Mismatch | 69 | 1 | 0 | 70 | Pending |
| `Style/MultilineIfThen` | Mismatch | 71 | 3 | 2 | 70 | Pending |
| `Lint/EmptyClass` | Mismatch | 65 | 41 | 21 | 64 | Pending |
| `Style/CommentedKeyword` | Mismatch | 170 | 230 | 168 | 64 | Pending |
| `Naming/MethodName` | Mismatch | 332 | 373 | 323 | 59 | Pending |
| `Style/EmptyLiteral` | Mismatch | 112 | 56 | 56 | 56 | Pending |
| `Style/MultilineBlockChain` | Mismatch | 130 | 130 | 102 | 56 | Pending |
| `Style/FileWrite` | Mismatch | 65 | 13 | 13 | 52 | Pending |
| `Layout/EmptyLineAfterMultilineCondition` | Mismatch | 1239 | 1210 | 1199 | 51 | Pending |
| `Lint/PercentStringArray` | Mismatch | 58 | 7 | 7 | 51 | Pending |
| `Style/CaseLikeIf` | Mismatch | 22 | 27 | 0 | 49 | Pending |
| `Style/EmptyStringInsideInterpolation` | Mismatch | 89 | 137 | 89 | 48 | Pending |
| `Lint/RequireRelativeSelfPath` | Mismatch | 45 | 0 | 0 | 45 | Passing |
| `Style/ArrayFirstLast` | Mismatch | 3516 | 3484 | 3479 | 42 | Pending |
| `Style/KeywordArgumentsMerging` | Mismatch | 145 | 104 | 104 | 41 | Pending |
| `Lint/UriEscapeUnescape` | Mismatch | 39 | 0 | 0 | 39 | Pending |
| `Layout/EmptyLinesAroundAccessModifier` | Mismatch | 1681 | 1717 | 1680 | 38 | Pending |
| `Naming/ClassAndModuleCamelCase` | Mismatch | 92 | 55 | 55 | 37 | Pending |
| `Layout/AccessModifierIndentation` | Mismatch | 152 | 118 | 118 | 34 | Pending |
| `Layout/SpaceAfterMethodName` | Mismatch | 33 | 0 | 0 | 33 | Passing |
| `Lint/EmptyFile` | Mismatch | 17 | 16 | 0 | 33 | Pending |
| `Style/CharacterLiteral` | Mismatch | 44 | 11 | 11 | 33 | Pending |
| `Style/ReturnNil` | Mismatch | 731 | 698 | 698 | 33 | Pending |
| `Lint/UselessMethodDefinition` | Mismatch | 43 | 19 | 15 | 32 | Pending |
| `Style/MapIntoArray` | Mismatch | 52 | 36 | 28 | 32 | Pending |
| `Layout/EmptyComment` | Mismatch | 48 | 21 | 19 | 31 | Pending |
| `Lint/ReturnInVoidContext` | Mismatch | 37 | 7 | 7 | 30 | Pending |
| `Lint/DisjunctiveAssignmentInConstructor` | Mismatch | 31 | 4 | 4 | 27 | Pending |
| `Lint/FormatParameterMismatch` | Mismatch | 27 | 0 | 0 | 27 | Pending |
| `Lint/HeredocMethodCallPosition` | Mismatch | 27 | 0 | 0 | 27 | Pending |
| `Style/AndOr` | Mismatch | 27 | 0 | 0 | 27 | Pending |
| `Gemspec/RequiredRubyVersion` | Mismatch | 26 | 0 | 0 | 26 | Passing |
| `Lint/ConstantDefinitionInBlock` | Mismatch | 310 | 284 | 284 | 26 | Pending |
| `Style/RedundantRegexpArgument` | Mismatch | 191 | 171 | 169 | 24 | Pending |
| `Style/CollectionCompact` | Mismatch | 31 | 8 | 8 | 23 | Pending |
| `Style/HashEachMethods` | Mismatch | 231 | 219 | 214 | 22 | Pending |
| `Style/RedundantHeredocDelimiterQuotes` | Mismatch | 115 | 93 | 93 | 22 | Pending |
| `Style/EmptyMethod` | Mismatch | 470 | 450 | 450 | 20 | Pending |
| `Style/FormatStringToken` | Mismatch | 3599 | 3581 | 3580 | 20 | Pending |
| `Lint/ConstantReassignment` | Mismatch | 18 | 1 | 0 | 19 | Pending |
| `Style/RedundantFetchBlock` | Mismatch | 77 | 88 | 73 | 19 | Pending |
| `Style/EachWithObject` | Mismatch | 43 | 61 | 43 | 18 | Pending |
| `Style/RedundantSelfAssignment` | Mismatch | 32 | 16 | 15 | 18 | Pending |
| `Layout/EmptyLinesAroundAttributeAccessor` | Mismatch | 138 | 122 | 122 | 16 | Pending |
| `Layout/SpaceInsideBlockBraces` | Mismatch | 534 | 549 | 534 | 15 | Pending |
| `Lint/TripleQuotes` | Mismatch | 15 | 0 | 0 | 15 | Pending |
| `Layout/SpaceInsideArrayPercentLiteral` | Mismatch | 39 | 25 | 25 | 14 | Pending |
| `Lint/ShadowedException` | Mismatch | 12 | 17 | 8 | 13 | Pending |
| `Style/EmptyElse` | Mismatch | 101 | 97 | 93 | 12 | Pending |
| `Style/ItAssignment` | Mismatch | 4 | 14 | 3 | 12 | Pending |
| `Layout/FirstArgumentIndentation` | Mismatch | 638 | 628 | 628 | 10 | Pending |
| `Lint/UnreachablePatternBranch` | Mismatch | 9 | 0 | 0 | 9 | Pending |
| `Style/ConditionalAssignment` | Mismatch | 314 | 316 | 311 | 8 | Pending |
| `Lint/DuplicateMethods` | Mismatch | 19 | 13 | 13 | 6 | Passing + pending |
| `Layout/ElseAlignment` | Mismatch | 931 | 927 | 927 | 4 | Pending |
| `Style/EndlessMethod` | Mismatch | 0 | 3 | 0 | 3 | Passing + pending |
| `Style/EachForSimpleLoop` | Mismatch | 2 | 0 | 0 | 2 | Pending |
| `Layout/BlockAlignment` | Mismatch | 95 | 96 | 95 | 1 | Pending |
| `Lint/SelfAssignment` | Mismatch | 14 | 13 | 13 | 1 | Passing + pending |
| `Lint/UnescapedBracketInRegexp` | Mismatch | 1 | 0 | 0 | 1 | Passing + pending |
| `Layout/FirstHashElementIndentation` | Rust crash | — | — | — | — | Passing |
| `Lint/RedundantCopDisableDirective` | RuboCop gate error | — | — | — | — | No |
| `Style/CombinableLoops` | Rust crash | — | — | — | — | Passing |
