# Current project-parity gaps

Generated from the same complete audit as [the evidence matrix](cop-support.md).
Evidence updated at `2026-08-26T01:01:10-04:00`.
This queue contains only failures from that 50-project audit; it does not use the old
Verified/Heuristic qualification labels or captured-case pass counts.

The gap is unmatched complete diagnostic signatures, not the difference
between aggregate offense counts. A cop can have equal counts and still have a
nonzero gap because its message, severity, path, or source range differs.

- Rust source: `cop source 2d402c0b6b599ec52de6db2fe51aa50717e74e87d68218cfee642ab10381de9d`
- Unresolved cops: 64

| Cop | Status | Rustocop | RuboCop | Exact | Signature gap | Project regression evidence |
| --- | --- | ---: | ---: | ---: | ---: | --- |
| `Metrics/AbcSize` | Mismatch | 55196 | 55118 | 41887 | 26540 | Pending unit |
| `Lint/UnusedMethodArgument` | Mismatch | 11152 | 3598 | 3284 | 8182 | No |
| `Style/Documentation` | Mismatch | 54391 | 49775 | 48846 | 6474 | No |
| `Naming/BlockParameterName` | Mismatch | 5996 | 14 | 7 | 5996 | No |
| `Layout/MultilineMethodCallIndentation` | Mismatch | 14541 | 15130 | 11882 | 5907 | Project-derived + pending |
| `Layout/LineLength` | Mismatch | 186034 | 181525 | 180828 | 5903 | Project-derived + pending |
| `Metrics/PerceivedComplexity` | Mismatch | 6041 | 6450 | 3399 | 5693 | No |
| `Metrics/CyclomaticComplexity` | Mismatch | 7360 | 8047 | 5342 | 4723 | No |
| `Naming/VariableNumber` | Mismatch | 13082 | 9963 | 9545 | 3955 | No |
| `Lint/DuplicateRegexpCharacterClassElement` | Mismatch | 3770 | 103 | 99 | 3675 | No |
| `Layout/MultilineOperationIndentation` | Mismatch | 4362 | 2263 | 1629 | 3367 | Project-derived |
| `Lint/AmbiguousRegexpLiteral` | Mismatch | 3450 | 722 | 722 | 2728 | No |
| `Metrics/BlockLength` | Mismatch | 86054 | 86297 | 85000 | 2351 | No |
| `Layout/ExtraSpacing` | Mismatch | 1329 | 1905 | 471 | 2292 | Project-derived |
| `Lint/MissingCopEnableDirective` | Mismatch | 2024 | 98 | 75 | 1972 | Project-derived |
| `Metrics/ClassLength` | Mismatch | 5003 | 5020 | 4042 | 1939 | No |
| `Metrics/MethodLength` | Mismatch | 36969 | 36690 | 35875 | 1909 | Pending unit |
| `Layout/LineEndStringConcatenationIndentation` | Mismatch | 3205 | 2646 | 2003 | 1845 | No |
| `Layout/IndentationWidth` | Mismatch | 8652 | 8652 | 7902 | 1500 | Project-derived |
| `Lint/UnderscorePrefixedVariableName` | Mismatch | 1036 | 2109 | 897 | 1351 | No |
| `Lint/UselessAssignment` | Mismatch | 1349 | 1313 | 696 | 1270 | No |
| `Naming/VariableName` | Mismatch | 1513 | 530 | 530 | 983 | No |
| `Lint/UselessConstantScoping` | Mismatch | 958 | 529 | 318 | 851 | No |
| `Style/DocumentationMethod` | Mismatch | 185727 | 186456 | 185717 | 749 | Pending unit |
| `Style/IfUnlessModifier` | Mismatch | 11199 | 11656 | 11179 | 497 | Project-derived |
| `Style/AutoResourceCleanup` | Mismatch | 548 | 86 | 86 | 462 | Project-derived |
| `Lint/AmbiguousRange` | Mismatch | 456 | 164 | 98 | 424 | Project-derived |
| `Layout/RedundantLineBreak` | Mismatch | 31715 | 31577 | 31471 | 350 | No |
| `Style/RedundantParentheses` | Mismatch | 712 | 522 | 480 | 274 | Project-derived |
| `Metrics/BlockNesting` | Mismatch | 631 | 663 | 538 | 218 | No |
| `Style/FetchEnvVar` | Mismatch | 1497 | 1386 | 1340 | 203 | Project-derived |
| `Naming/HeredocDelimiterNaming` | Mismatch | 2356 | 2184 | 2184 | 172 | Project-derived |
| `Layout/MultilineAssignmentLayout` | Mismatch | 17262 | 17367 | 17234 | 161 | Project-derived |
| `Lint/RedundantCopEnableDirective` | Mismatch | 176 | 20 | 20 | 156 | Project-derived |
| `Style/ExplicitBlockArgument` | Mismatch | 282 | 357 | 248 | 143 | Project-derived |
| `Layout/SpaceAroundOperators` | Mismatch | 7020 | 7084 | 6988 | 128 | Project-derived |
| `Style/EmptyLiteral` | Mismatch | 240 | 118 | 116 | 126 | Project-derived |
| `Style/RedundantAssignment` | Mismatch | 249 | 201 | 176 | 98 | Project-derived |
| `Style/StringConcatenation` | Mismatch | 3175 | 3080 | 3080 | 95 | Project-derived |
| `Layout/RescueEnsureAlignment` | Mismatch | 455 | 372 | 372 | 83 | Project-derived |
| `Style/EvalWithLocation` | Mismatch | 529 | 504 | 483 | 67 | Project-derived |
| `Style/AccessorGrouping` | Mismatch | 2024 | 1978 | 1968 | 66 | Project-derived |
| `Lint/OrAssignmentToConstant` | Mismatch | 159 | 172 | 136 | 59 | No |
| `Layout/SpaceInsideParens` | Mismatch | 959 | 1012 | 958 | 55 | Project-derived |
| `Style/MethodCallWithArgsParentheses` | Mismatch | 838151 | 838160 | 838129 | 53 | Pending unit |
| `Style/FormatStringToken` | Mismatch | 7663 | 7619 | 7619 | 44 | Project-derived |
| `Lint/ErbNewArguments` | Mismatch | 46 | 8 | 8 | 38 | No |
| `Lint/ConstantResolution` | Mismatch | 849548 | 849583 | 849548 | 35 | Pending unit |
| `Style/Semicolon` | Mismatch | 2726 | 2692 | 2692 | 34 | No |
| `Layout/ClosingParenthesisIndentation` | Mismatch | 341 | 372 | 340 | 33 | No |
| `Style/ItBlockParameter` | Mismatch | 209 | 240 | 208 | 33 | No |
| `Style/StringHashKeys` | Mismatch | 131615 | 131591 | 131587 | 32 | Project-derived |
| `Style/ClassMethods` | Mismatch | 36 | 45 | 25 | 31 | Project-derived |
| `Lint/SelfAssignment` | Mismatch | 48 | 18 | 18 | 30 | Project-derived |
| `Lint/SuppressedExceptionInNumberConversion` | Mismatch | 34 | 4 | 4 | 30 | Project-derived |
| `Style/RedundantFreeze` | Mismatch | 159 | 187 | 159 | 28 | No |
| `Layout/FirstParameterIndentation` | Mismatch | 24 | 5 | 1 | 27 | No |
| `Style/EmptyHeredoc` | Mismatch | 21 | 18 | 6 | 27 | Project-derived |
| `Style/EndlessMethod` | Mismatch | 29 | 3 | 3 | 26 | Project-derived |
| `Style/EnvHome` | Mismatch | 16 | 33 | 12 | 25 | Project-derived |
| `Layout/HeredocArgumentClosingParenthesis` | Mismatch | 58 | 34 | 34 | 24 | Project-derived |
| `Style/CombinableLoops` | Mismatch | 111 | 94 | 91 | 23 | Project-derived |
| `Style/CommentAnnotation` | Mismatch | 564 | 544 | 543 | 22 | No |
| `Style/RedundantCapitalW` | Mismatch | 55 | 35 | 34 | 22 | Project-derived |
