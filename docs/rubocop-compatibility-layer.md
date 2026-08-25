# RuboCop compatibility layer

Rustocop targets RuboCop 1.87.0. The compatibility layer under
`crates/rustocop/src/rubocop/` translates RuboCop's shared implementation
boundaries into Rust while preserving names, control flow, and source
provenance closely enough for static review.

The layer is deliberately separate from existing cops. Adding or testing a
translation does not authorize migrating a cop to use it.

## Completion status

The pinned source-shaped implementation audit is complete:

- All 228 source components are accounted for: 191 direct translations, 30
  native Rust equivalents, and 7 facilities documented as not applicable.
  There are no partial or pending components.
- All 2,586 Ruby APIs discovered from syntax and the pinned gems' actual
  runtime-defined method surface pass the strict gate. This includes readers,
  writers, `Struct` members, delegated methods, `define_method` hooks, and
  `class_eval`-generated callback families. A broad Rust
  helper cannot account for multiple distinct Ruby callbacks unless the Ruby
  APIs are actual aliases or generated equivalents. Same-named operations in a
  consolidated Rust file also require an exact source/API ownership declaration.
- Every public Rust target counted by that API ledger is exercised outside its
  own definition. Definition-only translations force their component back to
  `partial`; the current ledger contains zero unexercised public targets.
- All 83 discovered shared upstream spec files belong to components that pass
  the strict translation gate.
- The cached RSpec dry-run inventory contains all 3,139 expanded examples,
  including shared-example expansions that the previous source-line counter
  missed. Every individual RSpec ID is bound to one named executable Rust test,
  its upstream description hash, and either shared semantic terms or a
  source-reviewed explicit rule. The binding is protected by a checked SHA-256
  contract; 244 focused Rust test functions currently cover those files and
  the source-level branches that have no direct upstream example.
  Marker-only `= all` and suite-level all-to-all claims are not accepted by the
  manifest gate. The pinned upstream baseline executes
  3,135 examples successfully; four upstream examples tagged as broken on the
  parser backend are retained in the inventory and exercised by Rust's
  Prism-oriented contracts.
- Every source and spec entry records its upstream SHA-256 in
  `crates/rustocop/rubocop-translation.json`.

These figures describe the shared compatibility layer only. They do not claim
that the existing 606 cop implementations consume the layer, or that those cops
have full fixture or real-project parity. Cop migration is a separate phase and
was explicitly excluded from this implementation.

The generated [progress report](rubocop-compatibility-progress.md) is the
current component-level ledger. Its `updated_at` value is always an ISO 8601
timestamp.

## Translation rules

1. Mirror the RuboCop source path and module boundary where practical.
2. Preserve constants, method names, branch order, and intermediate concepts.
3. Put native Rust facilities behind RuboCop-shaped interfaces rather than
   changing the translated algorithm.
4. Record the RuboCop version, source path, source SHA-256, translated tests,
   and every known deviation in `crates/rustocop/rubocop-translation.json`.
5. Account for every Ruby method found by the combined syntax and runtime
   inventory. A renamed or consolidated
   Rust operation must be recorded explicitly in the generator's API
   equivalence ledger. Its exact destination file and function are verified;
   unresolved APIs force the component back to `partial`. When multiple Ruby
   sources map to one Rust file and share a function name, the Rust source must
   declare which exact source/API pairs own that operation.
   Public Rust destinations must also have executable use outside their
   definition; definition-only targets are recorded as unresolved.
6. Port the corresponding RuboCop unit tests into Rust. Keep descriptions and
   cases recognizable. The manifest separately records each expanded upstream
   example ID, its description hash, the exact Rust test responsible for it,
   the mapping basis, explicit covered-example counts, and a digest over that
   complete binding. The binding is traceability evidence; both the pinned
   upstream suite and the Rust suite must still execute successfully. `partial`
   versus `translated` spec status remains independent so implementation
   coverage cannot be mistaken for test-port coverage.
7. Do not add project-derived special cases to this layer. Differences must be
   resolved by matching RuboCop's shared semantics.

## Status meanings

- `translated`: all behavior not listed in `deviations` has a Rust translation
  and focused tests.
- `partial`: the file is mapped and has working Rust behavior, but still has
  unaccounted implementation or upstream-test behavior and does not count as complete.
- `native`: Rust supplies the capability, exposed through a RuboCop-compatible
  interface with equivalent tests.
- `not_applicable`: the Ruby facility is unnecessary in Rust; the manifest must
  explain why.

The Rust manifest test verifies that every registered translation and test file
is present and carries matching provenance. The repository spec also compares
the recorded hashes with the pinned RuboCop or rubocop-ast gem and the vendored
upstream specs:

```console
bundle exec rspec spec/rubocop_translation_manifest_spec.rb
```

The implementation and all focused contract ports run with:

```console
cargo test --manifest-path crates/rustocop/Cargo.toml -- --test-threads=1
```

The exact registered upstream suites can be rerun independently with:

```console
bundle exec ruby script/run_rubocop_compatibility_upstream.rb
```

Both inventory generation and upstream execution activate the exact pinned gem
versions before loading RuboCop, so an additional installed version cannot
silently change the audited API surface or test behavior.

When either pinned upstream package changes, refresh the expanded RSpec example
inventory before regenerating the manifest:

```console
bundle exec ruby script/capture_rubocop_compatibility_examples.rb
ruby script/generate_rubocop_compatibility_inventory.rb
```
