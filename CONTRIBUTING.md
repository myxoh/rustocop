# Rules of engagement

## Preserve the compatibility contract

1. Start from RuboCop's own cop specs and captured expectations.
2. Add offending, clean, adversarial, configuration, and correction fixtures;
   compare complete RuboCop diagnostics and corrected source.
3. Turn every real-project discrepancy into a minimized, provenance-backed
   fixture so it cannot recur silently.
4. Run the cop across all 50 pinned projects before making a compatibility
   claim. Representative examples and aggregate offense counts are insufficient.

Use evidence labels precisely:

- **registered** means only that the native registry advertises the cop;
- **fixture-validated** means the RuboCop-derived fixture corpus passes,
  including corrections where applicable;
- **project-exact** means complete diagnostic signatures match on all 50 pinned
  projects for one source-identified Rust binary.

Never turn a focused audit into a repository-wide claim. If the full 606-cop
matrix is older than `HEAD`, report its commit and reconcile only cops that have
been re-audited at the current Rust source. Call the result a reconciled estimate,
not a fresh full-matrix measurement. The current distinction and counts are
documented in [real-project output parity](docs/real-project-parity.md).

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
6. Keep cop-family implementations directly under `cops/prism`. Reusable
   authoring APIs belong in `cops/prism/framework`, while traversal, registry,
   and dispatch code belongs in `cops/prism/runtime`.

Start with [Building a cop](docs/building-a-cop.md) for the complete workflow.
The [Prism cop DSL reference](docs/adding-a-prism-cop.md) documents the shared
callback, matcher, context, and diagnostic APIs. Extend those APIs only for a
recurring concept, not to hide logic that belongs to one cop.

Before starting a cross-cutting subsystem, check the
[substantial-work roadmap](docs/substantial-work.md). It owns multi-stage
correctness and architecture work; the generated
[`docs/remaining-cops.md`](docs/remaining-cops.md) owns current project-parity
failures and `docs/bottlenecks.md` owns measured performance work.

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
2. Treat 60 function lines, cognitive complexity 15, and five arguments as
   review triggers. Hard ceilings are 350 lines and 16 cops per module, 120
   lines per function, cognitive complexity 25, and seven arguments. These are
   emergency brakes, not allowances.
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
bundle exec rake quality:test_contracts
bundle exec rspec
```

For a cop implementation, also run its cached controlled contract with
`script/verify_cop.rb Department/CopName`. For parser, traversal, correction
ordering, or registry changes, run `bundle exec rake fixtures:unit`. Neither
command starts RuboCop. Performance comparisons must use identical files,
configuration, Ruby version, warmup, and process model; report medians and
disclose whether RuboCop's Prism parser was enabled.

Every real-project discrepancy used to change a cop must become a minimized
unit contract under `spec/fixtures/cops/<Department>/<Cop>/unit/`. Preserve the
repository, revision, original path, and triggering line in the case's origins;
include a nearby clean control and cached diagnostics, `-a`, and `-A` outcomes.
Do not commit a second fixture format or a whole project tree. A known mismatch
remains a failing unit contract until the implementation matches RuboCop.

After unit and upstream checks pass, run the 50-project gate against the exact
source tree being evaluated:

```sh
bundle exec ruby script/audit_project_parity.rb \
  --cops Department/First,Department/Second \
  --report tmp/project-parity/current-head.json \
  --markdown tmp/project-parity/current-head.md
```

The gate records a clean-tree Git commit when available. For a dirty tree it
records a deterministic SHA-256 over every native cop source file; both forms
are paired with the release-binary SHA-256. Compare paths, severities, messages,
and full source ranges; equal offense counts are insufficient. If later work
changes an audited cop or a shared helper it depends on, re-run that cop at the
new source identity before calling it current. Project-exact is a statement
about the pinned corpus and configuration; fixture coverage remains required
for autocorrection and unexercised branches.

The full captured diagnostic run fails on any retained fixture difference.
Generate the public evidence matrix and work queue only from a complete
real-project audit:

```sh
ruby script/generate_project_parity_docs.rb \
  tmp/project-parity/all-cops-current.json
```

The generator rejects focused or truncated reports.

`quality:test_contracts` checks that every committed compatibility example is
a canonical cop-owned unit contract and that all cache digests are intact. Use
`script/verify_cop.rb Department/CopName` for the fast
focused contract. Add `--live-rubocop` only when deliberately validating the
capture pipeline; refresh the complete cache with
`bundle exec rake fixtures:refresh_unit`. Performance scripts consume the independent pinned
`benchmark/corpus.json` and update their marked README, performance-guide, and
ADR sections from the measured JSON reports.

The same gate checks `spec/source_cop_inventory.yml`. New source-wide callbacks
must appear there and begin as `unreviewed`; classify them as `lexical` only
when raw source is the actual contract, or `syntax_aware_migrate` when the rule
belongs on Prism nodes. Legacy text cops remain `temporary_text` until migrated.
