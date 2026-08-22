# rustocop

`rustocop` is an unfinished, Rust-backed attempt at making local RuboCop runs
less painfully slow.

> [!IMPORTANT]
> **This project was vibe coded.** I am not a Rust expert, and there has not
> been much human review. Treat it accordingly.

## Intent and project status

RuboCop performance sucks on a large local codebase, and I couldn't be arsed to
properly become a Rust expert before trying to make it faster. So this project
is being vibed into a RuboCop-compatible local linter instead.

The intended useful version of this project is good enough to run locally for
fast feedback before CI runs real RuboCop. The current build has not earned
that status: real-project comparisons have exposed severe false positives,
configuration incompatibility, and incomplete cop behavior. It must not replace
RuboCop as the final lint or security gate.

Maybe enough interest, use, and scrutiny will eventually turn this into a real
linter. Until then, expect incomplete cop and configuration compatibility,
false positives, false negatives, and breaking changes.

The repository contains native registry entries for all 606 RuboCop built-in
cops, but that is an implementation inventory, not a correctness claim. Public
support evidence now comes from RuboCop-derived fixtures and complete
ten-project signatures; the old Verified/Heuristic qualification scoreboard is
not used.

## Validation standard

The gold standard is executable behavior checked against RuboCop 1.87.0:

1. RuboCop-derived upstream, adversarial, autocorrection, and minimized
   real-project fixtures must pass with matching diagnostics and corrected
   source.
2. Complete diagnostic signatures must match across all ten pinned real
   projects: path, cop, severity, message, and complete source range.

A registered cop with no exercised fixture is unvalidated. A cop with exact
output on every pinned project is **project-exact** for that corpus and
configuration. Neither offense-count similarity, a captured-case label, nor an
old manual review record is accepted as compatibility evidence.

The minimized real-project corpus currently contains 126 passing cases, 12
pending isolated mismatches, and five configuration-mutation cases. These are
regression coverage, not a substitute for the complete project comparison.

The fixture review updated at `2026-08-21T22:18:32-04:00` matched
22,614/28,623 captured cases (79.0%); 496/606 cops matched every captured
fixture. Of those, 269 cops also satisfy the current project-exact gate.

## Ten-project output parity

The real-project matrix asks whether each cop emits the same path, severity,
message, and source range as RuboCop across 54,146 Ruby files in ten projects.

The complete audit updated at `2026-08-21T22:15:13-04:00` found 280
project-exact cops, 87 dormant cops, 237 mismatches, one Rust crash, and one
RuboCop command-line error. The mismatches and crash are real; the recent
repair work improved the matrix but did not reduce them to zero.

| Real-project classification | Complete checkpoint |
| --- | ---: |
| Project-exact | 280 / 606 (46.2%) |
| Exact but dormant | 87 / 606 |
| Mismatching | 237 / 606 |
| Rust crashes | 1 / 606 |
| RuboCop `--only` limitation | 1 / 606 |

The checkpoint is bound to Rust source `95ca434` and native binary SHA-256
`a3ad1372d52e2c73626163029c7a0f081e7a6a5592f9513e560ad23dde68ddb6`.
Project-exact status is the strongest current diagnostic evidence. Unexercised
configuration and autocorrection branches still require RuboCop-derived
fixtures.

See [the compatibility evidence table](docs/compatibility.md) for fixture and
project matching by cop, and [the real-project parity report](docs/real-project-parity.md)
for the ten projects, limitations, and reproduction commands.

## Performance

<!-- generated:rubocop-prism:start -->
On the pinned 500-file, 20-cop benchmark corpus, rustocop is currently
about 42 times faster than RuboCop with Prism. Both tools produced identical
normalized JSON before measurement.

| Files | rustocop | RuboCop (Prism) | Speedup |
| ---: | ---: | ---: | ---: |
| 1 | 5.41 ms | 417.51 ms | 77.20× |
| 25 | 6.01 ms | 432.35 ms | 71.96× |
| 100 | 6.92 ms | 438.53 ms | 63.40× |
| 500 | 11.66 ms | 494.82 ms | 42.44× |
<!-- generated:rubocop-prism:end -->

This uses RuboCop 1.87.0 with Prism, caching disabled, and server mode disabled.

The fixtures total only 9,110 bytes, so this mostly measures startup,
orchestration, and many tiny file reads—not performance on a representative
application. See the [full methodology and results](docs/performance.md), or
reproduce the comparison with:

```sh
bundle exec ruby script/benchmark_rubocop_prism.rb
```

### Ruby custom cops

<!-- generated:mixed-custom:start -->
Rustocop can keep recognized built-in cops native while delegating explicitly
selected Ruby custom cops to RuboCop. On the same 500 files, 20 native cops plus
one custom cop took 498.24 ms, versus 10.86 ms for pure native Rustocop and
508.84 ms for pure RuboCop.

| 500-file mode | Median |
| --- | ---: |
| Pure native, 20 built-in cops | **10.86 ms** |
| Mixed, 20 native + 1 Ruby custom cop | **498.24 ms** |
| Pure RuboCop, all 21 cops | **508.84 ms** |
<!-- generated:mixed-custom:end -->

The mixed report exactly matched RuboCop, but the Ruby custom cop still imposed
almost all of RuboCop's startup and parsing cost. See the [mixed custom-cop
benchmark](benchmark/mixed-custom-cops.md) for entrypoint overhead, p95 results,
and methodology.

### Real-project performance

The repository also has a sustained-workload runner for all ten pinned
projects. The checked-in timing table is a dated 2026-08-19 three-project
baseline and is no longer presented as current correctness evidence. See the
[project benchmark note](benchmark/project-benchmarks.md) for those historical
timings and the stronger requirements for publishing a replacement run.

## Using it locally

For now, run rustocop from a checkout. You need Ruby 3.1 or newer, Bundler, and a
working Rust toolchain.

```sh
git clone https://github.com/myxoh/rustocop.git
cd rustocop
bundle install
bundle exec rake build:native
```

Run it against a Ruby project by passing the project path:

```sh
/path/to/rustocop/exe/rustocop /path/to/your/ruby-project
```

From inside this repository, the equivalent development command is:

```sh
bundle exec ruby exe/rustocop /path/to/your/ruby-project
```

Common commands:

```sh
# Run one cop
exe/rustocop --only Style/ArrayJoin /path/to/project

# Run a department
exe/rustocop --only Style /path/to/project

# Pass the target config. Only part of RuboCop's config is understood so far.
exe/rustocop --config /path/to/project/.rubocop.yml /path/to/project

# Delegate an explicitly selected custom cop while built-in cops stay native
exe/rustocop --require ./lib/rubocop/cop/custom/no_foo.rb \
  --only Style/ArrayJoin,Custom/NoFoo /path/to/project

# Apply available corrections; use this on a clean working tree
exe/rustocop -A /path/to/project

# Produce RuboCop-style JSON
exe/rustocop --format json /path/to/project

# Inspect files in parallel using the machine's available CPU count
exe/rustocop --parallel /path/to/project

# Or set the worker count explicitly
exe/rustocop --jobs 4 /path/to/project

# See every cop rustocop currently advertises
exe/rustocop --show-cops
```

Custom delegation requires `--require` or `--plugin` and an explicit `--only`
list. Names advertised by `--show-cops` remain native; unknown names are passed
to RuboCop. Mixed runs currently reject autocorrection and `--stdin`, because
independent native and Ruby correction passes would not have safe ordering
semantics.

The intended setup is deliberately boring:

```sh
# Fast local feedback
/path/to/rustocop/exe/rustocop .

# Authoritative CI check
bundle exec rubocop
```

Do not remove RuboCop from CI just because rustocop is fast on your machine.
Check the [compatibility evidence table](docs/compatibility.md) and the
[real-project parity report](docs/real-project-parity.md) before depending on a cop.

## Current support

- Ruby gem entrypoint: `exe/rustocop`
- Native binary contract: `libexec/rustocop-native`
- Development fallback: `libexec/rustocop-ruby`
- Rust source: `crates/rustocop`
- Native registry entries for all 606 RuboCop 1.87 built-ins. RuboCop extension
  departments and project-specific cops are not native.
- A shared Prism parse and AST visitor powers the native cop registry.
- `--show-cops` prints the native support registry.
- [The built-in cop evidence matrix](docs/cop-support.md) records current
  project status and whether a minimized project regression exists.
- [The RuboCop + Prism performance verification](docs/performance.md) records
  reproducible end-to-end timings and JSON parity checks for the shared
  500-file, 20-cop corpus.

## Native architecture

Every inspected file is parsed once with Prism. Enabled AST cops are registered
with a shared visitor, receive typed nodes plus ancestor context, and report
byte-accurate source ranges. Compatible edits are collected during the traversal
and applied as one batch. The differential compatibility suite runs 20 cops
against 500 generated and committed Ruby fixture files, both cop-by-cop and as a
single corpus, and compares their JSON reports directly with RuboCop.

The native registry contains entries for all 606 RuboCop 1.87 built-ins. That
number does not imply compatibility; fixture validation and project parity are
the only current behavioral evidence.

## Upstream RuboCop contract

The official RuboCop 1.87.0 cop specs are vendored under
`spec/upstream/rubocop-1.87.0` at tag `v1.87.0`, commit
`e5b788dba181ad94de30cfbad661c5d6aa08a4e5`. The unchanged public suite contains
28,479 RSpec examples. It currently runs with zero failures and six pendings
declared by RuboCop itself.

The capture harness executes RuboCop's test DSL and records the resulting
source, configuration, path, Ruby version, offenses, and correction. It does
not infer expectations by scraping spec source. All 606 registered built-in
cops have executable captured cases. These cases are the upstream fixture
oracle: they become compatibility evidence only when Rustocop matches the
captured diagnostics and corrections. Project-exact output is the broader guard
against cases absent from upstream specs.

```sh
bundle exec ruby script/extract_upstream_cop_specs.rb
bundle exec ruby script/compare_upstream_cop_specs.rb
```

Generated corpora and reports live under `tmp/` and are intentionally ignored.
The comparison command is diagnostic-only for now; correction parity remains a
separate required fixture gate.

## Development

Read [Building a cop](docs/building-a-cop.md), the
[Prism cop DSL reference](docs/adding-a-prism-cop.md),
[the architecture](docs/architecture.md), and
[the rules of engagement](CONTRIBUTING.md) before adding cops. The
[substantial-work roadmap](docs/substantial-work.md) records the shared
correctness and architecture work that does not belong in the generated
per-cop queue. The default spec task enforces the documented module and function
complexity ceilings.

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
bundle exec ruby script/generate_compatibility_corpus.rb --check
```

The reproducible 500-example corpus is one fixture layer, not a compatibility
claim by itself. Benchmarks use the separate pinned `benchmark/corpus.json`, so
improving a correctness fixture does not silently redefine historical
performance work.

Run the complete upstream differential with its non-regression gate:

```sh
RUSTOCOP_NATIVE_PATH=crates/rustocop/target/debug/rustocop \
  bundle exec ruby script/compare_upstream_cop_specs.rb \
  --baseline spec/upstream/rubocop-1.87.0/status.yml \
  --report tmp/full-compatibility.json

bundle exec ruby script/report_compatibility_drift.rb \
  tmp/full-compatibility.json \
  --output tmp/compatibility-promotion-drift.md
```

Generate the public per-cop matrix and current gap queue only from a complete
ten-project audit:

```sh
ruby script/generate_project_parity_docs.rb \
  tmp/project-parity/all-cops-current.json
```

The generated [gap queue](docs/remaining-cops.md) is ordered by unmatched
complete signatures and deliberately ignores the retired qualification labels.

Build the native binary when Rust is installed:

```sh
bundle exec rake build:native
```

The build task copies the release binary to `libexec/rustocop-native`, which is
what `exe/rustocop` launches by default. Set `RUSTOCOP_DISABLE_NATIVE=1` to force
the Ruby fallback while developing the gem wrapper or compatibility specs.
