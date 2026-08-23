pub(crate) mod prism;
pub(crate) mod text;

pub(crate) const INTENTIONALLY_PENDING_COP_NAMES: &[&str] = &[
    "Layout/ArgumentAlignment",
    "Layout/EmptyLineAfterGuardClause",
    "Layout/EmptyLines",
    "Layout/EndAlignment",
    "Layout/ExtraSpacing",
    "Layout/FirstArrayElementIndentation",
    "Layout/FirstArrayElementLineBreak",
    "Layout/FirstHashElementLineBreak",
    "Layout/FirstMethodArgumentLineBreak",
    "Layout/FirstMethodParameterLineBreak",
    "Layout/HeredocIndentation",
    "Layout/IndentationConsistency",
    "Layout/IndentationWidth",
    "Layout/LineContinuationLeadingSpace",
    "Layout/LineEndStringConcatenationIndentation",
    "Layout/LineLength",
    "Layout/MultilineArrayBraceLayout",
    "Layout/MultilineArrayLineBreaks",
    "Layout/MultilineAssignmentLayout",
    "Layout/MultilineBlockLayout",
    "Layout/MultilineHashBraceLayout",
    "Layout/MultilineHashKeyLineBreaks",
    "Layout/MultilineMethodCallBraceLayout",
    "Layout/MultilineMethodCallIndentation",
    "Layout/MultilineOperationIndentation",
    "Layout/RedundantLineBreak",
    "Layout/RescueEnsureAlignment",
    "Layout/SingleLineBlockChain",
    "Layout/SpaceAfterComma",
    "Layout/SpaceAfterNot",
    "Layout/SpaceAfterSemicolon",
    "Layout/SpaceAroundBlockParameters",
    "Layout/SpaceBeforeBlockBraces",
    "Layout/SpaceBeforeComma",
    "Layout/SpaceBeforeComment",
    "Layout/SpaceBeforeSemicolon",
    "Layout/SpaceInsideHashLiteralBraces",
    "Layout/SpaceInsideRangeLiteral",
    "Lint/AmbiguousBlockAssociation",
    "Lint/AmbiguousOperator",
    "Lint/AmbiguousOperatorPrecedence",
    "Lint/AmbiguousRange",
    "Lint/AmbiguousRegexpLiteral",
    "Lint/ArrayLiteralInRegexp",
    "Lint/AssignmentInCondition",
    "Lint/ConstantResolution",
    "Lint/CopDirectiveSyntax",
    "Lint/DuplicateRegexpCharacterClassElement",
    "Lint/DuplicateRescueException",
    "Lint/LiteralAssignmentInCondition",
    "Lint/MissingCopEnableDirective",
    "Lint/NoReturnInBeginEndBlocks",
    "Lint/RedundantCopDisableDirective",
    "Lint/RedundantCopEnableDirective",
    "Lint/RescueType",
    "Lint/SymbolConversion",
    "Lint/UnusedMethodArgument",
    "Lint/UselessAssignment",
    "Lint/UselessConstantScoping",
    "Metrics/AbcSize",
    "Metrics/BlockLength",
    "Metrics/BlockNesting",
    "Metrics/ClassLength",
    "Metrics/CyclomaticComplexity",
    "Metrics/MethodLength",
    "Metrics/PerceivedComplexity",
    "Naming/BlockParameterName",
    "Naming/ConstantName",
    "Naming/HeredocDelimiterNaming",
    "Naming/VariableName",
    "Naming/VariableNumber",
    "Style/AccessorGrouping",
    "Style/AutoResourceCleanup",
    "Style/ConstantVisibility",
    "Style/Copyright",
    "Style/Documentation",
    "Style/DocumentationMethod",
    "Style/EvalWithLocation",
    "Style/ExplicitBlockArgument",
    "Style/FetchEnvVar",
    "Style/FileWrite",
    "Style/IfUnlessModifier",
    "Style/InlineComment",
    "Style/MethodCallWithArgsParentheses",
    "Style/MethodCallWithoutArgsParentheses",
    "Style/MissingElse",
    "Style/MultilineTernaryOperator",
    "Style/RedundantAssignment",
    "Style/RedundantParentheses",
];

pub(crate) fn intentionally_pending(cop: &str) -> bool {
    INTENTIONALLY_PENDING_COP_NAMES.binary_search(&cop).is_ok()
}

pub(crate) fn cop_names() -> Vec<&'static str> {
    let mut names = prism::cop_names();
    names.extend(text::LEGACY_COP_NAMES);
    names.retain(|cop| !intentionally_pending(cop));
    names.sort_unstable();
    names.dedup();
    names
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn public_inventory_is_sorted_unique_and_complete() {
        let names = cop_names();
        assert_eq!(names.len(), 533);
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(INTENTIONALLY_PENDING_COP_NAMES
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
        assert!(INTENTIONALLY_PENDING_COP_NAMES
            .iter()
            .all(|cop| !names.contains(cop)));
        assert!(text::LEGACY_COP_NAMES
            .windows(2)
            .all(|pair| pair[0] < pair[1]));
    }
}
