# Performance against RuboCop with Parser and Prism

Re-measured after the package-boundary refactor on 2026-08-18
against RuboCop 1.87.0 and Prism 1.9.0. This is an interim benchmark over the
committed 500-file compatibility corpus and its 20 shared cops. It is not the
final 606-cop benchmark.

Every size was verified by comparing normalized JSON reports before timing.
All three variants were identical at every size. Rustocop was built in release
mode; the two RuboCop variants explicitly selected their parser engines:

```yaml
AllCops:
  ParserEngine: parser_whitequark # base RuboCop measurement
  # ParserEngine: parser_prism    # RuboCop + Prism measurement
  TargetRubyVersion: 3.4
  NewCops: enable
```

RuboCop 1.87 defaults to Prism for a Ruby 3.4 target, so explicitly selecting
`parser_whitequark` is required for a distinct base-parser comparison. All
variants ran with caching and server mode disabled and used the JSON formatter.
Timed output was discarded. Commands were rotated between variants, with 2–3
warmups followed by 7–30 measured runs depending on corpus size.

## Results

| Files | Runs | Rustocop median / p95 | RuboCop Parser median / p95 | RuboCop Prism median / p95 | Speedup vs Parser / Prism |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 30 | 2.918 / 3.745 ms | 457.718 / 650.142 ms | 447.560 / 594.765 ms | 156.86× / 153.38× |
| 25 | 20 | 3.308 / 3.605 ms | 450.066 / 459.468 ms | 451.635 / 457.558 ms | 136.05× / 136.53× |
| 100 | 12 | 4.447 / 4.938 ms | 465.004 / 493.982 ms | 457.083 / 466.280 ms | 104.57× / 102.78× |
| 500 | 7 | 9.448 / 12.166 ms | 524.489 / 539.847 ms | 516.258 / 533.897 ms | 55.51× / 54.64× |

## Interpretation

The one-file result is dominated by process startup. At 500 files, Prism makes
RuboCop about 2% faster than the Parser-gem engine, while rustocop remains about
55 times faster than either RuboCop configuration. The benchmark does not show
that parsing itself is 54 times faster: this tiny corpus measures the complete
CLI, configuration, file, traversal, and formatting paths together.

## Peak memory

Peak resident memory was measured separately with macOS `/usr/bin/time -l` on
the same files, 20 cops, configuration, JSON formatter, and sequential process
model. Each size had one warmup and seven measured runs, alternating rustocop
and RuboCop. Normalized JSON output was identical before measurement.

| Files | Rustocop sequential median / p95 | Rustocop parallel median / p95 | RuboCop + Prism median / p95 |
| ---: | ---: | ---: | ---: |
| 1 | 2.05 / 2.05 MiB | 2.05 / 2.05 MiB | 87.22 / 88.27 MiB |
| 25 | 2.55 / 2.61 MiB | 2.81 / 2.86 MiB | 87.48 / 88.05 MiB |
| 100 | 3.02 / 3.02 MiB | 3.14 / 3.19 MiB | 87.89 / 89.78 MiB |
| 500 | 3.70 / 3.72 MiB | 4.16 / 4.25 MiB | 89.30 / 92.45 MiB |

The nearly flat curves show that fixed runtime and startup cost dominate this
corpus. At 500 files, automatic parallel execution added about 0.46 MiB over
sequential rustocop and still used less than one twentieth of RuboCop's peak
RSS. This does not imply that arbitrary Ruby
files cost only a few KiB each: the committed corpus totals just 9,090 source
bytes. Large files, large literals, and more complex syntax need a separate
sustained-memory benchmark.

This is a useful baseline for parallelization. A thread pool should retain much
of rustocop's shared process footprint, while adding worker stacks and multiple
simultaneously live Prism trees. A process pool would repeat more of the fixed
footprint per worker. File-level threads are therefore the better first design,
but worker-count defaults should be validated on a corpus of realistically sized
application files rather than inferred from this small fixture set.

Reproduce the memory measurement with:

```sh
bundle exec ruby script/benchmark_memory.rb
```

The raw report is written to
`tmp/performance-verification/memory-benchmark.json`.

## File-level parallelization

`--parallel` uses the machine's available CPU count; `--jobs N` sets a fixed
worker count. The runner assigns complete files to scoped threads and restores
discovery order before formatting. Sequential and every parallel variant below
produced byte-identical JSON.

| Files | Sequential | 2 workers | 4 workers | 8 workers | Automatic |
| ---: | ---: | ---: | ---: | ---: | ---: |
| 25 | 2.545 ms | 2.450 ms / 1.04× | 2.396 ms / 1.06× | 2.498 ms / 1.02× | 2.613 ms / 0.97× |
| 100 | 3.391 ms | 3.275 ms / 1.04× | 3.122 ms / 1.09× | 3.259 ms / 1.04× | 3.460 ms / 0.98× |
| 500 | 9.680 ms | 8.688 ms / 1.11× | 7.707 ms / 1.26× | 8.329 ms / 1.16× | 8.521 ms / 1.14× |

The execution plan removed enough serial per-file work that parallelism now has
less to recover: four workers are fastest at 500 files (1.26×), while automatic
mode provides 1.14×. At 25 tiny files, worker startup outweighs useful work.
Parallel mode remains opt-in so small invocations do not pay that overhead.

Reproduce with:

```sh
bundle exec ruby script/benchmark_parallel.rb
```

The raw report is written to
`tmp/performance-verification/parallel-benchmark.json`.

## Timing interpretation

```mermaid
xychart-beta
    title "End-to-end speedup over RuboCop + Prism"
    x-axis "Ruby files" [1, 25, 100, 500]
    y-axis "Speedup (times)" 0 --> 160
    bar [153.38, 136.53, 102.78, 54.64]
```

At 500 files, median throughput was approximately 52,921 files/second for
Rustocop, 953 for RuboCop with Parser, and 968 for RuboCop with Prism. The
corpus is deliberately small—500 files totaling 9,090 bytes—so these figures
primarily measure CLI startup, configuration, parsing, dispatch, and formatter
overhead rather than sustained performance on large application files.

Environment: Apple M5 Pro (15 cores), 24 GB RAM, macOS arm64, Ruby 3.4.9,
Rust 1.96.0. The raw report is generated under
`tmp/performance-verification/rubocop-prism-benchmark.json`.

Reproduce with:

```sh
bundle exec ruby script/benchmark_rubocop_prism.rb
```
