# Non-scalable cop implementations

`updated_at: 2026-08-23T22:35:26-04:00`

This is the catalog for category C: cops whose current implementation appears
too narrow to generalize from the fixture corpus to arbitrary Ruby projects.
It is an implementation-risk register, not a list of every cop that currently
mismatches RuboCop.

All 32 category-C cops in this catalog are intentionally pending. A further 9
project-inexact cops and two RuboCop-reference blockers are also pending, for a
total of 43; they are absent from
the active registry, qualification corpus, compatibility evidence, and fixture
suite. Sixty-one cops have now returned through structural rewrites and the full
fixture gate, with project status recorded separately. The machine-readable source of truth is
[`intentionally_pending_cops.yml`](../spec/upstream/rubocop-1.87.0/intentionally_pending_cops.yml).
Their old implementation source remains only as rewrite reference.

The additional withdrawals were established by the complete audit generated at
`2026-08-23T09:13:26-04:00`. They comprise every cop classified as mismatch in
the preceding 558-cop audit, plus `Lint/RedundantCopDisableDirective` and
`Style/FileWrite`, for which RuboCop 1.87 cannot produce stable isolated project
reference output. On that original ten-project corpus, the resulting 512-cop
active set had 402 exercised project-exact cops, 110 dormant cops, and no
mismatches or engine errors.

The expanded 50-project audit at `2026-08-23T22:35:26-04:00` shows that this
withdrawal was not sufficient to make the retained implementations scalable:
317 cops are project-exact, 53 are dormant, 192 mismatch, none crash, and one
is blocked by a RuboCop error. Those newly exposed cops require review and
minimized fixtures before this catalog can be treated as complete.

The current evidence snapshots are
`spec/compatibility_evidence/projects.json` and
`spec/compatibility_evidence/fixtures.json`. The project corpus contains 85,471
Ruby files. “Gap” is Rustocop-only plus RuboCop-only diagnostic signatures.

## Inclusion criteria

A cop is included when code review establishes at least one of these patterns:

1. It is registered through `catalog_cop::report` or `catalog_cop::replace`,
   which performs a literal search or replacement at code-masked offsets but
   does not verify the Ruby construct represented by the text.
2. It substitutes whole-source or line-oriented string matching for Ruby AST,
   lexical-state, scope, or regexp semantics, and the project audit confirms a
   mismatch.
3. Several distinct cops are wired to the same placeholder callback even
   though RuboCop gives them different syntax and configuration semantics.
4. A metric is approximated by counting lines or substrings instead of walking
   the relevant AST and maintaining RuboCop-compatible scope.

This deliberately excludes cops that have large project gaps but use a broad,
structural implementation. For example, `Style/MethodCallWithArgsParentheses`
is inaccurate, but it is not currently evidence of category C. A green fixture
row also does not remove a cop from this catalog: 16 of the cops below pass all
their fixtures while still mismatch real projects.

## Literal catalog reporters (7 cops)

These are mechanically confirmed. The shared implementation in
[`framework/catalog_cop.rs`](../crates/rustocop/src/cops/prism/framework/catalog_cop.rs)
calls `report_code(needle, ...)` or `replace_code(old, new, ...)` at every
code-masked literal match without checking the Ruby construct represented by
the text.

| Cop | Fixture result | Project result | Rustocop | RuboCop | Exact | Gap |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `Layout/LineContinuationLeadingSpace` | mismatch 11/32 | mismatch | 0 | 49 | 0 | 49 |
| `Layout/MultilineBlockLayout` | mismatch 7/30 | mismatch | 24,105 | 12 | 0 | 24,117 |
| `Layout/RedundantLineBreak` | mismatch 72/118 | mismatch | 0 | 14,996 | 0 | 14,996 |
| `Layout/SpaceAroundBlockParameters` | mismatch 19/45 | mismatch | 64 | 4 | 0 | 68 |
| `Layout/SpaceInsideHashLiteralBraces` | mismatch 20/40 | mismatch | 130 | 1,420 | 0 | 1,550 |
| `Lint/AmbiguousRegexpLiteral` | mismatch 6/30 | mismatch | 0 | 28 | 0 | 28 |
| `Lint/AssignmentInCondition` | mismatch 16/69 | mismatch | 4 | 1,276 | 0 | 1,280 |

Registrations are in
[`final_layout_batch_a/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a/registry.rs),
[`final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs),
[`final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs),
[`final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs),
[`final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs), and
[`final_control_flow_batch/registry.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch/registry.rs).

## Source scanners standing in for syntax or scope (15 cops)

These implementations scan delimiters, lines, assignments, identifiers, or
keywords without enough parser or scope context. The strongest examples are
`Lint/ConstantResolution`, which tests the trimmed entire file as if it were
one constant reference, and `Style/InlineComment`, which treats the first `#`
on a non-empty line as a comment.

| Cop | Fixture result | Project result | Rustocop | RuboCop | Exact | Gap |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `Style/InlineComment` | compatible 3/3 | mismatch | 129,107 | 10,115 | 9,470 | 120,282 |
| `Lint/ConstantResolution` | compatible 18/18 | mismatch | 111 | 544,649 | 29 | 544,702 |
| `Lint/DuplicateRegexpCharacterClassElement` | mismatch 6/16 | mismatch | 329,911 | 99 | 0 | 330,010 |
| `Layout/MultilineHashKeyLineBreaks` | compatible 10/10 | mismatch | 3,097 | 1,835 | 1,786 | 1,360 |
| `Layout/SingleLineBlockChain` | compatible 9/9 | mismatch | 40,204 | 25,643 | 25,335 | 15,177 |
| `Lint/UnusedMethodArgument` | mismatch 14/46 | mismatch | 3,902 | 1,933 | 777 | 4,281 |
| `Naming/VariableName` | mismatch 62/118 | mismatch | 73,576 | 123 | 42 | 73,615 |
| `Naming/VariableNumber` | mismatch 62/115 | mismatch | 0 | 8,050 | 0 | 8,050 |
| `Lint/UselessAssignment` | mismatch 82/149 | mismatch | 31,904 | 543 | 183 | 32,081 |
| `Style/Copyright` | compatible 21/21 | mismatch | 54,127 | 0 | 0 | 54,127 |

The relevant implementations are in
[`text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs),
[`text/layout.rs`](../crates/rustocop/src/cops/text/layout.rs),
[`text/lint_semantic.rs`](../crates/rustocop/src/cops/text/lint_semantic.rs),
[`semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs),
[`literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs),
[`final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs),
[`layout_line_break_completion.rs`](../crates/rustocop/src/cops/prism/layout_line_break_completion.rs),
[`final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs),
[`style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs),
[`source_rules_misc.rs`](../crates/rustocop/src/cops/prism/source_rules_misc.rs), and
[`source_semantics/parameters.rs`](../crates/rustocop/src/cops/prism/source_semantics/parameters.rs).

## Shared placeholder callbacks (6 cops)

Three brace-layout cops share one two-line `brace_layout` check, and three
unrelated alignment cops share one `align_continuation` check. Identical output
counts within each alignment group are a direct symptom of the shared
placeholder rather than coincidental behavior.

| Cop | Fixture result | Project result | Rustocop | RuboCop | Exact | Gap |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `Layout/MultilineArrayBraceLayout` | mismatch 16/35 | mismatch | 9,440 | 43 | 0 | 9,483 |
| `Layout/MultilineHashBraceLayout` | mismatch 16/34 | mismatch | 48,256 | 33 | 0 | 48,289 |
| `Layout/MultilineMethodCallBraceLayout` | mismatch 22/44 | mismatch | 74,680 | 3,268 | 0 | 77,948 |
| `Layout/LineEndStringConcatenationIndentation` | mismatch 39/59 | mismatch | 583 | 2,497 | 0 | 3,080 |

Both callbacks are in
[`final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs),
with registrations in its
[`registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a/registry.rs).

## Approximate metrics (5 cops)

These implementations count source lines or textual tokens and do not model
RuboCop's AST-based metric scopes and discount rules. `Metrics/AbcSize` starts
from an AST method node, but calculates assignments, branches, and conditions
from substring counts in the method body.

| Cop | Fixture result | Project result | Rustocop | RuboCop | Exact | Gap |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `Metrics/BlockNesting` | mismatch 7/26 | mismatch | 175 | 176 | 0 | 351 |
| `Metrics/PerceivedComplexity` | mismatch 15/31 | mismatch | 79 | 4,068 | 0 | 4,147 |
| `Metrics/CyclomaticComplexity` | mismatch 15/36 | mismatch | 105 | 4,982 | 0 | 5,087 |
| `Metrics/ClassLength` | mismatch 11/33 | mismatch | 121 | 3,148 | 0 | 3,269 |
| `Metrics/AbcSize` | compatible 24/24 | mismatch | 36,741 | 33,180 | 890 | 68,141 |

Implementations are in
[`final_metrics_batch.rs`](../crates/rustocop/src/cops/prism/final_metrics_batch.rs)
and [`metrics_completion.rs`](../crates/rustocop/src/cops/prism/metrics_completion.rs).

## Naive lexical spacing scanners (7 cops)

These all operate directly over bytes or lines using an `inside_quoted_text`
helper that only counts double quotes and checks for `<<-`. It cannot represent
Ruby's single-quoted strings, percent literals, regexp literals, interpolation,
comments, heredoc variants, or parser recovery.

| Cop | Fixture result | Project result | Rustocop | RuboCop | Exact | Gap |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `Layout/SpaceBeforeComment` | compatible 5/5 | mismatch | 4,519 | 0 | 0 | 4,519 |
| `Layout/SpaceAfterSemicolon` | compatible 9/9 | mismatch | 1,663 | 0 | 0 | 1,663 |
| `Layout/SpaceAfterComma` | compatible 9/9 | mismatch | 6,153 | 58 | 53 | 6,105 |
| `Layout/SpaceBeforeSemicolon` | compatible 9/9 | mismatch | 144 | 41 | 38 | 109 |
| `Layout/SpaceAfterNot` | compatible 6/6 | mismatch | 30,572 | 3 | 2 | 30,571 |
| `Layout/SpaceBeforeComma` | compatible 6/6 | mismatch | 193 | 0 | 0 | 193 |

All seven are implemented together in
[`source_rules_layout.rs`](../crates/rustocop/src/cops/prism/source_rules_layout.rs).

## Exit criteria

A cop stays in this catalog until its implementation no longer matches the
reason recorded above. Replacing a literal needle with a larger regexp or
adding project-specific exclusions is not sufficient. Removal requires:

1. syntax-aware or lexical-state-aware implementation appropriate to the cop;
2. positive, negative, configuration, and source-range fixtures derived from
   more than one real project;
3. all fixtures passing for the target and affected shared infrastructure; and
4. a focused 50-project run showing the target project-exact, or documenting
   a smaller residual gap whose cause is no longer a narrow implementation.

The project numbers are a snapshot, not manually maintained truth. Refresh
them from the cached RuboCop reference after each focused project run, and
always update this file's `updated_at` with a complete ISO 8601 timestamp.
