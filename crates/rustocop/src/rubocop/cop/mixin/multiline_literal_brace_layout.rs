// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/multiline_literal_brace_layout.rb
// Source SHA-256: 088de502fd25c62152c5466af620c0b433affe56ca1c69680d8bc8560df511e7

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;
use crate::rubocop::ast::source::SourceRange;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BraceLayoutOffense {
    pub(crate) message: String,
}

pub(crate) struct MultilineLiteralBraceLayout<'processed, 'source> {
    style: String,
    processed_source: &'processed ProcessedSource<'source>,
    same_line_message: String,
    new_line_message: String,
    always_same_line_message: String,
    always_new_line_message: String,
}

impl<'processed, 'source> MultilineLiteralBraceLayout<'processed, 'source> {
    pub(crate) fn new(
        style: impl Into<String>,
        processed_source: &'processed ProcessedSource<'source>,
    ) -> Self {
        Self {
            style: style.into(),
            processed_source,
            same_line_message: "Closing brace must be on the same line as opening brace.".into(),
            new_line_message: "Closing brace must be on a new line.".into(),
            always_same_line_message: "Closing brace must be on the same line as the last element."
                .into(),
            always_new_line_message: "Closing brace must be on a line by itself.".into(),
        }
    }

    pub(crate) fn check_brace_layout(&self, node: NodeRef<'_>) -> Option<BraceLayoutOffense> {
        if self.ignored_literal(node) {
            return None;
        }
        if self
            .children(node)
            .last()
            .is_some_and(|last| self.last_line_heredoc(*last, None))
        {
            return None;
        }
        self.check(node)
    }

    pub(crate) fn new_line_needed_before_closing_brace(&self, node: NodeRef<'_>) -> bool {
        let Some(last) = self.children(node).last().copied() else {
            return false;
        };
        self.processed_source
            .comment_at_line(last.last_line())
            .is_some()
            && (node.chained() || node.argument())
    }

    pub(crate) fn check(&self, node: NodeRef<'_>) -> Option<BraceLayoutOffense> {
        match self.style.as_str() {
            "symmetrical" => self.check_symmetrical(node),
            "new_line" => self.check_new_line(node),
            "same_line" => self.check_same_line(node),
            _ => None,
        }
    }

    pub(crate) fn check_new_line(&self, node: NodeRef<'_>) -> Option<BraceLayoutOffense> {
        self.closing_brace_on_same_line(node)
            .then(|| BraceLayoutOffense {
                message: self.always_new_line_message.clone(),
            })
    }

    pub(crate) fn check_same_line(&self, node: NodeRef<'_>) -> Option<BraceLayoutOffense> {
        (!self.closing_brace_on_same_line(node)).then(|| BraceLayoutOffense {
            message: self.always_same_line_message.clone(),
        })
    }

    pub(crate) fn check_symmetrical(&self, node: NodeRef<'_>) -> Option<BraceLayoutOffense> {
        if self.opening_brace_on_same_line(node) {
            (!self.closing_brace_on_same_line(node)).then(|| BraceLayoutOffense {
                message: self.same_line_message.clone(),
            })
        } else {
            self.closing_brace_on_same_line(node)
                .then(|| BraceLayoutOffense {
                    message: self.new_line_message.clone(),
                })
        }
    }

    pub(crate) fn empty_literal(&self, node: NodeRef<'_>) -> bool {
        self.children(node).is_empty()
    }

    pub(crate) fn implicit_literal(&self, node: NodeRef<'_>) -> bool {
        node.loc("begin").is_none()
    }

    pub(crate) fn ignored_literal(&self, node: NodeRef<'_>) -> bool {
        self.implicit_literal(node) || self.empty_literal(node) || node.single_line()
    }

    pub(crate) fn children<'ast>(&self, node: NodeRef<'ast>) -> Vec<NodeRef<'ast>> {
        node.child_nodes()
    }

    pub(crate) fn opening_brace_on_same_line(&self, node: NodeRef<'_>) -> bool {
        let Some(opening) = node.loc("begin") else {
            return false;
        };
        let Some(first) = self.children(node).first().copied() else {
            return false;
        };
        self.location_line(opening.0.start) == first.first_line()
    }

    pub(crate) fn closing_brace_on_same_line(&self, node: NodeRef<'_>) -> bool {
        let Some(closing) = node.loc("end") else {
            return false;
        };
        let Some(last) = self.children(node).last().copied() else {
            return false;
        };
        self.location_line(closing.0.start) == last.last_line()
    }

    pub(crate) fn last_line_heredoc(&self, node: NodeRef<'_>, parent: Option<NodeRef<'_>>) -> bool {
        let parent = parent.unwrap_or(node);
        if node
            .loc("heredoc_end")
            .is_some_and(|location| self.location_line(location.0.end) >= parent.last_line())
        {
            return true;
        }
        node.child_nodes()
            .into_iter()
            .any(|child| self.last_line_heredoc(child, Some(parent)))
    }

    fn location_line(&self, position: usize) -> usize {
        let buffer = self.processed_source.buffer();
        SourceRange::new(&buffer, position, position).line()
    }
}

#[cfg(test)]
mod spec;
