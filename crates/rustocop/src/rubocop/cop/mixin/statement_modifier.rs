// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/statement_modifier.rb
// Source SHA-256: c45a2318b642b5a3336c3591423f074152d95e5b83ca5069a382e5d2db8aa190

use regex::Regex;
use unicode_width::UnicodeWidthStr;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;

pub(crate) struct StatementModifier<'processed, 'source> {
    processed_source: &'processed ProcessedSource<'source>,
    max_line_length: Option<usize>,
    cop_name: String,
}

impl<'processed, 'source> StatementModifier<'processed, 'source> {
    pub(crate) fn new(
        processed_source: &'processed ProcessedSource<'source>,
        max_line_length: Option<usize>,
        cop_name: impl Into<String>,
    ) -> Self {
        Self {
            processed_source,
            max_line_length,
            cop_name: cop_name.into(),
        }
    }

    pub(crate) fn single_line_as_modifier(&self, node: NodeRef<'_>) -> bool {
        !self.non_eligible_node(node)
            && !self.non_eligible_body(node.if_branch())
            && !self.non_eligible_condition(node.condition())
            && self.modifier_fits_on_single_line(node)
    }

    pub(crate) fn non_eligible_node(&self, node: NodeRef<'_>) -> bool {
        node.modifier_form()
            || node.nonempty_line_count() > 3
            || self.processed_source.line_with_comment(node.last_line())
            || self.first_line_comment(node).is_some() && self.code_after(node).is_some()
    }

    pub(crate) fn non_eligible_body(&self, body: Option<NodeRef<'_>>) -> bool {
        body.is_none_or(|body| {
            body.empty_source()
                || body.kind() == "begin"
                || self
                    .processed_source
                    .contains_comment(body.first_line(), body.last_line())
        })
    }

    pub(crate) fn non_eligible_condition(&self, condition: Option<NodeRef<'_>>) -> bool {
        condition.is_some_and(|condition| {
            condition
                .each_node(&[])
                .into_iter()
                .any(|node| node.kind() == "lvasgn")
        })
    }

    pub(crate) fn modifier_fits_on_single_line(&self, node: NodeRef<'_>) -> bool {
        self.max_line_length
            .is_none_or(|max| self.length_in_modifier_form(node) <= max)
    }

    pub(crate) fn length_in_modifier_form(&self, node: NodeRef<'_>) -> usize {
        let code_before = node.loc("keyword").map_or_else(String::new, |_| {
            self.processed_source
                .buffer()
                .source_line(node.first_line())
                .chars()
                .take(node.loc_column("keyword").unwrap_or(0))
                .collect()
        });
        format!(
            "{code_before}{}{}",
            self.to_modifier_form(node),
            self.code_after(node).unwrap_or_default()
        )
        .width()
    }

    pub(crate) fn to_modifier_form(&self, node: NodeRef<'_>) -> String {
        let Some(body) = node.if_branch() else {
            return String::new();
        };
        let Some(condition) = node.condition().and_then(NodeRef::source) else {
            return String::new();
        };
        let keyword = node.keyword_name().unwrap_or("if");
        let expression = format!("{} {keyword} {condition}", self.if_body_source(body));
        let expression = if self.parenthesize(node) {
            format!("({expression})")
        } else {
            expression
        };
        self.first_line_comment(node)
            .map_or(expression.clone(), |comment| {
                format!("{expression} {comment}")
            })
    }

    pub(crate) fn if_body_source(&self, if_body: NodeRef<'_>) -> String {
        if if_body.call_type()
            && if_body.method_name() != Some("[]=")
            && self.omitted_value_in_last_hash_arg(if_body)
        {
            format!(
                "{}({})",
                self.method_source(if_body),
                if_body
                    .arguments()
                    .into_iter()
                    .filter_map(NodeRef::source)
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        } else {
            if_body.source().unwrap_or_default().to_owned()
        }
    }

    pub(crate) fn omitted_value_in_last_hash_arg(&self, if_body: NodeRef<'_>) -> bool {
        if_body
            .last_argument()
            .filter(|argument| argument.kind() == "hash")
            .and_then(|hash| hash.pairs().last().copied())
            .is_some_and(NodeRef::value_omission)
    }

    pub(crate) fn method_source(&self, if_body: NodeRef<'_>) -> String {
        let Some(source) = if_body.source_range() else {
            return String::new();
        };
        let implicit_call =
            if_body.method_name() == Some("call") && if_body.loc("selector").is_none();
        let end = if implicit_call {
            if_body.loc("dot").map(|location| location.0.end)
        } else {
            if_body.loc("selector").map(|location| location.0.end)
        }
        .unwrap_or(source.end);
        self.processed_source
            .buffer()
            .slice(source.start..end)
            .to_owned()
    }

    pub(crate) fn first_line_comment(&self, node: NodeRef<'_>) -> Option<String> {
        let comment = self
            .processed_source
            .comments()
            .iter()
            .find(|comment| comment.line == node.first_line())?;
        (!self.comment_disables_cop(&comment.text)).then(|| comment.text.clone())
    }

    pub(crate) fn code_after(&self, node: NodeRef<'_>) -> Option<String> {
        let end = node.loc("end")?;
        let buffer = self.processed_source.buffer();
        let line = crate::rubocop::ast::source::SourceRange::new(&buffer, end.0.start, end.0.end);
        let code = buffer
            .source_line(line.last_line())
            .chars()
            .skip(line.last_column())
            .collect::<String>();
        (!code.is_empty()).then_some(code)
    }

    pub(crate) fn parenthesize(&self, node: NodeRef<'_>) -> bool {
        node.parent().is_some_and(|parent| {
            parent.assignment()
                || parent.operator_keyword()
                || matches!(parent.kind(), "array" | "pair" | "send")
        })
    }

    pub(crate) fn comment_disables_cop(&self, comment: &str) -> bool {
        Regex::new(&format!(
            r"#\s*rubocop\s*:\s*(disable|todo)\s*([^,],)*\s*(all|{})",
            regex::escape(&self.cop_name)
        ))
        .unwrap()
        .is_match(comment)
    }
}

#[cfg(test)]
mod spec;
