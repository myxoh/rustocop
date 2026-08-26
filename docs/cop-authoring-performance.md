# Cop authoring performance

This report records a measured five-cop practice run completed on
2026-08-26. The pack translates `Performance/RedundantSortBlock`,
`Performance/ReverseEach`, `Performance/ReverseFirst`, `Performance/Size`, and
`Performance/StringBytesize` from `rubocop-performance` 1.26.1. It is stored
under [`examples/native_custom_cops`](../examples/native_custom_cops/README.md)
and remains separate from the 606 built-in compatibility denominator.

## Outcome

All 39 controlled diagnostic, `-a`, and `-A` contracts pass. All five cops are
project-exact across the 50 pinned projects: 41 native offenses match 41
RuboCop offenses with exact paths, severities, messages, and ranges.

| Development operation | Before | After |
| --- | ---: | ---: |
| Five focused cached cops, separate commands | 1.84 s | — |
| Five focused cached cops, one command | 0.37 s | 0.35–0.51 s |
| Third-party oracle refresh | 60.28 s | 8.12–9.08 s |
| Full built-in cached contracts (execution only) | — | 2.975 s |
| First project confirmation after a Rust change | — | 22.39–24.35 s |
| Warm 50-project confirmation | — | 1.76–1.79 s |
| Unchanged incremental reference refresh | 273.14 s | 2.06 s |

The initial valid RuboCop scan remains expensive: 273.14 seconds for these five
cops, with GitLab and Rails dominating. That scan is now revision-addressed and
reusable per project. Adding a project or changing one pinned revision no
longer invalidates the other project entries.

## Tooling bottlenecks found

1. The project runner rejected extension cop names because it built its matrix
   from RuboCop core only. `--extension GEM` now loads a pinned extension before
   validating the requested cops.
2. A plugin config outside the project corpora allowed RuboCop to inspect zero
   files, and an empty report was accepted. The runner now requires the exact
   expected file count before caching a reference.
3. The first oracle capture launched RuboCop once per case and correction mode.
   Captures are now grouped by cop, reducing process startups from three per
   case to three per cop.
4. The reference snapshot was monolithic. Refresh now reuses each project only
   when repository, revision, file count, RuboCop version, plugin configuration,
   and selected cops match.
5. Focused verification was often invoked once per cop. The cached runner
   already accepts a batch; the extension verifier makes the batched path the
   documented default.
6. Extension caches had no cheap staleness check. Their source definition,
   cases, and configs are now SHA-256-bound in the manifest, so ordinary edits
   do not need a live RuboCop run.
7. A dirty compatibility snapshot is bound to one hash of the entire cop tree
   and native binary. An unrelated extension edit therefore makes all 606
   built-in project rows appear stale. That evidence model needs per-cop plus
   shared-runtime fingerprints; regenerating the table as “0 compatible” would
   be misleading.
8. The repository-wide quality gate is currently noisy before this example is
   considered: architecture ratchets are stale across dozens of existing
   modules, Clippy reports 31 existing errors, and generated built-in fixture
   inputs are stale across multiple cops. The new module itself is 271 lines,
   owns five cops, and adds no lines to the already-ratcheted registry module.
   Until the baseline is repaired, focused green checks are a better authoring
   signal than the aggregate task.

## Reasoning bottlenecks found

- RuboCop AST blocks wrap their send node; Prism call nodes own their block.
  Treating every Prism call ancestor as a RuboCop send ancestor caused a false
  negative in a nested loop. The port now ignores structural block-owner calls
  when reproducing RuboCop's ancestor test.
- Prism exposes a transparent `StatementsNode` that RuboCop's `node.parent`
  logic does not. The port explicitly skips that node in the one cop that needs
  the distinction, without changing a global helper during focused cop work.
- Prism call locations include an attached block, while RuboCop send ranges do
  not. Controlled correction fixtures must cover block-bearing calls even when
  the translated cop listens to `on_send`.
- Obvious examples were insufficient. The first 50-project comparison found a
  nested-block offense and an enclosing-block clean case. Both became small,
  provenance-backed contracts before code changed.
- Safe correction is configuration, not an implementation guess.
  `Performance/StringBytesize` is correctable under `-A` but intentionally
  unchanged under `-a`; the generated config preserves the plugin default.

## Recommended workflow

1. Add a small YAML contract with offending, clean, block/context, and
   correction cases. Refresh the pinned extension oracle once.
2. Implement one cop at a time, but verify related cops together in one cached
   command. This loop should remain below one second when no Rust source changed.
3. After a Rust edit, accept the incremental compile cost, then stay in the
   cached loop. Current fixture-profile recompiles measured roughly 3.4–5.8
   seconds; actual contract execution was 30–70 ms.
4. Run the focused project gate once. Convert every mismatch into a controlled
   case with repository, revision, path, and line provenance.
5. Re-run the project gate using the cached RuboCop reference. A completely
   warm five-cop confirmation should take about two seconds.

The remaining meaningful authoring cost is human contract analysis and the
first RuboCop scan for a new plugin/configuration. More fixture automation would
be counterproductive if it generated many redundant examples; the tooling is
optimized for a small set of deliberate cases per cop.
