# Adding a Prism cop

Prism cops share one parsed tree, one traversal, and one diagnostic context per
file. A cop should recognize a syntax pattern and report its intent; it should
not parse source, manage correction flags, mutate files, or construct findings.

## Minimal cop

For a call-only cop, declare its registration and handler with the shared DSL.
The handler receives a reporter already scoped to the cop, so it cannot
accidentally attribute a diagnostic to another rule:

```rust
declare_cops!(Example);
define_call_cop!(Example => "Style/Example" => example);

fn example(node: &CallNode<'_>, reporter: &mut Reporter<'_>) {
    if !match_call(node).named(b"old_name").without_arguments().matches() {
        return;
    }
    let Some(selector) = node.message_loc() else { return };

    reporter.replace("Prefer `new_name`.", &selector, &selector, "new_name");
}
```

For several source-wide rules, one declaration handles their registry, marker
types, names, and reporter scoping:

```rust
declare_source_cops! {
    FirstRule => "Style/FirstRule" => first_rule,
    SecondRule => "Lint/SecondRule" => second_rule,
}

fn first_rule(source: &str, reporter: &mut Reporter<'_>) {
    // Inspect the already-loaded source exactly once.
}
```

Implement `Cop` directly when a rule needs arbitrary `on_node` traversal or
ancestor access. Then use `declare_cops!(FirstCop, SecondCop)` for its registry.
Every cop name must also be added to `PRISM_COPS`; a registry test fails if the
public list and implementations drift apart.

## Reporting and correction intents

`Reporter` accepts a Prism `Location`, a borrowed `&Location`, a Rust byte
range, or an `(start, end)` byte-offset pair. Prefer locations while working
with AST nodes and raw offsets only when detecting source punctuation.

- `report` records an offense with no correction.
- `replace` replaces one byte range with text.
- `remove` is a replacement with empty text.
- `insert` inserts text at a byte offset.

The context derives `correctable` and `corrected`, sorts findings, ignores
overlapping or out-of-bounds edits, and applies accepted edits in reverse order.
This keeps every cop out of correction bookkeeping.

## Reusable matchers

`cops/prism/matchers.rs` exposes focused helpers for recurring patterns:
`only_argument`, `receiver_call`, root constants, keyword presence,
`node_source`, source equality, location equality, and common literal
classification. Prefer `only_argument` when a rule requires exactly one
argument; `first_argument` deliberately accepts calls with additional values.

Call-based cops can express their structural requirements with the
allocation-free call matcher DSL:

```rust
if !match_call(node)
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

Both `Context` and a scoped `Reporter` expose the configured target Ruby
version for syntax and API behavior that changed between Ruby releases. Keep
version checks in the cop that owns the behavior.

`reporter.config_value("EnforcedStyle")` reads an option from the current cop's
configuration. Use `related_config_value` only when RuboCop explicitly defines
one cop's behavior in terms of another cop's configuration. The configuration
view is read-only and shared across every file in the run.

Likely next shared capabilities, as parity work requires them, are:

1. path context for file-sensitive cops;
2. typed configuration accessors once repeated option families justify them;
3. configuration-aware matcher predicates once repeated cases justify them.

Configuration and path context unlock more upstream cases than additional
syntax shortcuts, so they should come before a large matcher DSL.

## Validation

Start with the upstream RuboCop spec for the cop. Test diagnostic byte ranges,
messages where asserted, configuration branches, and every autocorrection. Run
the project gates in `CONTRIBUTING.md`, plus the extracted upstream contract for
the cop. Changes to traversal, diagnostics, correction ordering, or the registry
require the full upstream suite.
