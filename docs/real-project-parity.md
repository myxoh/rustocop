# Real-project output parity

This report is the broad real-world compatibility standard. It uses RuboCop
1.87.0 with `parser_prism`, the strict configuration in
`benchmark/project-rubocop.yml`. The published checkpoint below covers the
original ten pinned open-source projects containing 54,146 Ruby files. The
configured next audit expands that corpus to 50 projects and 85,472 Ruby files.

An exact signature includes the relative path, cop name, severity, message,
start line and column, and end line and column. Count-only equality is not
enough. RuboCop-derived fixtures separately cover autocorrection, adversarial
cases, and configuration branches that the projects do not exercise.

## Latest realistic status

The latest complete checkpoint was generated at
`2026-08-23T09:13:26-04:00` from Rust source
`67b6005782c5129ccae2b445a14897777bc8649e` and native binary SHA-256
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

## Recorded checkpoint projects

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

## Configured 50-project corpus

The 40-project expansion adds 31,326 Ruby files to the original 54,146-file
baseline. Every repository is pinned to a full immutable commit in
`lib/rustocop/project_corpus.rb`, and all 50 prepared corpora have been checked
to contain the expected total of 85,472 Ruby files. These projects do not count
as compatibility evidence until the complete RuboCop reference and Rustocop
comparison have been refreshed.

| Added project | Ruby files | Revision |
| --- | ---: | --- |
| Jekyll | 161 | `74d75133` |
| fastlane | 1,294 | `a9a72554` |
| Huginn | 455 | `9faad4ae` |
| Diaspora | 861 | `f9652786` |
| Postal | 291 | `d038eaa8` |
| Forem | 3,769 | `f354c376` |
| OpenProject | 11,466 | `95799956` |
| Spree | 2,585 | `bf44766b` |
| Solidus | 2,079 | `1f5bf5c6` |
| Fluentd | 442 | `dd45c6e1` |
| RuboCop | 1,763 | `3a42c622` |
| Puppet | 2,152 | `e227c275` |
| Capistrano | 87 | `b54b02fa` |
| Sinatra | 147 | `cb22afd7` |
| Hanami | 241 | `2c785981` |
| Rack | 95 | `8bf4eb07` |
| Puma | 169 | `b8341dc9` |
| Faker | 567 | `cca41849` |
| Factory Bot | 143 | `18ae8b58` |
| CarrierWave | 93 | `2072b120` |
| PaperTrail | 204 | `098058ae` |
| CanCanCan | 63 | `8c1bf153` |
| Simple Form | 99 | `18f38aad` |
| Ransack | 60 | `e82f6bab` |
| Resque | 64 | `bb0f0971` |
| redis-rb | 166 | `2ba9010b` |
| Grape | 325 | `60c1e842` |
| dry-validation | 71 | `e7dff1ed` |
| React on Rails | 554 | `e9dd9cdb` |
| Searchkick | 92 | `93e901a7` |
| PgHero | 58 | `7edb5798` |
| GitHub Linguist | 55 | `b45dbe9b` |
| GitHub Markup | 11 | `76e26821` |
| Rake | 95 | `ec87311d` |
| IRB | 111 | `3794e997` |
| debug | 107 | `6510cfbc` |
| Psych | 89 | `9b12bb3f` |
| net-http | 23 | `10433873` |
| Logger | 13 | `026eb968` |
| RDoc | 206 | `5bd8719f` |

## Reproducing the complete audit

Prepare the immutable repositories first; this downloads only missing archives
and reuses completed corpora:

```sh
PROJECT_BENCHMARK_PREPARE_ONLY=1 \
  bundle exec ruby script/benchmark_projects.rb
```

The next audit requires a clean committed Rust tree. It builds the release binary,
records both the Rust commit and binary SHA-256, runs Rust crash gates, and
then compares complete diagnostic signatures against the checked-in compressed
RuboCop reference. Because the configured project pins changed, the first
50-project run must include `--refresh-rubocop-reference`:

```sh
bundle exec ruby script/audit_project_parity.rb \
  --active \
  --refresh-rubocop-reference \
  --report tmp/project-parity/all-cops-current.json \
  --markdown tmp/project-parity/all-cops-current.md
```

After that one refresh, the normal command can omit
`--refresh-rubocop-reference` and run only Rustocop. The reference stores RuboCop's
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
