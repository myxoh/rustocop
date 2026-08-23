use super::*;
use unicode_width::UnicodeWidthChar;

define_cops! {
    BlockAlignment => "Layout/BlockAlignment" => any_node(block_alignment_node),
    DotPosition => "Layout/DotPosition" => call(dot_position),
    EmptyLineBetweenDefs => "Layout/EmptyLineBetweenDefs" => node(as_statements_node, empty_line_between_defs),
    EmptyLinesAfterModuleInclusion => "Layout/EmptyLinesAfterModuleInclusion" => call(empty_lines_after_module_inclusion),
    EmptyLinesAroundAccessModifier => "Layout/EmptyLinesAroundAccessModifier" => call(empty_lines_around_access_modifier),
    FirstArgumentIndentation => "Layout/FirstArgumentIndentation" => any_node(first_argument_indentation),
}

fn block_alignment_node(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(block) = node.as_block_node() {
        block_alignment(&block, context);
    } else if let Some(lambda) = node.as_lambda_node() {
        let lambda_start = lambda.location().start_offset();
        if !context.source()[..lambda_start].trim_end().ends_with('(') {
            return;
        }
        let opening = lambda.opening_loc();
        let block_start = lambda_start;
        let block_column = context.source_file().column(block_start);
        block_alignment_locations(
            lambda.closing_loc(),
            opening,
            block_start,
            block_column,
            true,
            context,
        );
    }
}

fn dot_position(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(receiver), Some(dot)) = (node.receiver(), node.call_operator_loc()) else {
        return;
    };
    let selector = node.message_loc().or_else(|| node.opening_loc());
    let Some(selector) = selector else { return };
    let file = context.source_file();
    let heredoc_receiver = context.source()
        [receiver.location().start_offset()..receiver.location().end_offset()]
        .contains("<<");
    if file.same_line(selector.start_offset(), receiver.location().end_offset())
        && !heredoc_receiver
    {
        return;
    }

    let selector_line = line_index(context.source(), selector.start_offset());
    let receiver_line = line_index(context.source(), receiver.location().end_offset());
    let dot_line = line_index(context.source(), dot.start_offset());
    if heredoc_receiver
        && dot_line == selector_line
        && dot_line == line_index(context.source(), receiver.location().start_offset())
    {
        return;
    }
    if !heredoc_receiver && selector_line.saturating_sub(receiver_line.max(dot_line)) > 1 {
        return;
    }
    let style = context.policy().enforced_style("leading").to_string();
    let proper = if style == "leading" {
        dot_line == selector_line
    } else {
        dot_line != selector_line
    };
    if proper {
        return;
    }

    let operator = String::from_utf8_lossy(dot.as_slice()).into_owned();
    let message = if style == "leading" {
        format!("Place the {operator} on the next line, together with the method name.")
    } else {
        format!(
            "Place the {operator} on the previous line, together with the method call receiver."
        )
    };
    let dot_line_source = line(context.source(), dot_line);
    let removal = if dot_line_source.trim() == operator {
        line_start(context.source(), dot_line)..line_start(context.source(), dot_line + 1)
    } else {
        dot.start_offset()..dot.end_offset()
    };
    let insertion = if style == "leading" {
        selector.start_offset()
    } else {
        receiver.location().end_offset()
    };
    context.replace_many(
        message,
        &dot,
        vec![(removal, String::new()), (insertion..insertion, operator)],
    );
}

fn empty_lines_after_module_inclusion(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.receiver().is_some()
        || node
            .arguments()
            .is_none_or(|arguments| arguments.arguments().is_empty())
        || !matches!(node.name().as_slice(), b"include" | b"extend" | b"prepend")
    {
        return;
    }
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_statements_node().is_some() {
            break;
        }
        if ancestor.as_call_node().is_some()
            || ancestor.as_block_node().is_some()
            || ancestor.as_array_node().is_some()
            || ancestor.as_if_node().is_some()
            || ancestor.as_unless_node().is_some()
        {
            return;
        }
    }

    let source = context.source();
    let current_line = line_index(source, node.location().end_offset());
    if line(source, current_line + 1).is_empty()
        || is_enable_directive(line(source, current_line + 1))
            && line(source, current_line + 2).is_empty()
    {
        return;
    }
    let Some(next) = next_code_line(source, current_line + 1) else {
        return;
    };
    let follower = line(source, next).trim_start();
    let follower_call = call_name(follower);
    if matches!(follower, "end" | "else" | "ensure" | "rescue")
        || ["include", "extend", "prepend"].iter().any(|method| {
            follower_call == *method
                || follower_call.ends_with(&format!(".{method}"))
                || follower_call.ends_with(&format!("&.{method}"))
        })
    {
        return;
    }

    let mut insertion_line = current_line + 1;
    if is_enable_directive(line(source, insertion_line)) {
        insertion_line += 1;
    }
    context.insert(
        "Add an empty line after module inclusion.",
        node.location(),
        line_start(source, insertion_line),
        "\n",
    );
}

fn empty_lines_around_access_modifier(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.receiver().is_some()
        || node.arguments().is_some()
        || node.block().is_some()
        || !matches!(
            node.name().as_slice(),
            b"public" | b"protected" | b"private" | b"module_function"
        )
    {
        return;
    }
    if context.ancestors().iter().rev().find(|ancestor| {
        ancestor.as_def_node().is_some()
            || ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
            || ancestor.as_singleton_class_node().is_some()
            || ancestor.as_block_node().is_some()
    }).is_some_and(|ancestor| ancestor.as_def_node().is_some())
    {
        return;
    }
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_def_node().is_some()
            || ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
            || ancestor.as_singleton_class_node().is_some()
            || ancestor.as_block_node().is_some()
        {
            break;
        }
        if ancestor.as_call_node().is_some() {
            return;
        }
    }
    let source = context.source();
    let location = node.location();
    let current_line = line_index(source, location.start_offset());
    if previous_non_comment_line(source, current_line)
        .is_some_and(|previous| {
            line(source, previous).trim_end().ends_with(',')
                || line(source, previous).len()
                    - line(source, previous).trim_start().len()
                    > line(source, current_line).len()
                        - line(source, current_line).trim_start().len()
        })
    {
        return;
    }
    if !source[location.end_offset()..line_end(source, current_line)]
        .split('#')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_start_matches(';')
        .trim()
        .is_empty()
    {
        return;
    }

    let bounds = enclosing_body_bounds(context);
    let before_ok = bounds.is_some_and(|(opening, _, _)| current_line == opening + 1)
        || previous_non_comment_line(source, current_line)
            .is_none_or(|previous| line(source, previous).trim().is_empty());
    let mut after_ok = bounds
        .is_some_and(|(_, closing, is_block)| !is_block && current_line + 1 == closing)
        || line(source, current_line + 1).trim().is_empty();
    if after_ok
        && bounds.is_some_and(|(_, closing, _)| current_line + 1 == closing)
        && source
            .lines()
            .skip(current_line + 2)
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| line.trim_start().starts_with("# == Schema Information"))
    {
        after_ok = false;
    }
    let style = context.policy().enforced_style("around").to_string();
    if style == "around" && before_ok && after_ok {
        return;
    }
    if style == "only_before" {
        let special_modifier = matches!(node.name().as_slice(), b"private" | b"protected");
        let next_line_exists = current_line + 1 < source.lines().count();
        if special_modifier {
            if line(source, current_line + 1).trim() == "end"
                || before_ok && (!after_ok || !next_line_exists)
            {
                return;
            }
        } else if before_ok {
            return;
        }
    }

    let modifier = String::from_utf8_lossy(node.name().as_slice());
    let message = if style == "around" {
        if bounds.is_some_and(|(opening, _, _)| current_line == opening + 1) {
            format!("Keep a blank line after `{modifier}`.")
        } else {
            format!("Keep a blank line before and after `{modifier}`.")
        }
    } else if after_ok {
        format!("Remove a blank line after `{modifier}`.")
    } else {
        format!("Keep a blank line before `{modifier}`.")
    };

    let mut edits = Vec::new();
    let denied_block_end = bounds.is_some_and(|(_, closing, is_block)| {
        is_block
            && current_line + 1 == closing
            && context.related_config_value("Layout/EmptyLinesAroundBlockBody", "EnforcedStyle")
                == Some("no_empty_lines")
    });
    if !before_ok {
        let start = line_start(source, current_line);
        edits.push((start..start, "\n".to_string()));
    }
    if style == "around" && !after_ok && !denied_block_end {
        let start = line_start(source, current_line + 1);
        edits.push((start..start, "\n".to_string()));
    } else if style == "only_before"
        && after_ok
        && bounds.is_none_or(|(_, closing, is_block)| is_block || current_line + 1 != closing)
    {
        edits.push((
            line_start(source, current_line + 1)..line_start(source, current_line + 2),
            String::new(),
        ));
    }
    if edits.is_empty() {
        context.report(message, &location);
    } else {
        context.replace_many(message, &location, edits);
    }
}

fn block_alignment(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let opening = node.opening_loc();
    let opening_line = line_index(context.source(), opening.start_offset());
    let block_column = context
        .source_file()
        .indentation(opening.start_offset())
        .len();
    let block_start = line_start(context.source(), opening_line) + block_column;
    block_alignment_locations(
        node.closing_loc(),
        opening,
        block_start,
        block_column,
        false,
        context,
    );
}

fn block_alignment_locations(
    closing: ruby_prism::Location<'_>,
    opening: ruby_prism::Location<'_>,
    block_start: usize,
    block_column: usize,
    prefer_block: bool,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    let closing_line = line_index(context.source(), closing.start_offset());
    let closing_start = line_start(context.source(), closing_line);
    if !context.source()[closing_start..closing.start_offset()]
        .chars()
        .all(char::is_whitespace)
    {
        return;
    }

    let ancestors = context.ancestors();
    let Some(call_index) = ancestors
        .iter()
        .rposition(|ancestor| ancestor.as_call_node().is_some())
    else {
        return;
    };
    let mut target = ancestors[call_index].location();
    for parent in ancestors[..call_index].iter().rev() {
        if parent.as_arguments_node().is_some() {
            continue;
        }
        if parent.as_statements_node().is_some() || parent.as_arguments_node().is_some() {
            continue;
        }
        let parent_line = line_index(context.source(), parent.location().start_offset());
        let target_line = line_index(context.source(), target.start_offset());
        let mass_assignment = parent.as_multi_write_node().is_some();
        if parent_line != target_line && !mass_assignment {
            break;
        }
        let absorbs_receiver = parent.as_call_node().is_some_and(|call| {
            call.name().as_slice() == b"<<"
                || call.receiver().is_some_and(|receiver| {
                    call.name().as_slice() != b"[]"
                        && receiver.location().start_offset() == target.start_offset()
                        && receiver.location().end_offset() == target.end_offset()
                })
        });
        let prefix = context
            .source()
            .get(parent.location().start_offset()..target.start_offset())
            .unwrap_or_default();
        let absorbs_expression = parent.as_def_node().is_some()
            || parent.as_splat_node().is_some()
            || mass_assignment
            || parent.as_and_node().is_some()
            || parent.as_or_node().is_some()
            || prefix.contains('=');
        if absorbs_receiver || absorbs_expression {
            target = parent.location();
        } else {
            break;
        }
    }
    let start_line = line_index(context.source(), target.start_offset());
    let start_offset = target.start_offset();
    let start_column = file.column(start_offset);
    let opening_line = line_index(context.source(), opening.start_offset());
    let current_column = file.column(closing.start_offset());
    let style = context
        .config_value("EnforcedStyleAlignWith")
        .unwrap_or("either");
    let aligned = match style {
        "start_of_block" => current_column == block_column,
        "start_of_line" => current_column == start_column,
        _ => current_column == start_column || current_column == block_column,
    };
    if aligned {
        return;
    }

    let current = format!(
        "`{}` at {}, {current_column}",
        String::from_utf8_lossy(closing.as_slice()),
        closing_line + 1
    );
    let start = assignment_lhs_target(
        context.source(),
        start_line,
        start_column,
        start_offset,
        target_is_mass_assignment(context.source(), target),
    )
    .unwrap_or_else(|| {
        source_line_column(context.source(), start_line, start_column, start_offset)
    });
    let block = source_line_column(
        context.source(),
        opening_line,
        block_column,
        block_start,
    );
    let preferred = if style == "start_of_block" || style == "either" && prefer_block {
        &block
    } else {
        &start
    };
    let alternate = if style == "either" && (start_line != opening_line || start_column != block_column)
    {
        if prefer_block {
            format!(" or {start}")
        } else {
            format!(" or {block}")
        }
    } else {
        String::new()
    };
    let target = if style == "start_of_block" {
        block_column
    } else {
        line(context.source(), start_line).len()
            - line(context.source(), start_line).trim_start().len()
    };
    context.replace(
        format!("{current} is not aligned with {preferred}{alternate}."),
        &closing,
        closing_start..closing.start_offset(),
        " ".repeat(target),
    );
}

fn source_line_column(source: &str, line_number: usize, column: usize, start: usize) -> String {
    let content = &source[start.min(line_end(source, line_number))..line_end(source, line_number)];
    format!("`{}` at {}, {column}", content.trim_end(), line_number + 1)
}

fn target_is_mass_assignment(source: &str, target: ruby_prism::Location<'_>) -> bool {
    source[target.start_offset()..line_end(source, line_index(source, target.start_offset()))]
        .split_once(" = ")
        .is_some_and(|(left, _)| left.contains(','))
}

fn assignment_lhs_target(
    source: &str,
    line_number: usize,
    column: usize,
    start: usize,
    mass_assignment: bool,
) -> Option<String> {
    let content = &source[start..line_end(source, line_number)];
    let mut operators = vec![" += ", " -= ", " *= ", " /= ", " %= "];
    if mass_assignment {
        operators.push(" = ");
    }
    let operator = operators
        .iter()
        .filter_map(|operator| content.find(operator))
        .min()?;
    Some(format!(
        "`{}` at {}, {column}",
        content[..operator].trim_end(),
        line_number + 1
    ))
}

fn empty_line_between_defs(
    statements: &ruby_prism::StatementsNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let children = statements.body().iter().collect::<Vec<_>>();
    for pair in children.windows(2) {
        let (Some(previous), Some(current)) = (
            definition_candidate(&pair[0], context),
            definition_candidate(&pair[1], context),
        ) else {
            continue;
        };
        check_definition_pair(context, previous, current);
    }
}

struct DefinitionCandidate<'pr> {
    location: ruby_prism::Location<'pr>,
    kind: &'static str,
    offense: std::ops::Range<usize>,
}

fn definition_candidate<'pr>(
    node: &Node<'pr>,
    context: &CopContext<'_, '_>,
) -> Option<DefinitionCandidate<'pr>> {
    if let Some(definition) = node.as_def_node() {
        if !context.config_bool("EmptyLineBetweenMethodDefs", true) {
            return None;
        }
        return Some(DefinitionCandidate {
            location: node.location(),
            kind: "method",
            offense: definition.def_keyword_loc().start_offset()
                ..definition.name_loc().end_offset(),
        });
    }
    if let Some(class) = node.as_class_node() {
        if !context.config_bool("EmptyLineBetweenClassDefs", true) {
            return None;
        }
        return Some(DefinitionCandidate {
            location: node.location(),
            kind: "class",
            offense: class.class_keyword_loc().start_offset()
                ..class.constant_path().location().end_offset(),
        });
    }
    if let Some(module) = node.as_module_node() {
        if !context.config_bool("EmptyLineBetweenModuleDefs", true) {
            return None;
        }
        return Some(DefinitionCandidate {
            location: node.location(),
            kind: "module",
            offense: module.module_keyword_loc().start_offset()
                ..module.constant_path().location().end_offset(),
        });
    }
    let call = node.as_call_node()?;
    if call.receiver().is_some()
        || !context
            .config_values("DefLikeMacros")
            .iter()
            .any(|name| name.as_bytes() == call.name().as_slice())
    {
        return None;
    }
    Some(DefinitionCandidate {
        location: node.location(),
        kind: if call.block().is_some() {
            "block"
        } else {
            "send"
        },
        offense: node.location().start_offset()..node.location().end_offset(),
    })
}

fn check_definition_pair(
    context: &mut CopContext<'_, '_>,
    previous: DefinitionCandidate<'_>,
    current: DefinitionCandidate<'_>,
) {
    let source = context.source();
    let previous_line = line_index(source, previous.location.end_offset());
    let current_line = line_index(source, current.location.start_offset());
    let between = (previous_line + 1..current_line)
        .map(|number| line(source, number))
        .collect::<Vec<_>>();
    let count = between.iter().filter(|line| line.trim().is_empty()).count();
    let values = context.config_values("NumberOfEmptyLines");
    let (minimum, maximum) = if values.is_empty() {
        let value = context.config_usize("NumberOfEmptyLines", 1);
        (value, value)
    } else {
        (
            values
                .first()
                .and_then(|value| value.parse().ok())
                .unwrap_or(1),
            values
                .last()
                .and_then(|value| value.parse().ok())
                .unwrap_or(1),
        )
    };
    if (minimum..=maximum).contains(&count) {
        return;
    }
    let last_blank = between.iter().rposition(|line| line.trim().is_empty());
    let first_nonblank = between.iter().position(|line| !line.trim().is_empty());
    if last_blank
        .zip(first_nonblank)
        .is_some_and(|(blank, code)| blank > code)
    {
        return;
    }
    if context.config_bool("AllowAdjacentOneLineDefs", true)
        && previous_line == line_index(source, previous.location.start_offset())
        && current_line == line_index(source, current.location.end_offset())
    {
        return;
    }

    let expected = if minimum == maximum {
        format!(
            "{maximum} empty {}",
            if maximum == 1 { "line" } else { "lines" }
        )
    } else {
        format!("{minimum}..{maximum} empty lines")
    };
    let message = format!(
        "Expected {expected} between {} definitions; found {count}.",
        current.kind
    );
    if previous_line == current_line {
        let insertion = current.location.start_offset().saturating_sub(1) + 1;
        context.insert(message, current.offense, insertion, "\n\n");
        return;
    }
    let newline = line_end(source, previous_line);
    if count > maximum {
        context.remove(
            message,
            current.offense,
            newline..newline + (count - maximum),
        );
    } else {
        context.insert(
            message,
            current.offense,
            (newline + 1).min(source.len()),
            "\n".repeat(minimum - count),
        );
    }
}

fn first_argument_indentation(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (first, call_start, special_eligible, call_node) = if let Some(call) = node.as_call_node() {
        let Some(first) = call
            .arguments()
            .and_then(|arguments| arguments.arguments().first())
        else {
            return;
        };
        let name = call.name().as_slice();
        if name == b"[]"
            || name == b"=~"
            || name.ends_with(b"=")
            || call.call_operator_loc().is_none() && is_operator_name(name)
        {
            return;
        }
        (first, call.location().start_offset(), true, Some(call))
    } else if let Some(call) = node.as_super_node() {
        let Some(first) = call
            .arguments()
            .and_then(|arguments| arguments.arguments().first())
        else {
            return;
        };
        (first, call.keyword_loc().start_offset(), false, None)
    } else {
        return;
    };

    let source = context.source();
    let argument_start = first.location().start_offset();
    if line_index(source, call_start) == line_index(source, argument_start) {
        return;
    }
    let argument_line = line_index(source, argument_start);
    if !source[line_start(source, argument_line)..argument_start]
        .chars()
        .all(char::is_whitespace)
    {
        return;
    }
    if context.related_config_value("Layout/ArgumentAlignment", "EnforcedStyle")
        == Some("with_fixed_indentation")
        && context.related_config_value("Layout/FirstMethodArgumentLineBreak", "Enabled")
            != Some("true")
    {
        return;
    }

    let style = context
        .policy()
        .enforced_style("special_for_inner_method_call_in_parentheses")
        .to_string();
    let semantic_parent = context
        .ancestors()
        .iter()
        .rev()
        .find(|ancestor| ancestor.as_arguments_node().is_none());
    let inside_interpolation = context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_interpolated_string_node().is_some())
        || source[..call_start]
            .rfind("#{")
            .is_some_and(|opening| !source[opening + 2..call_start].contains('}'));
    let inside_heredoc_interpolation = context.ancestors().iter().any(|ancestor| {
        ancestor.as_interpolated_string_node().is_some()
            && context.source_file().node(ancestor).trim_start().starts_with("<<")
    });
    if inside_heredoc_interpolation {
        return;
    }
    let outer = (!inside_interpolation)
        .then(|| semantic_parent.and_then(|parent| parent.as_call_node()))
        .flatten();
    let special_indentation = style == "consistent_relative_to_receiver"
        || special_eligible
            && style != "consistent"
            && outer.as_ref().is_some_and(|parent| {
                let permitted = style != "special_for_inner_method_call_in_parentheses"
                    || parent.opening_loc().is_some();
                permitted
                    && parent.name().as_slice() != b"[]="
                    && call_start > parent.location().start_offset()
            });
    let width = context
        .config_value("IndentationWidth")
        .and_then(|value| value.parse().ok())
        .or_else(|| {
            context
                .related_config_value("Layout/IndentationWidth", "Width")
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(2);
    let previous_line = previous_code_line(source, argument_line);
    let base_start = semantic_parent
        .filter(|parent| parent.as_splat_node().is_some() || parent.as_assoc_splat_node().is_some())
        .map_or(call_start, |parent| parent.location().start_offset());
    let base_source = source[base_start..argument_start].trim();
    let base = if inside_interpolation {
        let call_line = line_index(source, call_start);
        line(source, call_line).len() - line(source, call_line).trim_start().len()
    } else if special_indentation {
        if base_source.contains('\n') {
            previous_line
                .map(|number| line(source, number).len() - line(source, number).trim_start().len())
                .unwrap_or(0)
        } else {
            display_column(source, base_start)
        }
    } else {
        previous_line
            .map(|number| line(source, number).len() - line(source, number).trim_start().len())
            .unwrap_or(0)
    };
    let expected = base + width;
    let actual = context.source_file().column(argument_start);
    if actual == expected {
        return;
    }
    let correction_overlaps_outer = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_call_node)
        .and_then(|parent| {
            parent
                .arguments()
                .and_then(|arguments| arguments.arguments().first())
                .map(|outer_first| (parent, outer_first))
        })
        .is_some_and(|(parent, outer_first)| {
            let outer_location = outer_first.location();
            let contains = outer_location.start_offset() <= first.location().start_offset()
                && first.location().end_offset() <= outer_location.end_offset()
                && (outer_location.start_offset() != first.location().start_offset()
                    || outer_location.end_offset() != first.location().end_offset());
            if !contains {
                return false;
            }
            let outer_line = line_index(source, outer_location.start_offset());
            if line_index(source, parent.location().start_offset()) == outer_line
                || !source[line_start(source, outer_line)..outer_location.start_offset()]
                    .chars()
                    .all(char::is_whitespace)
            {
                return false;
            }
            let outer_base = if style == "consistent_relative_to_receiver" {
                context
                    .source_file()
                    .column(parent.location().start_offset())
            } else {
                previous_code_line(source, outer_line)
                    .map(|number| {
                        line(source, number).len() - line(source, number).trim_start().len()
                    })
                    .unwrap_or(0)
            };
            context.source_file().column(outer_location.start_offset()) != outer_base + width
        });
    if correction_overlaps_outer {
        if !context.autocorrect_enabled() {
            context.report("Bad indentation of the first argument.", first.location());
        }
        return;
    }

    let base_description = if special_indentation && !base_source.contains('\n') {
        format!("`{base_source}`")
    } else if base_source
        .lines()
        .next_back()
        .is_some_and(|line| line.trim_start().starts_with('#'))
    {
        "the start of the previous line (not counting the comment)".to_string()
    } else {
        "the start of the previous line".to_string()
    };
    let message = format!("Indent the first argument one step more than {base_description}.");
    let delta = expected as isize - actual as isize;
    let first_location = first.location();
    let first_line = line_index(source, first_location.start_offset());
    let mut correction_end = first_location.end_offset();
    let inside_parenthesized_argument = call_node.as_ref().is_some_and(|call| {
        context
            .ancestors()
            .iter()
            .filter_map(Node::as_call_node)
            .any(|parent| {
                parent.opening_loc().is_some()
                    && parent.arguments().is_some_and(|arguments| {
                        arguments.location().start_offset() <= call.location().start_offset()
                            && call.location().end_offset() <= arguments.location().end_offset()
                    })
            })
    });
    if style == "special_for_inner_method_call_in_parentheses" && inside_parenthesized_argument {
        if let Some(call) = call_node.as_ref() {
            correction_end = call.location().end_offset();
            for ancestor in context.ancestors().iter().rev() {
                if ancestor.as_call_node().is_some_and(|ancestor| {
                    ancestor.location().start_offset() == call.location().start_offset()
                }) {
                    correction_end = ancestor.location().end_offset();
                }
            }
        }
    }
    let last_line = line_index(source, correction_end);
    let mut previous = None::<(usize, bool)>;
    let edits = (first_line..=last_line)
        .filter_map(|number| {
            let start = line_start(source, number);
            let content = line(source, number);
            if content.trim().is_empty() {
                return None;
            }
            let indentation = content.len() - content.trim_start().len();
            let preserve_nested = delta > 0
                && previous.is_some_and(|(previous_indent, opened)| {
                    opened && indentation == previous_indent + width * 2
                });
            let adjusted = if preserve_nested {
                indentation
            } else {
                (indentation as isize + delta).max(0) as usize
            };
            previous = Some((indentation, content.trim_end().ends_with('(')));
            Some((start..start + indentation, " ".repeat(adjusted)))
        })
        .collect();
    context.replace_many(message, &first_location, edits);
}

fn display_column(source: &str, offset: usize) -> usize {
    source[line_start(source, line_index(source, offset))..offset]
        .chars()
        .map(|character| character.width().unwrap_or(0))
        .sum()
}

fn is_operator_name(name: &[u8]) -> bool {
    matches!(
        name,
        b"+" | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
            | b"=="
            | b"!="
            | b"<"
            | b">"
            | b"<="
            | b">="
            | b"<=>"
            | b"&"
            | b"|"
            | b"^"
            | b"<<"
            | b">>"
    )
}

fn enclosing_body_bounds(context: &CopContext<'_, '_>) -> Option<(usize, usize, bool)> {
    context.ancestors().iter().rev().find_map(|ancestor| {
        if let Some(class) = ancestor.as_class_node() {
            let opening = class.superclass().map_or_else(
                || line_index(context.source(), class.class_keyword_loc().start_offset()),
                |superclass| line_index(context.source(), superclass.location().end_offset()),
            );
            return Some((
                opening,
                line_index(context.source(), class.end_keyword_loc().start_offset()),
                false,
            ));
        }
        if let Some(module) = ancestor.as_module_node() {
            return Some((
                line_index(context.source(), module.module_keyword_loc().start_offset()),
                line_index(context.source(), module.end_keyword_loc().start_offset()),
                false,
            ));
        }
        if let Some(class) = ancestor.as_singleton_class_node() {
            return Some((
                line_index(context.source(), class.expression().location().end_offset()),
                line_index(context.source(), class.end_keyword_loc().start_offset()),
                false,
            ));
        }
        ancestor.as_block_node().map(|block| {
            (
                line_index(context.source(), block.opening_loc().start_offset()),
                line_index(context.source(), block.closing_loc().start_offset()),
                true,
            )
        })
    })
}

fn previous_non_comment_line(source: &str, line_number: usize) -> Option<usize> {
    (0..line_number)
        .rev()
        .find(|number| !line(source, *number).trim_start().starts_with('#'))
}

fn previous_code_line(source: &str, line_number: usize) -> Option<usize> {
    (0..line_number).rev().find(|number| {
        let candidate = line(source, *number).trim();
        !candidate.is_empty() && !candidate.starts_with('#')
    })
}

fn call_name(source: &str) -> &str {
    source
        .split(|character: char| character.is_whitespace() || character == '(')
        .next()
        .unwrap_or_default()
}

fn is_enable_directive(source: &str) -> bool {
    let source = source.trim();
    source.starts_with("# rubocop:enable") || source.starts_with("# rubocop:todo")
}

fn next_code_line(source: &str, mut line_number: usize) -> Option<usize> {
    while line_start(source, line_number) < source.len() {
        let candidate = line(source, line_number).trim_start();
        if !candidate.is_empty() && !candidate.starts_with('#') {
            return Some(line_number);
        }
        line_number += 1;
    }
    None
}

fn line_index(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
}

fn line_start(source: &str, line_number: usize) -> usize {
    if line_number == 0 {
        return 0;
    }
    source
        .match_indices('\n')
        .nth(line_number - 1)
        .map_or(source.len(), |(offset, _)| offset + 1)
}

fn line_end(source: &str, line_number: usize) -> usize {
    let start = line_start(source, line_number);
    source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset)
}

fn line(source: &str, line_number: usize) -> &str {
    let start = line_start(source, line_number);
    let end = line_end(source, line_number);
    source[start..end]
        .strip_suffix('\r')
        .unwrap_or(&source[start..end])
}
