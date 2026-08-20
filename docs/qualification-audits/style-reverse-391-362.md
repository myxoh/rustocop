# Reverse Style audit: positions 391–362

This batch was evaluated with the project-first qualification gate on 2026-08-20.
It covers `Style/FileRead` through `Style/Documentation` against Rust source SHA
`15032c62724a1bfcd1d7199a9342188ff3b96ee4` and RuboCop 1.87.0 at
`e5b788dba181ad94de30cfbad661c5d6aa08a4e5`.

## Outcome

No cop in this batch is added to the authoritative qualification ledger.

| Classification | Cops |
| --- | ---: |
| Real-project diagnostic mismatch | 14 |
| Rustocop crash | 1 |
| Exact but dormant on the project corpus | 10 |
| Project-exact, rejected by source boundary review | 5 |
| Newly qualified | 0 |

The gate inspected 34,073 Ruby files from pinned Chatwoot, RubyGems.org, and
GitLab CE snapshots. After quarantining the crashing cop, Rustocop emitted
85,003 diagnostics, RuboCop emitted 68,359, and 38,848 matched exactly.

## Per-cop project gate

`Rust`, `Ruby`, and `Exact` are aggregate diagnostic counts. Equality requires
the path, cop, severity, message, and complete start/end location to match.

| Cop | Rust | Ruby | Exact | Gate result |
| --- | ---: | ---: | ---: | --- |
| `Style/FileRead` | 2 | 5 | 2 | Mismatch |
| `Style/FileOpen` | 11 | 11 | 11 | Boundary mismatch |
| `Style/FileNull` | 41 | 41 | 41 | Boundary mismatch |
| `Style/FileEmpty` | 6 | 6 | 6 | Boundary mismatch |
| `Style/FetchEnvVar` | 832 | 475 | 444 | Mismatch |
| `Style/ExponentialNotation` | 2 | 2 | 2 | Boundary mismatch |
| `Style/ExplicitBlockArgument` | — | — | — | Crash |
| `Style/ExpandPathArguments` | 0 | 0 | 0 | Dormant |
| `Style/ExactRegexpMatch` | 0 | 0 | 0 | Dormant |
| `Style/EvenOdd` | 0 | 0 | 0 | Dormant |
| `Style/EvalWithLocation` | 46 | 0 | 0 | Mismatch |
| `Style/EnvHome` | 0 | 0 | 0 | Dormant |
| `Style/EndlessMethod` | 3,600 | 2 | 0 | Mismatch |
| `Style/EndBlock` | 131 | 0 | 0 | Mismatch |
| `Style/Encoding` | 0 | 0 | 0 | Dormant |
| `Style/EmptyStringInsideInterpolation` | 14 | 22 | 14 | Mismatch |
| `Style/EmptyMethod` | 23 | 22 | 22 | Mismatch |
| `Style/EmptyLiteral` | 28 | 9 | 9 | Mismatch |
| `Style/EmptyLambdaParameter` | 0 | 0 | 0 | Dormant |
| `Style/EmptyHeredoc` | 0 | 0 | 0 | Dormant |
| `Style/EmptyElse` | 0 | 17 | 0 | Mismatch |
| `Style/EmptyClassDefinition` | 881 | 881 | 881 | Boundary mismatch |
| `Style/EmptyCaseCondition` | 2 | 0 | 0 | Mismatch |
| `Style/EmptyBlockParameter` | 2 | 0 | 0 | Mismatch |
| `Style/EachWithObject` | 0 | 0 | 0 | Dormant |
| `Style/EachForSimpleLoop` | 0 | 0 | 0 | Dormant |
| `Style/DoubleNegation` | 167 | 81 | 74 | Mismatch |
| `Style/DoubleCopDisableDirective` | 0 | 0 | 0 | Dormant |
| `Style/DocumentationMethod` | 52,083 | 49,494 | 37,342 | Mismatch |
| `Style/Documentation` | 27,132 | 17,291 | 0 | Mismatch |

## Crash

`Style/ExplicitBlockArgument` panics while inspecting Chatwoot's
`lib/linear/mutations.rb`:

```text
crates/rustocop/src/cops/prism/structural_forwarding_completion.rs:259:25
byte range starts at 42 but ends at 37
```

## Boundary review of project-exact cops

The five project-exact cops also passed their captured upstream contracts
(140/140 cases total). Grouped unsafe correction produced byte-identical results
on all three projects. Source comparison then exposed semantic branches absent
from both test surfaces:

| Cop | Boundary example | RuboCop / Rustocop diagnostics |
| --- | --- | ---: |
| `Style/FileOpen` | A discarded `File.open` before another expression inside a method | 1 / 0 |
| `Style/FileNull` | A comment mentions `/dev/null`, followed by `"NUL"` | 0 / 1 |
| `Style/FileEmpty` | `File&.size("x") == 0` | 0 / 1 |
| `Style/ExponentialNotation` | `10E3` under the default scientific style | 0 / 1 |
| `Style/EmptyClassDefinition` | `Foo = Class&.new(StandardError)` | 0 / 1 |

Additional exponential-notation differences were confirmed for `0_1e3` in
scientific style and `1_1e3` in integral style.

## Timing and workflow result

The batched diagnostic gate took about two minutes. The five project-exact cops
then took under one second for all 140 captured upstream cases. Full-project
correction and automatic evidence extraction dominated the remaining machine
time. Manual source review was limited to five cops instead of all 30.

This audit intentionally leaves implementation work for the 20 crashing or
mismatching cops as separate tasks and does not count dormant cops as evidence.
