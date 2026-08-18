# Known performance bottlenecks

This document records measured and code-level performance bottlenecks for later
optimization. It is a backlog, not a claim that the current implementation is
slow: on the committed 500-file compatibility corpus, sequential Rustocop is
currently about 53 times faster than RuboCop with Prism. This document also
keeps addressed bottlenecks visible so later changes do not accidentally
reintroduce them.

## Benchmark scope

The measurements below were taken on 2026-08-18 using the committed 500-file
compatibility corpus and its 20 shared cops. The corpus contains only 9,090
bytes, so it deliberately emphasizes startup, file orchestration, registry
construction, dispatch, and formatting. It does not represent sustained parsing
of large application files.

All timed variants produced byte-identical JSON. RuboCop 1.87.0 used Prism
1.9.0, caching and server mode were disabled, and Rustocop was built in release
mode.

| Enabled cops | 500-file median |
| ---: | ---: |
| None | 9.32 ms |
| 1 | 9.46 ms |
| 5 | 9.89 ms |
| 20 | 9.74 ms |

The end-to-end process floor on one tiny file remains approximately 3 ms. The
20-cop median was 19.28 ms before the execution-plan refactor and is now 9.74
ms, a 1.98-times improvement. At this corpus size, the former per-file registry
and source-representation overhead had obscured traversal dispatch cost.

Raw benchmark reports are generated under `tmp/performance-verification/` by:

```sh
bundle exec ruby script/benchmark_rubocop_prism.rb
bundle exec ruby script/benchmark_parallel.rb
bundle exec ruby script/benchmark_memory.rb
bundle exec ruby script/benchmark_cop_scaling.rb
```

## 1. Addressed: the execution plan was rebuilt for every file

Status: addressed on 2026-08-18 and covered by unit, parallel-parity, and full
upstream-corpus tests.

`prism_engine::inspect` constructs `Registry::enabled` for every source file.
This allocates all registered cop objects, calls the enabled predicate for each
one, retains the selected cops, and destroys the registry after that file. Cop
selection does not normally change within one command, so most of this work is
invariant.

The enabled predicate also repeatedly splits and scans the `--only` string.
Line-oriented cops call the same predicate independently, multiplying the
selection work.

Implemented:

- `CopSelection` parses `--only` into an immutable enabled-cop set.
- `InspectionPlan` constructs one stateless Prism registry per command.
- Scoped file workers borrow and share that registry immutably.

Guardrail: cop implementations must remain stateless and `Sync`. Do not move
per-file diagnostic or traversal state into a cop instance.

## 2. Partially addressed: the line-oriented representation was always allocated

Status: removed for Prism-only runs on 2026-08-18.

Every file currently goes through `split_source`, clones the resulting
`SourceLine` collection, and then calls `join_source` before Prism parsing. This
creates several owned strings even when no enabled line cop needs to mutate the
source. The work is especially visible on a corpus of many tiny files.

Implemented:

- The execution plan records whether any textual cop is enabled.
- Prism-only runs parse the original source directly and skip splitting,
  cloning, and rejoining lines.

Remaining opportunity: a run containing even one textual cop still eagerly
allocates and clones the full line representation. Separate before-Prism
mutators from after-Prism readers before attempting copy-on-write storage.

Do not remove the single-Prism-parse invariant or change correction ordering to
save these allocations.

## 3. Every enabled cop sees every Prism node

Confidence: measured scaling plus code inspection.

The visitor loops over the complete enabled registry for every branch and leaf
node. Each cop then rejects irrelevant node types and call names itself. The
500-file median increased from 10.83 ms with five cops to 19.28 ms with twenty,
showing that this linear dispatch is already material on tiny syntax trees.

This is the most important bottleneck for eventual support of all built-in
cops. File parallelism distributes the work but does not remove it.

Preferred direction:

- Index cops by the Prism node kinds they subscribe to.
- Keep a general-node list only for cops that genuinely need it.
- Consider a secondary call-method index for restricted call cops such as those
  that only inspect `flatten`, `join`, or `sort_by`.
- Preserve deterministic cop ordering within each dispatch bucket.
- Add a benchmark that varies enabled cop count independently of file count.

A large matcher DSL is not required. A small subscription API or registry
metadata should be enough.

## 4. Addressed: source positions rescanned the file for every offense

Status: addressed on 2026-08-18.

`source_position` scans from the start of the source to an offense offset. It is
called for both ends of every Prism finding. This makes position conversion
proportional to source length times offense count and may become expensive on
large files with many findings.

Implemented:

- `diagnostic::SourceIndex` builds newline starts once per source.
- Byte offsets use binary search and count Unicode scalar values only within
  the selected line, preserving RuboCop-compatible columns.

## 5. Parallel scaling is limited by tiny tasks

Confidence: measured.

Automatic parallel execution currently improves the 500-file corpus from 8.88
ms to 7.80 ms, a 1.14-times speedup. At 25 files it is slightly slower than
sequential execution. The earlier 2.29-times speedup largely compensated for
per-file setup that the run-level execution plan has now removed.
Thread startup, atomic queue access, allocation, and file-open overhead dominate
when each file contains very little work.

This is expected rather than a correctness problem. Optimize repeated per-file
setup before adding more scheduling complexity. Re-evaluate worker defaults on
a corpus of realistically sized Ruby files; do not tune them solely for these
tiny fixtures.

## Recommended order

1. Separate line mutators from line readers, then make remaining splitting and
   cloning lazy or copy-on-write.
2. Dispatch Prism cops by node kind and, where useful, call method.
3. Add a realistic large-file/application benchmark before further scheduler
   tuning.

Each optimization must retain normalized JSON parity, correction parity, the
one-Prism-parse-per-file rule, deterministic parallel output, the architecture
checks, and the full upstream compatibility baseline.
