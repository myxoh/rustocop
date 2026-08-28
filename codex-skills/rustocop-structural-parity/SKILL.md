---
name: rustocop-structural-parity
description: Continuously drain the RuboCop-to-Rustocop structural-parity queue through evidence preparation, faithful implementation, independent subagent review, remediation, and audit. Use when asked to close, continue, review, or report Rustocop compatibility or implementation similarity.
---

# Rustocop structural parity

# Rustocop structural parity orchestrator

Continuously drain the structural-parity queue. Do not stop after one state
transition or one cop. Read `compatibility/structural/standard.md` completely,
then inspect the durable state:

```sh
ruby script/structural_parity.rb status
ruby script/structural_parity.rb next advance
```

Use the returned cop; never substitute an easier one. Keep only one cop in
flight because all agents share the worktree.

When goal support is available, create a durable goal to drain the queue until
every cop is accepted or genuinely blocked. Continue automatically across
turns and context compactions. Do not mark the goal complete while actionable
queue items remain.

## Per-cop loop

For each selected cop:

1. Prepare or resume its complete dossier.
2. Implement or remediate until it reaches `implementation_submitted`.
3. Spawn a fresh, context-isolated reviewer subagent. Give it only the cop
   name, this skill path, and the instruction to perform independent structural
   review. Do not give it the implementer's reasoning or intended verdict.
4. Inspect the review artifact and resulting state.
5. If rejected, remediate the precise findings and spawn a new fresh reviewer.
6. If accepted or genuinely blocked, select the next queue item immediately.

Never use the implementation agent as reviewer. Do not process multiple cops
in parallel. A subagent's completion message is not evidence; the persisted,
validated dossier or attestation is.

For **prepare**, initialize or resume the cop, read the complete upstream source, mixins
and specs plus its complete Rust implementation and material helpers, then fill
the bidirectional dossier. Do not change production code or attestations.

For **implement/remediate**, preserve upstream lifecycle, decomposition,
decisions, offenses, ranges, corrections, configuration, and shared
abstractions. Do not edit attestations, expected RuboCop output, the standard,
or validation. Run focused behavioral verification, update mappings and
fingerprints, and transition only to `implementation_submitted`. Use
`review_blocked` when faithful work requires missing shared infrastructure.

For **review**, use a fresh context-isolated subagent. Do not change production code or
governance. Independently reconstruct both inventories, verify every mapping,
challenge adaptations, and search for unmapped behavior. Record precise
findings and transition to `review_rejected` or `review_blocked` on any
failure. Only a fully demonstrated submission may receive an attestation
created from `attestation-template` and saved under
`compatibility/structural/attestations/`.

For **audit**, independently reconstruct an accepted mapping. Invalidate
unsound attestations and record findings under
`compatibility/structural/audits/`.

After every transition, run `ruby script/structural_parity.rb check`, persist
the result, and continue. Stop only when:

- the queue contains no actionable cops;
- every remaining cop is genuinely `review_blocked` with a precise prerequisite;
- an unsafe or unauthorized operation requires user approval; or
- the user explicitly stops the run.

At the terminal condition, report accepted, blocked, invalidated, and remaining
counts plus exact unresolved prerequisites. Rejection or blockage is valid
progress; never manufacture acceptance.
