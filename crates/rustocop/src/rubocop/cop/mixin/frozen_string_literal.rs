// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/frozen_string_literal.rb
// Source SHA-256: fe4649eb36fb5a56d21b08dca166cd699d3493521b884fb60d470dd334dcdfdc

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct MagicComment {
    pub(crate) frozen_string_literal: Option<bool>,
    pub(crate) valid_literal_value: bool,
}

pub(crate) struct FrozenStringLiteral<'processed, 'source> {
    processed_source: &'processed ProcessedSource<'source>,
    target_ruby_version: f64,
    string_literals_frozen_by_default: Option<bool>,
}

impl<'processed, 'source> FrozenStringLiteral<'processed, 'source> {
    pub(crate) fn new(
        processed_source: &'processed ProcessedSource<'source>,
        target_ruby_version: f64,
        string_literals_frozen_by_default: Option<bool>,
    ) -> Self {
        Self {
            processed_source,
            target_ruby_version,
            string_literals_frozen_by_default,
        }
    }

    pub(crate) fn frozen_string_literal_comment_exists(&self) -> bool {
        self.leading_magic_comments()
            .into_iter()
            .any(|comment| comment.valid_literal_value)
    }

    pub(crate) fn frozen_string_literal(&self, node: NodeRef<'_>) -> bool {
        let frozen_string = if self.target_ruby_version >= 3.0 {
            self.uninterpolated_string(node) || self.uninterpolated_heredoc(node)
        } else {
            matches!(node.kind(), "str" | "dstr")
        };
        frozen_string && self.frozen_string_literals_enabled()
    }

    pub(crate) fn uninterpolated_string(&self, node: NodeRef<'_>) -> bool {
        node.kind() == "str"
            || node.kind() == "dstr"
                && node
                    .each_descendant(&["begin", "ivar", "cvar", "gvar"])
                    .is_empty()
    }

    pub(crate) fn uninterpolated_heredoc(&self, node: NodeRef<'_>) -> bool {
        node.kind() == "dstr"
            && node.heredoc()
            && node
                .child_nodes()
                .into_iter()
                .all(|child| child.kind() == "str")
    }

    pub(crate) fn frozen_heredoc(&self, node: NodeRef<'_>) -> bool {
        self.uninterpolated_heredoc(node)
    }

    pub(crate) fn frozen_string_literals_enabled(&self) -> bool {
        let _ruby_version = self.processed_source.ruby_version();
        if let Some(comment) = self
            .leading_magic_comments()
            .into_iter()
            .find(|comment| comment.frozen_string_literal.is_some())
        {
            return comment.frozen_string_literal == Some(true);
        }
        self.string_literals_frozen_by_default.unwrap_or(false)
    }

    pub(crate) fn frozen_string_literals_disabled(&self) -> bool {
        self.leading_magic_comments()
            .into_iter()
            .any(|comment| comment.frozen_string_literal == Some(false))
    }

    pub(crate) fn frozen_string_literal_specified(&self) -> bool {
        self.leading_magic_comments()
            .into_iter()
            .any(|comment| comment.frozen_string_literal.is_some())
    }

    pub(crate) fn leading_magic_comments(&self) -> Vec<MagicComment> {
        self.leading_comment_lines()
            .into_iter()
            .map(parse_magic_comment)
            .collect()
    }

    pub(crate) fn leading_comment_lines(&self) -> Vec<&str> {
        let first_non_comment_line = self
            .processed_source
            .tokens()
            .iter()
            .find(|token| !token.comment() && !token.new_line())
            .map(|token| token.line);
        let lines = self.processed_source.lines();
        match first_non_comment_line {
            Some(line) => lines[..line.saturating_sub(1).min(lines.len())]
                .iter()
                .map(String::as_str)
                .collect(),
            None => lines.iter().map(String::as_str).collect(),
        }
    }
}

fn parse_magic_comment(line: &str) -> MagicComment {
    let normalized = line.trim().trim_start_matches('#').trim();
    let value = normalized
        .split_once(':')
        .filter(|(name, _)| name.trim() == "frozen_string_literal")
        .map(|(_, value)| value.trim());
    let frozen_string_literal = match value {
        Some("true") => Some(true),
        Some("false") => Some(false),
        _ => None,
    };
    MagicComment {
        frozen_string_literal,
        valid_literal_value: frozen_string_literal.is_some(),
    }
}

#[cfg(test)]
mod spec;
