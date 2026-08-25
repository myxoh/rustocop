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
timestamp. The current corpus has 28,788 controlled cases: 28,049 unique
upstream-spec inputs, 720 unique cases retained from the audited legacy
collections, and 19 minimized current-project cases covering 20 pending
mismatch directions across 10 cops.

Exact duplicate inputs share one case and retain all provenance entries. A
normal refresh rebuilds upstream expectations with RuboCop and preserves the
already-imported controlled cases, so removed legacy layouts cannot reappear.

Run focused contracts without starting Ruby or RuboCop:

```sh
ruby script/verify_cop.rb Security/Eval
```

Focused runs use Cargo's incremental `fixture` profile: it omits debug symbols
to reduce relinking work while leaving the full corpus on the optimized release
profile. On the audit machine, a one-line cop edit followed by a focused run
dropped from 10.7 seconds in the ordinary test profile (22.9 seconds in release)
to 2.5 seconds, including a roughly 50 ms contract check. A no-change run is
about 0.4 seconds. The first focused run builds and caches this profile once.
The remaining edit latency is compilation and linking of the single native
crate; splitting every cop into a separate binary or dynamic library is not
currently justified.

Run the complete cached corpus:

```sh
bundle exec rake fixtures:unit
```

The warm release run checks all 28,788 cases in about 1.9-2.3 seconds on the
audit machine. A sequential audit of all 606 cops measured a 3.4 ms median:
448 cops completed within 5 ms, 543 within 10 ms, and the 95th percentile was
16.1 ms. A few size-sensitive contracts intentionally exercise source files as
large as 100-400 KB, so they cannot have single-digit-millisecond isolated
runtimes without weakening or first minimizing those cases. Cargo process
startup is separate from these in-run measurements.

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
