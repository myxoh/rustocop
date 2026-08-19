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

The synthetic benchmark uses the committed 500-file compatibility corpus,
containing 9,090 bytes of Ruby. It enables 20 built-in cops plus one custom cop
that raises one offense in each file. RuboCop 1.87.0 uses Prism, with caching
and server mode disabled. Results are medians of seven runs after two warmups.

| Variant | Median | p95 |
| --- | ---: | ---: |
| Rustocop native binary, 20 built-in cops | **9.07 ms** | 9.19 ms |
| Rustocop Ruby entrypoint, 20 built-in cops | 85.01 ms | 87.38 ms |
| Mixed native binary, 20 native + 1 custom cop | **456.12 ms** | 463.33 ms |
| Mixed Ruby entrypoint, 20 native + 1 custom cop | 531.26 ms | 536.79 ms |
| RuboCop, custom cop only | 446.74 ms | 454.94 ms |
| RuboCop, all 20 built-ins + custom cop | 478.47 ms | 485.41 ms |

The direct mixed run produced JSON identical to pure RuboCop and was 4.7%
faster. It was only 2.1% slower than asking RuboCop to run the custom cop alone,
showing that concurrent native inspection adds little beyond the unavoidable
Ruby work.

One Ruby custom cop nevertheless made the run approximately 50 times slower
than pure native Rustocop. RuboCop startup and its second Prism parse are the
performance floor; the native tree cannot be shared across the process
boundary.

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

This is operationally simple but discards native work. It measured 478.47 ms,
compared with 456.12 ms for direct mixed execution, and the gap should grow when
the built-in set is more expensive.

### Coordinate mixed execution in the Ruby entrypoint

The first prototype worked but paid Ruby startup before launching another Ruby
process for RuboCop. It measured approximately 541 ms and was slower than pure
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
