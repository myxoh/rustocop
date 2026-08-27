# Rustocop structural parity standard

Structural parity means that Rustocop preserves RuboCop's semantic decomposition,
decision topology, callback lifecycle, state, configuration branches, offense
construction, ranges, corrections, and shared abstractions. Behavioral parity is
required corroboration, but does not establish structural parity by itself.

## Evidence and states

Existing fixture and project evidence is retained, while prior migration scores
and declarations are treated as historical metadata. A cop advances through:

1. `legacy_unverified`
2. `obligations_extracted`
3. `dossier_ready`
4. `implementation_submitted`
5. `review_rejected` or `review_blocked`
6. `accepted`, derived only from a valid independent attestation

Changing a relevant source, shared-runtime, evidence, schema, rubric, or standard
hash invalidates acceptance automatically.

## Required correspondence

Every dossier inventories callbacks, restrictions, configuration, lifecycle and
state, helpers, predicates, traversal, offenses, corrections, and mixins. Each
facet is explicitly `present` or `not_applicable`.

Every material upstream semantic unit maps to a precise Rust source span. Every
material Rust unit maps to upstream or to a necessary adaptation. Accepted
mappings are `direct` or `justified_adapter`. `missing` and
`unexplained_extra` are findings, never accepted mappings.

An adaptation identifies both sides, explains why it is necessary, states the
preserved invariant, and cites evidence. Generic parser, Prism, or ownership
prose is insufficient.

## Role separation

The implementation role may update dossiers and production code, but never
attestations. The review role runs in a fresh Codex task and may not change
production code, expected RuboCop output, this standard, or validation logic.

Behavioral parity alone cannot close review. Callback and traversal mismatches
remain findings until specifically mapped. Each offense, correction,
configuration branch, and piece of accumulated state maps individually.
Project-specific exceptions, fixture-specific exceptions, generic cohort
adaptations, and source scanning that replaces AST semantics are failures unless
a precise necessity is independently approved. Scores never establish
acceptance.

