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
cops, but that is an implementation inventory, not a correctness claim. The old
**verified**, **heuristic**, and **missing** labels describe captured-suite
results only. They do not count toward the qualification ledger below.

## Cop qualification progress

<!-- generated:qualification-progress:start -->
Qualification restarted from zero on 2026-08-19; the table now reflects the
authoritative records under `qualification/work/`. "Recorded evidence" means
a record contains all required evidence. "Current-source credit" additionally
requires the recorded Rust files to be unchanged from that record's pinned SHA.

| Check | Recorded evidence | Current-source credit | Current progress |
| --- | ---: | ---: | ---: |
| 1. Manual source verification | 185 / 606 | 167 / 606 | 27.6% |
| 2. Ported upstream unit tests | 185 / 606 | 167 / 606 | 27.6% |
| 3. Edge-case fixtures | 185 / 606 | 167 / 606 | 27.6% |
| 4. Real-world true positives | 185 / 606 | 167 / 606 | 27.6% |
| 5. Real-world true negatives | 185 / 606 | 167 / 606 | 27.6% |
| **Fully qualified** | **185 / 606** | **167 / 606** | **27.6%** |

185 cops have complete five-check records. 18 of those
records are currently invalidated by later changes to their Rust source, leaving
**167 currently qualified cops**. The RuboCop reference is
`e5b788dba181ad94de30cfbad661c5d6aa08a4e5`; the current native Rust source is `15032c62724a1bfcd1d7199a9342188ff3b96ee4`.

| Department | Currently qualified | Complete records | Stale records |
| --- | ---: | ---: | ---: |
| `Bundler` | 0 / 7 | 0 | 0 |
| `Gemspec` | 0 / 10 | 0 | 0 |
| `Layout` | 0 / 100 | 0 | 0 |
| `Lint` | 0 / 154 | 0 | 0 |
| `Metrics` | 0 / 10 | 0 | 0 |
| `Migration` | 0 / 1 | 0 | 0 |
| `Naming` | 0 / 19 | 0 | 0 |
| `Security` | 0 / 7 | 0 | 0 |
| `Style` | 167 / 298 | 185 | 18 |

See [the detailed qualification ledger](docs/qualification-progress.md) for
batch totals, every recorded cop, pinned SHAs, and the records needing revalidation.
<!-- generated:qualification-progress:end -->

## Performance

<!-- generated:rubocop-prism:start -->
On the pinned 500-file, 20-cop benchmark corpus, rustocop is currently
about 60 times faster than RuboCop with Prism. Both tools produced identical
normalized JSON before measurement.

| Files | rustocop | RuboCop (Prism) | Speedup |
| ---: | ---: | ---: | ---: |
| 1 | 3.04 ms | 402.38 ms | 132.45× |
| 25 | 3.49 ms | 400.83 ms | 114.72× |
| 100 | 4.50 ms | 412.80 ms | 91.65× |
| 500 | 7.79 ms | 467.51 ms | 60.02× |
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
one custom cop took 455.44 ms, versus 8.98 ms for pure native Rustocop and
474.89 ms for pure RuboCop.

| 500-file mode | Median |
| --- | ---: |
| Pure native, 20 built-in cops | **8.98 ms** |
| Mixed, 20 native + 1 Ruby custom cop | **455.44 ms** |
| Pure RuboCop, all 21 cops | **474.89 ms** |
<!-- generated:mixed-custom:end -->

The mixed report exactly matched RuboCop, but the Ruby custom cop still imposed
almost all of RuboCop's startup and parsing cost. See the [mixed custom-cop
benchmark](benchmark/mixed-custom-cops.md) for entrypoint overhead, p95 results,
and methodology.

### Real Rails projects

The sustained-workload benchmark runs deliberately strict built-in rules over
three pinned open-source Rails projects. Unlike the small compatibility corpus,
these reports are not yet identical: the exact-match column is the new
correctness target, matching path, cop, severity, message, and source range.

| Project | Ruby files | rustocop, 4 workers | RuboCop Prism | Speedup | Exact matches / RuboCop offenses |
| --- | ---: | ---: | ---: | ---: | ---: |
| [Chatwoot](https://github.com/chatwoot/chatwoot/tree/8d93d69e8e356216e85c28de7c4240e66b8e83fa) | 1,842 | **104 ms** | 5.72 s | **54.9×** | 429 / 48,951 |
| [RubyGems.org](https://github.com/rubygems/rubygems.org/tree/3201f8831866f82eb9acd7f66287a978d0e59079) | 1,337 | **50 ms** | 2.69 s | **53.4×** | 273 / 4,195 |
| [GitLab CE](https://github.com/gitlabhq/gitlabhq/tree/67a526442c20d20b6e80ebf916bd766b54018c5e) | 30,894 | **1.82 s** | 106.18 s | **58.5×** | 6,780 / 698,393 |

All timings are medians from five measured runs after warmup, with RuboCop's
cache and server disabled. The configuration intentionally forces offenses
using eight core cops; no custom cops or extensions are loaded. See the
[project benchmark methodology and full median/p95 results](benchmark/project-benchmarks.md).

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

# Project .rubocop.yml files are discovered automatically. An explicit config is also supported.
exe/rustocop --config /path/to/project/.rubocop.yml /path/to/project

# Include enabled extension and custom cops through RuboCop (substantially slower)
exe/rustocop --included-non-native-cops /path/to/project

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

For normal runs, the Ruby entrypoint asks RuboCop to resolve the effective
configuration, including inherited configuration, plugins, department settings,
`DisabledByDefault`, and `NewCops`. Rustocop runs the enabled cops from the base
RuboCop package. Enabled extension and custom cops are ignored with a warning by
default; `--included-non-native-cops` delegates those cops back to RuboCop and
merges their results. This is substantially slower because the delegated files
are parsed a second time in Ruby.

Explicit `--require`/`--plugin` plus `--only` custom-cop delegation remains
supported. Mixed runs currently reject autocorrection and `--stdin`, because
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
Check the [support matrix](docs/cop-support.md) before depending on a cop, and be
especially cautious with anything marked heuristic.

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
- A shared Prism parse and AST visitor powers the native cop registry. The
  previous captured-suite classification reported 361 cops as verified and 245
  as heuristic, but those historical labels grant no credit in the new
  five-check qualification ledger.
- `--show-cops` prints the native support registry.
- [The legacy captured-suite support matrix](docs/cop-support.md) records the
  old verified, heuristic, and missing classification. It is retained as
  engineering evidence, not as the current qualification record.
- [The RuboCop + Prism performance verification](docs/performance.md) records
  reproducible end-to-end timings and JSON parity checks for the shared
  500-file, 20-cop corpus.

Current qualification totals and invalidated records are reported in the
[generated qualification ledger](docs/qualification-progress.md).

## Native architecture

Every inspected file is parsed once with Prism. Enabled AST cops are registered
with a shared visitor, receive typed nodes plus ancestor context, and report
byte-accurate source ranges. Compatible edits are collected during the traversal
and applied as one batch. The differential compatibility suite runs 20 cops
against 500 generated and committed Ruby fixture files, both cop-by-cop and as a
single corpus, and compares their JSON reports directly with RuboCop.

The native registry contains entries for all 606 RuboCop 1.87 built-ins. The
qualification ledger intentionally started at zero regardless of previous
captured diagnostic or correction results; current progress is generated from
the new five-check records above.

## Upstream RuboCop contract

The official RuboCop 1.87.0 cop specs are vendored under
`spec/upstream/rubocop-1.87.0` at tag `v1.87.0`, commit
`e5b788dba181ad94de30cfbad661c5d6aa08a4e5`. The unchanged public suite contains
28,479 RSpec examples. It currently runs with zero failures and six pendings
declared by RuboCop itself.

The capture harness executes RuboCop's test DSL and records the resulting
source, configuration, path, Ruby version, offenses, and correction. It does
not infer expectations by scraping spec source. All 606 registered built-in
cops have executable captured cases. This is useful historical evidence, but it
does not satisfy any new qualification check until that cop is explicitly
reviewed and recorded under the five-check process.

```sh
bundle exec ruby script/extract_upstream_cop_specs.rb
bundle exec ruby script/compare_upstream_cop_specs.rb
```

Generated corpora and reports live under `tmp/` and are intentionally ignored.
The comparison command is diagnostic-only for now; correction parity remains a
separate required gate before a cop can be marked fully upstream-compatible.

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

The correctness corpus is status-checked and may contain only Verified cops.
Benchmarks use the separate pinned `benchmark/corpus.json`, so improving a
correctness fixture does not silently redefine historical performance work.

Run the complete upstream differential with its non-regression gate, then
regenerate the prioritized remaining-cop queue:

```sh
RUSTOCOP_NATIVE_PATH=crates/rustocop/target/debug/rustocop \
  bundle exec ruby script/compare_upstream_cop_specs.rb \
  --baseline spec/upstream/rubocop-1.87.0/status.yml \
  --report tmp/full-compatibility.json

RUSTOCOP_NATIVE_PATH=crates/rustocop/target/debug/rustocop \
  bundle exec ruby script/generate_remaining_cop_plan.rb \
  tmp/full-compatibility.json

bundle exec ruby script/report_compatibility_drift.rb \
  tmp/full-compatibility.json \
  --output tmp/compatibility-promotion-drift.md
```

The generated [remaining-cop plan](docs/remaining-cops.md) distinguishes partial
implementations, quick structural additions, and cops blocked on shared engine
capabilities.

Build the native binary when Rust is installed:

```sh
bundle exec rake build:native
```

The build task copies the release binary to `libexec/rustocop-native`, which is
what `exe/rustocop` launches by default. Set `RUSTOCOP_DISABLE_NATIVE=1` to force
the Ruby fallback while developing the gem wrapper or compatibility specs.
