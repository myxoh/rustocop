# Substantial work roadmap

This roadmap tracks shared work that cannot be completed by repairing one cop.
The live per-cop queue is generated from a complete project audit in
[Current project-parity gaps](remaining-cops.md). Captured upstream cases are a
regression layer, not the prioritization scoreboard.

## Current position

- RuboCop 1.87.0 defines 606 built-in cops. The native registry advertises 512;
  94 non-scalable, incomplete, or reference-blocked implementations are intentionally pending.
- The executable trust standard is differential: minimized fixtures plus
  complete diagnostic signatures across 50 pinned projects.
- The project-regression corpus currently contains 126 RuboCop-derived cases,
  with five additional configuration-mutation cases.
- Atomic multi-edit correction transactions and deterministic file-level
  parallelism are implemented.
- A cop being registered, passing captured upstream examples, or producing a
  similar offense count is not a compatibility claim.

Current matrix counts and the tested Rust/binary hashes live in
[Real-project output parity](real-project-parity.md), not in this roadmap.

## Priority summary

| Priority | Workstream | Outcome |
| --- | --- | --- |
| P0 | Eliminate current project-parity gaps | Make the pinned real-project outputs identical cop by cop. |
| P0 | Configuration parity | Ensure cop selection and every consumed setting match RuboCop. |
| P0 | Input failure semantics | Make syntax, encoding, and unreadable-file behavior explicit and differential. |
| P1 | Autocorrection convergence | Match multi-pass RuboCop corrections without loops or conflicting edits. |
| P1 | Shared semantic capabilities | Replace cop-local approximations with scope, control-flow, layout, regexp, and metrics helpers. |
| P1 | Dispatch metadata | Route cops by relevant node kind while preserving deterministic output. |
| P2 | Filesystem and mixed-mode effects | Define safe correction and stdin behavior for filesystem cops and Ruby custom cops. |
| P2 | RuboCop lifecycle | Make upgrading the pinned RuboCop version repeatable. |

## P0: close real-project output gaps

Use the generated [gap queue](remaining-cops.md), ordered by unmatched complete
diagnostic signatures. For each repaired cop:

1. isolate the smallest real-project trigger and a nearby clean control;
2. add it to `spec/fixtures/project_parity_regressions/manifest.tsv` with
   repository, revision, and source path;
3. match RuboCop diagnostics and correction output in the focused fixture;
4. run all Rust tests and the complete cross-engine fixture corpus;
5. commit the Rust code, then run the SHA-bound 50-project audit; and
6. regenerate the public evidence docs only from a complete active-cop audit.

Focused exact reports are useful development evidence but never replace the
complete matrix after a shared helper or traversal changes.

## P0: configuration parity

The config reader now rejects unreadable requested files, preserves typed
scalars, lists, maps, block scalars, quoted values, and merges the pinned
defaults. Remaining work should be driven by differential failures, especially:

- inheritance (`inherit_from`, `inherit_gem`) and cycle behavior;
- ERB, plugins, and required Ruby files;
- department/default selection and `NewCops` precedence;
- `Include`/`Exclude` path roots and merge behavior; and
- settings not yet represented by the shared `CopPolicy` API.

Unsupported configuration must fail visibly or delegate safely; it must not
silently select a different ruleset.

## P0: syntax, encoding, and file failures

Define one user-visible policy for invalid Ruby, unsupported encodings,
unreadable files, stdin, CRLF, BOMs, and partial Prism trees. `Lint/Syntax`,
simple output, JSON output, and exit status must agree with RuboCop wherever the
native engine claims support. Add differential fixtures for every failure
class instead of relying on recovered-tree behavior.

## P1: correction convergence

The engine already applies each accepted correction transaction atomically.
The remaining cross-cutting work is bounded repeat-pass correction:

- rerun until stable when a correction exposes another offense;
- preserve deterministic conflict priority;
- detect cycles and cap passes;
- require idempotence fixtures; and
- define ordering when native and delegated Ruby cops are selected together.

## P1: shared semantic capabilities

Extract a shared helper only when multiple observed failures need the same
answer. The main capability areas are:

- lexical scopes, definitions, assignments, aliases, and singleton ownership;
- conservative reachability and rescue/ensure control flow;
- indentation, delimiter, heredoc, continuation, and comment geometry;
- Ruby regexp structure and escaping;
- metrics calculations; and
- project/path/filesystem context.

These APIs must be tested independently and then adopted by representative
cops. Avoid building a second parser or a generic `utils` layer.

## P1: registry and dispatch metadata

Every file is already parsed once, but enabled node cops still receive many
irrelevant nodes. Add stable subscriptions by Prism node kind, then benchmark a
call-name index only if profiling justifies it. Preserve cop ordering,
normalized output, correction conflict behavior, and sequential/parallel
identity.

## P2: lifecycle and sustained validation

- Record Rust source and native SHA-256 in every committed correctness or
  performance claim.
- Keep project revisions and exclusion rules centralized in
  `lib/rustocop/project_corpus.rb`.
- Run configuration-mutation profiles after cop/config changes.
- Refresh benchmarks only in isolation; dated measurements are historical, not
  current compatibility evidence.
- Automate vendoring and recapturing when moving beyond RuboCop 1.87.0.

## Definition of done

A shared workstream is complete only when its differential fixtures pass,
affected project signatures match, architecture and test-contract checks pass,
and the documentation can be regenerated from a committed, SHA-bound report.
