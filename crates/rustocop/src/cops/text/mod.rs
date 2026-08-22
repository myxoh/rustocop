mod extensions;
mod helpers;
mod layout;
mod lint;
mod lint_semantic;
mod style;
mod style_declarations;

use crate::config::InspectionConfig;
pub(crate) use crate::model::{push_offense, CorrectionStatus, Offense, SourceLine};

// Prism cops advertise themselves through their registry. These names are the
// shrinking compatibility surface that still requires the line-based runner.
pub(crate) const LEGACY_COP_NAMES: &[&str] = &[
    "Layout/IndentationConsistency",
    "Layout/IndentationWidth",
    "Layout/LineLength",
    "Layout/TrailingWhitespace",
    "Lint/BigDecimalNew",
    "Lint/EmptyEnsure",
    "Lint/TrailingCommaInAttributeDeclaration",
    "Lint/UnusedMethodArgument",
    "Lint/UselessElseWithoutRescue",
    "RSpec/EmptyExampleGroup",
    "RSpec/ExampleLength",
    "RSpec/Focus",
    "RSpec/MessageChain",
    "RSpec/MultipleExpectations",
    "RSpec/MultipleMemoizedHelpers",
    "RSpec/NestedGroups",
    "RSpec/PendingWithoutReason",
    "RSpec/ScatteredSetup",
    "RSpec/SpecFilePathFormat",
    "RSpec/SpecFilePathSuffix",
    "RSpec/VariableName",
    "Rails/ApplicationJob",
    "Rails/DefaultScope",
    "Rails/FilePath",
    "Rails/ReversibleMigration",
    "Style/ColonMethodDefinition",
    "Style/ConditionalAssignment",
    "Style/Documentation",
    "Style/DoubleCopDisableDirective",
    "Style/EmptyElse",
    "Style/EndBlock",
    "Style/EndlessMethod",
    "Style/FrozenStringLiteralComment",
    "Style/GuardClause",
    "Style/HashLikeCase",
    "Style/HashSyntax",
    "Style/IfUnlessModifier",
    "Style/InlineComment",
    "Style/NumberedParameters",
];

pub(crate) fn before_prism(
    lines: &mut Vec<SourceLine>,
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    layout::before_prism(lines, options, offenses);
    lint::before_prism(lines, options, offenses);
    style::before_prism(lines, options, offenses);
}

pub(crate) fn after_prism(
    path: &str,
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    layout::after_prism(lines, options, offenses);
    style::after_prism(lines, options, offenses);
    lint::after_prism(lines, options, offenses);
    extensions::after_prism(path, lines, options, offenses);
}
