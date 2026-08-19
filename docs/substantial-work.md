# Substantial work roadmap

This document tracks unresolved work that needs a shared subsystem, a behavior
decision, or a multi-stage migration. It is deliberately not a list of every
failing cop or small cleanup. The generated [remaining-cop queue](remaining-cops.md)
owns individual compatibility gaps, while [known bottlenecks](bottlenecks.md)
owns measured performance opportunities.

## Current position

- RuboCop 1.87.0 defines 606 built-in cops in the pinned compatibility target.
- 361 cops pass their captured upstream diagnostic and correction contracts.
- 245 cops have native implementations but remain heuristic.
- The remaining queue contains no completely unimplemented cop names.
- Three heuristic cops pass all diagnostic cases in the latest complete report
  and need correction and promotion review: `Style/AccessorGrouping`,
  `Style/IfInsideElse`, and `Style/MultilineTernaryOperator`.
- Atomic multi-edit correction transactions are implemented. They are not an
  open item here; repeat-pass correction and mixed native/Ruby correction are.

“Verified” currently means captured-upstream-spec parity. It does not prove
equivalence for Ruby programs that RuboCop's specs do not exercise. That
distinction is the largest remaining trust risk.

## Priority summary

| Priority | Workstream | Why it comes here |
| --- | --- | --- |
| P0 | Trust levels and adversarial hardening | Prevents the compatibility score from overstating reliability. |
| P0 | Effective RuboCop configuration | A wrong enabled-cop set makes every individual cop result misleading. |
| P0 | Syntax, encoding, and input-failure policy | Silent partial analysis is worse than a visible unsupported case. |
| P1 | Registry metadata and execution-model consolidation | Gives migrations and hundreds of cops one explicit execution contract. |
| P1 | Shared semantic capability engines | Unblocks most of the 245 heuristic cops without duplicating logic. |
| P1 | Correction convergence | Needed for correction parity when cops expose or conflict with later offenses. |
| P2 | Filesystem/project effects and hybrid correction | Completes cases that do not fit source-only native inspection. |
| P2 | RuboCop-version lifecycle and sustained validation | Keeps parity claims reproducible after the pinned release ages. |
| P3 | Cop-family taxonomy migration | Reduces maintenance cost after capability boundaries stabilize. |

P0 means correctness work that should precede another large promotion batch.
P1 is the main route to closing the compatibility gap. P2 is necessary for
complete product behavior but does not block most cops. P3 should not churn
modules before the underlying APIs settle.

## Tracked TODOs

Effort is relative: M is a contained subsystem, L crosses several layers, and
XL is a multi-stage program that should land incrementally.

| Status | ID | Priority | Effort | Outcome |
| --- | --- | --- | --- | --- |
| In progress | TRUST-1 | P0 | XL | Separate captured verification from adversarial hardening. |
| In progress | CFG-1 | P0 | XL | Resolve the same effective configuration as RuboCop. |
| Open | INPUT-1 | P0 | L | Make syntax, encoding, and file failures explicit. |
| Open | REG-1 | P1 | L | Give every native cop one metadata and dispatch contract. |
| Open | SEM-1 | P1 | XL | Provide shared scope and symbol facts. |
| Open | SEM-2 | P1 | XL | Provide conservative control-flow facts. |
| Open | SEM-3 | P1 | XL | Provide shared layout geometry and policy inputs. |
| Open | SEM-4 | P1 | L | Parse Ruby regexp semantics once. |
| Open | SEM-5 | P1 | L | Calculate metrics through one shared engine. |
| Open | SEM-6 | P1 | L | Provide deterministic project and filesystem context. |
| Open | CORR-1 | P1 | L | Run bounded correction passes to stability. |
| Open | IO-1 | P2 | M | Support validated filesystem correction effects. |
| Open | HYBRID-1 | P2 | L | Define stdin and correction behavior for Ruby custom cops. |
| Open | LIFE-1 | P2 | L | Automate upgrades of the pinned RuboCop contract. |
| Open | STRUCT-1 | P3 | L | Replace historical batch modules with capability families. |

## P0: make the compatibility claim harder to misread

### TRUST-1: introduce a hardened compatibility level

The upstream capture is an excellent executable contract, but finite specs do
not cover every string, comment, heredoc, regexp, nesting, encoding, or unusual
valid syntax context. Source scanners have previously passed the captured suite
while producing straightforward false positives outside it.

Work:

- Rename or qualify the current user-facing status as captured-spec verified.
- Add a second `hardened` status for cops that pass adversarial and real-project
  checks in addition to the upstream contract.
- Generate all support counts and documentation from the centralized status
  source; do not maintain parallel prose counts.
- Inventory every source callback and legacy text cop. Classify each as truly
  lexical, temporarily textual, or incorrectly syntax-aware.
- Build reusable adversarial case families for strings, comments, heredocs,
  regexps, escaped delimiters, nested syntax, CRLF, UTF-8, and invalid input.
- Migrate syntax-aware scanners to typed Prism callbacks before hardening them.

The executable inventory now lives in `spec/source_cop_inventory.yml`. Its
quality gate prevents additions from bypassing review; classifying the existing
`unreviewed` Prism source callbacks remains part of this workstream.

Exit criteria:

- Every source-wide cop has an explicit lexical justification or a tracked AST
  migration.
- Default-enabled verified cops pass the adversarial suite.
- The CLI and documentation expose captured verification and hardening as
  distinct facts.
- Status promotion fails automatically when diagnostic, correction, or required
  hardening evidence is missing.

## P0: implement effective RuboCop configuration

### CFG-1: replace the handwritten YAML-shaped parser

The current parser reads a useful subset of scalar, list, and map values, but
it does not reproduce RuboCop's effective configuration. Requested config files
that cannot be read are currently ignored, and enabled-cop selection does not
fully reflect inheritance, department defaults, `Enabled`, or `NewCops`.

Work:

- Parse YAML with a real YAML implementation and preserve typed values.
- Return actionable errors for missing, unreadable, or invalid requested config.
- Resolve `inherit_from`, `inherit_gem`, and merge behavior with cycle detection.
- Resolve `Enabled`, `DisabledByDefault`, `NewCops`, department selection, and
  command-line `--only` precedence in one place.
- Apply `AllCops` and per-cop `Include`/`Exclude` centrally before dispatch.
- Define support for ERB, `require`, and plugins. Delegate when safe; otherwise
  warn or fail explicitly instead of silently approximating the result.
- Add differential config fixtures that compare the effective enabled set and
  typed options with RuboCop.

Exit criteria:

- A documented compatibility matrix covers every RuboCop configuration feature.
- Config inheritance and path filtering have differential integration tests.
- The native runner and Ruby-custom-cop delegation consume the same resolved
  configuration and target paths.
- Unsupported configuration cannot silently change which cops run.

## P0: define input and parser failure semantics

### INPUT-1: make unsupported analysis visible

Prism exposes parse errors, but the general CLI policy for invalid Ruby is not
yet explicit. File loading also assumes UTF-8 text. This leaves uncertainty
around partial trees, magic encoding comments, binary bytes, stdin, and the
still-heuristic `Lint/Syntax` implementation.

Work:

- Decide whether each parse failure becomes `Lint/Syntax`, a file-level error,
  or a skipped inspection, matching RuboCop where practical.
- Preserve byte offsets while respecting Ruby magic encoding comments.
- Distinguish unreadable files, invalid encodings, invalid syntax, and internal
  engine failures in both simple and JSON output.
- Apply the same policy to files and `--stdin`.
- Add malformed, non-UTF-8, CRLF, BOM, and multibyte regression fixtures.

Exit criteria:

- No file is silently inspected through an error tree without a visible result.
- `Lint/Syntax` passes its captured upstream cases.
- JSON output has stable, tested behavior for every input failure class.

## P1: converge on one explicit cop execution model

### REG-1: add registry subscription metadata

The project still has Prism source callbacks, Prism node callbacks, parse-error
callbacks, and a legacy text pipeline. Enabled node cops currently receive
every Prism node and reject irrelevant kinds in their handlers.

Work:

- Give every cop explicit metadata for node kinds, call names where useful,
  source/parse hooks, correction support, consumed config keys, path context,
  project context, and filesystem effects.
- Dispatch node callbacks by subscribed Prism node kind while preserving cop
  order; benchmark call-name indexing separately before adding it.
- Represent legacy text cops in the same registry, even while their execution
  phase remains separate.
- Split before-Prism textual mutators from after-Prism readers and make the line
  representation lazy when neither is enabled.
- Reject new syntax-aware text cops through an architecture or registration
  check.

Exit criteria:

- One registry describes all native cops and their requirements.
- Node-kind dispatch preserves normalized output and improves the measured
  all-cop workload.
- Prism-only runs never allocate the legacy line representation.
- The remaining text pipeline contains only documented lexical debt.

## P1: build the shared semantic capabilities

The 245 heuristic cops group into a small number of capability lanes. Cop count
alone is not a useful implementation order:

| Capability | Heuristic cops | Captured cases passing |
| --- | ---: | ---: |
| AST structural | 98 | 4,897 / 11,003 |
| Layout engine | 63 | 2,140 / 4,262 |
| Scope and symbols | 34 | 2,150 / 4,632 |
| Control flow | 25 | 663 / 1,629 |
| Regexp semantics | 11 | 500 / 969 |
| Metrics engine | 7 | 96 / 211 |
| File metadata and lexing | 4 | 45 / 98 |
| Project context | 3 | 62 / 124 |

### SEM-1: scope and symbol model

Build reusable lexical scopes, definitions, references, assignments, shadowing,
parameters, constant paths, and ancestor queries. The model must understand
Ruby's block-local and numbered parameters, rescue variables, pattern matching,
endless methods, singleton scopes, and version-specific syntax.

Exit criteria: scope facts have focused unit tests and unblock representative
assignment, shadowing, naming, and unused-variable cops without cop-local tree
rescans.

### SEM-2: control-flow model

Build conservative reachability, terminal-expression, branch equivalence, loop
exit, rescue/ensure, and guard-clause facts. Do not attempt general type
inference.

Exit criteria: the shared model handles nested branches and exceptional flow,
and representative unreachable, redundant-branch, guard, and return cops use
it instead of separate ancestry heuristics.

### SEM-3: layout geometry engine

Create shared line/column geometry for delimiters, indentation anchors,
continuations, heredocs, comments, multiline collections, and aligned groups.
Keep layout policy separate from source mutation.

Exit criteria: representative alignment, indentation, spacing, and multiline
layout cops share tested geometry primitives and corrections remain idempotent.

### SEM-4: regexp semantics

Use a Ruby-compatible regexp parser or a narrowly scoped regexp AST rather than
source substring matching. Track character classes, escapes, interpolation,
options, captures, quantifiers, and Ruby-version differences.

Exit criteria: regexp cops share one parsed representation and adversarial
fixtures cover interpolation and escape behavior.

### SEM-5: metrics engine

Share code-unit selection, allowed-method/pattern policy, counting rules, and
branch/condition weighting across ABC size, complexity, length, and nesting
cops.

Exit criteria: one traversal can calculate the enabled metrics for a code unit,
and values match RuboCop across configuration variants.

### SEM-6: project and filesystem context

Introduce a run-scoped, read-only project index for the small group of cops
that depend on Gemfiles, gemspecs, related files, paths, or file metadata. Keep
project discovery out of individual cops.

Exit criteria: project-context cops remain deterministic in parallel runs and
stdin behavior is explicitly supported or rejected.

## P1: support correction convergence

### CORR-1: add bounded repeat-pass correction

Atomic edit transactions now make a single pass truthful, but RuboCop can
repeat correction until stable. One correction may expose another offense, and
interacting cops can produce different results depending on pass policy.

Work:

- Define stable cop priority and repeat-pass ordering.
- Reparse once per pass, never once per cop.
- Stop on stability, a conservative pass limit, or a repeated-source cycle.
- Surface rejected conflicts and non-convergence in test diagnostics.
- Add differential cases for corrections that expose, subsume, or conflict with
  later corrections.

Exit criteria:

- Repeated `-A` is idempotent for the verified correction corpus.
- Multi-pass fixtures match RuboCop and cannot loop indefinitely.
- Performance reports separate inspection-only and correction-pass costs.

## P2: support effects outside a single source buffer

### IO-1: model filesystem correction effects

`Lint/ScriptPermission` depends on a stable filename and executable mode, and
its correction is `chmod`, not a source edit. The upstream capture currently
does not preserve this state.

Work:

- Capture path and permission metadata in executable upstream cases.
- Expose immutable file metadata through the run context.
- Model filesystem changes as validated correction effects with dry-run and
  failure behavior, separate from source edits.
- Keep stdin isolated from effects that require a real file.

Exit criteria: `Lint/ScriptPermission` passes diagnostics and correction cases
without a source-text heuristic, including permission failures.

### HYBRID-1: complete mixed native/Ruby execution semantics

Mixed custom-cop mode currently rejects stdin and autocorrection. Native and
RuboCop inspections also build independent Prism trees, so one custom cop pays
most of RuboCop's startup and parsing cost.

Work:

- Specify stdin behavior and virtual filenames for delegated custom cops.
- Define safe correction ownership. A first acceptable design may reject files
  where native and Ruby correction streams both produce edits.
- Preserve deterministic diagnostics and exit status when the subprocess fails.
- Investigate a persistent RuboCop worker only if measured repeated local runs
  justify the lifecycle complexity.

Exit criteria: supported mixed modes have parity fixtures, unsupported
combinations fail before inspection with actionable errors, and performance
tradeoffs remain documented from generated reports.

## P2: make compatibility sustainable across releases

### LIFE-1: automate the RuboCop target lifecycle

The current contract is intentionally pinned to RuboCop 1.87.0. A new RuboCop
release can add cops, change defaults, alter messages, or change corrections.

Work:

- Add one command that vendors a chosen RuboCop tag, extracts cases, validates
  the capture, initializes status, and reports added/removed/changed cops.
- Keep status transitions reviewable rather than rewriting them implicitly.
- Run the old and candidate RuboCop targets during an upgrade window.
- Pin generated benchmark and compatibility metadata to the toolchain versions
  that produced them.
- Add scheduled real-project differential runs and sustained all-cop performance
  measurements with explicit non-regression thresholds.

Exit criteria: upgrading the pinned RuboCop release is a documented,
reproducible workflow with a machine-readable compatibility delta and no manual
support-count edits.

## P3: finish the cop-family taxonomy migration

### STRUCT-1: replace historical batch names with capability names

Architecture limits prevent files from growing without bound, but many cop
families are close to the 350-line ceiling and names such as `completion`,
`final`, `batch`, `additional`, and `more` describe implementation history
rather than behavior.

Work:

- Move families only when a shared semantic API establishes a stable boundary.
- Group by lint concept and execution requirement, not department alone or an
  arbitrary line count.
- Keep registration generated from the family definitions.
- Add ownership documentation for shared helpers that span multiple families.

Exit criteria: a newcomer can predict the destination of a new cop from its
required capability, and no family needs a historical-batch suffix.

## Delivery plan

### Milestone 1: trustworthy results

Complete TRUST-1, CFG-1, and INPUT-1. During this milestone, only promote cops
that already pass the complete captured contract and do not deepen known source
scanner debt.

### Milestone 2: one execution contract

Complete REG-1 and benchmark node-kind dispatch on the pinned tiny corpus, the
all-cop scaling corpus, and at least one large real project. Keep output parity
as a hard gate.

### Milestone 3: semantic foundations

Implement SEM-1 through SEM-6 as independent vertical slices. Each slice must
ship with its shared API, infrastructure unit tests, adversarial fixtures, and
at least three migrated cops before the API is considered stable.

Suggested order: layout geometry, scope/symbols, control flow, regexp semantics,
metrics, then project context. Layout has the largest clearly shared lane;
scope and flow carry higher semantic risk and should not be rushed merely to
raise the cop count.

### Milestone 4: compatibility closure

Work through the generated queue by capability. Promote only after diagnostic
and correction parity, then harden default-enabled and source-sensitive cops.
Implement CORR-1 when the first remaining cop demonstrates a repeat-pass
requirement rather than approximating that behavior in one cop.

### Milestone 5: non-source effects and custom Ruby cops

Complete IO-1 and HYBRID-1. These have explicit process and filesystem safety
boundaries and should not be hidden inside the ordinary cop API.

### Milestone 6: sustainable releases

Complete LIFE-1 and STRUCT-1, establish scheduled differential/performance
runs, and document the release criteria for changing the authoritative RuboCop
target.

## Rules for changing this roadmap

- Add an item here only when it needs shared architecture or several coordinated
  changes. Put individual cop failures in the generated remaining-cop queue.
- Every workstream needs an observable exit criterion before implementation.
- Close or rewrite an item when its facts change; do not leave completed design
  criticism as permanent TODO prose.
- Performance work requires a measured bottleneck. Correctness work requires a
  differential, adversarial, or real-project failure that demonstrates the gap.
- Do not trade an explicit unsupported error for silent approximation merely to
  reduce the remaining-cop count.
