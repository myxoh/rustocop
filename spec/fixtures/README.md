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
timestamp. The current corpus has 28,774 controlled cases: 28,049 unique
upstream-spec inputs plus 725 unique cases retained from the audited legacy
project, configuration, Prism, hardening, end-to-end, and native collections.

Exact duplicate inputs share one case and retain all provenance entries. A
normal refresh rebuilds upstream expectations with RuboCop and preserves the
already-imported controlled cases, so removed legacy layouts cannot reappear.

Run focused contracts without starting Ruby or RuboCop:

```sh
bundle exec ruby script/verify_cop.rb Security/Eval
```

Run the complete cached corpus:

```sh
bundle exec rake fixtures:unit
```

The warm release run checks all 28,774 cases in about 1.4 seconds on the audit
machine. Focused cop runs remain millisecond-scale.

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
