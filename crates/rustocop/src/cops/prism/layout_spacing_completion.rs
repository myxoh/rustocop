use super::*;

define_cops! {
    AssignmentIndentation => "Layout/AssignmentIndentation" => any_node(assignment_indentation),
    BeginEndAlignment => "Layout/BeginEndAlignment" => node(as_begin_node, begin_end_alignment),
    EndOfLine => "Layout/EndOfLine" => source(end_of_line),
    FirstParameterIndentation => "Layout/FirstParameterIndentation" => source(first_parameter_indentation),
    SpaceBeforeBrackets => "Layout/SpaceBeforeBrackets" => call(space_before_brackets),
    SpaceBeforeFirstArg => "Layout/SpaceBeforeFirstArg" => call(space_before_first_arg),
    SpaceInsideStringInterpolation => "Layout/SpaceInsideStringInterpolation" => node(as_embedded_statements_node, space_inside_string_interpolation),
}

fn assignment_indentation(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(value) = assignment_value(node) else {
        return;
    };
    let source = context.source();
    let node_start = node.location().start_offset();
    let value_start = value.location().start_offset();
    let Some(operator) = source[node_start..value_start].rfind('=') else {
        return;
    };
    let operator = node_start + operator;
    let assignment_line_start = source[..operator].rfind('\n').map_or(0, |at| at + 1);
    let value_line_start = source[..value_start].rfind('\n').map_or(0, |at| at + 1);
    let assignment_line_end = source[assignment_line_start..]
        .find('\n')
        .map_or(source.len(), |at| assignment_line_start + at);
    if assignment_line_start == value_line_start
        || !source[assignment_line_start..assignment_line_end].is_ascii()
    {
        return;
    }
    let width = context.config_usize("IndentationWidth", 2);
    let line_indentation = source[assignment_line_start..assignment_line_end].len()
        - source[assignment_line_start..assignment_line_end]
            .trim_start()
            .len();
    let prefix = &source[assignment_line_start..operator];
    let chained = prefix.contains(" = ") || prefix.matches('=').count() > 0;
    let node_line_start = source[..node_start].rfind('\n').map_or(0, |at| at + 1);
    let multi_base = node
        .as_multi_write_node()
        .map(|_| node_start - node_line_start);
    let current_start = if node_line_start == assignment_line_start {
        node_start
    } else {
        assignment_line_start + line_indentation
    };
    let chain_start = context
        .ancestors()
        .iter()
        .filter(|ancestor| assignment_value(ancestor).is_some())
        .map(Node::location)
        .map(|location| location.start_offset())
        .filter(|start| {
            source[..*start].rfind('\n').map_or(0, |at| at + 1) == assignment_line_start
        })
        .chain(std::iter::once(current_start))
        .min()
        .unwrap_or(current_start);
    let base = if let Some(base) = multi_base {
        base
    } else if chained {
        line_indentation
    } else {
        chain_start - assignment_line_start
    };
    let current = value_start - value_line_start;
    let expected = base + width;
    if current == expected {
        return;
    }
    let location = value.location();
    let delta = expected as isize - current as isize;
    let edits = context
        .source_file()
        .lines()
        .filter(|(offset, _)| {
            value_line_start <= *offset && *offset < location.end_offset()
        })
        .filter_map(|(offset, line)| {
            if line.trim().is_empty() {
                return None;
            }
            let indentation = line.len() - line.trim_start().len();
            let adjusted = (indentation as isize + delta).max(0) as usize;
            Some((offset..offset + indentation, " ".repeat(adjusted)))
        })
        .collect::<Vec<_>>();
    context.replace_many(
        "Indent the first line of the right-hand-side of a multi-line assignment.",
        &location,
        edits,
    );
}

fn assignment_value<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    if let Some(write) = node.as_local_variable_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_instance_variable_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_class_variable_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_global_variable_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_constant_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_constant_path_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_local_variable_or_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_instance_variable_or_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_class_variable_or_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_global_variable_or_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_constant_or_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_constant_path_or_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_local_variable_and_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_instance_variable_and_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_class_variable_and_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_global_variable_and_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_constant_and_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_constant_path_and_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_multi_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_call_operator_write_node() {
        Some(write.value())
    } else if let Some(write) = node.as_index_operator_write_node() {
        Some(write.value())
    } else if let Some(call) = node.as_call_node().filter(|call| call.equal_loc().is_some()) {
        call.arguments()?.arguments().iter().last()
    } else {
        None
    }
}

fn begin_end_alignment(node: &ruby_prism::BeginNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(begin_keyword), Some(end_keyword)) =
        (node.begin_keyword_loc(), node.end_keyword_loc())
    else {
        return;
    };
    let source = context.source();
    let begin_line_start = source[..begin_keyword.start_offset()]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let end_line_start = source[..end_keyword.start_offset()]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    if begin_line_start == end_line_start {
        return;
    }
    let begin_column = begin_keyword.start_offset() - begin_line_start;
    let actual = end_keyword.start_offset() - end_line_start;
    let style = context
        .config_value("EnforcedStyleAlignWith")
        .unwrap_or("begin");
    let line = &source[begin_line_start
        ..source[begin_line_start..]
            .find('\n')
            .map_or(source.len(), |offset| begin_line_start + offset)];
    let expected = if style == "start_of_line" {
        line.len() - line.trim_start().len()
    } else {
        begin_column
    };
    if actual == expected {
        return;
    }
    let begin_line = source[..begin_keyword.start_offset()]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let end_line = source[..end_keyword.start_offset()]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let reference = if style == "start_of_line" {
        line.trim().to_string()
    } else {
        "begin".to_string()
    };
    context.replace(
        format!(
            "`end` at {end_line}, {actual} is not aligned with `{reference}` at {begin_line}, {expected}."
        ),
        end_keyword.start_offset()..end_keyword.end_offset(),
        end_line_start..end_keyword.start_offset(),
        " ".repeat(expected),
    );
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

fn space_before_brackets(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"[]" | b"[]=") {
        return;
    }
    let (Some(receiver), Some(opening)) = (node.receiver(), node.opening_loc()) else {
        return;
    };
    let start = receiver.location().end_offset();
    let end = opening.start_offset();
    if start >= end
        || !context.source()[start..end]
            .bytes()
            .all(|byte| byte.is_ascii_whitespace())
    {
        return;
    }
    context.remove(
        "Remove the space before the opening brackets.",
        start..end,
        start..end,
    );
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
            // RuboCop's `range_between` preserves Parser's reversed empty
            // range, whose JSON location ends immediately before it starts.
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

fn space_inside_string_interpolation(
    node: &ruby_prism::EmbeddedStatementsNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let source = context.source();
    let spaced = context.policy().enforced_style("no_space") == "space";
    let location = node.location();
    let opening = location.start_offset();
    let closing = location.end_offset().saturating_sub(1);
    let Some(inner) = source.get(opening + 2..closing) else {
        return;
    };
    if inner.trim().is_empty() || inner.contains('\n') {
        return;
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
                    closing..(closing + 1).min(context.source().len()),
                    closing..closing,
                    "",
                );
            } else {
                context.insert(
                    "Use space inside string interpolation.",
                    closing..(closing + 1).min(context.source().len()),
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
}
