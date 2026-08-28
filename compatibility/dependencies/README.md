# RuboCop dependency inventory

This directory is the version-pinned starting point for rebuilding Rustocop's
compatibility layer from the bottom up. Positive Rust equivalents are recorded
only after test-first implementation and independent whole-contract review.

## Scope

The inventory starts at RuboCop 1.87.0 and follows every declared runtime gem
dependency recursively. It currently covers 1,719 Ruby library files across 15
locked packages. This includes RuboCop's packaged dependencies such as
`rubocop-ast`, `parser`, `prism`, and `regexp_parser`.

Only loadable Ruby files under each gem's require paths are rows. Ruby standard
library files, optional undeclared integrations, specs, documentation,
executables, native binaries, and non-Ruby files are boundary dependencies and
are not rows. Static loads that cross that boundary and dynamic load sites are
retained in the graph metadata so the boundary is visible rather than silently
discarded.

## Artifacts

- `rubocop_dependency_graph.json` is the machine-readable graph. Its edges come
  from static `require`, `require_relative`, and `autoload` calls plus resolvable
  Ruby constant references.
- `rubocop_dependency_inventory.csv` is ordered from the most independent file
  to the least independent file. Direct dependency paths are JSON arrays inside
  the CSV field so paths remain unambiguous.
- `rubocop_dependency_annotations.json` is the hand-audited, hash-pinned source
  for the descriptive CSV fields. Its rows must form a contiguous prefix of the
  dependency ordering, which makes the audit advance one file at a time and
  prevents easier high-level files from being marked complete prematurely.
- `rubocop_dependency_rust_equivalents.json` records the separately reviewed
  Rust-equivalence judgment for each source-annotated file. It is subject to
  the same contiguous-prefix and source-MD5 rules.

Ruby libraries contain real cycles. The generator collapses strongly connected
components, ranks the resulting component graph, and sorts ties by direct
dependency count and path. A package file that bootstraps its own entrypoint is
kept as a real graph edge but excluded from ordering, preventing an integration
shim from making an entire package appear to be one indivisible cycle.

Every inventory row has these generated identity and dependency fields:

1. `RuboCop file`
2. `MD5 hash`
3. `path`
4. `known number of dependencies`
5. `actual dependency paths`

The completed annotation audit populates these source-side fields:

6. `classes and interfaces exposed` — structured JSON containing types,
   visibility-separated methods, constants, and top-level functions
7. `description`
8. `associated spec` — structured JSON containing the upstream package, spec
   path, MD5, and exact source revision

All 1,719 files are annotated. An empty associated-spec
array means that the exact upstream release has no known direct spec for that
file; it does not mean that downstream behavior is untested.

The Rust-equivalent field is populated only after a separate identity audit:

9. `Rust equivalent (if it exists)`

A Rust path may be recorded only when consumer-capability parity has been
demonstrated. A realistic Rust consumer must be able to express materially the
same logic as a Ruby consumer of the upstream library and obtain compatible
results for RuboCop's existing and reasonably foreseeable cop, DSL, traversal,
configuration, range, correction, and reporting use cases. A positive
equivalence must name existing workspace paths and include both API-shape and
behavior evidence in the equivalence manifest.

Necessary language adaptations are accepted when they preserve that consumer
capability. Rust does not need to reproduce incidental affordances of Ruby's
object runtime—such as reopening a constants-only module, rebinding a constant,
or mutating a string that all known and likely consumers treat as a protocol
constant—unless an actual or credible future library use depends on that
behavior. Conversely, an adaptation is not sufficient merely because current
tests pass: it must still let higher-level Rust code follow substantially the
same decomposition and decision logic as the Ruby source. `N/A` is reserved for
absent or partial capabilities, material semantic gaps, and cases where a
pragmatic faithful interface cannot be built.

`not_necessary` is a separate dependency-liveness judgment, not an equivalence
claim. It may be used only when the file has at least one known consumer and
every reverse dependency path terminates at a whole-file Rust equivalent with
explicit API-identity and behavior-identity evidence. The necessity audit does
not trust the legacy translation manifest as such evidence.

The initial equivalence-classification pass reviewed all 1,719 files. 1,498 had no provenance-linked Rust
candidate and were classified directly. The remaining 221 candidates were
reviewed against their compatibility metadata, APIs, and registered spec
evidence. None met the strict whole-file equivalence standard during that
classification pass.
Most of the existing spec links are inferred by semantic terms rather than
being assertion-preserving, one-to-one ports; therefore they are useful
candidate-discovery evidence but cannot certify equivalence. Exact review
findings and evidence counts are retained per row in
`rubocop_dependency_rust_equivalents.json`.

The dependency-ordered implementation phase has completed the first 77 files:

- all 77 now have test-first Rust equivalents approved by an independent
  whole-contract or consumer-capability review;
- no file in that implemented prefix remains `N/A`;
- 0 files are currently proven `not_necessary` by the reverse-dependency
  closure.

The dependency compatibility modules contain 151 focused contracts, in
addition to the remediated AST node and replacement-propagating processor
contracts. The full Rust suite passes 726 tests with 1 ignored test. The initial pass treated vendorized unfrozen
Ruby `String` constants as non-portable because Ruby permits mutating their
underlying objects. That was too strict for the consumer-capability goal and is
retained only as historical review context; constants used as protocol
constants should map to immutable Rust constants unless dependency evidence
shows mutation is material. Seven such protocol-constant files were revisited,
implemented, and independently approved. The original base AST node gap was
then closed with source-backed Ruby numeric identity, including arbitrary-size
integers and spelling-independent float, rational, and complex comparisons.
The processor gap was closed with a separate replacement-propagating processor
instead of misclassifying the inspection-only traversal visitor as equivalent.
The implementation frontier now extends through the dependency-ready LSP
constants and the first 39 LSP interface value objects, ending at
`CompletionOptions`.

The implementation phase populates these assessment fields only when the
corresponding situation has actually been encountered:

10. `Detailed reason why a Rust equivalent cannot be built`
11. `Known workarounds if you need the current file to implement your cop or
    higher level library`

An `N/A` row may carry a precise `cannot_build_reason` after a test-first port
and implementation attempt demonstrates that a faithful standalone Rust
equivalent would require unavailable semantics. `known_workarounds` is an
append-only array and is populated only for workarounds actually used by a
higher-level implementation; it must not be speculative.

## Reproduction

Generate the artifacts from the explicit 15-package version closure pinned in
the generator:

```sh
bundle exec ruby script/generate_rubocop_dependency_inventory.rb
```

Verify that the committed files are byte-for-byte reproducible:

```sh
bundle exec ruby script/generate_rubocop_dependency_inventory.rb --check
```

Rebuild source annotations from the pinned upstream checkouts, discover Rust
candidates, and conservatively finalize the equivalence review with:

```sh
bundle exec ruby script/complete_rubocop_dependency_annotations.rb
bundle exec ruby script/prepare_rubocop_dependency_equivalence_review.rb
bundle exec ruby script/finalize_rubocop_dependency_equivalence_review.rb
bundle exec ruby script/audit_rubocop_dependency_necessity.rb
bundle exec ruby script/generate_rubocop_dependency_inventory.rb
```

The generator fails on Ruby parse errors, duplicate rows, unknown or self
dependencies, invalid or stale source MD5 hashes, missing ranks, stale generated
artifacts, out-of-order annotations, incomplete annotation structures, and
malformed or revision-mismatched upstream spec evidence. Rust-equivalence rows
also fail validation if they are out of order, refer to an unaudited source
file, have a stale MD5, or claim a positive equivalent without the required
identity evidence.
