# Building a cop

This guide walks through the complete path from a RuboCop spec to a tested
Rustocop implementation. For the complete callback, matcher, context, and
correction API, keep the [Prism cop DSL reference](adding-a-prism-cop.md) open
alongside it.

The examples below mirror the existing `Style/Send` cop because its contract is
small: report the `send` selector when the call has arguments. Substitute the
real cop you are implementing; do not regenerate `Style/Send` in this checkout.

## 1. Read RuboCop's contract first

Find the vendored upstream spec:

```sh
rg --files spec/upstream/rubocop-1.87.0/spec/rubocop/cop \
  | rg '/send_spec\.rb$'
```

Read every context, not only the first offending example. Record:

- the syntax that produces an offense;
- clean examples that look similar;
- the exact offense range and message;
- configuration branches and target Ruby versions;
- whether correction is supported and the exact corrected source;
- whether behavior depends on the inspected path or surrounding scope.

Do not choose an implementation technique until those conditions are clear.
A source substring can look attractive while missing the distinction between
Ruby code, a string, a comment, a regexp, and a heredoc.

## 2. Choose the narrowest callback

Use this order of preference:

| Contract shape | Callback | Handler input |
| --- | --- | --- |
| One method call shape | `call` | `&CallNode` |
| One Prism node kind | `node` | A typed Prism node |
| A small set of node kinds | `any_node` | `&Node` |
| Truly lexical or file-level behavior | `source` | `&mut CopContext` |
| A parser diagnostic | `parse_error` | `&Diagnostic` |

Most new cops should use `call` or typed `node`. Use `any_node` only when the
same rule genuinely handles several AST shapes. Use `source` for contracts such
as magic comments or initial file indentation—not as a shortcut around Prism.

For `Style/Send`, the answer depends on whether `send` is an actual call, so
`call` is the correct choice.

## 3. Generate the skeleton

Preview everything the generator would create:

```sh
ruby script/new_cop.rb Style/Example call --dry-run
```

All 606 RuboCop 1.87 built-ins currently have a native implementation. Use the
generator for a genuinely new cop; when improving parity for an existing cop,
start from its current family module and fixture instead:

```sh
ruby script/new_cop.rb Department/CopName call
```

Useful variants:

```sh
# A typed Prism node callback
ruby script/new_cop.rb Style/Example node --node-cast as_if_node

# Several intentional node shapes
ruby script/new_cop.rb Lint/Example any_node

# A path-sensitive, correctable cop
ruby script/new_cop.rb Bundler/Example call \
  --fixture-path /project/Gemfile \
  --autocorrect

# A genuinely file-level rule
ruby script/new_cop.rb Layout/Example source

# Add the cop to an existing capability-oriented family
ruby script/new_cop.rb Style/Example call --family style_calls
```

Prefer `--family` when a cohesive module already owns the same kind of rule.
Create a focused module when the cop introduces a distinct capability. The
generator always creates cop-specific fixtures regardless of module choice.

Without `--family`, the generator creates and wires:

- a focused module under `crates/rustocop/src/cops/prism/`;
- one `cop_modules!` entry in `prism/mod.rs`, which declares and registers it;
- `input.rb` and `offenses.tsv` fixtures;
- `corrected.rb` when `--autocorrect` is used;
- a fixture test registration in `engine/fixture_tests.rs`.

With `--family`, it appends the declaration and callback to that module and
leaves the composition root unchanged.

Cop names are discovered from the Prism registry. Do not add a second public
inventory entry.

## 4. Inspect the Prism shape

When the node shape is unclear, ask Prism instead of guessing. For example:

```sh
printf 'Object.send(:work)\n' \
  | bundle exec ruby -rprism -e 'pp Prism.parse($stdin.read).value'
```

Compare an offending and clean form. Pay attention to:

- node kind;
- receiver shape;
- method name;
- argument nodes;
- call operator (`.`, `&.`, or `::`);
- block presence;
- the location that RuboCop highlights.

Use typed Prism locations for offenses and corrections whenever possible. Raw
byte offsets are appropriate for punctuation that Prism does not expose, not
for reconstructing an AST with string searches.

## 5. Implement the structural match

A concise call cop looks like this:

```rust
use super::*;

define_cops! {
    Send => "Style/Send" => call(send),
}

fn send(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !match_call(node).named(b"send").with_arguments().matches() {
        return;
    }

    context.report_selector(
        node,
        "Prefer `Object#__send__` or `Object#public_send` to `send`.",
    );
}
```

The declaration owns registration and dispatch. The matcher describes the
structural conditions. The handler keeps the semantic decision and diagnostic
visible.

Avoid putting every condition into a new helper. Extract a shared matcher only
when the same structural question appears in multiple cops.

## 6. Fill the generated fixture

Use the local fixture for fast, readable regression coverage. Include at least
one offense and nearby clean forms, especially adversarial ones:

```ruby
# input.rb
Object.send(:work)
Object.public_send(:work)
"send(:work)"
# send(:work)
```

`offenses.tsv` uses character-based RuboCop locations even though cop handlers
work with Prism byte ranges:

```text
cop	line	column	last_line	last_column	correctable	corrected	message
Style/Send	1	8	1	11	false	false	Prefer `Object#__send__` or `Object#public_send` to `send`.
```

Keep these fixture cases deliberately small. The upstream contract provides
breadth; the local fixture should explain the implementation's riskiest edge.

For a correctable cop generated with `--autocorrect`, edit `corrected.rb` to the
exact expected final source. The fixture runner checks offenses and corrected
content in one test.

Run the Rust tests while iterating:

```sh
cargo test --manifest-path crates/rustocop/Cargo.toml
```

## 7. Add a correction only after detection matches

Start diagnostic-only. Once ranges and clean cases match RuboCop, use the
smallest correction intent that expresses the edit:

```rust
context.replace_selector(node, "Prefer `perform`.", "perform");
```

Other common intents:

```rust
context.replace_node(node, message, replacement);
context.remove_node(node, message);
context.wrap_node(node, message, "(", ")");
context.remove_list_element(node, previous, next, message);
context.insert_before(node, message, prefix);
context.insert_after(node, message, suffix);
```

Do not mutate files, set `correctable` flags, or apply edits yourself. The
diagnostic context owns that bookkeeping. Corrections must use non-overlapping
byte ranges; if a RuboCop correction needs coordinated edits, check the current
correction limitations before approximating it with a broad replacement.

## 8. Implement configuration explicitly

Configuration is scoped to the current cop:

```rust
let max = context.config_usize("Max", 10);
let count_comments = context.config_bool("CountComments", false);
let allowed = context.config_values("AllowedMethods");
let style = context.policy().enforced_style("compact");

if context.policy().allows_method(call_name(node)) {
    return;
}
```

Also available:

- `config_value` for an individual scalar;
- `config_map` for name-to-name mappings;
- allowed method/receiver patterns through `CopPolicy`;
- `target_ruby_version()`;
- `path()`, `ancestors()`, `parent()`, and `inside_method()`.

Only implement configuration RuboCop actually defines. If the shared config
parser cannot represent a required shape, improve that subsystem and test it
rather than parsing YAML inside a cop.

## 9. Verify against every captured upstream case

Run the focused verifier:

```sh
ruby script/verify_cop.rb Department/CopName
```

It:

1. builds the debug native binary;
2. runs every captured diagnostic and correction case for that cop;
3. rejects an unknown cop or empty contract;
4. prints the first mismatch on failure;
5. writes the full report under `tmp/`.

A focused pass is necessary but not sufficient for promotion. Registry,
traversal, matching, and corrections can affect other cops.

## 10. Run the project gates

At minimum:

```sh
cargo test --manifest-path crates/rustocop/Cargo.toml
cargo clippy --manifest-path crates/rustocop/Cargo.toml --all-targets -- -D warnings
bundle exec rake quality:architecture
bundle exec rspec
git diff --check
```

For shared infrastructure or registry changes, also run the full captured
comparison:

```sh
RUSTOCOP_NATIVE_PATH=crates/rustocop/target/debug/rustocop \
  ruby script/compare_upstream_cop_specs.rb \
  --baseline spec/upstream/rubocop-1.87.0/status.yml \
  --report tmp/full-compatibility.json
```

The baseline option allows improvements while rejecting fewer passing cases,
fewer passing cops, a changed corpus, or any regression in a Verified cop.
After a full run, regenerate the implementation queue with:

```sh
RUSTOCOP_NATIVE_PATH=crates/rustocop/target/debug/rustocop \
  bundle exec ruby script/generate_remaining_cop_plan.rb \
  tmp/full-compatibility.json
```

This updates `docs/remaining-cops.md` and its machine-readable YAML companion.

For layout cops, use `SourceFile`'s `line_start`, `line_end`, `line`,
`line_range`, `full_line_range`, `indentation`, `indentation_text`, `same_line`,
and character-aware `column` helpers. Use `SourceFile::rewrite` with
`SourceEdit`s when constructing a correction from several edits inside one
container. Keep Prism byte offsets for diagnostic and correction ranges; use
`column` only when a rule needs display geometry.

Before adding a local body, modifier, argument, source-geometry, or call-shape
helper, check `framework/node_helpers.rs`, `framework/matchers.rs`, and nearby
family modules.

## 11. Promote support deliberately

Only mark a cop Verified when:

- every captured diagnostic case passes;
- every captured correction assertion passes;
- the local fixture covers the implementation's risky edge;
- the full comparison shows no regression;
- normal Rust and Ruby project gates pass.

Then add the cop to `fully_compatible_cops` and update the diagnostic totals in
`spec/upstream/rubocop-1.87.0/status.yml`, followed by:

```sh
RUSTOCOP_NATIVE_PATH=crates/rustocop/target/debug/rustocop \
  bundle exec ruby script/generate_cop_support.rb
```

If the implementation is useful but does not meet that contract, leave it
Heuristic. Passing a few representative examples is not verification.

## Final checklist

- [ ] Read every upstream spec context.
- [ ] Chose `call` or typed `node` unless the contract requires otherwise.
- [ ] Used Prism locations instead of reconstructing syntax from text.
- [ ] Added offending, clean, adversarial, and correction fixtures as relevant.
- [ ] Implemented configuration and Ruby-version branches from the contract.
- [ ] Ran the focused upstream verifier with corrections.
- [ ] Ran Rust, Clippy, architecture, Ruby, and full-regression gates.
- [ ] Updated status and regenerated support documentation only after passing.
