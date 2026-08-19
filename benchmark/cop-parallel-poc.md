# Intra-file cop parallelism POC

This experiment asks whether rustocop should parse a file once and run its
independent cops concurrently against that parsed representation. The short
answer is **yes for sufficiently large individual files, but no as the default
project scheduler with the current Prism binding**.

The POC lives on `codex/intra-file-cop-parallelism-poc`. It is intentionally
detection-only and exposes `--parallel-cops`, `--cop-jobs N`, and
`--no-parallel-cops`. It refuses to combine cop parallelism with file
parallelism so that nested worker pools cannot oversubscribe the machine.

## What rustocop already does

Rustocop does not create one process per file. It creates a scoped pool of Rust
threads, and each worker takes complete files from an atomic work queue. A file
is read and parsed exactly once. All enabled Prism cops already reuse that one
parse, but the AST cops traverse it sequentially inside the file worker.

This means the proposed pipeline is partly the current architecture:

1. file reads are parallel across file workers;
2. each file is parsed once by its worker;
3. every enabled cop for that file shares the result of that parse;
4. files with corrections are written independently by their file workers.

## Why the exact shared-tree design is not in this POC

`ruby-prism` 1.9.0 does not mark `ParseResult` or its nodes as thread-safe. The
parse result owns raw `NonNull<pm_parser_t>` and `NonNull<pm_node_t>` pointers,
and APIs such as diagnostics internally borrow parser-owned lists mutably. Rust
therefore correctly rejects sharing the parse result across scoped threads.

Declaring those pointers `Sync` in rustocop would require an unsafe wrapper and
a thread-safety guarantee that the binding does not currently provide. The POC
does not invent that guarantee. Safe exact AST cop parallelism needs one of:

- an upstream `ruby-prism` thread-safety contract and implementation;
- an audited local fork of the binding; or
- copying the AST into an owned, thread-safe representation, which is likely to
  cost more memory and construction time than it saves.

## Safe POC design

The registry already separates source, parse-error, and AST phases. In cop
parallel mode the main thread parses once and runs parse-error plus AST cops,
while scoped worker threads run independent source-phase cops over the shared
immutable source string. Each worker owns its finding context. Findings are
merged and sorted by the same stable source/cop ordering used by sequential
inspection.

The requested job count includes the main parse/AST worker. For example,
`--cop-jobs 4` uses the main worker plus three source-cop workers. Worker
threads are currently scoped per file; that keeps the experiment small and
safe, but deliberately exposes thread-creation overhead on tiny files.

Autocorrection is rejected. A detection pass followed by correction only for
offending files would have to parse and execute those files a second time.
That can help only when offenses are rare and correction startup dominates.
Rustocop already avoids writing unchanged files and safely corrects distinct
files in parallel, so the two-pass design is not an automatic win.

## Results

Measured on 2026-08-19 on an Apple M5 Pro with 15 available CPU cores and
24 GB RAM. Times are median / p95 over seven measured runs after a warmup.
Every variant's JSON output was required to be byte-identical to sequential
output before it was timed.

The real-project workloads select all 606 tracked built-in cops. `tiny_500`
uses the pinned 500-file, 20-cop compatibility corpus. The real files come from
the pinned Chatwoot corpus documented in
[`project-benchmarks.md`](project-benchmarks.md).

| Workload | Files | Sequential | Existing file pool | Cop jobs 2 | Cop jobs 4 | Cop jobs 8 | Automatic cop jobs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Tiny 500 | 500 | 7.87 / 10.77 ms | **7.32 / 9.19 ms (1.07×)** | 18.01 / 18.25 ms (0.44×) | 16.77 / 18.84 ms (0.47×) | 17.64 / 19.59 ms (0.45×) | 17.82 / 20.06 ms (0.44×) |
| Largest real file, 78 KB | 1 | 57.82 / 58.16 ms | 57.79 / 59.06 ms (1.00×) | 51.95 / 52.44 ms (1.11×) | 32.48 / 32.74 ms (1.78×) | **28.00 / 28.23 ms (2.07×)** | 28.24 / 28.50 ms (2.05×) |
| Largest 100 real files, 2.1 MB | 100 | 1,511.22 / 1,516.33 ms | **197.56 / 203.48 ms (7.65×)** | 1,378.60 / 1,483.08 ms (1.10×) | 843.55 / 927.35 ms (1.79×) | 711.23 / 718.76 ms (2.12×) | 692.58 / 697.41 ms (2.18×) |
| All Chatwoot, 5.9 MB | 1,842 | 4,580.57 / 4,624.31 ms | **572.42 / 579.00 ms (8.00×)** | 4,209.98 / 4,245.80 ms (1.09×) | 2,631.68 / 2,770.71 ms (1.74×) | 2,323.47 / 2,369.31 ms (1.97×) | 2,285.88 / 2,296.54 ms (2.00×) |

Cop parallelism is valuable when there is only one large file: eight jobs cut
the largest-file run by 52%. It also roughly halves a sequential full-project
run. It does not compete with distributing whole files: the existing file pool
is 4.0 times faster than automatic cop parallelism on Chatwoot. On tiny files,
per-file thread setup makes cop parallelism more than twice as slow.

## Recommendation

Keep complete-file parallelism as the default and do not merge the POC flags
into the main CLI yet. The useful production design would be an adaptive,
single global scheduler:

- many files: spend the worker budget on complete files;
- one or very few unusually large files with many enabled cops: spend spare
  workers inside those files;
- never create nested pools whose total concurrency exceeds the global budget.

With the current binding, that adaptive path can parallelize source-phase cops
only. Before making it permanent, node-kind dispatch is a simpler optimization
that reduces the AST work itself and benefits every scheduler. Revisit shared
AST cop parallelism if `ruby-prism` gains an explicit thread-safety guarantee.

## Reproducing

Build the release binary and run:

```sh
bundle exec rake build:native
bundle exec ruby script/benchmark_cop_parallel_poc.rb
```

The harness uses the cached Chatwoot corpus produced by
`script/benchmark_projects.rb`. Set `COP_PARALLEL_POC_RUNS` to change the sample
count. Machine-readable results are written to
`tmp/performance-verification/cop-parallel-poc.json`.
