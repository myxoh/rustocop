use super::helpers::*;
use crate::*;

pub(super) fn before_prism(
    lines: &mut [SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    check_empty_ensure(lines, options, offenses);
    check_small_line_cops(lines, options, offenses);
}

pub(super) fn after_prism(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    check_predicate_prefix(lines, options, offenses);
    check_accessor_method_name(lines, options, offenses);
    check_missing_super(lines, options, offenses);
    check_empty_block(lines, options, offenses);
    check_unused_method_argument(lines, options, offenses);
    check_debugger(lines, options, offenses);
}

fn check_empty_ensure(lines: &mut [SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Lint/EmptyEnsure";
    if !cop_enabled(options, cop) {
        return;
    }

    for index in 0..lines.len() {
        if !lines[index].body.trim_start().starts_with("ensure") {
            continue;
        }

        let has_statement = lines[index + 1..]
            .iter()
            .take_while(|candidate| {
                !matches!(candidate.body.trim(), "end" | "rescue" | "else" | "ensure")
            })
            .any(|candidate| {
                let body = candidate.body.trim();
                !body.is_empty() && !body.starts_with('#')
            });
        if !has_statement {
            let indentation = leading_spaces(&lines[index].body);
            push_offense(
                offenses,
                cop,
                "Empty `ensure` block detected.",
                index + 1,
                indentation + 1,
                "ensure".len(),
                true,
                options.autocorrect,
            );
            if options.autocorrect {
                lines[index]
                    .body
                    .replace_range(indentation..indentation + "ensure".len(), "");
            }
        }
    }
}

fn check_small_line_cops(lines: &mut [SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let mut begin_without_rescue = Vec::new();

    for (index, line) in lines.iter_mut().enumerate() {
        let original = line.body.clone();
        let trimmed = original.trim_start();
        let indentation = leading_spaces(&original);

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
            && cop_enabled(options, "Lint/UselessElseWithoutRescue")
        {
            push_offense(
                offenses,
                "Lint/UselessElseWithoutRescue",
                "`else` without `rescue` is useless.",
                index + 1,
                indentation + 1,
                4,
                false,
                false,
            );
        }

        if cop_enabled(options, "Lint/TrailingCommaInAttributeDeclaration")
            && ["attr_reader", "attr_writer", "attr_accessor"]
                .iter()
                .any(|keyword| trimmed.starts_with(keyword))
            && trimmed.ends_with(',')
        {
            let comma = original.rfind(',').expect("trailing comma");
            push_offense(
                offenses,
                "Lint/TrailingCommaInAttributeDeclaration",
                "Avoid leaving a trailing comma in attribute declarations.",
                index + 1,
                comma + 1,
                1,
                true,
                options.autocorrect,
            );
            if options.autocorrect {
                line.body.remove(comma);
            }
        }

        if cop_enabled(options, "Style/EndBlock") && trimmed.starts_with("END ") {
            push_offense(
                offenses,
                "Style/EndBlock",
                "Avoid the use of `END` blocks. Use `Kernel#at_exit` instead.",
                index + 1,
                indentation + 1,
                3,
                true,
                options.autocorrect,
            );
            if options.autocorrect {
                line.body
                    .replace_range(indentation..indentation + 3, "at_exit");
            }
        }

        if cop_enabled(options, "Style/ColonMethodDefinition") && trimmed.starts_with("def ") {
            if let Some(relative) = trimmed[4..].find("::") {
                let column = indentation + 4 + relative;
                push_offense(
                    offenses,
                    "Style/ColonMethodDefinition",
                    "Do not use `::` for defining class methods.",
                    index + 1,
                    column + 1,
                    2,
                    true,
                    options.autocorrect,
                );
                if options.autocorrect {
                    line.body.replace_range(column..column + 2, ".");
                }
            }
        }

        if cop_enabled(options, "Style/EmptyLambdaParameter") {
            if let Some(start) = original.find("-> ()") {
                push_offense(
                    offenses,
                    "Style/EmptyLambdaParameter",
                    "Omit parentheses for the empty lambda parameters.",
                    index + 1,
                    start + 4,
                    2,
                    true,
                    options.autocorrect,
                );
                if options.autocorrect {
                    line.body.replace_range(start + 2..start + 5, "");
                }
            }
        }

        if cop_enabled(options, "Lint/BigDecimalNew") {
            if let Some(new_start) = original.find("BigDecimal.new") {
                let selector = new_start + "BigDecimal.".len();
                push_offense(
                    offenses,
                    "Lint/BigDecimalNew",
                    "`BigDecimal.new()` is deprecated. Use `BigDecimal()` instead.",
                    index + 1,
                    selector + 1,
                    3,
                    true,
                    options.autocorrect,
                );
                if options.autocorrect {
                    line.body.replace_range(selector - 1..selector + 3, "");
                    if line.body[0..new_start].ends_with("::") {
                        line.body.replace_range(new_start - 2..new_start, "");
                    }
                }
            }
        }

        if cop_enabled(options, "Style/InlineComment") {
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
                        false,
                        false,
                    );
                }
            }
        }

        if cop_enabled(options, "Style/DoubleCopDisableDirective") {
            let directive = if original.matches("# rubocop:disable ").count() > 1 {
                Some("disable")
            } else if original.matches("# rubocop:todo ").count() > 1 {
                Some("todo")
            } else {
                None
            };
            if let Some(directive) = directive {
                let marker = format!("# rubocop:{} ", directive);
                let start = original.find(&marker).expect("duplicate directive");
                push_offense(
                    offenses,
                    "Style/DoubleCopDisableDirective",
                    "More than one disable comment on one line.",
                    index + 1,
                    start + 1,
                    original[start..].chars().count(),
                    true,
                    options.autocorrect,
                );
                if options.autocorrect {
                    let names = original[start..]
                        .split(&marker)
                        .filter(|name| !name.is_empty())
                        .map(str::trim)
                        .collect::<Vec<_>>()
                        .join(", ");
                    line.body
                        .replace_range(start.., &format!("{}{}", marker, names));
                }
            }
        }
    }
}

fn check_predicate_prefix(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Naming/PredicatePrefix";
    if !cop_enabled(options, cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim_start();
        let Some(name) = trimmed.strip_prefix("def ").and_then(first_identifier) else {
            continue;
        };

        if name.ends_with('?')
            && (name.starts_with("is_") || name.starts_with("has_") || name.starts_with("have_"))
        {
            push_offense(
                offenses,
                cop,
                "Rename predicate method to remove redundant predicate prefix.",
                index + 1,
                line.body.find("def").unwrap_or(0) + 5,
                name.len(),
                false,
                false,
            );
        }
    }
}

fn check_accessor_method_name(
    lines: &[SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Naming/AccessorMethodName";
    if !cop_enabled(options, cop) {
        return;
    }

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim_start();
        let Some(name) = trimmed.strip_prefix("def ").and_then(first_identifier) else {
            continue;
        };

        if name.starts_with("get_") || name.starts_with("set_") {
            push_offense(
                offenses,
                cop,
                "Do not prefix reader method names with `get_` or writer method names with `set_`.",
                index + 1,
                line.body.find("def").unwrap_or(0) + 5,
                name.len(),
                false,
                false,
            );
        }
    }
}

fn check_missing_super(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Lint/MissingSuper";
    if !cop_enabled(options, cop) {
        return;
    }

    let mut in_subclass = false;
    let mut initialize_start = None;
    let mut saw_super = false;

    for (index, line) in lines.iter().enumerate() {
        let trimmed = line.body.trim();

        if trimmed.starts_with("class ") && trimmed.contains(" < ") {
            in_subclass = true;
            continue;
        }

        if in_subclass && trimmed.starts_with("def initialize") {
            initialize_start = Some(index);
            saw_super = false;
            continue;
        }

        if initialize_start.is_some() && trimmed.starts_with("super") {
            saw_super = true;
        }

        if initialize_start.is_some() && trimmed == "end" {
            if !saw_super {
                let start = initialize_start.unwrap();
                push_offense(
                    offenses,
                    cop,
                    "Call `super` to initialize state of the parent class.",
                    start + 1,
                    leading_spaces(&lines[start].body) + 1,
                    lines[start].body.trim().len(),
                    false,
                    false,
                );
            }
            initialize_start = None;
        }
    }
}

fn check_empty_block(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Lint/EmptyBlock";
    if !cop_enabled(options, cop) {
        return;
    }

    for index in 0..lines.len() {
        let trimmed = lines[index].body.trim();
        if trimmed.ends_with("{}") || trimmed.ends_with("{ }") {
            push_offense(
                offenses,
                cop,
                "Empty block detected.",
                index + 1,
                lines[index].body.find('{').unwrap_or(0) + 1,
                2,
                false,
                false,
            );
        } else if trimmed.ends_with(" do")
            && index + 1 < lines.len()
            && lines[index + 1].body.trim() == "end"
        {
            push_offense(
                offenses,
                cop,
                "Empty block detected.",
                index + 1,
                lines[index].body.find("do").unwrap_or(0) + 1,
                2,
                false,
                false,
            );
        }
    }
}

fn check_unused_method_argument(
    lines: &[SourceLine],
    options: &Options,
    offenses: &mut Vec<Offense>,
) {
    let cop = "Lint/UnusedMethodArgument";
    if !cop_enabled(options, cop) {
        return;
    }

    let mut index = 0;
    while index < lines.len() {
        let line = &lines[index].body;
        let trimmed = line.trim();
        if !trimmed.starts_with("def ") || !trimmed.contains('(') {
            index += 1;
            continue;
        }

        let args = method_arguments(trimmed);
        if args.is_empty() {
            index += 1;
            continue;
        }

        let end = find_matching_end(lines, index).unwrap_or(index);
        let body = lines[index + 1..end]
            .iter()
            .map(|line| line.body.as_str())
            .collect::<Vec<&str>>()
            .join("\n");

        for arg in args {
            if arg.starts_with('_') || body.contains(&arg) {
                continue;
            }

            push_offense(
                offenses,
                cop,
                &format!("Unused method argument - `{}`.", arg),
                index + 1,
                line.find(&arg).unwrap_or(0) + 1,
                arg.len(),
                false,
                false,
            );
        }

        index = end + 1;
    }
}

fn check_debugger(lines: &[SourceLine], options: &Options, offenses: &mut Vec<Offense>) {
    let cop = "Lint/Debugger";
    if !cop_enabled(options, cop) {
        return;
    }

    let debuggers = [
        "binding.pry",
        "binding.irb",
        "debugger",
        "byebug",
        "save_and_open_page",
        "save_and_open_screenshot",
    ];

    for (index, line) in lines.iter().enumerate() {
        for debugger in debuggers {
            if let Some(position) = strip_comment(&line.body).find(debugger) {
                push_offense(
                    offenses,
                    cop,
                    "Remove debugger entry point.",
                    index + 1,
                    position + 1,
                    debugger.len(),
                    false,
                    false,
                );
            }
        }
    }
}
