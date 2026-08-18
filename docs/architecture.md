# Architecture

Rustocop has one inspection pipeline and two kinds of cop implementation. The
pipeline owns I/O, configuration, reporting, and correction ordering. Cops only
inspect the source representation they are given and emit offenses.

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
        v
line_cops::before_prism  (safe textual corrections)
        |
        v
one Prism parse per file
        |
        +--> registered AST cops share Context and tree
        |
        v
line_cops::after_prism   (read-only compatibility cops)
        |
        v
sort offenses / apply non-overlapping corrections / format report
```

## Module boundaries

- `main.rs` is the application shell: targets and output compatibility.
- `cop_registry.rs` is the native support inventory. `cop_selection.rs`
  resolves `--only` against it once into an immutable enabled-cop set.
  `inspection.rs` builds the run-level plan, owns file I/O, and chooses between
  the full compatibility pipeline and the allocation-light Prism-only path.
- `diagnostic.rs` translates Prism byte ranges into RuboCop-compatible
  locations using a per-source newline index. `source_lines.rs` owns the
  mutable line representation used by textual cops.
- `prism_engine.rs` owns the shared parse, traversal, and registry.
  `prism_engine/matchers.rs` owns side-effect-free Prism questions reused by
  multiple cop families. `prism_engine/diagnostic.rs` owns findings, byte-range
  normalization, and correction ordering. Department and behavior-focused
  modules contain AST cops; source-oriented Style cops stay in `style_source.rs`
  rather than adding scanner state to the coordinator.
- `line_cops/mod.rs` is the only entry point to textual cops.
  `line_cops/{layout,lint,style,bundler,metrics,extensions}.rs` own their
  departments; `helpers.rs` contains shared, side-effect-free text helpers.
- Specs and the extracted upstream corpus are the compatibility contract. A cop
  is not considered verified because it merely recognizes representative text.

Prism is parsed exactly once for each inspected source. Adding AST cops should
register another visitor against that shared tree, never parse the file again.
The stateless registry is built once per command and shared immutably by every
file worker.
See [Adding a Prism cop](adding-a-prism-cop.md) for the authoring API and a
minimal implementation template.

`--parallel` distributes complete files across scoped worker threads. A file is
never split between workers, and cops within that file continue to share one
Prism tree. Results are restored to discovery order before formatting, so
parallel execution must produce byte-for-byte identical output to sequential
execution. `--jobs N` sets an explicit worker count. Autocorrection falls back
to sequential execution if multiple requested paths resolve to the same file,
preventing concurrent writes to one correction target.

## Enforced limits

`rake quality:architecture` enforces these ceilings:

- 600 lines for a Rust module, with explicit transitional ceilings of 800 for
  `main.rs` and 600 for the Prism coordinator.
- 200 lines per function, cognitive complexity 30, and at most eight arguments.
- no unsafe Rust.

These are hard stop-lines, not targets. New modules should normally stay below
400 lines, functions below 60 lines, cognitive complexity below 15, and accept
at most five arguments. Split by responsibility before reaching a ceiling.

The architecture task is a prerequisite of the default spec task, so CI and
local `bundle exec rake` runs reject structural regressions.

Measured optimization opportunities and the required invariants for addressing
them are tracked in [Known performance bottlenecks](bottlenecks.md).
