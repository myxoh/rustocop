# Prism cop DSL reference

Prism cops share one parsed tree, one traversal, and one diagnostic context per
file. A cop should recognize a syntax pattern and report its intent; it should
not parse source, manage correction flags, mutate files, or construct findings.

For the complete newcomer workflow—from reading the upstream RuboCop spec to
promoting support—start with [Building a cop](building-a-cop.md).

## Callback DSL

Every concise cop is declared once inside `define_cops!`:

| Form | Handler signature | Use it for |
| --- | --- | --- |
| `call(handler)` | `fn(&CallNode, &mut CopContext)` | One method-call shape |
| `node(as_if_node, handler)` | `fn(&IfNode, &mut CopContext)` | One typed Prism node |
| `any_node(handler)` | `fn(&Node, &mut CopContext)` | Several intentional node kinds |
| `source(handler)` | `fn(&mut CopContext)` | Lexical or file-level rules |

One module can register related cops together:

```rust
define_cops! {
    First => "Style/First" => call(check_first),
    Second => "Lint/Second" => node(as_if_node, check_second),
}
```

The generated marker types are stateless and `Sync`. Per-file state belongs in
the callback/context, never in a cop instance.

## Start a cop

Generate the module, registry wiring, and focused fixture shell:

```sh
ruby script/new_cop.rb Style/Example call
ruby script/new_cop.rb Lint/Example node --node-cast as_if_node
ruby script/new_cop.rb Style/SeveralShapes any_node
ruby script/new_cop.rb Layout/Example source
ruby script/new_cop.rb Bundler/Example call --fixture-path /project/Gemfile --autocorrect
```

Use `--dry-run` to inspect the generated source, fixture templates, and test
registration. The generator wires the fixture into the Rust test suite
automatically. Use `--fixture-path` for path-sensitive cops and `--autocorrect`
when the fixture should compare `corrected.rb`. The implementation and fixture
contents must still come from RuboCop's upstream spec.

## Minimal cop

For a call-only cop, declare its registration and handler with the shared DSL.
The handler receives a reporter already scoped to the cop, so it cannot
accidentally attribute a diagnostic to another rule:

```rust
define_cops! {
    Example => "Style/Example" => call(example),
}

fn example(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node).named(b"old_name").without_arguments().matches() {
        return;
    }
    context.replace_selector(node, "Prefer `new_name`.", "new_name");
}
```

The same declaration supports typed Prism nodes and source-wide callbacks:

```rust
define_cops! {
    Conditional => "Style/Conditional" => node(as_if_node, conditional),
    FileHeader => "Layout/FileHeader" => source(file_header),
}

fn file_header(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        // Inspect the already-loaded source exactly once.
    }
}
```

Use `any_node(handler)` when one cop intentionally handles several node types.
Reserve `source(handler)` for genuinely lexical or file-level rules. If the
answer depends on whether text is Ruby code, a string, a comment, a regexp, a
heredoc, or a particular expression, use `call`, typed `node`, or `any_node`.
Implement `Cop` directly only for genuinely unusual traversal behavior.

Cop names are discovered from the actual registry. Do not edit a separate
Prism or supported-cop inventory. `--show-cops` and the support-document
generator read the runtime registry.

## Cop context

Every concise callback receives a cop-scoped `CopContext`. It exposes:

- `source()` and the safe `source_file()` geometry helpers;
- `path()`, `parent()`, `ancestors()`, `nearest_call()`, and `inside_method()`;
- target Ruby version and typed cop configuration;
- reporting and higher-level correction intents.

`source_file()` provides safe slicing, node/location text, offset-aware lines,
physical line ranges, and surrounding-whitespace ranges. Prefer it over raw
source indexing when a cop works with byte geometry.

## Reporting and correction intents

`CopContext` accepts a Prism `Location`, a borrowed `&Location`, a Rust byte
range, or an `(start, end)` byte-offset pair. Prefer locations while working
with AST nodes and raw offsets only when detecting source punctuation.

- `report` records an offense with no correction.
- `report_call`, `report_selector`, and `report_node` select common AST ranges.
- `replace` replaces one byte range with text.
- `remove` is a replacement with empty text.
- `insert` inserts text at a byte offset.
- `replace_selector` replaces a call's method name.
- `replace_call`, `remove_call`, `replace_node`, `remove_node`, `insert_before`,
  and `insert_after` express common AST correction intents without repeating
  offsets.
- `remove_list_element` owns adjacent separators, while `wrap_node` and
  `remove_statement` handle two other common structural corrections.

The context derives `correctable` and `corrected`, sorts findings, ignores
overlapping or out-of-bounds edits, and applies accepted edits in reverse order.
This keeps every cop out of correction bookkeeping.

## Reusable matchers

`cops/prism/matchers.rs` exposes focused helpers for recurring patterns:
`only_argument`, `receiver_call`, root constants, keyword presence,
`node_source`, source equality, location equality, and common literal
classification. Prefer `only_argument` when a rule requires exactly one
argument; `first_argument` deliberately accepts calls with additional values.
The matcher can also require argument presence, inspect the first or only
argument with a predicate, and match the receiver when it is another named
call.

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

The chainable matcher vocabulary is:

| Question | Methods |
| --- | --- |
| Method name | `named`, `named_any` |
| Receiver | `without_receiver`, `with_receiver`, `on_root_constant`, `on_constant_read`, `on_implicit_or_root_constant`, `on_receiver_call_named` |
| Arguments | `without_arguments`, `with_arguments`, `with_argument_count`, `with_only_argument_matching`, `with_first_argument_matching`, `with_keyword` |
| Call shape | `with_block`, `without_block`, `with_operator` |

End every chain with `matches()`. These checks compose with logical AND; use a
small predicate closure for an argument's node shape and keep configuration or
business rules outside the matcher.

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

Configuration is parsed once and shared across every file in the run. Prefer
typed access over string comparisons:

```rust
context.config_bool("CountComments", false);
context.config_usize("Max", 10);
context.config_values("AllowedMethods");
context.config_map("PreferredMethods");
context.policy().enforced_style("compact");
context.policy().allows_method(call_name(node));
context.policy().allows_receiver(receiver_name);
context.policy().included_path(context.path());
context.policy().excluded_path(context.path());
```

The shared policy also handles allowed patterns and receivers plus cop and
`AllCops` path inclusion/exclusion. Use `related_config_value` only when
RuboCop explicitly defines one cop in terms of another cop's configuration.

The matcher library includes call receiver/name/argument/keyword/block shapes,
static string and symbol extraction, constant paths, source equality, and
literal classification. Keep the DSL structural: the reason an offense exists
should remain visible in the cop function.

## Validation

Start with the upstream RuboCop spec for the cop. Test diagnostic byte ranges,
messages where asserted, configuration branches, and every autocorrection. Run
the focused contract with:

```sh
ruby script/verify_cop.rb Style/Example
```

It builds the current native binary, checks every captured diagnostic and
correction case for that cop, writes a categorized JSON report, and regenerates
the support matrix from the runtime registry. It does not promote a cop to
Verified until the full upstream comparison has also passed. Changes to
traversal, diagnostics, correction ordering, or the registry still require the
full upstream suite described in `CONTRIBUTING.md`.

The verifier rejects unknown cops or an empty captured contract instead of
treating zero executed cases as success. On failure it prints the first
expected/actual mismatch before pointing to the complete JSON report.
