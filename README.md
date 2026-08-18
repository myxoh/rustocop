# rustocop

`rustocop` is a RuboCop-compatible gem backed by a Rust native binary.

> [!IMPORTANT]
> **This project was vibe coded.** It was built through iterative human
> direction and AI coding agents, with upstream specs used as the primary
> correctness check. The code has not received the kind of sustained human
> review expected of a mature production linter.

## Intent and project status

Rustocop is an open-source experiment: how much of RuboCop's behavior can be
reproduced with a Rust engine and one shared Prism parse per Ruby file, and what
performance is possible without giving up an executable compatibility contract?

The goal is evidence-backed compatibility, not a loose collection of
RuboCop-like regexes. Every built-in cop is classified as **verified**,
**heuristic**, or **missing**, and only complete captured upstream diagnostic
and correction contracts qualify as verified.

This is not an official RuboCop project, is not yet a drop-in replacement, and
should not be trusted as the sole lint or security gate for production code.
Expect incomplete configuration handling, false positives, false negatives,
and breaking changes while parity work is underway. Contributions that improve
correctness, test coverage, architecture, or reproducible benchmarks are
welcome.

The gem name and executable are `rustocop`. RuboCop is the compatibility target:
the specs compare selected `rustocop` output against the real `rubocop` gem so
we can grow behavior cop by cop without guessing at formatter details.

## Current support

- Ruby gem entrypoint: `exe/rustocop`
- Native binary contract: `libexec/rustocop-native`
- Development fallback: `libexec/rustocop-ruby`
- Rust source: `crates/rustocop`
- Compatibility coverage: `Layout/TrailingWhitespace` JSON output and exit
  status for file and stdin input
- Native checks for the RuboCop, Rails, RSpec, Bundler, Layout, Metrics, Naming,
  Style, and Lint cops listed in the project seed config. Singulate-specific
  cops are intentionally excluded.
- A shared Prism parse and AST visitor powers 33 built-in cops across Layout,
  Lint, Security, and Style. The committed differential corpus contains 500 Ruby
  files (25 per cop) and compares JSON output directly with RuboCop.
- `--show-cops` prints the native support registry.
- [The complete built-in cop support matrix](docs/cop-support.md) records every
  RuboCop 1.87 cop as verified, heuristic, or missing. Regenerate it with
  `bundle exec ruby script/generate_cop_support.rb`.
- [The RuboCop + Prism performance verification](docs/performance.md) records
  reproducible end-to-end timings and JSON parity checks for the shared
  500-file, 20-cop corpus.

The Rails/RSpec/metrics and remaining line-oriented cops are lightweight native
implementations. Many are still heuristic and are documented that way rather
than presented as full compatibility.

## Native architecture

Every inspected file is parsed once with Prism. Enabled AST cops are registered
with a shared visitor, receive typed nodes plus ancestor context, and report
byte-accurate source ranges. Compatible edits are collected during the traversal
and applied as one batch. The differential compatibility suite runs 20 cops
against 500 generated and committed Ruby fixture files, both cop-by-cop and as a
single corpus, and compares their JSON reports directly with RuboCop.

The current RuboCop 1.87 matrix contains 28 upstream-spec-verified cops, 51
heuristic native implementations, and 527 missing built-in cops. A cop is only
“verified” after all captured upstream diagnostics and correction assertions
for that cop pass.

## Upstream RuboCop contract

The official RuboCop 1.87.0 cop specs are vendored under
`spec/upstream/rubocop-1.87.0` at tag `v1.87.0`, commit
`e5b788dba181ad94de30cfbad661c5d6aa08a4e5`. The unchanged public suite contains
28,479 RSpec examples. It currently runs with zero failures and six pendings
declared by RuboCop itself.

The capture harness executes RuboCop's test DSL and records the resulting
source, configuration, path, Ruby version, offenses, and correction. It does
not infer expectations by scraping spec source. All 606 registered built-in
cops have executable captured cases.

```sh
bundle exec ruby script/extract_upstream_cop_specs.rb
bundle exec ruby script/compare_upstream_cop_specs.rb
```

Generated corpora and reports live under `tmp/` and are intentionally ignored.
The comparison command is diagnostic-only for now; correction parity remains a
separate required gate before a cop can be marked fully upstream-compatible.

## Development

Read [the architecture](docs/architecture.md) and
[the rules of engagement](CONTRIBUTING.md) before adding cops. The default spec
task enforces the documented module and function complexity ceilings.

Install dependencies:

```sh
bundle install
```

Run specs:

```sh
bundle exec rake spec
```

Regenerate the compatibility corpus after changing its case templates:

```sh
bundle exec ruby script/generate_compatibility_corpus.rb
```

Build the native binary when Rust is installed:

```sh
bundle exec rake build:native
```

The build task copies the release binary to `libexec/rustocop-native`, which is
what `exe/rustocop` launches by default. Set `RUSTOCOP_DISABLE_NATIVE=1` to force
the Ruby fallback while developing the gem wrapper or compatibility specs.
