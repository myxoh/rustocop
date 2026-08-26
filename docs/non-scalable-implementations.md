# Non-scalable cop implementation register

`updated_at: 2026-08-24T07:46:27-04:00`

The intentionally-pending dataset is empty. All 23 cops that were withdrawn in
the final pending batch have been reimplemented, restored to the native registry,
and validated against every captured RuboCop 1.87.0 diagnostic and correction
case.

The former entries in this register are no longer current: literal catalog
rules, shared placeholder callbacks, line-count metrics, and the narrow
assignment scanner were replaced with syntax-aware implementations. The retired
`catalog_cop::report`, `catalog_cop::replace`, and shared continuation placeholder
APIs were removed so future cops cannot accidentally reuse those shortcuts.

This does not claim real-project parity. The complete 50-project audit remains
the broader negative-case and configuration gate, and its mismatch queue is
published separately in [Real-project output parity](real-project-parity.md) and
[Current project-parity gaps](remaining-cops.md).

## Reopening this register

Add a cop here only when review finds an implementation technique that cannot
reasonably generalize, such as literal source matching for a syntax rule,
line-oriented scope inference, or project-specific hardcoding. Each entry must
record:

1. the implementation mechanism that fails to scale;
2. fixture and project evidence demonstrating the limitation;
3. the structural capability required for a replacement; and
4. a complete ISO 8601 `updated_at` timestamp with UTC offset.

Ordinary project mismatches belong in the generated parity queue, not here.
