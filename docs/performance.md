# Performance against RuboCop with Prism

The current report compares RuboCop 1.87.0 with Prism 1.9.0. This benchmark
uses the pinned 500-file benchmark corpus and 20 shared verified cops;
it measures a representative local feedback path, not all 606 cops at once.

Every size was verified by comparing normalized JSON reports before timing.
Both tools were identical at every size. Rustocop was built in release mode;
RuboCop explicitly selected Prism to keep the benchmark configuration stable:

```yaml
AllCops:
  ParserEngine: parser_prism
  TargetRubyVersion: 3.4
  NewCops: enable
```

Both tools ran with caching and server mode disabled and used the JSON
formatter. Timed output was discarded. Commands were alternated, with 2–3
warmups followed by 7–30 measured runs depending on corpus size.

<!-- generated:rubocop-prism-results:start -->
## Results

| Files | Runs | Rustocop median / p95 | RuboCop Prism median / p95 | Speedup |
| ---: | ---: | ---: | ---: | ---: |
| 1 | 30 | 3.038 / 3.277 ms | 402.379 / 416.615 ms | 132.45× |
| 25 | 20 | 3.494 / 3.595 ms | 400.825 / 407.478 ms | 114.72× |
| 100 | 12 | 4.504 / 4.556 ms | 412.795 / 415.267 ms | 91.65× |
| 500 | 7 | 7.789 / 12.505 ms | 467.511 / 474.136 ms | 60.02× |

## Interpretation

The one-file result is dominated by process startup. At 500 files, rustocop is
about 60 times faster than RuboCop. This tiny corpus measures the complete CLI,
configuration, file, traversal, and formatting paths together—not parsing alone.
<!-- generated:rubocop-prism-results:end -->

## Peak memory

Peak resident memory was measured separately with macOS `/usr/bin/time -l` on
the same files, 20 cops, configuration, JSON formatter, and sequential process
model. Each size had one warmup and seven measured runs, alternating rustocop
and RuboCop. Normalized JSON output was identical before measurement.

<!-- generated:memory-results:start -->
| Files | Rustocop sequential median / p95 | Rustocop parallel median / p95 | RuboCop + Prism median / p95 |
| ---: | ---: | ---: | ---: |
| 1 | 3.52 / 3.53 MiB | 3.53 / 3.53 MiB | 86.36 / 88.25 MiB |
| 25 | 4.03 / 4.05 MiB | 4.20 / 4.27 MiB | 88.89 / 89.42 MiB |
| 100 | 4.36 / 4.47 MiB | 4.52 / 4.53 MiB | 89.53 / 90.16 MiB |
| 500 | 4.97 / 5.02 MiB | 5.45 / 5.48 MiB | 92.22 / 92.78 MiB |

The nearly flat curves show that fixed runtime and startup cost dominate this
corpus. At 500 files, automatic parallel execution added about
0.48 MiB over sequential rustocop and RuboCop used
16.9 times as much peak memory as parallel rustocop.
This does not imply that arbitrary Ruby files cost only a few KiB each: the
pinned corpus totals just 9,110 source bytes. Large files, large literals,
and more complex syntax need a separate sustained-memory benchmark.
<!-- generated:memory-results:end -->

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
discovery order before formatting.

On 30,894 GitLab Ruby files (103.5 MB) with warm filesystem caches and 20
verified cops, one worker took 2,303 ms, 8 took 704 ms, the automatic 15 took
572 ms, and 24 took 564 ms. Results were effectively flat from 18 through 128
workers; 256 regressed slightly to 571 ms. On this 15-core machine, automatic
parallelism is a sound default and 18–24 workers is the useful manual range.
Using 48 or more workers adds pressure without meaningful throughput.

See [Known performance bottlenecks](bottlenecks.md) for the measured sweep and
[real-project benchmarks](../benchmark/project-benchmarks.md) for the pinned
project methodology.

## Timing interpretation

<!-- generated:rubocop-prism-throughput:start -->
```mermaid
xychart-beta
    title "End-to-end speedup over RuboCop + Prism"
    x-axis "Ruby files" [1, 25, 100, 500]
    y-axis "Speedup (times)" 0 --> 140
    bar [132.45, 114.72, 91.65, 60.02]
```

At 500 files, median throughput was approximately
64,193 files/second for Rustocop and
1,069 for RuboCop with Prism.
<!-- generated:rubocop-prism-throughput:end -->
The corpus is deliberately small—500 files totaling 9,110 bytes—so these figures primarily measure CLI startup,
configuration, parsing, dispatch, and formatter overhead rather than sustained
performance on large application files.

Environment: Apple M5 Pro (15 cores), 24 GB RAM, macOS arm64, Ruby 3.4.9,
Rust 1.96.0. The raw report is generated under
`tmp/performance-verification/rubocop-prism-benchmark.json`.

Reproduce with:

```sh
bundle exec ruby script/benchmark_rubocop_prism.rb
```
