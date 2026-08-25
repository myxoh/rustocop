# RuboCop compatibility evidence

Generated at `2026-08-25T07:41:42-04:00` for RuboCop 1.87.0.
Compatibility is binary at the cop level: every exercised fixture must match,
and project output must have no false positives, false negatives, or signature
differences. Partial overlap is not classified as compatible.

This table covers 606 active built-in cops. The [`intentionally_pending_cops.yml`](../spec/upstream/rubocop-1.87.0/intentionally_pending_cops.yml) manifest is empty.

Fixture evidence was updated at `2026-08-25T07:41:42-04:00`. Project
evidence was updated at `2026-08-24T15:25:53-04:00` from
50 projects and 85471 Ruby files.
Fixture source: `cop source 4ae97b9b3b9c, native acb139b49821`. Project source:
`ddb32ffcc4aaa97d560add1482e3c33863409004`.

This evidence covers the complete configured 50-project corpus.

## Overall

| Measure | Result | Percent |
| --- | ---: | ---: |
| Cops with fixture coverage | 606/606 | 100.0% |
| Cops with current fixture evidence | 606/606 | 100.0% |
| Fixture cases matching | 28845/28846 | 100.0% |
| Cops matching every fixture | 605/606 | 99.8% |
| Cops exercised on projects | 550/606 | 90.8% |
| Cops with current project evidence | 547/606 | 90.3% |
| Project-exact cops among exercised cops | 291/550 | 52.9% |
| Cops compatible in both evidence sets | 291/606 | 48.0% |

“Project hits” is the number of RuboCop reference diagnostics. Project matching
is exact shared signatures divided by the union of Rustocop and RuboCop
signatures, so both extra and missing diagnostics reduce the percentage. A
zero-hit row is unexercised, not 100% compatible.

## Updating

Refresh the Rust-only controlled unit evidence while retaining the existing
project columns:

```sh
bundle exec ruby script/generate_compatibility_report.rb --refresh-fixtures
```

Refresh both evidence sets only when the expensive project scan is intended:

```sh
bundle exec ruby script/generate_compatibility_report.rb \
  --refresh-fixtures --refresh-projects
```

Without either refresh flag, the generator only renders the checked-in compact
snapshots. Use `--check` in CI to verify that the table is current.

A stale marker means one of that cop's implementation files differs from the
relevant evidence source identity. Stale rows remain visible but do not count as
compatible in the overall totals.

## Per-cop evidence

| Cop | Implementation file | Implementation updated | Fixture tests<br>(as of 2026-08-25T07:41:42-04:00) | Fixture matching<br>(as of 2026-08-25T07:41:42-04:00) | Project hits<br>(as of 2026-08-24T15:25:53-04:00) | Project matching<br>(as of 2026-08-24T15:25:53-04:00) |
| --- | --- | --- | ---: | ---: | ---: | ---: |
| `Bundler/DuplicatedGem` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 0 | — (unexercised) |
| `Bundler/DuplicatedGroup` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-24 | 21 | 21/21 (100.0%) | 0 | — (unexercised) |
| `Bundler/GemComment` | [`crates/rustocop/src/cops/prism/final_project_context_batch.rs`](../crates/rustocop/src/cops/prism/final_project_context_batch.rs) | 2026-08-24 | 26 | 26/26 (100.0%) | 0 | — (unexercised) |
| `Bundler/GemFilename` | [`crates/rustocop/src/cops/prism/bundler_completion.rs`](../crates/rustocop/src/cops/prism/bundler_completion.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 0 | — (unexercised) |
| `Bundler/GemVersion` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 7 | 7/7 (100.0%) | 0 ⚠ | — (unexercised) ⚠ stale |
| `Bundler/InsecureProtocolSource` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 0 | 0/3 (0.0%) |
| `Bundler/OrderedGems` | [`crates/rustocop/src/cops/prism/bundler_completion.rs`](../crates/rustocop/src/cops/prism/bundler_completion.rs) | 2026-08-24 | 19 | 19/19 (100.0%) | 0 | — (unexercised) |
| `Gemspec/AddRuntimeDependency` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Gemspec/AttributeAssignment` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | 0/37 (0.0%) |
| `Gemspec/DependencyVersion` | [`crates/rustocop/src/cops/prism/final_project_context_batch.rs`](../crates/rustocop/src/cops/prism/final_project_context_batch.rs) | 2026-08-24 | 77 | 77/77 (100.0%) | 0 | 0/45 (0.0%) |
| `Gemspec/DeprecatedAttributeAssignment` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-24 | 19 | 19/19 (100.0%) | 0 | — (unexercised) |
| `Gemspec/DevelopmentDependencies` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Gemspec/DuplicatedAssignment` | [`crates/rustocop/src/cops/prism/gemspec_completion.rs`](../crates/rustocop/src/cops/prism/gemspec_completion.rs) | 2026-08-22 | 18 | 18/18 (100.0%) | 0 | — (unexercised) |
| `Gemspec/OrderedDependencies` | [`crates/rustocop/src/cops/prism/gemspec_completion.rs`](../crates/rustocop/src/cops/prism/gemspec_completion.rs) | 2026-08-22 | 19 | 19/19 (100.0%) | 0 | — (unexercised) |
| `Gemspec/RequireMFA` | [`crates/rustocop/src/cops/prism/gemspec_completion.rs`](../crates/rustocop/src/cops/prism/gemspec_completion.rs) | 2026-08-22 | 16 | 16/16 (100.0%) | 0 | — (unexercised) |
| `Gemspec/RequiredRubyVersion` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-24 | 22 | 22/22 (100.0%) | 0 | — (unexercised) |
| `Gemspec/RubyVersionGlobalsUsage` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 0 | — (unexercised) |
| `Layout/AccessModifierIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 44 | 44/44 (100.0%) | 173 | 173/173 (100.0%) |
| `Layout/ArgumentAlignment` | [`crates/rustocop/src/cops/prism/restored_layout_indentation/core.rs`](../crates/rustocop/src/cops/prism/restored_layout_indentation/core.rs) | 2026-08-24 | 55 | 55/55 (100.0%) | 20632 | 20632/20632 (100.0%) |
| `Layout/ArrayAlignment` | [`crates/rustocop/src/cops/prism/layout_qualification.rs`](../crates/rustocop/src/cops/prism/layout_qualification.rs) | 2026-08-24 | 27 | 27/27 (100.0%) | 2299 | 2299/2299 (100.0%) |
| `Layout/AssignmentIndentation` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 22 | 18/22 (81.8%) |
| `Layout/BeginEndAlignment` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 39 | 39/39 (100.0%) |
| `Layout/BlockAlignment` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-24 | 79 | 79/79 (100.0%) | 215 | 213/220 (96.8%) |
| `Layout/BlockEndNewline` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-24 | 19 | 19/19 (100.0%) | 124 | 124/124 (100.0%) |
| `Layout/CaseIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 49 | 49/49 (100.0%) | 466 | 466/466 (100.0%) |
| `Layout/ClassStructure` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-24 | 23 | 23/23 (100.0%) | 2878 | 2878/2880 (99.9%) |
| `Layout/ClosingHeredocIndentation` | [`crates/rustocop/src/cops/prism/lexical_completion.rs`](../crates/rustocop/src/cops/prism/lexical_completion.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 1240 | 1240/1240 (100.0%) |
| `Layout/ClosingParenthesisIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 44 | 44/44 (100.0%) | 372 | 340/373 (91.2%) |
| `Layout/CommentIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 30 | 30/30 (100.0%) | 170 | 170/171 (99.4%) |
| `Layout/ConditionPosition` | [`crates/rustocop/src/cops/prism/layout_line_break_completion.rs`](../crates/rustocop/src/cops/prism/layout_line_break_completion.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 0 | — (unexercised) |
| `Layout/DefEndAlignment` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-24 | 18 | 18/18 (100.0%) | 4 | 4/4 (100.0%) |
| `Layout/DotPosition` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-24 | 40 | 40/40 (100.0%) | 1949 | 1949/1949 (100.0%) |
| `Layout/ElseAlignment` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 53 | 53/53 (100.0%) | 1043 | 1043/1043 (100.0%) |
| `Layout/EmptyComment` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 99 | 98/106 (92.5%) |
| `Layout/EmptyLineAfterGuardClause` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 70 | 70/70 (100.0%) | 4964 | 4957/4966 (99.8%) |
| `Layout/EmptyLineAfterMagicComment` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 22 | 22/22 (100.0%) | 1194 | 1192/1194 (99.8%) |
| `Layout/EmptyLineAfterMultilineCondition` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-24 | 23 | 23/23 (100.0%) | 1618 | 1618/1618 (100.0%) |
| `Layout/EmptyLineBetweenDefs` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-24 | 45 | 45/45 (100.0%) | 729 | 729/729 (100.0%) |
| `Layout/EmptyLines` | [`crates/rustocop/src/cops/prism/source_rules_layout.rs`](../crates/rustocop/src/cops/prism/source_rules_layout.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 498 ⚠ | 498/498 (100.0%) ⚠ stale |
| `Layout/EmptyLinesAfterModuleInclusion` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-24 | 61 | 61/61 (100.0%) | 1024 | 1024/1032 (99.2%) |
| `Layout/EmptyLinesAroundAccessModifier` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-24 | 178 | 178/178 (100.0%) | 1764 | 1764/1767 (99.8%) |
| `Layout/EmptyLinesAroundArguments` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 22 | 22/22 (100.0%) | 394 | 394/394 (100.0%) |
| `Layout/EmptyLinesAroundAttributeAccessor` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 240 | 240/242 (99.2%) |
| `Layout/EmptyLinesAroundBeginBody` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 5 | 5/5 (100.0%) |
| `Layout/EmptyLinesAroundBlockBody` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 742 | 742/742 (100.0%) |
| `Layout/EmptyLinesAroundClassBody` | [`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 46 | 46/46 (100.0%) | 971 | 971/971 (100.0%) |
| `Layout/EmptyLinesAroundExceptionHandlingKeywords` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 25 | 25/25 (100.0%) | 188 | 188/207 (90.8%) |
| `Layout/EmptyLinesAroundMethodBody` | [`crates/rustocop/src/cops/prism/layout_body_completion.rs`](../crates/rustocop/src/cops/prism/layout_body_completion.rs)<br>[`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 320 | 320/320 (100.0%) |
| `Layout/EmptyLinesAroundModuleBody` | [`crates/rustocop/src/cops/prism/layout_body_qualification.rs`](../crates/rustocop/src/cops/prism/layout_body_qualification.rs) | 2026-08-23 | 37 | 37/37 (100.0%) | 347 | 347/347 (100.0%) |
| `Layout/EndAlignment` | [`crates/rustocop/src/cops/prism/layout_qualification.rs`](../crates/rustocop/src/cops/prism/layout_qualification.rs) | 2026-08-24 | 217 | 217/217 (100.0%) | 1119 | 1119/1119 (100.0%) |
| `Layout/EndOfLine` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Layout/ExtraSpacing` | [`crates/rustocop/src/cops/prism/layout_qualification.rs`](../crates/rustocop/src/cops/prism/layout_qualification.rs) | 2026-08-24 | 90 | 90/90 (100.0%) | 1905 | 471/2763 (17.0%) |
| `Layout/FirstArgumentIndentation` | [`crates/rustocop/src/cops/prism/layout_core_qualification.rs`](../crates/rustocop/src/cops/prism/layout_core_qualification.rs) | 2026-08-24 | 140 | 140/140 (100.0%) | 883 | 878/883 (99.4%) |
| `Layout/FirstArrayElementIndentation` | [`crates/rustocop/src/cops/prism/restored_layout_indentation/core.rs`](../crates/rustocop/src/cops/prism/restored_layout_indentation/core.rs) | 2026-08-24 | 60 | 60/60 (100.0%) | 3431 | 3431/3443 (99.7%) |
| `Layout/FirstArrayElementLineBreak` | [`crates/rustocop/src/cops/prism/restored_layout_line_breaks.rs`](../crates/rustocop/src/cops/prism/restored_layout_line_breaks.rs) | 2026-08-23 | 17 | 17/17 (100.0%) | 3665 | 3665/3665 (100.0%) |
| `Layout/FirstHashElementIndentation` | [`crates/rustocop/src/cops/prism/layout_qualification.rs`](../crates/rustocop/src/cops/prism/layout_qualification.rs) | 2026-08-24 | 61 | 61/61 (100.0%) | 7366 | 7366/7366 (100.0%) |
| `Layout/FirstHashElementLineBreak` | [`crates/rustocop/src/cops/prism/restored_layout_line_breaks.rs`](../crates/rustocop/src/cops/prism/restored_layout_line_breaks.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 3723 | 3723/3723 (100.0%) |
| `Layout/FirstMethodArgumentLineBreak` | [`crates/rustocop/src/cops/prism/restored_layout_line_breaks.rs`](../crates/rustocop/src/cops/prism/restored_layout_line_breaks.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 31974 | 31974/31974 (100.0%) |
| `Layout/FirstMethodParameterLineBreak` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 417 | 417/417 (100.0%) |
| `Layout/FirstParameterIndentation` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-24 | 20 | 20/20 (100.0%) | 5 | 1/28 (3.6%) |
| `Layout/HashAlignment` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-24 | 132 | 132/132 (100.0%) | 23714 ⚠ | 23714/23714 (100.0%) ⚠ stale |
| `Layout/HeredocArgumentClosingParenthesis` | [`crates/rustocop/src/cops/prism/heredoc_argument_closing_parenthesis_rules.rs`](../crates/rustocop/src/cops/prism/heredoc_argument_closing_parenthesis_rules.rs) | 2026-08-24 | 81 | 81/81 (100.0%) | 34 | 34/58 (58.6%) |
| `Layout/HeredocIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 111 | 111/111 (100.0%) | 3447 | 3447/3447 (100.0%) |
| `Layout/IndentationConsistency` | [`crates/rustocop/src/cops/prism/restored_layout_indentation/core.rs`](../crates/rustocop/src/cops/prism/restored_layout_indentation/core.rs) | 2026-08-24 | 60 | 60/60 (100.0%) | 7197 | 7197/7197 (100.0%) |
| `Layout/IndentationStyle` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 26 | 26/26 (100.0%) | 22 | 7/22 (31.8%) |
| `Layout/IndentationWidth` | [`crates/rustocop/src/cops/prism/restored_layout_indentation/core.rs`](../crates/rustocop/src/cops/prism/restored_layout_indentation/core.rs) | 2026-08-24 | 198 | 198/198 (100.0%) | 8652 | 7902/9402 (84.0%) |
| `Layout/InitialIndentation` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 2 | 2/2 (100.0%) |
| `Layout/LeadingCommentSpace` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 29 | 29/29 (100.0%) | 538 | 538/538 (100.0%) |
| `Layout/LeadingEmptyLines` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 14 | 14/14 (100.0%) |
| `Layout/LineContinuationLeadingSpace` | [`crates/rustocop/src/cops/prism/final_layout_batch_a/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a/registry.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 117 | 114/117 (97.4%) |
| `Layout/LineContinuationSpacing` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 349 | 349/350 (99.7%) |
| `Layout/LineEndStringConcatenationIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_a/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a/registry.rs) | 2026-08-24 | 59 | 59/59 (100.0%) | 2646 | 1979/3838 (51.6%) |
| `Layout/LineLength` | [`crates/rustocop/src/cops/text/layout.rs`](../crates/rustocop/src/cops/text/layout.rs) | 2026-08-24 | 205 | 205/205 (100.0%) | 181525 ⚠ | 178339/190695 (93.5%) ⚠ stale |
| `Layout/MultilineArrayBraceLayout` | [`crates/rustocop/src/cops/prism/restored_multiline_delimiters.rs`](../crates/rustocop/src/cops/prism/restored_multiline_delimiters.rs) | 2026-08-23 | 35 | 35/35 (100.0%) | 329 | 329/329 (100.0%) |
| `Layout/MultilineArrayLineBreaks` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 19 | 19/19 (100.0%) | 22550 ⚠ | 13010/22885 (56.8%) ⚠ stale |
| `Layout/MultilineAssignmentLayout` | [`crates/rustocop/src/cops/prism/layout_qualification.rs`](../crates/rustocop/src/cops/prism/layout_qualification.rs) | 2026-08-24 | 36 | 36/36 (100.0%) | 17367 | 17234/17395 (99.1%) |
| `Layout/MultilineBlockLayout` | [`crates/rustocop/src/cops/prism/restored_multiline_delimiters.rs`](../crates/rustocop/src/cops/prism/restored_multiline_delimiters.rs) | 2026-08-23 | 30 | 30/30 (100.0%) | 200 | 200/200 (100.0%) |
| `Layout/MultilineHashBraceLayout` | [`crates/rustocop/src/cops/prism/restored_multiline_delimiters.rs`](../crates/rustocop/src/cops/prism/restored_multiline_delimiters.rs) | 2026-08-23 | 34 | 34/34 (100.0%) | 250 | 250/250 (100.0%) |
| `Layout/MultilineHashKeyLineBreaks` | [`crates/rustocop/src/cops/prism/layout_line_break_completion.rs`](../crates/rustocop/src/cops/prism/layout_line_break_completion.rs) | 2026-08-24 | 10 | 10/10 (100.0%) | 3242 | 3242/3242 (100.0%) |
| `Layout/MultilineMethodArgumentLineBreaks` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-24 | 18 | 18/18 (100.0%) | 47523 | 47523/47523 (100.0%) |
| `Layout/MultilineMethodCallBraceLayout` | [`crates/rustocop/src/cops/prism/restored_multiline_delimiters.rs`](../crates/rustocop/src/cops/prism/restored_multiline_delimiters.rs) | 2026-08-23 | 44 | 44/44 (100.0%) | 4092 | 4092/4092 (100.0%) |
| `Layout/MultilineMethodCallIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-24 | 283 | 283/283 (100.0%) | 15130 ⚠ | 9318/27650 (33.7%) ⚠ stale |
| `Layout/MultilineMethodDefinitionBraceLayout` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 27 | 27/27 (100.0%) | 108 | 108/108 (100.0%) |
| `Layout/MultilineMethodParameterLineBreaks` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-24 | 14 | 14/14 (100.0%) | 1961 | 1961/1961 (100.0%) |
| `Layout/MultilineOperationIndentation` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-24 | 103 | 103/103 (100.0%) | 2263 ⚠ | 1629/4996 (32.6%) ⚠ stale |
| `Layout/ParameterAlignment` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-24 | 20 | 20/20 (100.0%) | 20 | 20/20 (100.0%) |
| `Layout/RedundantLineBreak` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-24 | 118 | 118/118 (100.0%) | 31577 ⚠ | 31471/31821 (98.9%) ⚠ stale |
| `Layout/RescueEnsureAlignment` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 105 | 105/105 (100.0%) | 372 | 372/455 (81.8%) |
| `Layout/SingleLineBlockChain` | [`crates/rustocop/src/cops/prism/layout_line_break_completion.rs`](../crates/rustocop/src/cops/prism/layout_line_break_completion.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 34783 | 34783/34783 (100.0%) |
| `Layout/SpaceAfterColon` | [`crates/rustocop/src/cops/prism/layout.rs`](../crates/rustocop/src/cops/prism/layout.rs) | 2026-08-18 | 12 | 12/12 (100.0%) | 39 | 39/39 (100.0%) |
| `Layout/SpaceAfterComma` | [`crates/rustocop/src/cops/prism/source_rules_layout.rs`](../crates/rustocop/src/cops/prism/source_rules_layout.rs) | 2026-08-24 | 21 | 21/21 (100.0%) | 3203 ⚠ | 3203/3203 (100.0%) ⚠ stale |
| `Layout/SpaceAfterMethodName` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 1 | 1/15 (6.7%) |
| `Layout/SpaceAfterNot` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 8 | 8/8 (100.0%) | 89 | 89/89 (100.0%) |
| `Layout/SpaceAfterSemicolon` | [`crates/rustocop/src/cops/prism/source_rules_layout.rs`](../crates/rustocop/src/cops/prism/source_rules_layout.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 13 ⚠ | 13/13 (100.0%) ⚠ stale |
| `Layout/SpaceAroundBlockParameters` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-24 | 45 | 45/45 (100.0%) | 98 ⚠ | 98/98 (100.0%) ⚠ stale |
| `Layout/SpaceAroundEqualsInParameterDefault` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 789 | 789/789 (100.0%) |
| `Layout/SpaceAroundKeyword` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 122 | 122/122 (100.0%) | 232 | 230/239 (96.2%) |
| `Layout/SpaceAroundMethodCallOperator` | [`crates/rustocop/src/cops/prism/operator_method_call_rules.rs`](../crates/rustocop/src/cops/prism/operator_method_call_rules.rs) | 2026-08-21 | 56 | 56/56 (100.0%) | 33 | 33/33 (100.0%) |
| `Layout/SpaceAroundOperators` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 102 | 102/102 (100.0%) | 7084 | 6988/7117 (98.2%) |
| `Layout/SpaceBeforeBlockBraces` | [`crates/rustocop/src/cops/prism/layout_geometry_completion.rs`](../crates/rustocop/src/cops/prism/layout_geometry_completion.rs) | 2026-08-24 | 19 | 19/19 (100.0%) | 2527 | 2527/2527 (100.0%) |
| `Layout/SpaceBeforeBrackets` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-24 | 29 | 29/29 (100.0%) | 1 | 1/1 (100.0%) |
| `Layout/SpaceBeforeComma` | [`crates/rustocop/src/cops/prism/source_rules_layout.rs`](../crates/rustocop/src/cops/prism/source_rules_layout.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 92 ⚠ | 92/92 (100.0%) ⚠ stale |
| `Layout/SpaceBeforeComment` | [`crates/rustocop/src/cops/prism/source_rules_layout.rs`](../crates/rustocop/src/cops/prism/source_rules_layout.rs) | 2026-08-24 | 5 | 5/5 (100.0%) | 5 ⚠ | 5/5 (100.0%) ⚠ stale |
| `Layout/SpaceBeforeFirstArg` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 61 | 57/65 (87.7%) |
| `Layout/SpaceBeforeSemicolon` | [`crates/rustocop/src/cops/prism/source_rules_layout.rs`](../crates/rustocop/src/cops/prism/source_rules_layout.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 337 ⚠ | 337/337 (100.0%) ⚠ stale |
| `Layout/SpaceInLambdaLiteral` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 17 | 17/17 (100.0%) | 283 | 283/283 (100.0%) |
| `Layout/SpaceInsideArrayLiteralBrackets` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-24 | 101 | 101/101 (100.0%) | 3118 ⚠ | 3115/3118 (99.9%) ⚠ stale |
| `Layout/SpaceInsideArrayPercentLiteral` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 94 | 94/94 (100.0%) | 112 | 110/123 (89.4%) |
| `Layout/SpaceInsideBlockBraces` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 44 | 44/44 (100.0%) | 4193 | 4193/4193 (100.0%) |
| `Layout/SpaceInsideHashLiteralBraces` | [`crates/rustocop/src/cops/prism/final_layout_batch_a/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a/registry.rs) | 2026-08-24 | 40 | 40/40 (100.0%) | 23154 | 23154/23154 (100.0%) |
| `Layout/SpaceInsideParens` | [`crates/rustocop/src/cops/prism/final_layout_batch_a.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_a.rs) | 2026-08-24 | 31 | 31/31 (100.0%) | 1012 | 958/1013 (94.6%) |
| `Layout/SpaceInsidePercentLiteralDelimiters` | [`crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b/registry.rs) | 2026-08-23 | 218 | 218/218 (100.0%) | 995 | 995/995 (100.0%) |
| `Layout/SpaceInsideRangeLiteral` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 18 | 18/18 (100.0%) |
| `Layout/SpaceInsideReferenceBrackets` | [`crates/rustocop/src/cops/prism/final_layout_batch_b.rs`](../crates/rustocop/src/cops/prism/final_layout_batch_b.rs) | 2026-08-24 | 49 | 49/49 (100.0%) | 18 ⚠ | 18/18 (100.0%) ⚠ stale |
| `Layout/SpaceInsideStringInterpolation` | [`crates/rustocop/src/cops/prism/layout_spacing_completion.rs`](../crates/rustocop/src/cops/prism/layout_spacing_completion.rs) | 2026-08-24 | 14 | 14/14 (100.0%) | 117 | 117/117 (100.0%) |
| `Layout/TrailingEmptyLines` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 21 | 21/21 (100.0%) | 248 | 248/248 (100.0%) |
| `Layout/TrailingWhitespace` | [`crates/rustocop/src/cops/text/layout.rs`](../crates/rustocop/src/cops/text/layout.rs) | 2026-08-24 | 20 | 20/20 (100.0%) | 1536 ⚠ | 1536/1536 (100.0%) ⚠ stale |
| `Lint/AmbiguousAssignment` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-24 | 26 | 26/26 (100.0%) | 0 | 0/1 (0.0%) |
| `Lint/AmbiguousBlockAssociation` | [`crates/rustocop/src/cops/prism/block_association_rules.rs`](../crates/rustocop/src/cops/prism/block_association_rules.rs) | 2026-08-23 | 40 | 40/40 (100.0%) | 5623 | 5623/5623 (100.0%) |
| `Lint/AmbiguousOperator` | [`crates/rustocop/src/cops/prism/operator_ambiguity_rules.rs`](../crates/rustocop/src/cops/prism/operator_ambiguity_rules.rs) | 2026-08-24 | 21 | 21/21 (100.0%) | 332 | 332/337 (98.5%) |
| `Lint/AmbiguousOperatorPrecedence` | [`crates/rustocop/src/cops/prism/operator_ambiguity_rules.rs`](../crates/rustocop/src/cops/prism/operator_ambiguity_rules.rs) | 2026-08-24 | 17 | 17/17 (100.0%) | 1007 | 1007/1011 (99.6%) |
| `Lint/AmbiguousRange` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-24 | 60 | 60/60 (100.0%) | 164 ⚠ | 98/522 (18.8%) ⚠ stale |
| `Lint/AmbiguousRegexpLiteral` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-24 | 30 | 30/30 (100.0%) | 722 | 722/3450 (20.9%) |
| `Lint/ArrayLiteralInRegexp` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 0 | — (unexercised) |
| `Lint/AssignmentInCondition` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 69 | 69/69 (100.0%) | 1945 | 1944/1945 (99.9%) |
| `Lint/BigDecimalNew` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-24 | 4 | 4/4 (100.0%) | 0 | 0/10 (0.0%) |
| `Lint/BinaryOperatorWithIdenticalOperands` | [`crates/rustocop/src/cops/prism/lint_suspicious_calls.rs`](../crates/rustocop/src/cops/prism/lint_suspicious_calls.rs) | 2026-08-21 | 24 | 24/24 (100.0%) | 1407 | 1407/1407 (100.0%) |
| `Lint/BooleanSymbol` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 10 | 10/10 (100.0%) | 457 | 457/457 (100.0%) |
| `Lint/CircularArgumentReference` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Lint/ConstantDefinitionInBlock` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-24 | 29 | 29/29 (100.0%) | 883 | 883/886 (99.7%) |
| `Lint/ConstantOverwrittenInRescue` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 0 ⚠ | 0/10 (0.0%) ⚠ stale |
| `Lint/ConstantReassignment` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-24 | 59 | 59/59 (100.0%) | 1 | 1/2 (50.0%) |
| `Lint/ConstantResolution` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-24 | 24 | 24/24 (100.0%) | 849583 ⚠ | 83/849964 (0.0%) ⚠ stale |
| `Lint/CopDirectiveSyntax` | [`crates/rustocop/src/cops/prism/directive_completion.rs`](../crates/rustocop/src/cops/prism/directive_completion.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 225 | 224/225 (99.6%) |
| `Lint/DataDefineOverride` | [`crates/rustocop/src/cops/prism/lint_builtin_overrides.rs`](../crates/rustocop/src/cops/prism/lint_builtin_overrides.rs) | 2026-08-20 | 8 | 8/8 (100.0%) | 2 | 2/2 (100.0%) |
| `Lint/Debugger` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 98 | 98/98 (100.0%) | 8 | 8/8 (100.0%) |
| `Lint/DeprecatedClassMethods` | [`crates/rustocop/src/cops/prism/deprecated_api_rules.rs`](../crates/rustocop/src/cops/prism/deprecated_api_rules.rs) | 2026-08-18 | 29 | 29/29 (100.0%) | 12 | 12/12 (100.0%) |
| `Lint/DeprecatedConstants` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-24 | 21 | 21/21 (100.0%) | 12 | 12/12 (100.0%) |
| `Lint/DeprecatedOpenSSLConstant` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-24 | 25 | 25/25 (100.0%) | 26 | 26/26 (100.0%) |
| `Lint/DisjunctiveAssignmentInConstructor` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 5 | 5/8 (62.5%) |
| `Lint/DuplicateBranch` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-24 | 132 | 132/132 (100.0%) | 397 | 395/398 (99.2%) |
| `Lint/DuplicateCaseCondition` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/DuplicateElsifCondition` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | 0/13 (0.0%) |
| `Lint/DuplicateHashKey` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 37 | 37/37 (100.0%) | 2 | 2/2 (100.0%) |
| `Lint/DuplicateMagicComment` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 0 | 0/4 (0.0%) |
| `Lint/DuplicateMatchPattern` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-24 | 20 | 20/20 (100.0%) | 0 | — (unexercised) |
| `Lint/DuplicateMethods` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 477 | 477/477 (100.0%) | 114 | 112/114 (98.2%) |
| `Lint/DuplicateRegexpCharacterClassElement` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-24 | 16 | 16/16 (100.0%) | 103 | 99/3774 (2.6%) |
| `Lint/DuplicateRequire` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 11 | 11/11 (100.0%) | 5 | 5/5 (100.0%) |
| `Lint/DuplicateRescueException` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 7 | 7/7 (100.0%) | 0 | — (unexercised) |
| `Lint/DuplicateSetElement` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-24 | 38 | 38/38 (100.0%) | 2 | 2/9 (22.2%) |
| `Lint/EachWithObjectArgument` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | 0/5 (0.0%) |
| `Lint/ElseLayout` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 3 | 3/3 (100.0%) |
| `Lint/EmptyBlock` | [`crates/rustocop/src/cops/prism/lint_scope_completion.rs`](../crates/rustocop/src/cops/prism/lint_scope_completion.rs) | 2026-08-23 | 18 | 18/18 (100.0%) | 1639 | 1632/1640 (99.5%) |
| `Lint/EmptyClass` | [`crates/rustocop/src/cops/prism/empty_class_rules.rs`](../crates/rustocop/src/cops/prism/empty_class_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 77 | 77/78 (98.7%) |
| `Lint/EmptyConditionalBody` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-24 | 40 | 40/40 (100.0%) | 9 | 9/9 (100.0%) |
| `Lint/EmptyEnsure` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-24 | 3 | 3/3 (100.0%) | 0 | 0/6 (0.0%) |
| `Lint/EmptyExpression` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 1 | 0/1 (0.0%) |
| `Lint/EmptyFile` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 25 | 25/25 (100.0%) |
| `Lint/EmptyInPattern` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 2 | 2/2 (100.0%) |
| `Lint/EmptyInterpolation` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 3 | 2/3 (66.7%) |
| `Lint/EmptyWhen` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 42 | 42/42 (100.0%) |
| `Lint/EnsureReturn` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 1 | 0/8 (0.0%) |
| `Lint/ErbNewArguments` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 10 | 10/10 (100.0%) | 8 ⚠ | 8/46 (17.4%) ⚠ stale |
| `Lint/FlipFlop` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 2 | 2/2 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/FloatComparison` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 17 | 17/17 (100.0%) | 119 | 119/121 (98.3%) |
| `Lint/FloatOutOfRange` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Lint/FormatParameterMismatch` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-24 | 76 | 76/76 (100.0%) | 0 ⚠ | 0/1 (0.0%) ⚠ stale |
| `Lint/HashCompareByIdentity` | [`crates/rustocop/src/cops/prism/lint_suspicious_calls.rs`](../crates/rustocop/src/cops/prism/lint_suspicious_calls.rs) | 2026-08-21 | 4 | 4/4 (100.0%) | 11 | 11/11 (100.0%) |
| `Lint/HashNewWithKeywordArgumentsAsDefault` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 11 | 11/11 (100.0%) | 0 ⚠ | — (unexercised) ⚠ stale |
| `Lint/HeredocMethodCallPosition` | [`crates/rustocop/src/cops/prism/heredoc_call_rules.rs`](../crates/rustocop/src/cops/prism/heredoc_call_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 3 | 0/3 (0.0%) |
| `Lint/IdentityComparison` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 8 | 8/8 (100.0%) |
| `Lint/ImplicitStringConcatenation` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 17 | 17/17 (100.0%) |
| `Lint/IncompatibleIoSelectWithFiberScheduler` | [`crates/rustocop/src/cops/prism/io_scheduler_rules.rs`](../crates/rustocop/src/cops/prism/io_scheduler_rules.rs) | 2026-08-18 | 19 | 19/19 (100.0%) | 19 | 19/19 (100.0%) |
| `Lint/IneffectiveAccessModifier` | [`crates/rustocop/src/cops/prism/declaration_semantics.rs`](../crates/rustocop/src/cops/prism/declaration_semantics.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 414 | 414/414 (100.0%) |
| `Lint/InheritException` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 13 | 13/13 (100.0%) | 31 | 31/31 (100.0%) |
| `Lint/InterpolationCheck` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 20 | 20/20 (100.0%) | 290 | 290/290 (100.0%) |
| `Lint/ItWithoutArgumentsInBlock` | [`crates/rustocop/src/cops/prism/block_parameter_rules.rs`](../crates/rustocop/src/cops/prism/block_parameter_rules.rs) | 2026-08-18 | 19 | 19/19 (100.0%) | 0 | — (unexercised) |
| `Lint/LambdaWithoutLiteralBlock` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 6 | 6/6 (100.0%) | 0 ⚠ | 0/7 (0.0%) ⚠ stale |
| `Lint/LiteralAsCondition` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-24 | 219 | 219/219 (100.0%) | 18 | 17/18 (94.4%) |
| `Lint/LiteralAssignmentInCondition` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 35 | 35/35 (100.0%) | 17 | 17/17 (100.0%) |
| `Lint/LiteralInInterpolation` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 380 | 380/380 (100.0%) | 45 | 45/48 (93.8%) |
| `Lint/Loop` | [`crates/rustocop/src/cops/prism/lint_control_flow.rs`](../crates/rustocop/src/cops/prism/lint_control_flow.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 27 | 27/27 (100.0%) |
| `Lint/MissingCopEnableDirective` | [`crates/rustocop/src/cops/prism/directive_completion.rs`](../crates/rustocop/src/cops/prism/directive_completion.rs) | 2026-08-23 | 19 | 19/19 (100.0%) | 98 | 75/2047 (3.7%) |
| `Lint/MissingSuper` | [`crates/rustocop/src/cops/prism/lint_scope_completion.rs`](../crates/rustocop/src/cops/prism/lint_scope_completion.rs) | 2026-08-23 | 23 | 23/23 (100.0%) | 839 | 839/842 (99.6%) |
| `Lint/MixedCaseRange` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-24 | 31 | 31/31 (100.0%) | 15 | 15/15 (100.0%) |
| `Lint/MixedRegexpCaptureTypes` | [`crates/rustocop/src/cops/prism/lint_builtin_overrides.rs`](../crates/rustocop/src/cops/prism/lint_builtin_overrides.rs) | 2026-08-20 | 13 | 13/13 (100.0%) | 32 | 32/32 (100.0%) |
| `Lint/MultipleComparison` | [`crates/rustocop/src/cops/prism/logical_condition_rules.rs`](../crates/rustocop/src/cops/prism/logical_condition_rules.rs) | 2026-08-20 | 20 | 20/20 (100.0%) | 0 | — (unexercised) |
| `Lint/NestedMethodDefinition` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-24 | 38 | 38/38 (100.0%) | 113 ⚠ | 113/117 (96.6%) ⚠ stale |
| `Lint/NestedPercentLiteral` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 6 | 6/6 (100.0%) |
| `Lint/NextWithoutAccumulator` | [`crates/rustocop/src/cops/prism/block_arity_rules.rs`](../crates/rustocop/src/cops/prism/block_arity_rules.rs) | 2026-08-18 | 18 | 18/18 (100.0%) | 0 | — (unexercised) |
| `Lint/NoReturnInBeginEndBlocks` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 70 | 70/70 (100.0%) | 49 | 49/49 (100.0%) |
| `Lint/NonAtomicFileOperation` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-24 | 45 | 45/45 (100.0%) | 259 ⚠ | 249/268 (92.9%) ⚠ stale |
| `Lint/NonDeterministicRequireOrder` | [`crates/rustocop/src/cops/prism/non_deterministic_require_rules.rs`](../crates/rustocop/src/cops/prism/non_deterministic_require_rules.rs) | 2026-08-18 | 28 | 28/28 (100.0%) | 0 | — (unexercised) |
| `Lint/NonLocalExitFromIterator` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 14 | 14/14 (100.0%) | 30 | 30/30 (100.0%) |
| `Lint/NumberConversion` | [`crates/rustocop/src/cops/prism/number_conversion_rules.rs`](../crates/rustocop/src/cops/prism/number_conversion_rules.rs) | 2026-08-21 | 38 | 38/38 (100.0%) | 10808 | 10806/10808 (100.0%) |
| `Lint/NumberedParameterAssignment` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Lint/NumericOperationWithConstantResult` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-24 | 16 | 16/16 (100.0%) | 0 | 0/12 (0.0%) |
| `Lint/OrAssignmentToConstant` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 172 | 136/195 (69.7%) |
| `Lint/OrderedMagicComments` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 23 | 23/23 (100.0%) |
| `Lint/OutOfRangeRegexpRef` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-24 | 124 | 124/124 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/ParenthesesAsGroupedExpression` | [`crates/rustocop/src/cops/prism/operator_ambiguity_rules.rs`](../crates/rustocop/src/cops/prism/operator_ambiguity_rules.rs) | 2026-08-24 | 30 | 30/30 (100.0%) | 230 | 229/230 (99.6%) |
| `Lint/PercentStringArray` | [`crates/rustocop/src/cops/prism/percent_string_rules.rs`](../crates/rustocop/src/cops/prism/percent_string_rules.rs) | 2026-08-23 | 22 | 22/22 (100.0%) | 14 | 14/14 (100.0%) |
| `Lint/PercentSymbolArray` | [`crates/rustocop/src/cops/prism/percent_string_rules.rs`](../crates/rustocop/src/cops/prism/percent_string_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Lint/RaiseException` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 15 | 15/15 (100.0%) | 23 | 23/23 (100.0%) |
| `Lint/RandOne` | [`crates/rustocop/src/cops/prism/lint_suspicious_calls.rs`](../crates/rustocop/src/cops/prism/lint_suspicious_calls.rs) | 2026-08-21 | 16 | 16/16 (100.0%) | 0 | — (unexercised) |
| `Lint/RedundantCopDisableDirective` | [`crates/rustocop/src/cops/prism/final_file_metadata_batch.rs`](../crates/rustocop/src/cops/prism/final_file_metadata_batch.rs) | 2026-08-24 | 25 | 25/25 (100.0%) | — | — (rubocop_error) |
| `Lint/RedundantCopEnableDirective` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-24 | 28 | 28/28 (100.0%) | 20 | 20/176 (11.4%) |
| `Lint/RedundantDirGlobSort` | [`crates/rustocop/src/cops/prism/path_and_literal_rules.rs`](../crates/rustocop/src/cops/prism/path_and_literal_rules.rs) | 2026-08-23 | 16 | 16/16 (100.0%) | 48 | 48/48 (100.0%) |
| `Lint/RedundantRegexpQuantifiers` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-24 | 27 | 27/27 (100.0%) | 0 | 0/2 (0.0%) |
| `Lint/RedundantRequireStatement` | [`crates/rustocop/src/cops/prism/require_rules.rs`](../crates/rustocop/src/cops/prism/require_rules.rs) | 2026-08-18 | 15 | 15/15 (100.0%) | 40 | 40/40 (100.0%) |
| `Lint/RedundantSafeNavigation` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-24 | 92 | 92/92 (100.0%) | 43 | 42/43 (97.7%) |
| `Lint/RedundantSplatExpansion` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-24 | 60 | 60/60 (100.0%) | 55 ⚠ | 55/55 (100.0%) ⚠ stale |
| `Lint/RedundantStringCoercion` | [`crates/rustocop/src/cops/prism/coercion_rules.rs`](../crates/rustocop/src/cops/prism/coercion_rules.rs) | 2026-08-18 | 18 | 18/18 (100.0%) | 107 | 107/107 (100.0%) |
| `Lint/RedundantTypeConversion` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 611 | 611/611 (100.0%) | 14 | 14/14 (100.0%) |
| `Lint/RedundantWithIndex` | [`crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs`](../crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs) | 2026-08-24 | 17 | 17/17 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/RedundantWithObject` | [`crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs`](../crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs) | 2026-08-24 | 14 | 14/14 (100.0%) | 0 | — (unexercised) |
| `Lint/RefinementImportMethods` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 0 | 0/6 (0.0%) |
| `Lint/RegexpAsCondition` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 5 | 5/5 (100.0%) | 0 | — (unexercised) |
| `Lint/RequireParentheses` | [`crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs`](../crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs) | 2026-08-23 | 17 | 17/17 (100.0%) | 0 | — (unexercised) |
| `Lint/RequireRangeParentheses` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 0 | 0/5 (0.0%) |
| `Lint/RequireRelativeSelfPath` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 0 ⚠ | — (unexercised) ⚠ stale |
| `Lint/RescueException` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 191 | 191/240 (79.6%) |
| `Lint/RescueType` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 52 | 52/52 (100.0%) | 0 | — (unexercised) |
| `Lint/ReturnInVoidContext` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-24 | 19 | 19/19 (100.0%) | 24 ⚠ | 24/24 (100.0%) ⚠ stale |
| `Lint/SafeNavigationChain` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-24 | 67 | 67/67 (100.0%) | 3 | 3/3 (100.0%) |
| `Lint/SafeNavigationConsistency` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-24 | 45 | 45/45 (100.0%) | 2 | 2/2 (100.0%) |
| `Lint/SafeNavigationWithEmpty` | [`crates/rustocop/src/cops/prism/lint_control_flow.rs`](../crates/rustocop/src/cops/prism/lint_control_flow.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 3 | 3/3 (100.0%) |
| `Lint/ScriptPermission` | [`crates/rustocop/src/cops/prism/final_file_metadata_batch.rs`](../crates/rustocop/src/cops/prism/final_file_metadata_batch.rs) | 2026-08-24 | 7 | 7/7 (100.0%) | 10 | 10/10 (100.0%) |
| `Lint/SelfAssignment` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 60 | 60/60 (100.0%) | 18 | 18/48 (37.5%) |
| `Lint/SendWithMixinArgument` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 11 | 11/11 (100.0%) |
| `Lint/ShadowedArgument` | [`crates/rustocop/src/cops/prism/final_scope_batch_a/naming.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a/naming.rs) | 2026-08-24 | 56 | 56/56 (100.0%) | 101 | 101/101 (100.0%) |
| `Lint/ShadowedException` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-24 | 41 | 41/41 (100.0%) | 39 | 22/39 (56.4%) |
| `Lint/ShadowingOuterLocalVariable` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-24 | 33 | 33/33 (100.0%) | 386 | 383/388 (98.7%) |
| `Lint/SharedMutableDefault` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 6 | 6/6 (100.0%) | 11 ⚠ | 8/26 (30.8%) ⚠ stale |
| `Lint/StructNewOverride` | [`crates/rustocop/src/cops/prism/lint_builtin_overrides.rs`](../crates/rustocop/src/cops/prism/lint_builtin_overrides.rs) | 2026-08-20 | 10 | 10/10 (100.0%) | 23 | 23/23 (100.0%) |
| `Lint/SuppressedException` | [`crates/rustocop/src/cops/prism/rescue_rules.rs`](../crates/rustocop/src/cops/prism/rescue_rules.rs) | 2026-08-21 | 25 | 25/25 (100.0%) | 459 | 459/459 (100.0%) |
| `Lint/SuppressedExceptionInNumberConversion` | [`crates/rustocop/src/cops/prism/exception_location_completion.rs`](../crates/rustocop/src/cops/prism/exception_location_completion.rs) | 2026-08-24 | 27 | 27/27 (100.0%) | 4 | 4/34 (11.8%) |
| `Lint/SymbolConversion` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-24 | 46 | 46/46 (100.0%) | 1888 | 1888/1888 (100.0%) |
| `Lint/Syntax` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-24 | 5 | 5/5 (100.0%) | 60 ⚠ | 60/61 (98.4%) ⚠ stale |
| `Lint/ToEnumArguments` | [`crates/rustocop/src/cops/prism/enum_argument_rules.rs`](../crates/rustocop/src/cops/prism/enum_argument_rules.rs) | 2026-08-20 | 25 | 25/25 (100.0%) | 10 | 7/10 (70.0%) |
| `Lint/ToJSON` | [`crates/rustocop/src/cops/prism/lint.rs`](../crates/rustocop/src/cops/prism/lint.rs) | 2026-08-24 | 2 | 2/2 (100.0%) | 14 | 14/14 (100.0%) |
| `Lint/TopLevelReturnWithArgument` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Lint/TrailingCommaInAttributeDeclaration` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-24 | 3 | 3/3 (100.0%) | 0 | — (unexercised) |
| `Lint/TripleQuotes` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 0 | — (unexercised) |
| `Lint/UnderscorePrefixedVariableName` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-24 | 19 | 19/19 (100.0%) | 2109 | 897/2248 (39.9%) |
| `Lint/UnescapedBracketInRegexp` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-24 | 46 | 46/46 (100.0%) | 0 | — (unexercised) |
| `Lint/UnexpectedBlockArity` | [`crates/rustocop/src/cops/prism/block_arity_rules.rs`](../crates/rustocop/src/cops/prism/block_arity_rules.rs) | 2026-08-18 | 21 | 21/21 (100.0%) | 11 | 11/13 (84.6%) |
| `Lint/UnifiedInteger` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 7 | 7/7 (100.0%) |
| `Lint/UnmodifiedReduceAccumulator` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-24 | 166 | 166/166 (100.0%) | 1 ⚠ | 1/1 (100.0%) ⚠ stale |
| `Lint/UnreachableCode` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-24 | 267 | 267/267 (100.0%) | 9 | 9/9 (100.0%) |
| `Lint/UnreachableLoop` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-24 | 29 | 29/29 (100.0%) | 629 | 618/633 (97.6%) |
| `Lint/UnreachablePatternBranch` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-24 | 24 | 24/24 (100.0%) | 0 | 0/13 (0.0%) |
| `Lint/UnusedBlockArgument` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-24 | 31 | 31/31 (100.0%) | 3150 ⚠ | 3150/3150 (100.0%) ⚠ stale |
| `Lint/UnusedMethodArgument` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs)<br>[`crates/rustocop/src/cops/text/lint_semantic.rs`](../crates/rustocop/src/cops/text/lint_semantic.rs) | 2026-08-24 | 44 | 44/44 (100.0%) | 3598 ⚠ | 3284/11466 (28.6%) ⚠ stale |
| `Lint/UriEscapeUnescape` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 3 | 0/3 (0.0%) |
| `Lint/UriRegexp` | [`crates/rustocop/src/cops/prism/uri_regexp_rules.rs`](../crates/rustocop/src/cops/prism/uri_regexp_rules.rs) | 2026-08-21 | 11 | 11/11 (100.0%) | 12 | 12/12 (100.0%) |
| `Lint/UselessAccessModifier` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 193 | 193/193 (100.0%) | 127 | 126/127 (99.2%) |
| `Lint/UselessAssignment` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 149 | 149/149 (100.0%) | 1313 | 696/1966 (35.4%) |
| `Lint/UselessConstantScoping` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 529 ⚠ | 318/1169 (27.2%) ⚠ stale |
| `Lint/UselessDefaultValueArgument` | [`crates/rustocop/src/cops/prism/fetch_completion_rules.rs`](../crates/rustocop/src/cops/prism/fetch_completion_rules.rs) | 2026-08-22 | 25 | 25/25 (100.0%) | 19 | 18/19 (94.7%) |
| `Lint/UselessDefined` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 3 | 3/14 (21.4%) |
| `Lint/UselessElseWithoutRescue` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-24 | 3 | 3/3 (100.0%) | 0 | — (unexercised) |
| `Lint/UselessMethodDefinition` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-24 | 18 | 18/18 (100.0%) | 76 ⚠ | 74/79 (93.7%) ⚠ stale |
| `Lint/UselessNumericOperation` | [`crates/rustocop/src/cops/prism/numeric_operation_rules.rs`](../crates/rustocop/src/cops/prism/numeric_operation_rules.rs) | 2026-08-18 | 13 | 13/13 (100.0%) | 1 | 1/4 (25.0%) |
| `Lint/UselessOr` | [`crates/rustocop/src/cops/prism/final_control_flow_batch.rs`](../crates/rustocop/src/cops/prism/final_control_flow_batch.rs) | 2026-08-24 | 127 | 127/127 (100.0%) | 43 | 43/43 (100.0%) |
| `Lint/UselessRescue` | [`crates/rustocop/src/cops/prism/rescue_rules.rs`](../crates/rustocop/src/cops/prism/rescue_rules.rs) | 2026-08-21 | 12 | 12/12 (100.0%) | 10 | 6/10 (60.0%) |
| `Lint/UselessRuby2Keywords` | [`crates/rustocop/src/cops/prism/ruby2_keywords_rules.rs`](../crates/rustocop/src/cops/prism/ruby2_keywords_rules.rs) | 2026-08-18 | 23 | 23/23 (100.0%) | 0 | — (unexercised) |
| `Lint/UselessSetterCall` | [`crates/rustocop/src/cops/prism/setter_rules.rs`](../crates/rustocop/src/cops/prism/setter_rules.rs) | 2026-08-18 | 20 | 20/20 (100.0%) | 1 | 1/1 (100.0%) |
| `Lint/UselessTimes` | [`crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs`](../crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs) | 2026-08-24 | 25 | 25/25 (100.0%) | 10 | 10/10 (100.0%) |
| `Lint/Void` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 270 | 270/270 (100.0%) | 62 | 62/64 (96.9%) |
| `Metrics/AbcSize` | [`crates/rustocop/src/cops/prism/metrics_completion.rs`](../crates/rustocop/src/cops/prism/metrics_completion.rs) | 2026-08-25 | 33 | 33/33 (100.0%) | 55118 ⚠ | 1448/114574 (1.3%) ⚠ stale |
| `Metrics/BlockLength` | [`crates/rustocop/src/cops/prism/metrics_completion.rs`](../crates/rustocop/src/cops/prism/metrics_completion.rs) | 2026-08-25 | 33 | 33/33 (100.0%) | 86297 ⚠ | 84997/87355 (97.3%) ⚠ stale |
| `Metrics/BlockNesting` | [`crates/rustocop/src/cops/prism/final_metrics_batch.rs`](../crates/rustocop/src/cops/prism/final_metrics_batch.rs) | 2026-08-24 | 26 | 26/26 (100.0%) | 663 | 538/756 (71.2%) |
| `Metrics/ClassLength` | [`crates/rustocop/src/cops/prism/final_metrics_batch.rs`](../crates/rustocop/src/cops/prism/final_metrics_batch.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 5020 | 4038/6012 (67.2%) |
| `Metrics/CollectionLiteralLength` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 21 | 21/21 (100.0%) |
| `Metrics/CyclomaticComplexity` | [`crates/rustocop/src/cops/prism/final_metrics_batch.rs`](../crates/rustocop/src/cops/prism/final_metrics_batch.rs) | 2026-08-24 | 36 | 36/36 (100.0%) | 8047 | 5292/10073 (52.5%) |
| `Metrics/MethodLength` | [`crates/rustocop/src/cops/prism/metrics_completion.rs`](../crates/rustocop/src/cops/prism/metrics_completion.rs) | 2026-08-25 | 38 | 38/38 (100.0%) | 36690 ⚠ | 32344/40976 (78.9%) ⚠ stale |
| `Metrics/ModuleLength` | [`crates/rustocop/src/cops/prism/project_structural_completion_batch.rs`](../crates/rustocop/src/cops/prism/project_structural_completion_batch.rs) | 2026-08-24 | 21 | 21/21 (100.0%) | 1201 | 1195/1216 (98.3%) |
| `Metrics/ParameterLists` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-24 | 18 | 18/18 (100.0%) | 1558 | 1551/1576 (98.4%) |
| `Metrics/PerceivedComplexity` | [`crates/rustocop/src/cops/prism/final_metrics_batch.rs`](../crates/rustocop/src/cops/prism/final_metrics_batch.rs) | 2026-08-24 | 31 | 31/31 (100.0%) | 6450 | 3380/9149 (36.9%) |
| `Migration/DepartmentName` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 1 | 0/19 (0.0%) |
| `Naming/AccessorMethodName` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-24 | 24 | 24/24 (100.0%) | 996 | 996/996 (100.0%) |
| `Naming/AsciiIdentifiers` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 10 | 10/10 (100.0%) |
| `Naming/BinaryOperatorParameterName` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 186 | 183/187 (97.9%) |
| `Naming/BlockForwarding` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-24 | 37 | 37/37 (100.0%) | 4852 | 4852/4855 (99.9%) |
| `Naming/BlockParameterName` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 14 | 7/6003 (0.1%) |
| `Naming/ClassAndModuleCamelCase` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 234 | 234/235 (99.6%) |
| `Naming/ConstantName` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-24 | 27 | 27/27 (100.0%) | 353 | 353/353 (100.0%) |
| `Naming/FileName` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 122 | 122/122 (100.0%) | 469 | 369/569 (64.9%) |
| `Naming/HeredocDelimiterCase` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-24 | 27 | 27/27 (100.0%) | 154 | 154/154 (100.0%) |
| `Naming/HeredocDelimiterNaming` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-24 | 22 | 21/22 (95.5%) | 2184 | 2184/2356 (92.7%) |
| `Naming/InclusiveLanguage` | [`crates/rustocop/src/cops/prism/final_scope_batch_a/naming.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a/naming.rs) | 2026-08-24 | 36 | 36/36 (100.0%) | 480 | 474/482 (98.3%) |
| `Naming/MemoizedInstanceVariableName` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 74 | 74/74 (100.0%) | 554 | 554/555 (99.8%) |
| `Naming/MethodName` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 203 | 203/203 (100.0%) | 1903 | 1903/1903 (100.0%) |
| `Naming/MethodParameterName` | [`crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_naming_completion_batch.rs) | 2026-08-24 | 24 | 24/24 (100.0%) | 3353 | 3353/3353 (100.0%) |
| `Naming/PredicateMethod` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 1264 | 1264/1264 (100.0%) | 2217 | 2217/2217 (100.0%) |
| `Naming/PredicatePrefix` | [`crates/rustocop/src/cops/prism/metrics_naming_completion.rs`](../crates/rustocop/src/cops/prism/metrics_naming_completion.rs) | 2026-08-24 | 25 | 25/25 (100.0%) | 3560 | 3560/3560 (100.0%) |
| `Naming/RescuedExceptionsVariableName` | [`crates/rustocop/src/cops/prism/final_scope_batch_a.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_a.rs) | 2026-08-24 | 37 | 37/37 (100.0%) | 1195 | 1177/1196 (98.4%) |
| `Naming/VariableName` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 118 | 118/118 (100.0%) | 530 | 530/1513 (35.0%) |
| `Naming/VariableNumber` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 115 | 115/115 (100.0%) | 9963 | 9545/13500 (70.7%) |
| `Security/CompoundHash` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 22 | 22/22 (100.0%) | 26 | 26/29 (89.7%) |
| `Security/Eval` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 24 | 24/24 (100.0%) | 110 | 110/110 (100.0%) |
| `Security/IoMethods` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 38 | 38/38 (100.0%) | 14 | 14/14 (100.0%) |
| `Security/JSONLoad` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 16 | 16/16 (100.0%) | 48 | 48/48 (100.0%) |
| `Security/MarshalLoad` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 12 | 12/12 (100.0%) | 55 | 55/56 (98.2%) |
| `Security/Open` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 22 | 22/22 (100.0%) | 32 | 32/32 (100.0%) |
| `Security/YAMLLoad` | [`crates/rustocop/src/cops/prism/security.rs`](../crates/rustocop/src/cops/prism/security.rs) | 2026-08-20 | 8 | 8/8 (100.0%) | 0 | — (unexercised) |
| `Style/AccessModifierDeclarations` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 269 | 269/269 (100.0%) | 42 | 41/43 (95.3%) |
| `Style/AccessorGrouping` | [`crates/rustocop/src/cops/prism/accessor_grouping_completion.rs`](../crates/rustocop/src/cops/prism/accessor_grouping_completion.rs) | 2026-08-24 | 36 | 36/36 (100.0%) | 1978 | 1968/2034 (96.8%) |
| `Style/Alias` | [`crates/rustocop/src/cops/prism/alias_rules.rs`](../crates/rustocop/src/cops/prism/alias_rules.rs) | 2026-08-21 | 32 | 32/32 (100.0%) | 1719 | 1715/1719 (99.8%) |
| `Style/AmbiguousEndlessMethodDefinition` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 0 ⚠ | — (unexercised) ⚠ stale |
| `Style/AndOr` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-24 | 79 | 79/79 (100.0%) | 638 | 638/638 (100.0%) |
| `Style/ArgumentsForwarding` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 186 | 186/186 (100.0%) | 7614 | 7602/7614 (99.8%) |
| `Style/ArrayCoercion` | [`crates/rustocop/src/cops/prism/structural_forwarding_completion.rs`](../crates/rustocop/src/cops/prism/structural_forwarding_completion.rs) | 2026-08-24 | 7 | 7/7 (100.0%) | 130 | 130/130 (100.0%) |
| `Style/ArrayFirstLast` | [`crates/rustocop/src/cops/prism/style_collections.rs`](../crates/rustocop/src/cops/prism/style_collections.rs) | 2026-08-23 | 19 | 19/19 (100.0%) | 7093 | 7092/7093 (100.0%) |
| `Style/ArrayIntersect` | [`crates/rustocop/src/cops/prism/modern_collection_completion.rs`](../crates/rustocop/src/cops/prism/modern_collection_completion.rs) | 2026-08-24 | 83 | 83/83 (100.0%) | 119 | 119/119 (100.0%) |
| `Style/ArrayIntersectWithSingleElement` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 1 | 1/5 (20.0%) |
| `Style/ArrayJoin` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 15 | 15/15 (100.0%) |
| `Style/AsciiComments` | [`crates/rustocop/src/cops/prism/source_rules_misc.rs`](../crates/rustocop/src/cops/prism/source_rules_misc.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 4696 | 4696/4699 (99.9%) |
| `Style/Attr` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-24 | 11 | 11/11 (100.0%) | 7 | 7/7 (100.0%) |
| `Style/AutoResourceCleanup` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 86 | 86/548 (15.7%) |
| `Style/BarePercentLiterals` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 22 | 22/22 (100.0%) | 288 | 288/288 (100.0%) |
| `Style/BeginBlock` | [`crates/rustocop/src/cops/prism/style/misc.rs`](../crates/rustocop/src/cops/prism/style/misc.rs) | 2026-08-19 | 1 | 1/1 (100.0%) | 0 | — (unexercised) |
| `Style/BisectedAttrAccessor` | [`crates/rustocop/src/cops/prism/accessor_rules.rs`](../crates/rustocop/src/cops/prism/accessor_rules.rs) | 2026-08-18 | 14 | 14/14 (100.0%) | 22 | 22/22 (100.0%) |
| `Style/BitwisePredicate` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 19 | 19/19 (100.0%) | 54 | 54/54 (100.0%) |
| `Style/BlockComments` | [`crates/rustocop/src/cops/prism/block_comments_rules.rs`](../crates/rustocop/src/cops/prism/block_comments_rules.rs) | 2026-08-21 | 6 | 6/6 (100.0%) | 41 | 38/41 (92.7%) |
| `Style/BlockDelimiters` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-24 | 174 | 174/174 (100.0%) | 7814 | 7804/7816 (99.8%) |
| `Style/CaseEquality` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 21 | 21/21 (100.0%) | 869 | 869/870 (99.9%) |
| `Style/CaseLikeIf` | [`crates/rustocop/src/cops/prism/structural_next_completion.rs`](../crates/rustocop/src/cops/prism/structural_next_completion.rs) | 2026-08-24 | 40 | 40/40 (100.0%) | 52 | 46/66 (69.7%) |
| `Style/CharacterLiteral` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-24 | 6 | 6/6 (100.0%) | 186 | 186/186 (100.0%) |
| `Style/ClassAndModuleChildren` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-24 | 44 | 44/44 (100.0%) | — | — (rubocop_error) |
| `Style/ClassCheck` | [`crates/rustocop/src/cops/prism/class_check_rules.rs`](../crates/rustocop/src/cops/prism/class_check_rules.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 382 | 382/382 (100.0%) |
| `Style/ClassEqualityComparison` | [`crates/rustocop/src/cops/prism/class_comparison_rules.rs`](../crates/rustocop/src/cops/prism/class_comparison_rules.rs) | 2026-08-21 | 23 | 23/23 (100.0%) | 72 | 70/75 (93.3%) |
| `Style/ClassMethods` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 45 | 25/56 (44.6%) |
| `Style/ClassMethodsDefinitions` | [`crates/rustocop/src/cops/prism/class_methods_completion.rs`](../crates/rustocop/src/cops/prism/class_methods_completion.rs) | 2026-08-21 | 17 | 17/17 (100.0%) | 1292 | 1292/1292 (100.0%) |
| `Style/ClassVars` | [`crates/rustocop/src/cops/prism/class_vars_rules.rs`](../crates/rustocop/src/cops/prism/class_vars_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 558 | 558/558 (100.0%) |
| `Style/CollectionCompact` | [`crates/rustocop/src/cops/prism/collection_completion_rules.rs`](../crates/rustocop/src/cops/prism/collection_completion_rules.rs) | 2026-08-22 | 36 | 36/36 (100.0%) | 28 | 28/28 (100.0%) |
| `Style/CollectionMethods` | [`crates/rustocop/src/cops/prism/collection_transform_batch.rs`](../crates/rustocop/src/cops/prism/collection_transform_batch.rs) | 2026-08-21 | 68 | 68/68 (100.0%) | 1962 | 1962/1962 (100.0%) |
| `Style/CollectionQuerying` | [`crates/rustocop/src/cops/prism/collection_query_rules.rs`](../crates/rustocop/src/cops/prism/collection_query_rules.rs) | 2026-08-18 | 20 | 20/20 (100.0%) | 334 | 334/334 (100.0%) |
| `Style/ColonMethodCall` | [`crates/rustocop/src/cops/prism/style_calls.rs`](../crates/rustocop/src/cops/prism/style_calls.rs) | 2026-08-18 | 10 | 10/10 (100.0%) | 140 | 140/140 (100.0%) |
| `Style/ColonMethodDefinition` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-24 | 4 | 4/4 (100.0%) | 1 | 1/11 (9.1%) |
| `Style/CombinableDefined` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-24 | 41 | 41/41 (100.0%) | 3 | 3/11 (27.3%) |
| `Style/CombinableLoops` | [`crates/rustocop/src/cops/prism/control_flow_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_flow_completion_batch.rs) | 2026-08-24 | 24 | 24/24 (100.0%) | 94 | 91/121 (75.2%) |
| `Style/CommandLiteral` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-24 | 35 | 35/35 (100.0%) | 22 ⚠ | 22/22 (100.0%) ⚠ stale |
| `Style/CommentAnnotation` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 544 ⚠ | 543/565 (96.1%) ⚠ stale |
| `Style/CommentedKeyword` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-24 | 49 | 49/49 (100.0%) | 1044 ⚠ | 1044/1044 (100.0%) ⚠ stale |
| `Style/ComparableBetween` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 15 | 15/15 (100.0%) | 51 | 51/51 (100.0%) |
| `Style/ComparableClamp` | [`crates/rustocop/src/cops/prism/comparable_clamp_rules.rs`](../crates/rustocop/src/cops/prism/comparable_clamp_rules.rs) | 2026-08-18 | 23 | 23/23 (100.0%) | 12 | 12/12 (100.0%) |
| `Style/ConcatArrayLiterals` | [`crates/rustocop/src/cops/prism/collection_completion_rules.rs`](../crates/rustocop/src/cops/prism/collection_completion_rules.rs) | 2026-08-22 | 15 | 15/15 (100.0%) | 144 | 144/144 (100.0%) |
| `Style/ConditionalAssignment` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 1182 | 1182/1182 (100.0%) | 804 | 804/804 (100.0%) |
| `Style/ConstantVisibility` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 29907 | 29907/29907 (100.0%) |
| `Style/Copyright` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-24 | 27 | 27/27 (100.0%) | 85417 ⚠ | 74171/85434 (86.8%) ⚠ stale |
| `Style/DataInheritance` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-24 | 24 | 24/24 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/DateTime` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 13 | 13/13 (100.0%) | 1634 | 1632/1634 (99.9%) |
| `Style/DefWithParentheses` | [`crates/rustocop/src/cops/prism/declaration_semantics.rs`](../crates/rustocop/src/cops/prism/declaration_semantics.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 38 | 38/38 (100.0%) |
| `Style/DigChain` | [`crates/rustocop/src/cops/prism/dig_rules.rs`](../crates/rustocop/src/cops/prism/dig_rules.rs) | 2026-08-18 | 23 | 23/23 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/Dir` | [`crates/rustocop/src/cops/prism/dir_rules.rs`](../crates/rustocop/src/cops/prism/dir_rules.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 18 | 18/18 (100.0%) |
| `Style/DirEmpty` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 17 | 17/17 (100.0%) | 1 | 1/12 (8.3%) |
| `Style/DisableCopsWithinSourceCodeDirective` | [`crates/rustocop/src/cops/prism/lexical_completion.rs`](../crates/rustocop/src/cops/prism/lexical_completion.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 11435 | 11435/11439 (100.0%) |
| `Style/DocumentDynamicEvalDefinition` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 20 | 20/20 (100.0%) | 138 | 138/138 (100.0%) |
| `Style/Documentation` | [`crates/rustocop/src/cops/text/style_declarations.rs`](../crates/rustocop/src/cops/text/style_declarations.rs) | 2026-08-24 | 55 | 55/55 (100.0%) | 49775 | 48846/55320 (88.3%) |
| `Style/DocumentationMethod` | [`crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs`](../crates/rustocop/src/cops/prism/lint_signature_completion_batch.rs) | 2026-08-24 | 86 | 86/86 (100.0%) | 186456 ⚠ | 176574/194610 (90.7%) ⚠ stale |
| `Style/DoubleCopDisableDirective` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-24 | 3 | 3/3 (100.0%) | 0 | 0/5 (0.0%) |
| `Style/DoubleNegation` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-24 | 49 | 49/49 (100.0%) | 273 | 273/273 (100.0%) |
| `Style/EachForSimpleLoop` | [`crates/rustocop/src/cops/prism/control_flow_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_flow_completion_batch.rs) | 2026-08-24 | 21 | 21/21 (100.0%) | 0 | 0/13 (0.0%) |
| `Style/EachWithObject` | [`crates/rustocop/src/cops/prism/collection_completion_rules.rs`](../crates/rustocop/src/cops/prism/collection_completion_rules.rs) | 2026-08-22 | 17 | 17/17 (100.0%) | 109 | 109/109 (100.0%) |
| `Style/EmptyBlockParameter` | [`crates/rustocop/src/cops/prism/additional_rules_more.rs`](../crates/rustocop/src/cops/prism/additional_rules_more.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 4 | 3/16 (18.8%) |
| `Style/EmptyCaseCondition` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-24 | 30 | 30/30 (100.0%) | 63 | 63/63 (100.0%) |
| `Style/EmptyClassDefinition` | [`crates/rustocop/src/cops/prism/class_definition_rules.rs`](../crates/rustocop/src/cops/prism/class_definition_rules.rs) | 2026-08-18 | 52 | 52/52 (100.0%) | 990 | 990/990 (100.0%) |
| `Style/EmptyElse` | [`crates/rustocop/src/cops/prism/empty_else_rules.rs`](../crates/rustocop/src/cops/prism/empty_else_rules.rs) | 2026-08-23 | 126 | 126/126 (100.0%) | 244 | 244/244 (100.0%) |
| `Style/EmptyHeredoc` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 18 | 6/33 (18.2%) |
| `Style/EmptyLambdaParameter` | [`crates/rustocop/src/cops/prism/empty_lambda_parameter_rules.rs`](../crates/rustocop/src/cops/prism/empty_lambda_parameter_rules.rs) | 2026-08-21 | 4 | 4/4 (100.0%) | 49 | 49/49 (100.0%) |
| `Style/EmptyLiteral` | [`crates/rustocop/src/cops/prism/literal_integrity_completion.rs`](../crates/rustocop/src/cops/prism/literal_integrity_completion.rs) | 2026-08-24 | 50 | 50/50 (100.0%) | 118 | 116/242 (47.9%) |
| `Style/EmptyMethod` | [`crates/rustocop/src/cops/prism/empty_method_rules.rs`](../crates/rustocop/src/cops/prism/empty_method_rules.rs) | 2026-08-22 | 34 | 34/34 (100.0%) | 777 | 777/777 (100.0%) |
| `Style/EmptyStringInsideInterpolation` | [`crates/rustocop/src/cops/prism/interpolation_condition_rules.rs`](../crates/rustocop/src/cops/prism/interpolation_condition_rules.rs) | 2026-08-23 | 23 | 23/23 (100.0%) | 173 | 173/173 (100.0%) |
| `Style/Encoding` | [`crates/rustocop/src/cops/prism/lexical_completion.rs`](../crates/rustocop/src/cops/prism/lexical_completion.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 128 | 128/128 (100.0%) |
| `Style/EndBlock` | [`crates/rustocop/src/cops/text/lint.rs`](../crates/rustocop/src/cops/text/lint.rs) | 2026-08-24 | 3 | 3/3 (100.0%) | 0 | 0/1 (0.0%) |
| `Style/EndlessMethod` | [`crates/rustocop/src/cops/prism/endless_method_rules.rs`](../crates/rustocop/src/cops/prism/endless_method_rules.rs) | 2026-08-24 | 65 | 65/65 (100.0%) | 3 | 3/29 (10.3%) |
| `Style/EnvHome` | [`crates/rustocop/src/cops/prism/source_rules_misc.rs`](../crates/rustocop/src/cops/prism/source_rules_misc.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 33 | 12/37 (32.4%) |
| `Style/EvalWithLocation` | [`crates/rustocop/src/cops/prism/exception_location_completion.rs`](../crates/rustocop/src/cops/prism/exception_location_completion.rs) | 2026-08-24 | 37 | 37/37 (100.0%) | 504 | 483/550 (87.8%) |
| `Style/EvenOdd` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 18 | 18/18 (100.0%) | 8 | 8/8 (100.0%) |
| `Style/ExactRegexpMatch` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 0 | — (unexercised) |
| `Style/ExpandPathArguments` | [`crates/rustocop/src/cops/prism/path_and_literal_rules.rs`](../crates/rustocop/src/cops/prism/path_and_literal_rules.rs) | 2026-08-23 | 18 | 18/18 (100.0%) | 260 | 258/260 (99.2%) |
| `Style/ExplicitBlockArgument` | [`crates/rustocop/src/cops/prism/structural_forwarding_completion.rs`](../crates/rustocop/src/cops/prism/structural_forwarding_completion.rs) | 2026-08-24 | 30 | 30/30 (100.0%) | 357 | 248/391 (63.4%) |
| `Style/ExponentialNotation` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-24 | 27 | 27/27 (100.0%) | 12 | 12/12 (100.0%) |
| `Style/FetchEnvVar` | [`crates/rustocop/src/cops/prism/literal_string_completion_batch.rs`](../crates/rustocop/src/cops/prism/literal_string_completion_batch.rs) | 2026-08-24 | 49 | 49/49 (100.0%) | 1386 | 1340/1543 (86.8%) |
| `Style/FileEmpty` | [`crates/rustocop/src/cops/prism/file_predicate_rules.rs`](../crates/rustocop/src/cops/prism/file_predicate_rules.rs) | 2026-08-20 | 28 | 28/28 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/FileNull` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 71 | 71/71 (100.0%) |
| `Style/FileOpen` | [`crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs`](../crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs) | 2026-08-23 | 22 | 22/22 (100.0%) | 118 | 118/118 (100.0%) |
| `Style/FileRead` | [`crates/rustocop/src/cops/prism/compact_syntax_completion.rs`](../crates/rustocop/src/cops/prism/compact_syntax_completion.rs) | 2026-08-23 | 31 | 31/31 (100.0%) | 39 | 39/39 (100.0%) |
| `Style/FileTouch` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 0 | — (unexercised) |
| `Style/FileWrite` | [`crates/rustocop/src/cops/prism/compact_syntax_completion.rs`](../crates/rustocop/src/cops/prism/compact_syntax_completion.rs) | 2026-08-23 | 38 | 38/38 (100.0%) | — | — (rubocop_error) |
| `Style/FloatDivision` | [`crates/rustocop/src/cops/prism/numeric_operation_rules.rs`](../crates/rustocop/src/cops/prism/numeric_operation_rules.rs) | 2026-08-18 | 31 | 31/31 (100.0%) | 18 | 18/18 (100.0%) |
| `Style/For` | [`crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs`](../crates/rustocop/src/cops/prism/control_semantics_completion_batch.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/FormatString` | [`crates/rustocop/src/cops/prism/format_string_rules.rs`](../crates/rustocop/src/cops/prism/format_string_rules.rs) | 2026-08-20 | 46 | 46/46 (100.0%) | 2551 | 2551/2551 (100.0%) |
| `Style/FormatStringToken` | [`crates/rustocop/src/cops/prism/format_string_token_rules.rs`](../crates/rustocop/src/cops/prism/format_string_token_rules.rs) | 2026-08-22 | 267 | 267/267 (100.0%) | 7619 | 7618/7663 (99.4%) |
| `Style/FrozenStringLiteralComment` | [`crates/rustocop/src/cops/prism/frozen_string_literal_comment_rules.rs`](../crates/rustocop/src/cops/prism/frozen_string_literal_comment_rules.rs) | 2026-08-20 | 108 | 108/108 (100.0%) | 14117 | 14117/14123 (100.0%) |
| `Style/GlobalStdStream` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 6 | 6/6 (100.0%) | 417 | 417/417 (100.0%) |
| `Style/GlobalVars` | [`crates/rustocop/src/cops/prism/style_global_vars.rs`](../crates/rustocop/src/cops/prism/style_global_vars.rs) | 2026-08-21 | 74 | 74/74 (100.0%) | 1046 | 1046/1046 (100.0%) |
| `Style/GuardClause` | [`crates/rustocop/src/cops/prism/guard_clause_rules.rs`](../crates/rustocop/src/cops/prism/guard_clause_rules.rs) | 2026-08-23 | 93 | 93/93 (100.0%) | 4327 | 4324/4329 (99.9%) |
| `Style/HashAsLastArrayItem` | [`crates/rustocop/src/cops/prism/hash_array_rules.rs`](../crates/rustocop/src/cops/prism/hash_array_rules.rs) | 2026-08-18 | 19 | 19/19 (100.0%) | 1629 | 1629/1630 (99.9%) |
| `Style/HashConversion` | [`crates/rustocop/src/cops/prism/hash_conversion_rules.rs`](../crates/rustocop/src/cops/prism/hash_conversion_rules.rs) | 2026-08-20 | 24 | 24/24 (100.0%) | 156 | 156/157 (99.4%) |
| `Style/HashEachMethods` | [`crates/rustocop/src/cops/prism/hash_each_methods_rules.rs`](../crates/rustocop/src/cops/prism/hash_each_methods_rules.rs) | 2026-08-22 | 64 | 64/64 (100.0%) | 497 | 497/497 (100.0%) |
| `Style/HashExcept` | [`crates/rustocop/src/cops/prism/hash_subset_rules.rs`](../crates/rustocop/src/cops/prism/hash_subset_rules.rs) | 2026-08-20 | 114 | 114/114 (100.0%) | 32 ⚠ | 32/32 (100.0%) ⚠ stale |
| `Style/HashFetchChain` | [`crates/rustocop/src/cops/prism/hash_fetch_chain_rules.rs`](../crates/rustocop/src/cops/prism/hash_fetch_chain_rules.rs) | 2026-08-20 | 35 | 35/35 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/HashLikeCase` | [`crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs`](../crates/rustocop/src/cops/prism/literal_and_pattern_rules.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 42 | 41/42 (97.6%) |
| `Style/HashLookupMethod` | [`crates/rustocop/src/cops/prism/lookup_completion_rules.rs`](../crates/rustocop/src/cops/prism/lookup_completion_rules.rs) | 2026-08-21 | 19 | 19/19 (100.0%) | 4834 | 4834/4834 (100.0%) |
| `Style/HashSlice` | [`crates/rustocop/src/cops/prism/hash_subset_rules.rs`](../crates/rustocop/src/cops/prism/hash_subset_rules.rs) | 2026-08-20 | 116 | 116/116 (100.0%) | 43 ⚠ | 43/43 (100.0%) ⚠ stale |
| `Style/HashSyntax` | [`crates/rustocop/src/cops/prism/hash_syntax_rules.rs`](../crates/rustocop/src/cops/prism/hash_syntax_rules.rs) | 2026-08-21 | 190 | 190/190 (100.0%) | 19082 | 19081/19084 (100.0%) |
| `Style/HashTransformKeys` | [`crates/rustocop/src/cops/prism/hash_transform_rules.rs`](../crates/rustocop/src/cops/prism/hash_transform_rules.rs) | 2026-08-20 | 40 | 40/40 (100.0%) | 0 | — (unexercised) |
| `Style/HashTransformValues` | [`crates/rustocop/src/cops/prism/hash_transform_rules.rs`](../crates/rustocop/src/cops/prism/hash_transform_rules.rs) | 2026-08-20 | 40 | 40/40 (100.0%) | 4 | 4/4 (100.0%) |
| `Style/IdenticalConditionalBranches` | [`crates/rustocop/src/cops/prism/identical_conditional_branches_rules.rs`](../crates/rustocop/src/cops/prism/identical_conditional_branches_rules.rs) | 2026-08-23 | 49 | 49/49 (100.0%) | 172 | 159/172 (92.4%) |
| `Style/IfInsideElse` | [`crates/rustocop/src/cops/prism/structural_next_completion.rs`](../crates/rustocop/src/cops/prism/structural_next_completion.rs) | 2026-08-24 | 24 | 24/24 (100.0%) | 191 | 191/191 (100.0%) |
| `Style/IfUnlessModifier` | [`crates/rustocop/src/cops/prism/if_unless_modifier_rules.rs`](../crates/rustocop/src/cops/prism/if_unless_modifier_rules.rs) | 2026-08-23 | 148 | 148/148 (100.0%) | 11656 | 11179/11676 (95.7%) |
| `Style/IfUnlessModifierOfIfUnless` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 6 | 6/6 (100.0%) | 14 | 14/14 (100.0%) |
| `Style/IfWithBooleanLiteralBranches` | [`crates/rustocop/src/cops/prism/if_with_boolean_literal_branches_rules.rs`](../crates/rustocop/src/cops/prism/if_with_boolean_literal_branches_rules.rs) | 2026-08-21 | 89 | 89/89 (100.0%) | 21 | 21/23 (91.3%) |
| `Style/IfWithSemicolon` | [`crates/rustocop/src/cops/prism/if_with_semicolon_rules.rs`](../crates/rustocop/src/cops/prism/if_with_semicolon_rules.rs) | 2026-08-24 | 36 | 36/36 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/ImplicitRuntimeError` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 10 | 10/10 (100.0%) | 3265 | 3265/3265 (100.0%) |
| `Style/InPatternThen` | [`crates/rustocop/src/cops/prism/additional_rules.rs`](../crates/rustocop/src/cops/prism/additional_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 0 | 0/5 (0.0%) |
| `Style/InfiniteLoop` | [`crates/rustocop/src/cops/prism/infinite_loop_rules.rs`](../crates/rustocop/src/cops/prism/infinite_loop_rules.rs) | 2026-08-20 | 29 | 29/29 (100.0%) | 546 | 543/548 (99.1%) |
| `Style/InlineComment` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 14 | 14/14 (100.0%) | 18651 | 18651/229984 (8.1%) |
| `Style/InverseMethods` | [`crates/rustocop/src/cops/prism/inverse_methods_rules.rs`](../crates/rustocop/src/cops/prism/inverse_methods_rules.rs) | 2026-08-20 | 111 | 111/111 (100.0%) | 153 | 153/153 (100.0%) |
| `Style/InvertibleUnlessCondition` | [`crates/rustocop/src/cops/prism/invertible_unless_condition_rules.rs`](../crates/rustocop/src/cops/prism/invertible_unless_condition_rules.rs) | 2026-08-23 | 17 | 17/17 (100.0%) | 763 | 763/763 (100.0%) |
| `Style/IpAddresses` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 15 | 15/15 (100.0%) | 2914 | 2914/2914 (100.0%) |
| `Style/ItAssignment` | [`crates/rustocop/src/cops/prism/parameter_order_completion.rs`](../crates/rustocop/src/cops/prism/parameter_order_completion.rs) | 2026-08-24 | 25 | 25/25 (100.0%) | 20 | 20/20 (100.0%) |
| `Style/ItBlockParameter` | [`crates/rustocop/src/cops/prism/it_parameter_rules.rs`](../crates/rustocop/src/cops/prism/it_parameter_rules.rs) | 2026-08-18 | 34 | 34/34 (100.0%) | 240 | 208/241 (86.3%) |
| `Style/KeywordArgumentsMerging` | [`crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs`](../crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 196 | 196/196 (100.0%) |
| `Style/KeywordParametersOrder` | [`crates/rustocop/src/cops/prism/parameter_order_completion.rs`](../crates/rustocop/src/cops/prism/parameter_order_completion.rs) | 2026-08-24 | 11 | 11/11 (100.0%) | 306 | 306/306 (100.0%) |
| `Style/Lambda` | [`crates/rustocop/src/cops/prism/lambda_rules.rs`](../crates/rustocop/src/cops/prism/lambda_rules.rs) | 2026-08-20 | 38 | 38/38 (100.0%) | 3492 | 3492/3494 (99.9%) |
| `Style/LambdaCall` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 19 | 19/19 (100.0%) | 1645 | 1644/1646 (99.9%) |
| `Style/LineEndConcatenation` | [`crates/rustocop/src/cops/prism/line_concatenation_rules.rs`](../crates/rustocop/src/cops/prism/line_concatenation_rules.rs) | 2026-08-21 | 20 | 20/20 (100.0%) | 658 | 655/658 (99.5%) |
| `Style/MagicCommentFormat` | [`crates/rustocop/src/cops/prism/magic_comment_format_rules.rs`](../crates/rustocop/src/cops/prism/magic_comment_format_rules.rs) | 2026-08-20 | 25 | 25/25 (100.0%) | 6 | 6/8 (75.0%) |
| `Style/MapCompactWithConditionalBlock` | [`crates/rustocop/src/cops/prism/map_compact_conditional_rules.rs`](../crates/rustocop/src/cops/prism/map_compact_conditional_rules.rs) | 2026-08-21 | 34 | 34/34 (100.0%) | 13 | 13/13 (100.0%) |
| `Style/MapIntoArray` | [`crates/rustocop/src/cops/prism/map_into_array_rules.rs`](../crates/rustocop/src/cops/prism/map_into_array_rules.rs) | 2026-08-23 | 64 | 64/64 (100.0%) | 129 | 117/131 (89.3%) |
| `Style/MapJoin` | [`crates/rustocop/src/cops/prism/map_join_rules.rs`](../crates/rustocop/src/cops/prism/map_join_rules.rs) | 2026-08-18 | 24 | 24/24 (100.0%) | 48 | 48/48 (100.0%) |
| `Style/MapToHash` | [`crates/rustocop/src/cops/prism/map_conversion_rules.rs`](../crates/rustocop/src/cops/prism/map_conversion_rules.rs) | 2026-08-20 | 39 | 39/39 (100.0%) | 97 | 97/97 (100.0%) |
| `Style/MapToSet` | [`crates/rustocop/src/cops/prism/map_conversion_rules.rs`](../crates/rustocop/src/cops/prism/map_conversion_rules.rs) | 2026-08-20 | 31 | 31/31 (100.0%) | 125 | 125/125 (100.0%) |
| `Style/MethodCallWithArgsParentheses` | [`crates/rustocop/src/cops/prism/method_call_parentheses_rules.rs`](../crates/rustocop/src/cops/prism/method_call_parentheses_rules.rs) | 2026-08-24 | 184 | 184/184 (100.0%) | 838160 ⚠ | 837975/845079 (99.2%) ⚠ stale |
| `Style/MethodCallWithoutArgsParentheses` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-24 | 34 | 34/34 (100.0%) | 1262 | 1262/1262 (100.0%) |
| `Style/MethodCalledOnDoEndBlock` | [`crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs`](../crates/rustocop/src/cops/prism/resource_and_precedence_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 10426 | 10426/10426 (100.0%) |
| `Style/MethodDefParentheses` | [`crates/rustocop/src/cops/prism/method_def_parentheses_rules.rs`](../crates/rustocop/src/cops/prism/method_def_parentheses_rules.rs) | 2026-08-20 | 50 | 50/50 (100.0%) | 926 | 926/926 (100.0%) |
| `Style/MinMax` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 12 | 12/12 (100.0%) | 0 | — (unexercised) |
| `Style/MinMaxComparison` | [`crates/rustocop/src/cops/prism/predicate_conversion_rules.rs`](../crates/rustocop/src/cops/prism/predicate_conversion_rules.rs) | 2026-08-21 | 17 | 17/17 (100.0%) | 66 | 66/66 (100.0%) |
| `Style/MissingElse` | [`crates/rustocop/src/cops/prism/missing_else_rules.rs`](../crates/rustocop/src/cops/prism/missing_else_rules.rs) | 2026-08-24 | 91 | 91/91 (100.0%) | 40883 ⚠ | 32220/40923 (78.7%) ⚠ stale |
| `Style/MissingRespondToMissing` | [`crates/rustocop/src/cops/prism/declaration_semantics.rs`](../crates/rustocop/src/cops/prism/declaration_semantics.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 93 | 93/99 (93.9%) |
| `Style/MixinGrouping` | [`crates/rustocop/src/cops/prism/mixin_grouping_rules.rs`](../crates/rustocop/src/cops/prism/mixin_grouping_rules.rs) | 2026-08-18 | 18 | 18/18 (100.0%) | 30 | 30/30 (100.0%) |
| `Style/MixinUsage` | [`crates/rustocop/src/cops/prism/mixin_rules.rs`](../crates/rustocop/src/cops/prism/mixin_rules.rs) | 2026-08-18 | 18 | 18/18 (100.0%) | 47 | 47/48 (97.9%) |
| `Style/ModuleFunction` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 134 | 134/134 (100.0%) |
| `Style/ModuleMemberExistenceCheck` | [`crates/rustocop/src/cops/prism/module_member_existence_rules.rs`](../crates/rustocop/src/cops/prism/module_member_existence_rules.rs) | 2026-08-20 | 69 | 69/69 (100.0%) | 31 | 31/31 (100.0%) |
| `Style/MultilineBlockChain` | [`crates/rustocop/src/cops/prism/block_chain_rules.rs`](../crates/rustocop/src/cops/prism/block_chain_rules.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 275 | 275/296 (92.9%) |
| `Style/MultilineIfModifier` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 406 | 406/419 (96.9%) |
| `Style/MultilineIfThen` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 404 | 404/404 (100.0%) |
| `Style/MultilineInPatternThen` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 0 | — (unexercised) |
| `Style/MultilineMemoization` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 19 | 19/19 (100.0%) | 9 | 9/9 (100.0%) |
| `Style/MultilineMethodSignature` | [`crates/rustocop/src/cops/prism/method_signature_rules.rs`](../crates/rustocop/src/cops/prism/method_signature_rules.rs) | 2026-08-22 | 20 | 20/20 (100.0%) | 20 | 20/20 (100.0%) |
| `Style/MultilineTernaryOperator` | [`crates/rustocop/src/cops/prism/structural_next_completion.rs`](../crates/rustocop/src/cops/prism/structural_next_completion.rs) | 2026-08-24 | 19 | 19/19 (100.0%) | 148 | 148/148 (100.0%) |
| `Style/MultilineWhenThen` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 13 | 13/13 (100.0%) | 203 | 203/204 (99.5%) |
| `Style/MultipleComparison` | [`crates/rustocop/src/cops/prism/structural_forwarding_completion.rs`](../crates/rustocop/src/cops/prism/structural_forwarding_completion.rs) | 2026-08-24 | 36 | 36/36 (100.0%) | 391 | 389/392 (99.2%) |
| `Style/MutableConstant` | [`crates/rustocop/src/cops/prism/mutable_constant_rules.rs`](../crates/rustocop/src/cops/prism/mutable_constant_rules.rs) | 2026-08-20 | 356 | 356/356 (100.0%) | 2202 | 2202/2214 (99.5%) |
| `Style/NegatedIf` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 16 | 16/16 (100.0%) | 2352 | 2352/2352 (100.0%) |
| `Style/NegatedIfElseCondition` | [`crates/rustocop/src/cops/prism/negated_if_else_rules.rs`](../crates/rustocop/src/cops/prism/negated_if_else_rules.rs) | 2026-08-20 | 32 | 32/32 (100.0%) | 279 | 279/280 (99.6%) |
| `Style/NegatedUnless` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 16 | 16/16 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/NegatedWhile` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 11 | 11/11 (100.0%) | 43 | 43/43 (100.0%) |
| `Style/NegativeArrayIndex` | [`crates/rustocop/src/cops/prism/negative_array_index_rules.rs`](../crates/rustocop/src/cops/prism/negative_array_index_rules.rs) | 2026-08-20 | 411 | 411/411 (100.0%) | 8 | 8/8 (100.0%) |
| `Style/NestedFileDirname` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/NestedModifier` | [`crates/rustocop/src/cops/prism/nested_modifier_rules.rs`](../crates/rustocop/src/cops/prism/nested_modifier_rules.rs) | 2026-08-18 | 13 | 13/13 (100.0%) | 12 | 12/12 (100.0%) |
| `Style/NestedParenthesizedCalls` | [`crates/rustocop/src/cops/prism/nested_call_rules.rs`](../crates/rustocop/src/cops/prism/nested_call_rules.rs) | 2026-08-21 | 13 | 13/13 (100.0%) | 153 | 153/154 (99.4%) |
| `Style/NestedTernaryOperator` | [`crates/rustocop/src/cops/prism/ternary_rules.rs`](../crates/rustocop/src/cops/prism/ternary_rules.rs) | 2026-08-24 | 8 | 8/8 (100.0%) | 374 | 374/374 (100.0%) |
| `Style/Next` | [`crates/rustocop/src/cops/prism/next_rules.rs`](../crates/rustocop/src/cops/prism/next_rules.rs) | 2026-08-21 | 70 | 70/70 (100.0%) | 319 ⚠ | 319/319 (100.0%) ⚠ stale |
| `Style/NilComparison` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-24 | 8 | 8/8 (100.0%) | 80 | 80/80 (100.0%) |
| `Style/NilLambda` | [`crates/rustocop/src/cops/prism/nil_callable_rules.rs`](../crates/rustocop/src/cops/prism/nil_callable_rules.rs) | 2026-08-18 | 31 | 31/31 (100.0%) | 43 | 43/43 (100.0%) |
| `Style/NonNilCheck` | [`crates/rustocop/src/cops/prism/conditional_semantics_rules.rs`](../crates/rustocop/src/cops/prism/conditional_semantics_rules.rs) | 2026-08-19 | 21 | 21/21 (100.0%) | 23 | 23/23 (100.0%) |
| `Style/Not` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 109 | 109/109 (100.0%) |
| `Style/NumberedParameters` | [`crates/rustocop/src/cops/prism/block_parameter_rules.rs`](../crates/rustocop/src/cops/prism/block_parameter_rules.rs) | 2026-08-18 | 5 | 5/5 (100.0%) | 8 | 5/9 (55.6%) |
| `Style/NumberedParametersLimit` | [`crates/rustocop/src/cops/prism/block_parameter_rules.rs`](../crates/rustocop/src/cops/prism/block_parameter_rules.rs) | 2026-08-18 | 11 | 11/11 (100.0%) | 2 | 2/2 (100.0%) |
| `Style/NumericLiteralPrefix` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 1136 | 1134/1136 (99.8%) |
| `Style/NumericLiterals` | [`crates/rustocop/src/cops/prism/style_metadata_completion.rs`](../crates/rustocop/src/cops/prism/style_metadata_completion.rs) | 2026-08-24 | 30 | 30/30 (100.0%) | 3046 ⚠ | 3046/3046 (100.0%) ⚠ stale |
| `Style/NumericPredicate` | [`crates/rustocop/src/cops/prism/numeric_predicate_rules.rs`](../crates/rustocop/src/cops/prism/numeric_predicate_rules.rs) | 2026-08-21 | 43 | 43/43 (100.0%) | 4018 ⚠ | 4018/4018 (100.0%) ⚠ stale |
| `Style/ObjectThen` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 23 | 23/23 (100.0%) | 5 | 5/5 (100.0%) |
| `Style/OneClassPerFile` | [`crates/rustocop/src/cops/prism/file_structure_rules.rs`](../crates/rustocop/src/cops/prism/file_structure_rules.rs) | 2026-08-21 | 22 | 22/22 (100.0%) | 2061 | 2061/2061 (100.0%) |
| `Style/OneLineConditional` | [`crates/rustocop/src/cops/prism/one_line_conditional_rules.rs`](../crates/rustocop/src/cops/prism/one_line_conditional_rules.rs) | 2026-08-24 | 108 | 108/108 (100.0%) | 7 | 7/7 (100.0%) |
| `Style/OpenStructUse` | [`crates/rustocop/src/cops/prism/additional_rules_literals.rs`](../crates/rustocop/src/cops/prism/additional_rules_literals.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 278 | 278/278 (100.0%) |
| `Style/OperatorMethodCall` | [`crates/rustocop/src/cops/prism/operator_method_call_rules.rs`](../crates/rustocop/src/cops/prism/operator_method_call_rules.rs) | 2026-08-21 | 203 | 203/203 (100.0%) | 6 | 3/6 (50.0%) |
| `Style/OptionHash` | [`crates/rustocop/src/cops/prism/argument_default_rules.rs`](../crates/rustocop/src/cops/prism/argument_default_rules.rs) | 2026-08-24 | 10 | 10/10 (100.0%) | 2331 | 2331/2331 (100.0%) |
| `Style/OptionalArguments` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 16 ⚠ | 16/23 (69.6%) ⚠ stale |
| `Style/OptionalBooleanParameter` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 9 | 9/9 (100.0%) | 837 ⚠ | 837/837 (100.0%) ⚠ stale |
| `Style/OrAssignment` | [`crates/rustocop/src/cops/prism/assignment_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/assignment_rewrite_rules.rs) | 2026-08-24 | 25 | 25/25 (100.0%) | 56 | 56/56 (100.0%) |
| `Style/ParallelAssignment` | [`crates/rustocop/src/cops/prism/assignment_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/assignment_rewrite_rules.rs) | 2026-08-24 | 85 | 85/85 (100.0%) | 721 | 721/722 (99.9%) |
| `Style/ParenthesesAroundCondition` | [`crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs) | 2026-08-24 | 31 | 31/31 (100.0%) | 77 | 77/77 (100.0%) |
| `Style/PartitionInsteadOfDoubleSelect` | [`crates/rustocop/src/cops/prism/collection_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/collection_rewrite_rules.rs) | 2026-08-24 | 38 | 38/38 (100.0%) | 11 | 11/12 (91.7%) |
| `Style/PercentLiteralDelimiters` | [`crates/rustocop/src/cops/prism/literal_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/literal_rewrite_rules.rs) | 2026-08-24 | 67 | 67/67 (100.0%) | 6352 | 6352/6352 (100.0%) |
| `Style/PercentQLiterals` | [`crates/rustocop/src/cops/prism/path_and_literal_rules.rs`](../crates/rustocop/src/cops/prism/path_and_literal_rules.rs) | 2026-08-23 | 18 | 18/18 (100.0%) | 82 | 74/82 (90.2%) |
| `Style/PerlBackrefs` | [`crates/rustocop/src/cops/prism/style_global_vars.rs`](../crates/rustocop/src/cops/prism/style_global_vars.rs) | 2026-08-21 | 15 | 15/15 (100.0%) | 1310 | 1310/1310 (100.0%) |
| `Style/PredicateWithKind` | [`crates/rustocop/src/cops/prism/collection_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/collection_rewrite_rules.rs) | 2026-08-24 | 64 | 64/64 (100.0%) | 53 | 53/53 (100.0%) |
| `Style/PreferredHashMethods` | [`crates/rustocop/src/cops/prism/preferred_hash_methods_rules.rs`](../crates/rustocop/src/cops/prism/preferred_hash_methods_rules.rs) | 2026-08-21 | 11 | 11/11 (100.0%) | 881 | 881/881 (100.0%) |
| `Style/Proc` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 7 | 7/7 (100.0%) | 561 | 561/561 (100.0%) |
| `Style/QuotedSymbols` | [`crates/rustocop/src/cops/prism/literal_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/literal_rewrite_rules.rs) | 2026-08-24 | 95 | 95/95 (100.0%) | 3776 | 3776/3777 (100.0%) |
| `Style/RaiseArgs` | [`crates/rustocop/src/cops/prism/exception_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/exception_rewrite_rules.rs) | 2026-08-19 | 35 | 35/35 (100.0%) | 1138 | 1138/1138 (100.0%) |
| `Style/RandomWithOffset` | [`crates/rustocop/src/cops/prism/random_rules.rs`](../crates/rustocop/src/cops/prism/random_rules.rs) | 2026-08-18 | 29 | 29/29 (100.0%) | 17 | 17/17 (100.0%) |
| `Style/ReduceToHash` | [`crates/rustocop/src/cops/prism/collection_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/collection_rewrite_rules.rs) | 2026-08-24 | 25 | 25/25 (100.0%) | 164 | 163/164 (99.4%) |
| `Style/RedundantArgument` | [`crates/rustocop/src/cops/prism/argument_default_rules.rs`](../crates/rustocop/src/cops/prism/argument_default_rules.rs) | 2026-08-24 | 15 | 15/15 (100.0%) | 303 | 303/303 (100.0%) |
| `Style/RedundantArrayConstructor` | [`crates/rustocop/src/cops/prism/style.rs`](../crates/rustocop/src/cops/prism/style.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 212 | 212/212 (100.0%) |
| `Style/RedundantArrayFlatten` | [`crates/rustocop/src/cops/prism/style_collections.rs`](../crates/rustocop/src/cops/prism/style_collections.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 9 | 9/9 (100.0%) |
| `Style/RedundantAssignment` | [`crates/rustocop/src/cops/prism/semantic_gap_completion.rs`](../crates/rustocop/src/cops/prism/semantic_gap_completion.rs) | 2026-08-24 | 21 | 21/21 (100.0%) | 201 ⚠ | 176/274 (64.2%) ⚠ stale |
| `Style/RedundantBegin` | [`crates/rustocop/src/cops/prism/begin_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/begin_rewrite_rules.rs) | 2026-08-21 | 64 | 64/64 (100.0%) | 458 | 457/460 (99.3%) |
| `Style/RedundantCapitalW` | [`crates/rustocop/src/cops/prism/source_rules.rs`](../crates/rustocop/src/cops/prism/source_rules.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 35 | 34/56 (60.7%) |
| `Style/RedundantCondition` | [`crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs) | 2026-08-24 | 102 | 102/102 (100.0%) | 97 | 97/98 (99.0%) |
| `Style/RedundantConditional` | [`crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs`](../crates/rustocop/src/cops/prism/conditional_rewrite_rules.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 4 | 4/4 (100.0%) |
| `Style/RedundantConstantBase` | [`crates/rustocop/src/cops/prism/restored_structural_cops.rs`](../crates/rustocop/src/cops/prism/restored_structural_cops.rs) | 2026-08-24 | 11 | 11/11 (100.0%) | 7922 | 7922/7922 (100.0%) |
| `Style/RedundantCurrentDirectoryInPath` | [`crates/rustocop/src/cops/prism/argument_default_rules.rs`](../crates/rustocop/src/cops/prism/argument_default_rules.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 93 | 93/93 (100.0%) |
| `Style/RedundantDoubleSplatHashBraces` | [`crates/rustocop/src/cops/prism/double_splat_rules.rs`](../crates/rustocop/src/cops/prism/double_splat_rules.rs) | 2026-08-18 | 29 | 29/29 (100.0%) | 37 | 37/37 (100.0%) |
| `Style/RedundantEach` | [`crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs`](../crates/rustocop/src/cops/prism/iteration_redundancy_rules.rs) | 2026-08-24 | 36 | 36/36 (100.0%) | 8 | 8/8 (100.0%) |
| `Style/RedundantException` | [`crates/rustocop/src/cops/prism/exception_argument_rules.rs`](../crates/rustocop/src/cops/prism/exception_argument_rules.rs) | 2026-08-18 | 28 | 28/28 (100.0%) | 51 | 51/51 (100.0%) |
| `Style/RedundantFetchBlock` | [`crates/rustocop/src/cops/prism/fetch_completion_rules.rs`](../crates/rustocop/src/cops/prism/fetch_completion_rules.rs) | 2026-08-22 | 17 | 17/17 (100.0%) | 109 | 109/109 (100.0%) |
| `Style/RedundantFileExtensionInRequire` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 64 | 64/64 (100.0%) |
| `Style/RedundantFilterChain` | [`crates/rustocop/src/cops/prism/redundant_filter_chain_rules.rs`](../crates/rustocop/src/cops/prism/redundant_filter_chain_rules.rs) | 2026-08-19 | 39 | 39/39 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/RedundantFormat` | [`crates/rustocop/src/cops/prism/redundant_format_rules.rs`](../crates/rustocop/src/cops/prism/redundant_format_rules.rs) | 2026-08-19 | 280 | 280/280 (100.0%) | 11 | 9/11 (81.8%) |
| `Style/RedundantFreeze` | [`crates/rustocop/src/cops/prism/redundant_freeze_completion.rs`](../crates/rustocop/src/cops/prism/redundant_freeze_completion.rs) | 2026-08-19 | 62 | 62/62 (100.0%) | 187 | 159/187 (85.0%) |
| `Style/RedundantHeredocDelimiterQuotes` | [`crates/rustocop/src/cops/prism/lexical_completion.rs`](../crates/rustocop/src/cops/prism/lexical_completion.rs) | 2026-08-23 | 18 | 18/18 (100.0%) | 470 | 470/470 (100.0%) |
| `Style/RedundantInitialize` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-24 | 24 | 24/24 (100.0%) | 17 | 15/18 (83.3%) |
| `Style/RedundantInterpolation` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-24 | 30 | 30/30 (100.0%) | 704 | 704/704 (100.0%) |
| `Style/RedundantInterpolationUnfreeze` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-24 | 18 | 18/18 (100.0%) | 166 | 166/166 (100.0%) |
| `Style/RedundantLineContinuation` | [`crates/rustocop/src/cops/prism/redundant_line_continuation_rules.rs`](../crates/rustocop/src/cops/prism/redundant_line_continuation_rules.rs) | 2026-08-24 | 169 | 169/169 (100.0%) | 88 | 83/89 (93.3%) |
| `Style/RedundantMinMaxBy` | [`crates/rustocop/src/cops/prism/redundant_min_max_by_rules.rs`](../crates/rustocop/src/cops/prism/redundant_min_max_by_rules.rs) | 2026-08-19 | 33 | 33/33 (100.0%) | 0 | — (unexercised) |
| `Style/RedundantParentheses` | [`crates/rustocop/src/cops/prism/redundant_parentheses_rules.rs`](../crates/rustocop/src/cops/prism/redundant_parentheses_rules.rs) | 2026-08-24 | 358 | 358/358 (100.0%) | 522 | 480/754 (63.7%) |
| `Style/RedundantPercentQ` | [`crates/rustocop/src/cops/prism/percent_string_rules.rs`](../crates/rustocop/src/cops/prism/percent_string_rules.rs) | 2026-08-23 | 28 | 28/28 (100.0%) | 687 | 687/687 (100.0%) |
| `Style/RedundantRegexpArgument` | [`crates/rustocop/src/cops/prism/redundant_regexp_rules.rs`](../crates/rustocop/src/cops/prism/redundant_regexp_rules.rs) | 2026-08-24 | 52 | 52/52 (100.0%) | 308 | 308/308 (100.0%) |
| `Style/RedundantRegexpCharacterClass` | [`crates/rustocop/src/cops/prism/redundant_regexp_rules.rs`](../crates/rustocop/src/cops/prism/redundant_regexp_rules.rs) | 2026-08-24 | 48 | 48/48 (100.0%) | 367 | 367/367 (100.0%) |
| `Style/RedundantRegexpConstructor` | [`crates/rustocop/src/cops/prism/redundant_regexp_rules.rs`](../crates/rustocop/src/cops/prism/redundant_regexp_rules.rs) | 2026-08-24 | 10 | 10/10 (100.0%) | 10 | 10/10 (100.0%) |
| `Style/RedundantRegexpEscape` | [`crates/rustocop/src/cops/prism/redundant_regexp_rules.rs`](../crates/rustocop/src/cops/prism/redundant_regexp_rules.rs) | 2026-08-24 | 215 | 215/215 (100.0%) | 1133 | 1132/1133 (99.9%) |
| `Style/RedundantReturn` | [`crates/rustocop/src/cops/prism/redundant_return_rules.rs`](../crates/rustocop/src/cops/prism/redundant_return_rules.rs) | 2026-08-19 | 40 | 40/40 (100.0%) | 1823 | 1823/1823 (100.0%) |
| `Style/RedundantSelf` | [`crates/rustocop/src/cops/prism/self_rules.rs`](../crates/rustocop/src/cops/prism/self_rules.rs) | 2026-08-24 | 64 | 64/64 (100.0%) | 3583 | 3581/3583 (99.9%) |
| `Style/RedundantSelfAssignment` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-24 | 16 | 16/16 (100.0%) | 29 | 29/29 (100.0%) |
| `Style/RedundantSelfAssignmentBranch` | [`crates/rustocop/src/cops/prism/redundant_self_assignment_branch_rules.rs`](../crates/rustocop/src/cops/prism/redundant_self_assignment_branch_rules.rs) | 2026-08-20 | 23 | 23/23 (100.0%) | 84 | 84/84 (100.0%) |
| `Style/RedundantSort` | [`crates/rustocop/src/cops/prism/redundant_sort_rules.rs`](../crates/rustocop/src/cops/prism/redundant_sort_rules.rs) | 2026-08-19 | 50 | 50/50 (100.0%) | 33 | 33/33 (100.0%) |
| `Style/RedundantSortBy` | [`crates/rustocop/src/cops/prism/style_collections.rs`](../crates/rustocop/src/cops/prism/style_collections.rs) | 2026-08-23 | 8 | 8/8 (100.0%) | 2 | 1/2 (50.0%) |
| `Style/RedundantStringEscape` | [`crates/rustocop/src/cops/prism/redundant_string_escape_rules.rs`](../crates/rustocop/src/cops/prism/redundant_string_escape_rules.rs) | 2026-08-19 | 324 | 324/324 (100.0%) | 1875 | 1864/1875 (99.4%) |
| `Style/RedundantStructKeywordInit` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-24 | 18 | 18/18 (100.0%) | 231 | 231/231 (100.0%) |
| `Style/RegexpLiteral` | [`crates/rustocop/src/cops/prism/regexp_literal_rules.rs`](../crates/rustocop/src/cops/prism/regexp_literal_rules.rs) | 2026-08-24 | 66 | 66/66 (100.0%) | 2613 | 2613/2613 (100.0%) |
| `Style/RequireOrder` | [`crates/rustocop/src/cops/prism/require_order_rules.rs`](../crates/rustocop/src/cops/prism/require_order_rules.rs) | 2026-08-22 | 25 | 25/25 (100.0%) | 8661 | 8661/8661 (100.0%) |
| `Style/RescueModifier` | [`crates/rustocop/src/cops/prism/rescue_modifier_rules.rs`](../crates/rustocop/src/cops/prism/rescue_modifier_rules.rs) | 2026-08-19 | 21 | 21/21 (100.0%) | 427 | 427/427 (100.0%) |
| `Style/RescueStandardError` | [`crates/rustocop/src/cops/prism/rescue_standard_error_rules.rs`](../crates/rustocop/src/cops/prism/rescue_standard_error_rules.rs) | 2026-08-19 | 37 | 37/37 (100.0%) | 1378 | 1378/1378 (100.0%) |
| `Style/ReturnNil` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 6 | 6/6 (100.0%) | 1971 | 1971/1976 (99.7%) |
| `Style/ReturnNilInPredicateMethodDefinition` | [`crates/rustocop/src/cops/prism/return_nil_predicate_rules.rs`](../crates/rustocop/src/cops/prism/return_nil_predicate_rules.rs) | 2026-08-19 | 39 | 39/39 (100.0%) | 164 | 164/164 (100.0%) |
| `Style/ReverseFind` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 14 | 14/14 (100.0%) | 0 | — (unexercised) |
| `Style/SafeNavigation` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 784 | 784/784 (100.0%) | 1167 | 1167/1176 (99.2%) |
| `Style/SafeNavigationChainLength` | [`crates/rustocop/src/cops/prism/nested_call_rules.rs`](../crates/rustocop/src/cops/prism/nested_call_rules.rs) | 2026-08-21 | 9 | 9/9 (100.0%) | 140 | 140/140 (100.0%) |
| `Style/Sample` | [`crates/rustocop/src/cops/prism/collection_transform_batch.rs`](../crates/rustocop/src/cops/prism/collection_transform_batch.rs) | 2026-08-21 | 82 | 82/82 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/SelectByKind` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 144 | 144/144 (100.0%) | 75 | 75/75 (100.0%) |
| `Style/SelectByRange` | [`crates/rustocop/src/cops/prism/final_ast_structural_batch.rs`](../crates/rustocop/src/cops/prism/final_ast_structural_batch.rs) | 2026-08-24 | 120 | 120/120 (100.0%) | 0 | — (unexercised) |
| `Style/SelectByRegexp` | [`crates/rustocop/src/cops/prism/final_regexp_batch.rs`](../crates/rustocop/src/cops/prism/final_regexp_batch.rs) | 2026-08-24 | 320 | 320/320 (100.0%) | 107 | 107/107 (100.0%) |
| `Style/SelfAssignment` | [`crates/rustocop/src/cops/prism/final_scope_batch_b.rs`](../crates/rustocop/src/cops/prism/final_scope_batch_b.rs) | 2026-08-24 | 105 | 105/105 (100.0%) | 146 | 146/146 (100.0%) |
| `Style/Semicolon` | [`crates/rustocop/src/cops/prism/style_source.rs`](../crates/rustocop/src/cops/prism/style_source.rs) | 2026-08-24 | 37 | 37/37 (100.0%) | 2692 | 2692/2726 (98.8%) |
| `Style/Send` | [`crates/rustocop/src/cops/prism/source_semantics.rs`](../crates/rustocop/src/cops/prism/source_semantics.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 9491 ⚠ | 9491/9491 (100.0%) ⚠ stale |
| `Style/SendWithLiteralMethodName` | [`crates/rustocop/src/cops/prism/send_literal_rules.rs`](../crates/rustocop/src/cops/prism/send_literal_rules.rs) | 2026-08-19 | 115 | 115/115 (100.0%) | 28 | 28/28 (100.0%) |
| `Style/SignalException` | [`crates/rustocop/src/cops/prism/signal_exception_rules.rs`](../crates/rustocop/src/cops/prism/signal_exception_rules.rs) | 2026-08-19 | 27 | 27/27 (100.0%) | 494 | 494/494 (100.0%) |
| `Style/SingleArgumentDig` | [`crates/rustocop/src/cops/prism/style_call_simplifications.rs`](../crates/rustocop/src/cops/prism/style_call_simplifications.rs) | 2026-08-19 | 15 | 15/15 (100.0%) | 75 | 75/75 (100.0%) |
| `Style/SingleLineBlockParams` | [`crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs`](../crates/rustocop/src/cops/prism/compatibility_lexical_rules.rs) | 2026-08-23 | 12 | 12/12 (100.0%) | 156 | 156/156 (100.0%) |
| `Style/SingleLineDoEndBlock` | [`crates/rustocop/src/cops/prism/single_line_block_rules.rs`](../crates/rustocop/src/cops/prism/single_line_block_rules.rs) | 2026-08-19 | 15 | 15/15 (100.0%) | 832 | 832/832 (100.0%) |
| `Style/SingleLineMethods` | [`crates/rustocop/src/cops/prism/method_layout_rules.rs`](../crates/rustocop/src/cops/prism/method_layout_rules.rs) | 2026-08-20 | 148 | 148/148 (100.0%) | 1278 | 1278/1278 (100.0%) |
| `Style/SlicingWithRange` | [`crates/rustocop/src/cops/prism/path_and_literal_rules.rs`](../crates/rustocop/src/cops/prism/path_and_literal_rules.rs) | 2026-08-23 | 28 | 28/28 (100.0%) | 309 | 304/314 (96.8%) |
| `Style/SoleNestedConditional` | [`crates/rustocop/src/cops/prism/sole_nested_conditional_rules.rs`](../crates/rustocop/src/cops/prism/sole_nested_conditional_rules.rs) | 2026-08-19 | 74 | 74/74 (100.0%) | 573 | 573/574 (99.8%) |
| `Style/SpecialGlobalVars` | [`crates/rustocop/src/cops/prism/special_global_vars_rules.rs`](../crates/rustocop/src/cops/prism/special_global_vars_rules.rs) | 2026-08-22 | 31 | 31/31 (100.0%) | 519 | 519/539 (96.3%) |
| `Style/StabbyLambdaParentheses` | [`crates/rustocop/src/cops/prism/stabby_lambda_parentheses_rules.rs`](../crates/rustocop/src/cops/prism/stabby_lambda_parentheses_rules.rs) | 2026-08-19 | 9 | 9/9 (100.0%) | 90 | 90/90 (100.0%) |
| `Style/StaticClass` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 777 | 777/777 (100.0%) |
| `Style/StderrPuts` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 5 | 5/5 (100.0%) | 388 | 388/388 (100.0%) |
| `Style/StringChars` | [`crates/rustocop/src/cops/prism/redundant_freeze_completion.rs`](../crates/rustocop/src/cops/prism/redundant_freeze_completion.rs) | 2026-08-19 | 8 | 8/8 (100.0%) | 32 | 32/32 (100.0%) |
| `Style/StringConcatenation` | [`crates/rustocop/src/cops/prism/literal_string_completion_batch.rs`](../crates/rustocop/src/cops/prism/literal_string_completion_batch.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 3080 | 3080/3175 (97.0%) |
| `Style/StringHashKeys` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-24 | 11 | 11/11 (100.0%) | 131591 | 131587/131619 (100.0%) |
| `Style/StringLiterals` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-24 | 59 | 59/59 (100.0%) | 928483 | 928483/928483 (100.0%) |
| `Style/StringLiteralsInInterpolation` | [`crates/rustocop/src/cops/prism/string_conversion_rules.rs`](../crates/rustocop/src/cops/prism/string_conversion_rules.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 4012 | 4012/4012 (100.0%) |
| `Style/StringMethods` | [`crates/rustocop/src/cops/prism/style/misc.rs`](../crates/rustocop/src/cops/prism/style/misc.rs) | 2026-08-19 | 2 | 2/2 (100.0%) | 195 | 195/195 (100.0%) |
| `Style/Strip` | [`crates/rustocop/src/cops/prism/style_rewrites.rs`](../crates/rustocop/src/cops/prism/style_rewrites.rs) | 2026-08-21 | 6 | 6/6 (100.0%) | 0 | — (unexercised) |
| `Style/StructInheritance` | [`crates/rustocop/src/cops/prism/declaration_completion_rules.rs`](../crates/rustocop/src/cops/prism/declaration_completion_rules.rs) | 2026-08-24 | 13 | 13/13 (100.0%) | 35 | 35/36 (97.2%) |
| `Style/SuperArguments` | [`crates/rustocop/src/cops/prism/super_arguments_rules.rs`](../crates/rustocop/src/cops/prism/super_arguments_rules.rs) | 2026-08-19 | 92 | 92/92 (100.0%) | 443 | 443/447 (99.1%) |
| `Style/SuperWithArgsParentheses` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 4 | 4/4 (100.0%) | 175 | 175/175 (100.0%) |
| `Style/SwapValues` | [`crates/rustocop/src/cops/prism/assignment_completion_rules.rs`](../crates/rustocop/src/cops/prism/assignment_completion_rules.rs) | 2026-08-19 | 11 | 11/11 (100.0%) | 2 | 2/2 (100.0%) |
| `Style/SymbolArray` | [`crates/rustocop/src/cops/prism/literal_string_completion_batch.rs`](../crates/rustocop/src/cops/prism/literal_string_completion_batch.rs) | 2026-08-24 | 34 | 34/34 (100.0%) | 7945 | 7945/7945 (100.0%) |
| `Style/SymbolLiteral` | [`crates/rustocop/src/cops/prism/symbol_literal_rules.rs`](../crates/rustocop/src/cops/prism/symbol_literal_rules.rs) | 2026-08-19 | 5 | 5/5 (100.0%) | 588 | 588/588 (100.0%) |
| `Style/SymbolProc` | [`crates/rustocop/src/cops/prism/symbol_proc_rules.rs`](../crates/rustocop/src/cops/prism/symbol_proc_rules.rs) | 2026-08-21 | 82 | 82/82 (100.0%) | 1413 | 1413/1420 (99.5%) |
| `Style/TallyMethod` | [`crates/rustocop/src/cops/prism/modern_collection_completion.rs`](../crates/rustocop/src/cops/prism/modern_collection_completion.rs) | 2026-08-24 | 32 | 32/32 (100.0%) | 3 | 3/3 (100.0%) |
| `Style/TernaryParentheses` | [`crates/rustocop/src/cops/prism/ternary_parentheses_rules.rs`](../crates/rustocop/src/cops/prism/ternary_parentheses_rules.rs) | 2026-08-19 | 96 | 96/96 (100.0%) | 358 | 358/358 (100.0%) |
| `Style/TopLevelMethodDefinition` | [`crates/rustocop/src/cops/prism/project_scope_completion.rs`](../crates/rustocop/src/cops/prism/project_scope_completion.rs) | 2026-08-24 | 14 | 14/14 (100.0%) | 873 | 873/873 (100.0%) |
| `Style/TrailingBodyOnClass` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 0 | — (unexercised) |
| `Style/TrailingBodyOnMethodDefinition` | [`crates/rustocop/src/cops/prism/layout_finalization_completion.rs`](../crates/rustocop/src/cops/prism/layout_finalization_completion.rs) | 2026-08-24 | 12 | 12/12 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/TrailingBodyOnModule` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 7 | 7/7 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/TrailingCommaInArguments` | [`crates/rustocop/src/cops/prism/trailing_argument_comma_rules.rs`](../crates/rustocop/src/cops/prism/trailing_argument_comma_rules.rs) | 2026-08-24 | 169 | 169/169 (100.0%) | 36022 ⚠ | 36022/36022 (100.0%) ⚠ stale |
| `Style/TrailingCommaInArrayLiteral` | [`crates/rustocop/src/cops/prism/trailing_comma_completion.rs`](../crates/rustocop/src/cops/prism/trailing_comma_completion.rs) | 2026-08-24 | 51 | 51/51 (100.0%) | 4752 ⚠ | 4752/4752 (100.0%) ⚠ stale |
| `Style/TrailingCommaInBlockArgs` | [`crates/rustocop/src/cops/prism/style_compat.rs`](../crates/rustocop/src/cops/prism/style_compat.rs) | 2026-08-20 | 21 | 21/21 (100.0%) | 6 | 6/6 (100.0%) |
| `Style/TrailingCommaInHashLiteral` | [`crates/rustocop/src/cops/prism/trailing_comma_completion.rs`](../crates/rustocop/src/cops/prism/trailing_comma_completion.rs) | 2026-08-24 | 41 | 41/41 (100.0%) | 21428 ⚠ | 21428/21428 (100.0%) ⚠ stale |
| `Style/TrailingMethodEndStatement` | [`crates/rustocop/src/cops/prism/method_layout_rules.rs`](../crates/rustocop/src/cops/prism/method_layout_rules.rs) | 2026-08-20 | 12 | 12/12 (100.0%) | 1 | 1/1 (100.0%) |
| `Style/TrailingUnderscoreVariable` | [`crates/rustocop/src/cops/prism/trailing_underscore_rules.rs`](../crates/rustocop/src/cops/prism/trailing_underscore_rules.rs) | 2026-08-19 | 55 | 55/55 (100.0%) | 350 | 350/350 (100.0%) |
| `Style/TrivialAccessors` | [`crates/rustocop/src/cops/prism/trivial_accessor_rules.rs`](../crates/rustocop/src/cops/prism/trivial_accessor_rules.rs) | 2026-08-19 | 38 | 38/38 (100.0%) | 152 | 148/155 (95.5%) |
| `Style/UnlessElse` | [`crates/rustocop/src/cops/prism/style_source.rs`](../crates/rustocop/src/cops/prism/style_source.rs) | 2026-08-24 | 5 | 5/5 (100.0%) | 22 | 22/22 (100.0%) |
| `Style/UnlessLogicalOperators` | [`crates/rustocop/src/cops/prism/logical_condition_rules.rs`](../crates/rustocop/src/cops/prism/logical_condition_rules.rs) | 2026-08-20 | 30 | 30/30 (100.0%) | 94 | 93/94 (98.9%) |
| `Style/UnpackFirst` | [`crates/rustocop/src/cops/prism/call_conversion_rules.rs`](../crates/rustocop/src/cops/prism/call_conversion_rules.rs) | 2026-08-23 | 11 | 11/11 (100.0%) | 17 | 17/17 (100.0%) |
| `Style/VariableInterpolation` | [`crates/rustocop/src/cops/prism/lexical_rules.rs`](../crates/rustocop/src/cops/prism/lexical_rules.rs) | 2026-08-23 | 9 | 9/9 (100.0%) | 31 | 31/31 (100.0%) |
| `Style/WhenThen` | [`crates/rustocop/src/cops/prism/branch_layout_rules.rs`](../crates/rustocop/src/cops/prism/branch_layout_rules.rs) | 2026-08-23 | 5 | 5/5 (100.0%) | 166 | 166/166 (100.0%) |
| `Style/WhileUntilDo` | [`crates/rustocop/src/cops/prism/while_until_do_rules.rs`](../crates/rustocop/src/cops/prism/while_until_do_rules.rs) | 2026-08-19 | 6 | 6/6 (100.0%) | 29 | 29/29 (100.0%) |
| `Style/WhileUntilModifier` | [`crates/rustocop/src/cops/prism/compact_syntax_completion.rs`](../crates/rustocop/src/cops/prism/compact_syntax_completion.rs) | 2026-08-23 | 48 | 48/48 (100.0%) | 64 | 64/64 (100.0%) |
| `Style/WordArray` | [`crates/rustocop/src/cops/prism/literal_string_completion_batch.rs`](../crates/rustocop/src/cops/prism/literal_string_completion_batch.rs) | 2026-08-24 | 61 | 61/61 (100.0%) | 3513 | 3509/3514 (99.9%) |
| `Style/YAMLFileRead` | [`crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs`](../crates/rustocop/src/cops/prism/argument_and_inheritance_rules.rs) | 2026-08-21 | 11 | 11/11 (100.0%) | 63 | 63/63 (100.0%) |
| `Style/YodaCondition` | [`crates/rustocop/src/cops/prism/yoda_condition_rules.rs`](../crates/rustocop/src/cops/prism/yoda_condition_rules.rs) | 2026-08-19 | 76 | 76/76 (100.0%) | 171 | 171/171 (100.0%) |
| `Style/YodaExpression` | [`crates/rustocop/src/cops/prism/structural_completion_rules.rs`](../crates/rustocop/src/cops/prism/structural_completion_rules.rs) | 2026-08-23 | 10 | 10/10 (100.0%) | 1978 | 1978/1978 (100.0%) |
| `Style/ZeroLengthPredicate` | [`crates/rustocop/src/cops/prism/modern_collection_completion.rs`](../crates/rustocop/src/cops/prism/modern_collection_completion.rs) | 2026-08-24 | 68 | 68/68 (100.0%) | 791 | 791/791 (100.0%) |
