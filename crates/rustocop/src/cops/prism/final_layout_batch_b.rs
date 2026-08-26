use super::catalog_cop::custom;
use super::source_syntax::top_level_elements;
use super::*;
use regex::Regex;
use std::sync::OnceLock;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        custom(
            "Layout/SpaceAroundBlockParameters",
            space_around_block_parameters,
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
        Box::new(RedundantLineBreakCop),
        custom(
            "Layout/SpaceInsideArrayLiteralBrackets",
            array_literal_spacing,
        ),
    ];
    cops.extend(registry::cops());
    cops
}

fn space_around_block_parameters(context: &mut CopContext<'_, '_>) {
    #[derive(Default)]
    struct ParameterDelimiters {
        blocks: Vec<(usize, usize)>,
        lambdas: Vec<(usize, usize)>,
    }

    impl<'pr> Visit<'pr> for ParameterDelimiters {
        fn visit_block_parameters_node(&mut self, node: &ruby_prism::BlockParametersNode<'pr>) {
            if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
                self.blocks
                    .push((opening.start_offset(), closing.start_offset()));
            }
            ruby_prism::visit_block_parameters_node(self, node);
        }

        fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
            if let Some(parameters) = node
                .parameters()
                .and_then(|parameters| parameters.as_block_parameters_node())
            {
                if let (Some(opening), Some(closing)) =
                    (parameters.opening_loc(), parameters.closing_loc())
                {
                    self.lambdas
                        .push((opening.start_offset(), closing.start_offset()));
                }
            }
            ruby_prism::visit_lambda_node(self, node);
        }
    }

    let mut delimiters = ParameterDelimiters::default();
    delimiters.visit(&parse(context.source().as_bytes()).node());
    for (opening, closing) in delimiters
        .blocks
        .iter()
        .copied()
        .filter(|pair| !delimiters.lambdas.contains(pair))
    {
        enforce_parameter_spacing(context, opening, closing, true);
    }
    for (opening, closing) in delimiters.lambdas {
        enforce_parameter_spacing(context, opening, closing, false);
    }
}

fn enforce_parameter_spacing(
    context: &mut CopContext<'_, '_>,
    opening: usize,
    closing: usize,
    pipes: bool,
) {
    let source = context.source();
    if opening + 1 >= closing || source[opening + 1..closing].contains('\n') {
        return;
    }
    let space_style = context
        .config_value("EnforcedStyleInsidePipes")
        .unwrap_or("no_space")
        == "space";
    let inner = &source[opening + 1..closing];
    let first = inner.len() - inner.trim_start_matches([' ', '\t']).len();
    let last = inner.trim_end_matches([' ', '\t']).len();
    let elements = top_level_elements(source, opening + 1, closing);
    let Some(first_element) = elements.first().cloned() else {
        return;
    };
    let Some(last_element) = elements.last().cloned() else {
        return;
    };
    let last_non_space = opening + last;
    let trailing_comma = (last > 0 && source.as_bytes()[last_non_space] == b',')
        .then_some(last_non_space)
        .or_else(|| (source.as_bytes().get(closing - 1) == Some(&b',')).then_some(closing - 1));
    let wanted = usize::from(space_style);
    report_parameter_boundary(
        context,
        opening + 1,
        first,
        wanted,
        true,
        first_element,
        opening + 1,
    );
    report_parameter_boundary(
        context,
        opening + 1 + last,
        inner.len() - last,
        wanted,
        false,
        last_element,
        trailing_comma.unwrap_or(closing),
    );

    if !space_style && first > 1 {
        context.remove(
            "Extra space before block parameter detected.",
            opening + 1..opening + first - 1,
            opening + 1..opening + first - 1,
        );
    }

    let bytes = source.as_bytes();
    let mut at = opening + 1 + first;
    while at < closing {
        if bytes[at] == b',' {
            let whitespace_start = at + 1;
            let whitespace_end = whitespace_start
                + source[whitespace_start..closing]
                    .bytes()
                    .take_while(|byte| matches!(byte, b' ' | b'\t'))
                    .count();
            if whitespace_end > whitespace_start + 1 && whitespace_end < closing {
                context.remove(
                    "Extra space before block parameter detected.",
                    whitespace_start..whitespace_end - 1,
                    whitespace_start..whitespace_end - 1,
                );
            }
            at = whitespace_end;
        } else {
            at += 1;
        }
    }

    if pipes {
        let after = closing + 1;
        if after < source.len()
            && !source.as_bytes()[after].is_ascii_whitespace()
            && !matches!(source.as_bytes()[after], b'}' | b';')
        {
            context.insert(
                "Space after closing `|` missing.",
                closing..closing + 1,
                after,
                " ",
            );
        }
    }
}

fn report_parameter_boundary(
    context: &mut CopContext<'_, '_>,
    start: usize,
    actual: usize,
    wanted: usize,
    opening: bool,
    missing_offense: std::ops::Range<usize>,
    insertion: usize,
) {
    if actual == wanted {
        return;
    }
    let side = if opening {
        "before first"
    } else {
        "after last"
    };
    if actual < wanted {
        context.insert(
            format!("Space {side} block parameter missing."),
            missing_offense,
            insertion,
            " ",
        );
    } else {
        let remove = if opening {
            start..start + actual - wanted
        } else {
            start + wanted..start + actual
        };
        let adjective = if wanted == 0 { "Space" } else { "Extra space" };
        context.remove(
            format!("{adjective} {side} block parameter detected."),
            remove.clone(),
            remove,
        );
    }
}

struct RedundantLineBreakCop;

#[derive(Default)]
struct RedundantLineBreakState {
    comment_ranges: Option<Vec<std::ops::Range<usize>>>,
    heredoc_ranges: Option<Vec<std::ops::Range<usize>>>,
}

impl Cop for RedundantLineBreakCop {
    fn name(&self) -> &'static str {
        "Layout/RedundantLineBreak"
    }

    fn investigation_state(&self) -> Box<dyn Any> {
        Box::new(RedundantLineBreakState::default())
    }

    fn on_node_with_state<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
        state: &mut dyn Any,
    ) {
        let state = state
            .downcast_mut::<RedundantLineBreakState>()
            .expect("redundant line break investigation state");
        let comment_ranges = state
            .comment_ranges
            .get_or_insert_with(|| SourceFile::new(source).comment_ranges());
        let heredoc_ranges = state
            .heredoc_ranges
            .get_or_insert_with(|| SourceFile::new(source).heredoc_ranges());
        self.inspect_node(
            node,
            ancestors,
            source,
            context,
            comment_ranges,
            heredoc_ranges,
        );
    }
}

impl RedundantLineBreakCop {
    fn inspect_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
        comment_ranges: &[std::ops::Range<usize>],
        heredoc_ranges: &[std::ops::Range<usize>],
    ) {
        if redundant_assignment_wrapper(node) {
            if node.as_local_variable_write_node().is_some() && source.ends_with("%\n\n") {
                return;
            }
            let location = node.location();
            let range = location.start_offset()..location.end_offset();
            let mut reporter = context.cop_context(self.name(), source, ancestors);
            report_redundant_line_break(
                &mut reporter,
                node,
                range,
                comment_ranges,
                heredoc_ranges,
            );
            return;
        }
        let Some(call) = node.as_call_node() else {
            return;
        };
        let boundary = ancestors
            .iter()
            .rposition(|ancestor| ancestor.as_block_node().is_some())
            .map_or(0, |index| index + 1);
        let scoped = &ancestors[boundary..];
        if scoped
            .last()
            .is_some_and(|ancestor| ancestor.as_call_node().is_some())
        {
            return;
        }

        let mut reporter = context.cop_context(self.name(), source, ancestors);
        let mut call_range = call.location().start_offset()..call.location().end_offset();
        if let Some(block) = call.block().and_then(|block| block.as_block_node()) {
            let convertible = call.opening_loc().is_some() || argument_count(&call) == 0;
            if convertible {
                let opening = block.opening_loc();
                let closing = block.closing_loc();
                let multiline = source[opening.start_offset()..closing.end_offset()].contains('\n');
                if multiline && !reporter.config_bool("InspectBlocks", false) {
                    return;
                }
            } else {
                call_range.end = block.location().start_offset();
                while call_range.end > call_range.start
                    && source.as_bytes()[call_range.end - 1].is_ascii_whitespace()
                {
                    call_range.end -= 1;
                }
            }
        }
        let expanded = scoped
            .iter()
            .find(|ancestor| redundant_binary_wrapper(ancestor))
            .map(|ancestor| {
                (
                    ancestor,
                    ancestor.location().start_offset()..ancestor.location().end_offset(),
                )
            });
        if let Some(assignment) = scoped
            .iter()
            .find(|ancestor| redundant_assignment_wrapper(ancestor))
        {
            let location = assignment.location();
            let range = location.start_offset()..location.end_offset();
            if redundant_replacement(
                &reporter,
                assignment,
                &range,
                comment_ranges,
                heredoc_ranges,
            )
            .is_some()
            {
                return;
            }
        }
        if let Some((expanded_node, range)) = expanded {
            let binary = scoped.iter().any(|ancestor| {
                ancestor.as_and_node().is_some() || ancestor.as_or_node().is_some()
            });
            if binary && !binary_operator_precedes_backslash(source, &range) {
                return;
            }
            if binary && call_range.start != range.start {
                return;
            }
            if range.start <= call_range.start && call_range.end <= range.end {
                let reported = report_redundant_line_break(
                    &mut reporter,
                    expanded_node,
                    range,
                    comment_ranges,
                    heredoc_ranges,
                );
                if binary || reported {
                    return;
                }
            }
        }
        if source[call_range.clone()].contains('\n') {
            report_redundant_line_break(
                &mut reporter,
                node,
                call_range,
                comment_ranges,
                heredoc_ranges,
            );
        }
    }
}

fn redundant_binary_wrapper(node: &Node<'_>) -> bool {
    node.as_and_node().is_some() || node.as_or_node().is_some()
}

fn redundant_assignment_wrapper(node: &Node<'_>) -> bool {
    node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
        || node.as_multi_write_node().is_some()
        || node.as_local_variable_or_write_node().is_some()
        || node.as_local_variable_and_write_node().is_some()
        || node.as_instance_variable_or_write_node().is_some()
        || node.as_instance_variable_and_write_node().is_some()
        || node.as_class_variable_or_write_node().is_some()
        || node.as_class_variable_and_write_node().is_some()
        || node.as_global_variable_or_write_node().is_some()
        || node.as_global_variable_and_write_node().is_some()
        || node.as_constant_or_write_node().is_some()
        || node.as_constant_and_write_node().is_some()
        || node.as_constant_path_or_write_node().is_some()
        || node.as_constant_path_and_write_node().is_some()
        || node.as_index_or_write_node().is_some()
        || node.as_index_and_write_node().is_some()
        || node.as_call_or_write_node().is_some()
        || node.as_call_and_write_node().is_some()
        || node.as_local_variable_operator_write_node().is_some()
        || node.as_instance_variable_operator_write_node().is_some()
        || node.as_class_variable_operator_write_node().is_some()
        || node.as_global_variable_operator_write_node().is_some()
        || node.as_constant_operator_write_node().is_some()
        || node.as_constant_path_operator_write_node().is_some()
        || node.as_index_operator_write_node().is_some()
        || node.as_call_operator_write_node().is_some()
}

fn binary_operator_precedes_backslash(source: &str, range: &std::ops::Range<usize>) -> bool {
    source[range.clone()].lines().any(|line| {
        let line = line.trim_end();
        line.ends_with('\\')
            && (line[..line.len() - 1].trim_end().ends_with("&&")
                || line[..line.len() - 1].trim_end().ends_with("||"))
    })
}

fn report_redundant_line_break(
    context: &mut CopContext<'_, '_>,
    node: &Node<'_>,
    range: std::ops::Range<usize>,
    comment_ranges: &[std::ops::Range<usize>],
    heredoc_ranges: &[std::ops::Range<usize>],
) -> bool {
    let Some(replacement) =
        redundant_replacement(context, node, &range, comment_ranges, heredoc_ranges)
    else {
        return false;
    };
    context.replace(
        "Redundant line break detected.",
        range.clone(),
        range,
        replacement,
    );
    true
}

fn redundant_replacement(
    context: &CopContext<'_, '_>,
    node: &Node<'_>,
    range: &std::ops::Range<usize>,
    comment_ranges: &[std::ops::Range<usize>],
    heredoc_ranges: &[std::ops::Range<usize>],
) -> Option<String> {
    let source = context.source();
    let candidate = &source[range.clone()];
    if !candidate.contains('\n')
        || candidate.contains("rescue")
        || candidate.contains("ensure")
        || candidate.lines().any(|line| {
            line.split_ascii_whitespace()
                .any(|word| matches!(word, "if" | "unless" | "case" | "begin" | "def"))
        })
        || multiline_quoted_literal(candidate)
        || contains_multiline_unsafe_syntax(node, range, heredoc_ranges)
        || index_access_chained_across_line(candidate)
        || comment_within(context.source_file(), range, comment_ranges)
        || operator_after_line_break(candidate)
    {
        return None;
    }
    let inspect_blocks = context.config_bool("InspectBlocks", false);
    let multiline_block = contains_multiline_block(node, range);
    if multiline_block && !inspect_blocks {
        return None;
    }
    if multiline_block && block_has_multiple_body_lines(candidate) {
        return None;
    }
    if single_line_block_chain_takes_precedence(candidate, context) {
        return None;
    }
    let replacement = redundant_single_line(candidate)?;
    let line_length_enabled =
        context.related_config_value("Layout/LineLength", "Enabled") != Some("false");
    let max = context
        .related_config_value("Layout/LineLength", "Max")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120);
    let physical_lines = context.source_file().full_line_range(range.clone());
    let physical_source = &source[physical_lines];
    let leading = physical_source
        .chars()
        .take_while(|character| character.is_whitespace() && *character != '\n')
        .count();
    let physical_single_line = redundant_single_line(physical_source).unwrap_or_default();
    if line_length_enabled && leading + physical_single_line.chars().count() > max {
        return None;
    }
    Some(replacement)
}

fn contains_multiline_block(node: &Node<'_>, candidate: &std::ops::Range<usize>) -> bool {
    struct MultilineBlock<'a> {
        candidate: &'a std::ops::Range<usize>,
        found: bool,
    }

    impl<'pr> Visit<'pr> for MultilineBlock<'_> {
        fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
            let location = node.location();
            if location.start_offset() < self.candidate.start
                || location.end_offset() > self.candidate.end
            {
                ruby_prism::visit_block_node(self, node);
                return;
            }
            let opening = node.opening_loc();
            let closing = node.closing_loc();
            let range = opening.start_offset()..closing.end_offset();
            if location.as_slice()
                [range.start - location.start_offset()..range.end - location.start_offset()]
                .contains(&b'\n')
            {
                self.found = true;
                return;
            }
            ruby_prism::visit_block_node(self, node);
        }

        fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
            let location = node.location();
            if location.start_offset() < self.candidate.start
                || location.end_offset() > self.candidate.end
            {
                ruby_prism::visit_lambda_node(self, node);
                return;
            }
            let opening = node.opening_loc();
            let closing = node.closing_loc();
            let range = opening.start_offset()..closing.end_offset();
            if location.as_slice()
                [range.start - location.start_offset()..range.end - location.start_offset()]
                .contains(&b'\n')
            {
                self.found = true;
                return;
            }
            ruby_prism::visit_lambda_node(self, node);
        }
    }

    let mut block = MultilineBlock {
        candidate,
        found: false,
    };
    block.visit(node);
    block.found
}

fn contains_multiline_unsafe_syntax(
    node: &Node<'_>,
    candidate: &std::ops::Range<usize>,
    heredoc_ranges: &[std::ops::Range<usize>],
) -> bool {
    // Prism does not expose a heredoc string as a descendant of every call
    // whose source range contains its opener. Cache the file's heredoc ranges
    // and recover the sliced-parser behavior by matching the opener offset.
    if heredoc_ranges
        .iter()
        .any(|heredoc| candidate.start <= heredoc.start && heredoc.start < candidate.end)
    {
        return true;
    }

    struct UnsafeSyntax<'a> {
        candidate: &'a std::ops::Range<usize>,
        found: bool,
    }

    impl UnsafeSyntax<'_> {
        fn contains(&self, location: ruby_prism::Location<'_>) -> bool {
            self.candidate.start <= location.start_offset()
                && location.end_offset() <= self.candidate.end
        }
    }

    impl<'pr> Visit<'pr> for UnsafeSyntax<'_> {
        fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
            if self.contains(node.location()) {
                self.found = true;
            } else {
                ruby_prism::visit_if_node(self, node);
            }
        }

        fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
            if self.contains(node.location()) {
                self.found = true;
            } else {
                ruby_prism::visit_unless_node(self, node);
            }
        }

        fn visit_case_node(&mut self, node: &ruby_prism::CaseNode<'pr>) {
            if self.contains(node.location()) {
                self.found = true;
            } else {
                ruby_prism::visit_case_node(self, node);
            }
        }

        fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode<'pr>) {
            if self.contains(node.location()) {
                self.found = true;
            } else {
                ruby_prism::visit_case_match_node(self, node);
            }
        }

        fn visit_parentheses_node(&mut self, node: &ruby_prism::ParenthesesNode<'pr>) {
            if self.contains(node.location()) && node.location().as_slice().contains(&b'\n') {
                self.found = true;
                return;
            }
            ruby_prism::visit_parentheses_node(self, node);
        }

        fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
            if self.contains(node.location())
                && (node.unescaped().contains(&b'\n')
                    || node
                        .opening_loc()
                        .is_some_and(|loc| loc.as_slice().starts_with(b"<<")))
            {
                self.found = true;
            }
            ruby_prism::visit_string_node(self, node);
        }

        fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode<'pr>) {
            if self.contains(node.location()) && node.location().as_slice().contains(&b'\n') {
                self.found = true;
            }
            ruby_prism::visit_symbol_node(self, node);
        }

        fn visit_regular_expression_node(&mut self, node: &ruby_prism::RegularExpressionNode<'pr>) {
            if self.contains(node.location()) && node.location().as_slice().contains(&b'\n') {
                self.found = true;
            }
            ruby_prism::visit_regular_expression_node(self, node);
        }

        fn visit_interpolated_regular_expression_node(
            &mut self,
            node: &ruby_prism::InterpolatedRegularExpressionNode<'pr>,
        ) {
            if self.contains(node.location()) && node.location().as_slice().contains(&b'\n') {
                self.found = true;
            }
            ruby_prism::visit_interpolated_regular_expression_node(self, node);
        }
    }

    let mut unsafe_syntax = UnsafeSyntax {
        candidate,
        found: false,
    };
    unsafe_syntax.visit(node);
    unsafe_syntax.found
}

fn multiline_quoted_literal(source: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for byte in source.bytes() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if let Some(delimiter) = quote {
            if byte == delimiter {
                quote = None;
            } else if byte == b'\n' {
                return true;
            }
        } else if matches!(byte, b'\'' | b'"') {
            quote = Some(byte);
        }
    }
    false
}

fn index_access_chained_across_line(source: &str) -> bool {
    source.lines().collect::<Vec<_>>().windows(2).any(|lines| {
        lines[0].trim_end_matches([' ', '\t', '\\']).ends_with(']')
            && lines[1].trim_start().starts_with('[')
    })
}

fn comment_within(
    file: SourceFile<'_>,
    range: &std::ops::Range<usize>,
    comment_ranges: &[std::ops::Range<usize>],
) -> bool {
    let lines = file.full_line_range(range.clone());
    comment_ranges
        .iter()
        .any(|comment| lines.start <= comment.start && comment.start < lines.end)
}

fn block_has_multiple_body_lines(source: &str) -> bool {
    let lines: Vec<_> = source.lines().collect();
    lines.len() > 2
        && lines[1..lines.len() - 1]
            .iter()
            .filter(|line| !line.trim().is_empty())
            .count()
            > 1
}

fn operator_after_line_break(source: &str) -> bool {
    source.lines().skip(1).any(|line| {
        matches!(
            line.trim_start().as_bytes(),
            [b'&', b'&', ..] | [b'|', b'|', ..]
        )
    })
}

fn single_line_block_chain_takes_precedence(source: &str, context: &CopContext<'_, '_>) -> bool {
    if context.related_config_value("Layout/SingleLineBlockChain", "Enabled") == Some("false") {
        return false;
    }
    source.lines().collect::<Vec<_>>().windows(2).any(|lines| {
        lines[0].contains('{')
            && lines[0].trim_end().ends_with('}')
            && matches!(
                lines[1].trim_start().as_bytes(),
                [b'.', ..] | [b'&', b'.', ..]
            )
    })
}

fn redundant_single_line(source: &str) -> Option<String> {
    static DOUBLE_SINGLE: OnceLock<Regex> = OnceLock::new();
    static SINGLE_DOUBLE: OnceLock<Regex> = OnceLock::new();
    static DOUBLE_DOUBLE: OnceLock<Regex> = OnceLock::new();
    static SINGLE_SINGLE: OnceLock<Regex> = OnceLock::new();
    static CHAIN: OnceLock<Regex> = OnceLock::new();
    static BREAK: OnceLock<Regex> = OnceLock::new();
    let mut output = source.to_string();
    output = DOUBLE_SINGLE
        .get_or_init(|| Regex::new("\" *\\\\\\n\\s*'").expect("valid regex"))
        .replace_all(&output, "\" + '")
        .into_owned();
    output = SINGLE_DOUBLE
        .get_or_init(|| Regex::new("' *\\\\\\n\\s*\"").expect("valid regex"))
        .replace_all(&output, "' + \"")
        .into_owned();
    output = DOUBLE_DOUBLE
        .get_or_init(|| Regex::new("\" *\\\\\\n\\s*\"").expect("valid regex"))
        .replace_all(&output, "")
        .into_owned();
    output = SINGLE_SINGLE
        .get_or_init(|| Regex::new("' *\\\\\\n\\s*'").expect("valid regex"))
        .replace_all(&output, "")
        .into_owned();
    output = CHAIN
        .get_or_init(|| Regex::new(r"\n\s*(&?\.\w)").expect("valid regex"))
        .replace_all(&output, "$1")
        .into_owned();
    output = BREAK
        .get_or_init(|| Regex::new(r"\s*\\?\n\s*").expect("valid regex"))
        .replace_all(&output, " ")
        .trim()
        .to_string();
    Some(output)
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
    let left_horizontal_end = opening
        + 1
        + inside
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
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
    if (!left_newline || compact_left || !require_left_space && left_horizontal_end > opening + 1)
        && !(style == "no_space" && comment_after_opening)
    {
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
            } else if left_newline && left_horizontal_end > opening + 1 {
                opening + 1..left_horizontal_end
            } else {
                opening + 1..left_ws_end
            };
            let edit_end = if compact_left {
                left_ws_end
            } else if left_newline {
                left_horizontal_end
            } else {
                left_ws_end
            };
            let mut edits = vec![(opening + 1..edit_end, String::new())];
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

#[allow(clippy::too_many_lines)]
fn end_alignment(context: &mut CopContext<'_, '_>) {
    #[derive(Clone)]
    struct Opening {
        line: usize,
        column: usize,
        correction_column: usize,
        source: String,
        relevant: bool,
    }

    let start_of_line = context
        .related_config_value("Layout/BeginEndAlignment", "EnforcedStyleAlignWith")
        == Some("start_of_line")
        && context.related_config_explicit("Layout/BeginEndAlignment", "EnforcedStyleAlignWith")
        && context.related_config_value("Layout/BeginEndAlignment", "Enabled") != Some("false");
    let comment_ranges = context.source_file().comment_ranges();
    let excluded = context
        .source_file()
        .literal_ranges()
        .into_iter()
        .chain(comment_ranges.iter().cloned())
        .collect::<Vec<_>>();
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut stack: Vec<Opening> = Vec::new();
    for (line_index, &(offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        let indentation = line.len() - trimmed.len();
        if trimmed == "end"
            || trimmed.starts_with("end ")
            || trimmed.starts_with("end.")
            || trimmed.starts_with("end&.")
        {
            stack.pop();
            continue;
        }

        for keyword in ["rescue", "ensure"] {
            let relative = line.match_indices(keyword).find_map(|(relative, _)| {
                let before = &line[..relative];
                let boundary_before = before.as_bytes().last();
                let boundary_after = line.as_bytes().get(relative + keyword.len());
                let token = boundary_before
                    .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
                    && boundary_after
                        .is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_');
                let clause = before.trim().is_empty() || before.trim_end().ends_with(';');
                let absolute = offset + relative;
                (token
                    && clause
                    && !excluded
                        .iter()
                        .any(|range| range.start <= absolute && absolute < range.end))
                .then_some(relative)
            });
            let Some(relative) = relative else {
                continue;
            };
            let before = &line[..relative];
            let Some(opening) = stack.iter().rev().find(|opening| {
                opening.relevant
                    && !lines[opening.line..line_index]
                        .iter()
                        .any(|(_, candidate)| {
                            candidate.trim() == "end"
                                && candidate.len() - candidate.trim_start().len() == opening.column
                        })
            }) else {
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
                    " ".repeat(opening.correction_column),
                );
            } else {
                context.report(message, offense);
            }
        }

        let comment_at = comment_ranges
            .iter()
            .filter(|range| offset <= range.start && range.start < offset + line.len())
            .map(|range| range.start - offset)
            .min();
        let code = line[..comment_at.unwrap_or(line.len())].trim_end();
        let relevant = if let Some(begin_at) = code.find("begin") {
            let boundary = code.as_bytes().get(begin_at.wrapping_sub(1));
            let after = code.as_bytes().get(begin_at + "begin".len());
            if (begin_at == 0
                || boundary.is_some_and(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_'))
                && after.is_none_or(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
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
                .find(['<', ';'])
                .map_or(code.len(), |at| indentation + at);
            Some((indentation, code[indentation..end].trim_end().to_string()))
        } else if code.ends_with(" do") || code.contains(" do |") {
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
            let chained_call = (code.ends_with(" do") || code.contains(" do |"))
                && (trimmed.starts_with('.')
                    || line_index > 0 && lines[line_index - 1].1.trim_end().ends_with('.'));
            let correction_column = if chained_call && line_index > 0 {
                let previous = lines[line_index - 1].1;
                previous.len() - previous.trim_start().len()
            } else {
                column
            };
            stack.push(Opening {
                line: line_index + 1,
                column,
                correction_column,
                source,
                relevant: true,
            });
        } else if starts_end_delimited_construct(trimmed) {
            stack.push(Opening {
                line: line_index + 1,
                column: indentation,
                correction_column: indentation,
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
        .any(|keyword| {
            line.starts_with(keyword)
                || line.contains(&format!("= {keyword}"))
                || line.contains(&format!("||= {keyword}"))
                || line.contains(&format!("&&= {keyword}"))
        })
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
        let call_location = call.location();
        let hash_pair = ancestors
            .iter()
            .any(|ancestor| ancestor.as_assoc_node().is_some());
        if !hash_pair
            && ancestors.iter().any(|ancestor| {
                ancestor.as_parentheses_node().is_some_and(|group| {
                    let group = group.location();
                    group.start_offset() < call_location.start_offset()
                        && call_location.end_offset() < group.end_offset()
                }) || ancestor.as_call_node().is_some_and(|parent| {
                    parent.arguments().is_some_and(|arguments| {
                        let arguments = arguments.location();
                        (arguments.start_offset() < call_location.start_offset()
                            && call_location.end_offset() < arguments.end_offset())
                            || (parent.call_operator_loc().is_some()
                                && parent.opening_loc().is_some()
                                && arguments.start_offset() <= call_location.start_offset()
                                && call_location.end_offset() <= arguments.end_offset())
                    })
                })
            })
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

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
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
    let receiver_has_block = receiver.as_call_node().is_some_and(|receiver_call| {
        receiver_call
            .block()
            .and_then(|block| block.as_block_node())
            .is_some()
    });
    let call_has_literal_block = call
        .block()
        .and_then(|block| block.as_block_node())
        .is_some();
    let multiline_block_dot = multiline_block_chain_dot_column(&receiver, file);
    let expression_indent = continuation_expression_indent(receiver_range.start, file);
    let base_receiver = base_call_receiver(receiver);
    let multiline_block_dot =
        multiline_block_dot.or_else(|| prior_block_end_chain_dot_column(0, rhs.start, file));
    let pair_key = multiline_method_pair_key(context, file);
    let pair_key_column = pair_key.map(|(column, _)| column);
    let base_prefix = &context.source()[file.line_start(base_receiver.location().start_offset())
        ..base_receiver.location().start_offset()];
    if style == "aligned"
        && (multiline_block_dot == Some(actual)
            || (base_prefix.trim_start().starts_with('.')
                && base_receiver
                    .as_call_node()
                    .is_some_and(|base| base.receiver().is_none() && base.opening_loc().is_some()))
            || (actual
                == file
                    .indentation(base_receiver.location().start_offset())
                    .len()
                    + width
                && pair_key_column.is_none()
                && !multiline_method_assignment_context(call, file)
                && !alignment_context_before(base_receiver.location().start_offset(), file)
                && base_receiver
                    .as_call_node()
                    .is_some_and(|base| base.receiver().is_none() && base.opening_loc().is_some())))
    {
        return;
    }
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
    if call_has_literal_block
        && receiver_has_block
        && first_same_line_chain_rhs(call, file).is_none()
        && actual == expression_indent + width
    {
        return;
    }
    if pair_key_column.is_none()
        && call_has_literal_block
        && immediately_follows_continuation_at_column(rhs.start, actual, file)
    {
        return;
    }
    if call_has_literal_block && immediately_follows_block_end(rhs.start, file) {
        return;
    }
    if style == "aligned"
        && prior_continuation_at_column(call, rhs.start, actual, file)
        && (call
            .call_operator_loc()
            .zip(call.message_loc())
            .is_none_or(|(operator, selector)| {
                file.same_line(operator.start_offset(), selector.start_offset())
                    || alignment_context_before(base_receiver.location().start_offset(), file)
                    || actual == expression_indent + width
            }))
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
                .or_else(|| {
                    trailing_dot_alignment_base(
                        rhs.start,
                        base_receiver.location().start_offset(),
                        file,
                    )
                })
                .or_else(|| multiline_call_syntactic_base(&base_receiver, context.source_file()))
                .or_else(|| multiline_operation_alignment_base(receiver_range.clone(), file))
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
            let location = base_receiver.location();
            (
                expression_indent,
                Some((location.start_offset(), location.end_offset())),
                false,
            )
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

fn multiline_block_chain_dot_column(node: &Node<'_>, file: SourceFile<'_>) -> Option<usize> {
    let mut current = node.as_call_node();
    while let Some(call) = current {
        if let Some(block) = call.block().filter(|block| {
            !file.same_line(
                block.location().start_offset(),
                block.location().end_offset(),
            )
        }) {
            let end = block.location().end_offset();
            let line_end = file.line_end(end.saturating_sub(1));
            if let Some(dot) = file.as_str()[end..line_end].find('.') {
                return Some(file.column(end + dot));
            }
        }
        current = call.receiver().and_then(|receiver| receiver.as_call_node());
    }
    None
}

fn prior_block_end_chain_dot_column(
    call_start: usize,
    rhs_start: usize,
    file: SourceFile<'_>,
) -> Option<usize> {
    file.lines()
        .filter(|(offset, _)| *offset >= file.line_start(call_start) && *offset < rhs_start)
        .filter_map(|(offset, line)| {
            let trimmed = line.trim_start();
            let after_end = trimmed.strip_prefix("end")?;
            let dot = after_end.find('.')?;
            Some(file.column(offset + (line.len() - trimmed.len()) + 3 + dot))
        })
        .last()
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

fn trailing_dot_alignment_base(
    rhs_start: usize,
    context_start: usize,
    file: SourceFile<'_>,
) -> Option<(usize, usize)> {
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
    if !alignment_context_before(context_start, file) {
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
    let mut operations = MultilineOperationRanges {
        source: context.source().as_bytes(),
        ranges: std::collections::HashMap::new(),
    };
    operations.visit(&parse(context.source().as_bytes()).node());
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
        let condition = ["if", "elsif", "unless", "while", "until"]
            .into_iter()
            .find(|keyword| trimmed_first.starts_with(&format!("{keyword} ")));
        let modifier_condition = ["if", "unless", "while", "until"]
            .into_iter()
            .find(|keyword| trimmed_first.contains(&format!(" {keyword} ")));
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
        let offense_start = lines[index].0 + actual;
        let Some(offense_end) = operations.ranges.get(&offense_start).copied() else {
            continue;
        };
        if actual == desired {
            continue;
        }

        let noun = if let Some(keyword) = condition.or(modifier_condition) {
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
        context.replace(
            message,
            offense_start..offense_end,
            lines[index].0..offense_start,
            " ".repeat(desired),
        );
    }
}

struct MultilineOperationRanges<'a> {
    source: &'a [u8],
    ranges: std::collections::HashMap<usize, usize>,
}

impl MultilineOperationRanges<'_> {
    fn record(&mut self, left: Node<'_>, right: Node<'_>) {
        let right_location = right.location();
        let start = right_location.start_offset();
        let line_start = self.source[..start]
            .iter()
            .rposition(|byte| *byte == b'\n')
            .map_or(0, |newline| newline + 1);
        if left.location().end_offset() <= line_start
            && self.source[line_start..start]
                .iter()
                .all(|byte| byte.is_ascii_whitespace())
        {
            self.ranges.insert(start, right_location.end_offset());
        }
    }
}

impl<'pr> Visit<'pr> for MultilineOperationRanges<'_> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if node.call_operator_loc().is_none()
            && matches!(
                node.name().as_slice(),
                b"+" | b"-" | b"*" | b"/" | b"%" | b"<<" | b">>" | b"&" | b"|" | b"^"
            )
        {
            if let (Some(left), Some(right)) = (node.receiver(), node.first_argument()) {
                self.record(left, right);
            }
        }
        ruby_prism::visit_call_node(self, node);
    }

    fn visit_and_node(&mut self, node: &ruby_prism::AndNode<'pr>) {
        self.record(node.left(), node.right());
        ruby_prism::visit_and_node(self, node);
    }

    fn visit_or_node(&mut self, node: &ruby_prism::OrNode<'pr>) {
        self.record(node.left(), node.right());
        ruby_prism::visit_or_node(self, node);
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
    trimmed
        .find(['\'', '"'])
        .filter(|at| *at > 0)
        .map(|at| line.len() - trimmed.len() + at)
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

#[allow(clippy::too_many_lines)]
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
                    if index > 0 && !on_own_line {
                        return (*entry, delta);
                    }
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

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
fn operator_spacing(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_owned();
    let exponent_space = context
        .config_value("EnforcedStyleForExponentOperator")
        .is_some_and(|style| style == "space");
    let rational_space = context
        .config_value("EnforcedStyleForRationalLiterals")
        .is_some_and(|style| style == "space");
    let allow_alignment = context
        .config_value("AllowForAlignment")
        .is_none_or(|value| value != "false");
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
    // Literal ranges can overlap (notably heredocs followed by calls containing
    // regexp arguments). RuboCop's previous scanner selected the narrowest
    // containing range. Keep that exact choice with a sweep-line heap instead of
    // testing every source byte against every literal.
    let mut literal_events = context
        .source_file()
        .literal_ranges()
        .into_iter()
        .enumerate()
        .map(|(order, range)| (range.start, order, range.end))
        .collect::<Vec<_>>();
    literal_events.sort_by_key(|&(start, order, _)| (start, order));
    let mut active_literals = std::collections::BinaryHeap::new();
    let mut comment_ranges = context.source_file().comment_ranges();
    comment_ranges.sort_by_key(|range| range.start);
    let unary_operator_offsets = unary_operator_offsets(&source);
    let hash_pair_starts = hash_pair_starts(&source);
    let mut literal_index = 0;
    let mut comment_index = 0;

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
        let code_end = line.len();
        let code = &line[..code_end];
        let mut index = 0;
        let mut quote = None;
        let mut ternary_depth = 0usize;
        while index < code.len() {
            let absolute = line_offset + index;
            while comment_ranges
                .get(comment_index)
                .is_some_and(|range| range.end <= absolute)
            {
                comment_index += 1;
            }
            if let Some(range) = comment_ranges
                .get(comment_index)
                .filter(|range| range.start <= absolute && absolute < range.end)
            {
                index = range.end.saturating_sub(line_offset).min(code.len());
                continue;
            }
            while literal_events
                .get(literal_index)
                .is_some_and(|&(start, _, _)| start <= absolute)
            {
                let (start, order, end) = literal_events[literal_index];
                active_literals.push(std::cmp::Reverse((end - start, order, start, end)));
                literal_index += 1;
            }
            while active_literals
                .peek()
                .is_some_and(|entry| entry.0.3 <= absolute)
            {
                active_literals.pop();
            }
            if let Some(entry) = active_literals.peek() {
                let (_, _, start, end) = entry.0;
                let range = start..end;
                let literal = &source[range.clone()];
                let heredoc_tail = literal.starts_with("<<")
                    && line_offset <= range.start
                    && range.start < line_offset + line.len()
                    && literal.find('\n').is_some();
                if heredoc_tail {
                    let marker_end = range.start
                        + literal
                            .bytes()
                            .take_while(|byte| {
                                !byte.is_ascii_whitespace() && !matches!(byte, b',' | b')')
                            })
                            .count();
                    if absolute < marker_end {
                        index = marker_end.saturating_sub(line_offset).min(code.len());
                        continue;
                    }
                } else {
                    index = range.end.saturating_sub(line_offset).min(code.len());
                    continue;
                }
            }
            if !code.is_char_boundary(index) {
                index += 1;
                continue;
            }
            let byte = code.as_bytes()[index];
            let character_width = code[index..].chars().next().map_or(1, char::len_utf8);
            if let Some(delimiter) = quote {
                if byte == b'\\' {
                    index += 1;
                    if index < code.len() {
                        index += code[index..].chars().next().map_or(1, char::len_utf8);
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
                    at + code[at..].chars().next().map_or(1, char::len_utf8)
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
                if comment_ranges
                    .iter()
                    .any(|comment| comment.start == line_offset + right_end)
                {
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
                        hash_pair_starts.get(&(line_offset + index)).copied(),
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

        fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode<'pr>) {
            self.0.insert(node.operator_loc().start_offset());
            ruby_prism::visit_rest_parameter_node(self, node);
        }

        fn visit_keyword_rest_parameter_node(
            &mut self,
            node: &ruby_prism::KeywordRestParameterNode<'pr>,
        ) {
            self.0.insert(node.operator_loc().start_offset());
            ruby_prism::visit_keyword_rest_parameter_node(self, node);
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

fn hash_pair_starts(source: &str) -> std::collections::HashMap<usize, usize> {
    #[derive(Default)]
    struct HashPairs(std::collections::HashMap<usize, usize>);

    impl<'pr> ruby_prism::Visit<'pr> for HashPairs {
        fn visit_assoc_node(&mut self, node: &ruby_prism::AssocNode<'pr>) {
            if let Some(operator) = node.operator_loc() {
                self.0
                    .insert(operator.start_offset(), node.location().start_offset());
            }
            ruby_prism::visit_assoc_node(self, node);
        }
    }

    let parsed = ruby_prism::parse(source.as_bytes());
    let mut pairs = HashPairs::default();
    pairs.visit(&parsed.node());
    pairs.0
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
        } else if quote.is_none()
            && (matches!(byte, b'\'' | b'"' | b'`')
                || byte == b'/' && slash_starts_regexp(line, index))
        {
            quote = Some(byte);
        } else if quote.is_none() && byte == b'#' && line.as_bytes().get(index + 1) != Some(&b'{') {
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
    if operator == "<<"
        && source[end..].chars().next().is_some_and(|character| {
            character.is_ascii_uppercase() || matches!(character, '\'' | '"' | '`')
        })
    {
        return true;
    }
    if operator == "=" && source[..start].contains("def ") {
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
                .is_some_and(|word| matches!(word, "return" | "yield" | "when" | "in" | "rescue")))
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
            }))
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
    let (name_start, rest) = if let Some(rest) = trimmed.strip_prefix("def ") {
        (indentation + "def ".len(), rest)
    } else if let Some(definition) = source[..operator_start].rfind("def ") {
        let name_start = definition + "def ".len();
        return source[name_start..operator_start]
            .chars()
            .all(|character| !character.is_whitespace() && character != '(');
    } else {
        return false;
    };
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

#[allow(clippy::too_many_arguments)]
fn operator_alignment_is_allowed(
    source: &str,
    line_offset: usize,
    operator_start: usize,
    operator_end: usize,
    whitespace_start: usize,
    rhs_start: usize,
    operator: &str,
    operand_alignment_start: Option<usize>,
) -> bool {
    let current_line_end = source[line_offset..]
        .find('\n')
        .map_or(source.len(), |end| line_offset + end);
    let current_line_source = &source[line_offset..current_line_end];
    let operator_start_column = current_line_source[..operator_start].chars().count();
    let operator_end_column = current_line_source[..operator_end].chars().count();
    let rhs_start_column = current_line_source[..rhs_start].chars().count();
    let leading_excess = operator_start.saturating_sub(whitespace_start) > 1;
    let trailing_excess = rhs_start.saturating_sub(operator_end) > 1;
    let lhs = current_line_source[..whitespace_start].trim_end();
    let first_equal = (operator == "=")
        .then(|| {
            operator_layouts(current_line_source)
                .into_iter()
                .find(|layout| current_line_source[layout.0..layout.1].ends_with('='))
        })
        .flatten();
    let plain_assignment = matches!(operator, "=" | "||=" | "&&=")
        && (operator != "="
            || !(first_equal.is_some_and(|layout| layout.0 == operator_start)
                && (lhs.contains('.') || lhs.contains(']'))));

    let leading_aligned = !leading_excess
        || if plain_assignment {
            assignment_leading_alignment_is_allowed(source, line_offset, operator_end_column)
        } else {
            generic_operator_alignment_is_allowed(
                source,
                line_offset,
                operator_start_column,
                operator_end_column,
                operator,
            )
        };
    let (alignment_column, alignment_start) = operand_alignment_start
        .filter(|absolute| {
            (line_offset..=line_offset + current_line_source.len()).contains(absolute)
        })
        .map_or((rhs_start_column, rhs_start), |absolute| {
            (
                current_line_source[..absolute - line_offset]
                    .chars()
                    .count(),
                absolute - line_offset,
            )
        });
    let trailing_aligned = !trailing_excess
        || generic_rhs_alignment_is_allowed(source, line_offset, alignment_column, alignment_start);

    leading_aligned && trailing_aligned
}

fn alignment_search(
    source: &str,
    line_offset: usize,
    mut predicate: impl FnMut(&str) -> bool,
) -> bool {
    let lines = source.lines().collect::<Vec<_>>();
    let current = source[..line_offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let current_indent = lines[current].len() - lines[current].trim_start().len();
    let eligible = |line: &&str| !line.trim().is_empty() && !line.trim_start().starts_with('#');

    let preceding = lines[..current].iter().rev().find(|line| eligible(line));
    let subsequent = lines[current + 1..].iter().find(|line| eligible(line));
    if preceding.is_some_and(|line| predicate(line))
        || subsequent.is_some_and(|line| predicate(line))
    {
        return true;
    }

    let same_indent =
        |line: &&str| eligible(line) && line.len() - line.trim_start().len() == current_indent;
    lines[..current]
        .iter()
        .rev()
        .find(|line| same_indent(line))
        .is_some_and(|line| predicate(line))
        || lines[current + 1..]
            .iter()
            .find(|line| same_indent(line))
            .is_some_and(|line| predicate(line))
}

fn generic_operator_alignment_is_allowed(
    source: &str,
    line_offset: usize,
    operator_start_column: usize,
    operator_end_column: usize,
    operator: &str,
) -> bool {
    alignment_search(source, line_offset, |line| {
        operator_layouts(line).into_iter().any(|layout| {
            let candidate = &line[layout.0..layout.1];
            candidate == operator && line[..layout.0].chars().count() == operator_start_column
                || operator.ends_with('=')
                    && candidate.ends_with('=')
                    && line[..layout.1].chars().count() == operator_end_column
                || operator == "<<"
                    && candidate.ends_with('=')
                    && line[..layout.1].chars().count() == operator_end_column
                || operator.ends_with('=')
                    && candidate == "<<"
                    && line[..layout.1].chars().count() == operator_end_column
        })
    })
}

fn generic_rhs_alignment_is_allowed(
    source: &str,
    line_offset: usize,
    rhs_start_column: usize,
    rhs_start: usize,
) -> bool {
    let line_end = source[line_offset..]
        .find('\n')
        .map_or(source.len(), |end| line_offset + end);
    let current = &source[line_offset..line_end];
    alignment_search(source, line_offset, |line| {
        aligned_word_at_column(line, current, rhs_start_column, rhs_start)
    })
}

fn aligned_word_at_column(
    candidate: &str,
    current: &str,
    column: usize,
    current_rhs_start: usize,
) -> bool {
    let Some(candidate_at) = candidate
        .char_indices()
        .nth(column)
        .map(|(index, _)| index)
        .or_else(|| (candidate.chars().count() == column).then_some(candidate.len()))
    else {
        return false;
    };
    let Some(character) = candidate[candidate_at..].chars().next() else {
        return false;
    };
    let boundary = candidate[..candidate_at]
        .chars()
        .next_back()
        .is_some_and(char::is_whitespace)
        && !character.is_whitespace();
    if boundary {
        return true;
    }
    let operand = current[current_rhs_start..].trim_end();
    !operand.is_empty() && candidate[candidate_at..].starts_with(operand)
}

fn assignment_leading_alignment_is_allowed(
    source: &str,
    line_offset: usize,
    operator_end_column: usize,
) -> bool {
    let lines = source.lines().collect::<Vec<_>>();
    let current_line = source[..line_offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count();
    let current_indent = lines[current_line].len() - lines[current_line].trim_start().len();
    let relevant = |direction: isize| {
        let mut matches = Vec::new();
        let mut position = current_line as isize;
        let mut relevant_indent_at_level = true;
        while let Some(line) = usize::try_from(position)
            .ok()
            .and_then(|position| lines.get(position))
        {
            position += direction;
            let indent = line.len() - line.trim_start().len();
            let blank = line.trim().is_empty();
            if indent < current_indent && !blank || relevant_indent_at_level && blank {
                break;
            }
            if indent == current_indent {
                if let Some(layout) = operator_layouts(line)
                    .into_iter()
                    .find(|layout| line[layout.0..layout.1].ends_with('='))
                {
                    matches.push(line[..layout.1].chars().count());
                }
            }
            if !blank {
                relevant_indent_at_level = indent == current_indent;
            }
        }
        matches.get(1).copied()
    };

    let preceding = relevant(-1);
    preceding == Some(operator_end_column)
        || relevant(1).is_none_or(|column| column == operator_end_column)
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
        let character_width = code[index..].chars().next().map_or(1, char::len_utf8);
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                index += 1;
                if index < code.len() {
                    index += code[index..].chars().next().map_or(1, char::len_utf8);
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
                    at + code[at..].chars().next().map_or(1, char::len_utf8)
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

    let mut skip_until = 0;
    let mut block_comment = false;
    for (opening_index, (opening_offset, opening_line)) in lines.iter().copied().enumerate() {
        if opening_line.starts_with("=begin") {
            block_comment = true;
        }
        if block_comment {
            if opening_line.starts_with("=end") {
                block_comment = false;
            }
            continue;
        }
        if opening_index < skip_until
            && !opening_line
                .split("<<")
                .next()
                .is_some_and(inside_interpolation)
        {
            continue;
        }
        let openers = heredoc_openers(opening_line);
        let mut body_index = opening_index + 1;
        for (marker_at, modifier, marker) in openers {
            let Some(closing_index) = lines[body_index..]
                .iter()
                .position(|(_, line)| line.trim() == marker)
                .map(|index| index + body_index)
            else {
                continue;
            };
            let body = &lines[body_index..closing_index];
            body_index = closing_index + 1;
            skip_until = skip_until.max(body_index);
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
            let desired = opening_indent + width;
            let squish = active_support && opening_line.contains(".squish");
            let nested_interpolation = inside_interpolation(&opening_line[..marker_at]);
            let wrong = if modifier == "<<~" {
                min_indent != desired
            } else {
                min_indent == 0 || squish
            };
            if !wrong
                || (!nested_interpolation
                    && min_indent < desired
                    && max.is_some_and(|max| {
                        body.iter().any(|(_, line)| {
                            line.chars().count() + desired.saturating_sub(min_indent) > max
                        })
                    }))
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
            let body_start = lines[closing_index - body.len()].0;
            let closing_start = lines[closing_index].0;
            let mut edits = Vec::new();
            for (offset, line) in body {
                if line.trim().is_empty() {
                    if modifier != "<<~" {
                        edits.push((*offset..*offset + line.len(), " ".repeat(desired)));
                    }
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
}

fn heredoc_openers(line: &str) -> Vec<(usize, &'static str, &str)> {
    let mut openers = Vec::new();
    for (marker_at, _) in line.match_indices("<<") {
        let prefix = line[..marker_at].trim_start();
        if prefix.starts_with('#') && !prefix.starts_with("#{") {
            break;
        }
        if inside_quoted_literal(&line[..marker_at]) && !inside_interpolation(&line[..marker_at]) {
            continue;
        }
        if line[..marker_at].trim_end().ends_with('/') {
            continue;
        }
        let after = &line[marker_at + 2..];
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
        let quoted = quote.is_some();
        let name_start = usize::from(quoted);
        let name_end = if let Some(quote) = quote {
            after[name_start..]
                .bytes()
                .position(|byte| byte == quote)
                .map_or(after.len(), |at| at + name_start)
        } else {
            after[name_start..]
                .find(|character: char| !(character.is_ascii_alphanumeric() || character == '_'))
                .map_or(after.len(), |at| at + name_start)
        };
        let marker = &after[name_start..name_end];
        if !marker.is_empty() {
            openers.push((marker_at, modifier, marker));
        }
    }
    openers
}

fn inside_quoted_literal(source: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for byte in source.bytes() {
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' && quote.is_some() {
            escaped = true;
            continue;
        }
        if quote == Some(byte) {
            quote = None;
        } else if quote.is_none() && matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
        }
    }
    quote.is_some()
}

fn inside_interpolation(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut depth = 0usize;
    let mut index = 0usize;
    while index < bytes.len() {
        if bytes[index..].starts_with(b"#{") {
            depth += 1;
            index += 2;
            continue;
        }
        if bytes[index] == b'}' && depth > 0 {
            depth -= 1;
        }
        index += 1;
    }
    depth > 0
}
