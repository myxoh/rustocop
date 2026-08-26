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
`2026-08-26T14:47:18-04:00` from dirty-worktree cop-source SHA-256
`3e72741c0ea482dfb92beb24594d784c1c1862ed67dbec53a5b93c3ec97352b1`
and native binary SHA-256
`d06f7ed679bd43ff8212ff12da956473ba552014ab416b4f9a993fdd64efa4d3`.
The stored RuboCop reference has SHA-256
`d9f16acf805c8a76b324447497ec18c5acbd3e917b5dd24656c5eaf878a5620c`.
The intentionally-pending dataset is empty, so the active-cop slice covers all
606 built-in cops:

| Classification | Complete checkpoint |
| --- | ---: |
| Project-exact | 531 |
| Exact but dormant | 75 |
| Mismatching | 0 |
| Rust crash | 0 |
| RuboCop gate error | 0 |

All 531 exercised cops are exact (100%), and the complete native project run
has no crashes. The other 75 cops emitted no diagnostics in either engine for
this pinned configuration and corpus, so the project audit classifies them as
dormant rather than claiming unexercised project parity.

The audited legacy project corpus retains 597 provenance entries. The current
audit adds 20 pending mismatch directions across 19 minimized cases for 10
cops. The
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

The audit builds the release binary, records either the clean-tree Git commit or
a deterministic native-cop source SHA-256 together with the binary SHA-256,
runs Rust crash gates, and then compares complete diagnostic signatures against
the checked-in compressed RuboCop reference. Rustocop preserves the complete
selected cop set in one process by default so selection-sensitive behavior
matches the reference. Results are cached under `tmp/project-parity/native-cache/` using
the native binary, configuration, pinned project revision, corpus file count,
and exact cop selection as the cache key. Alongside the concise JSON report,
it writes a `.mismatches.json.gz` sidecar containing every distinct unmatched
signature, its multiplicity, project revision, and source-file digest. The
normal command reuses the complete cached 50-project reference and runs only
Rustocop. An unchanged exhaustive rerun normally reads the native cache as well;
any changed native binary or comparison input produces a safe cache miss:

```sh
bundle exec ruby script/audit_project_parity.rb \
  --active \
  --report tmp/project-parity/all-cops-current.json \
  --markdown tmp/project-parity/all-cops-current.md
```

Use `--no-native-cache` when measuring cold native execution. Use
`--native-cache-root PATH` to isolate a benchmark cache. Reports record the
worker count, cop batch size, and per-project cache hits and misses so cached
and cold timings cannot be confused.

Turn newly observed signature differences into minimized, provenance-backed
cop-owned unit contracts before changing implementations. The isolator consumes
the exhaustive sidecar rather than the three-example report previews, and tracks
individual signature fingerprints instead of treating one mismatch direction as
complete coverage:

```sh
bundle exec ruby script/isolate_project_parity_mismatches.rb \
  tmp/project-parity/all-cops-current.json
```

Use `--dry-run` to inspect candidate counts without invoking either engine, and
`--cop` or `--limit-per-cop` to process the stored inventory in focused batches.
These batches do not require another project comparison.

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
source identity, binary digest, reference digest, corpus revisions,
classification counts, and whether it came from a complete matrix or a focused
reconciliation.
