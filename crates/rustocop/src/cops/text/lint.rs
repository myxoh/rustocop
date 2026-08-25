use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;

pub(super) fn before_prism(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_small_line_cops(lines, options, offenses);
}

pub(super) fn after_prism(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    super::lint_semantic::check(lines, options, offenses);
}

#[allow(clippy::too_many_lines)]
fn check_small_line_cops(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let mut begin_without_rescue = Vec::new();
    let next_statement_is_definition = (0..lines.len())
        .map(|index| {
            lines[index + 1..]
                .iter()
                .map(|line| line.body.trim_start())
                .find(|line| !line.trim().is_empty() && !line.starts_with('#'))
                .is_some_and(|line| line.starts_with("def "))
        })
        .collect::<Vec<_>>();

    for (index, line) in lines.iter_mut().enumerate() {
        let original = line.body.clone();
        let trimmed = original.trim_start();
        let indentation = leading_spaces(&original);

        check_useless_else(
            trimmed,
            index,
            indentation,
            &mut begin_without_rescue,
            options,
            offenses,
        );
        check_trailing_attribute_comma(
            index,
            line,
            &original,
            next_statement_is_definition[index],
            options,
            offenses,
        );

        if options.cop_enabled("Style/InlineComment")
            && !crate::cops::intentionally_pending("Style/InlineComment")
        {
            if let Some(comment) = original.find('#') {
                let text = &original[comment..];
                if !original[..comment].trim().is_empty() && !text.starts_with("# rubocop:") {
                    push_offense(
                        offenses,
                        "Style/InlineComment",
                        "Avoid trailing inline comments.",
                        index + 1,
                        comment + 1,
                        text.chars().count(),
                        CorrectionStatus::Unavailable,
                    );
                }
            }
        }
    }
}

fn check_useless_else(
    trimmed: &str,
    index: usize,
    indentation: usize,
    begin_without_rescue: &mut Vec<bool>,
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    if trimmed == "begin" {
        begin_without_rescue.push(true);
    } else if trimmed.starts_with("rescue") {
        if let Some(begin) = begin_without_rescue.last_mut() {
            *begin = false;
        }
    } else if trimmed == "end" {
        begin_without_rescue.pop();
    } else if trimmed == "else"
        && begin_without_rescue.last().copied() == Some(true)
        && options.cop_enabled("Lint/UselessElseWithoutRescue")
        && !options.target_ruby_version.at_least(2, 6)
    {
        push_offense(
            offenses,
            "Lint/UselessElseWithoutRescue",
            "`else` without `rescue` is useless.",
            index + 1,
            indentation + 1,
            4,
            CorrectionStatus::Unavailable,
        );
    }
}

fn check_trailing_attribute_comma(
    index: usize,
    line: &mut SourceLine,
    original: &str,
    next_statement_is_definition: bool,
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let trimmed = original.trim_start();
    if !options.cop_enabled("Lint/TrailingCommaInAttributeDeclaration")
        || !["attr_reader", "attr_writer", "attr_accessor", "attr"]
            .iter()
            .any(|keyword| trimmed.starts_with(keyword))
        || !trimmed.ends_with(',')
        || !next_statement_is_definition
    {
        return;
    }
    let comma = original.rfind(',').expect("trailing comma");
    push_offense(
        offenses,
        "Lint/TrailingCommaInAttributeDeclaration",
        "Avoid leaving a trailing comma in attribute declarations.",
        index + 1,
        comma + 1,
        1,
        CorrectionStatus::correctable(
            options.autocorrect_for("Lint/TrailingCommaInAttributeDeclaration"),
        ),
    );
    if options.autocorrect_for("Lint/TrailingCommaInAttributeDeclaration") {
        line.body.remove(comma);
    }
}
