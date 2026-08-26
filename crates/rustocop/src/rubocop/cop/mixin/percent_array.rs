// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/percent_array.rb
// Source SHA-256: ce85c24d9a1b1c26805b379231de174ca2d6016fccff9c88300c664946d7aa42

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PercentArrayOffense {
    pub(crate) message: String,
    pub(crate) replacement: Option<String>,
    pub(crate) detected_style: &'static str,
    pub(crate) no_acceptable_style: bool,
}

pub(crate) struct PercentArray<'processed, 'source> {
    processed_source: &'processed ProcessedSource<'source>,
    style: String,
    minimum_size: usize,
    invalid_contents: bool,
    array_message: String,
    percent_message: String,
}

impl<'processed, 'source> PercentArray<'processed, 'source> {
    pub(crate) fn new(
        processed_source: &'processed ProcessedSource<'source>,
        style: impl Into<String>,
        minimum_size: usize,
        invalid_contents: bool,
    ) -> Self {
        Self {
            processed_source,
            style: style.into(),
            minimum_size,
            invalid_contents,
            array_message: "Use %<prefer>s for an array of words.".into(),
            percent_message: "Use a percent literal for this array.".into(),
        }
    }

    pub(crate) fn invalid_percent_array_context(&self, node: NodeRef<'_>) -> bool {
        let Some(parent) = node.parent().filter(|parent| parent.kind() == "send") else {
            return false;
        };
        parent.arguments().contains(&node) && !parent.parenthesized_call() && parent.block_literal()
    }

    pub(crate) const fn invalid_percent_array_contents(&self, _node: NodeRef<'_>) -> bool {
        self.invalid_contents
    }

    pub(crate) fn allowed_bracket_array(&self, node: NodeRef<'_>) -> bool {
        self.comments_in_array(node)
            || node.values().len() < self.minimum_size
            || self.invalid_percent_array_context(node)
    }

    pub(crate) fn comments_in_array(&self, node: NodeRef<'_>) -> bool {
        !self
            .processed_source
            .each_comment_in_lines(node.first_line()..node.last_line())
            .is_empty()
    }

    pub(crate) fn check_percent_array(&self, node: NodeRef<'_>) -> Option<PercentArrayOffense> {
        let brackets_required = self.invalid_percent_array_contents(node);
        if self.style != "brackets" && !brackets_required {
            return None;
        }
        let elements = node
            .values()
            .into_iter()
            .map(|value| {
                let content = value
                    .string_child(0)
                    .or_else(|| value.symbol_child(0))
                    .unwrap_or_default();
                format!("{content:?}")
            })
            .collect::<Vec<_>>();
        let bracketed_array =
            self.build_bracketed_array_with_appropriate_whitespace(&elements, node);
        Some(PercentArrayOffense {
            message: self.build_message_for_bracketed_array(&bracketed_array),
            replacement: Some(bracketed_array),
            detected_style: "percent",
            no_acceptable_style: brackets_required,
        })
    }

    pub(crate) fn build_message_for_bracketed_array(&self, preferred_array_code: &str) -> String {
        let preferred = if preferred_array_code.contains('\n') {
            "an array literal `[...]`".to_owned()
        } else {
            format!("`{preferred_array_code}`")
        };
        self.array_message.replace("%<prefer>s", &preferred)
    }

    pub(crate) fn check_bracketed_array(
        &self,
        node: NodeRef<'_>,
        literal_prefix: char,
    ) -> Option<PercentArrayOffense> {
        if self.allowed_bracket_array(node) || self.style != "percent" {
            return None;
        }
        Some(PercentArrayOffense {
            message: self.percent_message.clone(),
            replacement: Some(format!("%{literal_prefix}")),
            detected_style: "brackets",
            no_acceptable_style: false,
        })
    }

    pub(crate) fn build_bracketed_array_with_appropriate_whitespace(
        &self,
        elements: &[String],
        node: NodeRef<'_>,
    ) -> String {
        format!(
            "[{}{}{}]",
            self.whitespace_leading(node),
            elements.join(&format!(",{}", self.whitespace_between(node))),
            self.whitespace_trailing(node)
        )
    }

    pub(crate) fn whitespace_between(&self, node: NodeRef<'_>) -> String {
        let children = node.child_nodes();
        if children.len() < 2 {
            return " ".into();
        }
        let (Some(first), Some(second)) = (children[0].source_range(), children[1].source_range())
        else {
            return " ".into();
        };
        self.processed_source
            .buffer()
            .slice(first.end..second.start)
            .to_owned()
    }

    pub(crate) fn whitespace_leading(&self, node: NodeRef<'_>) -> String {
        let Some(opening) = node.loc("begin") else {
            return String::new();
        };
        let Some(first) = node
            .child_nodes()
            .first()
            .and_then(|child| child.source_range())
        else {
            return String::new();
        };
        self.processed_source
            .buffer()
            .slice(opening.0.end..first.start)
            .to_owned()
    }

    pub(crate) fn whitespace_trailing(&self, node: NodeRef<'_>) -> String {
        let Some(closing) = node.loc("end") else {
            return String::new();
        };
        let Some(last) = node
            .child_nodes()
            .last()
            .and_then(|child| child.source_range())
        else {
            return String::new();
        };
        self.processed_source
            .buffer()
            .slice(last.end..closing.0.start)
            .to_owned()
    }
}

#[cfg(test)]
mod spec;
