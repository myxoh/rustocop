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

## Highest-value capabilities to build next

The generated remaining-cop plan currently groups 329 non-Verified cops into
85 layout, 52 scope/symbol, 27 control-flow, 12 project-context, 11 regexp,
10 metrics, and 126 other AST-structural cops. That distribution argues for
shared capabilities in this order.

### 1. Statement and scope index

Build an immutable per-file index during the existing Prism traversal:

- containing lexical scope for every node;
- statement list, statement index, previous sibling, and next sibling;
- method/class/module/block boundaries;
- local declarations, reads, writes, and shadowing depth;
- visibility changes and definitions within a scope.

This would directly simplify unused arguments, duplicate methods, access
modifiers, redundant assignments, method-definition rules, and many naming
cops. Individual cops should query this index rather than each implementing a
custom `Visit` collector.

Do not begin with full Ruby data-flow analysis. A correct lexical index plus
scope-aware reads and writes covers a large portion of the 52-cop cluster and
creates the input needed by later flow analysis.

### 2. Layout context

Add a `LayoutContext` backed by Prism locations and `SourceFile`:

- first/last token and first/last line of a node;
- indentation column and indentation text;
- delimiter pairs and element/argument spans;
- sibling alignment anchors;
- comments attached before, after, or inside a node;
- line-break and blank-line queries;
- configured indentation width.

The key abstraction should be geometry, not one helper per Layout cop. The 85
remaining layout cops repeatedly ask the same questions with different policy.
A shared context would also reduce unsafe raw slicing in existing layout rules.

### 3. Call-chain view and renderer

Normalize a call chain into receiver, operator, selector, arguments, block, and
parent-call links. Provide rendering operations such as replacing one segment,
removing a segment, and parenthesizing command-form arguments.

This should serve safe-navigation consistency, redundant call chains, inverse
methods, collection querying, hash/array transformations, and parentheses
cops. Rendering must preserve comments and safe-navigation operators; a string
builder that reconstructs the complete call is not sufficient.

### 4. Conditional and branch view

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

### 5. Literal identity

Provide a canonical, hashable representation for static strings, symbols,
numbers, ranges, arrays, hashes, and constants when Ruby semantics make the
comparison safe. Keep “static value” distinct from “same source text.”

This would serve duplicate hash/set elements, duplicate branches, literal
conversion, collection literals, and several pattern cops. It should refuse
dynamic interpolation and other ambiguous values instead of guessing.

### 6. Focused parsers shared by cop families

Some domains are languages inside Ruby and deserve dedicated parsers:

- format-string tokens;
- regexp structure and capture groups;
- RuboCop directive comments;
- magic comments and encoding declarations.

These belong in tested infrastructure modules. They should not become methods
on a general-purpose AST DSL.

### 7. Project context and metrics

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
