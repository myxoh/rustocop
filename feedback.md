# Maintainer feedback

Reviewed on 2026-08-18 against the current working tree.

## Implemented follow-ups

- Correction intents are now atomic transactions. A finding is marked
  corrected only after all of its edits pass range and conflict validation.
- The historical `parity_calls.rs` and `parity_source.rs` buckets were replaced
  by capability-oriented `source_semantics.rs` and `lexical_rules.rs` modules.
- Reusable delimiter, definition, and argument scanning lives in the tested
  `source_syntax.rs` infrastructure module.
- The architecture ceiling now enforces 350 lines and at most 16 declarations
  per cop module, without composition-root exceptions. The Prism composition
  tests live separately.
- `script/new_cop.rb --family <module>` can add a cop to an existing cohesive
  family without creating another module and registration entry.
- Prism families are declared and registered through one `cop_modules!` list;
  public names are derived from implementations rather than a duplicate global
  catalog.

## Short version

Rustocop has a good execution core, unusually useful compatibility tooling, and
a realistic product goal. It is already credible as a fast, advisory local
check that runs before authoritative RuboCop in CI.

It is not yet credible as a generally RuboCop-compatible linter. The hardest
part is no longer parsing Ruby quickly; Prism already solves that. The hard
part is reproducing RuboCop's semantics, configuration model, and
autocorrection behavior without accumulating source-text heuristics that pass a
finite spec corpus but fail on ordinary Ruby syntax.

My honest assessment is:

- the execution architecture is stronger than the individual cop quality;
- the upstream-spec capture harness is the project's most valuable asset;
- the current “Verified” label is too easy to overread, even though the README
  carefully defines it as captured-spec verification;
- adding another hundred simple cops is plausible with the present foundation;
- reaching trustworthy parity for all 606 cops will require deeper APIs for
  configuration, correction transactions, scope, and semantic analysis—not
  just more cop functions.

## What is genuinely good

### The product boundary is honest

The README says this is fast local feedback before real RuboCop runs in CI.
That is exactly the right boundary today. It makes false negatives survivable
and avoids pretending the project is ready to be a security or merge gate.

### Parse once, inspect many is the right core architecture

The run-level `InspectionPlan`, stateless cop registry, single Prism parse per
file, shared traversal, and deterministic file-level parallelism are all sound
choices. They leave room to improve dispatch without changing the public
execution model.

The split between application I/O, execution, and cops is also clear enough
that a newcomer can find the relevant layer without reading the whole project.

### The RuboCop spec capture is excellent leverage

Vendoring RuboCop's own specs and capturing executable expectations is much
better than inventing a home-grown fixture suite. It checks exact messages,
ranges, severity, configuration, and asserted corrections across 28,623 cases.
The recent filename and UTF-8 correction-capture fixes also show why owning the
capture pipeline matters.

This harness is the best reason the project could eventually become reliable.
It turns parity from opinion into a measurable target.

### The authoring surface is heading in the right direction

`define_cops!`, typed node callbacks, `CopContext`, `CopPolicy`, source
geometry, structural call matchers, and high-level correction helpers remove a
lot of incidental Rust. A straightforward AST cop can now be short and
readable.

The scaffolding and focused verifier make the happy path discoverable:

```sh
ruby script/new_cop.rb Style/Example call
ruby script/verify_cop.rb Style/Example
```

### The project is unusually transparent about measurements

Performance and memory results disclose that the 500-file corpus contains only
9,090 bytes and mostly measures startup and orchestration. That caveat is
important. The reported speedup is real for that benchmark, but the docs do not
claim it proves a 55-times faster parser.

### There are useful guardrails

Formatting, Clippy, layer checks, module ceilings, the generated support
matrix, integration comparisons, and deterministic parallel-output tests all
reduce the chance of accidental architectural drift.

## The hardest parts

### 1. Knowing Ruby syntax is not the same as scanning Ruby source

This is the most immediate correctness problem.

Several cops registered through the Prism engine are actually source-wide
substring or line scanners. That can be appropriate for genuine lexical rules,
but it is brittle for syntax-aware cops. Passing all captured upstream cases
does not prove that a scanner ignores strings, comments, heredocs, regular
expressions, nested delimiters, escaped delimiters, or unusual but valid Ruby.

The review demonstrated false positives outside the captured corpus:

```ruby
"send(foo)"
# send(foo)
```

Rustocop originally reported two `Style/Send` offenses; RuboCop reported none.
`Style/Send` was subsequently migrated to a Prism call callback during the
authoring quick-win pass, and this particular gap is now fixed. It remains a
useful example of why the source-scanner audit is necessary.

Likewise:

```ruby
'#{ }'
```

Rustocop reports `Lint/EmptyInterpolation`; RuboCop reports none because a
single-quoted string does not interpolate.

Both cops were marked Verified when these gaps were found because every
captured upstream case passed. This is not a failure of the capture harness;
finite upstream specs are not an exhaustive Ruby grammar. It does mean that
“Verified” should be read as “upstream-spec verified,” not “behaviorally
equivalent.”

Recommended response:

1. Move syntax-aware source callbacks onto Prism nodes.
2. Restrict raw source callbacks to truly lexical/file-level contracts.
3. Add adversarial negative fixtures for every scanner: strings, comments,
   heredocs, regexps, escaped text, nested structures, and UTF-8.
4. Add a second status such as `SpecVerified` versus `Hardened`, or rename the
   current label in user-facing documentation.

### 2. Autocorrection needs transactions, not a flat list of edits

The current diagnostic context marks a finding as corrected when an edit is
submitted. Later, `apply_edits` silently drops overlapping or out-of-bounds
edits. A finding can therefore be reported as corrected even when its edit was
not applied.

There is also only one edit per finding. Some RuboCop corrections naturally
need multiple coordinated edits. The multiline-array implementation already
has to replace a larger region to represent what is conceptually two changes.

RuboCop may also run correction passes repeatedly until stable. Rustocop
currently applies one batch. This will become difficult when more cops interact
or when one correction exposes another offense.

A better model would provide:

- an edit group or correction transaction per finding;
- validation of every range before the finding is marked corrected;
- explicit conflict resolution and diagnostics for rejected edits;
- deterministic cop priority for conflicting corrections;
- optional bounded repeat passes for cops whose contract requires them.

This is one of the hardest pieces because it touches correctness, output
compatibility, and safe file mutation at the same time.

### 3. RuboCop configuration is a language of its own

The current parser is a useful subset, but it is a handwritten line parser for
YAML-shaped input. It does not reproduce RuboCop's effective configuration.
Important gaps include inheritance, plugins/requires, department defaults,
`Enabled`, `NewCops`, merging behavior, and globally enforced `Include` and
`Exclude` semantics. Unreadable config files are silently ignored.

This can cause the most confusing class of local failure: rustocop and CI may
run different cops before they even disagree about a cop's implementation.

Configuration should become a first-class subsystem with an explicit support
contract. At minimum:

- parse actual YAML rather than approximating it line by line;
- fail visibly on an unreadable requested config;
- resolve enabled cops from config, not a hardcoded disabled list alone;
- apply path inclusion/exclusion centrally before dispatch;
- either implement inheritance or warn when it is encountered;
- report ignored plugin and require directives.

This work is less exciting than adding cops, but it will improve real-world
parity more than many additional lint rules.

### 4. The remaining cops are not evenly difficult

The missing 433 cops are not just more examples of the first 116. The long tail
contains rules requiring:

- lexical scope and ancestor-sensitive behavior;
- local variable and assignment tracking;
- control-flow and unreachable-code reasoning;
- method definition and call semantics;
- project or cross-file indexes;
- target Ruby-version feature knowledge;
- sophisticated formatting and multi-pass corrections;
- extension-specific context from Rails, RSpec, and other plugins.

Adding a selector rewrite and implementing `Lint/UselessAssignment` are
fundamentally different tasks. Cop counts will become a misleading progress
measure unless the backlog is grouped by required capability.

### 5. The hybrid text/Prism architecture carries migration debt

The text pipeline was a sensible bootstrap, but it now creates two authoring
models, two registries, two correction phases, and source-position concerns
across pre- and post-Prism representations.

The old global `SUPPORTED_COPS` catalog has been removed. Prism names are
runtime-discovered and the remaining legacy text names are isolated in the
text layer. `InspectionPlan` only checks that short legacy list when deciding
whether textual processing is necessary. This removes the cross-layer catalog,
but the two execution models still remain.

The long-term direction should be one registry with explicit metadata:

- callback/node subscriptions;
- whether a cop needs lexical source, AST, or both;
- whether it can correct;
- whether it requires project context;
- configuration keys it consumes.

Text cops do not need to disappear immediately, but new syntax-aware work
should not deepen that branch of the architecture.

### 6. Parser failures and unsupported input need an explicit policy

The Prism result is traversed, but parse errors are not surfaced as user-facing
diagnostics. File reads require UTF-8 through `read_to_string`. A fast local
linter that silently analyzes a partial/error tree can give more confidence
than it should.

The CLI should clearly decide whether invalid syntax is an error, a dedicated
offense, or a skipped file. Unsupported encodings should also produce an
actionable error rather than being treated as generic I/O failure.

### 7. Scaling dispatch to hundreds of cops will require indexing

Every enabled cop currently sees every Prism node and rejects irrelevant nodes
inside its callback. This is simple and fine at the current scale, but grows
roughly with nodes multiplied by enabled cops.

The existing bottleneck document has the right recommendation: index callbacks
by Prism node kind, then optionally by call method name. This is technically
easier than semantic parity, but it should happen before enabling hundreds of
AST cops by default.

## The easiest parts

### Straightforward call cops

Rules shaped like “for method `x` on receiver `Y`, with N arguments, report or
replace the selector” are the best fit for the current DSL. The matcher and
correction APIs already express these cleanly and Prism supplies reliable
ranges.

These are low-risk when they:

- match a precise receiver and method;
- do not require type inference;
- inspect static literal arguments;
- have one local correction;
- have little configuration.

### Single-node literal and keyword cops

Cops that recognize one Prism node kind or one explicit token are also
relatively easy. Examples include deprecated literal forms, redundant wrappers,
or keyword substitutions where ancestry does not change the answer.

### Diagnostic-only cops

A correctable cop has to match RuboCop's range, replacement, conflict behavior,
and correction safety. A diagnostic-only cop avoids half of that surface.
These are the safest way for newcomers to learn the project.

### Genuine file-level lexical rules

Magic-comment ordering, duplicate file headers, byte-order marks, and initial
file indentation can reasonably operate on source geometry. They still need
care around encoding and comments, but they do not require pretending a string
scanner is an AST.

### Mechanical project work

Registration, focused comparison, support-matrix generation, and file-level
parallel execution are comparatively easy now. The project has already
factored most of this boilerplate away.

## Structure and maintainability

The architecture is understandable, but the newest batch shows early signs of
“parity bucket” modules becoming dumping grounds:

- `parity_calls.rs` is 521 lines;
- `parity_source.rs` is 401 lines;
- both names describe when the code was added rather than what the code does.

They pass the enforced 600-line emergency ceiling, but one already exceeds the
documented 400-line review trigger. The current architecture check therefore
enforces a looser rule than the prose recommends.

I would split these by capability, not arbitrary size—for example parameters,
interpolation, collection layout, file directives, and constructor calls. The
shared delimiter/argument scanners should either become tested lexical
infrastructure or disappear as those cops migrate to typed Prism nodes.

The opposite extreme is also worth avoiding: `new_cop.rb` creates one module
per cop, which would make `mod.rs` and the filesystem noisy at hundreds of cops.
A generator should be able to append to a named family module as well as create
a new module.

Unit coverage is modest for roughly 10,500 lines of Rust, but the upstream
differential suite compensates for that in important ways. What is still
missing is property/adversarial testing around the infrastructure most likely
to fail broadly: source scanners, delimiter matching, UTF-8 byte geometry,
config parsing, edit conflicts, and repeated corrections.

Some hand-written status prose is already drifting. For example, the README
still says the shared Prism visitor powers 50 built-in cops even after multiple
large Prism batches. Prefer generating numerical claims from the registry and
status file, or omit counts outside the generated support matrix.

## Performance take

The project is clearly fast enough to justify continuing. A roughly 3 ms
process floor, low single-digit MiB memory use, and deterministic file-level
threads are excellent for the intended local-feedback use case.

The current headline benchmark should not guide architecture too strongly. It
uses 500 tiny files totaling 9,090 bytes and only 20 shared cops. Before making
larger performance claims, add at least two more benchmark classes:

1. a real medium/large Ruby application with realistic file sizes and config;
2. a cop-scaling benchmark using every implemented cop, with node-dispatch cost
   separated from process startup and file discovery.

I would not invest heavily in more parallel scheduling yet. Node-kind dispatch,
lazy removal of the text representation, and real config loading are more
valuable next steps.

## Recommended order of work

1. **Fix trust issues before raising the verified count again.** Convert the
   demonstrated scanner false positives to AST implementations and add an
   adversarial negative corpus for every spec-verified cop.
2. **Make correction accounting truthful.** Introduce grouped edits and mark a
   finding corrected only after its complete transaction is accepted.
3. **Build effective configuration.** Correct enabled-cop selection and central
   path exclusion will improve every cop at once.
4. **Replace historical batch modules with capability-oriented modules.** Do
   this while the new code is still fresh.
5. **Add registry subscription metadata.** Dispatch by node kind before the
   enabled AST population becomes much larger.
6. **Classify the remaining cops by required engine capability.** Implement
   easy structural cops in batches, but separately plan scope, flow, and
   project-index infrastructure.
7. **Add realistic application benchmarks.** Preserve the current tiny corpus
   as a startup benchmark rather than replacing it.
8. **Create a regression-baseline command.** The full comparison currently
   exits unsuccessfully until all 606 cops pass. A CI mode should compare
   against `status.yml` and fail only on regressions or unapproved status
   changes.

## Final opinion

This is a promising prototype with a real architectural spine, not merely a
pile of regexes. The parse-once engine, compatibility capture, deterministic
parallelism, and authoring APIs are worth keeping.

The main danger is optimizing for the visible scoreboard—verified cops up,
missing cops down—faster than the semantic foundation can support it. The
recent 20-cop batch reached 184/184 captured cases, yet two trivial adversarial
examples immediately found false positives. One was easy to migrate to an AST
callback; the discovery is still the clearest signal about what the project
needs next.

If the project keeps RuboCop in CI, tightens the meaning of verification, and
invests in AST semantics/configuration/corrections before chasing all 606, it
can be genuinely useful. If it treats captured-spec passage as complete parity,
the support matrix will become more impressive while local trust gets worse.
