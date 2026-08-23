# RuboCop compatibility evidence

Generated at `2026-08-23T08:48:49-04:00` for RuboCop 1.87.0.
Compatibility is binary at the cop level: every exercised fixture must match,
and project output must have no false positives, false negatives, or signature
differences. Partial overlap is not classified as compatible.

This table covers 512 active built-in cops. The
94 cops in
[`intentionally_pending_cops.yml`](../spec/upstream/rubocop-1.87.0/intentionally_pending_cops.yml)
are deliberately unregistered and excluded from both evidence corpora.

Fixture evidence was updated at `2026-08-23T08:48:49-04:00`. Project
evidence was updated at `2026-08-23T08:45:38-04:00` from
10 projects and 54146 Ruby files.
Fixture source: `9e7049de18d0caf78c0e0e519cf24f016e9f650a`. Project source:
`9e7049de18d0caf78c0e0e519cf24f016e9f650a`.

## Overall

| Measure | Result | Percent |
| --- | ---: | ---: |
| Cops with fixture coverage | 512/512 | 100.0% |
| Cops with current fixture evidence | 512/512 | 100.0% |
| Fixture cases matching | 23999/23999 | 100.0% |
| Cops matching every fixture | 512/512 | 100.0% |
| Cops exercised on projects | 402/512 | 78.5% |
| Cops with current project evidence | 512/512 | 100.0% |
| Project-exact cops among exercised cops | 402/402 | 100.0% |
| Cops compatible in both evidence sets | 402/512 | 78.5% |

“Project hits” is the number of RuboCop reference diagnostics. Project matching
is exact shared signatures divided by the union of Rustocop and RuboCop
signatures, so both extra and missing diagnostics reduce the percentage. A
zero-hit row is unexercised, not 100% compatible.

## Updating

Refresh fixture evidence while retaining the existing project columns:

```sh
bundle exec ruby script/generate_compatibility_report.rb --refresh-fixtures
```

Refresh both evidence sets only when the expensive legacy RuboCop project scan
is intended:

```sh
bundle exec ruby script/generate_compatibility_report.rb \
  --refresh-fixtures --refresh-projects
```

Without either refresh flag, the generator only renders the checked-in compact
snapshots. Use `--check` in CI to verify that the table is current.

A stale marker means one of that cop's implementation files changed after the
relevant evidence commit. Stale rows remain visible but do not count as
compatible in the overall totals.

## Per-cop evidence

| Cop | Implementation file | Implementation updated | Fixture tests<br>(as of 2026-08-23T08:48:49-04:00) | Fixture matching<br>(as of 2026-08-23T08:48:49-04:00) | Project hits<br>(as of 2026-08-23T08:45:38-04:00) | Project matching<br>(as of 2026-08-23T08:45:38-04:00) |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `Bundler/DuplicatedGem` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 0 | — (unexercised) |
| `Bundler/DuplicatedGroup` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-22 | 21 | 21/21 (100.0%) | 0 | — (unexercised) |
| `Bundler/GemComment` | [`crates/rustocop/src/cops/prism/final_project_context_batch.rs`](../crates/rustocop/src/cops/prism/final_project_context_batch.rs) | 2026-08-19 | 26 | 26/26 (100.0%) | 0 | — (unexercised) |
| `Bundler/GemFilename` | [`crates/rustocop/src/cops/prism/bundler_completion.rs`](../crates/rustocop/src/cops/prism/bundler_completion.rs) | 2026-08-21 | 15 | 15/15 (100.0%) | 0 | — (unexercised) |
| `Bundler/GemVersion` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 0 | — (unexercised) |
| `Bundler/InsecureProtocolSource` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 0 | — (unexercised) |
| `Bundler/OrderedGems` | [`crates/rustocop/src/cops/prism/bundler_completion.rs`](../crates/rustocop/src/cops/prism/bundler_completion.rs) | 2026-08-21 | 17 | 17/17 (100.0%) | 0 | — (unexercised) |
| `Gemspec/AddRuntimeDependency` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Gemspec/AttributeAssignment` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Gemspec/DependencyVersion` | [`crates/rustocop/src/cops/prism/final_project_context_batch.rs`](../crates/rustocop/src/cops/prism/final_project_context_batch.rs) | 2026-08-19 | 77 | 77/77 (100.0%) | 0 | — (unexercised) |
| `Gemspec/DeprecatedAttributeAssignment` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-22 | 18 | 18/18 (100.0%) | 0 | — (unexercised) |
| `Gemspec/DevelopmentDependencies` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-22 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Gemspec/DuplicatedAssignment` | [`crates/rustocop/src/cops/prism/gemspec_completion.rs`](../crates/rustocop/src/cops/prism/gemspec_completion.rs) | 2026-08-22 | 17 | 17/17 (100.0%) | 0 | — (unexercised) |
| `Gemspec/OrderedDependencies` | [`crates/rustocop/src/cops/prism/gemspec_completion.rs`](../crates/rustocop/src/cops/prism/gemspec_completion.rs) | 2026-08-22 | 18 | 18/18 (100.0%) | 0 | — (unexercised) |
| `Gemspec/RequireMFA` | [`crates/rustocop/src/cops/prism/gemspec_completion.rs`](../crates/rustocop/src/cops/prism/gemspec_completion.rs) | 2026-08-22 | 15 | 15/15 (100.0%) | 0 | — (unexercised) |
| `Gemspec/RequiredRubyVersion` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 0 | — (unexercised) |
| `Gemspec/RubyVersionGlobalsUsage` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Layout/AccessModifierIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 43 | 43/43 (100.0%) | 118 | 118/118 (100.0%) |
| `Layout/ArrayAlignment` | [`crates/rustocop/src/cops/prism/layout_qualification.rs`](../crates/rustocop/src/cops/prism/layout_qualification.rs) | 2026-08-23 | 25 | 25/25 (100.0%) | 1685 | 1685/1685 (100.0%) |
| `Layout/AssignmentIndentation` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 7 | 7/7 (100.0%) |
| `Layout/BeginEndAlignment` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 8 | 8/8 (100.0%) |
| `Layout/BlockAlignment` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-23 | 78 | 78/78 (100.0%) | 96 | 96/96 (100.0%) |
| `Layout/BlockEndNewline` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-22 | 18 | 18/18 (100.0%) | 12 | 12/12 (100.0%) |
| `Layout/CaseIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 50 | 50/50 (100.0%) | 271 | 271/271 (100.0%) |
| `Layout/ClassStructure` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 1540 | 1540/1540 (100.0%) |
| `Layout/ClosingHeredocIndentation` | [`crates/rustocop/src/cops/prism/lexical_completion.rs`](../crates/rustocop/src/cops/prism/lexical_completion.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 701 | 701/701 (100.0%) |
| `Layout/ClosingParenthesisIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 44 | 44/44 (100.0%) | 13 | 13/13 (100.0%) |
| `Layout/CommentIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 29 | 29/29 (100.0%) | 32 | 32/32 (100.0%) |
| `Layout/ConditionPosition` | [`crates/rustocop/src/cops/prism/layout_line_break_completion.rs`](../crates/rustocop/src/cops/prism/layout_line_break_completion.rs) | 2026-08-21 | 14 | 14/14 (100.0%) | 0 | — (unexercised) |
| `Layout/DefEndAlignment` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-22 | 18 | 18/18 (100.0%) | 1 | 1/1 (100.0%) |
| `Layout/DotPosition` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-23 | 40 | 40/40 (100.0%) | 161 | 161/161 (100.0%) |
| `Layout/ElseAlignment` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 52 | 52/52 (100.0%) | 927 | 927/927 (100.0%) |
| `Layout/EmptyComment` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 21 | 21/21 (100.0%) |
| `Layout/EmptyLineAfterMagicComment` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 537 | 537/537 (100.0%) |
| `Layout/EmptyLineAfterMultilineCondition` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-23 | 22 | 22/22 (100.0%) | 1210 | 1210/1210 (100.0%) |
| `Layout/EmptyLineBetweenDefs` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-23 | 45 | 45/45 (100.0%) | 220 | 220/220 (100.0%) |
| `Layout/EmptyLinesAfterModuleInclusion` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-23 | 59 | 59/59 (100.0%) | 581 | 581/581 (100.0%) |
| `Layout/EmptyLinesAroundAccessModifier` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-23 | 176 | 176/176 (100.0%) | 1717 | 1717/1717 (100.0%) |
| `Layout/EmptyLinesAroundArguments` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 22 | 22/22 (100.0%) | 4 | 4/4 (100.0%) |
| `Layout/EmptyLinesAroundAttributeAccessor` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 122 | 122/122 (100.0%) |
| `Layout/EmptyLinesAroundBeginBody` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 0 | — (unexercised) |
| `Layout/EmptyLinesAroundBlockBody` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 40 | 40/40 (100.0%) |
| `Layout/EmptyLinesAroundClassBody` | [`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 46 | 46/46 (100.0%) | 26 | 26/26 (100.0%) |
| `Layout/EmptyLinesAroundExceptionHandlingKeywords` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 24 | 24/24 (100.0%) | 123 | 123/123 (100.0%) |
| `Layout/EmptyLinesAroundMethodBody` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 2 | 2/2 (100.0%) |
| `Layout/EmptyLinesAroundModuleBody` | [`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 37 | 37/37 (100.0%) | 13 | 13/13 (100.0%) |
| `Layout/EndOfLine` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Layout/FirstArgumentIndentation` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-23 | 139 | 139/139 (100.0%) | 628 | 628/628 (100.0%) |
| `Layout/FirstHashElementIndentation` | [`crates/rustocop/src/cops/prism/layout_qualification.rs`](../crates/rustocop/src/cops/prism/layout_qualification.rs) | 2026-08-23 | 60 | 60/60 (100.0%) | 4640 | 4640/4640 (100.0%) |
| `Layout/FirstParameterIndentation` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 0 | — (unexercised) |
| `Layout/HashAlignment` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-23 | 130 | 130/130 (100.0%) | 15158 | 15158/15158 (100.0%) |
| `Layout/HeredocArgumentClosingParenthesis` | [`crates/rustocop/src/cops/prism/heredoc_argument_closing_parenthesis_rules.rs`](../crates/rustocop/src/cops/prism/heredoc_argument_closing_parenthesis_rules.rs) | 2026-08-22 | 82 | 82/82 (100.0%) | 0 | — (unexercised) |
| `Layout/IndentationStyle` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 25 | 25/25 (100.0%) | 6 | 6/6 (100.0%) |
| `Layout/InitialIndentation` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 2 | 2/2 (100.0%) |
| `Layout/LeadingCommentSpace` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 27 | 27/27 (100.0%) | 212 | 212/212 (100.0%) |
| `Layout/LeadingEmptyLines` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 0 | — (unexercised) |
| `Layout/LineContinuationSpacing` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 31 | 31/31 (100.0%) | 211 | 211/211 (100.0%) |
| `Layout/MultilineMethodArgumentLineBreaks` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-22 | 18 | 18/18 (100.0%) | 30414 | 30414/30414 (100.0%) |
| `Layout/MultilineMethodDefinitionBraceLayout` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 26 | 26/26 (100.0%) | 107 | 107/107 (100.0%) |
| `Layout/MultilineMethodParameterLineBreaks` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-22 | 14 | 14/14 (100.0%) | 1221 | 1221/1221 (100.0%) |
| `Layout/ParameterAlignment` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-22 | 19 | 19/19 (100.0%) | 7 | 7/7 (100.0%) |
| `Layout/SpaceAfterColon` | [`crates/rustocop/src/cops/prism/layout.rs`](../crates/rustocop/src/cops/prism/layout.rs) | 2026-08-18 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Layout/SpaceAfterMethodName` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 0 | — (unexercised) |
| `Layout/SpaceAroundEqualsInParameterDefault` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 88 | 88/88 (100.0%) |
| `Layout/SpaceAroundKeyword` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 116 | 116/116 (100.0%) | 216 | 216/216 (100.0%) |
| `Layout/SpaceAroundMethodCallOperator` | [`crates/rustocop/src/cops/prism/operator_method_call_rules.rs`](../crates/rustocop/src/cops/prism/operator_method_call_rules.rs) | 2026-08-21 | 51 | 51/51 (100.0%) | 2 | 2/2 (100.0%) |
| `Layout/SpaceAroundOperators` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 99 | 99/99 (100.0%) | 3859 | 3859/3859 (100.0%) |
| `Layout/SpaceBeforeBrackets` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-23 | 28 | 28/28 (100.0%) | 0 | — (unexercised) |
| `Layout/SpaceBeforeFirstArg` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 4 | 4/4 (100.0%) |
| `Layout/SpaceInLambdaLiteral` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-23 | 15 | 15/15 (100.0%) | 125 | 125/125 (100.0%) |
| `Layout/SpaceInsideArrayLiteralBrackets` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-23 | 99 | 99/99 (100.0%) | 2069 | 2069/2069 (100.0%) |
| `Layout/SpaceInsideArrayPercentLiteral` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 129 | 129/129 (100.0%) | 25 | 25/25 (100.0%) |
| `Layout/SpaceInsideBlockBraces` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 42 | 42/42 (100.0%) | 549 | 549/549 (100.0%) |
| `Layout/SpaceInsideParens` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-23 | 28 | 28/28 (100.0%) | 50 | 50/50 (100.0%) |
| `Layout/SpaceInsidePercentLiteralDelimiters` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 262 | 262/262 (100.0%) | 815 | 815/815 (100.0%) |
| `Layout/SpaceInsideReferenceBrackets` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-23 | 47 | 47/47 (100.0%) | 1 | 1/1 (100.0%) |
| `Layout/SpaceInsideStringInterpolation` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 73 | 73/73 (100.0%) |
| `Layout/TrailingEmptyLines` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 12 | 12/12 (100.0%) |
| `Layout/TrailingWhitespace` | [`crates/rustocop/src/cops/text/layout.rs`](../crates/rustocop/src/cops/text/layout.rs) | 2026-08-22 | 19 | 19/19 (100.0%) | 47 | 47/47 (100.0%) |
| `Lint/AmbiguousAssignment` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-23 | 40 | 40/40 (100.0%) | 0 | — (unexercised) |
| `Lint/BigDecimalNew` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-23 | 3 | 3/3 (100.0%) | 0 | — (unexercised) |
| `Lint/BinaryOperatorWithIdenticalOperands` | [`crates/rustocop/src/cops/prism/lint_suspicious_calls.rs`](../crates/rustocop/src/cops/prism/lint_suspicious_calls.rs) | 2026-08-21 | 23 | 23/23 (100.0%) | 1369 | 1369/1369 (100.0%) |
| `Lint/BooleanSymbol` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 10 | 10/10 (100.0%) | 41 | 41/41 (100.0%) |
| `Lint/CircularArgumentReference` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Lint/ConstantDefinitionInBlock` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-23 | 27 | 27/27 (100.0%) | 284 | 284/284 (100.0%) |
| `Lint/ConstantOverwrittenInRescue` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 0 | — (unexercised) |
| `Lint/ConstantReassignment` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-23 | 57 | 57/57 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/DataDefineOverride` | [`crates/rustocop/src/cops/prism/lint_builtin_overrides.rs`](../crates/rustocop/src/cops/prism/lint_builtin_overrides.rs) | 2026-08-20 | 8 | 8/8 (100.0%) | 0 | — (unexercised) |
| `Lint/Debugger` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 97 | 97/97 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/DeprecatedClassMethods` | [`crates/rustocop/src/cops/prism/deprecated_api_rules.rs`](../crates/rustocop/src/cops/prism/deprecated_api_rules.rs) | 2026-08-18 | 31 | 31/31 (100.0%) | 0 | — (unexercised) |
| `Lint/DeprecatedConstants` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 12 | 12/12 (100.0%) |
| `Lint/DeprecatedOpenSSLConstant` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-23 | 24 | 24/24 (100.0%) | 6 | 6/6 (100.0%) |
| `Lint/DisjunctiveAssignmentInConstructor` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 4 | 4/4 (100.0%) |
| `Lint/DuplicateBranch` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-23 | 131 | 131/131 (100.0%) | 168 | 168/168 (100.0%) |
| `Lint/DuplicateCaseCondition` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-22 | 9 | 9/9 (100.0%) | 0 | — (unexercised) |
| `Lint/DuplicateElsifCondition` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Lint/DuplicateMagicComment` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 0 | — (unexercised) |
| `Lint/DuplicateMatchPattern` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-22 | 19 | 19/19 (100.0%) | 0 | — (unexercised) |
| `Lint/DuplicateMethods` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 475 | 475/475 (100.0%) | 13 | 13/13 (100.0%) |
| `Lint/DuplicateRequire` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/DuplicateSetElement` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-23 | 36 | 36/36 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/EachWithObjectArgument` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Lint/ElseLayout` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Lint/EmptyBlock` | [`crates/rustocop/src/cops/prism/lint_scope_completion.rs`](../crates/rustocop/src/cops/prism/lint_scope_completion.rs) | 2026-08-23 | 17 | 17/17 (100.0%) | 988 | 988/988 (100.0%) |
| `Lint/EmptyClass` | [`crates/rustocop/src/cops/prism/empty_class_rules.rs`](../crates/rustocop/src/cops/prism/empty_class_rules.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 41 | 41/41 (100.0%) |
| `Lint/EmptyConditionalBody` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-23 | 42 | 42/42 (100.0%) | 3 | 3/3 (100.0%) |
| `Lint/EmptyEnsure` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-23 | 2 | 2/2 (100.0%) | 0 | — (unexercised) |
| `Lint/EmptyExpression` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Lint/EmptyFile` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 4 | 4/4 (100.0%) | 16 | 16/16 (100.0%) |
| `Lint/EmptyInPattern` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-22 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Lint/EmptyInterpolation` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 2 | 2/2 (100.0%) |
| `Lint/EmptyWhen` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 15 | 15/15 (100.0%) |
| `Lint/EnsureReturn` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Lint/ErbNewArguments` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 2 | 2/2 (100.0%) |
| `Lint/FlipFlop` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 2 | 2/2 (100.0%) | 0 | — (unexercised) |
| `Lint/FloatComparison` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 17 | 17/17 (100.0%) | 117 | 117/117 (100.0%) |
| `Lint/FloatOutOfRange` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Lint/FormatParameterMismatch` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-23 | 75 | 75/75 (100.0%) | 0 | — (unexercised) |
| `Lint/HashCompareByIdentity` | [`crates/rustocop/src/cops/prism/lint_suspicious_calls.rs`](../crates/rustocop/src/cops/prism/lint_suspicious_calls.rs) | 2026-08-21 | 4 | 4/4 (100.0%) | 10 | 10/10 (100.0%) |
| `Lint/HashNewWithKeywordArgumentsAsDefault` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 0 | — (unexercised) |
| `Lint/HeredocMethodCallPosition` | [`crates/rustocop/src/cops/prism/heredoc_call_rules.rs`](../crates/rustocop/src/cops/prism/heredoc_call_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 0 | — (unexercised) |
| `Lint/IdentityComparison` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Lint/ImplicitStringConcatenation` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-20 | 12 | 12/12 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/IncompatibleIoSelectWithFiberScheduler` | [`crates/rustocop/src/cops/prism/io_scheduler_rules.rs`](../crates/rustocop/src/cops/prism/io_scheduler_rules.rs) | 2026-08-18 | 19 | 19/19 (100.0%) | 4 | 4/4 (100.0%) |
| `Lint/IneffectiveAccessModifier` | [`crates/rustocop/src/cops/prism/declaration_semantics.rs`](../crates/rustocop/src/cops/prism/declaration_semantics.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 360 | 360/360 (100.0%) |
| `Lint/InheritException` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 13 | 13/13 (100.0%) | 18 | 18/18 (100.0%) |
| `Lint/ItWithoutArgumentsInBlock` | [`crates/rustocop/src/cops/prism/block_parameter_rules.rs`](../crates/rustocop/src/cops/prism/block_parameter_rules.rs) | 2026-08-18 | 19 | 19/19 (100.0%) | 0 | — (unexercised) |
| `Lint/LambdaWithoutLiteralBlock` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 0 | — (unexercised) |
| `Lint/LiteralAsCondition` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-23 | 229 | 229/229 (100.0%) | 6 | 6/6 (100.0%) |
| `Lint/LiteralInInterpolation` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 378 | 378/378 (100.0%) | 26 | 26/26 (100.0%) |
| `Lint/Loop` | [`crates/rustocop/src/cops/prism/lint_control_flow.rs`](../crates/rustocop/src/cops/prism/lint_control_flow.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 20 | 20/20 (100.0%) |
| `Lint/MissingSuper` | [`crates/rustocop/src/cops/prism/lint_scope_completion.rs`](../crates/rustocop/src/cops/prism/lint_scope_completion.rs) | 2026-08-23 | 22 | 22/22 (100.0%) | 525 | 525/525 (100.0%) |
| `Lint/MixedCaseRange` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-22 | 31 | 31/31 (100.0%) | 12 | 12/12 (100.0%) |
| `Lint/MixedRegexpCaptureTypes` | [`crates/rustocop/src/cops/prism/lint_builtin_overrides.rs`](../crates/rustocop/src/cops/prism/lint_builtin_overrides.rs) | 2026-08-20 | 12 | 12/12 (100.0%) | 20 | 20/20 (100.0%) |
| `Lint/MultipleComparison` | [`crates/rustocop/src/cops/prism/logical_condition_rules.rs`](../crates/rustocop/src/cops/prism/logical_condition_rules.rs) | 2026-08-20 | 20 | 20/20 (100.0%) | 0 | — (unexercised) |
| `Lint/NestedMethodDefinition` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-23 | 38 | 38/38 (100.0%) | 2 | 2/2 (100.0%) |
| `Lint/NestedPercentLiteral` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/NextWithoutAccumulator` | [`crates/rustocop/src/cops/prism/block_arity_rules.rs`](../crates/rustocop/src/cops/prism/block_arity_rules.rs) | 2026-08-18 | 18 | 18/18 (100.0%) | 0 | — (unexercised) |
| `Lint/NonAtomicFileOperation` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-23 | 43 | 43/43 (100.0%) | 82 | 82/82 (100.0%) |
| `Lint/NonDeterministicRequireOrder` | [`crates/rustocop/src/cops/prism/non_deterministic_require_rules.rs`](../crates/rustocop/src/cops/prism/non_deterministic_require_rules.rs) | 2026-08-18 | 28 | 28/28 (100.0%) | 0 | — (unexercised) |
| `Lint/NonLocalExitFromIterator` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 14 | 14/14 (100.0%) | 6 | 6/6 (100.0%) |
| `Lint/NumberConversion` | [`crates/rustocop/src/cops/prism/number_conversion_rules.rs`](../crates/rustocop/src/cops/prism/number_conversion_rules.rs) | 2026-08-21 | 37 | 37/37 (100.0%) | 6776 | 6776/6776 (100.0%) |
| `Lint/NumberedParameterAssignment` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Lint/NumericOperationWithConstantResult` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 0 | — (unexercised) |
| `Lint/OrAssignmentToConstant` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/OrderedMagicComments` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/OutOfRangeRegexpRef` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-23 | 122 | 122/122 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/ParenthesesAsGroupedExpression` | [`crates/rustocop/src/cops/prism/operator_ambiguity_rules.rs`](../crates/rustocop/src/cops/prism/operator_ambiguity_rules.rs) | 2026-08-21 | 29 | 29/29 (100.0%) | 13 | 13/13 (100.0%) |
| `Lint/PercentStringArray` | [`crates/rustocop/src/cops/prism/percent_string_rules.rs`](../crates/rustocop/src/cops/prism/percent_string_rules.rs) | 2026-08-23 | 22 | 22/22 (100.0%) | 7 | 7/7 (100.0%) |
| `Lint/PercentSymbolArray` | [`crates/rustocop/src/cops/prism/percent_string_rules.rs`](../crates/rustocop/src/cops/prism/percent_string_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Lint/RaiseException` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 15 | 15/15 (100.0%) | 12 | 12/12 (100.0%) |
| `Lint/RandOne` | [`crates/rustocop/src/cops/prism/lint_suspicious_calls.rs`](../crates/rustocop/src/cops/prism/lint_suspicious_calls.rs) | 2026-08-21 | 16 | 16/16 (100.0%) | 0 | — (unexercised) |
| `Lint/RedundantDirGlobSort` | [`crates/rustocop/src/cops/prism/path_and_literal_rules.rs`](../crates/rustocop/src/cops/prism/path_and_literal_rules.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 19 | 19/19 (100.0%) |
| `Lint/RedundantRegexpQuantifiers` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-23 | 26 | 26/26 (100.0%) | 0 | — (unexercised) |
| `Lint/RedundantRequireStatement` | [`crates/rustocop/src/cops/prism/require_rules.rs`](../crates/rustocop/src/cops/prism/require_rules.rs) | 2026-08-18 | 15 | 15/15 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/RedundantSafeNavigation` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-23 | 92 | 92/92 (100.0%) | 0 | — (unexercised) |
| `Lint/RedundantSplatExpansion` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-23 | 59 | 59/59 (100.0%) | 22 | 22/22 (100.0%) |
| `Lint/RedundantStringCoercion` | [`crates/rustocop/src/cops/prism/coercion_rules.rs`](../crates/rustocop/src/cops/prism/coercion_rules.rs) | 2026-08-18 | 18 | 18/18 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/RedundantTypeConversion` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 613 | 613/613 (100.0%) | 13 | 13/13 (100.0%) |
| `Lint/RedundantWithIndex` | [`crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs`](../crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs) | 2026-08-22 | 17 | 17/17 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/RedundantWithObject` | [`crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs`](../crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs) | 2026-08-22 | 14 | 14/14 (100.0%) | 0 | — (unexercised) |
| `Lint/RefinementImportMethods` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Lint/RegexpAsCondition` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Lint/RequireParentheses` | [`crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs`](../crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 0 | — (unexercised) |
| `Lint/RequireRangeParentheses` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 0 | — (unexercised) |
| `Lint/RequireRelativeSelfPath` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 0 | — (unexercised) |
| `Lint/RescueException` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 78 | 78/78 (100.0%) |
| `Lint/ReturnInVoidContext` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-23 | 18 | 18/18 (100.0%) | 7 | 7/7 (100.0%) |
| `Lint/SafeNavigationChain` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-23 | 66 | 66/66 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/SafeNavigationConsistency` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-23 | 43 | 43/43 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/SafeNavigationWithEmpty` | [`crates/rustocop/src/cops/prism/lint_control_flow.rs`](../crates/rustocop/src/cops/prism/lint_control_flow.rs) | 2026-08-20 | 3 | 3/3 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/ScriptPermission` | [`crates/rustocop/src/cops/prism/final_file_metadata_batch.rs`](../crates/rustocop/src/cops/prism/final_file_metadata_batch.rs) | 2026-08-22 | 6 | 6/6 (100.0%) | 3 | 3/3 (100.0%) |
| `Lint/SelfAssignment` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 58 | 58/58 (100.0%) | 13 | 13/13 (100.0%) |
| `Lint/SendWithMixinArgument` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 4 | 4/4 (100.0%) |
| `Lint/ShadowedArgument` | [`crates/rustocop/src/cops/prism/final_scope_batch_a/naming.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a/naming.rs) | 2026-08-23 | 54 | 54/54 (100.0%) | 4 | 4/4 (100.0%) |
| `Lint/ShadowedException` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-23 | 38 | 38/38 (100.0%) | 17 | 17/17 (100.0%) |
| `Lint/ShadowingOuterLocalVariable` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-23 | 31 | 31/31 (100.0%) | 263 | 263/263 (100.0%) |
| `Lint/SharedMutableDefault` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 7 | 7/7 (100.0%) |
| `Lint/StructNewOverride` | [`crates/rustocop/src/cops/prism/lint_builtin_overrides.rs`](../crates/rustocop/src/cops/prism/lint_builtin_overrides.rs) | 2026-08-20 | 10 | 10/10 (100.0%) | 14 | 14/14 (100.0%) |
| `Lint/SuppressedException` | [`crates/rustocop/src/cops/prism/rescue_rules.rs`](../crates/rustocop/src/cops/prism/rescue_rules.rs) | 2026-08-21 | 24 | 24/24 (100.0%) | 218 | 218/218 (100.0%) |
| `Lint/SuppressedExceptionInNumberConversion` | [`crates/rustocop/src/cops/prism/exception_location_completion.rs`](../crates/rustocop/src/cops/prism/exception_location_completion.rs) | 2026-08-20 | 26 | 26/26 (100.0%) | 3 | 3/3 (100.0%) |
| `Lint/Syntax` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-23 | 4 | 4/4 (100.0%) | 44 | 44/44 (100.0%) |
| `Lint/ToEnumArguments` | [`crates/rustocop/src/cops/prism/enum_argument_rules.rs`](../crates/rustocop/src/cops/prism/enum_argument_rules.rs) | 2026-08-20 | 24 | 24/24 (100.0%) | 5 | 5/5 (100.0%) |
| `Lint/ToJSON` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-22 | 2 | 2/2 (100.0%) | 8 | 8/8 (100.0%) |
| `Lint/TrailingCommaInAttributeDeclaration` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-23 | 2 | 2/2 (100.0%) | 0 | — (unexercised) |
| `Lint/TripleQuotes` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 0 | — (unexercised) |
| `Lint/UnderscorePrefixedVariableName` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-23 | 19 | 19/19 (100.0%) | 97 | 97/97 (100.0%) |
| `Lint/UnescapedBracketInRegexp` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-23 | 44 | 44/44 (100.0%) | 0 | — (unexercised) |
| `Lint/UnexpectedBlockArity` | [`crates/rustocop/src/cops/prism/block_arity_rules.rs`](../crates/rustocop/src/cops/prism/block_arity_rules.rs) | 2026-08-18 | 22 | 22/22 (100.0%) | 10 | 10/10 (100.0%) |
| `Lint/UnifiedInteger` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-22 | 15 | 15/15 (100.0%) | 0 | — (unexercised) |
| `Lint/UnmodifiedReduceAccumulator` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-23 | 168 | 168/168 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/UnreachableCode` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-23 | 266 | 266/266 (100.0%) | 4 | 4/4 (100.0%) |
| `Lint/UnreachableLoop` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-23 | 28 | 28/28 (100.0%) | 4 | 4/4 (100.0%) |
| `Lint/UnreachablePatternBranch` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-23 | 23 | 23/23 (100.0%) | 0 | — (unexercised) |
| `Lint/UnusedBlockArgument` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-23 | 30 | 30/30 (100.0%) | 1608 | 1608/1608 (100.0%) |
| `Lint/UriEscapeUnescape` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 0 | — (unexercised) |
| `Lint/UriRegexp` | [`crates/rustocop/src/cops/prism/uri_regexp_rules.rs`](../crates/rustocop/src/cops/prism/uri_regexp_rules.rs) | 2026-08-21 | 10 | 10/10 (100.0%) | 7 | 7/7 (100.0%) |
| `Lint/UselessAccessModifier` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 198 | 198/198 (100.0%) | 100 | 100/100 (100.0%) |
| `Lint/UselessDefaultValueArgument` | [`crates/rustocop/src/cops/prism/fetch_completion_rules.rs`](../crates/rustocop/src/cops/prism/fetch_completion_rules.rs) | 2026-08-22 | 25 | 25/25 (100.0%) | 4 | 4/4 (100.0%) |
| `Lint/UselessDefined` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Lint/UselessElseWithoutRescue` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-23 | 2 | 2/2 (100.0%) | 0 | — (unexercised) |
| `Lint/UselessMethodDefinition` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 19 | 19/19 (100.0%) |
| `Lint/UselessNumericOperation` | [`crates/rustocop/src/cops/prism/numeric_operation_rules.rs`](../crates/rustocop/src/cops/prism/numeric_operation_rules.rs) | 2026-08-18 | 13 | 13/13 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/UselessOr` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-23 | 127 | 127/127 (100.0%) | 29 | 29/29 (100.0%) |
| `Lint/UselessRescue` | [`crates/rustocop/src/cops/prism/rescue_rules.rs`](../crates/rustocop/src/cops/prism/rescue_rules.rs) | 2026-08-21 | 12 | 12/12 (100.0%) | 2 | 2/2 (100.0%) |
| `Lint/UselessRuby2Keywords` | [`crates/rustocop/src/cops/prism/ruby2_keywords_rules.rs`](../crates/rustocop/src/cops/prism/ruby2_keywords_rules.rs) | 2026-08-18 | 23 | 23/23 (100.0%) | 0 | — (unexercised) |
| `Lint/UselessSetterCall` | [`crates/rustocop/src/cops/prism/setter_rules.rs`](../crates/rustocop/src/cops/prism/setter_rules.rs) | 2026-08-18 | 20 | 20/20 (100.0%) | 0 | — (unexercised) |
| `Lint/UselessTimes` | [`crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs`](../crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs) | 2026-08-22 | 25 | 25/25 (100.0%) | 10 | 10/10 (100.0%) |
| `Lint/Void` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 270 | 270/270 (100.0%) | 7 | 7/7 (100.0%) |
| `Metrics/CollectionLiteralLength` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 9 | 9/9 (100.0%) |
| `Metrics/ModuleLength` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 685 | 685/685 (100.0%) |
| `Metrics/ParameterLists` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 896 | 896/896 (100.0%) |
| `Migration/DepartmentName` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 0 | — (unexercised) |
| `Naming/AccessorMethodName` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-23 | 23 | 23/23 (100.0%) | 524 | 524/524 (100.0%) |
| `Naming/AsciiIdentifiers` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 10 | 10/10 (100.0%) |
| `Naming/BinaryOperatorParameterName` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-23 | 15 | 15/15 (100.0%) | 36 | 36/36 (100.0%) |
| `Naming/BlockForwarding` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-23 | 36 | 36/36 (100.0%) | 2997 | 2997/2997 (100.0%) |
| `Naming/ClassAndModuleCamelCase` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 55 | 55/55 (100.0%) |
| `Naming/FileName` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-23 | 120 | 120/120 (100.0%) | 393 | 393/393 (100.0%) |
| `Naming/HeredocDelimiterCase` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-23 | 26 | 26/26 (100.0%) | 38 | 38/38 (100.0%) |
| `Naming/InclusiveLanguage` | [`crates/rustocop/src/cops/prism/final_scope_batch_a/naming.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a/naming.rs) | 2026-08-23 | 35 | 35/35 (100.0%) | 193 | 193/193 (100.0%) |
| `Naming/MemoizedInstanceVariableName` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-23 | 72 | 72/72 (100.0%) | 426 | 426/426 (100.0%) |
| `Naming/MethodName` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-23 | 239 | 239/239 (100.0%) | 373 | 373/373 (100.0%) |
| `Naming/MethodParameterName` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-23 | 23 | 23/23 (100.0%) | 858 | 858/858 (100.0%) |
| `Naming/PredicateMethod` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-23 | 1262 | 1262/1262 (100.0%) | 1560 | 1560/1560 (100.0%) |
| `Naming/PredicatePrefix` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-23 | 24 | 24/24 (100.0%) | 2837 | 2837/2837 (100.0%) |
| `Naming/RescuedExceptionsVariableName` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-23 | 36 | 36/36 (100.0%) | 592 | 592/592 (100.0%) |
| `Security/CompoundHash` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 21 | 21/21 (100.0%) | 6 | 6/6 (100.0%) |
| `Security/Eval` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 15 | 15/15 (100.0%) | 54 | 54/54 (100.0%) |
| `Security/IoMethods` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 32 | 32/32 (100.0%) | 8 | 8/8 (100.0%) |
| `Security/JSONLoad` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 7 | 7/7 (100.0%) | 46 | 46/46 (100.0%) |
| `Security/MarshalLoad` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 5 | 5/5 (100.0%) | 34 | 34/34 (100.0%) |
| `Security/Open` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 16 | 16/16 (100.0%) | 4 | 4/4 (100.0%) |
| `Security/YAMLLoad` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 6 | 6/6 (100.0%) | 0 | — (unexercised) |
| `Style/AccessModifierDeclarations` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 393 | 393/393 (100.0%) | 8 | 8/8 (100.0%) |
| `Style/Alias` | [`crates/rustocop/src/cops/prism/alias_rules.rs`](../crates/rustocop/src/cops/prism/alias_rules.rs) | 2026-08-21 | 31 | 31/31 (100.0%) | 1236 | 1236/1236 (100.0%) |
| `Style/AmbiguousEndlessMethodDefinition` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-23 | 31 | 31/31 (100.0%) | 0 | — (unexercised) |
| `Style/AndOr` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-23 | 78 | 78/78 (100.0%) | 0 | — (unexercised) |
| `Style/ArgumentsForwarding` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 187 | 187/187 (100.0%) | 4834 | 4834/4834 (100.0%) |
| `Style/ArrayCoercion` | [`crates/rustocop/src/cops/prism/structural_forwarding_completion.rs`](../crates/rustocop/src/cops/prism/structural_forwarding_completion.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 45 | 45/45 (100.0%) |
| `Style/ArrayFirstLast` | [`crates/rustocop/src/cops/prism/style_collections.rs`](../crates/rustocop/src/cops/prism/style_collections.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 3484 | 3484/3484 (100.0%) |
| `Style/ArrayIntersect` | [`crates/rustocop/src/cops/prism/modern_collection_completion.rs`](../crates/rustocop/src/cops/prism/modern_collection_completion.rs) | 2026-08-22 | 81 | 81/81 (100.0%) | 76 | 76/76 (100.0%) |
| `Style/ArrayIntersectWithSingleElement` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 4 | 4/4 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/ArrayJoin` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 15 | 15/15 (100.0%) |
| `Style/AsciiComments` | [`crates/rustocop/src/cops/prism/source_rules_misc.rs`](../crates/rustocop/src/cops/prism/source_rules_misc.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 609 | 609/609 (100.0%) |
| `Style/Attr` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-22 | 11 | 11/11 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/BarePercentLiterals` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 36 | 36/36 (100.0%) | 184 | 184/184 (100.0%) |
| `Style/BeginBlock` | [`crates/rustocop/src/cops/prism/style/misc.rs`](../crates/rustocop/src/cops/prism/style/misc.rs) | 2026-08-19 | 1 | 1/1 (100.0%) | 0 | — (unexercised) |
| `Style/BisectedAttrAccessor` | [`crates/rustocop/src/cops/prism/accessor_rules.rs`](../crates/rustocop/src/cops/prism/accessor_rules.rs) | 2026-08-18 | 14 | 14/14 (100.0%) | 8 | 8/8 (100.0%) |
| `Style/BitwisePredicate` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 18 | 18/18 (100.0%) | 11 | 11/11 (100.0%) |
| `Style/BlockComments` | [`crates/rustocop/src/cops/prism/block_comments_rules.rs`](../crates/rustocop/src/cops/prism/block_comments_rules.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 27 | 27/27 (100.0%) |
| `Style/BlockDelimiters` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-23 | 173 | 173/173 (100.0%) | 2831 | 2831/2831 (100.0%) |
| `Style/CaseEquality` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 25 | 25/25 (100.0%) | 630 | 630/630 (100.0%) |
| `Style/CaseLikeIf` | [`crates/rustocop/src/cops/prism/structural_next_completion.rs`](../crates/rustocop/src/cops/prism/structural_next_completion.rs) | 2026-08-23 | 38 | 38/38 (100.0%) | 27 | 27/27 (100.0%) |
| `Style/CharacterLiteral` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 11 | 11/11 (100.0%) |
| `Style/ClassAndModuleChildren` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-23 | 44 | 44/44 (100.0%) | 3247 | 3247/3247 (100.0%) |
| `Style/ClassCheck` | [`crates/rustocop/src/cops/prism/class_check_rules.rs`](../crates/rustocop/src/cops/prism/class_check_rules.rs) | 2026-08-21 | 4 | 4/4 (100.0%) | 56 | 56/56 (100.0%) |
| `Style/ClassEqualityComparison` | [`crates/rustocop/src/cops/prism/class_comparison_rules.rs`](../crates/rustocop/src/cops/prism/class_comparison_rules.rs) | 2026-08-21 | 22 | 22/22 (100.0%) | 24 | 24/24 (100.0%) |
| `Style/ClassMethods` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/ClassMethodsDefinitions` | [`crates/rustocop/src/cops/prism/class_methods_completion.rs`](../crates/rustocop/src/cops/prism/class_methods_completion.rs) | 2026-08-21 | 16 | 16/16 (100.0%) | 631 | 631/631 (100.0%) |
| `Style/ClassVars` | [`crates/rustocop/src/cops/prism/class_vars_rules.rs`](../crates/rustocop/src/cops/prism/class_vars_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 298 | 298/298 (100.0%) |
| `Style/CollectionCompact` | [`crates/rustocop/src/cops/prism/collection_completion_rules.rs`](../crates/rustocop/src/cops/prism/collection_completion_rules.rs) | 2026-08-22 | 35 | 35/35 (100.0%) | 8 | 8/8 (100.0%) |
| `Style/CollectionMethods` | [`crates/rustocop/src/cops/prism/collection_transform_batch.rs`](../crates/rustocop/src/cops/prism/collection_transform_batch.rs) | 2026-08-21 | 68 | 68/68 (100.0%) | 667 | 667/667 (100.0%) |
| `Style/CollectionQuerying` | [`crates/rustocop/src/cops/prism/collection_query_rules.rs`](../crates/rustocop/src/cops/prism/collection_query_rules.rs) | 2026-08-18 | 20 | 20/20 (100.0%) | 125 | 125/125 (100.0%) |
| `Style/ColonMethodCall` | [`crates/rustocop/src/cops/prism/style_calls.rs`](../crates/rustocop/src/cops/prism/style_calls.rs) | 2026-08-18 | 10 | 10/10 (100.0%) | 9 | 9/9 (100.0%) |
| `Style/ColonMethodDefinition` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-23 | 3 | 3/3 (100.0%) | 0 | — (unexercised) |
| `Style/CombinableDefined` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-23 | 39 | 39/39 (100.0%) | 2 | 2/2 (100.0%) |
| `Style/CombinableLoops` | [`crates/rustocop/src/cops/prism/control_flow_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_flow_completion_batch.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 55 | 55/55 (100.0%) |
| `Style/CommandLiteral` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-23 | 35 | 35/35 (100.0%) | 2 | 2/2 (100.0%) |
| `Style/CommentAnnotation` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-23 | 31 | 31/31 (100.0%) | 295 | 295/295 (100.0%) |
| `Style/CommentedKeyword` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-23 | 47 | 47/47 (100.0%) | 230 | 230/230 (100.0%) |
| `Style/ComparableBetween` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 15 | 15/15 (100.0%) | 16 | 16/16 (100.0%) |
| `Style/ComparableClamp` | [`crates/rustocop/src/cops/prism/comparable_clamp_rules.rs`](../crates/rustocop/src/cops/prism/comparable_clamp_rules.rs) | 2026-08-18 | 23 | 23/23 (100.0%) | 8 | 8/8 (100.0%) |
| `Style/ConcatArrayLiterals` | [`crates/rustocop/src/cops/prism/collection_completion_rules.rs`](../crates/rustocop/src/cops/prism/collection_completion_rules.rs) | 2026-08-22 | 15 | 15/15 (100.0%) | 74 | 74/74 (100.0%) |
| `Style/ConditionalAssignment` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 1199 | 1199/1199 (100.0%) | 316 | 316/316 (100.0%) |
| `Style/DataInheritance` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-22 | 24 | 24/24 (100.0%) | 0 | — (unexercised) |
| `Style/DateTime` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 12 | 12/12 (100.0%) | 1249 | 1249/1249 (100.0%) |
| `Style/DefWithParentheses` | [`crates/rustocop/src/cops/prism/declaration_semantics.rs`](../crates/rustocop/src/cops/prism/declaration_semantics.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/DigChain` | [`crates/rustocop/src/cops/prism/dig_rules.rs`](../crates/rustocop/src/cops/prism/dig_rules.rs) | 2026-08-18 | 23 | 23/23 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/Dir` | [`crates/rustocop/src/cops/prism/dir_rules.rs`](../crates/rustocop/src/cops/prism/dir_rules.rs) | 2026-08-21 | 4 | 4/4 (100.0%) | 5 | 5/5 (100.0%) |
| `Style/DirEmpty` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 16 | 16/16 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/DisableCopsWithinSourceCodeDirective` | [`crates/rustocop/src/cops/prism/lexical_completion.rs`](../crates/rustocop/src/cops/prism/lexical_completion.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 8468 | 8468/8468 (100.0%) |
| `Style/DocumentDynamicEvalDefinition` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 90 | 90/90 (100.0%) |
| `Style/DoubleCopDisableDirective` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-23 | 3 | 3/3 (100.0%) | 0 | — (unexercised) |
| `Style/DoubleNegation` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-23 | 47 | 47/47 (100.0%) | 212 | 212/212 (100.0%) |
| `Style/EachForSimpleLoop` | [`crates/rustocop/src/cops/prism/control_flow_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_flow_completion_batch.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 0 | — (unexercised) |
| `Style/EachWithObject` | [`crates/rustocop/src/cops/prism/collection_completion_rules.rs`](../crates/rustocop/src/cops/prism/collection_completion_rules.rs) | 2026-08-22 | 16 | 16/16 (100.0%) | 61 | 61/61 (100.0%) |
| `Style/EmptyBlockParameter` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 0 | — (unexercised) |
| `Style/EmptyCaseCondition` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-22 | 29 | 29/29 (100.0%) | 21 | 21/21 (100.0%) |
| `Style/EmptyClassDefinition` | [`crates/rustocop/src/cops/prism/class_definition_rules.rs`](../crates/rustocop/src/cops/prism/class_definition_rules.rs) | 2026-08-18 | 52 | 52/52 (100.0%) | 946 | 946/946 (100.0%) |
| `Style/EmptyElse` | [`crates/rustocop/src/cops/prism/empty_else_rules.rs`](../crates/rustocop/src/cops/prism/empty_else_rules.rs) | 2026-08-23 | 124 | 124/124 (100.0%) | 97 | 97/97 (100.0%) |
| `Style/EmptyHeredoc` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 2 | 2/2 (100.0%) |
| `Style/EmptyLambdaParameter` | [`crates/rustocop/src/cops/prism/empty_lambda_parameter_rules.rs`](../crates/rustocop/src/cops/prism/empty_lambda_parameter_rules.rs) | 2026-08-21 | 3 | 3/3 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/EmptyLiteral` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-23 | 49 | 49/49 (100.0%) | 56 | 56/56 (100.0%) |
| `Style/EmptyMethod` | [`crates/rustocop/src/cops/prism/empty_method_rules.rs`](../crates/rustocop/src/cops/prism/empty_method_rules.rs) | 2026-08-22 | 32 | 32/32 (100.0%) | 450 | 450/450 (100.0%) |
| `Style/EmptyStringInsideInterpolation` | [`crates/rustocop/src/cops/prism/interpolation_condition_rules.rs`](../crates/rustocop/src/cops/prism/interpolation_condition_rules.rs) | 2026-08-23 | 24 | 24/24 (100.0%) | 137 | 137/137 (100.0%) |
| `Style/Encoding` | [`crates/rustocop/src/cops/prism/lexical_completion.rs`](../crates/rustocop/src/cops/prism/lexical_completion.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 42 | 42/42 (100.0%) |
| `Style/EndBlock` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-23 | 2 | 2/2 (100.0%) | 0 | — (unexercised) |
| `Style/EndlessMethod` | [`crates/rustocop/src/cops/prism/endless_method_rules.rs`](../crates/rustocop/src/cops/prism/endless_method_rules.rs) | 2026-08-22 | 63 | 63/63 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/EnvHome` | [`crates/rustocop/src/cops/prism/source_rules_misc.rs`](../crates/rustocop/src/cops/prism/source_rules_misc.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/EvenOdd` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 18 | 18/18 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/ExactRegexpMatch` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 0 | — (unexercised) |
| `Style/ExpandPathArguments` | [`crates/rustocop/src/cops/prism/path_and_literal_rules.rs`](../crates/rustocop/src/cops/prism/path_and_literal_rules.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 69 | 69/69 (100.0%) |
| `Style/ExponentialNotation` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-22 | 27 | 27/27 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/FileEmpty` | [`crates/rustocop/src/cops/prism/file_predicate_rules.rs`](../crates/rustocop/src/cops/prism/file_predicate_rules.rs) | 2026-08-20 | 27 | 27/27 (100.0%) | 7 | 7/7 (100.0%) |
| `Style/FileNull` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-22 | 13 | 13/13 (100.0%) | 49 | 49/49 (100.0%) |
| `Style/FileOpen` | [`crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs`](../crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 42 | 42/42 (100.0%) |
| `Style/FileRead` | [`crates/rustocop/src/cops/prism/compact_syntax_completion.rs`](../crates/rustocop/src/cops/prism/compact_syntax_completion.rs) | 2026-08-23 | 30 | 30/30 (100.0%) | 13 | 13/13 (100.0%) |
| `Style/FileTouch` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 0 | — (unexercised) |
| `Style/FloatDivision` | [`crates/rustocop/src/cops/prism/numeric_operation_rules.rs`](../crates/rustocop/src/cops/prism/numeric_operation_rules.rs) | 2026-08-18 | 31 | 31/31 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/For` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-23 | 32 | 32/32 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/FormatString` | [`crates/rustocop/src/cops/prism/format_string_rules.rs`](../crates/rustocop/src/cops/prism/format_string_rules.rs) | 2026-08-20 | 46 | 46/46 (100.0%) | 543 | 543/543 (100.0%) |
| `Style/FormatStringToken` | [`crates/rustocop/src/cops/prism/format_string_token_rules.rs`](../crates/rustocop/src/cops/prism/format_string_token_rules.rs) | 2026-08-22 | 366 | 366/366 (100.0%) | 3581 | 3581/3581 (100.0%) |
| `Style/FrozenStringLiteralComment` | [`crates/rustocop/src/cops/prism/frozen_string_literal_comment_rules.rs`](../crates/rustocop/src/cops/prism/frozen_string_literal_comment_rules.rs) | 2026-08-20 | 107 | 107/107 (100.0%) | 2315 | 2315/2315 (100.0%) |
| `Style/GlobalStdStream` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 6 | 6/6 (100.0%) | 182 | 182/182 (100.0%) |
| `Style/GlobalVars` | [`crates/rustocop/src/cops/prism/style_global_vars.rs`](../crates/rustocop/src/cops/prism/style_global_vars.rs) | 2026-08-21 | 74 | 74/74 (100.0%) | 351 | 351/351 (100.0%) |
| `Style/GuardClause` | [`crates/rustocop/src/cops/prism/guard_clause_rules.rs`](../crates/rustocop/src/cops/prism/guard_clause_rules.rs) | 2026-08-23 | 91 | 91/91 (100.0%) | 2111 | 2111/2111 (100.0%) |
| `Style/HashAsLastArrayItem` | [`crates/rustocop/src/cops/prism/hash_array_rules.rs`](../crates/rustocop/src/cops/prism/hash_array_rules.rs) | 2026-08-18 | 19 | 19/19 (100.0%) | 460 | 460/460 (100.0%) |
| `Style/HashConversion` | [`crates/rustocop/src/cops/prism/hash_conversion_rules.rs`](../crates/rustocop/src/cops/prism/hash_conversion_rules.rs) | 2026-08-20 | 24 | 24/24 (100.0%) | 88 | 88/88 (100.0%) |
| `Style/HashEachMethods` | [`crates/rustocop/src/cops/prism/hash_each_methods_rules.rs`](../crates/rustocop/src/cops/prism/hash_each_methods_rules.rs) | 2026-08-22 | 62 | 62/62 (100.0%) | 219 | 219/219 (100.0%) |
| `Style/HashExcept` | [`crates/rustocop/src/cops/prism/hash_subset_rules.rs`](../crates/rustocop/src/cops/prism/hash_subset_rules.rs) | 2026-08-20 | 114 | 114/114 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/HashFetchChain` | [`crates/rustocop/src/cops/prism/hash_fetch_chain_rules.rs`](../crates/rustocop/src/cops/prism/hash_fetch_chain_rules.rs) | 2026-08-20 | 35 | 35/35 (100.0%) | 4 | 4/4 (100.0%) |
| `Style/HashLikeCase` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-22 | 8 | 8/8 (100.0%) | 32 | 32/32 (100.0%) |
| `Style/HashLookupMethod` | [`crates/rustocop/src/cops/prism/lookup_completion_rules.rs`](../crates/rustocop/src/cops/prism/lookup_completion_rules.rs) | 2026-08-21 | 18 | 18/18 (100.0%) | 2470 | 2470/2470 (100.0%) |
| `Style/HashSlice` | [`crates/rustocop/src/cops/prism/hash_subset_rules.rs`](../crates/rustocop/src/cops/prism/hash_subset_rules.rs) | 2026-08-20 | 116 | 116/116 (100.0%) | 22 | 22/22 (100.0%) |
| `Style/HashSyntax` | [`crates/rustocop/src/cops/prism/hash_syntax_rules.rs`](../crates/rustocop/src/cops/prism/hash_syntax_rules.rs) | 2026-08-21 | 189 | 189/189 (100.0%) | 1658 | 1658/1658 (100.0%) |
| `Style/HashTransformKeys` | [`crates/rustocop/src/cops/prism/hash_transform_rules.rs`](../crates/rustocop/src/cops/prism/hash_transform_rules.rs) | 2026-08-20 | 40 | 40/40 (100.0%) | 0 | — (unexercised) |
| `Style/HashTransformValues` | [`crates/rustocop/src/cops/prism/hash_transform_rules.rs`](../crates/rustocop/src/cops/prism/hash_transform_rules.rs) | 2026-08-20 | 40 | 40/40 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/IdenticalConditionalBranches` | [`crates/rustocop/src/cops/prism/identical_conditional_branches_rules.rs`](../crates/rustocop/src/cops/prism/identical_conditional_branches_rules.rs) | 2026-08-23 | 48 | 48/48 (100.0%) | 75 | 75/75 (100.0%) |
| `Style/IfInsideElse` | [`crates/rustocop/src/cops/prism/structural_next_completion.rs`](../crates/rustocop/src/cops/prism/structural_next_completion.rs) | 2026-08-23 | 23 | 23/23 (100.0%) | 107 | 107/107 (100.0%) |
| `Style/IfUnlessModifierOfIfUnless` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 7 | 7/7 (100.0%) | 2 | 2/2 (100.0%) |
| `Style/IfWithBooleanLiteralBranches` | [`crates/rustocop/src/cops/prism/if_with_boolean_literal_branches_rules.rs`](../crates/rustocop/src/cops/prism/if_with_boolean_literal_branches_rules.rs) | 2026-08-21 | 94 | 94/94 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/IfWithSemicolon` | [`crates/rustocop/src/cops/prism/if_with_semicolon_rules.rs`](../crates/rustocop/src/cops/prism/if_with_semicolon_rules.rs) | 2026-08-23 | 35 | 35/35 (100.0%) | 0 | — (unexercised) |
| `Style/InPatternThen` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Style/InfiniteLoop` | [`crates/rustocop/src/cops/prism/infinite_loop_rules.rs`](../crates/rustocop/src/cops/prism/infinite_loop_rules.rs) | 2026-08-20 | 28 | 28/28 (100.0%) | 21 | 21/21 (100.0%) |
| `Style/InverseMethods` | [`crates/rustocop/src/cops/prism/inverse_methods_rules.rs`](../crates/rustocop/src/cops/prism/inverse_methods_rules.rs) | 2026-08-20 | 110 | 110/110 (100.0%) | 98 | 98/98 (100.0%) |
| `Style/InvertibleUnlessCondition` | [`crates/rustocop/src/cops/prism/invertible_unless_condition_rules.rs`](../crates/rustocop/src/cops/prism/invertible_unless_condition_rules.rs) | 2026-08-23 | 15 | 15/15 (100.0%) | 376 | 376/376 (100.0%) |
| `Style/IpAddresses` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 1724 | 1724/1724 (100.0%) |
| `Style/ItAssignment` | [`crates/rustocop/src/cops/prism/parameter_order_completion.rs`](../crates/rustocop/src/cops/prism/parameter_order_completion.rs) | 2026-08-22 | 23 | 23/23 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/ItBlockParameter` | [`crates/rustocop/src/cops/prism/it_parameter_rules.rs`](../crates/rustocop/src/cops/prism/it_parameter_rules.rs) | 2026-08-18 | 34 | 34/34 (100.0%) | 125 | 125/125 (100.0%) |
| `Style/KeywordArgumentsMerging` | [`crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs`](../crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 104 | 104/104 (100.0%) |
| `Style/KeywordParametersOrder` | [`crates/rustocop/src/cops/prism/parameter_order_completion.rs`](../crates/rustocop/src/cops/prism/parameter_order_completion.rs) | 2026-08-22 | 10 | 10/10 (100.0%) | 255 | 255/255 (100.0%) |
| `Style/Lambda` | [`crates/rustocop/src/cops/prism/lambda_rules.rs`](../crates/rustocop/src/cops/prism/lambda_rules.rs) | 2026-08-20 | 38 | 38/38 (100.0%) | 2073 | 2073/2073 (100.0%) |
| `Style/LambdaCall` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 19 | 19/19 (100.0%) | 37 | 37/37 (100.0%) |
| `Style/LineEndConcatenation` | [`crates/rustocop/src/cops/prism/line_concatenation_rules.rs`](../crates/rustocop/src/cops/prism/line_concatenation_rules.rs) | 2026-08-21 | 19 | 19/19 (100.0%) | 78 | 78/78 (100.0%) |
| `Style/MagicCommentFormat` | [`crates/rustocop/src/cops/prism/magic_comment_format_rules.rs`](../crates/rustocop/src/cops/prism/magic_comment_format_rules.rs) | 2026-08-20 | 25 | 25/25 (100.0%) | 0 | — (unexercised) |
| `Style/MapCompactWithConditionalBlock` | [`crates/rustocop/src/cops/prism/map_compact_conditional_rules.rs`](../crates/rustocop/src/cops/prism/map_compact_conditional_rules.rs) | 2026-08-21 | 33 | 33/33 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/MapIntoArray` | [`crates/rustocop/src/cops/prism/map_into_array_rules.rs`](../crates/rustocop/src/cops/prism/map_into_array_rules.rs) | 2026-08-23 | 64 | 64/64 (100.0%) | 36 | 36/36 (100.0%) |
| `Style/MapJoin` | [`crates/rustocop/src/cops/prism/map_join_rules.rs`](../crates/rustocop/src/cops/prism/map_join_rules.rs) | 2026-08-18 | 24 | 24/24 (100.0%) | 18 | 18/18 (100.0%) |
| `Style/MapToHash` | [`crates/rustocop/src/cops/prism/map_conversion_rules.rs`](../crates/rustocop/src/cops/prism/map_conversion_rules.rs) | 2026-08-20 | 38 | 38/38 (100.0%) | 51 | 51/51 (100.0%) |
| `Style/MapToSet` | [`crates/rustocop/src/cops/prism/map_conversion_rules.rs`](../crates/rustocop/src/cops/prism/map_conversion_rules.rs) | 2026-08-20 | 32 | 32/32 (100.0%) | 102 | 102/102 (100.0%) |
| `Style/MethodCalledOnDoEndBlock` | [`crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs`](../crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 6149 | 6149/6149 (100.0%) |
| `Style/MethodDefParentheses` | [`crates/rustocop/src/cops/prism/method_def_parentheses_rules.rs`](../crates/rustocop/src/cops/prism/method_def_parentheses_rules.rs) | 2026-08-20 | 49 | 49/49 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/MinMax` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Style/MinMaxComparison` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 17 | 17/17 (100.0%) | 17 | 17/17 (100.0%) |
| `Style/MissingRespondToMissing` | [`crates/rustocop/src/cops/prism/declaration_semantics.rs`](../crates/rustocop/src/cops/prism/declaration_semantics.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 52 | 52/52 (100.0%) |
| `Style/MixinGrouping` | [`crates/rustocop/src/cops/prism/mixin_grouping_rules.rs`](../crates/rustocop/src/cops/prism/mixin_grouping_rules.rs) | 2026-08-18 | 18 | 18/18 (100.0%) | 28 | 28/28 (100.0%) |
| `Style/MixinUsage` | [`crates/rustocop/src/cops/prism/mixin_rules.rs`](../crates/rustocop/src/cops/prism/mixin_rules.rs) | 2026-08-18 | 18 | 18/18 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/ModuleFunction` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 114 | 114/114 (100.0%) |
| `Style/ModuleMemberExistenceCheck` | [`crates/rustocop/src/cops/prism/module_member_existence_rules.rs`](../crates/rustocop/src/cops/prism/module_member_existence_rules.rs) | 2026-08-20 | 69 | 69/69 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/MultilineBlockChain` | [`crates/rustocop/src/cops/prism/block_chain_rules.rs`](../crates/rustocop/src/cops/prism/block_chain_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 130 | 130/130 (100.0%) |
| `Style/MultilineIfModifier` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 215 | 215/215 (100.0%) |
| `Style/MultilineIfThen` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/MultilineInPatternThen` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Style/MultilineMemoization` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 17 | 17/17 (100.0%) | 2 | 2/2 (100.0%) |
| `Style/MultilineMethodSignature` | [`crates/rustocop/src/cops/prism/method_signature_rules.rs`](../crates/rustocop/src/cops/prism/method_signature_rules.rs) | 2026-08-22 | 19 | 19/19 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/MultilineWhenThen` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 41 | 41/41 (100.0%) |
| `Style/MultipleComparison` | [`crates/rustocop/src/cops/prism/structural_forwarding_completion.rs`](../crates/rustocop/src/cops/prism/structural_forwarding_completion.rs) | 2026-08-23 | 34 | 34/34 (100.0%) | 202 | 202/202 (100.0%) |
| `Style/MutableConstant` | [`crates/rustocop/src/cops/prism/mutable_constant_rules.rs`](../crates/rustocop/src/cops/prism/mutable_constant_rules.rs) | 2026-08-20 | 354 | 354/354 (100.0%) | 768 | 768/768 (100.0%) |
| `Style/NegatedIf` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 15 | 15/15 (100.0%) | 2185 | 2185/2185 (100.0%) |
| `Style/NegatedIfElseCondition` | [`crates/rustocop/src/cops/prism/negated_if_else_rules.rs`](../crates/rustocop/src/cops/prism/negated_if_else_rules.rs) | 2026-08-20 | 32 | 32/32 (100.0%) | 152 | 152/152 (100.0%) |
| `Style/NegatedUnless` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 14 | 14/14 (100.0%) | 0 | — (unexercised) |
| `Style/NegatedWhile` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 10 | 10/10 (100.0%) | 30 | 30/30 (100.0%) |
| `Style/NegativeArrayIndex` | [`crates/rustocop/src/cops/prism/negative_array_index_rules.rs`](../crates/rustocop/src/cops/prism/negative_array_index_rules.rs) | 2026-08-20 | 423 | 423/423 (100.0%) | 4 | 4/4 (100.0%) |
| `Style/NestedFileDirname` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Style/NestedModifier` | [`crates/rustocop/src/cops/prism/nested_modifier_rules.rs`](../crates/rustocop/src/cops/prism/nested_modifier_rules.rs) | 2026-08-18 | 13 | 13/13 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/NestedParenthesizedCalls` | [`crates/rustocop/src/cops/prism/nested_call_rules.rs`](../crates/rustocop/src/cops/prism/nested_call_rules.rs) | 2026-08-21 | 12 | 12/12 (100.0%) | 97 | 97/97 (100.0%) |
| `Style/NestedTernaryOperator` | [`crates/rustocop/src/cops/prism/ternary_rules.rs`](../crates/rustocop/src/cops/prism/ternary_rules.rs) | 2026-08-20 | 7 | 7/7 (100.0%) | 325 | 325/325 (100.0%) |
| `Style/Next` | [`crates/rustocop/src/cops/prism/next_rules.rs`](../crates/rustocop/src/cops/prism/next_rules.rs) | 2026-08-21 | 71 | 71/71 (100.0%) | 209 | 209/209 (100.0%) |
| `Style/NilComparison` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 31 | 31/31 (100.0%) |
| `Style/NilLambda` | [`crates/rustocop/src/cops/prism/nil_callable_rules.rs`](../crates/rustocop/src/cops/prism/nil_callable_rules.rs) | 2026-08-18 | 31 | 31/31 (100.0%) | 35 | 35/35 (100.0%) |
| `Style/NonNilCheck` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 21 | 21/21 (100.0%) | 13 | 13/13 (100.0%) |
| `Style/Not` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/NumberedParameters` | [`crates/rustocop/src/cops/prism/block_parameter_rules.rs`](../crates/rustocop/src/cops/prism/block_parameter_rules.rs) | 2026-08-18 | 4 | 4/4 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/NumberedParametersLimit` | [`crates/rustocop/src/cops/prism/block_parameter_rules.rs`](../crates/rustocop/src/cops/prism/block_parameter_rules.rs) | 2026-08-18 | 12 | 12/12 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/NumericLiteralPrefix` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 525 | 525/525 (100.0%) |
| `Style/NumericLiterals` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-23 | 28 | 28/28 (100.0%) | 2089 | 2089/2089 (100.0%) |
| `Style/NumericPredicate` | [`crates/rustocop/src/cops/prism/numeric_predicate_rules.rs`](../crates/rustocop/src/cops/prism/numeric_predicate_rules.rs) | 2026-08-21 | 43 | 43/43 (100.0%) | 2325 | 2325/2325 (100.0%) |
| `Style/ObjectThen` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 23 | 23/23 (100.0%) | 5 | 5/5 (100.0%) |
| `Style/OneClassPerFile` | [`crates/rustocop/src/cops/prism/file_structure_rules.rs`](../crates/rustocop/src/cops/prism/file_structure_rules.rs) | 2026-08-21 | 21 | 21/21 (100.0%) | 1690 | 1690/1690 (100.0%) |
| `Style/OneLineConditional` | [`crates/rustocop/src/cops/prism/one_line_conditional_rules.rs`](../crates/rustocop/src/cops/prism/one_line_conditional_rules.rs) | 2026-08-19 | 108 | 108/108 (100.0%) | 0 | — (unexercised) |
| `Style/OpenStructUse` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 124 | 124/124 (100.0%) |
| `Style/OperatorMethodCall` | [`crates/rustocop/src/cops/prism/operator_method_call_rules.rs`](../crates/rustocop/src/cops/prism/operator_method_call_rules.rs) | 2026-08-21 | 202 | 202/202 (100.0%) | 0 | — (unexercised) |
| `Style/OptionHash` | [`crates/rustocop/src/cops/prism/argument_default_rules.rs`](../crates/rustocop/src/cops/prism/argument_default_rules.rs) | 2026-08-21 | 9 | 9/9 (100.0%) | 1563 | 1563/1563 (100.0%) |
| `Style/OptionalArguments` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 10 | 10/10 (100.0%) |
| `Style/OptionalBooleanParameter` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 356 | 356/356 (100.0%) |
| `Style/OrAssignment` | [`crates/rustocop/src/cops/prism/assignment_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/assignment_rewrite_rules.rs) | 2026-08-20 | 25 | 25/25 (100.0%) | 18 | 18/18 (100.0%) |
| `Style/ParallelAssignment` | [`crates/rustocop/src/cops/prism/assignment_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/assignment_rewrite_rules.rs) | 2026-08-20 | 86 | 86/86 (100.0%) | 543 | 543/543 (100.0%) |
| `Style/ParenthesesAroundCondition` | [`crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs) | 2026-08-21 | 30 | 30/30 (100.0%) | 52 | 52/52 (100.0%) |
| `Style/PartitionInsteadOfDoubleSelect` | [`crates/rustocop/src/cops/prism/collection_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/collection_rewrite_rules.rs) | 2026-08-21 | 37 | 37/37 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/PercentLiteralDelimiters` | [`crates/rustocop/src/cops/prism/literal_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/literal_rewrite_rules.rs) | 2026-08-20 | 64 | 64/64 (100.0%) | 2574 | 2574/2574 (100.0%) |
| `Style/PercentQLiterals` | [`crates/rustocop/src/cops/prism/path_and_literal_rules.rs`](../crates/rustocop/src/cops/prism/path_and_literal_rules.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 56 | 56/56 (100.0%) |
| `Style/PerlBackrefs` | [`crates/rustocop/src/cops/prism/style_global_vars.rs`](../crates/rustocop/src/cops/prism/style_global_vars.rs) | 2026-08-21 | 14 | 14/14 (100.0%) | 644 | 644/644 (100.0%) |
| `Style/PredicateWithKind` | [`crates/rustocop/src/cops/prism/collection_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/collection_rewrite_rules.rs) | 2026-08-21 | 64 | 64/64 (100.0%) | 20 | 20/20 (100.0%) |
| `Style/PreferredHashMethods` | [`crates/rustocop/src/cops/prism/preferred_hash_methods_rules.rs`](../crates/rustocop/src/cops/prism/preferred_hash_methods_rules.rs) | 2026-08-21 | 9 | 9/9 (100.0%) | 411 | 411/411 (100.0%) |
| `Style/Proc` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 6 | 6/6 (100.0%) | 352 | 352/352 (100.0%) |
| `Style/QuotedSymbols` | [`crates/rustocop/src/cops/prism/literal_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/literal_rewrite_rules.rs) | 2026-08-20 | 97 | 97/97 (100.0%) | 2823 | 2823/2823 (100.0%) |
| `Style/RaiseArgs` | [`crates/rustocop/src/cops/prism/exception_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/exception_rewrite_rules.rs) | 2026-08-19 | 35 | 35/35 (100.0%) | 906 | 906/906 (100.0%) |
| `Style/RandomWithOffset` | [`crates/rustocop/src/cops/prism/random_rules.rs`](../crates/rustocop/src/cops/prism/random_rules.rs) | 2026-08-18 | 29 | 29/29 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/ReduceToHash` | [`crates/rustocop/src/cops/prism/collection_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/collection_rewrite_rules.rs) | 2026-08-21 | 25 | 25/25 (100.0%) | 97 | 97/97 (100.0%) |
| `Style/RedundantArgument` | [`crates/rustocop/src/cops/prism/argument_default_rules.rs`](../crates/rustocop/src/cops/prism/argument_default_rules.rs) | 2026-08-21 | 15 | 15/15 (100.0%) | 175 | 175/175 (100.0%) |
| `Style/RedundantArrayConstructor` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 123 | 123/123 (100.0%) |
| `Style/RedundantArrayFlatten` | [`crates/rustocop/src/cops/prism/style_collections.rs`](../crates/rustocop/src/cops/prism/style_collections.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 8 | 8/8 (100.0%) |
| `Style/RedundantBegin` | [`crates/rustocop/src/cops/prism/begin_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/begin_rewrite_rules.rs) | 2026-08-21 | 63 | 63/63 (100.0%) | 42 | 42/42 (100.0%) |
| `Style/RedundantCapitalW` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 18 | 18/18 (100.0%) |
| `Style/RedundantCondition` | [`crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs) | 2026-08-21 | 102 | 102/102 (100.0%) | 36 | 36/36 (100.0%) |
| `Style/RedundantConditional` | [`crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs) | 2026-08-21 | 11 | 11/11 (100.0%) | 0 | — (unexercised) |
| `Style/RedundantCurrentDirectoryInPath` | [`crates/rustocop/src/cops/prism/argument_default_rules.rs`](../crates/rustocop/src/cops/prism/argument_default_rules.rs) | 2026-08-21 | 12 | 12/12 (100.0%) | 26 | 26/26 (100.0%) |
| `Style/RedundantDoubleSplatHashBraces` | [`crates/rustocop/src/cops/prism/double_splat_rules.rs`](../crates/rustocop/src/cops/prism/double_splat_rules.rs) | 2026-08-18 | 29 | 29/29 (100.0%) | 24 | 24/24 (100.0%) |
| `Style/RedundantEach` | [`crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs`](../crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs) | 2026-08-22 | 36 | 36/36 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/RedundantException` | [`crates/rustocop/src/cops/prism/exception_argument_rules.rs`](../crates/rustocop/src/cops/prism/exception_argument_rules.rs) | 2026-08-18 | 30 | 30/30 (100.0%) | 31 | 31/31 (100.0%) |
| `Style/RedundantFetchBlock` | [`crates/rustocop/src/cops/prism/fetch_completion_rules.rs`](../crates/rustocop/src/cops/prism/fetch_completion_rules.rs) | 2026-08-22 | 15 | 15/15 (100.0%) | 88 | 88/88 (100.0%) |
| `Style/RedundantFileExtensionInRequire` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/RedundantFilterChain` | [`crates/rustocop/src/cops/prism/redundant_filter_chain_rules.rs`](../crates/rustocop/src/cops/prism/redundant_filter_chain_rules.rs) | 2026-08-19 | 39 | 39/39 (100.0%) | 5 | 5/5 (100.0%) |
| `Style/RedundantFormat` | [`crates/rustocop/src/cops/prism/redundant_format_rules.rs`](../crates/rustocop/src/cops/prism/redundant_format_rules.rs) | 2026-08-19 | 290 | 290/290 (100.0%) | 5 | 5/5 (100.0%) |
| `Style/RedundantFreeze` | [`crates/rustocop/src/cops/prism/redundant_freeze_completion.rs`](../crates/rustocop/src/cops/prism/redundant_freeze_completion.rs) | 2026-08-19 | 62 | 62/62 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/RedundantHeredocDelimiterQuotes` | [`crates/rustocop/src/cops/prism/lexical_completion.rs`](../crates/rustocop/src/cops/prism/lexical_completion.rs) | 2026-08-23 | 17 | 17/17 (100.0%) | 93 | 93/93 (100.0%) |
| `Style/RedundantInitialize` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-22 | 23 | 23/23 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/RedundantInterpolation` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-20 | 29 | 29/29 (100.0%) | 526 | 526/526 (100.0%) |
| `Style/RedundantInterpolationUnfreeze` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-20 | 17 | 17/17 (100.0%) | 146 | 146/146 (100.0%) |
| `Style/RedundantLineContinuation` | [`crates/rustocop/src/cops/prism/redundant_line_continuation_rules.rs`](../crates/rustocop/src/cops/prism/redundant_line_continuation_rules.rs) | 2026-08-23 | 167 | 167/167 (100.0%) | 61 | 61/61 (100.0%) |
| `Style/RedundantMinMaxBy` | [`crates/rustocop/src/cops/prism/redundant_min_max_by_rules.rs`](../crates/rustocop/src/cops/prism/redundant_min_max_by_rules.rs) | 2026-08-19 | 33 | 33/33 (100.0%) | 0 | — (unexercised) |
| `Style/RedundantPercentQ` | [`crates/rustocop/src/cops/prism/percent_string_rules.rs`](../crates/rustocop/src/cops/prism/percent_string_rules.rs) | 2026-08-23 | 27 | 27/27 (100.0%) | 304 | 304/304 (100.0%) |
| `Style/RedundantRegexpArgument` | [`crates/rustocop/src/cops/prism/redundant_regexp_rules.rs`](../crates/rustocop/src/cops/prism/redundant_regexp_rules.rs) | 2026-08-22 | 50 | 50/50 (100.0%) | 171 | 171/171 (100.0%) |
| `Style/RedundantRegexpCharacterClass` | [`crates/rustocop/src/cops/prism/redundant_regexp_rules.rs`](../crates/rustocop/src/cops/prism/redundant_regexp_rules.rs) | 2026-08-22 | 47 | 47/47 (100.0%) | 38 | 38/38 (100.0%) |
| `Style/RedundantRegexpConstructor` | [`crates/rustocop/src/cops/prism/redundant_regexp_rules.rs`](../crates/rustocop/src/cops/prism/redundant_regexp_rules.rs) | 2026-08-22 | 10 | 10/10 (100.0%) | 5 | 5/5 (100.0%) |
| `Style/RedundantRegexpEscape` | [`crates/rustocop/src/cops/prism/redundant_regexp_rules.rs`](../crates/rustocop/src/cops/prism/redundant_regexp_rules.rs) | 2026-08-22 | 217 | 217/217 (100.0%) | 580 | 580/580 (100.0%) |
| `Style/RedundantReturn` | [`crates/rustocop/src/cops/prism/redundant_return_rules.rs`](../crates/rustocop/src/cops/prism/redundant_return_rules.rs) | 2026-08-19 | 39 | 39/39 (100.0%) | 41 | 41/41 (100.0%) |
| `Style/RedundantSelf` | [`crates/rustocop/src/cops/prism/self_rules.rs`](../crates/rustocop/src/cops/prism/self_rules.rs) | 2026-08-23 | 63 | 63/63 (100.0%) | 1033 | 1033/1033 (100.0%) |
| `Style/RedundantSelfAssignment` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-22 | 14 | 14/14 (100.0%) | 16 | 16/16 (100.0%) |
| `Style/RedundantSelfAssignmentBranch` | [`crates/rustocop/src/cops/prism/redundant_self_assignment_branch_rules.rs`](../crates/rustocop/src/cops/prism/redundant_self_assignment_branch_rules.rs) | 2026-08-20 | 22 | 22/22 (100.0%) | 44 | 44/44 (100.0%) |
| `Style/RedundantSort` | [`crates/rustocop/src/cops/prism/redundant_sort_rules.rs`](../crates/rustocop/src/cops/prism/redundant_sort_rules.rs) | 2026-08-19 | 50 | 50/50 (100.0%) | 12 | 12/12 (100.0%) |
| `Style/RedundantSortBy` | [`crates/rustocop/src/cops/prism/style_collections.rs`](../crates/rustocop/src/cops/prism/style_collections.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 0 | — (unexercised) |
| `Style/RedundantStringEscape` | [`crates/rustocop/src/cops/prism/redundant_string_escape_rules.rs`](../crates/rustocop/src/cops/prism/redundant_string_escape_rules.rs) | 2026-08-19 | 328 | 328/328 (100.0%) | 1604 | 1604/1604 (100.0%) |
| `Style/RedundantStructKeywordInit` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-22 | 17 | 17/17 (100.0%) | 141 | 141/141 (100.0%) |
| `Style/RegexpLiteral` | [`crates/rustocop/src/cops/prism/regexp_literal_rules.rs`](../crates/rustocop/src/cops/prism/regexp_literal_rules.rs) | 2026-08-20 | 65 | 65/65 (100.0%) | 1406 | 1406/1406 (100.0%) |
| `Style/RequireOrder` | [`crates/rustocop/src/cops/prism/require_order_rules.rs`](../crates/rustocop/src/cops/prism/require_order_rules.rs) | 2026-08-22 | 24 | 24/24 (100.0%) | 4298 | 4298/4298 (100.0%) |
| `Style/RescueModifier` | [`crates/rustocop/src/cops/prism/rescue_modifier_rules.rs`](../crates/rustocop/src/cops/prism/rescue_modifier_rules.rs) | 2026-08-19 | 21 | 21/21 (100.0%) | 166 | 166/166 (100.0%) |
| `Style/RescueStandardError` | [`crates/rustocop/src/cops/prism/rescue_standard_error_rules.rs`](../crates/rustocop/src/cops/prism/rescue_standard_error_rules.rs) | 2026-08-19 | 37 | 37/37 (100.0%) | 600 | 600/600 (100.0%) |
| `Style/ReturnNil` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 698 | 698/698 (100.0%) |
| `Style/ReturnNilInPredicateMethodDefinition` | [`crates/rustocop/src/cops/prism/return_nil_predicate_rules.rs`](../crates/rustocop/src/cops/prism/return_nil_predicate_rules.rs) | 2026-08-19 | 39 | 39/39 (100.0%) | 140 | 140/140 (100.0%) |
| `Style/ReverseFind` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 0 | — (unexercised) |
| `Style/SafeNavigation` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 798 | 798/798 (100.0%) | 586 | 586/586 (100.0%) |
| `Style/SafeNavigationChainLength` | [`crates/rustocop/src/cops/prism/nested_call_rules.rs`](../crates/rustocop/src/cops/prism/nested_call_rules.rs) | 2026-08-21 | 8 | 8/8 (100.0%) | 100 | 100/100 (100.0%) |
| `Style/Sample` | [`crates/rustocop/src/cops/prism/collection_transform_batch.rs`](../crates/rustocop/src/cops/prism/collection_transform_batch.rs) | 2026-08-21 | 82 | 82/82 (100.0%) | 0 | — (unexercised) |
| `Style/SelectByKind` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 144 | 144/144 (100.0%) | 41 | 41/41 (100.0%) |
| `Style/SelectByRange` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-23 | 120 | 120/120 (100.0%) | 0 | — (unexercised) |
| `Style/SelectByRegexp` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-23 | 320 | 320/320 (100.0%) | 66 | 66/66 (100.0%) |
| `Style/SelfAssignment` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-23 | 105 | 105/105 (100.0%) | 83 | 83/83 (100.0%) |
| `Style/Semicolon` | [`crates/rustocop/src/cops/prism/style_source.rs`](../crates/rustocop/src/cops/prism/style_source.rs) | 2026-08-20 | 33 | 33/33 (100.0%) | 291 | 291/291 (100.0%) |
| `Style/Send` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 4746 | 4746/4746 (100.0%) |
| `Style/SendWithLiteralMethodName` | [`crates/rustocop/src/cops/prism/send_literal_rules.rs`](../crates/rustocop/src/cops/prism/send_literal_rules.rs) | 2026-08-19 | 115 | 115/115 (100.0%) | 27 | 27/27 (100.0%) |
| `Style/SignalException` | [`crates/rustocop/src/cops/prism/signal_exception_rules.rs`](../crates/rustocop/src/cops/prism/signal_exception_rules.rs) | 2026-08-19 | 27 | 27/27 (100.0%) | 159 | 159/159 (100.0%) |
| `Style/SingleArgumentDig` | [`crates/rustocop/src/cops/prism/style_call_simplifications.rs`](../crates/rustocop/src/cops/prism/style_call_simplifications.rs) | 2026-08-19 | 15 | 15/15 (100.0%) | 57 | 57/57 (100.0%) |
| `Style/SingleLineBlockParams` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 82 | 82/82 (100.0%) |
| `Style/SingleLineDoEndBlock` | [`crates/rustocop/src/cops/prism/single_line_block_rules.rs`](../crates/rustocop/src/cops/prism/single_line_block_rules.rs) | 2026-08-19 | 15 | 15/15 (100.0%) | 322 | 322/322 (100.0%) |
| `Style/SingleLineMethods` | [`crates/rustocop/src/cops/prism/method_layout_rules.rs`](../crates/rustocop/src/cops/prism/method_layout_rules.rs) | 2026-08-20 | 147 | 147/147 (100.0%) | 690 | 690/690 (100.0%) |
| `Style/SlicingWithRange` | [`crates/rustocop/src/cops/prism/path_and_literal_rules.rs`](../crates/rustocop/src/cops/prism/path_and_literal_rules.rs) | 2026-08-23 | 28 | 28/28 (100.0%) | 152 | 152/152 (100.0%) |
| `Style/SoleNestedConditional` | [`crates/rustocop/src/cops/prism/sole_nested_conditional_rules.rs`](../crates/rustocop/src/cops/prism/sole_nested_conditional_rules.rs) | 2026-08-19 | 74 | 74/74 (100.0%) | 306 | 306/306 (100.0%) |
| `Style/SpecialGlobalVars` | [`crates/rustocop/src/cops/prism/special_global_vars_rules.rs`](../crates/rustocop/src/cops/prism/special_global_vars_rules.rs) | 2026-08-22 | 31 | 31/31 (100.0%) | 242 | 242/242 (100.0%) |
| `Style/StabbyLambdaParentheses` | [`crates/rustocop/src/cops/prism/stabby_lambda_parentheses_rules.rs`](../crates/rustocop/src/cops/prism/stabby_lambda_parentheses_rules.rs) | 2026-08-19 | 9 | 9/9 (100.0%) | 66 | 66/66 (100.0%) |
| `Style/StaticClass` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 462 | 462/462 (100.0%) |
| `Style/StderrPuts` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 240 | 240/240 (100.0%) |
| `Style/StringChars` | [`crates/rustocop/src/cops/prism/redundant_freeze_completion.rs`](../crates/rustocop/src/cops/prism/redundant_freeze_completion.rs) | 2026-08-19 | 8 | 8/8 (100.0%) | 17 | 17/17 (100.0%) |
| `Style/StringConcatenation` | [`crates/rustocop/src/cops/prism/literal_string_completion_batch.rs`](../crates/rustocop/src/cops/prism/literal_string_completion_batch.rs) | 2026-08-23 | 30 | 30/30 (100.0%) | 1320 | 1320/1320 (100.0%) |
| `Style/StringHashKeys` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-20 | 10 | 10/10 (100.0%) | 75506 | 75506/75506 (100.0%) |
| `Style/StringLiterals` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-20 | 57 | 57/57 (100.0%) | 672527 | 672527/672527 (100.0%) |
| `Style/StringLiteralsInInterpolation` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-20 | 12 | 12/12 (100.0%) | 3552 | 3552/3552 (100.0%) |
| `Style/StringMethods` | [`crates/rustocop/src/cops/prism/style/misc.rs`](../crates/rustocop/src/cops/prism/style/misc.rs) | 2026-08-19 | 2 | 2/2 (100.0%) | 4 | 4/4 (100.0%) |
| `Style/Strip` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 6 | 6/6 (100.0%) | 0 | — (unexercised) |
| `Style/StructInheritance` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-22 | 13 | 13/13 (100.0%) | 21 | 21/21 (100.0%) |
| `Style/SuperArguments` | [`crates/rustocop/src/cops/prism/super_arguments_rules.rs`](../crates/rustocop/src/cops/prism/super_arguments_rules.rs) | 2026-08-19 | 92 | 92/92 (100.0%) | 265 | 265/265 (100.0%) |
| `Style/SuperWithArgsParentheses` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 124 | 124/124 (100.0%) |
| `Style/SwapValues` | [`crates/rustocop/src/cops/prism/assignment_completion_rules.rs`](../crates/rustocop/src/cops/prism/assignment_completion_rules.rs) | 2026-08-19 | 11 | 11/11 (100.0%) | 0 | — (unexercised) |
| `Style/SymbolArray` | [`crates/rustocop/src/cops/prism/literal_string_completion_batch.rs`](../crates/rustocop/src/cops/prism/literal_string_completion_batch.rs) | 2026-08-23 | 33 | 33/33 (100.0%) | 5867 | 5867/5867 (100.0%) |
| `Style/SymbolLiteral` | [`crates/rustocop/src/cops/prism/symbol_literal_rules.rs`](../crates/rustocop/src/cops/prism/symbol_literal_rules.rs) | 2026-08-19 | 4 | 4/4 (100.0%) | 440 | 440/440 (100.0%) |
| `Style/SymbolProc` | [`crates/rustocop/src/cops/prism/symbol_proc_rules.rs`](../crates/rustocop/src/cops/prism/symbol_proc_rules.rs) | 2026-08-21 | 83 | 83/83 (100.0%) | 650 | 650/650 (100.0%) |
| `Style/TallyMethod` | [`crates/rustocop/src/cops/prism/modern_collection_completion.rs`](../crates/rustocop/src/cops/prism/modern_collection_completion.rs) | 2026-08-22 | 32 | 32/32 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/TernaryParentheses` | [`crates/rustocop/src/cops/prism/ternary_parentheses_rules.rs`](../crates/rustocop/src/cops/prism/ternary_parentheses_rules.rs) | 2026-08-19 | 98 | 98/98 (100.0%) | 218 | 218/218 (100.0%) |
| `Style/TopLevelMethodDefinition` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-22 | 14 | 14/14 (100.0%) | 297 | 297/297 (100.0%) |
| `Style/TrailingBodyOnClass` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Style/TrailingBodyOnMethodDefinition` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Style/TrailingBodyOnModule` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Style/TrailingCommaInArguments` | [`crates/rustocop/src/cops/prism/trailing_argument_comma_rules.rs`](../crates/rustocop/src/cops/prism/trailing_argument_comma_rules.rs) | 2026-08-19 | 178 | 178/178 (100.0%) | 34337 | 34337/34337 (100.0%) |
| `Style/TrailingCommaInArrayLiteral` | [`crates/rustocop/src/cops/prism/trailing_comma_completion.rs`](../crates/rustocop/src/cops/prism/trailing_comma_completion.rs) | 2026-08-19 | 48 | 48/48 (100.0%) | 3704 | 3704/3704 (100.0%) |
| `Style/TrailingCommaInBlockArgs` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 20 | 20/20 (100.0%) | 4 | 4/4 (100.0%) |
| `Style/TrailingCommaInHashLiteral` | [`crates/rustocop/src/cops/prism/trailing_comma_completion.rs`](../crates/rustocop/src/cops/prism/trailing_comma_completion.rs) | 2026-08-19 | 41 | 41/41 (100.0%) | 19453 | 19453/19453 (100.0%) |
| `Style/TrailingMethodEndStatement` | [`crates/rustocop/src/cops/prism/method_layout_rules.rs`](../crates/rustocop/src/cops/prism/method_layout_rules.rs) | 2026-08-20 | 11 | 11/11 (100.0%) | 0 | — (unexercised) |
| `Style/TrailingUnderscoreVariable` | [`crates/rustocop/src/cops/prism/trailing_underscore_rules.rs`](../crates/rustocop/src/cops/prism/trailing_underscore_rules.rs) | 2026-08-19 | 58 | 58/58 (100.0%) | 231 | 231/231 (100.0%) |
| `Style/TrivialAccessors` | [`crates/rustocop/src/cops/prism/trivial_accessor_rules.rs`](../crates/rustocop/src/cops/prism/trivial_accessor_rules.rs) | 2026-08-19 | 38 | 38/38 (100.0%) | 50 | 50/50 (100.0%) |
| `Style/UnlessElse` | [`crates/rustocop/src/cops/prism/style_source.rs`](../crates/rustocop/src/cops/prism/style_source.rs) | 2026-08-20 | 5 | 5/5 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/UnlessLogicalOperators` | [`crates/rustocop/src/cops/prism/logical_condition_rules.rs`](../crates/rustocop/src/cops/prism/logical_condition_rules.rs) | 2026-08-20 | 28 | 28/28 (100.0%) | 38 | 38/38 (100.0%) |
| `Style/UnpackFirst` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 5 | 5/5 (100.0%) |
| `Style/VariableInterpolation` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 11 | 11/11 (100.0%) |
| `Style/WhenThen` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 4 | 4/4 (100.0%) | 42 | 42/42 (100.0%) |
| `Style/WhileUntilDo` | [`crates/rustocop/src/cops/prism/while_until_do_rules.rs`](../crates/rustocop/src/cops/prism/while_until_do_rules.rs) | 2026-08-19 | 6 | 6/6 (100.0%) | 2 | 2/2 (100.0%) |
| `Style/WhileUntilModifier` | [`crates/rustocop/src/cops/prism/compact_syntax_completion.rs`](../crates/rustocop/src/cops/prism/compact_syntax_completion.rs) | 2026-08-23 | 48 | 48/48 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/WordArray` | [`crates/rustocop/src/cops/prism/literal_string_completion_batch.rs`](../crates/rustocop/src/cops/prism/literal_string_completion_batch.rs) | 2026-08-23 | 59 | 59/59 (100.0%) | 1081 | 1081/1081 (100.0%) |
| `Style/YAMLFileRead` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 11 | 11/11 (100.0%) | 53 | 53/53 (100.0%) |
| `Style/YodaCondition` | [`crates/rustocop/src/cops/prism/yoda_condition_rules.rs`](../crates/rustocop/src/cops/prism/yoda_condition_rules.rs) | 2026-08-19 | 76 | 76/76 (100.0%) | 66 | 66/66 (100.0%) |
| `Style/YodaExpression` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 1562 | 1562/1562 (100.0%) |
| `Style/ZeroLengthPredicate` | [`crates/rustocop/src/cops/prism/modern_collection_completion.rs`](../crates/rustocop/src/cops/prism/modern_collection_completion.rs) | 2026-08-22 | 68 | 68/68 (100.0%) | 321 | 321/321 (100.0%) |
