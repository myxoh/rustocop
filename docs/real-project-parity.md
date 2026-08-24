# Real-project output parity

This report is the broad real-world compatibility standard. It uses RuboCop
1.87.0 with `parser_prism`, the strict configuration in
`benchmark/project-rubocop.yml`, and 50 pinned open-source projects containing
85,471 Ruby files.

An exact signature includes the relative path, cop name, severity, message,
start line and column, and end line and column. Count-only equality is not
enough. RuboCop-derived fixtures separately cover autocorrection, adversarial
cases, and configuration branches that the projects do not exercise.

## Latest realistic status

The latest complete checkpoint was generated at
`2026-08-24T00:43:41-04:00` from worktree native binary SHA-256
`f58c04887943d65b7ffee3e99af11599e239891dcd4a6c1c16d99fb0e7b6e466`.
The stored RuboCop reference has SHA-256
`e3e1677ba0ac4c8dc8723e7a17a0fd8b0e89fdf29226bfc548f55dcd628c8576`.
After excluding the 33 intentionally pending cops, its active-cop slice reports:

| Classification | Complete checkpoint |
| --- | ---: |
| Project-exact | 323 |
| Exact but dormant | 53 |
| Mismatching | 196 |
| Rust crash | 0 |
| RuboCop gate error | 1 |

Among the 519 exercised cops, 323 are exact (62.2%). The five cops restored in
this iteration pass every captured fixture. `Layout/SpaceBeforeBlockBraces` is
project-exact; the other four retain complete-matrix gaps for future isolation.
`Style/ClassAndModuleChildren` still triggers a RuboCop 1.87 error on Puppet.

The minimized project-regression corpus contains 609 passing cases and no
pending active-cop mismatch directions. The
configuration-mutation corpus contains six. They preserve fixed pathological
examples, while the complete matrix catches interactions and unrepresented
syntax across the full 85,471-file corpus.

Fixture parity does not imply project parity: upstream examples establish the
captured contract, while this complete project matrix exercises negative cases,
configuration interactions, recovered syntax, and source-range behavior that
the fixture corpus may not contain.

## Original project cohort

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

## Expanded project cohort

The 40-project expansion adds 31,325 Ruby files to the original 54,146-file
baseline. Every repository is pinned to a full immutable commit in
`lib/rustocop/project_corpus.rb`, and all 50 prepared corpora have been checked
to contain the expected total of 85,471 Ruby files. The checkpoint above covers
the complete original and expanded cohorts.

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
| Logger | 12 | `026eb968` |
| RDoc | 206 | `5bd8719f` |

## Reproducing the complete audit

Prepare the immutable repositories first; this downloads only missing archives
and reuses completed corpora:

```sh
PROJECT_BENCHMARK_PREPARE_ONLY=1 \
  bundle exec ruby script/benchmark_projects.rb
```

The audit requires a clean committed Rust tree. It builds the release binary,
records both the Rust commit and binary SHA-256, runs Rust crash gates, and
then compares complete diagnostic signatures against the checked-in compressed
RuboCop reference. The normal command reuses the complete cached 50-project
reference and runs only Rustocop:

```sh
bundle exec ruby script/audit_project_parity.rb \
  --active \
  --report tmp/project-parity/all-cops-current.json \
  --markdown tmp/project-parity/all-cops-current.md
```

The reference stores RuboCop's normalized diagnostic signatures and is accepted
only when its RuboCop version, strict-config SHA-256, complete cop selection,
pinned project revisions, and per-project file counts match. Refresh it after
any of those inputs intentionally changes:

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
