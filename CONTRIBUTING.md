# Rules of engagement

## Preserve the compatibility contract

1. Start from RuboCop's own cop specs. Add the relevant upstream cases to the
   compatibility corpus before claiming support.
2. “Verified” means the upstream cases pass with matching offense locations,
   messages where asserted, configuration behavior, and autocorrection.
   Pattern-based or partial behavior must remain documented as heuristic.
3. Update `docs/cop-support.md` whenever a cop changes status. Never inflate the
   supported count to describe a registered name without compatible behavior.

## Put code in the right layer

1. Parse each file with Prism once. All AST cops share the existing context and
   traversal; a cop must not open or parse the source independently.
2. Prefer an AST cop when correctness depends on Ruby syntax, scope, receiver,
   or byte locations. Keep a textual cop only when line semantics are the actual
   RuboCop contract or while it is explicitly tracked as compatibility debt.
3. Start new cops with `script/new_cop.rb`; it creates a focused module or
   appends to a cohesive existing module with `--family`. Cross-department
   utilities must be pure helpers; they must not know about CLI flags,
   filesystem discovery, or output formatting.
4. The inspection coordinator decides ordering. Individual cops emit findings
   and corrections; they do not write files or print output.
5. Corrections use Prism byte offsets and must be non-overlapping. Add an
   autocorrection regression case for every correctable cop.

Start with [Building a cop](docs/building-a-cop.md) for the complete workflow.
The [Prism cop DSL reference](docs/adding-a-prism-cop.md) documents the shared
callback, matcher, context, and diagnostic APIs. Extend those APIs only for a
recurring concept, not to hide logic that belongs to one cop.

## Add a cop

```sh
ruby script/new_cop.rb Style/Example call
# implement from RuboCop's spec and complete the generated fixture
ruby script/verify_cop.rb Style/Example
```

The supported callback kinds are `call`, `node`, `any_node`, and `source`.
Node callbacks accept `--node-cast as_if_node` (or another Prism cast). Use
`any_node` only when one cop genuinely handles several node kinds, and reserve
`source` for lexical or file-level rules; syntax-aware cops belong on Prism
nodes. The generated cop name is discovered from the runtime registry, so there
is no public cop list to edit. Use `--dry-run` to preview the generated source
and fixture templates.

## Keep complexity bounded

1. Run `bundle exec rake quality:architecture` before submitting a change.
2. Treat 400 module lines, 60 function lines, cognitive complexity 15, and five
   arguments as review triggers. The enforced ceilings in
   `docs/architecture.md` are emergency brakes, not allowances.
3. Split around concepts (department, traversal phase, registry, formatter), not
   arbitrary line ranges. Do not create a generic `utils` or `misc` module.
4. Do not raise a size or Clippy threshold to land a feature. A threshold change
   requires an architectural rationale and a follow-up removal plan.
5. Avoid broad lint suppressions. A local suppression must explain why the API
   shape is intrinsically clearer than the lint's recommendation.

## Validate changes

Run, at minimum:

```sh
cargo test --manifest-path crates/rustocop/Cargo.toml
bundle exec rake quality:architecture
bundle exec rspec
```

For a cop implementation, also run the extracted upstream contract for that cop.
For parser, traversal, correction ordering, or registry changes, run the full
upstream suite. Performance comparisons must use identical files, configuration,
Ruby version, warmup, and process model; report medians and disclose whether
RuboCop's Prism parser was enabled.

Use `--baseline spec/upstream/rubocop-1.87.0/status.yml` for the full diagnostic
run. The gate accepts improvements but rejects aggregate or Verified-cop
regressions. Regenerate `docs/remaining-cops.md` from that complete report; the
generator deliberately refuses focused or truncated reports.
