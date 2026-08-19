# Known performance bottlenecks

This is the measured, still-actionable performance backlog. Addressed
investigations have been removed; their tests and implementation history live
in git. Optimizations must preserve normalized JSON and correction parity, one
Prism parse per file, deterministic parallel output, architecture checks, and
the full upstream compatibility baseline.

## Prism node dispatch

Confidence: measured.

The registry already separates source, node, and parse-error phases, so
source-wide cops no longer receive every AST node. Every enabled *node* cop
still sees every Prism node and rejects irrelevant kinds or call names in its
handler.

On 500 Chatwoot files (886,046 bytes), phase separation reduced a run with 573
non-legacy Prism cops from 584.36 ms to 515.53 ms (11.8%). The remaining
dispatch work becomes more important as the enabled-cop count grows.

The next low-risk design is stable dispatch buckets keyed by subscribed Prism
node kind, with a general bucket for genuinely polymorphic cops. A secondary
call-name index may help narrowly targeted call cops, but should only be added
after the node-kind index is benchmarked. Cop ordering must remain stable.

## Legacy text representation

Confidence: code-level, previously measured.

Prism-only runs parse the original source directly. Enabling even one legacy
text cop still eagerly splits and clones the complete line representation,
then rejoins it before Prism parsing.

The safe next step is to separate before-Prism mutators from after-Prism
readers, measure them independently, and only then consider lazy or
copy-on-write line storage. Correction ordering and the single-parse invariant
must not change.

## Independent source-wide scans

Confidence: code-level; optimize only after profiling a representative cop set.

Several source cops scan the complete file independently. A shared lexical or
line index could amortize this work, but a general token-cache API would add
lifetime and ownership complexity. Add only a narrowly named index that serves
multiple measured consumers; do not build a second parser or generic cache.

## Correction conflict resolution

Confidence: measured synthetic worst case; low priority for ordinary linting.

Selecting non-overlapping corrections currently compares accepted edits in a
way that trends quadratically when a file produces many correction candidates.
Synthetic low-priority correction batches measured 18.3 ms at 8,000 edits,
49.0 ms at 16,000, and 153.6 ms at 32,000.

An interval-aware conflict check could remove that curve, but ordinary files
rarely approach these counts. Preserve deterministic priority and transaction
atomicity if this is changed.

## Parallel scaling and file orchestration

Confidence: measured on an Apple M5 Pro with 15 CPU cores and 24 GB RAM.

With warm filesystem caches and 20 verified cops, complete-file workers scale
well until CPU saturation and then plateau. Extra workers do not create more
disk throughput on this workload; measured physical block input was zero.

| Project | Files / source | 1 job | 8 jobs | 15 jobs | 24 jobs | 48 jobs | 96 jobs |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Chatwoot | 1,842 / 5.9 MB | 130.75 ms | 45.95 ms | 36.46 ms | 36.44 ms | 36.25 ms | — |
| GitLab | 30,894 / 103.5 MB | 2,303.29 ms | 703.63 ms | 572.12 ms | 563.93 ms | 562.96 ms | 565.87 ms |

GitLab was effectively flat from 18 through 128 workers, with a small
regression at 256 (571.28 ms). The machine's automatic count of 15 is a sound
default. Explicit 18–24 workers can recover the last few milliseconds on large
warm-cache projects; 48 or more adds scheduling, stack reservation, and file
descriptor pressure without meaningful throughput.

Do not add work stealing or asynchronous I/O for this plateau. Revisit the
scheduler only if cold-cache, network-filesystem, or materially heavier-cop
profiles demonstrate a different bottleneck.

## Recommended order

1. Dispatch Prism node cops by subscribed node kind.
2. Make the legacy line representation lazy after separating readers and
   mutators.
3. Replace quadratic correction conflict checks if real projects exhibit large
   correction batches.
4. Add shared lexical indexes only for a measured group of source cops.
