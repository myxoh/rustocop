# Mixed native and custom-cop benchmark

Rustocop can delegate explicitly selected Ruby custom cops to RuboCop while
keeping recognized built-in cops native. The two inspections start
concurrently and Rustocop merges RuboCop's custom diagnostics into its normal
report.

## Result

Recorded 2026-08-19 result:

<!-- generated:mixed-custom-results:start -->
| Variant | Median | p95 | Relative to native binary |
| --- | ---: | ---: | ---: |
| Rustocop native binary, 20 built-in cops | 17.54 ms | 18.50 ms | 1.0× |
| Rustocop Ruby entrypoint, 20 built-in cops | 122.01 ms | 125.10 ms | 7.0× |
| Mixed native binary, 20 native + 1 custom cop | 545.39 ms | 550.42 ms | 31.1× |
| Mixed Ruby entrypoint, 20 native + 1 custom cop | 638.75 ms | 657.74 ms | 36.4× |
| RuboCop, custom cop only | 527.37 ms | 534.53 ms | 30.1× |
| RuboCop, all 20 built-ins + custom cop | 555.00 ms | 563.15 ms | 31.6× |

The direct mixed run was 1.7% faster than pure RuboCop and produced
identical normalized JSON. One Ruby custom cop still made this tiny-corpus run
31.1 times slower than pure native Rustocop because RuboCop must
start Ruby, load the custom cop, and build a second set of Prism trees.
<!-- generated:mixed-custom-results:end -->

The Ruby `exe/rustocop` entrypoint adds roughly 75 ms of Ruby startup before
it replaces itself with the native binary. That existing packaging overhead
makes the entrypoint mixed run slower than invoking RuboCop directly on this
startup-dominated corpus. Use `libexec/rustocop-native` when measuring the
native engine itself.

## Method

The benchmark uses the pinned 500-file benchmark corpus: 20 built-in
cops, 9,110 bytes of Ruby, and one synthetic custom cop that raises exactly one
offense per file. RuboCop 1.87.0 uses Prism with its cache and server disabled.
Each result is the median of seven measured runs after two warmups.

Mixed and pure-RuboCop JSON reports are normalized and compared before timing.
The benchmark aborts if diagnostics differ or the custom cop does not report
exactly 500 offenses.

Reproduce it with:

```sh
bundle exec rake build:native
bundle exec ruby script/benchmark_mixed_custom_cop.rb
```

The machine-readable report is written to
`tmp/performance-verification/mixed-custom-cop-benchmark.json`.

## Interpretation

This is close to the worst case for mixed execution: the files are tiny, the
native rules are cheap, and RuboCop startup dominates. Larger projects or more
expensive built-in cops can benefit more from leaving those cops native, but a
Ruby custom cop always introduces a large fixed floor. Keeping frequently used
custom cops native remains the only way to retain pure-Rustocop latency.
