# Work in progress: documentation accuracy audit

Last updated: 2026-08-21T22:36:46-04:00

This file records the state of the documentation audit when work was paused. The
changes in this commit are intentionally a checkpoint, not a claim that the
documentation review is complete.

## Current parity evidence

A fresh audit ran all 606 registered cops against the ten-project real-world
corpus (54,146 Ruby files) using the current Rust source.

- Project evidence updated at: `2026-08-21T22:15:13-04:00`
- Fixture evidence updated at: `2026-08-21T22:36:46-04:00`
- Fixture source commit: `20caa1a2151458d08cbc72f03f5dc44d5d2fa23a`
- Project source commit: `95ca43471d3d905df411c070b3995594c4ed6baa`
- Native extension SHA-256: `a3ad1372d52e2c73626163029c7a0f081e7a6a5592f9513e560ad23dde68ddb6`
- Project-exact cops: 280
- Dormant cops (not exercised by these projects): 87
- Cops with mismatches: 237
- Rust crashes: 1 (`Layout/FirstHashElementIndentation` on RubyGems.org)
- RuboCop isolation errors: 1 (`Lint/RedundantCopDisableDirective` refuses
  isolated execution through `--only`)

The ignored raw artifacts are:

- `tmp/project-parity/all-cops-current.json`
- `tmp/project-parity/all-cops-current.md`

The important conclusion is that real-project output parity is **not yet at
zero mismatches**. Previous documentation figures of 173 exact, 73 dormant, and
359 mismatching cops are obsolete, but the newer result still leaves 237 cops
to investigate.

The checked-in corpus currently also includes:

- 126 real-project regression cases in
  `spec/fixtures/project_parity_regressions/manifest.tsv`
- 5 configuration-mutation cases

## Documentation changes in this checkpoint

- Replaced the retired Verified/Heuristic qualification presentation with
  project-output classifications backed by the complete audit.
- Regenerated `docs/cop-support.md` with all 606 cops and their current
  project-exact, dormant, mismatch, or RuboCop-error state.
- Regenerated `docs/remaining-cops.md` as the current unresolved parity queue.
- Reworked the README and parity documentation to state explicitly that 237
  mismatches remain.
- Reframed old performance tables as dated historical measurements instead of
  current correctness evidence.
- Rewrote the substantial-work roadmap around fixtures, configuration
  mutations, and real-project parity.
- Added `script/generate_project_parity_docs.rb`; the older support/remaining
  generators now act as compatibility entry points for it.
- Updated the project benchmark script so future reports record both the Rust
  source commit and native binary SHA-256.
- Removed the obsolete missing-cop note for `Lint/ScriptPermission`, which now
  has real-project regression evidence.

## Fresh isolated microbenchmarks

The first benchmark attempt overlapped the long parity audit and was discarded.
After the audit finished, the lightweight benchmarks were rerun without that
competing workload.

RuboCop Prism comparison:

| Files | Rustocop | RuboCop Prism | Speedup | Output verified |
| ---: | ---: | ---: | ---: | :---: |
| 1 | 5.408 ms | 417.507 ms | 77.20x | yes |
| 25 | 6.008 ms | 432.348 ms | 71.96x | yes |
| 100 | 6.917 ms | 438.527 ms | 63.40x | yes |
| 500 | 11.660 ms | 494.815 ms | 42.44x | yes |

Mixed custom-cop comparison:

| Variant | Median | p95 |
| --- | ---: | ---: |
| Native binary | 10.863 ms | 11.453 ms |
| Native entrypoint | 95.607 ms | 99.319 ms |
| Mixed | 498.243 ms | 507.180 ms |
| Mixed entrypoint | 573.140 ms | 612.803 ms |
| RuboCop custom only | 482.526 ms | 491.367 ms |
| RuboCop all | 508.839 ms | 519.271 ms |

Raw benchmark reports are ignored under `tmp/performance-verification/`.

## Work still required

This checkpoint has not received the final end-to-end documentation review or
full repository test pass. On resumption:

1. Review the complete diff for unintended wording or generated-table errors.
2. Search all active documentation for remaining stale qualification counts,
   old parity totals, or claims that mismatches have reached zero.
3. Run syntax checks for the changed Ruby scripts and verify the new generator
   is deterministic from the complete audit report.
4. Run repository documentation/architecture checks and the relevant test
   suite.
5. Decide whether the dated large project, memory, and parallel benchmark
   baselines should be rerun; they are currently labelled historical because a
   current full rerun was not completed.
