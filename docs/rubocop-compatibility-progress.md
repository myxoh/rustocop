# RuboCop compatibility implementation progress

Updated at: `2026-08-28T02:55:35Z`

Target: RuboCop 1.87.0 and rubocop-ast 1.49.1.
Existing cops are not consumers of this layer yet.

## Progress

- Accounted components: 228/228 (100.0%)
- Partially implemented: 0
- Translated: 191
- Native equivalent: 30
- Not applicable in Rust: 7
- Pending: 0
- Resolved syntax- and runtime-discovered APIs: 2586/2586 (100.0%)
- Unexercised public Rust API targets: 0

Registered upstream spec ports are tracked independently from component code:

- Fully ported spec files: 83/83
- Partially ported spec files: 0
- Upstream examples in registered files: 3139
- Focused Rust test functions for those files: 270
- Discovered RuboCop shared spec files: 18
- Registered RuboCop shared spec files: 18
- Unregistered RuboCop shared spec files: 0
- Discovered rubocop-ast spec files: 65
- Registered rubocop-ast spec files: 65
- Unregistered rubocop-ast spec files: 0

| Surface | Components |
| --- | ---: |
| `ast` | 101 |
| `cop_framework` | 28 |
| `cop_mixin` | 80 |
| `corrector` | 17 |
| `legacy` | 2 |

## Pending components

None.

## Unregistered rubocop-ast specs

None.

## Unregistered RuboCop shared specs

None.
