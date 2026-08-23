# Real-project performance benchmark

This benchmark measures sustained parsing and cop execution on the same 50
pinned repositories used by the correctness audit. It is a performance tool,
not a source of compatibility status: `docs/real-project-parity.md` owns current
complete-signature results.

## Status of the checked-in measurements

The only completed report currently available was measured on 2026-08-19,
before the runner expanded from three projects to ten and before the recent
parity repairs. Its three timing rows are retained below as a dated historical
baseline. Its old offense and exact-match counts have been removed because they
are superseded by the current ten-project audit and should not be mistaken for
the present implementation.

| Historical project | Ruby files | Lines | Rustocop sequential | Rustocop, 4 workers | RuboCop Prism | 4-worker speedup |
| --- | ---: | ---: | ---: | ---: | ---: | ---: |
| [Chatwoot](https://github.com/chatwoot/chatwoot/tree/8d93d69e8e356216e85c28de7c4240e66b8e83fa) | 1,842 | 162,476 | 225.6 ms | **104.2 ms** | 5.719 s | **54.9×** |
| [RubyGems.org](https://github.com/rubygems/rubygems.org/tree/3201f8831866f82eb9acd7f66287a978d0e59079) | 1,337 | 87,088 | 108.3 ms | **50.4 ms** | 2.691 s | **53.4×** |
| [GitLab CE](https://github.com/gitlabhq/gitlabhq/tree/67a526442c20d20b6e80ebf916bd766b54018c5e) | 30,894 | 3,140,004 | 6.602 s | **1.815 s** | 106.184 s | **58.5×** |

Those numbers were medians of five measured runs after one warmup on an Apple
M5 Pro with 24 GB RAM, macOS arm64, Ruby 3.4.9, RuboCop 1.87.0, Prism 1.9.0,
and Rust 1.96.0. They are not labelled “current.”

## Current benchmark contract

The runner now covers all projects in `lib/rustocop/project_corpus.rb` and
records:

- the Rust source commit and release-binary SHA-256;
- corpus file, byte, and line counts;
- sequential and four-worker Rustocop timing;
- RuboCop 1.87.0 with Prism, cache disabled, and server disabled;
- complete diagnostic-signature overlap as context, never as the compatibility
  scoreboard; and
- sequential/parallel Rust output identity.

The configuration intentionally enables eight strict built-in cops so the run
does real diagnostic work:

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

No custom cops or extension gems are loaded. A new result should replace the
historical table only after the full 50-project run completes in isolation and
its commit and binary digest are recorded.

## Corpus construction

The benchmark downloads immutable GitHub archives and copies every `.rb` file
except hidden paths and these excluded components:

```text
coverage/  ee/  enterprise/  log/  node_modules/  public/  tmp/  vendor/
db/schema.rb
```

Tests, migrations, scripts, application code, and library code remain. Paths
are preserved, both engines must inspect the expected file count, and RuboCop
must produce at least one offense.

## Reproducing

Prepare all 50 pinned corpora without running the benchmark:

```sh
PROJECT_BENCHMARK_PREPARE_ONLY=1 \
  bundle exec ruby script/benchmark_projects.rb
```

The prepared corpus contains 85,472 Ruby files. The preparation command is
idempotent and reuses the immutable archives and completed filtered corpora.

Run this only when the machine can remain otherwise idle:

```sh
bundle exec rake build:native
bundle exec ruby script/benchmark_projects.rb
```

Archives and prepared corpora are cached under `tmp/project-benchmarks/`. Set
`PROJECT_BENCHMARK_RUNS` or `PROJECT_BENCHMARK_WARMUPS` to change sampling.
Reducing either is useful for a smoke test but is not suitable for publishing a
replacement timing table. The machine-readable report is
`tmp/project-benchmarks/project-benchmarks.json`.
