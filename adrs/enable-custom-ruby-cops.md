# Enable Ruby custom cops through mixed execution

- Status: Accepted
- Date: 2026-08-19

## Context

Rustocop implements built-in cops natively, but real projects often load local
Ruby cops through RuboCop's `--require` or `--plugin` options. Reimplementing
every project-specific cop in Rust is not always practical, while silently
ignoring required cops would make local results misleading.

A Ruby custom cop cannot directly consume Rustocop's Prism tree. Running one
therefore requires a Ruby process, the RuboCop framework, and a second parse of
every inspected file.

## Decision

Rustocop supports a mixed execution mode when all of the following are true:

1. At least one `--require` or `--plugin` option is present.
2. The run has an explicit `--only` list.
3. At least one selected name is not in Rustocop's advertised cop inventory.

Names advertised by `rustocop --show-cops` remain native. Unknown selections
are passed to RuboCop together with the loader options, configuration path, and
file targets.

The native process starts RuboCop before beginning native inspection, allowing
both engines to work concurrently. RuboCop runs only the custom selections,
with its cache and server disabled, and returns JSON. Rustocop then merges the
custom diagnostics into its native results and restores deterministic offense
ordering.

Example:

```sh
exe/rustocop \
  --require ./lib/rubocop/cop/custom/no_foo.rb \
  --only Style/ArrayJoin,Custom/NoFoo \
  /path/to/project
```

## Performance result

The synthetic benchmark uses the pinned 500-file benchmark corpus,
containing 9,110 bytes of Ruby. It enables 20 built-in cops plus one custom cop
that raises one offense in each file. RuboCop 1.87.0 uses Prism, with caching
and server mode disabled. Results are medians of seven runs after two warmups.

<!-- generated:mixed-custom-results:start -->
| Variant | Median | p95 | Relative to native binary |
| --- | ---: | ---: | ---: |
| Rustocop native binary, 20 built-in cops | 10.86 ms | 11.45 ms | 1.0× |
| Rustocop Ruby entrypoint, 20 built-in cops | 95.61 ms | 99.32 ms | 8.8× |
| Mixed native binary, 20 native + 1 custom cop | 498.24 ms | 507.18 ms | 45.9× |
| Mixed Ruby entrypoint, 20 native + 1 custom cop | 573.14 ms | 612.80 ms | 52.8× |
| RuboCop, custom cop only | 482.53 ms | 491.37 ms | 44.4× |
| RuboCop, all 20 built-ins + custom cop | 508.84 ms | 519.27 ms | 46.8× |

The direct mixed run was 2.1% faster than pure RuboCop and produced
identical normalized JSON. One Ruby custom cop still made this tiny-corpus run
45.9 times slower than pure native Rustocop because RuboCop must
start Ruby, load the custom cop, and build a second set of Prism trees.
<!-- generated:mixed-custom-results:end -->

The existing Ruby `exe/rustocop` entrypoint adds approximately 75 ms before it
replaces itself with the native executable. On this startup-dominated corpus,
that makes entrypoint-based mixed execution slower than invoking RuboCop
directly. Engine benchmarks should use `libexec/rustocop-native`.

## Consequences

Positive consequences:

- Projects can retain Ruby custom cops while moving supported built-in work to
  Rustocop.
- Native and Ruby inspections overlap rather than running sequentially.
- Mixed diagnostics have been verified against pure RuboCop output.
- RuboCop receives only custom selections, so expensive built-in cops do not
  need to run twice.

Negative consequences:

- A single Ruby custom cop forfeits most of Rustocop's startup advantage.
- Every file is read and parsed independently by both engines.
- Concurrent processes increase CPU and peak-memory pressure compared with a
  pure native run.
- The Ruby entrypoint has a measurable startup penalty beyond the engine cost.
- RuboCop must be installed and discoverable. The Ruby entrypoint supplies its
  gem executable path; direct native use can set `RUSTOCOP_RUBOCOP_PATH`.

## Current limitations

- Mixed execution requires an explicit `--only` list. Rustocop does not load
  Ruby code merely to discover custom cops enabled implicitly by configuration.
- Mixed autocorrection is rejected. Independent native and Ruby correction
  passes do not currently have a safe or RuboCop-compatible ordering contract.
- Mixed `--stdin` inspection is rejected because both processes would need the
  same input stream and merged filename semantics.
- A custom cop whose name collides with an advertised native cop remains
  native; there is currently no explicit override flag.

## Alternatives considered

### Run all selected cops through RuboCop when a custom cop is present

This is operationally simple but discards native work. It measured 478.32 ms,
compared with 457.36 ms for direct mixed execution, and the gap should grow when
the built-in set is more expensive.

### Coordinate mixed execution in the Ruby entrypoint

The first prototype worked but paid Ruby startup before launching another Ruby
process for RuboCop. It measured approximately 533 ms and was slower than pure
RuboCop, so coordination was moved into the native executable.

### Ignore custom cops

This preserves native performance but produces incomplete local results without
making that omission sufficiently visible. It was rejected.

### Reimplement every custom cop in Rust

This provides the best performance but is not a general compatibility strategy
for project-local code. Frequently used custom cops can still be ported
individually when their latency matters.

## Verification and reproduction

The benchmark aborts unless mixed and pure-RuboCop normalized JSON are equal and
the custom cop raises exactly 500 offenses. Reproduce it with:

```sh
bundle exec rake build:native
bundle exec ruby script/benchmark_mixed_custom_cop.rb
```

Detailed methodology is also recorded in
[`benchmark/mixed-custom-cops.md`](../benchmark/mixed-custom-cops.md).
