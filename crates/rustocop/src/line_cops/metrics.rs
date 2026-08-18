use super::helpers::*;
use super::{push_offense, Offense, SourceLine};
use crate::{cop_enabled, Options};

pub(super) fn after_prism(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    check_metrics(lines, options, offenses);
}

fn check_metrics(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    check_method_length(lines, options, offenses);
    check_block_length(lines, options, offenses);
    check_abc_size(lines, options, offenses);
}

fn check_method_length(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Metrics/MethodLength";
    if !cop_enabled(options, cop) {
        return;
    }

    let max = 25;
    for (index, line) in lines.iter().enumerate() {
        if !line.body.trim().starts_with("def ") {
            continue;
        }

        if let Some(end) = find_matching_end(lines, index) {
            let length = lines[index + 1..end]
                .iter()
                .filter(|line| !line.body.trim().is_empty())
                .count();

            if length > max {
                push_offense(
                    offenses,
                    cop,
                    &format!("Method has too many lines. [{}/{}]", length, max),
                    index + 1,
                    leading_spaces(&line.body) + 1,
                    line.body.trim().len(),
                    false,
                    false,
                );
            }
        }
    }
}

fn check_block_length(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Metrics/BlockLength";
    if !cop_enabled(options, cop) {
        return;
    }

    let max = 25;
    let mut stack = Vec::<usize>::new();
    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();
        if trimmed.ends_with(" do") || trimmed.contains(" do |") {
            stack.push(index);
        } else if trimmed == "end" {
            if let Some(start) = stack.pop() {
                let length = lines[start + 1..index]
                    .iter()
                    .filter(|line| !line.body.trim().is_empty())
                    .count();

                if length > max {
                    push_offense(
                        offenses,
                        cop,
                        &format!("Block has too many lines. [{}/{}]", length, max),
                        start + 1,
                        leading_spaces(&lines[start].body) + 1,
                        lines[start].body.trim().len(),
                        false,
                        false,
                    );
                }
            }
        }
    }
}

fn check_abc_size(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Metrics/AbcSize";
    if !cop_enabled(options, cop) {
        return;
    }

    let max = 100.0;
    for (index, line) in lines.iter().enumerate() {
        if !line.body.trim().starts_with("def ") {
            continue;
        }

        if let Some(end) = find_matching_end(lines, index) {
            let body = lines[index + 1..end]
                .iter()
                .map(|line| line.body.as_str())
                .collect::<Vec<&str>>()
                .join("\n");
            let assignments = body
                .matches('=')
                .count()
                .saturating_sub(body.matches("==").count());
            let branches = body.matches('.').count() + body.matches(" do").count();
            let conditions = [" if ", " unless ", "&&", "||", " case "]
                .iter()
                .map(|needle| body.matches(needle).count())
                .sum::<usize>();
            let score =
                ((assignments * assignments + branches * branches + conditions * conditions)
                    as f64)
                    .sqrt();

            if score > max {
                push_offense(
                    offenses,
                    cop,
                    &format!(
                        "Assignment Branch Condition size for method is too high. [{:.1}/{}]",
                        score, max
                    ),
                    index + 1,
                    leading_spaces(&line.body) + 1,
                    line.body.trim().len(),
                    false,
                    false,
                );
            }
        }
    }
}
