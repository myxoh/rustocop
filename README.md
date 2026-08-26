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

The active native registry contains all 606 RuboCop 1.87 built-in cops, and the
intentionally-pending dataset is empty. Public support evidence comes from
RuboCop-derived fixtures and complete project signatures; the old
Verified/Heuristic qualification scoreboard is not used. The configured and
audited real-project corpus contains 50 pinned repositories.

The separate [RuboCop compatibility layer](docs/rubocop-compatibility-layer.md)
tracks all 228 shared components from RuboCop 1.87.0 and rubocop-ast
1.49.1. The strict generated ledger accounts for all of them: 191 translated,
30 native equivalents, and 7 facilities that do not apply in Rust, with no
partial or pending components. It is also the authority for all 2,586 APIs
discovered from both Ruby syntax and the pinned gems' runtime-defined method
surface, including `attr_*`, `Struct`, delegation, `define_method`, and
`class_eval` generation. Source-shaped Rust functions, explicit ownership for
ambiguous operations in consolidated files, and source-reviewed,
destination-checked equivalences are counted. A public translated function is
also rejected as incomplete until it is exercised outside its own definition;
the ledger currently has zero unexercised public targets. All 3,139
expanded examples from the 83 shared upstream RSpec suites are recorded in a
cached inventory. Each individual RSpec ID is bound to a named executable Rust
test, its upstream description hash, and reviewable semantic terms or an
explicit source rule; the complete binding is digest-checked. Its
[generated progress ledger](docs/rubocop-compatibility-progress.md) tracks the
source hashes and upstream spec ports. Ten cops now consume the layer through
reviewed Prism adapters; all 649 of their cached unit contracts pass and all ten
are exact on the 50-project corpus. Shared-layer completion and cop parity
remain separate claims.

## Validation standard

The gold standard is executable behavior checked against RuboCop 1.87.0:

1. RuboCop-derived upstream, adversarial, autocorrection, and minimized
   real-project unit contracts must pass with matching diagnostics and corrected
   source.
2. Complete diagnostic signatures must match across all 50 pinned real
   projects: path, cop, severity, message, and complete source range.

A registered cop with no exercised fixture is unvalidated. A cop with exact
output on every pinned project is **project-exact** for that corpus and
configuration. Neither offense-count similarity, a captured-case label, nor an
old manual review record is accepted as compatibility evidence.

The controlled corpus currently contains 29,526 cop-owned fixture rows across
all 606 cops. The strict differential runner executes 29,616 cached cases,
including diagnostics plus the applicable `-a` and `-A` correction contracts,
and all of them pass. Whole projects remain transient and are not a substitute
for this focused coverage: real-project failures are minimized into cop-owned
fixtures before implementations change.

## Fifty-project output parity

The real-project matrix asks whether each cop emits the same path, severity,
message, and source range as RuboCop across 85,471 Ruby files in 50 projects.

The complete audit updated at `2026-08-26T14:47:18-04:00` covers all 606 built-in
cops. All 531 cops exercised by the corpus are project-exact. The remaining 75
cops are dormant in these projects; there are no mismatches, native crashes, or
RuboCop gate errors.

| Real-project classification | Complete checkpoint |
| --- | ---: |
| Project-exact | 531 / 606 (100% of 531 exercised cops) |
| Exact but dormant | 75 / 606 |
| Mismatching | 0 / 606 |
| Rust crashes | 0 / 606 |
| RuboCop gate errors | 0 / 606 |

The dirty-worktree checkpoint is bound to cop-source SHA-256
`3e72741c0ea482dfb92beb24594d784c1c1862ed67dbec53a5b93c3ec97352b1` and
native binary SHA-256
`d06f7ed679bd43ff8212ff12da956473ba552014ab416b4f9a993fdd64efa4d3`.
Its RuboCop reference SHA-256 is
`d9f16acf805c8a76b324447497ec18c5acbd3e917b5dd24656c5eaf878a5620c`.
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
or pinned project revisions change. Evidence produced from a dirty worktree is
bound to a deterministic SHA-256 of every native cop source file as well as the
compiled binary, so unrelated edits do not invalidate it and cop changes cannot
silently reuse stale results.

A warm complete audit measured on 2026-08-26 took 66.4 seconds wall-clock with
50/50 native cache hits and no mismatches. Caching avoids both linter runs, but
the runner must still load and exactly compare the large stored diagnostic
inventories. Focused cop audits remain the fast development path; the complete
audit is the final corpus gate.

## Performance

<!-- generated:rubocop-prism:start -->
On the pinned 500-file, 20-cop benchmark corpus, rustocop is currently
about 36 times faster than RuboCop with Prism. Both tools produced identical
normalized JSON before measurement.

| Files | rustocop | RuboCop (Prism) | Speedup |
| ---: | ---: | ---: | ---: |
| 1 | 8.54 ms | 497.19 ms | 58.22× |
| 25 | 9.24 ms | 566.01 ms | 61.28× |
| 100 | 15.02 ms | 722.40 ms | 48.09× |
| 500 | 18.00 ms | 646.75 ms | 35.92× |
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

# Apply only corrections RuboCop marks safe; use this on a clean working tree
exe/rustocop -a /path/to/project

# Apply safe and unsafe corrections
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
- All 606 RuboCop 1.87 built-in cops are registered natively. RuboCop extension
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
and applied as one batch. Every committed example is owned by its target cop
under `spec/fixtures/cops/<Department>/<Cop>/unit/`; whole-project artifacts
remain transient under ignored `tmp/` paths.

The intentionally-pending manifest is currently empty. A cop remains registered
only after a scalable implementation passes the fixture gate; registration by
itself is never compatibility evidence, and only an exact full-project result is
project parity.

## Upstream RuboCop contract

The official RuboCop 1.87.0 cop specs are vendored under
`spec/upstream/rubocop-1.87.0` at tag `v1.87.0`, commit
`e5b788dba181ad94de30cfbad661c5d6aa08a4e5`. The intentionally-pending manifest
is empty, so no built-in cop specs are excluded from the active fixture corpus.

The capture harness executes RuboCop's test DSL and records the resulting
source, configuration, path, Ruby version, parser, encodings, offenses, and
correction. It does not infer expectations by scraping spec source. The
committed cache contains 29,526 cop-owned rows and stores exact diagnostics plus
distinct safe `-a` and all-cop `-A` results. The strict runner currently checks
29,616 cached cases. These cases become compatibility evidence only when
Rustocop matches the cache.
Project-exact output is the broader guard against cases absent from unit
contracts.

```sh
bundle exec ruby script/extract_upstream_cop_specs.rb
bundle exec ruby script/generate_unit_fixtures.rb
```

The raw capture lives under ignored `tmp/`; the deduplicated per-cop cache is
committed under `spec/fixtures/cops/`. Routine Rustocop work reads the cache and
does not invoke RuboCop.

Prepare or restore the pinned 50-project corpus without running either linter:

```sh
PROJECT_BENCHMARK_PREPARE_ONLY=1 \
  bundle exec ruby script/benchmark_projects.rb
```

The immutable archives and filtered corpora are cached under
`tmp/project-benchmarks/`; a repeated preparation reuses them.

Project-derived regressions and configuration variations are part of the same
cached unit contracts. Only complete repository audits start the project-scale
comparison workflow.

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

The canonical repository gate is:

```sh
bundle exec rake
```

It builds the release binary, rejects Clippy warnings, checks generated
inventories and compatibility contracts, enforces architecture boundaries, and
runs the complete spec suite. Existing oversized Rust modules are recorded with
exact, non-increasing ceilings in
[the architecture-debt manifest](spec/architecture_debt.yml): growth fails the
gate, while reductions must lower or remove the corresponding ceiling. New
modules receive no architecture-debt allowance. Run Rust formatting separately
with `cargo fmt --manifest-path crates/rustocop/Cargo.toml --all -- --check`.

Install dependencies:

```sh
bundle install
```

The explicit spec task is equivalent to the default task:

```sh
bundle exec rake spec
```

Run the cached controlled unit contract for only the cops being changed:

```sh
ruby script/verify_cop.rb Style/StringLiterals Layout/TrailingWhitespace
```

Run all 29,616 cached checks, or explicitly refresh the slow RuboCop cache:

```sh
bundle exec rake fixtures:unit
bundle exec rake fixtures:refresh_unit
```

The ownership gate rejects orphaned and cross-cop fixture paths:

```sh
bundle exec ruby script/check_fixture_ownership.rb
```

See [`spec/fixtures/README.md`](spec/fixtures/README.md) for the unit corpus.
Benchmarks use the separate pinned `benchmark/corpus.json`, so improving a
correctness fixture does not silently redefine historical performance work.

Run the complete upstream differential. The command fails if any retained case
differs:

```sh
RUSTOCOP_NATIVE_PATH=crates/rustocop/target/debug/rustocop \
  bundle exec ruby script/compare_upstream_cop_specs.rb \
  --report tmp/full-compatibility.json
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
