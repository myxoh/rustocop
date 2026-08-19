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

The useful version of this project is good enough to run locally for fast
feedback before CI runs real RuboCop and catches anything rustocop missed or got
wrong. It is not currently supposed to replace RuboCop as the final lint or
security gate.

Maybe enough interest, use, and scrutiny will eventually turn this into a real
linter. Until then, expect incomplete cop and configuration compatibility,
false positives, false negatives, and breaking changes.

We do at least run this against RuboCop's own specs. Every built-in cop is listed
as **verified**, **heuristic**, or **missing**. Verified means its captured
diagnostics and corrections pass; it does not mean the whole project is ready
to replace RuboCop.

## Performance

On the committed 500-file, 20-cop compatibility corpus, rustocop is currently
about 53 times faster than RuboCop with Prism. Both tools produced identical
normalized JSON before measurement.

| Files | rustocop | RuboCop (Prism) | Speedup |
| ---: | ---: | ---: | ---: |
| 1 | 3.02 ms | 398.12 ms | 132.00× |
| 25 | 3.46 ms | 402.18 ms | 116.24× |
| 100 | 4.33 ms | 414.00 ms | 95.59× |
| 500 | **8.96 ms** | **477.11 ms** | **53.27×** |

This uses RuboCop 1.87.0 with Prism, caching disabled, and server mode disabled.

The fixtures total only 9,090 bytes, so this mostly measures startup,
orchestration, and many tiny file reads—not performance on a representative
application. See the [full methodology and results](docs/performance.md), or
reproduce the comparison with:

```sh
bundle exec ruby script/benchmark_rubocop_prism.rb
```

### Ruby custom cops

Rustocop can keep recognized built-in cops native while delegating explicitly
selected Ruby custom cops to RuboCop. On the same 500 files, 20 native cops plus
one custom cop took 456.12 ms, versus 9.07 ms for pure native Rustocop and
478.47 ms for pure RuboCop.

| 500-file mode | Median |
| --- | ---: |
| Pure native, 20 built-in cops | **9.07 ms** |
| Mixed, 20 native + 1 Ruby custom cop | **456.12 ms** |
| Pure RuboCop, all 21 cops | 478.47 ms |

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
- A shared Prism parse and AST visitor powers the native cop registry. Against
  RuboCop 1.87, 361 built-in cops are verified and the other 245 have heuristic
  implementations. Verification uses 28,623 captured upstream diagnostic and
  correction cases rather than only the smaller performance corpus.
- `--show-cops` prints the native support registry.
- [The complete built-in cop support matrix](docs/cop-support.md) records every
  RuboCop 1.87 cop as verified, heuristic, or missing. Regenerate it with
  `bundle exec ruby script/generate_cop_support.rb`.
- [The RuboCop + Prism performance verification](docs/performance.md) records
  reproducible end-to-end timings and JSON parity checks for the shared
  500-file, 20-cop corpus.

The remaining heuristic implementations are documented that way rather than
presented as full compatibility.

## Native architecture

Every inspected file is parsed once with Prism. Enabled AST cops are registered
with a shared visitor, receive typed nodes plus ancestor context, and report
byte-accurate source ranges. Compatible edits are collected during the traversal
and applied as one batch. The differential compatibility suite runs 20 cops
against 500 generated and committed Ruby fixture files, both cop-by-cop and as a
single corpus, and compares their JSON reports directly with RuboCop.

The current RuboCop 1.87 matrix contains 361 upstream-spec-verified cops, 245
heuristic native implementations, and no missing built-in cops. A cop is only
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

Read [Building a cop](docs/building-a-cop.md), the
[Prism cop DSL reference](docs/adding-a-prism-cop.md),
[the architecture](docs/architecture.md), and
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
