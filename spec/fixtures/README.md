# Unit-contract corpus

This directory contains only controlled, cop-owned unit contracts. There are
no project snapshots, native snapshots, end-to-end fixtures, or other parallel
fixture formats.

Each `cops/<Department>/<Cop>/unit/` directory contains:

- `cases.jsonl`: source, virtual path, runtime settings, exact diagnostics,
  cached safe (`-a`) and all (`-A`) correction outcomes, and provenance;
- `configs.json`: content-addressed RuboCop YAML used by those cases.

`unit_manifest.json` maps all 606 built-in cops to their unit files and records
counts, SHA-256 digests, the RuboCop version, and an ISO 8601 `updated_at`
timestamp. The current corpus has 29,526 controlled, cop-owned fixture rows.
The strict runner performs 29,616 cached checks after accounting for every
diagnostic and applicable safe (`-a`) and all-cop (`-A`) correction contract.
Project-isolation origins are retained as regression evidence after their
mismatches are fixed; they are not a queue of currently failing cases.

Exact duplicate inputs share one case and retain all provenance entries. A
normal refresh rebuilds upstream expectations with RuboCop and preserves the
already-imported controlled cases, so removed legacy layouts cannot reappear.

Run focused contracts without starting Ruby or RuboCop:

```sh
ruby script/verify_cop.rb Security/Eval
```

Focused runs use Cargo's incremental `fixture` profile: it omits debug symbols
to reduce relinking work while leaving the full corpus on the optimized release
profile. On the audit machine on 2026-08-26, three warm runs of a representative
32-case cop took 0.42-0.77 seconds inside the test process and 0.71-1.32 seconds
wall-clock. Touching that cop's Rust source to simulate an implementation edit
took 4.11 seconds to compile and 5.33 seconds end to end. The first focused run
builds and caches this profile once. Compilation and linking of the single
native crate remain the dominant edit-cycle cost; the previous 2.5-second
one-file rebuild measurement no longer describes the current crate.

Run the complete cached corpus:

```sh
bundle exec rake fixtures:unit
```

Three warm release runs checked all 29,616 cases in 2.75-2.87 seconds inside the
test process and 2.96-3.20 seconds wall-clock. The intentionally sequential
per-cop timing audit measured a 10.592 ms median, 46.641 ms p95, 113.803 ms p99,
and 11.35 seconds for its complete run. Of 606 cops, 85 completed within 5 ms,
270 within 10 ms, 525 within 25 ms, and 580 within 50 ms. A few size-sensitive
contracts intentionally exercise source files as large as 100-400 KB. Cargo
process startup is separate from these in-run measurements.

Reproduce the per-cop timing audit without cross-cop parallelism:

```sh
bundle exec rake fixtures:benchmark
bundle exec rake fixtures:benchmark REPORT=tmp/unit-timings.json
```

The optional JSON report records `duration_ms` beside every cop's passing and
total case counts. The benchmark retains all parity assertions: it still exits
nonzero when any cached contract differs.

Validate cache integrity and the unit-only layout:

```sh
bundle exec ruby script/generate_unit_fixtures.rb --check
bundle exec ruby script/check_fixture_ownership.rb
```

Refresh upstream-derived expectations only on the deliberate slow path:

```sh
bundle exec rake fixtures:refresh_unit
```

Whole-project repositories and audit outputs are transient and live under
ignored `tmp/project-benchmarks/` and `tmp/project-parity/`. Any useful
project discrepancy must be minimized and added to its cop's unit contract;
whole project trees are never committed here.
