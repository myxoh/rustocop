# Architecture

Rustocop is organized into application, execution, and cop layers. Dependencies
flow inward: cops never know about file discovery, reporting, worker threads, or
the command-line interface.

```text
main.rs
  |
  v
app/  ----------------------> catalog.rs / config.rs / model.rs
  |                                      ^
  v                                      |
engine/ --------------------> cops/ -----+
  |                            |
  |                            +-- prism/  AST cops and one shared traversal
  |                            +-- text/   compatibility-oriented textual cops
  v
read/write files, sort diagnostics, format-independent inspection results
```

The only Rust modules allowed at `src/` are the small composition root and
shared leaf modules:

```text
src/
  main.rs          process entrypoint only
  catalog.rs       advertised cop inventory and defaults
  config.rs        run options, cop selection, Ruby version
  model.rs         source lines and formatter-independent offenses
  app/
    cli.rs         argument and RuboCop-config parsing
    targets.rs     target expansion, discovery, and stdin
    report.rs      JSON/simple formatting and exit status
  engine/
    mod.rs         run-level InspectionPlan and file inspection pipeline
    runner.rs      deterministic file worker pool
    diagnostic.rs  Prism finding-to-offense location conversion
    source.rs      mutable textual source representation
  cops/
    prism/         shared Prism registry, traversal, matchers, and AST cops
    text/          textual compatibility cops grouped by department
```

## Inspection pipeline

```text
CLI / cop selection / file discovery
        |
        v
run-level InspectionPlan
  (immutable enabled-cop set + shared Prism registry)
        |
        +--> optional fixed-size file worker pool
        |        (one complete file per worker)
        |
        +--> text::before_prism  (safe textual corrections, when enabled)
        |
        +--> one Prism parse per file
        |        (all enabled AST cops share Context and tree)
        |
        +--> text::after_prism   (read-only compatibility cops, when enabled)
        |
        v
sort offenses / apply non-overlapping corrections / format report
```

Prism-only runs bypass line splitting, cloning, and rejoining. A run containing
textual cops retains the compatibility pipeline and its correction ordering.

## Layer rules

- `app` may depend on every lower layer. It owns user-facing I/O, not lint
  behavior.
- `engine` may depend on cops and shared leaf modules. It owns ordering,
  concurrency, file writes, and the one-parse-per-file invariant.
- `cops` may depend only on `catalog`, `config`, `model`, and sibling cop
  modules. A cop cannot discover files, format reports, or invoke the engine.
- `catalog`, `config`, and `model` are leaf modules. They cannot depend on the
  application, engine, or cop implementations.
- Specs and the extracted upstream corpus are the compatibility contract. A cop
  is not verified merely because it recognizes representative text.

`script/check_architecture.rb` enforces the root layout, dependency direction,
module line ceilings, and the 50-line limit on `main.rs`. Clippy owns
function-level complexity and argument limits.

## Cop authoring and parallelism

Every inspected file is parsed exactly once. Adding an AST cop registers a
stateless visitor in `cops/prism`; it must never open or parse the source
independently. See [Adding a Prism cop](adding-a-prism-cop.md) for the authoring
API.

`--parallel` distributes complete files across scoped worker threads. Results
are restored to discovery order before formatting, so parallel and sequential
output must remain byte-identical. `--jobs N` sets an explicit worker count.
Autocorrection falls back to sequential execution when requested paths resolve
to the same file.

## Complexity limits

Rust modules have an enforced 600-line emergency ceiling; new modules should
normally remain below 400 lines. Functions should normally remain below 60
lines, cognitive complexity 15, and five arguments. The enforced Clippy limits
are 200 lines, cognitive complexity 30, and eight arguments. Do not raise a
limit to land a feature.

Measured optimization opportunities and their invariants are tracked in
[Known performance bottlenecks](bottlenecks.md).
