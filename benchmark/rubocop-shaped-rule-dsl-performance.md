# RuboCop-shaped rule DSL performance

Measured on 2026-08-19 after refactoring eight fixture-covered cops to
class-like rule objects, typed callback registration, and per-investigation
state. This is a revision-to-revision historical comparison, not a current
compatibility claim.

## Revisions

- Before: `5314902b997eaeffff9bccaa1241a92a3309c93b`
- After: `f320e68de63159c5e1e78d186c8a7c2244b5bbca`
- Cops: `Style/ZeroLengthPredicate`, `Style/YodaExpression`,
  `Style/YodaCondition`, `Style/YAMLFileRead`, `Style/WordArray`,
  `Style/WhileUntilModifier`, `Style/WhileUntilDo`, and `Style/WhenThen`

Both revisions were compiled with `cargo build --release`. Every timed command
used `--no-parallel --format json --only <the eight cops>`. Before and after
commands were alternated to limit ordering and thermal bias. Their JSON output
was compared before measurement and was identical.

Host: Apple M5 Pro, arm64, macOS 26.5.2, Rust 1.96.0.

## Results

| Workload | Runs | Before median | After median | Median change | Before p95 | After p95 |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| Pinned 500-file benchmark corpus | 60 | 9.488 ms | 9.518 ms | +0.3% | 11.629 ms | 11.634 ms |
| Node-dense synthetic file | 120 | 26.432 ms | 27.794 ms | +5.2% (+1.362 ms) | 27.109 ms | 28.273 ms |

The pinned corpus contains 500 very small files totalling roughly 9 KB. It is
primarily sensitive to startup, orchestration, and file handling. No meaningful
regression was detected there.

The synthetic workload was one 384,010-byte Ruby file containing 2,000 copies
of representative syntax for every refactored cop. It produced 16,000 offenses.
This deliberately amplifies per-node and per-cop dispatch overhead. Its 5.2%
regression was repeatable and is significant enough to retain as a worst-case
performance finding, although the absolute difference was 1.362 ms.

## Per-cop synthetic breakdown

Each cop was also measured alone for 30 alternating runs on the synthetic file.
These short timings are noisier than the combined 120-run result.

| Cop | Before median | After median | Change | Findings |
| --- | ---: | ---: | ---: | ---: |
| ZeroLengthPredicate | 14.187 ms | 14.209 ms | +0.2% | 2,000 |
| YodaExpression | 12.186 ms | 12.196 ms | +0.1% | 0 |
| YodaCondition | 13.594 ms | 13.741 ms | +1.1% | 2,000 |
| YAMLFileRead | 13.863 ms | 13.782 ms | -0.6% | 2,000 |
| WordArray | 14.123 ms | 14.071 ms | -0.4% | 2,000 |
| WhileUntilModifier | 15.819 ms | 16.144 ms | +2.1% | 4,000 |
| WhileUntilDo | 13.437 ms | 13.601 ms | +1.2% | 2,000 |
| WhenThen | 13.561 ms | 13.882 ms | +2.4% | 2,000 |

## Likely cause

The rule objects themselves are short-lived stack wrappers and should optimize
away. Typed source and range helpers are also ordinary inlined Rust methods.

The shared runner now calls `Cop::on_node_with_state`. Stateful rule cops
implement that callback directly. Stateless rule cops currently implement
`on_node`, so the trait's default `on_node_with_state` implementation forwards
to a second virtual callback. That extra dispatch occurs for every visited node
and every enabled stateless cop, matching the workload shape that exposes the
regression. Per-investigation state construction is another smaller fixed cost.

## Interpretation

The refactor did not cause a significant regression in the normal pinned
many-file workload. It did cause a modest but measurable regression in a
deliberately node-dense, offense-heavy workload. Future optimization should
first make stateless rule registrations implement `on_node_with_state`
directly, then repeat these two comparisons. The semantic and correction output
must remain byte-for-byte identical.
