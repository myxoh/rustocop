use super::helpers::*;
use super::{push_offense, CorrectionStatus, Offense, SourceLine};
use crate::config::InspectionConfig;

const TRAILING_WHITESPACE_COP: &str = "Layout/TrailingWhitespace";

pub(super) fn before_prism(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_trailing_whitespace(lines, options, offenses);
}

pub(super) fn after_prism(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    check_line_length(lines, options, offenses);
    check_indentation(lines, options, offenses);
}

fn check_trailing_whitespace(
    lines: &mut [SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = TRAILING_WHITESPACE_COP;
    if !options.cop_enabled(cop) {
        return;
    }

    let allow_in_heredoc = options
        .cop_config
        .bool(cop, "AllowInHeredoc")
        .unwrap_or(false);
    let mut openings = heredoc_openings(lines);
    let mut heredoc: Option<(String, bool, Option<usize>)> = None;
    let mut in_documentation_comment = false;

    for (index, line) in lines.iter_mut().enumerate() {
        let in_heredoc = heredoc.is_some();
        let heredoc_is_interpolated = heredoc
            .as_ref()
            .is_none_or(|(_, interpolated, _)| *interpolated);
        let heredoc_indentation = heredoc
            .as_ref()
            .and_then(|(_, _, indentation)| *indentation);
        let closes_heredoc = heredoc
            .as_ref()
            .is_some_and(|(terminator, _, _)| line.body.trim() == terminator);
        if !in_heredoc && !in_documentation_comment && line.body == "__END__" {
            break;
        }

        if !in_heredoc && line.body.starts_with("=begin") {
            in_documentation_comment = true;
        }

        let length = trailing_whitespace_len(&line.body);
        if length != 0 && !(allow_in_heredoc && in_heredoc) {
            let correctable = !in_heredoc || heredoc_is_interpolated;
            let corrected = options.autocorrect && correctable;
            let column = line.body.chars().count() - length + 1;
            push_offense(
                offenses,
                cop,
                "Trailing whitespace detected.",
                index + 1,
                column,
                length,
                CorrectionStatus::from_flags(correctable, corrected),
            );

            if corrected {
                if in_heredoc && line.body.chars().count() > length {
                    escape_heredoc_trailing_whitespace(&mut line.body, length);
                } else if in_heredoc
                    && heredoc_indentation.is_some_and(|indentation| length > indentation)
                {
                    let indentation = heredoc_indentation.expect("checked above");
                    escape_whitespace_beyond_indentation(&mut line.body, indentation);
                } else {
                    trim_trailing_spaces(&mut line.body);
                }
            }
        }

        if in_heredoc {
            if closes_heredoc {
                heredoc = None;
            }
        } else if let Some(opening) = openings[index].take() {
            heredoc = Some(opening);
        }

        if !in_heredoc && line.body.starts_with("=end") {
            in_documentation_comment = false;
        }
    }
}

fn heredoc_opening(line: &str) -> Option<(String, bool, bool)> {
    let marker = line.find("<<")?;
    let mut rest = &line[marker + 2..];
    let squiggly = rest.starts_with('~');
    rest = rest.strip_prefix(['-', '~']).unwrap_or(rest);
    let quote = rest
        .chars()
        .next()
        .filter(|character| matches!(character, '\'' | '"' | '`'));
    if quote.is_some() {
        rest = &rest[1..];
    }
    let name = rest
        .chars()
        .take_while(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    (!name.is_empty()).then_some((name, quote != Some('\''), squiggly))
}

fn heredoc_openings(lines: &[SourceLine]) -> Vec<Option<(String, bool, Option<usize>)>> {
    lines
        .iter()
        .enumerate()
        .map(|(index, line)| {
            let (terminator, interpolated, squiggly) = heredoc_opening(&line.body)?;
            let indentation = squiggly.then(|| {
                lines[index + 1..]
                    .iter()
                    .take_while(|line| line.body.trim() != terminator)
                    .filter(|line| !line.body.trim().is_empty())
                    .map(|line| {
                        line.body
                            .chars()
                            .take_while(|character| matches!(character, ' ' | '\t'))
                            .count()
                    })
                    .min()
                    .unwrap_or(0)
            });
            Some((terminator, interpolated, indentation))
        })
        .collect()
}

fn escape_heredoc_trailing_whitespace(line: &mut String, length: usize) {
    let split = line
        .char_indices()
        .nth(line.chars().count() - length)
        .map_or(line.len(), |(offset, _)| offset);
    let whitespace = line[split..].to_string();
    line.replace_range(split.., &format!("#{{'{whitespace}'}}"));
}

fn escape_whitespace_beyond_indentation(line: &mut String, indentation: usize) {
    let split = line
        .char_indices()
        .nth(indentation)
        .map_or(line.len(), |(offset, _)| offset);
    let whitespace = line[split..].to_string();
    line.replace_range(split.., &format!("#{{'{whitespace}'}}"));
}

fn check_line_length(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Layout/LineLength";
    if !options.cop_enabled(cop) {
        return;
    }

    let max = options
        .cop_config
        .value(cop, "Max")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120);
    for (index, line) in lines.iter().enumerate() {
        let length = line.body.chars().count();
        if length > max {
            push_offense(
                offenses,
                cop,
                &format!("Line is too long. [{}/{}]", length, max),
                index + 1,
                max + 1,
                length - max,
                CorrectionStatus::Unavailable,
            );
        }
    }
}

fn check_extra_spacing(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Layout/ExtraSpacing";
    if !options.cop_enabled(cop) {
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
                CorrectionStatus::Pending,
            );
        }
    }
}

fn check_indentation(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    for cop in ["Layout/IndentationConsistency", "Layout/IndentationWidth"] {
        if crate::cops::intentionally_pending(cop) {
            continue;
        }
        if !options.cop_enabled(cop) {
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
                    CorrectionStatus::Unavailable,
                );
            }
        }
    }
}

fn check_end_alignment(
    lines: &[SourceLine],
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Layout/EndAlignment";
    if !options.cop_enabled(cop) {
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
                        CorrectionStatus::Unavailable,
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
    options: &InspectionConfig,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Layout/FirstHashElementIndentation";
    if !options.cop_enabled(cop) {
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
                CorrectionStatus::Unavailable,
            );
        }
    }
}
