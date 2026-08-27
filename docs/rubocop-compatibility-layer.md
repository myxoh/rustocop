# RuboCop compatibility layer

Rustocop targets RuboCop 1.87.0. The compatibility layer under
`crates/rustocop/src/rubocop/` translates RuboCop's shared implementation
boundaries into Rust while preserving names, control flow, and source
provenance closely enough for static review.

The layer is the implementation foundation for source-shaped cops. A cop
migration still has its own fixture and project-parity gates, but parser
translation, callback dispatch, configuration, ranges, comments, corrections,
and investigation lifecycle belong in the shared layer rather than in
cop-local substitutes.

## Completion status

The pinned source-shaped implementation audit is complete:

- All 228 source components are accounted for: 191 direct translations, 30
  native Rust equivalents, and 7 facilities documented as not applicable.
  There are no partial or pending components.
- All 2,586 Ruby APIs discovered from syntax and the pinned gems' actual
  runtime-defined method surface pass the strict gate. This includes readers,
  writers, `Struct` members, delegated methods, `define_method` hooks, and
  `class_eval`-generated callback families. A broad Rust
  helper cannot account for multiple distinct Ruby callbacks unless the Ruby
  APIs are actual aliases or generated equivalents. Same-named operations in a
  consolidated Rust file also require an exact source/API ownership declaration.
- Every public Rust target counted by that API ledger is exercised outside its
  own definition. Definition-only translations force their component back to
  `partial`; the current ledger contains zero unexercised public targets.
- All 83 discovered shared upstream spec files belong to components that pass
  the strict translation gate.
- The cached RSpec dry-run inventory contains all 3,139 expanded examples,
  including shared-example expansions that the previous source-line counter
  missed. Every individual RSpec ID is bound to one named executable Rust test,
  its upstream description hash, and either shared semantic terms or a
  source-reviewed explicit rule. The binding is protected by a checked SHA-256
  contract; 244 focused Rust test functions currently cover those files and
  the source-level branches that have no direct upstream example.
  Marker-only `= all` and suite-level all-to-all claims are not accepted by the
  manifest gate. The pinned upstream baseline executes
  3,135 examples successfully; four upstream examples tagged as broken on the
  parser backend are retained in the inventory and exercised by Rust's
  Prism-oriented contracts.
- Every source and spec entry records its upstream SHA-256 in
  `crates/rustocop/rubocop-translation.json`.

These figures describe the shared compatibility layer itself. The separate cop
migration audit is now complete for all 606 built-in cops, as recorded below;
that structural result does not turn a dormant project result into behavioral
evidence, so fixture and project classifications remain independently visible.

## Production adoption

As of `2026-08-25T07:41:06-04:00`, ten cops use translated shared behavior:

- `Layout/SpaceAfterComma` and `Layout/SpaceAfterSemicolon` use
  `SpaceAfterPunctuation`;
- `Style/TrailingCommaInArguments`, `Style/TrailingCommaInArrayLiteral`, and
  `Style/TrailingCommaInHashLiteral` use `TrailingComma`;
- `Style/HashSlice` and `Style/HashExcept` use `HashSubset`;
- `Style/Next` uses the minimum-body-length policy;
- `Style/OptionalBooleanParameter` uses `AllowedMethods`;
- `Style/NumericPredicate` uses `AllowedMethods` and `AllowedPattern`.

Their 649 cached RuboCop unit contracts pass. A scoped comparison against all
50 pinned projects classified all ten as `project_exact`, with 70,667 exact
offense signatures, zero mismatch signatures, and zero unmatched offenses.
Project-by-project diagnostic coverage is recorded in the generated
[compatibility-layer adoption report](rubocop-compatibility-adoption.md). The
consumer manifest distinguishes a cop merely being selected for a project from
the project actually producing a reference diagnostic through that cop.

The generated [progress report](rubocop-compatibility-progress.md) is the
current component-level ledger. Its `updated_at` value is always an ISO 8601
timestamp.

Cop migration is tracked separately in the machine-readable
[`rubocop-cop-migrations.json`](../crates/rustocop/rubocop-cop-migrations.json)
ledger. It pins each audited cop to its RuboCop source hash, callbacks, mixins,
native implementation, fixture/project evidence, structural similarity score,
and explicit gaps. An audited cop is not counted as migrated until those gaps
are resolved through the compatibility layer; behavioral parity alone does not
close the source-shape gate.

### Source-shaped migration checkpoint

As of `2026-08-27T00:19:48Z`, a second set of ten cops has been migrated through
the compatibility callback and investigation adapters:

- `Layout/DotPosition`, `Layout/EmptyLineBetweenDefs`,
  `Layout/EmptyLinesAroundAttributeAccessor`,
  `Layout/LineContinuationSpacing`, and `Layout/SpaceInsideRangeLiteral`;
- `Lint/EmptyWhen`, `Lint/LiteralInInterpolation`, `Lint/RescueType`, and
  `Lint/TrailingCommaInAttributeDeclaration`;
- `Style/RedundantRegexpCharacterClass`.

These implementations share one already-parsed `ProcessedSource`, RuboCop AST
and source ranges, callback dispatch, `RangeHelp`, `CommentsHelp`,
`AllowedMethods`, configuration access, directive handling, and corrector
operations. Parser-versus-Prism differences needed by these cops were fixed in
the shared AST adapter instead of being hidden in individual cops. The common
DSL now supports both node callbacks and RuboCop's source-wide
`on_new_investigation` lifecycle.

All 655 focused cached fixtures pass, as do all 29,585 cached fixtures across
the repository. A fresh run of the pinned upstream examples produced 638/638
matching cases for all ten cops. A fresh scoped comparison against the complete
50-project corpus classified eight exercised cops as `project_exact` and the
other two as dormant, with zero mismatch signatures and zero unmatched
offenses. The full generated compatibility table remains conservative: any
changed shared implementation invalidates older global evidence until the next
complete project refresh.

As of `2026-08-27T03:31:11Z`, the migration ledger covers 40 of the 606 built-in
cops. The latest source-shaped batch added 30 cops:

- `Layout/SpaceBeforeComment`, `Layout/SpaceAfterMethodName`,
  `Layout/SpaceAfterNot`, and `Layout/SpaceBeforeBrackets`;
- `Lint/FlipFlop`, `Lint/RescueException`, `Lint/DuplicateCaseCondition`,
  `Lint/EmptyExpression`, `Lint/UnifiedInteger`,
  `Lint/OrAssignmentToConstant`, `Lint/EmptyInterpolation`,
  `Lint/BooleanSymbol`, and `Lint/IdentityComparison`;
- `Security/MarshalLoad`;
- `Style/SymbolLiteral`, `Style/Send`, `Style/ImplicitRuntimeError`,
  `Style/SuperWithArgsParentheses`, `Style/StringMethods`,
  `Style/ColonMethodDefinition`, `Style/InlineComment`, `Style/WhenThen`,
  `Style/Proc`, `Style/ArrayJoin`, `Style/StringChars`,
  `Style/RedundantFileExtensionInRequire`, `Style/UnlessElse`,
  `Style/StderrPuts`, `Style/EnvHome`, and `Style/WhileUntilDo`.

The batch uses RuboCop-shaped callbacks and one shared investigation lifecycle;
the superseded native implementations were removed. Project-derived failures
strengthened shared boundaries for authoritative Prism comments, Parser-shaped
constant assignment nodes, heredoc interpolation structure, semicolon
locations, symbol-label ranges, and structural equality for invalid-byte
literals. They did not introduce project-specific branches.

All 282 cached unit cases and all 234 freshly captured upstream cases pass.
The final scoped comparison classified all 30 cops as `project_exact` across
the 50 pinned projects: 34,668 exact offense signatures, zero mismatch
signatures, and zero unmatched offenses.

As of `2026-08-27T00:11:24-04:00`, the migration ledger covers 48 of the 606
built-in cops. An attempted 100-cop cohort accepted eight source-shaped
migrations:

- `Lint/EmptyFile`, `Lint/EnsureReturn`, and `Lint/ToJSON`;
- `Style/ColonMethodCall`, `Style/NestedTernaryOperator`,
  `Style/NumberedParameters`, `Style/RedundantRegexpConstructor`, and
  `Style/StringHashKeys`.

All eight use the compatibility callback or investigation lifecycle and the
superseded implementations were removed. The shared layer gained Parser-shaped
physical-multiline heredoc/string normalization, `__FILE__` string values,
point-range formatting for global offenses, corrector wrapping, and a bounded
report filename for large focused batches. Two project-derived
`Style/StringHashKeys` cases were retained as controlled unit fixtures.

The eight cops pass 67 cached unit contracts, 52 freshly extracted upstream
cases, the complete 29,587-case fixture corpus, and all 50 projects with
132,163 exact offense signatures and no mismatch signatures. Four additional
punctuation migrations were explicitly rejected and rolled back: their unit
and upstream cases passed, but the project gate showed that the lightweight
compatibility token stream is not yet a complete replacement for RuboCop's
lexer. The other 88 candidates were not relabeled or modified after that
shared-layer boundary failed.

As of `2026-08-27T01:57:51-04:00`, the migration ledger covers 148 of the 606
built-in cops. The next 100 cops were migrated one at a time through the shared
compatibility callback or investigation lifecycle and are registered in
`compatibility_migration_batch_four.rs` and
`compatibility_migration_batch_five.rs`. Superseded registrations were removed
so the compatibility implementations are the production paths.

The cohort passes all 1,351 cached RuboCop unit contracts. The authoritative
scoped comparison against all 50 pinned projects classified all 70 exercised
cops as `project_exact`; the remaining 30 are dormant in this fixed corpus.
Rustocop and RuboCop produced the same 26,206 complete offense signatures, with
zero mismatch signatures, zero unmatched offenses, and no native crash. The
project-derived gaps strengthened shared Parser-shaped AST locations and body
semantics rather than adding repository-specific branches, including arbitrary
percent-array delimiters and physical-line handling for `begin` bodies.

The machine-readable migration inventory was regenerated at an ISO 8601
timestamp and now reports `148/606` audited and migrated cops. Each of these
rows retains the pinned RuboCop source hash, effective callbacks, native source
path, fixture/project evidence, structural classification, and documented
parser adaptation.

As of `2026-08-27T06:17:39-04:00`, the source-shaped migration audit is
complete: the machine-readable ledger reports `606/606` built-in cops audited
and migrated. The final 238-cop pass moved 171 homogeneous rule registrations
onto explicit compatibility-Prism DSL forms and records the remaining 67
stateful, multi-callback, source-wide, or text-engine adapters in a checked
implementation map. The generator rejects an adapter whose registered source
file moves, so bespoke dispatch cannot silently retain a stale audit claim.

The final 238-cop migration pass was first checked against all 50 pinned
projects in cohorts of at most 50: 228 cops were `project_exact`, 10 were
dormant, and none mismatched. A subsequent canonical 606-cop audit found two
previously hidden edge cases in the already-audited corpus. Both are now
isolated in provenance-backed unit fixtures: `Naming/HeredocDelimiterNaming`
ignores blank-heredoc-looking bytes inside comments and heredoc bodies, and
`Style/RedundantRegexpCharacterClass` handles single escape classes inside
multiline interpolated free-spacing regexps. The exhaustive focused rerun of
those two cops is exact across all 50 projects.

The refreshed canonical project evidence contains no mismatching built-in
cops: all 531 cops exercised by RuboCop are `project_exact`, and the remaining
75 are dormant on this fixed corpus. The completed tree passes all 29,608
cached unit contracts in 2.880 seconds. The earlier migration gates also found
and fixed contract boundaries in `Layout/IndentationWidth` and
`Style/Semicolon`; their minimized regressions remain in the unit corpus.

## Translation rules

1. Mirror the RuboCop source path and module boundary where practical.
2. Preserve constants, method names, branch order, and intermediate concepts.
3. Put native Rust facilities behind RuboCop-shaped interfaces rather than
   changing the translated algorithm.
4. Record the RuboCop version, source path, source SHA-256, translated tests,
   and every known deviation in `crates/rustocop/rubocop-translation.json`.
5. Account for every Ruby method found by the combined syntax and runtime
   inventory. A renamed or consolidated
   Rust operation must be recorded explicitly in the generator's API
   equivalence ledger. Its exact destination file and function are verified;
   unresolved APIs force the component back to `partial`. When multiple Ruby
   sources map to one Rust file and share a function name, the Rust source must
   declare which exact source/API pairs own that operation.
   Public Rust destinations must also have executable use outside their
   definition; definition-only targets are recorded as unresolved.
6. Port the corresponding RuboCop unit tests into Rust. Keep descriptions and
   cases recognizable. The manifest separately records each expanded upstream
   example ID, its description hash, the exact Rust test responsible for it,
   the mapping basis, explicit covered-example counts, and a digest over that
   complete binding. The binding is traceability evidence; both the pinned
   upstream suite and the Rust suite must still execute successfully. `partial`
   versus `translated` spec status remains independent so implementation
   coverage cannot be mistaken for test-port coverage.
7. Do not add project-derived special cases to this layer. Differences must be
   resolved by matching RuboCop's shared semantics.

## Status meanings

- `translated`: all behavior not listed in `deviations` has a Rust translation
  and focused tests.
- `partial`: the file is mapped and has working Rust behavior, but still has
  unaccounted implementation or upstream-test behavior and does not count as complete.
- `native`: Rust supplies the capability, exposed through a RuboCop-compatible
  interface with equivalent tests.
- `not_applicable`: the Ruby facility is unnecessary in Rust; the manifest must
  explain why.

The Rust manifest test verifies that every registered translation and test file
is present and carries matching provenance. The repository spec also compares
the recorded hashes with the pinned RuboCop or rubocop-ast gem and the vendored
upstream specs:

```console
bundle exec rspec spec/rubocop_translation_manifest_spec.rb
```

The implementation and all focused contract ports run with:

```console
cargo test --manifest-path crates/rustocop/Cargo.toml -- --test-threads=1
```

The exact registered upstream suites can be rerun independently with:

```console
bundle exec ruby script/run_rubocop_compatibility_upstream.rb
```

Both inventory generation and upstream execution activate the exact pinned gem
versions before loading RuboCop, so an additional installed version cannot
silently change the audited API surface or test behavior.

When either pinned upstream package changes, refresh the expanded RSpec example
inventory before regenerating the manifest:

```console
bundle exec ruby script/capture_rubocop_compatibility_examples.rb
ruby script/generate_rubocop_compatibility_inventory.rb
```
