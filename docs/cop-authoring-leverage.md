# Cop authoring leverage

This is the refactoring backlog for making the remaining built-in cops faster
to implement. It records patterns observed in real implementations rather than
speculative framework ideas.

The guiding rule is simple: extract syntax and correction mechanics that recur
across cop families, but leave the reason for an offense visible in the cop.
Helpers should remove Prism and byte-offset ceremony; they should not hide a
cop's policy behind a generic rule language.

## Extracted in the first leverage pass

### Node and body normalization

`node_helpers.rs` now owns common AST extraction:

- `arguments` materializes a complete call argument list;
- `joined_arguments` renders arguments without repeating Prism iteration;
- `only_statement` and `only_statement_in` unwrap one-statement bodies;
- `single_expression` accepts either a direct expression or a one-statement
  wrapper;
- `ModifierConditional` normalizes modifier-form `if`, `unless`, `while`, and
  `until` nodes into one typed view.

This split is intentional. `matchers.rs` answers structural yes/no questions;
`node_helpers.rs` extracts or renders a matched shape.

### Source geometry and local rewrites

`SourceFile` now provides:

- `node_range`, `full_line_range`, and `indentation_text`;
- `rewrite(container, edits)` for building one larger correction from absolute
  `SourceEdit`s.

`rewrite` validates containment and overlap before applying anything. This
replaces the repeated pattern of sorting edits backwards, rebasing offsets, and
calling `replace_range` by hand. `Style/BisectedAttrAccessor`,
`Style/MixinGrouping`, and `Style/ItBlockParameter` now exercise this API.

These helpers are small, but they remove some of the most error-prone code in
correctable cops: newline ownership, indentation slicing, and relative offset
arithmetic.

## Lessons from the 30-cop completion batch

The August 2026 batch registered 30 cops, moved 13 through every captured
diagnostic and correction assertion, and left 17 as explicit heuristics. That
split is useful evidence: a source callback can establish broad behavior very
quickly, but syntax-aware negative cases and exact corrections are where raw
string matching stops scaling.

### Text scanning is a prototype, not the default implementation

The batch repeatedly needed local replacements for balanced delimiters,
top-level comma splitting, quote handling, comments, safe navigation, root
constants, command-form arguments, and block forms. Each local scanner made a
few more examples pass, but the remaining failures are mostly cases Prism
already distinguishes correctly.

New syntax-aware cops should therefore begin on Prism nodes even when a source
pattern looks easier. A source implementation is appropriate for actual file
metadata, comments, magic comments, and whitespace. For call chains, literals,
assignments, definitions, branches, and blocks, use a source callback only as a
short-lived heuristic and record the AST migration as part of the same cop's
completion work.

### Negative examples define the abstraction

The most informative upstream cases were not the obvious offenses. They were
examples that must remain untouched:

- dynamic set elements that have identical source but different evaluations;
- Bundler groups with the same name but different option sets;
- visibility declarations using splats or multiple constants;
- invalid magic-comment values;
- assignment conditions whose modifier conversion would change semantics;
- root constants, safe navigation, and command-form calls whose correction
  must preserve their original operator or parentheses.

When extracting a helper, model these refusal cases first. A helper that only
recognizes the positive example moves boilerplate while leaving every caller
to rediscover the dangerous exceptions.

### Detection, offense geometry, and correction are three contracts

Many cases had the correct message but still failed because RuboCop highlights
only a selector, operator, body expression, or zero-width boundary. Other cases
had identical diagnostics but a different correction because a leading `::`,
safe-navigation operator, comment, space, or command-form parenthesis was lost.

Shared APIs should keep these concepts separate:

1. the semantic node or chain that establishes the offense;
2. the exact range RuboCop highlights;
3. the edit plan that preserves surrounding syntax.

`CopContext::replace` already accepts separate offense and edit ranges. The
next helpers should make the correct ranges easy to obtain rather than
reconstructing them from `find`, `rfind`, and string lengths in each cop.

### Configuration and related cops are part of matching

Target Ruby version, `Allowed*` values, style selection, file role, and related
`Layout/LineLength` configuration changed whether several examples were
offenses. These checks should happen before expensive matching and before a
correction is planned. Typed configuration access exists; new shared views
should accept policy inputs rather than reading configuration implicitly.

### Diagnostic parity and correction parity need separate queues

The refreshed remaining plan contains heuristic cops whose diagnostic cases
are already complete but whose correction assertions are not. Those are not
missing matchers. They need focused correction work. Reporting them together
with detection gaps wastes implementation time and makes apparently complete
cops difficult to promote.

The authoring tools should expose four counts per cop: expected-clean cases,
diagnostic matches, correction assertions, and complete cases. Promotion to
Verified must continue to require all four.

## Highest-leverage extractions after the batch

The current queue has 250 non-Verified cops: 43 partial implementations, 68
structural-batch candidates, and 139 engine-capability candidates. The recent
failures suggest the following implementation order.

### 1. Call-chain view and targeted chain edits

This is now the clearest immediate win. Add a normalized view with:

- receiver and root-constant identity;
- `.` versus `&.` operator locations;
- selector, arguments, and block locations;
- parent and child call segments;
- block form (`{}` or `do`/`end`) and parameters;
- targeted replacement or removal of a sequence of segments.

It should preserve source outside the selected segments and provide a
parenthesization edit for command-form arguments. This directly serves the
recent `FileRead`, `FileWrite`, `TallyMethod`, `RedundantSort`,
`RedundantMinMaxBy`, `ArrayIntersect`, `ZeroLengthPredicate`, and
`SymbolConversion` heuristics, plus safe-navigation and collection cops still
in the queue.

### 2. Delimited-list and token geometry

Add one quote-aware, comment-aware abstraction backed by Prism delimiter and
element locations. It should expose:

- matching opening and closing delimiters;
- top-level elements and their separator ownership;
- leading and trailing whitespace for each element;
- trailing comma and comment attachment;
- safe removal ranges for first, middle, last, and only elements.

This replaces the local balanced-parenthesis and top-level-entry scanners added
for lambda, hash, and set behavior. Prefer AST element locations; keep a small
lexical fallback only for constructs Prism does not locate directly.

### 3. Static literal identity

Implement a conservative `StaticValue` representation for strings, symbols,
numbers, ranges, arrays, hashes, constants, and static pattern elements. It
must return `None` for calls, interpolation, mutable evaluation, or any value
whose equality cannot be established safely.

This would let duplicate hash keys, set elements, match patterns, literal
constructors, and related cops share Ruby-aware equality instead of comparing
normalized source text.

### 4. Statement position and lexical scope index

Add queries for previous/next statement, tail position, enclosing branch,
definition scope, block variable, and declarations in the current scope. The
batch needed ad hoc versions for top-level methods, constant visibility,
gemspec block variables, and return-position double negation.

This should remain a lexical index, not a speculative full data-flow engine.
Tail-position and sibling queries alone would close several current heuristic
gaps and prepare the remaining unused-variable and shadowing cops.

### 5. Comment and file-header view

Centralize shebangs, recognized magic comments, documentation comments,
comment borders/margins, and comments attached before or after a statement.
This is a smaller abstraction, but it is safety-critical for corrections that
move definitions or alter blank lines.

### 6. Correction plan helpers

Build narrow correction operations on top of `replace_many` and
`SourceFile::rewrite`:

- replace a call-chain segment sequence;
- preserve or normalize a root constant intentionally;
- parenthesize command-form arguments;
- move an attached comment with its statement;
- replace a body while preserving its indentation and terminator;
- insert at beginning/end of file with RuboCop-compatible zero-width ranges.

These are preferable to a generic renderer. Each operation has a testable
source-preservation contract and can be reused without hiding why the cop
reports an offense.

## Workflow improvements with no runtime complexity

The manual compatibility loop itself exposed a tooling opportunity. Add a
batch verifier that accepts a cop list and prints, for every failing cop:

- passed and total diagnostics;
- passed and total correction assertions;
- the first failing source and effective configuration;
- expected versus actual offenses;
- expected versus actual corrected source.

It should rank cops by remaining failures and distinguish correction-only gaps.
This is likely to save more implementation time immediately than another DSL
macro.

For each future batch, build an AST-shape matrix before coding: root versus
relative constants, ordinary versus safe navigation, receiver versus no
receiver, parentheses versus command form, braces versus `do`/`end`, and
single-line versus multiline with comments. Running that matrix against the
upstream corpus early prevents a narrow textual implementation from becoming
the accidental architecture.

## Longer-term capability backlog

The updated priorities above cover the abstractions most strongly supported by
the recent batch. Four broader capabilities still matter for the rest of the
queue.

### Layout context

Add a `LayoutContext` backed by Prism locations and `SourceFile`:

- first/last token and first/last line of a node;
- indentation column and indentation text;
- delimiter pairs and element/argument spans;
- sibling alignment anchors;
- comments attached before, after, or inside a node;
- line-break and blank-line queries;
- configured indentation width.

The key abstraction should be geometry, not one helper per Layout cop. The 67
remaining layout cops repeatedly ask the same questions with different policy.
A shared context would also reduce unsafe raw slicing in existing layout rules.

### Conditional and branch view

`ModifierConditional` is the first small piece. Extend it only as consumers
arrive, toward a normalized conditional API covering:

- block and modifier forms;
- ordered branches and optional else;
- branch bodies as statement lists;
- negation/inversion with precedence-aware rendering;
- modifier-to-block and block-to-modifier correction plans.

This should unlock `Style/NestedModifier`, `Style/WhileUntilModifier`,
`Style/IfInsideElse`, `Style/MissingElse`, and parts of guard-clause and
conditional-assignment behavior.

### Focused parsers shared by cop families

Some domains are languages inside Ruby and deserve dedicated parsers:

- format-string tokens;
- regexp structure and capture groups;
- RuboCop directive comments;
- magic comments and encoding declarations.

These belong in tested infrastructure modules. They should not become methods
on a general-purpose AST DSL.

### Project context and metrics

Project-wide gem/dependency cops need a run-level read-only index rather than
global state in individual cops. Metrics cops need a shared counter model with
consistent treatment of blocks, branches, repeated attributes, and allowed
methods. Both are valuable, but each serves a smaller backlog than scope and
layout.

## What not to abstract

- Do not create a macro that combines matching, policy, message, and correction
  into an opaque one-liner. The present registration DSL removes boilerplate
  without hiding behavior; keep that boundary.
- Do not add generic `utils`, `common`, or `misc` modules. Name modules for the
  concept they model.
- Do not reconstruct complete AST nodes from normalized strings when a targeted
  edit can preserve comments and formatting.
- Do not move a one-off RuboCop exception into shared code merely to shorten one
  cop.
- Do not cache analysis inside cop instances. Cops are stateless and files may
  be inspected in parallel.

## Extraction rule of thumb

Promote a helper when at least two independent cop families ask the same
syntax-level question, or when the operation is safety-critical enough to own
centrally (source geometry and correction validation are examples). Otherwise,
keep it local until the next consumer makes the shared concept clear.

Every shared extraction should migrate representative existing cops and pass
their complete captured diagnostic and correction contracts. A shorter
implementation is not a win if exact RuboCop parity moves.
