# Fixture strategy

Fixtures have two deliberately separate roles.

## Controlled unit contracts

`cops/<Department>/<Cop>/` owns every example intended to validate one cop.
The generated RuboCop 1.87 contract lives in `unit/cases.jsonl`, with
deduplicated YAML configurations in `unit/configs.json`. The root
`unit_manifest.json` maps all 606 cops to those files and records counts and
SHA-256 digests.

Each controlled case preserves:

- source, virtual path, Ruby version, parser, encodings, and file mode;
- exact message, severity, correctability, and complete source range;
- RuboCop's final `-a` and `-A` source, including correction failures; and
- every upstream example from which the case was derived.

The audit reduced 28,601 comparable captures to 28,049 controlled cases by
merging 552 exact duplicate inputs without losing their provenance. The median
cop has 21 cases; the range is 1–1,262. Routine tests read only this committed
cache and never start Ruby or RuboCop.

Run one or more cops:

```sh
bundle exec ruby script/verify_cop.rb Security/Eval
bundle exec ruby script/verify_cop.rb Security/Eval Style/StringLiterals
```

Run the complete cache:

```sh
bundle exec rake fixtures:unit
```

On a warm release build, `Security/Eval` checks 15 cases in about 7 ms and the
full 28,049-case audit takes about 2.3 seconds. The gate intentionally fails on
any RuboCop difference; it is a parity test, not a Rustocop snapshot test.

Other cop-owned directories (`configuration`, `end_to_end`, `hardening`,
`native`, `prism`, and minimized `project` regressions) retain specialized
test representations and provenance. They are unit-scoped because they target
a known cop; they are not substitutes for a whole-project audit. The legacy
project/configuration regression differential is opt-in because it still starts
both linters:

```sh
PROJECT_AUDIT=1 bundle exec rspec \
  spec/project_parity_regressions_spec.rb \
  spec/configuration_parity_regressions_spec.rb
```

Refreshing the controlled cache is the exceptional slow path. It runs RuboCop,
then commits its output for subsequent Rust-only cycles:

```sh
bundle exec rake fixtures:refresh_unit
```

Use `bundle exec ruby script/generate_unit_fixtures.rb --check` to validate the
cache without RuboCop. Use `--live-rubocop` with `script/verify_cop.rb` only
when deliberately checking the capture pipeline itself.

## Transient project audits

`projects/<Project>/` is reserved for whole-project artifacts that cannot be
honestly assigned to one cop. Prepared repositories and configuration
variations live under ignored `tmp/project-benchmarks/corpora/`; they are large,
reproducible, and intentionally outside the fast unit cycle. Run those audits
sporadically, then minimize every useful discrepancy into its owning cop's
controlled fixtures.

`cop_project_cases.tsv`, `cop_project_mismatches.tsv`, and
`cop_configuration_cases.tsv` retain provenance for the existing minimized
cases. Fixtures that necessarily exercise several cops live under `shared/`.

Run `bundle exec ruby script/check_fixture_ownership.rb` to reject unknown cop
paths, missing indexed files, and unindexed project/configuration fixtures.
