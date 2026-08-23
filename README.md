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

The active native registry contains 523 RuboCop built-in cops. Another 83 are
intentionally pending and unregistered because their implementations are not
project-exact or RuboCop 1.87 cannot produce stable comparison output. Public
support evidence comes from RuboCop-derived fixtures and complete project
signatures; the old Verified/Heuristic qualification scoreboard is not used.
The configured and audited real-project corpus contains 50 pinned repositories.

## Validation standard

The gold standard is executable behavior checked against RuboCop 1.87.0:

1. RuboCop-derived upstream, adversarial, autocorrection, and minimized
   real-project fixtures must pass with matching diagnostics and corrected
   source.
2. Complete diagnostic signatures must match across all 50 pinned real
   projects: path, cop, severity, message, and complete source range.

A registered cop with no exercised fixture is unvalidated. A cop with exact
output on every pinned project is **project-exact** for that corpus and
configuration. Neither offense-count similarity, a captured-case label, nor an
old manual review record is accepted as compatibility evidence.

The minimized real-project corpus currently contains 336 previously passing
cases and six configuration-mutation cases. The new 50-project mismatches have
not yet been minimized; these fixtures remain regression coverage, not a
substitute for the complete project comparison.

After restoring eleven cops with structural implementations, the fixture review
updated at `2026-08-23T13:05:46-04:00` contains 24,298 retained executable cases.
All 24,297 comparable cases match RuboCop 1.87.0 diagnostics, and all correction
expectations also match; 523/523 active cops pass every retained fixture. One captured case
with unsupported synthetic upstream state is explicitly excluded rather than
counted as passing.

## Fifty-project output parity

The real-project matrix asks whether each cop emits the same path, severity,
message, and source range as RuboCop across 85,471 Ruby files in 50 projects.

The complete audit updated at `2026-08-23T13:19:41-04:00` covers all 523 active
cops. It leaves 296 project-exact cops, 53 dormant cops, 172 mismatches, one
native crash, and one RuboCop command-line error.

| Real-project classification | Complete checkpoint |
| --- | ---: |
| Project-exact | 296 / 523 (56.6%) |
| Exact but dormant | 53 / 523 |
| Mismatching | 172 / 523 |
| Rust crashes | 1 / 523 |
| RuboCop `--only` limitation | 1 / 523 |

The checkpoint is bound to Rust source `7dd38fc` and native binary SHA-256
`efb33b58b19285d581a49b84a16544f5309b50b083adf3db3ddd606410f4ab24`.
Its RuboCop reference SHA-256 is
`49dee51b7573e6faf0af6a75f370b9039a814d62a6ccf869faa51dab84eac99d`.
Project-exact status is the strongest current diagnostic evidence. Unexercised
configuration and autocorrection branches still require RuboCop-derived
fixtures.

See [the compatibility evidence table](docs/compatibility.md) for fixture and
project matching by cop, and [the real-project parity report](docs/real-project-parity.md)
for the pinned projects, limitations, and reproduction commands. Fixture parity
and project parity are deliberately separate: upstream examples prove the
captured contract, while the 50-project run exposes unrepresented syntax,
configuration, and negative cases. The related
[non-scalable implementation catalog](docs/non-scalable-implementations.md)
tracks the cops whose present design appears too narrow to generalize.

Complete project audits reuse a checked-in, input-validated RuboCop diagnostic
reference, so routine compatibility refreshes run only Rustocop. The reference
is regenerated explicitly when RuboCop, the audit configuration, selected cops,
or pinned project revisions change.

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

The repository also has a sustained-workload runner for all 50 pinned
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
- 523 active native RuboCop 1.87 built-ins and 83 explicitly unregistered,
  intentionally pending cops. RuboCop extension departments and project-specific
  cops are not native.
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

The native registry intentionally excludes 83 withdrawn implementations. A cop
returns only after a scalable implementation passes fixtures and project
parity; registration by itself is never compatibility evidence.

## Upstream RuboCop contract

The official RuboCop 1.87.0 cop specs are vendored under
`spec/upstream/rubocop-1.87.0` at tag `v1.87.0`, commit
`e5b788dba181ad94de30cfbad661c5d6aa08a4e5`. Specs for the 83 intentionally
pending cops are excluded from the active fixture corpus.

The capture harness executes RuboCop's test DSL and records the resulting
source, configuration, path, Ruby version, offenses, and correction. It does
not infer expectations by scraping spec source. The active corpus contains
executable captured cases for all 523 active cops and excludes every cop in the
intentionally-pending manifest. These cases become compatibility evidence only
when Rustocop matches the captured diagnostics and corrections. Project-exact
output is the broader guard against cases absent from upstream specs.

```sh
bundle exec ruby script/extract_upstream_cop_specs.rb
bundle exec ruby script/compare_upstream_cop_specs.rb
```

Generated corpora and reports live under `tmp/` and are intentionally ignored.
The comparison command is diagnostic-only for now; correction parity remains a
separate required fixture gate.

Prepare or restore the pinned 50-project corpus without running either linter:

```sh
PROJECT_BENCHMARK_PREPARE_ONLY=1 \
  bundle exec ruby script/benchmark_projects.rb
```

The immutable archives and filtered corpora are cached under
`tmp/project-benchmarks/`; a repeated preparation reuses them.

## Development

The maintained documentation is intentionally small:

- [Building a cop](docs/building-a-cop.md) is the end-to-end implementation and
  promotion workflow; the [Prism cop DSL reference](docs/adding-a-prism-cop.md)
  documents its callback and correction APIs.
- [Architecture](docs/architecture.md) and the
  [rules of engagement](CONTRIBUTING.md) define repository boundaries and
  validation requirements.
- [Compatibility](docs/compatibility.md), [cop support](docs/cop-support.md),
  [remaining cops](docs/remaining-cops.md), and
  [real-project parity](docs/real-project-parity.md) are the current generated
  or evidence-backed status sources.
- [Non-scalable implementations](docs/non-scalable-implementations.md) and the
  [substantial-work roadmap](docs/substantial-work.md) contain the active manual
  backlog. Historical checkpoints and completed review notes live in Git
  history instead of permanent Markdown files.

The default spec task enforces the documented module and function complexity
ceilings.

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
50-project audit:

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
