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

Rustocop supports a mixed execution mode in either of these cases:

1. At least one `--require` or `--plugin` option is present, the run has an
   explicit `--only` list, and at least one selected name is not in Rustocop's
   advertised cop inventory.
2. RuboCop's resolved project configuration enables cops outside the base
   RuboCop package and the user passes `--included-non-native-cops`.

Names advertised by `rustocop --show-cops` remain native. Unknown selections
are passed to RuboCop together with the loader options, configuration path, and
file targets.

For implicit project configuration, the base RuboCop registry is captured
before project plugins and custom requires are loaded. The enabled base cops run
natively. Enabled names added by plugins or project requires are ignored with a
warning unless the user opts in, at which point RuboCop runs those names and its
JSON report is merged with the native report.

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
| Rustocop native binary, 20 built-in cops | 17.54 ms | 18.50 ms | 1.0× |
| Rustocop Ruby entrypoint, 20 built-in cops | 122.01 ms | 125.10 ms | 7.0× |
| Mixed native binary, 20 native + 1 custom cop | 545.39 ms | 550.42 ms | 31.1× |
| Mixed Ruby entrypoint, 20 native + 1 custom cop | 638.75 ms | 657.74 ms | 36.4× |
| RuboCop, custom cop only | 527.37 ms | 534.53 ms | 30.1× |
| RuboCop, all 20 built-ins + custom cop | 555.00 ms | 563.15 ms | 31.6× |

The direct mixed run was 1.7% faster than pure RuboCop and produced
identical normalized JSON. One Ruby custom cop still made this tiny-corpus run
31.1 times slower than pure native Rustocop because RuboCop must
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

- Resolving an effective project configuration loads RuboCop before native
  inspection and can add several hundred milliseconds to short runs. Explicit
  `--only` runs without a project or user configuration bypass this work.
- A single Ruby custom cop forfeits most of Rustocop's startup advantage.
- Every file is read and parsed independently by both engines.
- Concurrent processes increase CPU and peak-memory pressure compared with a
  pure native run.
- The Ruby entrypoint has a measurable startup penalty beyond the engine cost.
- RuboCop must be installed and discoverable. The Ruby entrypoint supplies its
  gem executable path; direct native use can set `RUSTOCOP_RUBOCOP_PATH`.

## Current limitations

- The automatic effective-config path is provided by the Ruby entrypoint.
  Direct execution of `libexec/rustocop-native` does not load RuboCop's Ruby
  configuration subsystem.
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
