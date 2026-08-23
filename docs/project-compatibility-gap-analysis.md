# Why project compatibility is not near 90%

The category-C implementation-risk register is maintained separately in
[`non-scalable-implementations.md`](non-scalable-implementations.md).

> [!NOTE]
> This analysis records the 606-cop checkpoint that motivated withdrawing 48
> non-scalable implementations. Those cops are now intentionally pending and
> excluded from the 558-cop active corpus; the figures below remain the
> pre-withdrawal diagnostic evidence.

The current retained fixture corpus reached 26,717/26,717 cases and 558/558 cops
at `2026-08-22T20:40:35-04:00`. That completes the fixture gate but does not
supersede this report's central distinction: complete upstream-spec parity and
complete real-project output parity are separate compatibility claims.

This report records the complete cached-reference audit generated at
`2026-08-22T00:56:18-04:00` for RuboCop 1.87.0. It covers 606 cops across ten
pinned projects containing 54,146 Ruby files. The Rust source is
`7f296411cbe9762b81306f8e45a0072d8b856b59`; the native binary SHA-256 is
`fb13931fadb0e490cbeebdf070f7cf69648d36376109499c608dda2220801ed3`.

## Verdict

Project compatibility is not near 90%:

| Measure | Result |
| --- | ---: |
| Project-exact cops, all registered cops | 285/606 (47.0%) |
| Project-exact cops, exercised cops | 285/517 (55.1%) |
| Dormant cops | 87/606 (14.4%) |
| Mismatching cops | 232/606 (38.3%) |
| Rust crashes | 1/606 |
| RuboCop gate errors | 1/606 |

Ninety percent of all 606 cops would require 546 project-exact cops. That is
impossible on the current corpus even if every mismatch were fixed, because 87
cops are dormant; the maximum without expanding the corpus is 517/606 (85.3%).
Ninety percent of the 517 exercised cops would require 466 exact cops, or 181
promotions from the current checkpoint.

The expectation of roughly 90% came from conflating fixture metrics with
project metrics. The fixture differential matches 25,652/28,618 executable
cases (89.6%) and 528/606 cops match every fixture (87.1%). Neither number means
that those cops match all diagnostics in real projects.

## What the previous iteration improved

The last fixture cycle promoted four cops to complete fixture compatibility,
but none could change the project classification:

| Cop | Project diagnostics | Project classification |
| --- | ---: | --- |
| `Lint/Debugger` | 1/1 exact | Project-exact already |
| `Lint/RedundantRegexpQuantifiers` | 0 | Dormant |
| `Style/NegatedIf` | 2,185/2,185 exact | Project-exact already |
| `Style/NegatedUnless` | 0 | Dormant |

The work was valid—it fixed evidence capture and real regexp behavior—but it
selected the smallest fixture gaps rather than project-impacting gaps. A green
RSpec suite also establishes that the harness and checked-in regressions pass;
it does not establish broad RuboCop compatibility.

## Evidence that fixture coverage is not representative enough

Of the 528 fixture-compatible cops:

- 280 are project-exact;
- 163 still mismatch projects;
- 84 are dormant;
- one crashes Rustocop.

Thus 30.9% of fixture-compatible cops still mismatch real projects. Conversely,
five cops with incomplete fixture compatibility are project-exact. The fixture
and project gates measure different surfaces: upstream examples emphasize
specified behavior and autocorrection, while the projects expose frequency,
negative cases, path rules, configuration interactions, recovered syntax, and
complete source-range behavior.

The minimized project corpus is intentionally a regression set, not a
representative sample. Its 126 passing cases preserve bugs already found, but
cannot approximate 54,146 files or the negative-case distribution needed to
control false positives.

## Shape and concentration of the remaining errors

Across all cops, Rustocop emitted 3,764,214 signatures, RuboCop emitted
2,698,019, and 1,909,482 were exact. This gives 50.7% precision, 70.8% recall,
and 41.9% exact-signature Jaccard overlap. The total unmatched signature gap is
2,643,269, so the low cop-level score is not merely a binary-gate artifact.

The 232 mismatching cops divide into:

| Mismatch shape | Cops |
| --- | ---: |
| Both extra and missing signatures | 143 |
| Rustocop-only signatures | 70 |
| RuboCop-only signatures | 19 |

The gap is highly concentrated: the largest three cops account for 54.8% of
all unmatched signatures, and the largest ten account for 74.3%.

| Cop | Rustocop | RuboCop | Exact | Signature gap |
| --- | ---: | ---: | ---: | ---: |
| `Style/MethodCallWithArgsParentheses` | 1,041,796 | 524,118 | 495,680 | 574,554 |
| `Lint/ConstantResolution` | 111 | 544,649 | 29 | 544,702 |
| `Lint/DuplicateRegexpCharacterClassElement` | 329,911 | 99 | 0 | 330,010 |
| `Style/InlineComment` | 129,107 | 10,115 | 9,470 | 120,282 |
| `Layout/MultilineMethodCallBraceLayout` | 74,680 | 3,268 | 0 | 77,948 |
| `Naming/VariableName` | 73,576 | 123 | 42 | 73,615 |
| `Layout/FirstArrayElementLineBreak` | 71,921 | 1,739 | 1,491 | 70,678 |
| `Metrics/AbcSize` | 36,741 | 33,180 | 890 | 68,141 |
| `Style/Copyright` | 54,127 | 0 | 0 | 54,127 |
| `Lint/DuplicateHashKey` | 48,709 | 0 | 0 | 48,709 |

Several of these are broad semantic gaps rather than isolated edge cases. For
example, `Style/InlineComment` uses a line-level first-`#` search, which cannot
reliably distinguish comments from all Ruby string and heredoc forms.
`Lint/ConstantResolution` currently reasons from the trimmed whole-file source
instead of visiting every relevant constant reference. These implementations
can pass narrow positive fixtures while producing very large real-project false
positive or false negative sets.

Department results show where this is most acute:

| Department | Registered | Exact | Mismatch | Dormant | Error/crash |
| --- | ---: | ---: | ---: | ---: | ---: |
| Layout | 100 | 26 | 66 | 7 | 1 |
| Lint | 154 | 53 | 63 | 37 | 1 |
| Metrics | 10 | 1 | 9 | 0 | 0 |
| Naming | 19 | 6 | 13 | 0 | 0 |
| Style | 298 | 193 | 77 | 28 | 0 |
| Security | 7 | 6 | 0 | 1 | 0 |

## Corrected iterative process

The fixture → fix → full-fixture loop remains necessary, but target selection
must start from real-project signature impact:

1. Fix the `Layout/FirstHashElementIndentation` crash first; crashes invalidate
   an entire cop gate.
2. Rank mismatches by unmatched signature gap, with false-positive-heavy cops
   ahead of low-volume edge cases.
3. For one target cop, sample both Rustocop-only and RuboCop-only signatures
   across multiple projects. Add minimized positive and negative fixtures that
   retain the project configuration, path, and parser context.
4. Fix the implementation at the AST/configuration level rather than adding
   source-pattern exceptions.
5. Run the target fixtures, all upstream fixtures for the target and adjacent
   cops sharing the implementation file, then the complete 28,618-case fixture
   differential.
6. Run a focused ten-project audit for the target cop. A fix is successful only
   when its complete signature gap falls without regressions.
7. Rerun the full project audit after each high-impact cluster, not only after
   fixture promotions.
8. Expand the project corpus or add synthetic project fixtures for the 87
   dormant cops before using an all-606 project-exact percentage as a target.

The immediate high-leverage cluster is the top ten table above, not the cops
with the fewest failing upstream examples. This changes the optimization target
from “number of fixture-compatible cops” to “real-project exact signatures and
project-exact promotions,” while retaining the full fixture run as the
regression gate.
