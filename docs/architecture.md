# Architecture

Rustocop is organized into application, execution, and cop layers. Dependencies
flow inward: cops never know about file discovery, reporting, worker threads, or
the command-line interface.

```text
main.rs
  |
  v
app/  ----------------------> config.rs / model.rs
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
  config.rs        run options, cop selection, Ruby version
  config/
    selection.rs   cop selection and Ruby-version value objects
  model.rs         source lines and formatter-independent offenses
  app/
    cli.rs         argument and RuboCop-config parsing
    targets.rs     target expansion, discovery, and stdin
    report.rs      JSON/simple formatting and exit status
    mixed.rs       RuboCop delegation and merged reporting for custom Ruby cops
  engine/
    mod.rs         run-level InspectionPlan and file inspection pipeline
    runner.rs      deterministic file worker pool
    diagnostic.rs  Prism finding-to-offense location conversion
    source.rs      mutable textual source representation
  cops/
    prism/
      mod.rs       composition root and cop-family registry
      framework/   CopContext, DSL, matchers, diagnostics, and source geometry
      runtime/     shared traversal, enabled-cop registry, and dispatch
      tests/       cross-family Prism integration tests
      *.rs         cohesive cop-family implementations
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
        |        (source-wide hooks run once; AST cops share Context and tree)
        |
        +--> text::after_prism   (read-only compatibility cops, when enabled)
        |
        v
sort offenses / apply non-overlapping corrections / format report
```

Prism-only runs bypass line splitting, cloning, and rejoining. A run containing
textual cops retains the compatibility pipeline and its correction ordering.
When `--require` or `--plugin` selects unknown cops alongside native cops,
`app/mixed.rs` runs the native inspection and one RuboCop subprocess
concurrently, then merges their formatter-independent offenses. This boundary
keeps Ruby plugin loading and JSON translation out of the engine and cop layers.
Mixed mode currently rejects stdin and autocorrection because merging two
independent correction streams would not be deterministic.

## Correction transactions

Each correctable finding owns one correction transaction containing one or
more edits. Range validation and conflict resolution happen before edits are
applied. A transaction is accepted atomically, and only then is its finding
marked corrected; rejected or partially conflicting transactions remain
correctable but uncorrected. This keeps diagnostics truthful and lets cops use
coordinated edits without replacing an unnecessarily large source region.

## Layer rules

- `app` may depend on every lower layer. It owns user-facing I/O, not lint
  behavior.
- `engine` may depend on cops and shared leaf modules. It owns ordering,
  concurrency, file writes, and the one-parse-per-file invariant.
- `cops` may depend only on `config`, `model`, and sibling cop
  modules. A cop cannot discover files, format reports, or invoke the engine.
- `config` and `model` are leaf modules. They cannot depend on the
  application, engine, or cop implementations.
- Differential fixtures and complete project signatures are the compatibility
  contract. A cop is not compatible merely because it recognizes representative
  text or passes the captured upstream examples.

`script/check_architecture.rb` enforces the root layout, dependency direction,
module line ceilings, and the 50-line limit on `main.rs`. Clippy owns
function-level complexity and argument limits.

## Cop authoring and parallelism

Every inspected file is parsed exactly once. Adding an AST cop registers a
stateless visitor in `cops/prism`; it must never open or parse the source
independently. See [Adding a Prism cop](adding-a-prism-cop.md) for the authoring
API. The `cop_modules!` composition list declares and registers each cop family
in one place. Public cop names come from implementations rather than a parallel
catalog, and generated scaffolding performs the composition-root wiring.

The physical Prism layout deliberately does not add another Rust module layer:
`mod.rs` uses explicit paths for `framework/` and `runtime/`, so existing cop
families keep the short `super::*` authoring surface. New reusable authoring
APIs belong in `framework/`; parse/traversal/dispatch code belongs in
`runtime/`; actual lint behavior stays in a named cop-family file.

`--parallel` distributes complete files across scoped worker threads. Results
are restored to discovery order before formatting, so parallel and sequential
output must remain byte-identical. `--jobs N` sets an explicit worker count.
Autocorrection falls back to sequential execution when requested paths resolve
to the same file.

## Complexity limits

Rust modules have an enforced 350-line ceiling, cop modules may declare at most
16 cops, and the process entrypoint has a 50-line ceiling. There are no
exceptions for cop-family or composition-root modules. Functions should
normally remain below 60 lines, cognitive complexity 15, and five arguments.
The enforced Clippy limits are 120 lines, cognitive complexity 25, and seven
arguments. Do not raise a limit to land a feature.

Measured optimization opportunities and their invariants are tracked in
[Known performance bottlenecks](bottlenecks.md).
