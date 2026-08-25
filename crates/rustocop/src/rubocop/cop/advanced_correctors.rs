#![allow(clippy::too_many_arguments)]
// RuboCop 1.87.0 advanced corrector compatibility.
// Source: lib/rubocop/cop/correctors/alignment_corrector.rb
// Source SHA-256: f220a87f1b71afe7082e73961487be72a779b6cb835bddde95b913a6f666126d
// Source: lib/rubocop/cop/correctors/each_to_for_corrector.rb
// Source SHA-256: a47c43dacb20daa3e9538509b1a3a4750e06b18fbc71df898bfcb920fa9b9b4c
// Source: lib/rubocop/cop/correctors/for_to_each_corrector.rb
// Source SHA-256: c50a87270a5d4048244f69bb3835d88f7a2be49bc1b698ac0346bbe988fef21e
// Source: lib/rubocop/cop/correctors/if_then_corrector.rb
// Source SHA-256: ac216aac6602d007c4d74b7f7aa9367942c9b64161d9ca269901fae9261a7af0
// Source: lib/rubocop/cop/correctors/lambda_literal_to_method_corrector.rb
// Source SHA-256: e8194ba4595304674a38b17cf678182bd90fc1d1365c794c46d44f6d82c31b3a
// Source: lib/rubocop/cop/correctors/line_break_corrector.rb
// Source SHA-256: cbb26d40657110e2f74e9c75832d6be308bd9c4c1be007cf55b2631c3c1beb11
// Source: lib/rubocop/cop/correctors/multiline_literal_brace_corrector.rb
// Source SHA-256: f3de31199f2ff42aed2683b0c1c76e0ec9cafc019125867de2f894afaeff1a5c
// Source: lib/rubocop/cop/correctors/ordered_gem_corrector.rb
// Source SHA-256: ff98bb7cb98d1234b10a9d1eeb84c016119131f967f49a0c5bd2a38ce28cd061
// Source: lib/rubocop/cop/correctors/parentheses_corrector.rb
// Source SHA-256: 513e01b2b527690d925e0ec44fd513f14bf847ef1622051ca15872a3ec324b27
// Source: lib/rubocop/cop/correctors/percent_literal_corrector.rb
// Source SHA-256: 3130997767a9c6092f5728b27bed958a545292ba01e9cb4dbf49dbc26d698eb6
// Source: lib/rubocop/cop/correctors/space_corrector.rb
// Source SHA-256: b63a4f9f3b6cb23d9f92fad604e58aa764ef14dfbbb843bb7bd508a2b5118995

use crate::rubocop::ast::source::SourceRange;
use crate::rubocop::ast::token::Token;
use crate::rubocop::ast::{
    node::core::NodeRef,
    processed_source::{ProcessedSource, SourceToken},
};

use super::corrector::Corrector;
use super::mixin::advanced::side_space_range;
use super::mixin::range_help::{RangeHelp, Side, SurroundingSpace};

pub(crate) struct AlignmentCorrector;
impl AlignmentCorrector {
    pub(crate) fn processed_source<'source>(
        processed_source: &'source ProcessedSource<'source>,
    ) -> &'source ProcessedSource<'source> {
        processed_source
    }

    pub(crate) const fn alignment_column(align_to_column: Option<usize>) -> usize {
        match align_to_column {
            Some(column) => column,
            None => 0,
        }
    }

    pub(crate) fn correct_node<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        processed_source: &ProcessedSource<'_>,
        node: Option<NodeRef<'_>>,
        column_delta: isize,
        using_tabs: bool,
    ) {
        let Some(node) = node else { return };
        let Some(range) = node.source_range() else {
            return;
        };
        let buffer = corrector.source_buffer();
        let expression = SourceRange::new(buffer, range.start, range.end);
        let block_comment = processed_source.comments().iter().any(|comment| {
            comment.embedded_document
                && comment.range.start >= expression.begin_pos()
                && comment.range.end <= expression.end_pos()
        });
        let taboo = Self::inside_string_ranges(buffer, node);
        Self::correct(
            corrector,
            expression,
            column_delta,
            using_tabs,
            block_comment,
            &taboo,
        );
    }

    pub(crate) fn inside_string_ranges<'b, 's>(
        buffer: &'b crate::rubocop::ast::source::SourceBuffer<'s>,
        node: NodeRef<'_>,
    ) -> Vec<SourceRange<'b, 's>> {
        node.each_node(&["any_str"])
            .into_iter()
            .filter_map(|string| Self::inside_string_range(buffer, string))
            .collect()
    }

    pub(crate) fn inside_string_range<'b, 's>(
        buffer: &'b crate::rubocop::ast::source::SourceBuffer<'s>,
        node: NodeRef<'_>,
    ) -> Option<SourceRange<'b, 's>> {
        if node.heredoc() {
            let body = node.loc("heredoc_body")?.0.clone();
            let ending = node.loc("heredoc_end")?.0.clone();
            Some(SourceRange::new(buffer, body.start, ending.end))
        } else {
            let opening = node.loc("begin")?.0.clone();
            let closing = node.loc("end")?.0.clone();
            Some(SourceRange::new(buffer, opening.end, closing.start))
        }
    }

    pub(crate) fn delimited_string_literal(node: NodeRef<'_>) -> bool {
        node.loc("begin").is_some() && node.loc("end").is_some()
    }

    pub(crate) fn calculate_range<'b, 's>(
        expression: SourceRange<'b, 's>,
        line_begin_pos: usize,
        column_delta: isize,
    ) -> SourceRange<'b, 's> {
        let buffer = expression.buffer();
        if column_delta > 0 {
            return SourceRange::new(buffer, line_begin_pos, line_begin_pos);
        }
        let width = column_delta.unsigned_abs();
        if buffer.character(line_begin_pos) == Some(' ') {
            SourceRange::new(
                buffer,
                line_begin_pos,
                (line_begin_pos + width).min(buffer.len()),
            )
        } else {
            SourceRange::new(buffer, line_begin_pos.saturating_sub(width), line_begin_pos)
        }
    }

    pub(crate) fn each_line(expression: SourceRange<'_, '_>) -> Vec<usize> {
        let mut position = expression.begin_pos();
        expression
            .source()
            .split_inclusive('\n')
            .map(|line| {
                let current = position;
                position += line.chars().count();
                current
            })
            .collect()
    }

    pub(crate) fn indentation_string(column: usize, using_tabs: bool) -> String {
        if using_tabs {
            "\t".repeat(column)
        } else {
            " ".repeat(column)
        }
    }

    pub(crate) fn correct<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        expression: SourceRange<'b, 's>,
        column_delta: isize,
        tab_indentation: bool,
        block_comment: bool,
        taboo: &[SourceRange<'b, 's>],
    ) {
        if tab_indentation || block_comment || column_delta == 0 {
            return;
        }
        let buffer = expression.buffer();
        for line_start in Self::each_line(expression) {
            let range = Self::calculate_range(expression, line_start, column_delta);
            if !taboo.iter().any(|t| {
                range.begin_pos() >= t.begin_pos()
                    && range.begin_pos() < t.end_pos()
                    && range.end_pos() <= t.end_pos()
            }) {
                if column_delta > 0 && buffer.character(line_start) != Some('\n') {
                    corrector.insert_before(range, " ".repeat(column_delta as usize));
                } else if range.source().chars().all(|c| matches!(c, ' ' | '\t')) {
                    corrector.remove(range);
                }
            }
        }
    }
    pub(crate) fn align_end<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        whitespace: SourceRange<'b, 's>,
        column: usize,
        using_tabs: bool,
    ) {
        let indentation = if using_tabs {
            "\t".repeat(column)
        } else {
            " ".repeat(column)
        };
        if whitespace.source().trim().is_empty() {
            corrector.replace(whitespace, indentation)
        } else {
            corrector.insert_after(whitespace, format!("\n{indentation}"))
        }
    }
}

pub(crate) struct EachToForCorrector;
impl EachToForCorrector {
    pub(crate) fn block_node(block_node: NodeRef<'_>) -> NodeRef<'_> {
        block_node
    }

    pub(crate) fn collection_node(block_node: NodeRef<'_>) -> Option<NodeRef<'_>> {
        block_node.send_node()?.receiver()
    }

    pub(crate) fn argument_node(block_node: NodeRef<'_>) -> Option<NodeRef<'_>> {
        block_node.arguments_node()?.first_node()
    }

    pub(crate) fn offending_range<'b, 's>(
        buffer: &'b crate::rubocop::ast::source::SourceBuffer<'s>,
        block_node: NodeRef<'_>,
    ) -> Option<SourceRange<'b, 's>> {
        let begin = block_node.source_range()?.start;
        let end = if block_node.has_arguments() {
            block_node.arguments_node()?.source_range()?.end
        } else {
            block_node.loc("begin")?.0.end
        };
        Some(SourceRange::new(buffer, begin, end))
    }

    pub(crate) fn correction_for_node(block_node: NodeRef<'_>) -> Option<String> {
        let collection = block_node.send_node()?.receiver()?.source()?;
        if block_node.has_arguments() {
            let variables = block_node.arguments_node()?.first_node()?.source()?;
            Some(Self::correction(collection, Some(variables)))
        } else {
            Some(Self::correction(collection, None))
        }
    }

    pub(crate) fn call<'b, 's>(corrector: &mut Corrector<'b, 's>, block_node: NodeRef<'_>) {
        let Some(range) = Self::offending_range(corrector.source_buffer(), block_node) else {
            return;
        };
        let Some(correction) = Self::correction_for_node(block_node) else {
            return;
        };
        corrector.replace(range, correction);
    }

    pub(crate) fn correction(collection: &str, argument: Option<&str>) -> String {
        argument.map_or_else(
            || format!("for _ in {collection} do"),
            |variables| format!("for {variables} in {collection} do"),
        )
    }
    pub(crate) fn correct<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        range: SourceRange<'b, 's>,
        collection: &str,
        argument: Option<&str>,
    ) {
        corrector.replace(range, Self::correction(collection, argument));
    }
}

pub(crate) struct ForToEachCorrector;
impl ForToEachCorrector {
    pub(crate) fn for_node(for_node: NodeRef<'_>) -> NodeRef<'_> {
        for_node
    }

    pub(crate) fn variable_node(for_node: NodeRef<'_>) -> Option<NodeRef<'_>> {
        for_node.loop_variable()
    }

    pub(crate) fn collection_node(for_node: NodeRef<'_>) -> Option<NodeRef<'_>> {
        for_node.collection()
    }

    pub(crate) fn keyword_begin(for_node: NodeRef<'_>) -> Option<std::ops::Range<usize>> {
        for_node.loc("begin").map(|location| location.0.clone())
    }

    pub(crate) fn collection_end(collection: NodeRef<'_>) -> Option<std::ops::Range<usize>> {
        if collection.kind() == "begin" {
            collection.loc("end").map(|location| location.0.clone())
        } else {
            collection.source_range()
        }
    }

    pub(crate) fn end_range(for_node: NodeRef<'_>) -> Option<std::ops::Range<usize>> {
        if for_node.do_keyword() {
            Self::keyword_begin(for_node)
        } else {
            Self::collection_end(for_node.collection()?)
        }
    }

    pub(crate) fn requires_parentheses(collection: NodeRef<'_>) -> bool {
        let operator_method = collection.kind() == "send"
            && collection.method_name().is_some_and(|name| {
                matches!(
                    name,
                    "+" | "-"
                        | "*"
                        | "/"
                        | "%"
                        | "**"
                        | "=="
                        | "!="
                        | "<"
                        | ">"
                        | "<="
                        | ">="
                        | "<=>"
                        | "==="
                        | "=~"
                        | "!~"
                        | "<<"
                        | ">>"
                        | "&"
                        | "|"
                        | "^"
                        | "[]"
                        | "[]="
                        | "+@"
                        | "-@"
                        | "~"
                        | "!"
                )
            });
        operator_method || collection.type_is(&["range"]) || collection.operator_keyword()
    }

    pub(crate) fn collection_source(collection: NodeRef<'_>) -> Option<String> {
        let source = collection.source()?;
        Some(if Self::requires_parentheses(collection) {
            format!("({source})")
        } else {
            source.to_owned()
        })
    }

    pub(crate) fn correction_for_node(for_node: NodeRef<'_>) -> Option<String> {
        let variable = for_node.loop_variable()?;
        let collection = for_node.collection()?;
        Some(Self::correction(
            &Self::collection_source(collection)?,
            variable.source()?,
            collection.kind() == "csend",
            false,
        ))
    }

    pub(crate) fn offending_range<'b, 's>(
        buffer: &'b crate::rubocop::ast::source::SourceBuffer<'s>,
        for_node: NodeRef<'_>,
    ) -> Option<SourceRange<'b, 's>> {
        let begin = for_node.source_range()?.start;
        let collection = for_node.collection()?;
        let end = if for_node.do_keyword() {
            for_node.loc("begin")?.0.end
        } else if collection.kind() == "begin" {
            collection.loc("end")?.0.end
        } else {
            collection.source_range()?.end
        };
        Some(SourceRange::new(buffer, begin, end))
    }

    pub(crate) fn call<'b, 's>(corrector: &mut Corrector<'b, 's>, for_node: NodeRef<'_>) {
        let Some(range) = Self::offending_range(corrector.source_buffer(), for_node) else {
            return;
        };
        let Some(correction) = Self::correction_for_node(for_node) else {
            return;
        };
        corrector.replace(range, correction);
    }

    pub(crate) fn correction(
        collection: &str,
        argument: &str,
        safe_navigation: bool,
        requires_parentheses: bool,
    ) -> String {
        let collection = if requires_parentheses {
            format!("({collection})")
        } else {
            collection.into()
        };
        format!(
            "{collection}{}each do |{argument}|",
            if safe_navigation { "&." } else { "." }
        )
    }
    pub(crate) fn correct<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        range: SourceRange<'b, 's>,
        collection: &str,
        argument: &str,
        safe_navigation: bool,
        requires_parentheses: bool,
    ) {
        corrector.replace(
            range,
            Self::correction(collection, argument, safe_navigation, requires_parentheses),
        );
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IfThenBranch<'a> {
    pub(crate) keyword: &'a str,
    pub(crate) condition: &'a str,
    pub(crate) body: Option<&'a str>,
    pub(crate) elsif: bool,
    pub(crate) else_branch: Option<Box<IfThenBranch<'a>>>,
    pub(crate) else_source: Option<&'a str>,
}
pub(crate) struct IfThenCorrector;
impl IfThenCorrector {
    pub(crate) fn if_node(if_node: NodeRef<'_>) -> NodeRef<'_> {
        if_node
    }

    pub(crate) fn indentation(indentation: Option<usize>) -> Option<usize> {
        indentation
    }

    pub(crate) fn branch_from_node<'ast>(node: NodeRef<'ast>) -> Option<IfThenBranch<'ast>> {
        if node.kind() != "if" {
            return None;
        }
        let keyword = node.loc("keyword").map_or_else(
            || node.keyword_name().unwrap_or("if"),
            |(_, source)| source.as_str(),
        );
        let condition = node.condition()?.source()?;
        let body = node.if_branch().and_then(NodeRef::source);
        let else_node = node.else_branch();
        let (else_branch, else_source) =
            if else_node.is_some_and(|branch| branch.kind() == "if" && branch.elsif()) {
                (Some(Box::new(Self::branch_from_node(else_node?)?)), None)
            } else {
                (None, else_node.and_then(NodeRef::source))
            };
        Some(IfThenBranch {
            keyword,
            condition,
            body,
            elsif: node.elsif(),
            else_branch,
            else_source,
        })
    }

    pub(crate) fn call<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        if_node: NodeRef<'_>,
        indentation: Option<usize>,
    ) {
        let Some(range) = if_node.source_range() else {
            return;
        };
        let Some(branch) = Self::branch_from_node(if_node) else {
            return;
        };
        corrector.replace(
            SourceRange::new(corrector.source_buffer(), range.start, range.end),
            Self::replacement(&branch, if_node.column(), indentation.unwrap_or(2)),
        );
    }

    pub(crate) fn replacement(branch: &IfThenBranch<'_>, column: usize, width: usize) -> String {
        rewrite_if(branch, &" ".repeat(column), width)
    }
    pub(crate) fn correct<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        range: SourceRange<'b, 's>,
        branch: &IfThenBranch<'_>,
        column: usize,
        width: usize,
    ) {
        corrector.replace(range, Self::replacement(branch, column, width));
    }
}
fn rewrite_if(branch: &IfThenBranch<'_>, indent: &str, width: usize) -> String {
    let prefix = if branch.elsif { indent } else { "" };
    let body = branch.body.unwrap_or("nil");
    let mut result = format!(
        "{prefix}{} {}\n{}{}{}\n",
        branch.keyword,
        branch.condition,
        indent,
        " ".repeat(width),
        body
    );
    if let Some(elsif) = &branch.else_branch {
        result.push_str(&rewrite_if(elsif, indent, width));
    } else if let Some(other) = branch.else_source {
        result.push_str(&format!(
            "{indent}else\n{indent}{}{other}\n{indent}end",
            " ".repeat(width)
        ));
    } else {
        result.push_str("end");
    }
    result
}

#[derive(Clone, Copy)]
pub(crate) struct LambdaCorrection<'b, 's> {
    pub(crate) method: SourceRange<'b, 's>,
    pub(crate) arguments: Option<SourceRange<'b, 's>>,
    pub(crate) block_begin: SourceRange<'b, 's>,
    pub(crate) block_end: SourceRange<'b, 's>,
    pub(crate) argument_sources: &'b [&'s str],
    pub(crate) braces: bool,
    pub(crate) convert_do_to_braces: bool,
    pub(crate) needs_space: bool,
}
pub(crate) struct LambdaLiteralToMethodCorrector;
impl LambdaLiteralToMethodCorrector {
    pub(crate) fn call<'b, 's>(corrector: &mut Corrector<'b, 's>, block_node: NodeRef<'_>) {
        let Some(method) = block_node.send_node() else {
            return;
        };
        let Some(arguments) = block_node.arguments_node() else {
            return;
        };
        let Some(method_range) = method.source_range() else {
            return;
        };
        let Some(block_begin) = block_node.loc("begin").map(|location| location.0.clone()) else {
            return;
        };
        let Some(block_end) = block_node.loc("end").map(|location| location.0.clone()) else {
            return;
        };
        let buffer = corrector.source_buffer();
        let method_range = SourceRange::new(buffer, method_range.start, method_range.end);
        let block_begin = SourceRange::new(buffer, block_begin.start, block_begin.end);
        let block_end = SourceRange::new(buffer, block_end.start, block_end.end);
        let argument_range = arguments
            .source_range()
            .map(|range| SourceRange::new(buffer, range.start, range.end));
        let argument_sources = arguments
            .child_nodes()
            .into_iter()
            .filter_map(NodeRef::source)
            .collect::<Vec<_>>();

        if !argument_sources.is_empty() && !arguments.parenthesized_call() {
            if let Some(argument_range) = argument_range {
                let leading = argument_range
                    .begin_pos()
                    .saturating_sub(method_range.end_pos());
                corrector.remove_preceding(argument_range, leading);
                let trailing = block_begin
                    .begin_pos()
                    .saturating_sub(argument_range.end_pos())
                    .saturating_sub(1);
                if trailing > 0 {
                    corrector.remove_preceding(block_begin, trailing);
                }
            }
        }

        let arguments_begin = arguments.loc("begin").map(|location| location.0.start);
        let arguments_end = arguments.loc("end").map(|location| location.0.end);
        let selector_end = method
            .loc("selector")
            .map_or(method_range.end_pos(), |location| location.0.end);
        let needs_space = arguments_end.is_some_and(|end| {
            block_begin.begin_pos() == end && arguments_begin == Some(selector_end)
        }) || block_begin.begin_pos() == selector_end;
        if block_node.kind() == "block" && needs_space {
            corrector.insert_before(block_begin, " ");
        }

        if block_node.kind() == "block" && !arguments.empty_and_without_delimiters() {
            if let Some(argument_range) = argument_range {
                corrector.remove(argument_range);
            }
        }
        corrector.replace(method_range, "lambda");

        if !block_node.braces() && Self::argument_to_unparenthesized_call(block_node) {
            let separating_space = buffer
                .character(block_begin.begin_pos().saturating_add(2))
                .is_some_and(char::is_whitespace);
            if !separating_space {
                corrector.insert_after(block_begin, " ");
            }
            corrector.replace(block_begin, "{");
            corrector.replace(block_end, "}");
        }
        if !argument_sources.is_empty() {
            corrector.insert_after(block_begin, format!(" |{}|", argument_sources.join(", ")));
        }
    }

    fn argument_to_unparenthesized_call(block_node: NodeRef<'_>) -> bool {
        let mut current = block_node;
        let mut parent = current.parent();
        if parent.is_some_and(|node| node.kind() == "pair") {
            current = parent.and_then(NodeRef::parent).unwrap_or(current);
            parent = current.parent();
        }
        parent.is_some_and(|parent| {
            parent.kind() == "send"
                && !parent.parenthesized_call()
                && current.sibling_index().is_some_and(|index| index > 1)
        })
    }

    pub(crate) fn correct<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        input: LambdaCorrection<'b, 's>,
    ) {
        corrector.replace(input.method, "lambda");
        if let Some(args) = input.arguments {
            corrector.remove(args);
        }
        if input.needs_space {
            corrector.insert_before(input.block_begin, " ");
        }
        if !input.braces && input.convert_do_to_braces {
            corrector.replace(input.block_begin, "{");
            corrector.replace(input.block_end, "}");
        }
        if !input.argument_sources.is_empty() {
            corrector.insert_after(
                input.block_begin,
                format!(" |{}|", input.argument_sources.join(", ")),
            );
        }
    }
}

pub(crate) struct LineBreakCorrector;
impl LineBreakCorrector {
    pub(crate) fn processed_source<'source>(
        processed_source: &'source ProcessedSource<'source>,
    ) -> &'source ProcessedSource<'source> {
        processed_source
    }

    pub(crate) fn semicolon<'tokens>(
        node_begin: usize,
        body_line: usize,
        body_column: usize,
        tokens: &'tokens [&SourceToken],
    ) -> Option<&'tokens SourceToken> {
        tokens
            .iter()
            .filter(|token| token.semicolon())
            .find(|token| {
                token.end_pos() > node_begin
                    && token.line == body_line
                    && Self::trailing_class_definition(token, body_column)
            })
            .copied()
    }

    pub(crate) fn trailing_class_definition(token: &SourceToken, body_column: usize) -> bool {
        token.column < body_column
    }

    pub(crate) fn correct_trailing_body<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        node: NodeRef<'_>,
        processed_source: &ProcessedSource<'_>,
        configured_width: usize,
    ) {
        let Some(body) = node.body().or_else(|| node.child_nodes().last().copied()) else {
            return;
        };
        let first = if body.kind() == "begin" {
            body.first_node().unwrap_or(body)
        } else {
            body
        };
        let Some(first_range) = first.source_range() else {
            return;
        };
        let Some(keyword_column) = node.loc_column("keyword") else {
            return;
        };
        let buffer = corrector.source_buffer();
        Self::break_line_before(
            corrector,
            SourceRange::new(buffer, first_range.start, first_range.end),
            keyword_column,
            configured_width,
            1,
        );
        let comment = processed_source
            .comment_at_line(node.first_line())
            .map(|comment| SourceRange::new(buffer, comment.range.start, comment.range.end));
        let node_range = node
            .source_range()
            .map(|range| SourceRange::new(buffer, range.start, range.end));
        if let Some(node_range) = node_range {
            Self::move_comment(corrector, node_range, comment, keyword_column);
        }
        let semicolon = processed_source
            .sorted_tokens()
            .into_iter()
            .filter(|token| token.semicolon())
            .find(|token| {
                node.source_range()
                    .is_some_and(|range| token.end_pos() > range.start)
                    && token.line == body.first_line()
                    && token.column < body.column()
            })
            .map(|token| SourceRange::new(buffer, token.range.start, token.range.end));
        Self::remove_semicolon(corrector, semicolon);
    }

    pub(crate) fn break_line_before<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        range: SourceRange<'b, 's>,
        keyword_column: usize,
        width: usize,
        steps: usize,
    ) {
        corrector.insert_before(
            range,
            format!("\n{}", " ".repeat(keyword_column + steps * width)),
        );
    }
    pub(crate) fn move_comment<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        node: SourceRange<'b, 's>,
        comment: Option<SourceRange<'b, 's>>,
        keyword_column: usize,
    ) {
        if let Some(comment) = comment {
            corrector.insert_before(
                node,
                format!("{}\n{}", comment.source(), " ".repeat(keyword_column)),
            );
            corrector.remove(comment);
        }
    }
    pub(crate) fn remove_semicolon<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        semicolon: Option<SourceRange<'b, 's>>,
    ) {
        if let Some(range) = semicolon {
            corrector.remove(range)
        }
    }
}

pub(crate) struct MultilineLiteralBraceCorrector;
impl MultilineLiteralBraceCorrector {
    pub(crate) fn call<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        node: NodeRef<'_>,
        processed_source: &ProcessedSource<'_>,
    ) {
        let children = node.child_nodes();
        let Some(last) = children.last().copied() else {
            return;
        };
        let Some(closing) = node.loc("end").map(|location| location.0.clone()) else {
            return;
        };
        let buffer = corrector.source_buffer();
        let closing = SourceRange::new(buffer, closing.start, closing.end);
        if closing.line() == last.last_line() {
            Self::move_to_next_line(corrector, closing);
            return;
        }

        let helper = RangeHelp::new(buffer);
        let Some(last_range) = last.source_range() else {
            return;
        };
        let last_range = SourceRange::new(buffer, last_range.start, last_range.end);
        let trailing_space = helper.range_with_surrounding_space(
            last_range,
            SurroundingSpace {
                side: Side::Right,
                newlines: false,
                whitespace: false,
                continuations: false,
            },
        );
        let comma = SourceRange::new(
            buffer,
            trailing_space.end_pos(),
            (trailing_space.end_pos() + 1).min(buffer.len()),
        );
        let last_with_comma = if comma.source() == "," {
            last_range.join(comma)
        } else {
            last_range
        };
        let commented = processed_source
            .comment_at_line(last_with_comma.last_line())
            .is_some();
        if commented && (node.chained() || node.argument()) {
            return;
        }

        let closing_with_left_space = helper.range_with_surrounding_space(
            closing,
            SurroundingSpace {
                side: Side::Left,
                newlines: true,
                whitespace: false,
                continuations: false,
            },
        );
        let (content, removal) = if commented {
            let whole = helper.range_by_whole_lines(closing, false);
            let range = SourceRange::new(buffer, closing.begin_pos(), whole.end_pos());
            (
                range.source().to_owned(),
                closing_with_left_space.join(range),
            )
        } else {
            (closing.source().to_owned(), closing_with_left_space)
        };
        corrector.remove(removal);
        corrector.insert_after(last_with_comma, content);

        if let Some(parent) = node.parent().filter(|parent| parent.call_type()) {
            if node
                .first_argument()
                .is_some_and(|argument| argument.heredoc())
            {
                if let (Some(dot), Some(parent_range)) = (
                    parent.loc("dot").map(|location| location.0.clone()),
                    parent.source_range(),
                ) {
                    let chain = SourceRange::new(buffer, dot.start, parent_range.end);
                    let source = chain.source().to_owned();
                    corrector.remove(chain);
                    corrector.insert_after(last_with_comma, source);
                }
            }
        }
    }

    pub(crate) fn move_to_next_line<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        closing: SourceRange<'b, 's>,
    ) {
        corrector.insert_before(closing, "\n");
    }
    pub(crate) fn move_to_same_line<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        closing_with_left_space: SourceRange<'b, 's>,
        last_element_end: SourceRange<'b, 's>,
        content: &str,
        chained: Option<SourceRange<'b, 's>>,
    ) {
        corrector.remove(closing_with_left_space);
        corrector.insert_after(last_element_end, content);
        if let Some(chain) = chained {
            corrector.remove(chain);
            corrector.insert_after(last_element_end, chain.source());
        }
    }
}

pub(crate) struct OrderedGemCorrector;
impl OrderedGemCorrector {
    pub(crate) fn processed_source<'source>(
        processed_source: &'source ProcessedSource<'source>,
    ) -> &'source ProcessedSource<'source> {
        processed_source
    }

    pub(crate) fn comments_as_separators(value: bool) -> bool {
        value
    }

    pub(crate) fn call<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        processed_source: &ProcessedSource<'_>,
        node: NodeRef<'_>,
        previous_declaration: NodeRef<'_>,
        comments_as_separators: bool,
    ) {
        let Some(current) = Self::declaration_with_comment(
            corrector.source_buffer(),
            processed_source,
            node,
            comments_as_separators,
        ) else {
            return;
        };
        let Some(previous) = Self::declaration_with_comment(
            corrector.source_buffer(),
            processed_source,
            previous_declaration,
            comments_as_separators,
        ) else {
            return;
        };
        Self::correct(corrector, current, previous);
    }

    fn declaration_with_comment<'b, 's>(
        buffer: &'b crate::rubocop::ast::source::SourceBuffer<'s>,
        processed_source: &ProcessedSource<'_>,
        node: NodeRef<'_>,
        comments_as_separators: bool,
    ) -> Option<SourceRange<'b, 's>> {
        let node_range = node.source_range()?;
        let first = if comments_as_separators {
            node_range.start
        } else {
            let mut start = node_range.start;
            let mut expected_line = node.first_line().saturating_sub(1);
            for comment in processed_source.comments().iter().rev() {
                if comment.line == expected_line
                    && buffer
                        .slice(comment.range.end..start)
                        .chars()
                        .all(char::is_whitespace)
                {
                    start = comment.range.start;
                    expected_line = expected_line.saturating_sub(1);
                } else if comment.range.end <= start && comment.line < expected_line {
                    break;
                }
            }
            start
        };
        let begin_line = SourceRange::new(buffer, first, first).line();
        let end_line = SourceRange::new(buffer, node_range.end, node_range.end).line();
        let begin = buffer.line_start(begin_line);
        let end_range = buffer.line_range(end_line);
        let end = (end_range.end + usize::from(buffer.character(end_range.end) == Some('\n')))
            .min(buffer.len());
        Some(SourceRange::new(buffer, begin, end))
    }

    pub(crate) fn correct<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        current_with_comment: SourceRange<'b, 's>,
        previous_with_comment: SourceRange<'b, 's>,
    ) {
        corrector.swap(current_with_comment, previous_with_comment)
    }
}

pub(crate) struct ParenthesesCorrector;
impl ParenthesesCorrector {
    pub(crate) fn call<'b, 's>(corrector: &mut Corrector<'b, 's>, node: NodeRef<'_>) {
        let Some(opening) = node.loc("begin").map(|location| location.0.clone()) else {
            return;
        };
        let Some(closing) = node.loc("end").map(|location| location.0.clone()) else {
            return;
        };
        let buffer = corrector.source_buffer();
        let helper = RangeHelp::new(buffer);
        let opening = SourceRange::new(buffer, opening.start, opening.end);
        let closing = SourceRange::new(buffer, closing.start, closing.end);
        let opening_with_space = helper.range_with_surrounding_space(
            opening,
            SurroundingSpace {
                side: Side::Right,
                whitespace: true,
                newlines: false,
                continuations: false,
            },
        );
        corrector.remove(opening_with_space);

        let preserve_newline = Self::comment_above_close_paren_swallows_chain(node, buffer);
        let closing_with_space = helper.range_with_surrounding_space(
            closing,
            SurroundingSpace {
                side: Side::Left,
                newlines: !preserve_newline,
                whitespace: false,
                continuations: false,
            },
        );
        let orphaned_comma = Self::only_closing_paren_before_comma(closing, buffer);
        if orphaned_comma {
            let extended = Self::extend_range_for_heredoc(node, closing_with_space, &helper);
            corrector.remove(extended);
            if let Some(heredoc) = node
                .child_nodes()
                .last()
                .copied()
                .filter(|child| child.heredoc())
            {
                if let Some(range) = heredoc.source_range() {
                    corrector.insert_after(SourceRange::new(buffer, range.start, range.end), ",");
                }
            }
        } else {
            corrector.remove(closing_with_space);
        }

        let space_before_question = node.parent().is_some_and(|parent| {
            parent.kind() == "if"
                && parent.ternary()
                && parent.loc("question").is_some_and(|question| {
                    node.loc_last_column("end")
                        == Some(parent.loc_column("question").unwrap_or(usize::MAX))
                        || closing.end_pos() == question.0.start
                })
        });
        if space_before_question {
            corrector.insert_after(closing, " ");
        }
    }

    fn comment_above_close_paren_swallows_chain(
        node: NodeRef<'_>,
        buffer: &crate::rubocop::ast::source::SourceBuffer<'_>,
    ) -> bool {
        let Some(last_child) = node.child_nodes().last().copied() else {
            return false;
        };
        let Some(body_end) = last_child.source_range().map(|range| range.end) else {
            return false;
        };
        let Some(close_begin) = node.loc("end").map(|location| location.0.start) else {
            return false;
        };
        if body_end >= close_begin
            || !buffer
                .slice(body_end..close_begin)
                .lines()
                .any(|line| line.contains('#'))
        {
            return false;
        }
        let line = node.loc("end").map_or(1, |location| {
            SourceRange::new(buffer, location.0.start, location.0.end).line()
        });
        let after = buffer.slice(close_begin.saturating_add(1)..buffer.line_range(line).end);
        let trimmed = after.trim_start();
        !trimmed.is_empty() && !trimmed.starts_with('#')
    }

    fn only_closing_paren_before_comma(
        closing: SourceRange<'_, '_>,
        buffer: &crate::rubocop::ast::source::SourceBuffer<'_>,
    ) -> bool {
        let line = buffer.source_line(closing.line()).trim_start();
        line.strip_prefix(')')
            .is_some_and(|after| after.trim_start().starts_with(','))
    }

    fn extend_range_for_heredoc<'b, 's>(
        node: NodeRef<'_>,
        range: SourceRange<'b, 's>,
        helper: &RangeHelp<'b, 's>,
    ) -> SourceRange<'b, 's> {
        if !node
            .child_nodes()
            .last()
            .is_some_and(|child| child.heredoc())
        {
            return range;
        }
        let line = helper.range_by_whole_lines(range, false).source();
        let offset = line
            .find(')')
            .and_then(|closing| line[closing + 1..].find(',').map(|comma| comma + 1))
            .unwrap_or(0);
        range.adjust(0, offset as isize)
    }

    pub(crate) fn correct<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        open_with_right_space: SourceRange<'b, 's>,
        close_with_left_space: SourceRange<'b, 's>,
        space_before_question: bool,
        heredoc_end: Option<SourceRange<'b, 's>>,
    ) {
        corrector.remove(open_with_right_space);
        corrector.remove(close_with_left_space);
        if space_before_question {
            corrector.insert_after(close_with_left_space, " ");
        }
        if let Some(end) = heredoc_end {
            corrector.insert_after(end, ",");
        }
    }
}

pub(crate) struct PercentLiteralCorrector;
impl PercentLiteralCorrector {
    pub(crate) fn call<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        node: NodeRef<'_>,
        kind: char,
        delimiters: (char, char),
    ) {
        let children = node.child_nodes();
        let words = children
            .iter()
            .filter_map(|word| word.string_child(0).or_else(|| word.symbol_child(0)))
            .collect::<Vec<_>>();
        if words.len() != children.len() {
            return;
        }
        let escape = words.iter().any(|word| percent_word_needs_escaping(word));
        let kind = if escape {
            kind.to_ascii_uppercase()
        } else {
            kind
        };
        let mut contents = String::new();
        for (index, (word, node_word)) in words.iter().zip(children.iter()).enumerate() {
            if node.multiline() {
                let previous_line = index
                    .checked_sub(1)
                    .map_or(node.first_line(), |previous| children[previous].last_line());
                if node_word.first_line() == previous_line {
                    if index > 0 || node_word.first_line() != node.first_line() {
                        contents.push(' ');
                    }
                } else {
                    contents.push('\n');
                    contents.push_str(&" ".repeat(node_word.column()));
                }
            } else if index > 0 {
                contents.push(' ');
            }
            contents.push_str(&fix_percent_word(word, escape, delimiters));
        }
        if node.multiline() {
            if let Some(closing) = node.loc("end") {
                let closing =
                    SourceRange::new(corrector.source_buffer(), closing.0.start, closing.0.end);
                if children
                    .last()
                    .is_some_and(|last| last.last_line() < closing.line())
                {
                    contents.push('\n');
                    contents.push_str(&" ".repeat(closing.column()));
                }
            }
        }
        let Some(range) = node.source_range() else {
            return;
        };
        corrector.replace(
            SourceRange::new(corrector.source_buffer(), range.start, range.end),
            format!("%{kind}{}{contents}{}", delimiters.0, delimiters.1),
        );
    }

    pub(crate) fn correction(
        words: &[&str],
        kind: char,
        delimiters: (char, char),
        multiline_gaps: Option<&[&str]>,
    ) -> String {
        let escape = words.iter().any(|word| percent_word_needs_escaping(word));
        let kind = if escape {
            kind.to_ascii_uppercase()
        } else {
            kind
        };
        let contents = if let Some(gaps) = multiline_gaps {
            let mut out = String::new();
            for (index, word) in words.iter().enumerate() {
                if index > 0 {
                    out.push_str(gaps.get(index - 1).copied().unwrap_or(" "));
                }
                out.push_str(&fix_percent_word(word, escape, delimiters));
            }
            out
        } else {
            words
                .iter()
                .map(|word| fix_percent_word(word, escape, delimiters))
                .collect::<Vec<_>>()
                .join(" ")
        };
        format!("%{kind}{}{contents}{}", delimiters.0, delimiters.1)
    }
    pub(crate) fn correct<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        range: SourceRange<'b, 's>,
        words: &[&str],
        kind: char,
        delimiters: (char, char),
        multiline_gaps: Option<&[&str]>,
    ) {
        corrector.replace(
            range,
            Self::correction(words, kind, delimiters, multiline_gaps),
        );
    }
}
fn percent_word_needs_escaping(word: &str) -> bool {
    super::framework::needs_escaping(word)
}
fn fix_percent_word(word: &str, escape: bool, delimiters: (char, char)) -> String {
    let mut content = if escape {
        super::framework::escape_string(word)
    } else {
        word.into()
    };
    let (open, close) = delimiters;
    if open == close || content.matches(open).count() != content.matches(close).count() {
        content = content.replace(open, &format!("\\{open}"));
        if close != open {
            content = content.replace(close, &format!("\\{close}"));
        }
    }
    content
}

pub(crate) struct SpaceCorrector;
impl SpaceCorrector {
    pub(crate) fn processed_source<'source>(
        processed_source: &'source ProcessedSource<'source>,
    ) -> &'source ProcessedSource<'source> {
        processed_source
    }

    pub(crate) fn empty_corrections<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        empty_config: &str,
        left: &Token<'b, 's>,
        right: &Token<'b, 's>,
    ) {
        let buffer = corrector.source_buffer();
        let between = SourceRange::new(buffer, left.end_pos(), right.begin_pos());
        let has_exactly_one_space = left.end_pos() + 1 == right.begin_pos()
            && buffer.character(left.end_pos()) == Some(' ');
        let has_no_character = left.end_pos() == right.begin_pos();
        if empty_config == "space" && !has_exactly_one_space {
            Self::empty_correction(corrector, between, left.pos(), true);
        } else if empty_config == "no_space" && !has_no_character {
            Self::empty_correction(corrector, between, left.pos(), false);
        }
    }

    pub(crate) fn remove_token_space<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        left: &Token<'b, 's>,
        right: &Token<'b, 's>,
    ) {
        let buffer = corrector.source_buffer();
        let left_space = left
            .space_after()
            .then(|| side_space_range(buffer, left.end_pos(), false));
        let right_space = right
            .space_before()
            .then(|| side_space_range(buffer, right.begin_pos(), true));
        Self::remove_space(corrector, left_space, right_space);
    }

    pub(crate) fn add_token_space<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        left: &Token<'b, 's>,
        right: &Token<'b, 's>,
    ) {
        Self::add_space(
            corrector,
            left.pos(),
            right.pos(),
            left.space_after(),
            right.space_before(),
        );
    }

    pub(crate) fn remove_space<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        left_space: Option<SourceRange<'b, 's>>,
        right_space: Option<SourceRange<'b, 's>>,
    ) {
        if let Some(range) = left_space {
            corrector.remove(range)
        }
        if let Some(range) = right_space.filter(|right| {
            left_space.is_none_or(|left| {
                left.begin_pos() != right.begin_pos() || left.end_pos() != right.end_pos()
            })
        }) {
            corrector.remove(range)
        }
    }
    pub(crate) fn add_space<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        left: SourceRange<'b, 's>,
        right: SourceRange<'b, 's>,
        left_has_space: bool,
        right_has_space: bool,
    ) {
        if !left_has_space {
            corrector.insert_after(left, " ")
        }
        if !right_has_space {
            corrector.insert_before(right, " ")
        }
    }
    pub(crate) fn empty_correction<'b, 's>(
        corrector: &mut Corrector<'b, 's>,
        between: SourceRange<'b, 's>,
        left: SourceRange<'b, 's>,
        want_one_space: bool,
    ) {
        corrector.remove(between);
        if want_one_space {
            corrector.insert_after(left, " ")
        }
    }
}
// RuboCop API ownership: lib/rubocop/cop/correctors/alignment_corrector.rb => correct, processed_source
// RuboCop API ownership: lib/rubocop/cop/correctors/ordered_gem_corrector.rb => comments_as_separators, correct, processed_source
// RuboCop API ownership: lib/rubocop/cop/correctors/each_to_for_corrector.rb => argument_node, block_node, call, collection_node, correction
// RuboCop API ownership: lib/rubocop/cop/correctors/for_to_each_corrector.rb => call, collection_node, correction, for_node, variable_node
// RuboCop API ownership: lib/rubocop/cop/correctors/if_then_corrector.rb => call, if_node, indentation
// RuboCop API ownership: lib/rubocop/cop/correctors/line_break_corrector.rb => processed_source
// RuboCop API ownership: lib/rubocop/cop/correctors/space_corrector.rb => processed_source
