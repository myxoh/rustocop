use super::*;

define_cops! {
    FileRead => "Style/FileRead" => source(file_read),
    FileWrite => "Style/FileWrite" => source(file_write),
    IfWithSemicolon => "Style/IfWithSemicolon" => source(if_with_semicolon),
    MethodDefParentheses => "Style/MethodDefParentheses" => source(method_def_parentheses),
    WhileUntilModifier => "Style/WhileUntilModifier" => source(while_until_modifier),
}

fn file_read(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (open_method, read_method, preferred) in [
        ("File.open(", ").read", "File.read"),
        ("File.open(", ").binread", "File.binread"),
    ] {
        let mut search = 0;
        while let Some(relative) = source[search..].find(open_method) {
            let start = search + relative;
            let offense_start = if source.get(start.saturating_sub(2)..start) == Some("::") {
                start - 2
            } else {
                start
            };
            let Some(close_relative) = source[start + open_method.len()..].find(read_method) else {
                break;
            };
            let call_end = start + open_method.len() + close_relative + read_method.len();
            let arguments =
                &source[start + open_method.len()..start + open_method.len() + close_relative];
            if arguments.contains(',') {
                search = call_end;
                continue;
            }
            context.replace(
                format!("Use `{preferred}`."),
                offense_start..call_end,
                offense_start..call_end,
                format!(
                    "{}{preferred}({arguments})",
                    if offense_start < start { "::" } else { "" }
                ),
            );
            search = call_end;
        }
    }
}

fn file_write(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let mut search = 0;
    while let Some(relative) = source[search..].find("File.open(") {
        let start = search + relative;
        let offense_start = if source.get(start.saturating_sub(2)..start) == Some("::") {
            start - 2
        } else {
            start
        };
        let Some(write_relative) = source[start..].find(").write(") else {
            break;
        };
        let write = start + write_relative;
        let Some(end_relative) = source[write + ").write(".len()..].find(')') else {
            break;
        };
        let end = write + ").write(".len() + end_relative + 1;
        let open_args = &source[start + "File.open(".len()..write];
        let Some((path, mode)) = open_args.rsplit_once(',') else {
            search = end;
            continue;
        };
        let mode = mode.trim().trim_matches(['\'', '"']);
        let preferred = if mode.contains('b') {
            "File.binwrite"
        } else {
            "File.write"
        };
        if !mode.starts_with('w') && !mode.starts_with('a') {
            search = end;
            continue;
        }
        let content = &source[write + ").write(".len()..end - 1];
        context.replace(
            format!("Use `{preferred}`."),
            offense_start..end,
            offense_start..end,
            format!(
                "{}{preferred}({}, {content})",
                if offense_start < start { "::" } else { "" },
                path.trim()
            ),
        );
        search = end;
    }
}

fn if_with_semicolon(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let code = line.trim();
        let (keyword, condition_start) = if code.starts_with("if ") {
            ("if", 3)
        } else if code.starts_with("unless ") {
            ("unless", 7)
        } else {
            continue;
        };
        if !code.ends_with(" end") {
            continue;
        }
        let Some((condition, rest)) = code[condition_start..].split_once(';') else {
            continue;
        };
        let Some(body) = rest.trim().strip_suffix(" end") else {
            continue;
        };
        let Some((mut truthy, mut falsey)) = body
            .split_once(" else ")
            .or_else(|| body.strip_prefix("else ").map(|falsey| ("", falsey)))
        else {
            continue;
        };
        if keyword == "unless" {
            std::mem::swap(&mut truthy, &mut falsey);
        }
        let replacement = format!(
            "{} ? {} : {}",
            condition.trim(),
            if truthy.trim().is_empty() {
                "nil"
            } else {
                truthy.trim()
            },
            if falsey.trim().is_empty() {
                "nil"
            } else {
                falsey.trim()
            }
        );
        let start = offset + line.find(code).unwrap_or(0);
        context.replace(
            format!(
                "Do not use `{keyword} {};` - use a ternary operator instead.",
                condition.trim()
            ),
            start..start + code.len(),
            start..start + code.len(),
            replacement,
        );
    }
}

fn method_def_parentheses(context: &mut CopContext<'_, '_>) {
    let style = context
        .policy()
        .enforced_style("require_parentheses")
        .to_string();
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        let Some(signature) = trimmed.strip_prefix("def ") else {
            continue;
        };
        let name_end = signature.find([' ', '(']).unwrap_or(signature.len());
        let raw_parameters = signature[name_end..].trim_start();
        let parameters = raw_parameters
            .split_once(';')
            .map_or(raw_parameters, |(parameters, _)| parameters)
            .trim_end();
        if parameters.is_empty() {
            continue;
        }
        let indent = line.len() - trimmed.len();
        if style == "require_parentheses" && !parameters.starts_with('(') {
            let gap = signature[name_end..].len() - raw_parameters.len();
            let start = offset + indent + "def ".len() + name_end + gap;
            context.replace(
                "Use def with parentheses when there are parameters.",
                start..start + parameters.len(),
                start - gap..start + parameters.len(),
                format!("({parameters})"),
            );
        } else if style != "require_parentheses"
            && parameters.starts_with('(')
            && parameters.ends_with(')')
        {
            let start = offset + line.find('(').unwrap_or(0);
            let end = offset + line.rfind(')').unwrap_or(line.len() - 1) + 1;
            context.replace(
                "Do not use parentheses for method parameters.",
                start..end,
                start..end,
                parameters.trim_matches(['(', ')']),
            );
        }
    }
}

fn while_until_modifier(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(3) {
        let (start, header) = window[0];
        let (_, body) = window[1];
        let (end_offset, end) = window[2];
        let keyword = if header.trim_start().starts_with("while ") {
            "while"
        } else if header.trim_start().starts_with("until ") {
            "until"
        } else {
            continue;
        };
        if end.trim() != "end" || body.trim().is_empty() {
            continue;
        }
        let condition = header.trim_start()[keyword.len()..].trim();
        if condition.contains(" = ") {
            continue;
        }
        let replacement = format!("{} {keyword} {condition}", body.trim());
        let maximum = context
            .related_config_value("Layout/LineLength", "Max")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(120);
        if replacement.len() > maximum {
            continue;
        }
        let finish = end_offset + end.len();
        context.replace(
            format!("Favor modifier `{keyword}` usage when having a single-line body."),
            start..start + keyword.len(),
            start..finish,
            replacement,
        );
    }
}
