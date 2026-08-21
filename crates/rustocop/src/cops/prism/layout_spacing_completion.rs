use super::*;

define_cops! {
    AssignmentIndentation => "Layout/AssignmentIndentation" => source(assignment_indentation),
    BeginEndAlignment => "Layout/BeginEndAlignment" => source(begin_end_alignment),
    EndOfLine => "Layout/EndOfLine" => source(end_of_line),
    FirstParameterIndentation => "Layout/FirstParameterIndentation" => source(first_parameter_indentation),
    SpaceBeforeBrackets => "Layout/SpaceBeforeBrackets" => source(space_before_brackets),
    SpaceBeforeFirstArg => "Layout/SpaceBeforeFirstArg" => call(space_before_first_arg),
    SpaceInsideStringInterpolation => "Layout/SpaceInsideStringInterpolation" => source(space_inside_string_interpolation),
}

fn assignment_indentation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let width = context.config_usize("IndentationWidth", 2);
    for pair in lines.windows(2) {
        let (_, left) = pair[0];
        let (right_start, right) = pair[1];
        let left_trimmed = left.trim_end();
        if !left_trimmed.ends_with('=') || left_trimmed.ends_with("==") || right.trim().is_empty() {
            continue;
        }
        if !left.is_ascii() {
            continue;
        }
        let current = right.len() - right.trim_start().len();
        let expected = left.len() - left.trim_start().len() + width;
        if current == expected {
            continue;
        }
        let expression_start = right_start + current;
        context.replace(
            "Indent the first line of the right-hand-side of a multi-line assignment.",
            expression_start..right_start + right.len(),
            right_start..expression_start,
            " ".repeat(expected),
        );
    }
}

fn begin_end_alignment(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut stack = Vec::new();
    for (line_number, (offset, line)) in lines.iter().enumerate() {
        if let Some(begin_at) = line.find("begin") {
            stack.push((line_number, *offset, begin_at, line.trim_end().to_string()));
        }
        if line.trim() != "end" {
            continue;
        }
        let Some((begin_line, _, begin_column, begin_text)) = stack.pop() else {
            continue;
        };
        let actual = line.len() - line.trim_start().len();
        let style = context
            .config_value("EnforcedStyleAlignWith")
            .unwrap_or("begin");
        let expected = if style == "start_of_line" {
            0
        } else {
            begin_column
        };
        if actual == expected {
            continue;
        }
        let start = offset + actual;
        let reference = if style == "start_of_line" {
            begin_text.trim().to_string()
        } else {
            "begin".to_string()
        };
        context.replace(
            format!(
                "`end` at {}, {} is not aligned with `{reference}` at {}, {expected}.",
                line_number + 1,
                actual,
                begin_line + 1
            ),
            start..start + 3,
            *offset..start,
            " ".repeat(expected),
        );
    }
}

fn end_of_line(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let style = context.policy().enforced_style("native");
    let wants_crlf = style == "crlf";
    let mut bad_lines = Vec::new();
    let bytes = source.as_bytes();
    let data_start = source
        .find("\n__END__")
        .map_or(source.len(), |offset| offset + 1);
    let mut line_start = 0;
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'\n' {
            continue;
        }
        if line_start >= data_start {
            break;
        }
        let has_cr = index > 0 && bytes[index - 1] == b'\r';
        if has_cr != wants_crlf {
            bad_lines.push((line_start, index + 1));
        }
        line_start = index + 1;
    }
    let (Some(first), Some(last)) = (bad_lines.first(), bad_lines.last()) else {
        return;
    };
    let message = if wants_crlf {
        "Carriage return character missing."
    } else {
        "Carriage return character detected."
    };
    let end = if wants_crlf {
        first.1
    } else if bad_lines.len() == 1 {
        last.1 + 1
    } else {
        last.1
    };
    context.report(message, first.0..end);
}

fn first_parameter_indentation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for pair in lines.windows(2) {
        let (_, signature) = pair[0];
        let (parameter_start, parameter) = pair[1];
        let Some(opening) = signature.find('(') else {
            continue;
        };
        if !signature.trim_start().starts_with("def ")
            || !signature[opening + 1..].trim().is_empty()
        {
            continue;
        }
        let current = parameter.len() - parameter.trim_start().len();
        let style = context.policy().enforced_style("consistent");
        let width = context.config_usize("IndentationWidth", 2);
        let base = signature.len() - signature.trim_start().len();
        let expected = if style == "align_parentheses" {
            opening + 2
        } else {
            base + width
        };
        if current == expected {
            continue;
        }
        let start = parameter_start + current;
        let message = if style == "align_parentheses" {
            format!("Use {width} spaces for indentation in method args, relative to the position of the opening parenthesis.")
        } else {
            format!("Use {width} spaces for indentation in method args, relative to the start of the line where the left parenthesis is.")
        };
        let offense_end = parameter_start + parameter.trim_end_matches(',').trim_end().len();
        context.replace(
            message,
            start..offense_end,
            parameter_start..start,
            " ".repeat(expected),
        );
    }
}

fn space_before_brackets(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (at, _) in source.match_indices(" [") {
        let before = source[..at].bytes().next_back();
        if before.is_none_or(|byte| {
            !(byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b')' | b']'))
        }) {
            continue;
        }
        let line_start = source[..at].rfind('\n').map_or(0, |offset| offset + 1);
        let receiver = source[line_start..at]
            .split(|character: char| {
                !(character.is_alphanumeric() || matches!(character, '_' | '@' | '$'))
            })
            .next_back()
            .unwrap_or_default();
        let known_local = receiver.starts_with(['@', '$'])
            || source[line_start..at].trim_end().ends_with(')')
            || source[..line_start]
                .lines()
                .any(|line| line.trim_start().starts_with(&format!("{receiver} =")));
        if !known_local {
            continue;
        }
        context.remove(
            "Remove the space before the opening brackets.",
            at..at + 1,
            at..at + 1,
        );
    }
}

fn space_before_first_arg(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let name = node.name().as_slice();
    if node.opening_loc().is_some() || space_before_operator_or_setter(name) {
        return;
    }
    let Some(first_argument) = node.arguments().and_then(|arguments| arguments.arguments().first())
    else {
        return;
    };
    let Some(selector) = node.message_loc() else {
        return;
    };
    let selector_end = selector.end_offset();
    let argument_start = first_argument.location().start_offset();
    if argument_start < selector_end {
        return;
    }
    let file = context.source_file();
    if file.line_start(selector_end) != file.line_start(argument_start) {
        return;
    }
    let whitespace_start = context.source()[selector_end..argument_start]
        .rfind(|character: char| !matches!(character, ' ' | '\t'))
        .map_or(selector_end, |offset| selector_end + offset + 1);
    let space = whitespace_start..argument_start;
    if space.len() == 1 {
        return;
    }
    if !space.is_empty()
        && context.config_bool("AllowForAlignment", true)
        && aligned_first_argument(context, &first_argument)
    {
        return;
    }
    context.replace(
        "Put one space between the method name and the first argument.",
        if space.is_empty() {
            argument_start..argument_start.saturating_sub(1)
        } else {
            space.clone()
        },
        space,
        " ",
    );
}

fn space_before_operator_or_setter(name: &[u8]) -> bool {
    matches!(
        name,
        b"+" | b"-" | b"*" | b"/" | b"%" | b"**" | b"==" | b"!=" | b"==="
            | b"=~" | b"!~" | b"<" | b">" | b"<=" | b">=" | b"<=>" | b"<<"
            | b">>" | b"&" | b"|" | b"^" | b"[]" | b"[]=" | b"!" | b"~" | b"+@"
            | b"-@"
    ) || name.ends_with(b"=")
}

fn aligned_first_argument(context: &CopContext<'_, '_>, argument: &Node<'_>) -> bool {
    let file = context.source_file();
    let location = argument.location();
    let line_start = file.line_start(location.start_offset());
    let column = location.start_offset() - line_start;
    let token = file.node(argument);
    let lines = file.lines().collect::<Vec<_>>();
    let Some(current) = lines.iter().position(|(start, _)| *start == line_start) else {
        return false;
    };
    let base_indent = lines[current].1.len() - lines[current].1.trim_start().len();
    let aligns = |line: &str| {
        let bytes = line.as_bytes();
        column > 0
            && bytes.get(column - 1).is_some_and(u8::is_ascii_whitespace)
            && bytes.get(column).is_some_and(|byte| !byte.is_ascii_whitespace())
            || line
                .get(column..column.saturating_add(token.len()))
                .is_some_and(|candidate| candidate == token)
    };
    let eligible = |line: &str| {
        !line.trim().is_empty() && !line.trim_start().starts_with('#')
    };

    let nearest_before = lines[..current]
        .iter()
        .rev()
        .map(|(_, line)| *line)
        .find(|line| eligible(line));
    if nearest_before.is_some_and(aligns) {
        return true;
    }
    let nearest_after = lines[current + 1..]
        .iter()
        .map(|(_, line)| *line)
        .find(|line| eligible(line));
    if nearest_after.is_some_and(aligns) {
        return true;
    }

    let same_indent = |line: &str| {
        eligible(line) && line.len() - line.trim_start().len() == base_indent
    };
    lines[..current]
        .iter()
        .rev()
        .map(|(_, line)| *line)
        .find(|line| same_indent(line))
        .is_some_and(aligns)
        || lines[current + 1..]
            .iter()
            .map(|(_, line)| *line)
            .find(|line| same_indent(line))
            .is_some_and(aligns)
}

fn space_inside_string_interpolation(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let spaced = context.policy().enforced_style("no_space") == "space";
    let mut search = 0;
    while let Some(relative) = source[search..].find("#{") {
        let opening = search + relative;
        let Some(relative_close) = source[opening + 2..].find('}') else {
            break;
        };
        let closing = opening + 2 + relative_close;
        let inner = &source[opening + 2..closing];
        if inner.is_empty() {
            search = closing + 1;
            continue;
        }
        if inner.contains('\n') {
            search = closing + 1;
            continue;
        }
        let leading = inner.len() - inner.trim_start_matches([' ', '\t']).len();
        let trailing = inner.len() - inner.trim_end_matches([' ', '\t']).len();
        if spaced {
            if leading == 0 {
                let mut edits = vec![(opening + 2..opening + 2, " ".to_string())];
                if trailing == 0 {
                    edits.push((closing..closing, " ".to_string()));
                }
                context.replace_many(
                    "Use space inside string interpolation.",
                    opening..opening + 2,
                    edits,
                );
            }
            if trailing == 0 {
                if leading == 0 {
                    context.replace_indirectly(
                        "Use space inside string interpolation.",
                        closing..closing,
                        closing..closing,
                        "",
                    );
                } else {
                    context.insert(
                        "Use space inside string interpolation.",
                        closing..closing,
                        closing,
                        " ",
                    );
                }
            }
        } else if leading > 0 || trailing > 0 {
            let message = "Do not use space inside string interpolation.";
            if leading > 0 {
                context.replace(
                    message,
                    opening + 2..opening + 2 + leading,
                    opening + 2..closing,
                    inner.trim(),
                );
                if trailing > 0 {
                    context.replace_indirectly(
                        message,
                        closing - trailing..closing,
                        closing - trailing..closing,
                        "",
                    );
                }
            } else {
                context.remove(
                    message,
                    closing - trailing..closing,
                    closing - trailing..closing,
                );
            }
        }
        search = closing + 1;
    }
}
