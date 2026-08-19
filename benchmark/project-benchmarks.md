# Large Rails project benchmark

Measured on 2026-08-19 against three pinned, production-scale Rails
repositories. This benchmark complements the tiny 500-file compatibility
corpus: it measures sustained parsing and cop execution over 3,389,568 lines of
real Ruby and records the current correctness gap instead of requiring both
linters to agree before timing.

## Results

Times are median / p95 over five measured runs after one warmup. RuboCop 1.87.0
used Prism 1.9.0 with its cache and server disabled. Rustocop used the same
eight built-in cops and configuration, first sequentially and then with four
file workers.

| Project | Ruby files | Lines | rustocop sequential | rustocop, 4 workers | RuboCop Prism | 4-worker speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| [Chatwoot](https://github.com/chatwoot/chatwoot/tree/8d93d69e8e356216e85c28de7c4240e66b8e83fa) | 1,842 | 162,476 | 225.6 / 231.0 ms | **104.2 / 106.7 ms** | 5.719 / 5.766 s | **54.9×** |
| [RubyGems.org](https://github.com/rubygems/rubygems.org/tree/3201f8831866f82eb9acd7f66287a978d0e59079) | 1,337 | 87,088 | 108.3 / 110.7 ms | **50.4 / 52.3 ms** | 2.691 / 2.725 s | **53.4×** |
| [GitLab CE](https://github.com/gitlabhq/gitlabhq/tree/67a526442c20d20b6e80ebf916bd766b54018c5e) | 30,894 | 3,140,004 | 6.602 / 7.184 s | **1.815 / 2.374 s** | 106.184 / 106.608 s | **58.5×** |

The four-worker result scales much better here than on the old tiny-file
corpus. It is 2.17× faster than sequential rustocop on Chatwoot, 2.15× on
RubyGems.org, and 3.64× on GitLab CE. The GitLab result is the most useful
sustained-workload number: rustocop inspected roughly 103.5 MB of Ruby in 1.8
seconds while RuboCop took 106 seconds.

## Correctness target

The projects intentionally do **not** pass the benchmark rules. A result is an
exact match only when path, cop, severity, message, and complete source range
are identical. Duplicate signatures are compared as a multiset. This is much
stricter than comparing total offense counts.

| Project | rustocop offenses | RuboCop offenses | Exact matches | Precision | Recall |
| --- | ---: | ---: | ---: | ---: | ---: |
| Chatwoot | 41,364 | 48,951 | 429 | 1.04% | 0.88% |
| RubyGems.org | 3,832 | 4,195 | 273 | 7.12% | 6.51% |
| GitLab CE | 635,060 | 698,393 | 6,780 | 1.07% | 0.97% |

These numbers are a baseline, not a parity claim. Similar aggregate counts can
hide different source ranges, messages, or false positives. Raising exact
precision and recall without regressing the timing table is now an explicit
project target. Sequential and four-worker rustocop reports were byte-identical
for every project.

## Projects and licensing

- [Chatwoot](https://github.com/chatwoot/chatwoot) is a Rails customer-support
  application. Its [license](https://github.com/chatwoot/chatwoot/blob/8d93d69e8e356216e85c28de7c4240e66b8e83fa/LICENSE)
  applies MIT to content outside `enterprise/`; that directory is excluded.
- [RubyGems.org](https://github.com/rubygems/rubygems.org) is the production
  Rails service behind RubyGems. Its
  [license](https://github.com/rubygems/rubygems.org/blob/3201f8831866f82eb9acd7f66287a978d0e59079/MIT-LICENSE)
  is MIT.
- [GitLab CE](https://github.com/gitlabhq/gitlabhq) is GitLab's Rails Community
  Edition mirror. Its pinned
  [license](https://github.com/gitlabhq/gitlabhq/blob/67a526442c20d20b6e80ebf916bd766b54018c5e/LICENSE)
  covers the Community Edition under MIT terms; `ee/` is excluded if present.

## Corpus construction

The benchmark downloads immutable GitHub archives and creates a local corpus
containing every `.rb` file except hidden paths and these non-project or
differently licensed components:

```text
coverage/  ee/  enterprise/  log/  node_modules/  public/  tmp/  vendor/
db/schema.rb
```

Tests, migrations, scripts, application code, and library code remain in the
corpus. Files are copied with their relative paths intact. Both linters must
report the selected file count, and RuboCop must find at least one offense.

The shared [`project-rubocop.yml`](project-rubocop.yml) disables all defaults
and enables only these built-in cops:

```text
Layout/LineLength (Max: 100)
Metrics/AbcSize (Max: 10)
Metrics/MethodLength (Max: 10)
Style/Documentation
Style/HashSyntax
Style/RedundantReturn
Style/Semicolon
Style/StringLiterals (EnforcedStyle: double_quotes)
```

The lower metrics thresholds and double-quote requirement are deliberate: the
benchmark must exercise offense creation rather than mostly measure clean-file
parsing. No custom cops or extension gems are loaded. GitLab emits stale
`RSpec/VariableName` directive warnings in five files; the harness records that
known warning and still rejects any other stderr.

## Reproducing

Build the release binary, then run:

```sh
bundle exec rake build:native
bundle exec ruby script/benchmark_projects.rb
```

Archives and prepared corpora are cached under `tmp/project-benchmarks/`. Set
`PROJECT_BENCHMARK_RUNS` or `PROJECT_BENCHMARK_WARMUPS` to change the sample
count. The machine-readable report is written to
`tmp/project-benchmarks/project-benchmarks.json`.

Environment for this run: Apple M5 Pro, 24 GB RAM, macOS arm64, Ruby 3.4.9,
RuboCop 1.87.0, Prism 1.9.0, and Rust 1.96.0.
