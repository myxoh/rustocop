# Adding a Prism cop

Prism cops share one parsed tree, one traversal, and one diagnostic context per
file. A cop should recognize a syntax pattern and report its intent; it should
not parse source, manage correction flags, mutate files, or construct findings.

## Minimal cop

Add the cop to the department module's `cops()` list and implement `Cop`. Use
`on_call` when calls are the only relevant node type, `on_node` for other AST
nodes, and `on_source` for a rule that must inspect the complete file exactly
once. Source-wide rules still share the existing parse and correction context;
they must not read the file or parse it again.

```rust
struct Example;

impl Cop for Example {
    fn name(&self) -> &'static str {
        "Style/Example"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if call_name(node) != b"old_name" {
            return;
        }
        let Some(selector) = node.message_loc() else {
            return;
        };

        context.replace(
            self.name(),
            "Prefer `new_name`.",
            &selector,
            &selector,
            "new_name",
        );
    }
}
```

Then add the name to `PRISM_COPS`. A registry test fails if that public list and
the implementations drift apart.

## Reporting and correction intents

`Context` accepts a Prism `Location`, a borrowed `&Location`, a Rust byte range,
or an `(start, end)` byte-offset pair. Prefer locations while working with AST
nodes and raw offsets only when detecting source punctuation.

- `report` records an offense with no correction.
- `replace` replaces one byte range with text.
- `remove` is a replacement with empty text.
- `insert` inserts text at a byte offset.

The context derives `correctable` and `corrected`, sorts findings, ignores
overlapping or out-of-bounds edits, and applies accepted edits in reverse order.
This keeps every cop out of correction bookkeeping.

## Reusable matchers

`cops/prism/matchers.rs` exposes focused helpers for recurring patterns: first
arguments, root constants, keyword presence, source slices, and common literal
classification. The coordinator imports this matcher surface for all cop
modules. Reuse it before spelling out an equivalent tree walk in a cop.

Call-based cops can express their structural requirements with the
allocation-free `CallMatcher` DSL:

```rust
if !CallMatcher::new(node)
    .named(b"load")
    .on_root_constant(b"Example")
    .with_argument_count(1)
    .matches()
{
    return;
}
```

Keep semantic conditions, source reconstruction, and diagnostic ranges in the
cop itself. The matcher is for method names, receiver shapes, and argument
counts—not a place to hide the reason a rule reports an offense.

Extract a new matcher when the same semantic question appears in multiple cops
and can be answered without configuration or side effects. Keep a helper in its
department when it represents one cop family. Do not create generic `utils` or
`misc` modules; names should describe the concept being shared.

The context also exposes the configured target Ruby version for syntax and API
behavior that changed between Ruby releases. Keep version checks in the cop that
owns the behavior.

Likely next shared capabilities, as parity work requires them, are:

1. a read-only cop configuration view on `Context`;
2. path context for file-sensitive cops;
3. a declarative registry once repeated registration metadata justifies it.

Configuration and path context unlock more upstream cases than additional
syntax shortcuts, so they should come before a large matcher DSL.

## Validation

Start with the upstream RuboCop spec for the cop. Test diagnostic byte ranges,
messages where asserted, configuration branches, and every autocorrection. Run
the project gates in `CONTRIBUTING.md`, plus the extracted upstream contract for
the cop. Changes to traversal, diagnostics, correction ordering, or the registry
require the full upstream suite.
