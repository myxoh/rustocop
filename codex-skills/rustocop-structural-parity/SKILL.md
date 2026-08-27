---
name: rustocop-structural-parity
description: Advance one RuboCop-to-Rustocop structural-parity item through evidence preparation, faithful implementation, independent review, remediation, or audit. Use when asked to close, continue, review, or report Rustocop compatibility or implementation similarity.
---

# Rustocop structural parity

Advance exactly one queue item by one state transition. Read
`compatibility/structural/standard.md` completely first, then run:

```sh
ruby script/structural_parity.rb next advance
```

Use the returned role and cop; never substitute an easier one.

For **prepare**, initialize the cop, read the complete upstream source, mixins
and specs plus its complete Rust implementation and material helpers, then fill
the bidirectional dossier. Do not change production code or attestations.

For **implement/remediate**, preserve upstream lifecycle, decomposition,
decisions, offenses, ranges, corrections, configuration, and shared
abstractions. Do not edit attestations, expected RuboCop output, the standard,
or validation. Run focused behavioral verification, update mappings and
fingerprints, and transition only to `implementation_submitted`. Use
`review_blocked` when faithful work requires missing shared infrastructure.

For **review**, work from a fresh Codex task. Do not change production code or
governance. Independently reconstruct both inventories, verify every mapping,
challenge adaptations, and search for unmapped behavior. Record precise
findings and transition to `review_rejected` or `review_blocked` on any
failure. Only a fully demonstrated submission may receive an attestation
created from `attestation-template` and saved under
`compatibility/structural/attestations/`.

For **audit**, independently reconstruct an accepted mapping. Invalidate
unsound attestations and record findings under
`compatibility/structural/audits/`.

Finish with `ruby script/structural_parity.rb check`. Report the cop, role,
transition, verification, unresolved findings, and trustworthy state counts.
Rejection or blockage is valid progress; never manufacture acceptance.

