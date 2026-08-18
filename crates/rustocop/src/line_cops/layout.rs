use super::helpers::*;
use super::{push_offense, Offense, SourceLine};
use crate::cop_registry::TRAILING_WHITESPACE_COP;
use crate::{cop_enabled, Options};

pub(super) fn before_prism(
    lines: &mut [SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    check_trailing_whitespace(lines, options, offenses);
}

pub(super) fn after_prism(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    check_line_length(lines, options, offenses);
    check_extra_spacing(lines, options, offenses);
    check_indentation(lines, options, offenses);
    check_end_alignment(lines, options, offenses);
    check_first_hash_element_indentation(lines, options, offenses);
}

fn check_trailing_whitespace(
    lines: &mut [SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = TRAILING_WHITESPACE_COP;
    if !cop_enabled(options, cop) {
        return;
    }

    for (index, line) in lines.iter_mut().enumerate() {
        let length = trailing_whitespace_len(&line.body);
        if length == 0 {
            continue;
        }

        let corrected = options.autocorrect;
        let column = line.body.chars().count() - length + 1;
        push_offense(
            offenses,
            cop,
            "Trailing whitespace detected.",
            index + 1,
            column,
            length,
            true,
            corrected,
        );

        if corrected {
            trim_trailing_spaces(&mut line.body);
        }
    }
}

fn check_line_length(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Layout/LineLength";
    if !cop_enabled(options, cop) {
        return;
    }

    let max = 120;
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim_start();
        let length = line.body.chars().count();
        if length > max && !trimmed.starts_with('#') {
            push_offense(
                offenses,
                cop,
                &format!("Line is too long. [{}/{}]", length, max),
                index + 1,
                max + 1,
                length - max,
                false,
                false,
            );
        }
    }
}

fn check_extra_spacing(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Layout/ExtraSpacing";
    if !cop_enabled(options, cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        let text = strip_comment(&line.body);
        if let Some(column) = find_extra_spacing(text) {
            push_offense(
                offenses,
                cop,
                "Unnecessary spacing detected.",
                index + 1,
                column,
                2,
                true,
                false,
            );
        }
    }
}

fn check_indentation(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    for cop in ["Layout/IndentationConsistency", "Layout/IndentationWidth"] {
        if !cop_enabled(options, cop) {
            continue;
        }

        for (index, line) in lines.iter().enumerate() {
            if line.body.trim().is_empty() {
                continue;
            }

            let indent = leading_spaces(&line.body);
            if indent != line.body.len() - line.body.trim_start_matches(' ').len()
                || !indent.is_multiple_of(2)
            {
                push_offense(
                    offenses,
                    cop,
                    "Use 2 spaces for indentation.",
                    index + 1,
                    1,
                    indent.max(1),
                    false,
                    false,
                );
            }
        }
    }
}

fn check_end_alignment(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Layout/EndAlignment";
    if !cop_enabled(options, cop) {
        return;
    }

    let mut stack = Vec::<usize>::new();

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();
        let indent = leading_spaces(&line.body);

        if trimmed == "end" {
            if let Some(expected) = stack.pop() {
                if indent != expected {
                    push_offense(
                        offenses,
                        cop,
                        "`end` at this line is not aligned with the opening keyword.",
                        index + 1,
                        indent + 1,
                        3,
                        false,
                        false,
                    );
                }
            }
            continue;
        }

        if starts_block(trimmed) {
            stack.push(indent);
        }
    }
}

fn check_first_hash_element_indentation(
    lines: &[SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Layout/FirstHashElementIndentation";
    if !cop_enabled(options, cop) {
        return;
    }

    for index in 0..lines.len().saturating_sub(1) {
        let line = &lines[index].body;
        let next = &lines[index + 1].body;

        if line.trim_end().ends_with('{')
            && !next.trim().is_empty()
            && leading_spaces(next) <= leading_spaces(line)
        {
            push_offense(
                offenses,
                cop,
                "Indent the first key one step more than the opening brace.",
                index + 2,
                1,
                leading_spaces(next).max(1),
                false,
                false,
            );
        }
    }
}
