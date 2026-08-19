# Mixed native and custom-cop benchmark

Rustocop can delegate explicitly selected Ruby custom cops to RuboCop while
keeping recognized built-in cops native. The two inspections start
concurrently and Rustocop merges RuboCop's custom diagnostics into its normal
report.

## Result

Measured on 2026-08-19:

| Variant | Median | p95 | Relative to native binary |
| --- | ---: | ---: | ---: |
| Rustocop native binary, 20 built-in cops | **9.07 ms** | 9.19 ms | 1.0× |
| Rustocop Ruby entrypoint, 20 built-in cops | 85.01 ms | 87.38 ms | 9.4× |
| Mixed native binary, 20 native + 1 custom cop | **456.12 ms** | 463.33 ms | 50.3× |
| Mixed Ruby entrypoint, 20 native + 1 custom cop | 531.26 ms | 536.79 ms | 58.6× |
| RuboCop, custom cop only | 446.74 ms | 454.94 ms | 49.3× |
| RuboCop, all 20 built-ins + custom cop | 478.47 ms | 485.41 ms | 52.8× |

The direct mixed run was 4.7% faster than pure RuboCop and produced identical
normalized JSON. It was only 2.1% slower than asking RuboCop to run the custom
cop alone, which shows that concurrent native inspection adds little to the
unavoidable Ruby work.

The consequence is still severe: one Ruby custom cop makes this tiny-corpus run
about 51 times slower than pure native Rustocop. RuboCop must start Ruby, load
its framework and the custom file, read all 500 files, and build its own Prism
trees. Rustocop cannot reuse its native Prism trees across that process
boundary.

The Ruby `exe/rustocop` entrypoint adds roughly 75 ms of Ruby startup before
it replaces itself with the native binary. That existing packaging overhead
makes the entrypoint mixed run slower than invoking RuboCop directly on this
startup-dominated corpus. Use `libexec/rustocop-native` when measuring the
native engine itself.

## Method

The benchmark uses the committed 500-file compatibility corpus: 20 built-in
cops, 9,090 bytes of Ruby, and one synthetic custom cop that raises exactly one
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
