# Remaining cop project gate: positions 361–1

Generated from 7 batched project-gate reports against Rust source
`15032c62724a1bfcd1d7199a9342188ff3b96ee4` and RuboCop 1.87.0.
This completes the project-first gate for the remaining reverse-order range.

## Summary

| Classification | Cops |
| --- | ---: |
| Exact and active | 20 |
| Exact but dormant | 108 |
| Diagnostic mismatch | 230 |
| Rustocop crash | 2 |
| RuboCop gate error | 1 |
| **Total** | **361** |

The 7 final comparison runs took 35.2
seconds in Rustocop and 963.9 seconds in
RuboCop. Timings exclude crash/error isolation probes.

## Exact-active candidates

These 20 cops advance only to upstream, correction, evidence, and
manual source-boundary review. They are not qualified by this gate alone.

- `Style/DigChain`
- `Style/DateTime`
- `Style/ConcatArrayLiterals`
- `Style/ComparableClamp`
- `Style/CollectionQuerying`
- `Style/CaseEquality`
- `Style/ArrayIntersectWithSingleElement`
- `Style/Alias`
- `Security/JSONLoad`
- `Naming/BinaryOperatorParameterName`
- `Lint/UselessDefaultValueArgument`
- `Lint/SharedMutableDefault`
- `Lint/OrAssignmentToConstant`
- `Lint/Loop`
- `Lint/ImplicitStringConcatenation`
- `Layout/TrailingWhitespace`
- `Layout/TrailingEmptyLines`
- `Layout/EmptyLinesAroundModuleBody`
- `Layout/EmptyLinesAroundMethodBody`
- `Layout/EmptyLineAfterMagicComment`

## Engine failures

- `Lint/UnusedMethodArgument`: `crash` on chatwoot: thread '<unnamed>' (32109563) panicked at src/cops/text/lint_semantic.rs:62:25: slice index starts at 149 but ends at 148 note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
- `Lint/RedundantCopDisableDirective`: `rubocop_error` on chatwoot: Lint/RedundantCopDisableDirective cannot be used with --only.
- `Gemspec/OrderedDependencies`: `crash` on gitlab-ce: thread '<unnamed>' (32204892) panicked at src/cops/prism/gemspec_completion.rs:138:22: byte range starts at 921 but ends at 101 note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

## Complete classification

| Position | Cop | Rustocop | RuboCop | Exact | Classification |
| ---: | --- | ---: | ---: | ---: | --- |
| 361 | `Style/DocumentDynamicEvalDefinition` | 0 | 5 | 0 | mismatch |
| 360 | `Style/DisableCopsWithinSourceCodeDirective` | 7904 | 7948 | 7873 | mismatch |
| 359 | `Style/DirEmpty` | 0 | 0 | 0 | dormant |
| 358 | `Style/Dir` | 0 | 0 | 0 | dormant |
| 357 | `Style/DigChain` | 1 | 1 | 1 | exact_active |
| 356 | `Style/DefWithParentheses` | 0 | 0 | 0 | dormant |
| 355 | `Style/DateTime` | 370 | 370 | 370 | exact_active |
| 354 | `Style/DataInheritance` | 0 | 0 | 0 | dormant |
| 353 | `Style/Copyright` | 34069 | 0 | 0 | mismatch |
| 352 | `Style/ConstantVisibility` | 13776 | 13284 | 10795 | mismatch |
| 351 | `Style/ConditionalAssignment` | 109 | 4 | 0 | mismatch |
| 350 | `Style/ConcatArrayLiterals` | 23 | 23 | 23 | exact_active |
| 349 | `Style/ComparableClamp` | 3 | 3 | 3 | exact_active |
| 348 | `Style/ComparableBetween` | 18 | 7 | 7 | mismatch |
| 347 | `Style/CommentedKeyword` | 0 | 0 | 0 | dormant |
| 346 | `Style/CommentAnnotation` | 0 | 134 | 0 | mismatch |
| 345 | `Style/CommandLiteral` | 0 | 0 | 0 | dormant |
| 344 | `Style/CombinableLoops` | 0 | 0 | 0 | dormant |
| 343 | `Style/CombinableDefined` | 0 | 1 | 0 | mismatch |
| 342 | `Style/ColonMethodDefinition` | 178 | 0 | 0 | mismatch |
| 341 | `Style/ColonMethodCall` | 0 | 0 | 0 | dormant |
| 340 | `Style/CollectionQuerying` | 50 | 50 | 50 | exact_active |
| 339 | `Style/CollectionMethods` | 0 | 188 | 0 | mismatch |
| 338 | `Style/CollectionCompact` | 15 | 6 | 6 | mismatch |
| 337 | `Style/ClassVars` | 25 | 12 | 10 | mismatch |
| 336 | `Style/ClassMethodsDefinitions` | 351 | 350 | 350 | mismatch |
| 335 | `Style/ClassMethods` | 0 | 0 | 0 | dormant |
| 334 | `Style/ClassEqualityComparison` | 16 | 0 | 0 | mismatch |
| 333 | `Style/ClassCheck` | 5 | 0 | 0 | mismatch |
| 332 | `Style/ClassAndModuleChildren` | 402 | 1530 | 0 | mismatch |
| 331 | `Style/CharacterLiteral` | 4 | 0 | 0 | mismatch |
| 330 | `Style/CaseLikeIf` | 42 | 1 | 0 | mismatch |
| 329 | `Style/CaseEquality` | 32 | 32 | 32 | exact_active |
| 328 | `Style/BlockDelimiters` | 2 | 9 | 0 | mismatch |
| 327 | `Style/BlockComments` | 22 | 19 | 19 | mismatch |
| 326 | `Style/BitwisePredicate` | 1 | 7 | 1 | mismatch |
| 325 | `Style/BisectedAttrAccessor` | 0 | 0 | 0 | dormant |
| 324 | `Style/BeginBlock` | 0 | 0 | 0 | dormant |
| 323 | `Style/BarePercentLiterals` | 0 | 0 | 0 | dormant |
| 322 | `Style/AutoResourceCleanup` | 168 | 5 | 4 | mismatch |
| 321 | `Style/Attr` | 0 | 0 | 0 | dormant |
| 320 | `Style/AsciiComments` | 186 | 56 | 56 | mismatch |
| 319 | `Style/ArrayJoin` | 0 | 0 | 0 | dormant |
| 318 | `Style/ArrayIntersectWithSingleElement` | 1 | 1 | 1 | exact_active |
| 317 | `Style/ArrayIntersect` | 27 | 37 | 12 | mismatch |
| 316 | `Style/ArrayFirstLast` | 1394 | 1389 | 1386 | mismatch |
| 315 | `Style/ArrayCoercion` | 64 | 7 | 6 | mismatch |
| 314 | `Style/ArgumentsForwarding` | 63 | 1553 | 0 | mismatch |
| 313 | `Style/AndOr` | 15 | 0 | 0 | mismatch |
| 312 | `Style/AmbiguousEndlessMethodDefinition` | 3 | 0 | 0 | mismatch |
| 311 | `Style/Alias` | 580 | 580 | 580 | exact_active |
| 310 | `Style/AccessorGrouping` | 317 | 195 | 194 | mismatch |
| 309 | `Style/AccessModifierDeclarations` | 0 | 0 | 0 | dormant |
| 308 | `Security/YAMLLoad` | 0 | 0 | 0 | dormant |
| 307 | `Security/Open` | 0 | 0 | 0 | dormant |
| 306 | `Security/MarshalLoad` | 4 | 0 | 0 | mismatch |
| 305 | `Security/JSONLoad` | 39 | 39 | 39 | exact_active |
| 304 | `Security/IoMethods` | 0 | 0 | 0 | dormant |
| 303 | `Security/Eval` | 0 | 0 | 0 | dormant |
| 302 | `Security/CompoundHash` | 3 | 1 | 1 | mismatch |
| 301 | `Naming/VariableNumber` | 0 | 4211 | 0 | mismatch |
| 300 | `Naming/VariableName` | 1883 | 0 | 0 | mismatch |
| 299 | `Naming/RescuedExceptionsVariableName` | 10 | 260 | 0 | mismatch |
| 298 | `Naming/PredicatePrefix` | 0 | 634 | 0 | mismatch |
| 297 | `Naming/PredicateMethod` | 24 | 496 | 5 | mismatch |
| 296 | `Naming/MethodParameterName` | 574 | 126 | 0 | mismatch |
| 295 | `Naming/MethodName` | 6 | 0 | 0 | mismatch |
| 294 | `Naming/MemoizedInstanceVariableName` | 148 | 231 | 0 | mismatch |
| 293 | `Naming/InclusiveLanguage` | 3793 | 160 | 0 | mismatch |
| 292 | `Naming/HeredocDelimiterNaming` | 0 | 22 | 0 | mismatch |
| 291 | `Naming/HeredocDelimiterCase` | 5257 | 0 | 0 | mismatch |
| 290 | `Naming/FileName` | 29 | 12 | 0 | mismatch |
| 289 | `Naming/ConstantName` | 1179 | 0 | 0 | mismatch |
| 288 | `Naming/ClassAndModuleCamelCase` | 22 | 20 | 20 | mismatch |
| 287 | `Naming/BlockParameterName` | 8668 | 0 | 0 | mismatch |
| 286 | `Naming/BlockForwarding` | 962 | 923 | 695 | mismatch |
| 285 | `Naming/BinaryOperatorParameterName` | 18 | 18 | 18 | exact_active |
| 284 | `Naming/AsciiIdentifiers` | 275 | 0 | 0 | mismatch |
| 283 | `Naming/AccessorMethodName` | 1256 | 346 | 0 | mismatch |
| 282 | `Migration/DepartmentName` | 0 | 0 | 0 | dormant |
| 281 | `Metrics/PerceivedComplexity` | 27 | 784 | 0 | mismatch |
| 280 | `Metrics/ParameterLists` | 384 | 325 | 190 | mismatch |
| 279 | `Metrics/ModuleLength` | 170 | 344 | 0 | mismatch |
| 278 | `Metrics/MethodLength` | 8353 | 8297 | 7019 | mismatch |
| 277 | `Metrics/CyclomaticComplexity` | 26 | 1126 | 0 | mismatch |
| 276 | `Metrics/CollectionLiteralLength` | 5 | 6 | 5 | mismatch |
| 275 | `Metrics/ClassLength` | 53 | 1287 | 0 | mismatch |
| 274 | `Metrics/BlockNesting` | 20 | 6 | 0 | mismatch |
| 273 | `Metrics/BlockLength` | 39305 | 39318 | 39108 | mismatch |
| 272 | `Metrics/AbcSize` | 11222 | 13223 | 461 | mismatch |
| 271 | `Lint/Void` | 350 | 1 | 0 | mismatch |
| 270 | `Lint/UselessTimes` | 0 | 0 | 0 | dormant |
| 269 | `Lint/UselessSetterCall` | 0 | 0 | 0 | dormant |
| 268 | `Lint/UselessRuby2Keywords` | 0 | 0 | 0 | dormant |
| 267 | `Lint/UselessRescue` | 0 | 0 | 0 | dormant |
| 266 | `Lint/UselessOr` | 168 | 8 | 0 | mismatch |
| 265 | `Lint/UselessNumericOperation` | 0 | 0 | 0 | dormant |
| 264 | `Lint/UselessMethodDefinition` | 17 | 1 | 1 | mismatch |
| 263 | `Lint/UselessElseWithoutRescue` | 19 | 0 | 0 | mismatch |
| 262 | `Lint/UselessDefined` | 115 | 0 | 0 | mismatch |
| 261 | `Lint/UselessDefaultValueArgument` | 3 | 3 | 3 | exact_active |
| 260 | `Lint/UselessConstantScoping` | 169 | 89 | 58 | mismatch |
| 259 | `Lint/UselessAssignment` | 0 | 0 | 0 | dormant |
| 258 | `Lint/UselessAccessModifier` | 0 | 11 | 0 | mismatch |
| 257 | `Lint/UriRegexp` | 0 | 0 | 0 | dormant |
| 256 | `Lint/UriEscapeUnescape` | 37 | 0 | 0 | mismatch |
| 255 | `Lint/UnusedMethodArgument` | — | — | — | crash |
| 254 | `Lint/UnusedBlockArgument` | 41736 | 531 | 0 | mismatch |
| 253 | `Lint/UnreachablePatternBranch` | 0 | 0 | 0 | dormant |
| 252 | `Lint/UnreachableLoop` | 0 | 0 | 0 | dormant |
| 251 | `Lint/UnreachableCode` | 2 | 0 | 0 | mismatch |
| 250 | `Lint/UnmodifiedReduceAccumulator` | 1 | 0 | 0 | mismatch |
| 249 | `Lint/UnifiedInteger` | 0 | 0 | 0 | dormant |
| 248 | `Lint/UnexpectedBlockArity` | 0 | 0 | 0 | dormant |
| 247 | `Lint/UnescapedBracketInRegexp` | 0 | 0 | 0 | dormant |
| 246 | `Lint/UnderscorePrefixedVariableName` | 2495260 | 1 | 1 | mismatch |
| 245 | `Lint/TripleQuotes` | 9 | 0 | 0 | mismatch |
| 244 | `Lint/TrailingCommaInAttributeDeclaration` | 84 | 0 | 0 | mismatch |
| 243 | `Lint/TopLevelReturnWithArgument` | 4986 | 0 | 0 | mismatch |
| 242 | `Lint/ToJSON` | 0 | 0 | 0 | dormant |
| 241 | `Lint/ToEnumArguments` | 0 | 0 | 0 | dormant |
| 240 | `Lint/Syntax` | 0 | 0 | 0 | dormant |
| 239 | `Lint/SymbolConversion` | 38 | 1204 | 0 | mismatch |
| 238 | `Lint/SuppressedExceptionInNumberConversion` | 0 | 3 | 0 | mismatch |
| 237 | `Lint/SuppressedException` | 75 | 72 | 71 | mismatch |
| 236 | `Lint/StructNewOverride` | 1 | 0 | 0 | mismatch |
| 235 | `Lint/SharedMutableDefault` | 1 | 1 | 1 | exact_active |
| 234 | `Lint/ShadowingOuterLocalVariable` | 22 | 172 | 0 | mismatch |
| 233 | `Lint/ShadowedException` | 1 | 11 | 0 | mismatch |
| 232 | `Lint/ShadowedArgument` | 97 | 0 | 0 | mismatch |
| 231 | `Lint/SendWithMixinArgument` | 0 | 0 | 0 | dormant |
| 230 | `Lint/SelfAssignment` | 0 | 0 | 0 | dormant |
| 229 | `Lint/ScriptPermission` | 0 | 0 | 0 | dormant |
| 228 | `Lint/SafeNavigationWithEmpty` | 1 | 0 | 0 | mismatch |
| 227 | `Lint/SafeNavigationConsistency` | 336 | 1 | 0 | mismatch |
| 226 | `Lint/SafeNavigationChain` | 202 | 1 | 0 | mismatch |
| 225 | `Lint/ReturnInVoidContext` | 17 | 0 | 0 | mismatch |
| 224 | `Lint/RescueType` | 0 | 0 | 0 | dormant |
| 223 | `Lint/RescueException` | 8 | 0 | 0 | mismatch |
| 222 | `Lint/RequireRelativeSelfPath` | 43 | 0 | 0 | mismatch |
| 221 | `Lint/RequireRangeParentheses` | 212 | 0 | 0 | mismatch |
| 220 | `Lint/RequireParentheses` | 0 | 0 | 0 | dormant |
| 219 | `Lint/RegexpAsCondition` | 0 | 0 | 0 | dormant |
| 218 | `Lint/RefinementImportMethods` | 2 | 0 | 0 | mismatch |
| 217 | `Lint/RedundantWithObject` | 0 | 0 | 0 | dormant |
| 216 | `Lint/RedundantWithIndex` | 0 | 0 | 0 | dormant |
| 215 | `Lint/RedundantTypeConversion` | 0 | 6 | 0 | mismatch |
| 214 | `Lint/RedundantStringCoercion` | 1 | 0 | 0 | mismatch |
| 213 | `Lint/RedundantSplatExpansion` | 12 | 20 | 0 | mismatch |
| 212 | `Lint/RedundantSafeNavigation` | 0 | 0 | 0 | dormant |
| 211 | `Lint/RedundantRequireStatement` | 0 | 0 | 0 | dormant |
| 210 | `Lint/RedundantRegexpQuantifiers` | 2 | 0 | 0 | mismatch |
| 209 | `Lint/RedundantDirGlobSort` | 0 | 0 | 0 | dormant |
| 208 | `Lint/RedundantCopEnableDirective` | 1061 | 2 | 2 | mismatch |
| 207 | `Lint/RedundantCopDisableDirective` | — | — | — | rubocop_error |
| 206 | `Lint/RandOne` | 0 | 0 | 0 | dormant |
| 205 | `Lint/RaiseException` | 1 | 0 | 0 | mismatch |
| 204 | `Lint/PercentSymbolArray` | 0 | 0 | 0 | dormant |
| 203 | `Lint/PercentStringArray` | 35 | 0 | 0 | mismatch |
| 202 | `Lint/ParenthesesAsGroupedExpression` | 1 | 0 | 0 | mismatch |
| 201 | `Lint/OutOfRangeRegexpRef` | 29 | 0 | 0 | mismatch |
| 200 | `Lint/OrderedMagicComments` | 1 | 0 | 0 | mismatch |
| 199 | `Lint/OrAssignmentToConstant` | 1 | 1 | 1 | exact_active |
| 198 | `Lint/NumericOperationWithConstantResult` | 0 | 0 | 0 | dormant |
| 197 | `Lint/NumberedParameterAssignment` | 0 | 0 | 0 | dormant |
| 196 | `Lint/NumberConversion` | 2989 | 2679 | 2677 | mismatch |
| 195 | `Lint/NonLocalExitFromIterator` | 1 | 0 | 0 | mismatch |
| 194 | `Lint/NonDeterministicRequireOrder` | 0 | 0 | 0 | dormant |
| 193 | `Lint/NonAtomicFileOperation` | 2 | 6 | 0 | mismatch |
| 192 | `Lint/NoReturnInBeginEndBlocks` | 0 | 0 | 0 | dormant |
| 191 | `Lint/NextWithoutAccumulator` | 0 | 0 | 0 | dormant |
| 190 | `Lint/NestedPercentLiteral` | 3 | 0 | 0 | mismatch |
| 189 | `Lint/NestedMethodDefinition` | 0 | 0 | 0 | dormant |
| 188 | `Lint/MultipleComparison` | 0 | 0 | 0 | dormant |
| 187 | `Lint/MixedRegexpCaptureTypes` | 4 | 0 | 0 | mismatch |
| 186 | `Lint/MixedCaseRange` | 0 | 0 | 0 | dormant |
| 185 | `Lint/MissingSuper` | 357 | 356 | 355 | mismatch |
| 184 | `Lint/MissingCopEnableDirective` | 1206 | 59 | 17 | mismatch |
| 183 | `Lint/Loop` | 5 | 5 | 5 | exact_active |
| 182 | `Lint/LiteralInInterpolation` | 0 | 0 | 0 | dormant |
| 181 | `Lint/LiteralAssignmentInCondition` | 0 | 0 | 0 | dormant |
| 180 | `Lint/LiteralAsCondition` | 29 | 2 | 0 | mismatch |
| 179 | `Lint/LambdaWithoutLiteralBlock` | 0 | 0 | 0 | dormant |
| 178 | `Lint/ItWithoutArgumentsInBlock` | 0 | 0 | 0 | dormant |
| 177 | `Lint/InterpolationCheck` | 2448 | 1 | 1 | mismatch |
| 176 | `Lint/InheritException` | 5 | 0 | 0 | mismatch |
| 175 | `Lint/IneffectiveAccessModifier` | 63 | 53 | 53 | mismatch |
| 174 | `Lint/IncompatibleIoSelectWithFiberScheduler` | 0 | 0 | 0 | dormant |
| 173 | `Lint/ImplicitStringConcatenation` | 1 | 1 | 1 | exact_active |
| 172 | `Lint/IdentityComparison` | 0 | 0 | 0 | dormant |
| 171 | `Lint/HeredocMethodCallPosition` | 5 | 0 | 0 | mismatch |
| 170 | `Lint/HashNewWithKeywordArgumentsAsDefault` | 27 | 0 | 0 | mismatch |
| 169 | `Lint/HashCompareByIdentity` | 6 | 0 | 0 | mismatch |
| 168 | `Lint/FormatParameterMismatch` | 11 | 0 | 0 | mismatch |
| 167 | `Lint/FloatOutOfRange` | 0 | 0 | 0 | dormant |
| 166 | `Lint/FloatComparison` | 0 | 0 | 0 | dormant |
| 165 | `Lint/FlipFlop` | 0 | 0 | 0 | dormant |
| 164 | `Lint/ErbNewArguments` | 0 | 0 | 0 | dormant |
| 163 | `Lint/EnsureReturn` | 0 | 0 | 0 | dormant |
| 162 | `Lint/EmptyWhen` | 13 | 0 | 0 | mismatch |
| 161 | `Lint/EmptyInterpolation` | 0 | 0 | 0 | dormant |
| 160 | `Lint/EmptyInPattern` | 0 | 0 | 0 | dormant |
| 159 | `Lint/EmptyFile` | 2 | 1 | 0 | mismatch |
| 158 | `Lint/EmptyExpression` | 0 | 0 | 0 | dormant |
| 157 | `Lint/EmptyEnsure` | 51 | 0 | 0 | mismatch |
| 156 | `Lint/EmptyConditionalBody` | 3 | 0 | 0 | mismatch |
| 155 | `Lint/EmptyClass` | 36 | 0 | 0 | mismatch |
| 154 | `Lint/EmptyBlock` | 359 | 332 | 301 | mismatch |
| 153 | `Lint/ElseLayout` | 80 | 0 | 0 | mismatch |
| 152 | `Lint/EachWithObjectArgument` | 0 | 0 | 0 | dormant |
| 151 | `Lint/DuplicateSetElement` | 0 | 1 | 0 | mismatch |
| 150 | `Lint/DuplicateRescueException` | 527 | 0 | 0 | mismatch |
| 149 | `Lint/DuplicateRequire` | 23 | 0 | 0 | mismatch |
| 148 | `Lint/DuplicateRegexpCharacterClassElement` | 180361 | 0 | 0 | mismatch |
| 147 | `Lint/DuplicateMethods` | 7 | 10 | 0 | mismatch |
| 146 | `Lint/DuplicateMatchPattern` | 2 | 0 | 0 | mismatch |
| 145 | `Lint/DuplicateMagicComment` | 0 | 0 | 0 | dormant |
| 144 | `Lint/DuplicateHashKey` | 31566 | 0 | 0 | mismatch |
| 143 | `Lint/DuplicateElsifCondition` | 0 | 0 | 0 | dormant |
| 142 | `Lint/DuplicateCaseCondition` | 0 | 0 | 0 | dormant |
| 141 | `Lint/DuplicateBranch` | 0 | 50 | 0 | mismatch |
| 140 | `Lint/DisjunctiveAssignmentInConstructor` | 11 | 0 | 0 | mismatch |
| 139 | `Lint/DeprecatedOpenSSLConstant` | 0 | 0 | 0 | dormant |
| 138 | `Lint/DeprecatedConstants` | 212 | 0 | 0 | mismatch |
| 137 | `Lint/DeprecatedClassMethods` | 0 | 0 | 0 | dormant |
| 136 | `Lint/Debugger` | 2 | 0 | 0 | mismatch |
| 135 | `Lint/DataDefineOverride` | 0 | 0 | 0 | dormant |
| 134 | `Lint/CopDirectiveSyntax` | 1826 | 30 | 6 | mismatch |
| 133 | `Lint/ConstantResolution` | 68 | 240309 | 4 | mismatch |
| 132 | `Lint/ConstantReassignment` | 18 | 0 | 0 | mismatch |
| 131 | `Lint/ConstantOverwrittenInRescue` | 2 | 0 | 0 | mismatch |
| 130 | `Lint/ConstantDefinitionInBlock` | 9649 | 6 | 0 | mismatch |
| 129 | `Lint/CircularArgumentReference` | 0 | 0 | 0 | dormant |
| 128 | `Lint/BooleanSymbol` | 5 | 0 | 0 | mismatch |
| 127 | `Lint/BinaryOperatorWithIdenticalOperands` | 1352 | 1354 | 1350 | mismatch |
| 126 | `Lint/BigDecimalNew` | 0 | 0 | 0 | dormant |
| 125 | `Lint/AssignmentInCondition` | 1 | 164 | 0 | mismatch |
| 124 | `Lint/ArrayLiteralInRegexp` | 0 | 0 | 0 | dormant |
| 123 | `Lint/AmbiguousRegexpLiteral` | 0 | 0 | 0 | dormant |
| 122 | `Lint/AmbiguousRange` | 72 | 29 | 3 | mismatch |
| 121 | `Lint/AmbiguousOperatorPrecedence` | 186 | 3 | 3 | mismatch |
| 120 | `Lint/AmbiguousOperator` | 577 | 0 | 0 | mismatch |
| 119 | `Lint/AmbiguousBlockAssociation` | 3163 | 3133 | 3085 | mismatch |
| 118 | `Lint/AmbiguousAssignment` | 0 | 0 | 0 | dormant |
| 117 | `Layout/TrailingWhitespace` | 7 | 7 | 7 | exact_active |
| 116 | `Layout/TrailingEmptyLines` | 1 | 1 | 1 | exact_active |
| 115 | `Layout/SpaceInsideStringInterpolation` | 46 | 0 | 0 | mismatch |
| 114 | `Layout/SpaceInsideReferenceBrackets` | 35 | 0 | 0 | mismatch |
| 113 | `Layout/SpaceInsideRangeLiteral` | 1072 | 0 | 0 | mismatch |
| 112 | `Layout/SpaceInsidePercentLiteralDelimiters` | 0 | 0 | 0 | dormant |
| 111 | `Layout/SpaceInsideParens` | 83 | 1 | 0 | mismatch |
| 110 | `Layout/SpaceInsideHashLiteralBraces` | 64 | 5 | 0 | mismatch |
| 109 | `Layout/SpaceInsideBlockBraces` | 0 | 1 | 0 | mismatch |
| 108 | `Layout/SpaceInsideArrayPercentLiteral` | 0 | 0 | 0 | dormant |
| 107 | `Layout/SpaceInsideArrayLiteralBrackets` | 28 | 9 | 0 | mismatch |
| 106 | `Layout/SpaceInLambdaLiteral` | 20 | 0 | 0 | mismatch |
| 105 | `Layout/SpaceBeforeSemicolon` | 19 | 0 | 0 | mismatch |
| 104 | `Layout/SpaceBeforeFirstArg` | 603303 | 0 | 0 | mismatch |
| 103 | `Layout/SpaceBeforeComment` | 2114 | 0 | 0 | mismatch |
| 102 | `Layout/SpaceBeforeComma` | 156 | 0 | 0 | mismatch |
| 101 | `Layout/SpaceBeforeBrackets` | 3010 | 0 | 0 | mismatch |
| 100 | `Layout/SpaceBeforeBlockBraces` | 12615 | 0 | 0 | mismatch |
| 99 | `Layout/SpaceAroundOperators` | 8229 | 5 | 0 | mismatch |
| 98 | `Layout/SpaceAroundMethodCallOperator` | 0 | 0 | 0 | dormant |
| 97 | `Layout/SpaceAroundKeyword` | 0 | 0 | 0 | dormant |
| 96 | `Layout/SpaceAroundEqualsInParameterDefault` | 288 | 0 | 0 | mismatch |
| 95 | `Layout/SpaceAroundBlockParameters` | 6 | 0 | 0 | mismatch |
| 94 | `Layout/SpaceAfterSemicolon` | 773 | 0 | 0 | mismatch |
| 93 | `Layout/SpaceAfterNot` | 16249 | 0 | 0 | mismatch |
| 92 | `Layout/SpaceAfterMethodName` | 5 | 0 | 0 | mismatch |
| 91 | `Layout/SpaceAfterComma` | 2733 | 1 | 1 | mismatch |
| 90 | `Layout/SpaceAfterColon` | 0 | 0 | 0 | dormant |
| 89 | `Layout/SingleLineBlockChain` | 24915 | 18448 | 18443 | mismatch |
| 88 | `Layout/RescueEnsureAlignment` | 1 | 78 | 0 | mismatch |
| 87 | `Layout/RedundantLineBreak` | 0 | 10179 | 0 | mismatch |
| 86 | `Layout/ParameterAlignment` | 120505 | 5 | 0 | mismatch |
| 85 | `Layout/MultilineOperationIndentation` | 289 | 452 | 0 | mismatch |
| 84 | `Layout/MultilineMethodParameterLineBreaks` | 195320 | 569 | 346 | mismatch |
| 83 | `Layout/MultilineMethodDefinitionBraceLayout` | 0 | 106 | 0 | mismatch |
| 82 | `Layout/MultilineMethodCallIndentation` | 289 | 6629 | 0 | mismatch |
| 81 | `Layout/MultilineMethodCallBraceLayout` | 34706 | 3050 | 0 | mismatch |
| 80 | `Layout/MultilineMethodArgumentLineBreaks` | 195320 | 24091 | 7609 | mismatch |
| 79 | `Layout/MultilineHashKeyLineBreaks` | 2099 | 1577 | 1542 | mismatch |
| 78 | `Layout/MultilineHashBraceLayout` | 25167 | 0 | 0 | mismatch |
| 77 | `Layout/MultilineBlockLayout` | 9702 | 0 | 0 | mismatch |
| 76 | `Layout/MultilineAssignmentLayout` | 895 | 4031 | 0 | mismatch |
| 75 | `Layout/MultilineArrayLineBreaks` | 993 | 5689 | 972 | mismatch |
| 74 | `Layout/MultilineArrayBraceLayout` | 5530 | 0 | 0 | mismatch |
| 73 | `Layout/LineLength` | 19373 | 81224 | 0 | mismatch |
| 72 | `Layout/LineEndStringConcatenationIndentation` | 296 | 2114 | 0 | mismatch |
| 71 | `Layout/LineContinuationSpacing` | 58 | 181 | 0 | mismatch |
| 70 | `Layout/LineContinuationLeadingSpace` | 0 | 0 | 0 | dormant |
| 69 | `Layout/LeadingEmptyLines` | 0 | 0 | 0 | dormant |
| 68 | `Layout/LeadingCommentSpace` | 0 | 14 | 0 | mismatch |
| 67 | `Layout/InitialIndentation` | 0 | 0 | 0 | dormant |
| 66 | `Layout/IndentationWidth` | 9010 | 2 | 0 | mismatch |
| 65 | `Layout/IndentationStyle` | 0 | 0 | 0 | dormant |
| 64 | `Layout/IndentationConsistency` | 9010 | 0 | 0 | mismatch |
| 63 | `Layout/HeredocIndentation` | 10271 | 1044 | 0 | mismatch |
| 62 | `Layout/HeredocArgumentClosingParenthesis` | 7 | 0 | 0 | mismatch |
| 61 | `Layout/HashAlignment` | 163 | 5807 | 0 | mismatch |
| 60 | `Layout/FirstParameterIndentation` | 0 | 0 | 0 | dormant |
| 59 | `Layout/FirstMethodParameterLineBreak` | 19 | 6 | 6 | mismatch |
| 58 | `Layout/FirstMethodArgumentLineBreak` | 12447 | 10358 | 10321 | mismatch |
| 57 | `Layout/FirstHashElementLineBreak` | 1719 | 1553 | 1553 | mismatch |
| 56 | `Layout/FirstHashElementIndentation` | 219 | 3616 | 0 | mismatch |
| 55 | `Layout/FirstArrayElementLineBreak` | 31988 | 1213 | 997 | mismatch |
| 54 | `Layout/FirstArrayElementIndentation` | 296 | 1725 | 0 | mismatch |
| 53 | `Layout/FirstArgumentIndentation` | 289 | 365 | 0 | mismatch |
| 52 | `Layout/ExtraSpacing` | 321 | 4 | 0 | mismatch |
| 51 | `Layout/EndOfLine` | 0 | 0 | 0 | dormant |
| 50 | `Layout/EndAlignment` | 17157 | 1 | 0 | mismatch |
| 49 | `Layout/EmptyLinesAroundModuleBody` | 9 | 9 | 9 | exact_active |
| 48 | `Layout/EmptyLinesAroundMethodBody` | 1 | 1 | 1 | exact_active |
| 47 | `Layout/EmptyLinesAroundExceptionHandlingKeywords` | 359 | 101 | 101 | mismatch |
| 46 | `Layout/EmptyLinesAroundClassBody` | 11 | 10 | 10 | mismatch |
| 45 | `Layout/EmptyLinesAroundBlockBody` | 0 | 0 | 0 | dormant |
| 44 | `Layout/EmptyLinesAroundBeginBody` | 0 | 0 | 0 | dormant |
| 43 | `Layout/EmptyLinesAroundAttributeAccessor` | 34 | 0 | 0 | mismatch |
| 42 | `Layout/EmptyLinesAroundArguments` | 0 | 0 | 0 | dormant |
| 41 | `Layout/EmptyLinesAroundAccessModifier` | 0 | 0 | 0 | dormant |
| 40 | `Layout/EmptyLinesAfterModuleInclusion` | 362 | 317 | 0 | mismatch |
| 39 | `Layout/EmptyLines` | 18 | 4 | 4 | mismatch |
| 38 | `Layout/EmptyLineBetweenDefs` | 6 | 6 | 0 | mismatch |
| 37 | `Layout/EmptyLineAfterMultilineCondition` | 125 | 157 | 0 | mismatch |
| 36 | `Layout/EmptyLineAfterMagicComment` | 49 | 49 | 49 | exact_active |
| 35 | `Layout/EmptyLineAfterGuardClause` | 513 | 277 | 0 | mismatch |
| 34 | `Layout/EmptyComment` | 20 | 0 | 0 | mismatch |
| 33 | `Layout/ElseAlignment` | 0 | 1 | 0 | mismatch |
| 32 | `Layout/DotPosition` | 12587 | 13 | 0 | mismatch |
| 31 | `Layout/DefEndAlignment` | 93932 | 1 | 0 | mismatch |
| 30 | `Layout/ConditionPosition` | 4 | 0 | 0 | mismatch |
| 29 | `Layout/CommentIndentation` | 7044 | 0 | 0 | mismatch |
| 28 | `Layout/ClosingParenthesisIndentation` | 0 | 0 | 0 | dormant |
| 27 | `Layout/ClosingHeredocIndentation` | 2 | 0 | 0 | mismatch |
| 26 | `Layout/ClassStructure` | 1069 | 452 | 1 | mismatch |
| 25 | `Layout/CaseIndentation` | 0 | 0 | 0 | dormant |
| 24 | `Layout/BlockEndNewline` | 365486 | 0 | 0 | mismatch |
| 23 | `Layout/BlockAlignment` | 1 | 0 | 0 | mismatch |
| 22 | `Layout/BeginEndAlignment` | 1064 | 0 | 0 | mismatch |
| 21 | `Layout/AssignmentIndentation` | 109 | 0 | 0 | mismatch |
| 20 | `Layout/ArrayAlignment` | 296 | 1553 | 0 | mismatch |
| 19 | `Layout/ArgumentAlignment` | 296 | 17064 | 0 | mismatch |
| 18 | `Layout/AccessModifierIndentation` | 0 | 0 | 0 | dormant |
| 17 | `Gemspec/RubyVersionGlobalsUsage` | 33 | 0 | 0 | mismatch |
| 16 | `Gemspec/RequiredRubyVersion` | 19 | 0 | 0 | mismatch |
| 15 | `Gemspec/RequireMFA` | 8 | 0 | 0 | mismatch |
| 14 | `Gemspec/OrderedDependencies` | — | — | — | crash |
| 13 | `Gemspec/DuplicatedAssignment` | 660 | 0 | 0 | mismatch |
| 12 | `Gemspec/DevelopmentDependencies` | 0 | 0 | 0 | dormant |
| 11 | `Gemspec/DeprecatedAttributeAssignment` | 4 | 0 | 0 | mismatch |
| 10 | `Gemspec/DependencyVersion` | 0 | 0 | 0 | dormant |
| 9 | `Gemspec/AttributeAssignment` | 0 | 0 | 0 | dormant |
| 8 | `Gemspec/AddRuntimeDependency` | 3 | 0 | 0 | mismatch |
| 7 | `Bundler/OrderedGems` | 2 | 0 | 0 | mismatch |
| 6 | `Bundler/InsecureProtocolSource` | 0 | 0 | 0 | dormant |
| 5 | `Bundler/GemVersion` | 6 | 0 | 0 | mismatch |
| 4 | `Bundler/GemFilename` | 0 | 0 | 0 | dormant |
| 3 | `Bundler/GemComment` | 0 | 0 | 0 | dormant |
| 2 | `Bundler/DuplicatedGroup` | 0 | 0 | 0 | dormant |
| 1 | `Bundler/DuplicatedGem` | 0 | 0 | 0 | dormant |
