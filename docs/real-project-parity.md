# Real-project output parity

This report is the broad real-world compatibility standard. It uses RuboCop
1.87.0 with `parser_prism`, the strict configuration in
`benchmark/project-rubocop.yml`, and ten pinned open-source projects containing
54,146 Ruby files.

An exact signature includes the relative path, cop name, severity, message,
start line and column, and end line and column. Count-only equality is not
enough. RuboCop-derived fixtures separately cover autocorrection, adversarial
cases, and configuration branches that the projects do not exercise.

## Latest realistic status

The latest complete checkpoint was generated at
`2026-08-23T08:45:38-04:00` from Rust source
`9e7049de18d0caf78c0e0e519cf24f016e9f650a` and native binary SHA-256
`2f200485daea030ae11336049256d46959e62e5f8f326961893c8849e53cf85a`.
The stored RuboCop reference was captured at `2026-08-23T08:43:27-04:00`
and has SHA-256
`34c374aaf167efc7ca3d92a60c15b2c6f4d45e86aeeb1e2cb2c6891eaae89688`.
After excluding the 94 intentionally pending cops, its active-cop slice reports:

| Classification | Complete checkpoint |
| --- | ---: |
| Project-exact | 402 |
| Exact but dormant | 110 |
| Mismatching | 0 |
| Rust crash | 0 |
| RuboCop gate error | 0 |

Every exercised active cop matches RuboCop's complete diagnostic signatures.
Dormant cops emitted no diagnostics in either engine and remain subject to the
separate 100%-passing fixture gate. `Lint/RedundantCopDisableDirective` and
`Style/FileWrite` are intentionally pending because RuboCop 1.87 cannot produce
stable isolated project-reference output for them.

The minimized project-regression corpus contains 336 passing cases and no
pending active-cop mismatch directions. The
configuration-mutation corpus contains six. They preserve fixed pathological
examples, while the complete matrix catches interactions and unrepresented
syntax across the full 54,146-file corpus.

See [the compatibility gap analysis](project-compatibility-gap-analysis.md) for
why the near-90% fixture result does not imply near-90% project parity and for
the revised real-project-first repair loop.

## Pinned projects

| Project | Ruby files | Revision |
| --- | ---: | --- |
| Chatwoot | 1,842 | `8d93d69e` |
| RubyGems.org | 1,337 | `3201f883` |
| GitLab CE | 30,894 | `67a52644` |
| Rails | 3,445 | `ba4f7369` |
| Discourse | 10,897 | `cec79c60` |
| Mastodon | 3,257 | `60593f6a` |
| Sidekiq | 168 | `1bb4aa06` |
| Devise | 197 | `372b295f` |
| RSpec Core | 233 | `aec5f494` |
| Homebrew | 1,876 | `44d5dd83` |

## Reproducing the complete audit

The audit requires a clean committed Rust tree. It builds the release binary,
records both the Rust commit and binary SHA-256, runs Rust crash gates, and
then compares complete diagnostic signatures against the checked-in compressed
RuboCop reference:

```sh
bundle exec ruby script/audit_project_parity.rb \
  --active \
  --report tmp/project-parity/all-cops-current.json \
  --markdown tmp/project-parity/all-cops-current.md
```

That normal command runs only Rustocop. The reference stores RuboCop's
normalized diagnostic signatures and is accepted only when its RuboCop
version, strict-config SHA-256, complete cop selection, pinned project
revisions, and per-project file counts match. Refresh it after any of those
inputs intentionally changes:

```sh
bundle exec ruby script/audit_project_parity.rb \
  --active \
  --refresh-rubocop-reference \
  --report tmp/project-parity/all-cops-current.json \
  --markdown tmp/project-parity/all-cops-current.md
```

The one-command documentation refresh uses the stored reference by default;
add `--refresh-rubocop-reference` only when RuboCop must be rerun:

```sh
bundle exec ruby script/generate_compatibility_report.rb --refresh-projects
```

Generated JSON and Markdown under `tmp/project-parity/` are intentionally
untracked. The compressed RuboCop reference under
`spec/compatibility_evidence/` is tracked. Any committed claim must include the
source commit, binary digest, reference digest, corpus revisions,
classification counts, and whether it came from a complete matrix or a focused
reconciliation.
