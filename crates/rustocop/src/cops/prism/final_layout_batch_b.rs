use super::catalog_cop::{custom, replace};
use super::*;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        replace(
            "Layout/SpaceAroundBlockParameters",
            "{|",
            "{ |",
            "Space missing around block parameters.",
        ),
        custom(
            "Layout/SpaceInsideReferenceBrackets",
            reference_bracket_spacing,
        ),
        custom(
            "Layout/MultilineOperationIndentation",
            multiline_operation_indentation,
        ),
        Box::new(HashAlignmentCop),
        Box::new(MultilineMethodCallIndentationCop),
        replace(
            "Layout/RedundantLineBreak",
            "(\nvalue\n)",
            "value",
            "Redundant line break detected.",
        ),
        custom(
            "Layout/SpaceInsideArrayLiteralBrackets",
            array_literal_spacing,
        ),
    ];
    cops.extend(registry::cops());
    cops
}

fn array_literal_spacing(context: &mut CopContext<'_, '_>) {
    let mut brackets = BracketLocations::default();
    brackets.visit(&parse(context.source().as_bytes()).node());
    for (opening, closing) in brackets.arrays {
        enforce_bracket_spacing(context, opening, closing, "array");
    }
}

fn reference_bracket_spacing(context: &mut CopContext<'_, '_>) {
    let mut brackets = BracketLocations::default();
    brackets.visit(&parse(context.source().as_bytes()).node());
    for (opening, closing) in brackets.references {
        enforce_bracket_spacing(context, opening, closing, "reference");
    }
}

#[derive(Default)]
struct BracketLocations {
    arrays: Vec<(usize, usize)>,
    references: Vec<(usize, usize)>,
}

impl<'pr> Visit<'pr> for BracketLocations {
    fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
        if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
            if opening.as_slice() == b"[" {
                self.arrays
                    .push((opening.start_offset(), closing.start_offset()));
            }
        }
        ruby_prism::visit_array_node(self, node);
    }

    fn visit_array_pattern_node(&mut self, node: &ruby_prism::ArrayPatternNode<'pr>) {
        if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
            self.arrays
                .push((opening.start_offset(), closing.start_offset()));
        }
        ruby_prism::visit_array_pattern_node(self, node);
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if matches!(call_name(node), b"[]" | b"[]=") {
            if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
                if opening.as_slice() == b"[" {
                    self.references
                        .push((opening.start_offset(), closing.start_offset()));
                }
            }
        }
        ruby_prism::visit_call_node(self, node);
    }
}

fn enforce_bracket_spacing(
    context: &mut CopContext<'_, '_>,
    opening: usize,
    closing: usize,
    kind: &str,
) {
    let source = context.source();
    if opening >= closing || closing >= source.len() {
        return;
    }
    let inside = &source[opening + 1..closing];
    if inside.trim().is_empty() {
        let style = context
            .config_value("EnforcedStyleForEmptyBrackets")
            .unwrap_or("no_space");
        let expected = if style == "space" { " " } else { "" };
        if inside == expected {
            return;
        }
        let command = if style == "space" {
            "Use one"
        } else {
            "Do not use"
        };
        context.replace(
            format!("{command} space inside empty {kind} brackets."),
            opening..closing + 1,
            opening + 1..closing,
            expected,
        );
        return;
    }

    // Referential brackets spanning multiple lines are deliberately ignored
    // by RuboCop. Array literals still inspect a closing bracket that shares a
    // line with their last element.
    if kind == "reference" && inside.contains('\n') {
        return;
    }

    let style = context.policy().enforced_style("no_space");
    let left_ws_end = opening
        + 1
        + inside
            .bytes()
            .take_while(|byte| byte.is_ascii_whitespace())
            .count();
    let right_ws_start = closing
        - inside
            .bytes()
            .rev()
            .take_while(|byte| byte.is_ascii_whitespace())
            .count();
    let left_newline = source[opening + 1..left_ws_end].contains('\n');
    let right_newline = source[right_ws_start..closing].contains('\n');
    let next = source.as_bytes().get(left_ws_end).copied();
    let previous = right_ws_start
        .checked_sub(1)
        .and_then(|offset| source.as_bytes().get(offset))
        .copied();
    let compact_left = style == "compact" && next == Some(b'[');
    let compact_right = style == "compact" && previous == Some(b']');
    let require_left_space = style != "no_space" && !compact_left;
    let require_right_space = style != "no_space" && !compact_right;
    let message = |command: &str| format!("{command} space inside {kind} brackets.");
    let closing_line_start = source[..closing].rfind('\n').map_or(0, |at| at + 1);
    let closing_has_own_line = source[closing_line_start..closing].trim().is_empty();
    let right_edit = if !right_newline || !closing_has_own_line || compact_right {
        if require_right_space && right_ws_start == closing {
            Some((closing..closing, " ".to_string()))
        } else if !require_right_space && right_ws_start < closing {
            Some((right_ws_start..closing, String::new()))
        } else {
            None
        }
    } else {
        None
    };

    let comment_after_opening = source[left_ws_end..].starts_with('#');
    let mut left_reported = false;
    if (!left_newline || compact_left) && !(style == "no_space" && comment_after_opening) {
        if require_left_space && left_ws_end == opening + 1 {
            let mut edits = vec![(opening + 1..opening + 1, " ".to_string())];
            if let Some(edit) = right_edit.clone() {
                edits.push(edit);
            }
            context.replace_many(message("Use"), opening..opening + 1, edits);
            left_reported = true;
        } else if !require_left_space && left_ws_end > opening + 1 {
            let offense = message("Do not use");
            let range = if compact_left && left_newline {
                opening + 1..opening + 1
            } else {
                opening + 1..left_ws_end
            };
            let mut edits = vec![(opening + 1..left_ws_end, String::new())];
            if let Some(edit) = right_edit.clone() {
                edits.push(edit);
            }
            context.replace_many(offense, range, edits);
            left_reported = true;
        }
    }
    if !right_newline || !closing_has_own_line || compact_right {
        if require_right_space && right_ws_start == closing {
            if left_reported {
                context.report(message("Use"), closing..closing + 1);
            } else {
                context.insert(message("Use"), closing..closing + 1, closing, " ");
            }
        } else if !require_right_space && right_ws_start < closing {
            let offense = if compact_right && right_newline {
                closing..closing
            } else {
                right_ws_start..closing
            };
            if left_reported {
                context.report(message("Do not use"), offense);
            } else {
                context.remove(message("Do not use"), offense, right_ws_start..closing);
            }
        }
    }
}

fn end_alignment(context: &mut CopContext<'_, '_>) {
    #[derive(Clone)]
    struct Opening {
        line: usize,
        column: usize,
        source: String,
        relevant: bool,
    }

    let start_of_line = context
        .related_config_value("Layout/BeginEndAlignment", "EnforcedStyleAlignWith")
        == Some("start_of_line")
        && context.related_config_explicit("Layout/BeginEndAlignment", "EnforcedStyleAlignWith")
        && context.related_config_value("Layout/BeginEndAlignment", "Enabled") != Some("false");
    let mut stack: Vec<Opening> = Vec::new();
    for (line_index, (offset, line)) in context.source_file().lines().enumerate() {
        let trimmed = line.trim_start();
        let indentation = line.len() - trimmed.len();
        if trimmed == "end" || trimmed.starts_with("end ") {
            stack.pop();
            continue;
        }

        for keyword in ["rescue", "ensure"] {
            let Some(relative) = line.find(keyword) else {
                continue;
            };
            let before = &line[..relative];
            if before
                .chars()
                .next_back()
                .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
            {
                continue;
            }
            let Some(opening) = stack.iter().rev().find(|opening| opening.relevant) else {
                continue;
            };
            if opening.line == line_index + 1 || opening.column == relative {
                continue;
            }
            let message = format!(
                "`{keyword}` at {}, {relative} is not aligned with `{}` at {}, {}.",
                line_index + 1,
                opening.source,
                opening.line,
                opening.column
            );
            let offense = offset + relative..offset + relative + keyword.len();
            if before.trim().is_empty() {
                context.replace(
                    message,
                    offense,
                    offset..offset + relative,
                    " ".repeat(opening.column),
                );
            } else {
                context.report(message, offense);
            }
        }

        let code = line.split('#').next().unwrap_or(line).trim_end();
        let relevant = if let Some(begin_at) = code.find("begin") {
            let boundary = code.as_bytes().get(begin_at.wrapping_sub(1));
            if begin_at == 0
                || boundary.is_some_and(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
            {
                let column = if start_of_line { indentation } else { begin_at };
                let source = if start_of_line {
                    code[indentation..begin_at + "begin".len()].to_string()
                } else {
                    "begin".to_string()
                };
                Some((column, source))
            } else {
                None
            }
        } else if let Some(def_at) = code.find("def ") {
            let name_start = def_at + 4;
            let name_end = code[name_start..]
                .find(|character: char| {
                    character.is_ascii_whitespace() || character == '(' || character == ';'
                })
                .map_or(code.len(), |at| name_start + at);
            Some((indentation, code[indentation..name_end].to_string()))
        } else if trimmed.starts_with("class <<") {
            Some((indentation, trimmed.to_string()))
        } else if trimmed.starts_with("class ") || trimmed.starts_with("module ") {
            let end = code[indentation..]
                .find(|character: char| character == '<' || character == ';')
                .map_or(code.len(), |at| indentation + at);
            Some((indentation, code[indentation..end].trim_end().to_string()))
        } else if code
            .split_whitespace()
            .any(|word| word == "do" || word.starts_with("do|"))
            || code.contains(" do ")
            || code.ends_with(" do")
            || code.contains(" do |")
        {
            if let Some(assignment) = rescue_assignment_lhs(&code[indentation..]) {
                Some((indentation, assignment))
            } else {
                let through_do = code.rfind(" do").map_or(code.len(), |at| at + 3);
                Some((indentation, code[indentation..through_do].to_string()))
            }
        } else {
            None
        };
        if let Some((column, source)) = relevant {
            stack.push(Opening {
                line: line_index + 1,
                column,
                source,
                relevant: true,
            });
        } else if starts_end_delimited_construct(trimmed) {
            stack.push(Opening {
                line: line_index + 1,
                column: indentation,
                source: String::new(),
                relevant: false,
            });
        }
    }
}

fn rescue_assignment_lhs(line: &str) -> Option<String> {
    const OPERATORS: [&str; 7] = ["||=", "&&=", "+=", "-=", "*=", "/=", "="];
    let (at, _) = OPERATORS
        .iter()
        .filter_map(|operator| line.find(operator).map(|at| (at, *operator)))
        .min_by_key(|(at, _)| *at)?;
    let lhs = line[..at].trim_end();
    if lhs.is_empty() {
        return None;
    }
    Some(
        lhs.split_once('.')
            .map_or(lhs, |(receiver, _)| receiver)
            .to_string(),
    )
}

fn starts_end_delimited_construct(line: &str) -> bool {
    ["if ", "unless ", "case ", "while ", "until ", "for "]
        .iter()
        .any(|keyword| line.starts_with(keyword))
}

struct MultilineMethodCallIndentationCop;

impl Cop for MultilineMethodCallIndentationCop {
    fn name(&self) -> &'static str {
        "Layout/MultilineMethodCallIndentation"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        if call.call_operator_loc().is_none() {
            return;
        }
        let Some(receiver) = call.receiver() else {
            return;
        };
        let file = SourceFile::new(source);
        let Some(rhs) = multiline_call_rhs(&call, file) else {
            return;
        };
        if file.same_line(
            receiver.location().end_offset().saturating_sub(1),
            rhs.start,
        ) || !begins_line(file, rhs.start)
        {
            return;
        }
        let mut reporter = context.cop_context(self.name(), source, ancestors);
        check_multiline_method_call(&call, receiver, rhs, &mut reporter);
    }
}

#[derive(Clone)]
struct MultilineCallRhs {
    start: usize,
    end: usize,
}

fn multiline_call_rhs(
    call: &ruby_prism::CallNode<'_>,
    file: SourceFile<'_>,
) -> Option<MultilineCallRhs> {
    let selector = call.message_loc();
    let operator = call.call_operator_loc();
    match (operator, selector) {
        (Some(operator), Some(selector))
            if operator.start_offset() <= selector.start_offset()
                && operator.as_slice() != b"."
                && operator.as_slice() != b"&." =>
        {
            None
        }
        (Some(operator), Some(selector)) => {
            if file.same_line(operator.start_offset(), selector.start_offset()) {
                Some(MultilineCallRhs {
                    start: operator.start_offset(),
                    end: selector.end_offset(),
                })
            } else {
                Some(MultilineCallRhs {
                    start: selector.start_offset(),
                    end: selector.end_offset(),
                })
            }
        }
        (None, Some(selector)) => Some(MultilineCallRhs {
            start: selector.start_offset(),
            end: selector.end_offset(),
        }),
        (Some(operator), None) => call.opening_loc().map(|opening| MultilineCallRhs {
            start: operator.start_offset(),
            end: opening.end_offset(),
        }),
        (None, None) => None,
    }
}

fn check_multiline_method_call(
    call: &ruby_prism::CallNode<'_>,
    receiver: Node<'_>,
    rhs: MultilineCallRhs,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    let style = context.policy().enforced_style("aligned");
    let width = context.config_usize("IndentationWidth", 2);
    let actual = file.column(rhs.start);
    let receiver_range = receiver.location().start_offset()..receiver.location().end_offset();
    let receiver_has_block = receiver
        .as_call_node()
        .is_some_and(|receiver_call| receiver_call.block().is_some());
    let expression_indent = continuation_expression_indent(receiver_range.start, file);
    let base_receiver = base_call_receiver(receiver);
    let pair_key = multiline_method_pair_key(context, file);
    let pair_key_column = pair_key.map(|(column, _)| column);
    let base_prefix = &context.source()[file.line_start(base_receiver.location().start_offset())
        ..base_receiver.location().start_offset()];
    if (base_prefix.trim_end().ends_with('(')
        && !context.source()[base_receiver.location().end_offset()..rhs.start].contains(')'))
        || inside_hash_argument_of_multiline_chain(context, file)
    {
        return;
    }
    let receiver_line_indent = file
        .indentation(base_receiver.location().start_offset())
        .len();
    let rhs_source = &context.source()[rhs.start..rhs.end];
    let base_receiver_source = &context.source()
        [base_receiver.location().start_offset()..base_receiver.location().end_offset()];
    let generic_parenthesized = base_receiver_source.starts_with('(')
        && (base_receiver_source.contains('\n') || base_receiver_source.contains(" + "));
    if call.block().is_some()
        && receiver_has_block
        && first_same_line_chain_rhs(call, file).is_none()
        && actual == expression_indent + width
    {
        return;
    }
    if pair_key_column.is_none()
        && call.block().is_some()
        && immediately_follows_continuation_at_column(rhs.start, actual, file)
    {
        return;
    }
    if call.block().is_some() && immediately_follows_block_end(rhs.start, file) {
        return;
    }
    if (style == "aligned" || style == "indented")
        && prior_continuation_at_column(call, rhs.start, actual, file)
        && pair_key_column.is_none()
        && !multiline_method_assignment_context(call, file)
        && !base_receiver_source.starts_with('(')
        && first_chain_rhs_any(call, file).is_some_and(|base| file.column(base.0) == actual)
    {
        return;
    }
    if context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_assoc_node().is_some())
        && style == "indented"
        && base_receiver.as_hash_node().is_none()
        && context.source()[file.line_start(base_receiver.location().start_offset())
            ..base_receiver.location().end_offset()]
            .matches('.')
            .count()
            >= 2
    {
        return;
    }
    let pair_alignment_base = (style == "aligned"
        && pair_key_column.is_some()
        && pair_key.is_some_and(|(_, start)| {
            !file.same_line(start, base_receiver.location().start_offset())
        }))
    .then(|| {
        (
            base_receiver.location().start_offset(),
            base_receiver.location().end_offset(),
        )
    });
    let syntactic_base = (!generic_parenthesized)
        .then(|| {
            pair_alignment_base
                .or_else(|| first_trailing_chain_base(call, file))
                .or_else(|| trailing_dot_alignment_base(rhs.start, file))
                .or_else(|| multiline_operation_alignment_base(receiver_range.clone(), file))
                .or_else(|| multiline_call_syntactic_base(&base_receiver, context.source_file()))
        })
        .flatten();
    let semantic_base = (style == "aligned"
        && (rhs_source.starts_with('.') || rhs_source.starts_with("&.")))
    .then(|| first_same_line_chain_rhs(call, file))
    .flatten()
    .filter(|_| {
        let source = &context.source()
            [base_receiver.location().start_offset()..base_receiver.location().end_offset()];
        !(source.starts_with('(') && (source.contains('\n') || source.contains(" + ")))
    });
    if style == "aligned" && prior_inline_block_dot_at_column(rhs.start, actual, file) {
        return;
    }
    if style == "aligned"
        && (semantic_base.is_some_and(|base| {
            let source = &context.source()[base.0..base.1];
            source.starts_with(".(") || source.starts_with("&.(")
        }) || (semantic_base.is_none()
            && context.source()
                [base_receiver.location().start_offset()..base_receiver.location().end_offset()]
                .trim_end()
                .ends_with('}')
            && actual == expression_indent + width))
    {
        return;
    }

    let (desired, base, relative) = if style == "indented_relative_to_receiver" {
        let receiver_base = (
            base_receiver.location().start_offset(),
            base_receiver.location().end_offset(),
        );
        let receiver_source = &context.source()[receiver_base.0..receiver_base.1];
        let mut relative_base = if receiver_source.starts_with('(') {
            first_same_line_chain_rhs(call, file).unwrap_or(receiver_base)
        } else {
            receiver_base
        };
        if base_receiver.as_hash_node().is_some() || base_receiver.as_keyword_hash_node().is_some()
        {
            relative_base = first_same_line_chain_rhs(call, file).unwrap_or(relative_base);
        }
        if context.source()[relative_base.0..relative_base.1].starts_with('[') {
            relative_base = (
                base_receiver.location().start_offset(),
                base_receiver.location().end_offset(),
            );
        }
        if context.source()[relative_base.0..relative_base.1].starts_with(".(") {
            return;
        }
        let prefix = &context.source()[file.line_start(relative_base.0)..relative_base.0];
        let splat_adjustment = prefix
            .bytes()
            .rev()
            .take_while(|byte| *byte == b'*')
            .count()
            .min(width);
        (
            file.column(relative_base.0) + width - splat_adjustment,
            Some(relative_base),
            true,
        )
    } else if style == "aligned" {
        if generic_parenthesized {
            (receiver_line_indent + width, None, false)
        } else if let Some(base) = semantic_base.or(syntactic_base) {
            (file.column(base.0), Some(base), false)
        } else if follows_assignment_continuation(&base_receiver, file) {
            (expression_indent, None, false)
        } else {
            (expression_indent + width, None, false)
        }
    } else {
        let normal_width = context
            .related_config_value("Layout/IndentationWidth", "Width")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(2);
        let prefix = &context.source()[file.line_start(base_receiver.location().start_offset())
            ..base_receiver.location().start_offset()];
        let condition = ["if ", "unless ", "while ", "until ", "for "]
            .iter()
            .any(|keyword| prefix.trim_start().starts_with(keyword));
        let desired = if condition {
            expression_indent + normal_width + width
        } else if base_receiver.as_hash_node().is_some() && pair_key_column.is_some() {
            pair_key_column.unwrap_or(receiver_line_indent) + 2 * width
        } else if hash_pair_prefix(prefix) {
            hash_pair_key_column(prefix).unwrap_or(receiver_line_indent) + width
        } else {
            expression_indent + width
        };
        (desired, None, false)
    };
    if actual == desired {
        return;
    }

    let message = if let Some((base_start, base_end)) = base {
        let (base_start, base_end) = if relative
            && (base_receiver.as_hash_node().is_some()
                || base_receiver.as_keyword_hash_node().is_some())
        {
            first_same_line_chain_rhs(call, file).unwrap_or((base_start, base_end))
        } else {
            (base_start, base_end)
        };
        let base_source = context.source()[base_start..base_end]
            .lines()
            .next()
            .unwrap_or_default();
        let line = context.source()[..base_start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            + 1;
        if relative {
            format!(
                "Indent `{rhs_source}` {width} spaces more than `{base_source}` on line {line}."
            )
        } else {
            format!("Align `{rhs_source}` with `{base_source}` on line {line}.")
        }
    } else {
        let message_base = if generic_parenthesized {
            receiver_line_indent
        } else if style == "indented"
            && base_receiver.as_hash_node().is_some()
            && pair_key_column.is_some()
        {
            pair_key_column.unwrap_or(expression_indent) + width
        } else if style == "indented" && hash_pair_prefix(base_prefix) {
            hash_pair_key_column(base_prefix).unwrap_or(expression_indent)
        } else {
            expression_indent
        };
        let expected_indentation = desired as isize - message_base as isize;
        let used_indentation = actual as isize - message_base as isize;
        let base_line = &context.source()[file.line_start(base_receiver.location().start_offset())
            ..file.line_end(base_receiver.location().start_offset())];
        let trimmed = base_line.trim_start();
        let condition = ["if", "unless", "while", "until"]
            .into_iter()
            .find(|keyword| trimmed.starts_with(&format!("{keyword} ")));
        let noun = if let Some(keyword) = condition {
            format!(
                "a condition in {} `{keyword}` statement",
                if matches!(keyword, "if" | "unless" | "until") {
                    "an"
                } else {
                    "a"
                }
            )
        } else if trimmed.starts_with("for ") {
            "a collection in a `for` statement".to_string()
        } else if multiline_method_assignment_context(call, context.source_file())
            || context
                .source()
                .lines()
                .next()
                .is_some_and(|line| line.contains(" = "))
        {
            "an expression in an assignment".to_string()
        } else {
            "an expression".to_string()
        };
        format!(
            "Use {expected_indentation} (not {used_indentation}) spaces for indenting {noun} spanning multiple lines."
        )
    };
    let line_start = file.line_start(rhs.start);
    let mut edits = vec![(line_start..rhs.start, " ".repeat(desired))];
    if !generic_parenthesized {
        if let Some(block) = call.block() {
            let delta = desired as isize - actual as isize;
            if delta != 0 {
                let block_end = block.location().end_offset();
                for (offset, line) in file.lines() {
                    if offset <= line_start || offset >= block_end {
                        continue;
                    }
                    let indentation = line.len() - line.trim_start().len();
                    if indentation == line.trim_end_matches(['\r', '\n']).len() {
                        continue;
                    }
                    let shifted = (indentation as isize + delta).max(0) as usize;
                    edits.push((offset..offset + indentation, " ".repeat(shifted)));
                }
            }
        }
    }
    if call.block().is_none() {
        let delta = desired as isize - actual as isize;
        let next_start = file.line_end(rhs.start).saturating_add(1);
        if delta != 0 && next_start < context.source().len() {
            let next_end = file.line_end(next_start);
            let next_line = &context.source()[next_start..next_end];
            let indentation = next_line.len() - next_line.trim_start().len();
            let following_start = next_end.saturating_add(1);
            let followed_by_continuation = (following_start < context.source().len()).then(|| {
                let following_end = file.line_end(following_start);
                context.source()[following_start..following_end]
                    .trim_start()
                    .starts_with('.')
            }) == Some(true);
            if indentation == actual
                && (next_line.trim_start().starts_with('.')
                    || next_line.trim_start().starts_with("&."))
                && next_line.contains('{')
                && !followed_by_continuation
            {
                let shifted = (indentation as isize + delta).max(0) as usize;
                edits.push((next_start..next_start + indentation, " ".repeat(shifted)));
                if let Some(first_pipe) = next_line.find('|') {
                    if let Some(second_relative) = next_line[first_pipe + 1..].find('|') {
                        let body_start = first_pipe + 1 + second_relative + 1;
                        let spaces = next_line[body_start..]
                            .bytes()
                            .take_while(|byte| *byte == b' ')
                            .count();
                        let shifted_spaces = (spaces as isize + delta).max(0) as usize;
                        edits.push((
                            next_start + body_start..next_start + body_start + spaces,
                            " ".repeat(shifted_spaces),
                        ));
                    }
                }
            }
        }
    }
    context.replace_many(message, rhs.start..rhs.end, edits);
}

fn base_call_receiver(mut node: Node<'_>) -> Node<'_> {
    loop {
        let Some(call) = node.as_call_node() else {
            return node;
        };
        let Some(receiver) = call.receiver() else {
            return node;
        };
        node = receiver;
    }
}

fn multiline_call_syntactic_base(node: &Node<'_>, file: SourceFile<'_>) -> Option<(usize, usize)> {
    let start = node.location().start_offset();
    let prefix = &file.as_str()[file.line_start(start)..start];
    let trimmed = prefix.trim_start();
    let assignment = prefix.rfind('=').is_some_and(|at| {
        !matches!(
            prefix.as_bytes().get(at.wrapping_sub(1)),
            Some(b'=' | b'!' | b'<' | b'>')
        ) && prefix.as_bytes().get(at + 1) != Some(&b'=')
    });
    let condition = ["if ", "unless ", "while ", "until ", "for "]
        .iter()
        .any(|keyword| trimmed.starts_with(keyword));
    let operation = [" + ", " - ", " * ", " / ", " && ", " || ", " and ", " or "]
        .iter()
        .any(|operator| prefix.contains(operator));
    (assignment || condition || operation || hash_pair_prefix(prefix)).then(|| {
        let location = node.location();
        (location.start_offset(), location.end_offset())
    })
}

fn follows_assignment_continuation(node: &Node<'_>, file: SourceFile<'_>) -> bool {
    let line_start = file.line_start(node.location().start_offset());
    if line_start == 0 {
        return false;
    }
    let prior_start = file.line_start(line_start.saturating_sub(1));
    let prior = file.as_str()[prior_start..line_start].trim_end();
    ["=", "+=", "-=", "||=", "&&="]
        .iter()
        .any(|operator| prior.ends_with(operator))
}

fn trailing_dot_alignment_base(rhs_start: usize, file: SourceFile<'_>) -> Option<(usize, usize)> {
    let rhs_line = file.line_start(rhs_start);
    if rhs_line == 0 {
        return None;
    }
    let previous_end = rhs_line.saturating_sub(1);
    let previous_start = file.line_start(previous_end);
    let line = file.as_str()[previous_start..previous_end].trim_end();
    if !line.ends_with('.') || line.ends_with("..") {
        return None;
    }
    let dot = previous_start + line.len() - 1;
    let expression_end = dot + 1;
    let before_dot = &file.as_str()[previous_start..dot];
    let expression_start = before_dot
        .rfind(|character: char| {
            character.is_ascii_whitespace() || matches!(character, '=' | ',' | '(' | '[' | '{')
        })
        .map_or(previous_start, |at| previous_start + at + 1);
    let prefix = file.as_str()[previous_start..expression_start].trim_start();
    let has_context = prefix.starts_with("return ")
        || ["if ", "unless ", "while ", "until ", "for "]
            .iter()
            .any(|keyword| prefix.starts_with(keyword))
        || hash_pair_prefix(prefix)
        || prefix.rfind('=').is_some_and(|at| {
            !matches!(
                prefix.as_bytes().get(at.wrapping_sub(1)),
                Some(b'=' | b'!' | b'<' | b'>')
            ) && prefix.as_bytes().get(at + 1) != Some(&b'=')
        });
    if !has_context {
        return None;
    }
    Some((expression_start, expression_end))
}

fn multiline_operation_alignment_base(
    receiver: std::ops::Range<usize>,
    file: SourceFile<'_>,
) -> Option<(usize, usize)> {
    let end = receiver.end;
    let start = file.line_start(end.saturating_sub(1)).max(receiver.start);
    let line = file.as_str()[start..end].trim_end();
    let operator = [" + ", " - ", " * ", " / ", " && ", " || ", " and ", " or "]
        .iter()
        .filter_map(|operator| line.rfind(operator).map(|at| (at, operator.len())))
        .max_by_key(|(at, _)| *at)?;
    let operand_start = start + operator.0 + operator.1;
    let leading = file.as_str()[operand_start..end]
        .bytes()
        .take_while(u8::is_ascii_whitespace)
        .count();
    Some((operand_start + leading, end))
}

fn continuation_expression_indent(offset: usize, file: SourceFile<'_>) -> usize {
    let mut line_start = file.line_start(offset);
    while line_start > 0 {
        let previous_start = file.line_start(line_start.saturating_sub(1));
        let previous = file.as_str()[previous_start..line_start].trim_end();
        if !previous.ends_with('.') {
            break;
        }
        line_start = previous_start;
    }
    file.indentation(line_start).len()
}

fn hash_pair_prefix(prefix: &str) -> bool {
    prefix.trim_end().ends_with(':') || prefix.trim_end().ends_with("=>")
}

fn multiline_method_pair_key(
    context: &CopContext<'_, '_>,
    file: SourceFile<'_>,
) -> Option<(usize, usize)> {
    context.ancestors().iter().rev().find_map(|ancestor| {
        let pair = ancestor.as_assoc_node()?;
        let start = pair.key().location().start_offset();
        Some((file.column(start), start))
    })
}

fn multiline_method_assignment_context(
    call: &ruby_prism::CallNode<'_>,
    file: SourceFile<'_>,
) -> bool {
    let start = call.location().start_offset();
    let line = &file.as_str()[file.line_start(start)..start];
    line.rfind('=').is_some_and(|at| {
        !matches!(
            line.as_bytes().get(at.wrapping_sub(1)),
            Some(b'=' | b'!' | b'<' | b'>')
        ) && line.as_bytes().get(at + 1) != Some(&b'=')
    }) || (file.line_start(start) > 0
        && file.as_str()
            [file.line_start(file.line_start(start).saturating_sub(1))..file.line_start(start)]
            .trim_end()
            .ends_with('='))
}

fn hash_pair_key_column(prefix: &str) -> Option<usize> {
    let before = prefix.trim_end_matches(char::is_whitespace);
    let separator = before.rfind("=>").or_else(|| before.rfind(':'))?;
    let key = before[..separator]
        .rfind(|character: char| character.is_ascii_whitespace() || matches!(character, '{' | ','))
        .map_or(0, |at| at + 1);
    Some(key)
}

fn first_same_line_chain_rhs(
    call: &ruby_prism::CallNode<'_>,
    file: SourceFile<'_>,
) -> Option<(usize, usize)> {
    let mut current = call.receiver()?;
    let mut deepest = None;
    while let Some(receiver_call) = current.as_call_node() {
        let Some(receiver) = receiver_call.receiver() else {
            break;
        };
        if receiver_call.call_operator_loc().is_some() {
            if let Some(rhs) = multiline_call_rhs(&receiver_call, file) {
                let same_line = file.same_line(
                    receiver.location().end_offset().saturating_sub(1),
                    rhs.start,
                );
                if same_line
                    && receiver
                        .as_call_node()
                        .is_some_and(|call| call.block().is_some())
                {
                    return Some((rhs.start, rhs.end));
                }
                deepest = Some((rhs.start, rhs.end, same_line));
            }
        }
        current = receiver;
    }
    deepest.and_then(|(start, end, same_line)| same_line.then_some((start, end)))
}

fn first_chain_rhs_any(
    call: &ruby_prism::CallNode<'_>,
    file: SourceFile<'_>,
) -> Option<(usize, usize)> {
    let mut current = call.receiver()?;
    let mut deepest = None;
    while let Some(receiver_call) = current.as_call_node() {
        let Some(receiver) = receiver_call.receiver() else {
            break;
        };
        if receiver_call.call_operator_loc().is_some() {
            if let Some(rhs) = multiline_call_rhs(&receiver_call, file) {
                deepest = Some((rhs.start, rhs.end));
            }
        }
        current = receiver;
    }
    deepest
}

fn alignment_context_before(start: usize, file: SourceFile<'_>) -> bool {
    let prefix = &file.as_str()[file.line_start(start)..start];
    hash_pair_prefix(prefix)
        || ["if ", "unless ", "while ", "until ", "for ", "return "]
            .iter()
            .any(|keyword| prefix.trim_start().starts_with(keyword))
        || prefix.rfind('=').is_some_and(|at| {
            !matches!(
                prefix.as_bytes().get(at.wrapping_sub(1)),
                Some(b'=' | b'!' | b'<' | b'>')
            ) && prefix.as_bytes().get(at + 1) != Some(&b'=')
        })
}

fn first_trailing_chain_base(
    call: &ruby_prism::CallNode<'_>,
    file: SourceFile<'_>,
) -> Option<(usize, usize)> {
    if let (Some(receiver), Some(operator), Some(selector)) = (
        call.receiver(),
        call.call_operator_loc(),
        call.message_loc(),
    ) {
        if !file.same_line(operator.start_offset(), selector.start_offset())
            && file.same_line(
                receiver.location().start_offset(),
                receiver.location().end_offset(),
            )
            && alignment_context_before(receiver.location().start_offset(), file)
        {
            return Some((receiver.location().start_offset(), operator.end_offset()));
        }
    }
    let mut current = call.receiver()?;
    let mut deepest = None;
    while let Some(receiver_call) = current.as_call_node() {
        let Some(receiver) = receiver_call.receiver() else {
            break;
        };
        if let (Some(operator), Some(selector)) = (
            receiver_call.call_operator_loc(),
            receiver_call.message_loc(),
        ) {
            if !file.same_line(operator.start_offset(), selector.start_offset()) {
                deepest = Some((receiver.location().start_offset(), operator.end_offset()));
            }
        }
        current = receiver;
    }
    deepest.filter(|(start, _)| alignment_context_before(*start, file))
}

fn prior_continuation_at_column(
    call: &ruby_prism::CallNode<'_>,
    rhs_start: usize,
    column: usize,
    file: SourceFile<'_>,
) -> bool {
    let first_line = file.line_start(call.location().start_offset());
    let current_line = file.line_start(rhs_start);
    file.lines().any(|(offset, line)| {
        if offset <= first_line || offset >= current_line {
            return false;
        }
        let actual = line.len() - line.trim_start().len();
        if actual != column {
            return false;
        }
        let trimmed = line.trim_start();
        trimmed.starts_with('.')
            || trimmed.starts_with("&.")
            || (offset > 0
                && file.as_str()[file.line_start(offset.saturating_sub(1))..offset]
                    .trim_end()
                    .ends_with('.'))
    })
}

fn immediately_follows_continuation_at_column(
    rhs_start: usize,
    column: usize,
    file: SourceFile<'_>,
) -> bool {
    let current_line = file.line_start(rhs_start);
    if current_line == 0 {
        return false;
    }
    let previous_start = file.line_start(current_line.saturating_sub(1));
    let previous = &file.as_str()[previous_start..current_line];
    previous.len() - previous.trim_start().len() == column
        && (previous.trim_start().starts_with('.') || previous.trim_start().starts_with("&."))
}

fn immediately_follows_block_end(rhs_start: usize, file: SourceFile<'_>) -> bool {
    let current_line = file.line_start(rhs_start);
    if current_line == 0 {
        return false;
    }
    let previous_start = file.line_start(current_line.saturating_sub(1));
    matches!(
        file.as_str()[previous_start..current_line].trim(),
        "}" | "end"
    )
}

fn prior_inline_block_dot_at_column(rhs_start: usize, column: usize, file: SourceFile<'_>) -> bool {
    file.lines().any(|(offset, line)| {
        offset < file.line_start(rhs_start)
            && line
                .find(" }.")
                .is_some_and(|at| file.column(offset + at + 2) == column)
    })
}

fn inside_hash_argument_of_multiline_chain(
    context: &CopContext<'_, '_>,
    file: SourceFile<'_>,
) -> bool {
    let Some(hash_index) = context.ancestors().iter().rposition(|ancestor| {
        ancestor.as_hash_node().is_some() || ancestor.as_keyword_hash_node().is_some()
    }) else {
        return false;
    };
    let hash = context.ancestors()[hash_index].location();
    context.ancestors()[..hash_index]
        .iter()
        .rev()
        .find_map(|ancestor| {
            let call = ancestor.as_call_node()?;
            call.arguments()
                .is_some_and(|arguments| {
                    let location = arguments.location();
                    location.start_offset() <= hash.start_offset()
                        && hash.end_offset() <= location.end_offset()
                })
                .then_some(call)
        })
        .is_some_and(|call| {
            call.receiver().is_some_and(|receiver| {
                multiline_call_rhs(&call, file).is_some_and(|rhs| {
                    !file.same_line(
                        receiver.location().end_offset().saturating_sub(1),
                        rhs.start,
                    )
                })
            })
        })
}

fn multiline_operation_indentation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let style = context.policy().enforced_style("aligned").to_string();
    let width = context.config_usize("IndentationWidth", 2);
    let normal_width = context
        .related_config_value("Layout/IndentationWidth", "Width")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2);
    for index in 1..lines.len() {
        if !line_ends_with_binary_operator(lines[index - 1].1) {
            continue;
        }
        let (_, current) = lines[index];
        if current.trim().is_empty() || current.trim_start().starts_with('#') {
            continue;
        }

        let mut first = index - 1;
        while first > 0 && line_ends_with_binary_operator(lines[first - 1].1) {
            first -= 1;
        }
        let first_line = lines[first].1;
        let mut base = first_line.len() - first_line.trim_start().len();
        let trimmed_first = first_line.trim_start();
        if method_argument_operation(trimmed_first) {
            continue;
        }
        let condition = ["if", "unless", "while", "until"]
            .into_iter()
            .find(|keyword| trimmed_first.starts_with(&format!("{keyword} ")));
        let collection = trimmed_first.starts_with("for ").then_some("for");
        let mut assignment = operation_assignment_rhs_column(first_line);
        if first > 0 && lines[first - 1].1.trim_end().ends_with('=') {
            base = lines[first - 1].1.len() - lines[first - 1].1.trim_start().len();
            assignment = Some(base + width);
        } else if first > 0 && lines[first - 1].1.trim_end().ends_with('\\') {
            assignment = operation_assignment_rhs_column(lines[first - 1].1);
            base = lines[first - 1].1.len() - lines[first - 1].1.trim_start().len();
        }
        let aligned_column = condition
            .map(|keyword| base + keyword.len() + 1)
            .or_else(|| collection.map(|_| first_line.find(" in ").map_or(base, |at| at + 4)))
            .or(assignment)
            .or_else(|| operation_argument_column(first_line));
        let grouped_expression = trimmed_first.starts_with('(');
        let nested_group_expression = first > 0
            && lines[..first]
                .iter()
                .map(|(_, line)| line)
                .fold(0isize, |depth, line| {
                    depth + line.matches('(').count() as isize - line.matches(')').count() as isize
                })
                > 0;
        let desired = if grouped_expression {
            base + 1
        } else if nested_group_expression && condition.is_none() && collection.is_none() {
            base
        } else if style == "indented" {
            if condition.is_some() || collection.is_some() {
                base + normal_width + width
            } else {
                base + width
            }
        } else {
            aligned_column
                .filter(|column| *column >= base + width)
                .unwrap_or(base + width)
        };
        let actual = current.len() - current.trim_start().len();
        if actual == desired {
            continue;
        }

        let noun = if let Some(keyword) = condition {
            format!(
                "a condition in {} `{keyword}` statement",
                if matches!(keyword, "if" | "unless" | "until") {
                    "an"
                } else {
                    "a"
                }
            )
        } else if collection.is_some() {
            "a collection in a `for` statement".to_string()
        } else if assignment.is_some() {
            "an expression in an assignment".to_string()
        } else {
            "an expression".to_string()
        };
        let aligned = style == "aligned"
            && aligned_column.is_some_and(|column| column == desired && column != base + width);
        let message = if aligned {
            format!("Align the operands of {noun} spanning multiple lines.")
        } else {
            format!(
                "Use {} (not {}) spaces for indenting {noun} spanning multiple lines.",
                desired.saturating_sub(base),
                actual.saturating_sub(base)
            )
        };
        let trimmed = current.trim_start();
        let operand_len = operation_operand_len(trimmed);
        let offense_start = lines[index].0 + actual;
        context.replace(
            message,
            offense_start..offense_start + operand_len,
            lines[index].0..offense_start,
            " ".repeat(desired),
        );
    }
}

fn line_ends_with_binary_operator(line: &str) -> bool {
    let code = line.split('#').next().unwrap_or(line).trim_end();
    if code.ends_with('|') && (code.contains(" do |") || code.contains("{ |")) {
        return false;
    }
    [
        "&&", "||", "and", "or", "+", "-", "*", "/", "%", "<<", ">>", "&", "|", "^",
    ]
    .iter()
    .any(|operator| code.ends_with(operator))
}

fn operation_assignment_rhs_column(line: &str) -> Option<usize> {
    const OPERATORS: [&str; 11] = [
        "||=", "&&=", "*=", "+=", "-=", "/=", "%=", "^=", "|=", "&=", " = ",
    ];
    OPERATORS.iter().find_map(|operator| {
        line.find(operator).map(|at| {
            let after = at + operator.len();
            after
                + line[after..]
                    .bytes()
                    .take_while(u8::is_ascii_whitespace)
                    .count()
        })
    })
}

fn operation_argument_column(line: &str) -> Option<usize> {
    if let Some(comma) = line.rfind(',') {
        let after = comma + 1;
        return Some(
            after
                + line[after..]
                    .bytes()
                    .take_while(u8::is_ascii_whitespace)
                    .count(),
        );
    }
    let trimmed = line.trim_start();
    if trimmed.starts_with('(') {
        return Some(line.len() - trimmed.len() + 1);
    }
    let quote = trimmed.find(['\'', '"']);
    quote.map(|at| line.len() - trimmed.len() + at)
}

fn operation_operand_len(line: &str) -> usize {
    let code = line.split('#').next().unwrap_or(line).trim_end();
    if matches!(code.as_bytes().first(), Some(b'\'' | b'"')) {
        let quote = code.as_bytes()[0];
        if let Some(closing) = code.as_bytes()[1..].iter().position(|byte| *byte == quote) {
            return closing + 2;
        }
    }
    for operator in [
        " &&", " ||", " and", " or", " +", " -", " *", " /", " <<", " >>",
    ] {
        if let Some(at) = code.rfind(operator) {
            return at.max(1);
        }
    }
    if let Some(at) = code.find(|character: char| character.is_ascii_whitespace()) {
        if code[at..].trim_start().starts_with(['<', '>', '=']) {
            return at.max(1);
        }
    }
    code.len().max(1)
}

fn method_argument_operation(line: &str) -> bool {
    let Some(opening) = line.find('(') else {
        return false;
    };
    opening > 0
        && line[..opening]
            .chars()
            .next_back()
            .is_some_and(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '!' | '?')
            })
        && !line[opening + 1..].contains(')')
}

struct HashAlignmentCop;

impl Cop for HashAlignmentCop {
    fn name(&self) -> &'static str {
        "Layout/HashAlignment"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let (elements, explicit, location) = if let Some(hash) = node.as_hash_node() {
            (
                hash.elements().iter().collect::<Vec<_>>(),
                true,
                hash.location(),
            )
        } else if let Some(hash) = node.as_keyword_hash_node() {
            (
                hash.elements().iter().collect::<Vec<_>>(),
                false,
                hash.location(),
            )
        } else {
            return;
        };
        let mut reporter = context.cop_context(self.name(), source, ancestors);
        check_hash_alignment(&elements, explicit, location, &mut reporter);
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum HashAlignmentStyle {
    Key,
    Separator,
    Table,
}

#[derive(Clone)]
struct HashAlignmentEntry {
    start: usize,
    end: usize,
    key_start: usize,
    key_end: usize,
    operator_start: Option<usize>,
    operator_end: Option<usize>,
    value_start: Option<usize>,
    rocket: bool,
    omission: bool,
    splat: bool,
}

#[derive(Clone, Copy, Default)]
struct HashAlignmentDelta {
    key: isize,
    separator: isize,
    value: isize,
}

fn check_hash_alignment(
    elements: &[Node<'_>],
    explicit: bool,
    hash_location: ruby_prism::Location<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    if file.same_line(hash_location.start_offset(), hash_location.end_offset()) {
        return;
    }
    if hash_is_ignored_last_argument(explicit, &hash_location, context) {
        return;
    }
    let entries = elements
        .iter()
        .filter_map(|element| {
            if let Some(pair) = element.as_assoc_node() {
                let location = pair.location();
                let key = pair.key().location();
                let value = pair.value().location();
                let operator = pair.operator_loc();
                let rocket = operator
                    .as_ref()
                    .is_some_and(|location| location.as_slice() == b"=>");
                let operator_start = operator
                    .as_ref()
                    .map(ruby_prism::Location::start_offset)
                    .or_else(|| {
                        (key.end_offset() > key.start_offset()).then(|| key.end_offset() - 1)
                    });
                let operator_end = operator
                    .as_ref()
                    .map(ruby_prism::Location::end_offset)
                    .or(Some(key.end_offset()));
                let semantic_key_end = if rocket {
                    key.end_offset()
                } else {
                    key.end_offset().saturating_sub(1)
                };
                Some(HashAlignmentEntry {
                    start: location.start_offset(),
                    end: location.end_offset(),
                    key_start: key.start_offset(),
                    key_end: semantic_key_end,
                    operator_start,
                    operator_end,
                    value_start: Some(value.start_offset()),
                    rocket,
                    omission: value.start_offset() < key.end_offset(),
                    splat: false,
                })
            } else {
                let splat = element.as_assoc_splat_node()?;
                let location = splat.location();
                Some(HashAlignmentEntry {
                    start: location.start_offset(),
                    end: location.end_offset(),
                    key_start: location.start_offset(),
                    key_end: location.end_offset(),
                    operator_start: None,
                    operator_end: None,
                    value_start: None,
                    rocket: false,
                    omission: true,
                    splat: true,
                })
            }
        })
        .collect::<Vec<_>>();
    let Some(first) = entries.iter().find(|entry| !entry.splat) else {
        return;
    };
    if context.related_config_value("Layout/ArgumentAlignment", "EnforcedStyle")
        == Some("with_fixed_indentation")
        && hash_starts_beside_call_argument(first.start, &hash_location, context)
    {
        return;
    }

    let pair_entries = entries
        .iter()
        .filter(|entry| !entry.splat)
        .collect::<Vec<_>>();
    let same_line_pairs = pair_entries
        .windows(2)
        .any(|pair| file.same_line(pair[0].end.saturating_sub(1), pair[1].start));
    let mixed_delimiters = pair_entries.iter().any(|entry| entry.rocket)
        && pair_entries.iter().any(|entry| !entry.rocket);
    let configured = configured_hash_styles(context, pair_entries.iter().any(|entry| entry.rocket));
    let first_configured_style = configured.first().copied();
    if mixed_delimiters && !configured.contains(&HashAlignmentStyle::Key) {
        return;
    }
    let mut candidates = Vec::new();
    for style in configured {
        if style != HashAlignmentStyle::Key && same_line_pairs {
            continue;
        }
        let deltas = hash_alignment_deltas(style, &pair_entries, file);
        let offenses = deltas
            .iter()
            .filter(|(_, delta)| delta.key != 0 || delta.separator != 0 || delta.value != 0)
            .count();
        candidates.push((offenses, style, deltas));
    }
    let Some((_, style, deltas)) = candidates.into_iter().min_by_key(|candidate| candidate.0)
    else {
        return;
    };
    let first_key_column = file.column(first.key_start);
    for entry in entries.iter().filter(|entry| entry.splat) {
        if begins_line(file, entry.start) {
            let delta = HashAlignmentDelta {
                key: first_key_column as isize - file.column(entry.start) as isize,
                ..HashAlignmentDelta::default()
            };
            if delta.key != 0 {
                report_hash_alignment(
                    context,
                    entry,
                    delta,
                    "Align keyword splats with the rest of the hash if it spans more than one line.",
                );
            }
        }
    }
    let message = match style {
        HashAlignmentStyle::Key => {
            "Align the keys of a hash literal if they span more than one line."
        }
        HashAlignmentStyle::Separator => {
            "Align the separators of a hash literal if they span more than one line."
        }
        HashAlignmentStyle::Table => {
            "Align the keys and values of a hash literal if they span more than one line."
        }
    };
    let correction_deltas = first_configured_style
        .map(|correction_style| hash_alignment_deltas(correction_style, &pair_entries, file))
        .unwrap_or_default();
    for (entry, delta) in deltas {
        if delta.key != 0 || delta.separator != 0 || delta.value != 0 {
            let correction = correction_deltas
                .iter()
                .find(|(candidate, _)| candidate.start == entry.start)
                .map_or(delta, |(_, correction)| *correction);
            if correction.key == 0 && correction.separator == 0 && correction.value == 0 {
                if !context.autocorrect_enabled() {
                    context.report(message, entry.start..entry.end);
                }
            } else {
                report_hash_alignment(context, entry, correction, message);
            }
        }
    }
}

fn configured_hash_styles(context: &CopContext<'_, '_>, rockets: bool) -> Vec<HashAlignmentStyle> {
    let key = if rockets {
        "EnforcedHashRocketStyle"
    } else {
        "EnforcedColonStyle"
    };
    let array_values = context.config_values(key);
    let values = if let Some(value) = context.config_value(key) {
        vec![value]
    } else if array_values.is_empty() {
        vec!["key"]
    } else {
        array_values.iter().map(String::as_str).collect()
    };
    values
        .into_iter()
        .filter_map(|value| match value {
            "key" => Some(HashAlignmentStyle::Key),
            "separator" => Some(HashAlignmentStyle::Separator),
            "table" => Some(HashAlignmentStyle::Table),
            _ => None,
        })
        .collect()
}

fn hash_alignment_deltas<'a>(
    style: HashAlignmentStyle,
    entries: &[&'a HashAlignmentEntry],
    file: SourceFile<'_>,
) -> Vec<(&'a HashAlignmentEntry, HashAlignmentDelta)> {
    let Some(first) = entries.first().copied() else {
        return Vec::new();
    };
    let first_key_column = file.column(first.key_start);
    let first_key_end_column = file.column(first.key_end);
    let first_operator_column = first.operator_start.map(|offset| file.column(offset));
    let first_value_column = first.value_start.map(|offset| file.column(offset));
    let max_key_width = entries
        .iter()
        .map(|entry| entry.key_end - entry.key_start)
        .max()
        .unwrap_or(0);
    let max_delimiter_width = if entries.iter().any(|entry| entry.rocket) {
        4
    } else {
        2
    };

    entries
        .iter()
        .enumerate()
        .map(|(index, entry)| {
            let key_column = file.column(entry.key_start);
            let key_end_column = file.column(entry.key_end);
            let operator_column = entry.operator_start.map(|offset| file.column(offset));
            let operator_end_column = entry.operator_end.map(|offset| file.column(offset));
            let value_column = entry.value_start.map(|offset| file.column(offset));
            let on_own_line = begins_line(file, entry.start);
            let mut delta = HashAlignmentDelta::default();
            match style {
                HashAlignmentStyle::Key => {
                    if index > 0 && on_own_line {
                        delta.key = first_key_column as isize - key_column as isize;
                    }
                    if entry.rocket {
                        let desired = key_end_column + 1;
                        delta.separator =
                            desired as isize - operator_column.unwrap_or(desired) as isize;
                    }
                    if !entry.omission
                        && entry
                            .value_start
                            .is_some_and(|value| file.same_line(entry.start, value))
                    {
                        let desired = operator_end_column.unwrap_or(key_end_column) + 1;
                        delta.value = desired as isize - value_column.unwrap_or(desired) as isize;
                    }
                }
                HashAlignmentStyle::Table => {
                    if index > 0 && on_own_line {
                        delta.key = first_key_column as isize - key_column as isize;
                    }
                    if entry.rocket {
                        let desired = first_key_column + max_key_width + 1;
                        delta.separator = desired as isize
                            - operator_column.unwrap_or(desired) as isize
                            - delta.key;
                    }
                    if !entry.omission {
                        let desired = first_key_column + max_key_width + max_delimiter_width;
                        delta.value = desired as isize
                            - value_column.unwrap_or(desired) as isize
                            - delta.key
                            - delta.separator;
                    }
                }
                HashAlignmentStyle::Separator => {
                    if index > 0 && on_own_line {
                        delta.key = first_key_end_column as isize - key_end_column as isize;
                    }
                    if entry.rocket {
                        let desired = first_operator_column.unwrap_or(key_end_column);
                        delta.separator = desired as isize
                            - operator_column.unwrap_or(desired) as isize
                            - delta.key;
                    }
                    if !entry.omission {
                        let desired = first_value_column.unwrap_or(key_end_column);
                        delta.value = desired as isize
                            - value_column.unwrap_or(desired) as isize
                            - delta.key
                            - delta.separator;
                    }
                }
            }
            delta.key = delta.key.max(-(key_column as isize));
            (*entry, delta)
        })
        .collect()
}

fn begins_line(file: SourceFile<'_>, offset: usize) -> bool {
    file.as_str()[file.line_start(offset)..offset]
        .trim()
        .is_empty()
}

fn hash_is_ignored_last_argument(
    explicit: bool,
    location: &ruby_prism::Location<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let style = context.config_value("EnforcedLastArgumentHashStyle");
    if style == Some("always_inspect") || style.is_none() {
        return false;
    }
    let is_last_argument = context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| {
            call.arguments().is_some_and(|arguments| {
                arguments.arguments().iter().last().is_some_and(|last| {
                    last.location().start_offset() == location.start_offset()
                        && last.location().end_offset() == location.end_offset()
                })
            })
        }) || ancestor.as_super_node().is_some_and(|call| {
            call.arguments().is_some_and(|arguments| {
                arguments.arguments().iter().last().is_some_and(|last| {
                    last.location().start_offset() == location.start_offset()
                        && last.location().end_offset() == location.end_offset()
                })
            })
        }) || ancestor.as_yield_node().is_some_and(|call| {
            call.arguments().is_some_and(|arguments| {
                arguments.arguments().iter().last().is_some_and(|last| {
                    last.location().start_offset() == location.start_offset()
                        && last.location().end_offset() == location.end_offset()
                })
            })
        })
    });
    is_last_argument
        && matches!(
            (style, explicit),
            (Some("always_ignore"), _)
                | (Some("ignore_explicit"), true)
                | (Some("ignore_implicit"), false)
        )
}

fn hash_starts_beside_call_argument(
    first_pair: usize,
    hash_location: &ruby_prism::Location<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let file = context.source_file();
    context.ancestors().iter().rev().any(|ancestor| {
        let Some(call) = ancestor.as_call_node() else {
            return false;
        };
        let Some(arguments) = call.arguments() else {
            return false;
        };
        let nodes = arguments.arguments().iter().collect::<Vec<_>>();
        let Some(index) = nodes.iter().position(|argument| {
            argument.location().start_offset() == hash_location.start_offset()
                && argument.location().end_offset() == hash_location.end_offset()
        }) else {
            return false;
        };
        let anchor = if index > 0 {
            nodes[index - 1].location().end_offset().saturating_sub(1)
        } else {
            call.message_loc()
                .map_or(call.location().start_offset(), |location| {
                    location.start_offset()
                })
        };
        file.same_line(anchor, first_pair)
    })
}

fn report_hash_alignment(
    context: &mut CopContext<'_, '_>,
    entry: &HashAlignmentEntry,
    delta: HashAlignmentDelta,
    message: &str,
) {
    let source = context.source();
    let mut edits = Vec::new();
    for (offset, amount) in [
        (entry.key_start, delta.key),
        (
            entry.operator_start.unwrap_or(entry.key_start),
            delta.separator,
        ),
        (entry.value_start.unwrap_or(entry.key_start), delta.value),
    ] {
        if amount != 0 {
            edits.push((offset, amount));
        }
    }
    edits.sort_by_key(|(offset, _)| std::cmp::Reverse(*offset));
    let correction_start = edits
        .iter()
        .filter(|(_, amount)| *amount < 0)
        .map(|(offset, amount)| offset.saturating_sub(amount.unsigned_abs()))
        .min()
        .unwrap_or(entry.start)
        .min(entry.start);
    let mut replacement = source[correction_start..entry.end].to_string();
    for (offset, amount) in edits {
        let relative = offset - correction_start;
        if amount > 0 {
            replacement.insert_str(relative, &" ".repeat(amount as usize));
        } else {
            let start = relative.saturating_sub(amount.unsigned_abs());
            replacement.replace_range(start..relative, "");
        }
    }
    context.replace(
        message,
        entry.start..entry.end,
        correction_start..entry.end,
        replacement,
    );
}

fn operator_spacing(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_owned();
    let exponent_space = context
        .config_value("EnforcedStyleForExponentOperator")
        .is_some_and(|style| style == "space");
    let rational_space = context
        .config_value("EnforcedStyleForRationalLiterals")
        .is_some_and(|style| style == "space");
    let allow_alignment = !context
        .config_value("AllowForAlignment")
        .is_some_and(|value| value == "false");
    let force_equal_alignment = context
        .related_config_value("Layout/ExtraSpacing", "ForceEqualSignAlignment")
        .is_some_and(|value| value == "true");
    let hash_table_style = context
        .related_config_value("Layout/HashAlignment", "EnforcedHashRocketStyle")
        .is_some_and(|style| style.contains("table"))
        || context
            .related_config_values("Layout/HashAlignment", "EnforcedHashRocketStyle")
            .iter()
            .any(|style| style == "table");
    let literal_ranges = context.source_file().literal_ranges();
    let comment_ranges = context.source_file().comment_ranges();
    let unary_operator_offsets = unary_operator_offsets(&source);
    let embedded_code_ranges = embedded_code_ranges(&source);

    for (line_offset, line) in context.source_file().lines().collect::<Vec<_>>() {
        if line.trim() == "__END__" {
            break;
        }
        // ERB-backed generator templates are not Ruby syntax at the template
        // delimiters. RuboCop's Ruby parser does not expose those delimiters
        // as operator nodes, so source-oriented scanning must ignore the line.
        if line.contains("<%") || line.contains("%>") {
            continue;
        }
        let code_end = ruby_comment_start(line).unwrap_or(line.len());
        let code = &line[..code_end];
        let mut index = 0;
        let mut quote = None;
        let mut ternary_depth = 0usize;
        while index < code.len() {
            let absolute = line_offset + index;
            if let Some(range) = comment_ranges
                .iter()
                .find(|range| range.start <= absolute && absolute < range.end)
            {
                index = range.end.saturating_sub(line_offset).min(code.len());
                continue;
            }
            if let Some(range) = literal_ranges
                .iter()
                .filter(|range| range.start <= absolute && absolute < range.end)
                .min_by_key(|range| range.end - range.start)
            {
                let containing_embedded = embedded_code_ranges
                    .iter()
                    .find(|embedded| embedded.start <= absolute && absolute < embedded.end);
                if containing_embedded.is_some_and(|embedded| {
                    range.start < embedded.start || embedded.end < range.end
                }) {
                    // The selected range is the enclosing interpolated literal,
                    // while this offset is executable Ruby inside `#{...}`.
                } else {
                let next_embedded = embedded_code_ranges
                    .iter()
                    .filter(|embedded| {
                        absolute < embedded.start
                            && range.start <= embedded.start
                            && embedded.start < range.end
                    })
                    .map(|embedded| embedded.start)
                    .min();
                index = next_embedded
                    .unwrap_or(range.end)
                    .saturating_sub(line_offset)
                    .min(code.len());
                continue;
                }
            }
            if !code.is_char_boundary(index) {
                index += 1;
                continue;
            }
            let byte = code.as_bytes()[index];
            let character_width = code[index..]
                .chars()
                .next()
                .map_or(1, char::len_utf8);
            if let Some(delimiter) = quote {
                if byte == b'\\' {
                    index += 1;
                    if index < code.len() {
                        index += code[index..]
                            .chars()
                            .next()
                            .map_or(1, char::len_utf8);
                    }
                    continue;
                }
                if byte == delimiter {
                    quote = None;
                }
                index += character_width;
                continue;
            }
            if matches!(byte, b'\'' | b'"' | b'`') {
                quote = Some(byte);
                index += 1;
                continue;
            }
            if byte == b'/' && slash_starts_regexp(code, index) {
                quote = Some(b'/');
                index += 1;
                continue;
            }

            if !byte.is_ascii() {
                index += character_width;
                continue;
            }

            let Some(operator) = spacing_operator_at(code, index) else {
                index += 1;
                continue;
            };
            let end = index + operator.len();
            if unary_operator_offsets.contains(&absolute) {
                index = end;
                continue;
            }
            if operator == ":" && ternary_depth == 0 {
                index = end;
                continue;
            }
            if operator_is_non_binary(code, index, end, operator) {
                index = end;
                continue;
            }
            if operator == "?" {
                ternary_depth += 1;
            } else if operator == ":" {
                ternary_depth = ternary_depth.saturating_sub(1);
            }

            let left_start = code[..index]
                .rfind(|character: char| !character.is_ascii_whitespace())
                .map_or(index, |at| {
                    at + code[at..]
                        .chars()
                        .next()
                        .map_or(1, char::len_utf8)
                });
            let right_end = end
                + code[end..]
                    .find(|character: char| !character.is_ascii_whitespace())
                    .unwrap_or(code.len() - end);
            let left_space = &code[left_start..index];
            let right_space = &code[end..right_end];
            let rational = operator == "/" && rational_rhs(&code[right_end..]);
            let compact = operator == "**" && !exponent_space || rational && !rational_space;

            if allow_alignment
                && operator == "|"
                && single_pipe_count(code) >= 2
                && !left_space.is_empty()
                && !right_space.is_empty()
            {
                index = end;
                continue;
            }

            if operator == "=>"
                && hash_table_style
                && source.contains('\n')
                && line.trim_end().ends_with(',')
            {
                index = end;
                continue;
            }

            let message = if compact {
                if left_space.is_empty() && right_space.is_empty() {
                    index = end;
                    continue;
                }
                format!("Space around operator `{operator}` detected.")
            } else if left_space.is_empty() || right_space.is_empty() {
                // A line break immediately after an operator is accepted.
                if right_end == code.len() && right_space.is_empty() && !left_space.is_empty() {
                    index = end;
                    continue;
                }
                format!("Surrounding space missing for operator `{operator}`.")
            } else if left_space.len() > 1 || right_space.len() > 1 {
                if right_end == code.len() && code_end < line.len() {
                    index = end;
                    continue;
                }
                if force_equal_alignment && operator == "=" && right_space.len() == 1 {
                    index = end;
                    continue;
                }
                if allow_alignment
                    && operator_alignment_is_allowed(
                        &source,
                        line_offset,
                        index,
                        end,
                        left_start,
                        right_end,
                        operator,
                    )
                {
                    index = end;
                    continue;
                }
                format!("Operator `{operator}` should be surrounded by a single space.")
            } else {
                index = end;
                continue;
            };

            let replacement = if compact {
                operator.to_owned()
            } else if right_end == code.len() && right_space.is_empty() {
                format!(" {operator}")
            } else if right_end == code.len() && !right_space.is_empty() {
                format!(
                    " {operator}{}",
                    if line.ends_with('\n') { "\n" } else { "" }
                )
            } else {
                format!(" {operator} ")
            };
            context.replace(
                message,
                line_offset + index..line_offset + end,
                line_offset + left_start..line_offset + right_end,
                replacement,
            );
            index = end;
        }
    }
}

fn unary_operator_offsets(source: &str) -> std::collections::HashSet<usize> {
    #[derive(Default)]
    struct UnaryOperators(std::collections::HashSet<usize>);

    impl<'pr> ruby_prism::Visit<'pr> for UnaryOperators {
        fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode<'pr>) {
            self.0.insert(node.operator_loc().start_offset());
            ruby_prism::visit_splat_node(self, node);
        }

        fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode<'pr>) {
            self.0.insert(node.operator_loc().start_offset());
            ruby_prism::visit_block_argument_node(self, node);
        }

        fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode<'pr>) {
            if let Some(opening) = node.opening_loc() {
                self.0.insert(opening.start_offset());
            }
            if let Some(closing) = node.closing_loc() {
                self.0.insert(closing.start_offset());
            }
            ruby_prism::visit_block_parameters_node(self, node);
        }

        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            if matches!(node.name().as_slice(), b"+@" | b"-@" | b"!" | b"~") {
                if let Some(operator) = node.message_loc() {
                    self.0.insert(operator.start_offset());
                }
            }
            ruby_prism::visit_call_node(self, node);
        }

        fn visit_integer_node(&mut self, node: &ruby_prism::IntegerNode<'pr>) {
            if node
                .location()
                .as_slice()
                .first()
                .is_some_and(|byte| matches!(byte, b'+' | b'-'))
            {
                self.0.insert(node.location().start_offset());
            }
        }

        fn visit_float_node(&mut self, node: &ruby_prism::FloatNode<'pr>) {
            if node
                .location()
                .as_slice()
                .first()
                .is_some_and(|byte| matches!(byte, b'+' | b'-'))
            {
                self.0.insert(node.location().start_offset());
            }
        }

        fn visit_rational_node(&mut self, node: &ruby_prism::RationalNode<'pr>) {
            if node
                .location()
                .as_slice()
                .first()
                .is_some_and(|byte| matches!(byte, b'+' | b'-'))
            {
                self.0.insert(node.location().start_offset());
            }
        }

        fn visit_imaginary_node(&mut self, node: &ruby_prism::ImaginaryNode<'pr>) {
            if node
                .location()
                .as_slice()
                .first()
                .is_some_and(|byte| matches!(byte, b'+' | b'-'))
            {
                self.0.insert(node.location().start_offset());
            }
        }
    }

    let parsed = ruby_prism::parse(source.as_bytes());
    let mut operators = UnaryOperators::default();
    operators.visit(&parsed.node());
    operators.0
}

fn embedded_code_ranges(source: &str) -> Vec<std::ops::Range<usize>> {
    #[derive(Default)]
    struct EmbeddedCode(Vec<std::ops::Range<usize>>);

    impl<'pr> ruby_prism::Visit<'pr> for EmbeddedCode {
        fn visit_embedded_statements_node(
            &mut self,
            node: &ruby_prism::EmbeddedStatementsNode<'pr>,
        ) {
            let location = node.location();
            self.0
                .push(location.start_offset()..location.end_offset());
            ruby_prism::visit_embedded_statements_node(self, node);
        }

        fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode<'pr>) {
            let location = node.location();
            self.0
                .push(location.start_offset()..location.end_offset());
            ruby_prism::visit_embedded_variable_node(self, node);
        }
    }

    let parsed = ruby_prism::parse(source.as_bytes());
    let mut embedded = EmbeddedCode::default();
    embedded.visit(&parsed.node());
    embedded.0
}

fn ruby_comment_start(line: &str) -> Option<usize> {
    let mut quote = None;
    let mut escaped = false;
    for (index, byte) in line.bytes().enumerate() {
        if escaped {
            escaped = false;
        } else if quote.is_some() && byte == b'\\' {
            escaped = true;
        } else if quote == Some(byte) {
            quote = None;
        } else if quote.is_none() && matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
        } else if quote.is_none() && byte == b'/' && slash_starts_regexp(line, index) {
            quote = Some(byte);
        } else if quote.is_none() && byte == b'#' {
            return Some(index);
        }
    }
    None
}

fn spacing_operator_at(source: &str, index: usize) -> Option<&str> {
    const OPERATORS: [&str; 40] = [
        "<=>", "||=", "&&=", "===", "**=", "<<=", ">>=", "==", "!=", "=~", "!~", "<=", ">=", "=>",
        "+=", "-=", "*=", "/=", "%=", "^=", "|=", "&=", "<<", ">>", "&&", "||", "**", "+", "-",
        "*", "/", "%", "^", "&", "|", "<", ">", "?", ":", "=",
    ];
    OPERATORS
        .iter()
        .find(|operator| source[index..].starts_with(**operator))
        .copied()
}

fn single_pipe_count(source: &str) -> usize {
    let bytes = source.as_bytes();
    bytes
        .iter()
        .enumerate()
        .filter(|(index, byte)| {
            **byte == b'|'
                && (*index == 0 || bytes.get(index - 1) != Some(&b'|'))
                && bytes.get(index + 1) != Some(&b'|')
        })
        .count()
}

fn slash_starts_regexp(source: &str, index: usize) -> bool {
    let before = source[..index].trim_end();
    before.is_empty()
        || before.ends_with(['(', '[', '{', ',', '=', '~', '!', ':', ';'])
        || before
            .split_ascii_whitespace()
            .next_back()
            .is_some_and(|word| {
                matches!(
                    word,
                    "if" | "unless" | "while" | "until" | "when" | "return" | "next" | "break"
                )
            })
}

fn operator_is_non_binary(source: &str, start: usize, end: usize, operator: &str) -> bool {
    let before = source[..start].trim_end();
    let after = source[end..].trim_start();
    if before.is_empty() {
        return true;
    }
    if operator_is_method_name(source, start) {
        return true;
    }
    if source.trim_start().starts_with("def ") && matches!(operator, "*" | "**" | "&") {
        return true;
    }
    if before.ends_with("def") || before.ends_with("def self.") || before.ends_with('.') {
        return true;
    }
    if operator == "<<" && source[end..].starts_with(['~', '-']) {
        return true;
    }
    if operator == "=" && source.trim_start().starts_with("def ") && before.contains('(') {
        return true;
    }
    if operator == "=" && (before.ends_with(['!', '<', '>', '=']) || after.starts_with('>')) {
        return true;
    }
    if operator == "&" && after.starts_with('.') {
        return true;
    }
    if source[..start].ends_with('$') {
        return true;
    }
    if (operator == "-" && source[end..].starts_with('>'))
        || (operator == ">" && source[..start].ends_with('-'))
    {
        return true;
    }
    if operator == ":" && (before.ends_with(':') || source[end..].starts_with(':')) {
        return true;
    }
    if operator == ":"
        && before.ends_with(['(', '[', '{', ','])
        && source[end..]
            .chars()
            .next()
            .is_some_and(|character| character.is_alphanumeric() || character == '_')
    {
        return true;
    }
    let predicate_suffix = source[..start]
        .chars()
        .rev()
        .take_while(|character| character.is_alphanumeric() || *character == '_')
        .any(|character| character.is_alphabetic() || character == '_');
    if operator == "?"
        && (predicate_suffix
            || after.starts_with([':', '?', '('])
            || source
                .as_bytes()
                .get(end)
                .is_some_and(u8::is_ascii_whitespace)
                && !source[end..].contains(':'))
    {
        return true;
    }
    if matches!(operator, "*" | "**" | "&")
        && (before.is_empty()
            || before.ends_with(['(', '[', '{', ',', '=', ':', '|'])
            || before
                .split_whitespace()
                .last()
                .is_some_and(|word| {
                    matches!(word, "return" | "yield" | "when" | "in" | "rescue")
                }))
    {
        return true;
    }
    if matches!(operator, "+" | "-" | "!" | "~")
        && (before.is_empty()
            || before.ends_with(['(', '[', '{', ',', '=', ':', '|', '>', '<', '?'])
            || before.ends_with("<<")
            || before
                .split_ascii_whitespace()
                .next_back()
                .is_some_and(|word| matches!(word, "return" | "next" | "break" | "when")))
    {
        return true;
    }
    if operator == "|"
        && (before.ends_with('{')
            || before
                .split_ascii_whitespace()
                .next_back()
                .is_some_and(|word| word == "do")
            || after.starts_with('}')
            || source[..start].rfind('|').is_some_and(|previous| {
                let before_previous = source[..previous].trim_end();
                before_previous.ends_with('{')
                    || before_previous
                        .split_ascii_whitespace()
                        .next_back()
                        .is_some_and(|word| word == "do")
            })
            || !source.contains("in ") && source[..start].matches('|').count() % 2 == 1)
    {
        return true;
    }
    // Range, namespace, symbol, and keyword-argument punctuation are not
    // binary operators handled by this cop.
    if matches!(operator, "+" | "-") && before.ends_with(':') {
        return true;
    }
    false
}

fn operator_is_method_name(source: &str, operator_start: usize) -> bool {
    let indentation = source.len() - source.trim_start().len();
    let trimmed = &source[indentation..];
    let Some(rest) = trimmed.strip_prefix("def ") else {
        return false;
    };
    let name_start = indentation + "def ".len();
    let name_len = rest
        .find(|character: char| character.is_ascii_whitespace() || character == '(')
        .unwrap_or(rest.len());
    name_start <= operator_start && operator_start < name_start + name_len
}

fn rational_rhs(source: &str) -> bool {
    let token = source
        .trim_start()
        .bytes()
        .take_while(|byte| byte.is_ascii_digit() || *byte == b'_')
        .count();
    token > 0 && source.trim_start().as_bytes().get(token) == Some(&b'r')
}

fn operator_alignment_is_allowed(
    source: &str,
    line_offset: usize,
    operator_start: usize,
    operator_end: usize,
    whitespace_start: usize,
    rhs_start: usize,
    operator: &str,
) -> bool {
    let before = &source[..line_offset];
    let previous = before
        .lines()
        .rev()
        .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'));
    let after_line = source[line_offset..]
        .find('\n')
        .map_or(source.len(), |at| line_offset + at + 1);
    let next = source[after_line..]
        .lines()
        .find(|line| !line.trim().is_empty() && !line.trim_start().starts_with('#'));
    let current_lhs = source[line_offset..line_offset + whitespace_start]
        .trim_end()
        .len();
    let leading_excess = operator_start.saturating_sub(whitespace_start) > 1;
    let trailing_excess = rhs_start.saturating_sub(operator_end) > 1;
    [previous, next].into_iter().flatten().any(|line| {
        operator_layouts(line).into_iter().any(|layout| {
            let leading_aligned = !leading_excess
                || layout.1 == operator_end
                    && ((operator.ends_with('=') && line[layout.0..layout.1].ends_with('='))
                        || layout.2 != current_lhs);
            let trailing_aligned = !trailing_excess
                || layout.2 != current_lhs && (layout.3 == rhs_start || layout.1 == operator_end);
            leading_aligned && trailing_aligned
        })
    }) || alignment_table_is_allowed(
        source,
        line_offset,
        operator_start,
        operator_end,
        leading_excess,
        trailing_excess,
        rhs_start,
        operator,
    )
}

fn alignment_table_is_allowed(
    source: &str,
    line_offset: usize,
    operator_start: usize,
    operator_end: usize,
    leading_excess: bool,
    trailing_excess: bool,
    rhs_start: usize,
    operator: &str,
) -> bool {
    if !matches!(operator, "|" | "<<") && !operator.ends_with('=') {
        return false;
    }
    if !source
        .as_bytes()
        .get(line_offset)
        .is_some_and(u8::is_ascii_whitespace)
    {
        return false;
    }
    let line_end = source[line_offset..]
        .find('\n')
        .map_or(source.len(), |end| line_offset + end);
    let current_line = &source[line_offset..line_end];
    let current_indent = current_line.len() - current_line.trim_start().len();
    let first_aligned_operator = operator_layouts(current_line)
        .into_iter()
        .find(|layout| {
            let candidate = &current_line[layout.0..layout.1];
            candidate.ends_with('=') || matches!(candidate, "<<" | "|")
        })
        .is_some_and(|layout| layout.0 == operator_start);
    if operator.ends_with('=') && !first_aligned_operator {
        return false;
    }

    source.lines().any(|line| {
        if line == current_line || line.len() - line.trim_start().len() != current_indent {
            return false;
        }
        operator_layouts(line).into_iter().any(|layout| {
            let candidate = &line[layout.0..layout.1];
            let candidate_lhs = line[..layout.2].trim_end().len();
            let current_lhs = current_line[..operator_start]
                .trim_end_matches(char::is_whitespace)
                .len();
            let different_width = candidate_lhs != current_lhs;
            let compatible = candidate == operator
                || operator.ends_with('=') && (candidate.ends_with('=') || candidate == "<<")
                || operator == "<<" && candidate.ends_with('=');
            let leading_aligned = !leading_excess || compatible && layout.1 == operator_end;
            let trailing_aligned = !trailing_excess || different_width && layout.3 == rhs_start;
            leading_aligned && trailing_aligned
        })
    })
}

fn operator_layouts(line: &str) -> Vec<(usize, usize, usize, usize)> {
    let mut layouts = Vec::new();
    let code = &line[..ruby_comment_start(line).unwrap_or(line.len())];
    let mut index = 0;
    let mut quote = None;
    while index < code.len() {
        if !code.is_char_boundary(index) {
            index += 1;
            continue;
        }
        let byte = code.as_bytes()[index];
        let character_width = code[index..]
            .chars()
            .next()
            .map_or(1, char::len_utf8);
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                index += 1;
                if index < code.len() {
                    index += code[index..]
                        .chars()
                        .next()
                        .map_or(1, char::len_utf8);
                }
                continue;
            }
            if byte == delimiter {
                quote = None;
            }
            index += character_width;
            continue;
        }
        if matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
            index += 1;
            continue;
        }
        if byte == b'/' && slash_starts_regexp(code, index) {
            quote = Some(b'/');
            index += 1;
            continue;
        }
        if !byte.is_ascii() {
            index += character_width;
            continue;
        }
        let Some(operator) = spacing_operator_at(code, index) else {
            index += 1;
            continue;
        };
        let end = index + operator.len();
        if !operator_is_non_binary(code, index, end, operator) {
            let whitespace_start = code[..index]
                .rfind(|character: char| !character.is_ascii_whitespace())
                .map_or(index, |at| {
                    at + code[at..]
                        .chars()
                        .next()
                        .map_or(1, char::len_utf8)
                });
            let rhs_start = end
                + code[end..]
                    .find(|character: char| !character.is_ascii_whitespace())
                    .unwrap_or(code.len() - end);
            layouts.push((index, end, whitespace_start, rhs_start));
        }
        index = end;
    }
    layouts
}

fn heredoc_indentation(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 3) {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let width = context.config_usize("IndentationWidth", 2);
    let active_support = context
        .related_config_value("AllCops", "ActiveSupportExtensionsEnabled")
        .is_some_and(|value| value == "true");
    let max = (context.related_config_value("Layout/LineLength", "Enabled") != Some("false")
        && context.related_config_value("Layout/LineLength", "AllowHeredoc") == Some("false"))
    .then(|| {
        context
            .related_config_value("Layout/LineLength", "Max")
            .and_then(|value| value.parse::<usize>().ok())
    })
    .flatten();

    for (opening_index, (opening_offset, opening_line)) in lines.iter().copied().enumerate() {
        let Some(marker_at) = opening_line.find("<<") else {
            continue;
        };
        let after = &opening_line[marker_at + 2..];
        let (modifier, after) = if let Some(after) = after.strip_prefix('~') {
            ("<<~", after)
        } else if let Some(after) = after.strip_prefix('-') {
            ("<<-", after)
        } else {
            ("<<", after)
        };
        let quote = after
            .as_bytes()
            .first()
            .copied()
            .filter(|byte| matches!(byte, b'\'' | b'"' | b'`'));
        let name_start = usize::from(quote.is_some());
        let Some(name_end) = after[name_start..]
            .find(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
            .map(|at| at + name_start)
            .or(Some(after.len()))
        else {
            continue;
        };
        let marker = &after[name_start..name_end];
        if marker.is_empty() {
            continue;
        }
        let Some(closing_index) = lines[opening_index + 1..]
            .iter()
            .position(|(_, line)| line.trim() == marker)
            .map(|index| index + opening_index + 1)
        else {
            continue;
        };
        let body = &lines[opening_index + 1..closing_index];
        if body.is_empty() || body.iter().all(|(_, line)| line.trim().is_empty()) {
            continue;
        }
        let closing_indent =
            lines[closing_index].1.len() - lines[closing_index].1.trim_start().len();
        let opening_indent = opening_line.len() - opening_line.trim_start().len();
        let min_indent = body
            .iter()
            .filter(|(_, line)| !line.trim().is_empty())
            .map(|(_, line)| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(0);
        let desired = if modifier == "<<~" {
            closing_indent + width
        } else {
            opening_indent + width
        };
        let squish = active_support && opening_line.contains(".squish");
        let wrong = if modifier == "<<~" {
            min_indent != desired
        } else {
            min_indent == 0 || squish
        };
        if !wrong {
            continue;
        }
        if min_indent < desired
            && max.is_some_and(|max| {
                body.iter().any(|(_, line)| {
                    line.chars().count() + desired.saturating_sub(min_indent) > max
                })
            })
        {
            continue;
        }

        let message = if modifier == "<<~" {
            format!("Use {width} spaces for indentation in a heredoc.")
        } else {
            format!(
                "Use {width} spaces for indentation in a heredoc by using `<<~` instead of `{modifier}`."
            )
        };
        let body_start = lines[opening_index + 1].0;
        let closing_start = lines[closing_index].0;
        let mut edits = Vec::new();
        for (offset, line) in body {
            if line.trim().is_empty() {
                continue;
            }
            let indentation = line.len() - line.trim_start().len();
            let replacement = if indentation >= min_indent {
                " ".repeat(desired + indentation - min_indent)
            } else {
                " ".repeat(desired)
            };
            edits.push((*offset..*offset + indentation, replacement));
        }
        if modifier != "<<~" {
            edits.push((
                opening_offset + marker_at..opening_offset + marker_at + modifier.len(),
                "<<~".to_string(),
            ));
            edits.push((
                closing_start..closing_start + closing_indent,
                " ".repeat(opening_indent),
            ));
        }
        context.replace_many(message, body_start..closing_start, edits);
    }
}
