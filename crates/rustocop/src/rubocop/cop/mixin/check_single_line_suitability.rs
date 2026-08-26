// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/check_single_line_suitability.rb
// Source SHA-256: e545083edafa2d21eafe80a0a453c02fb065a3e3d668eb424c1eab553ce3e04b

use regex::Regex;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;

pub(crate) struct CheckSingleLineSuitability<'source, 'input> {
    pub(crate) processed_source: &'source ProcessedSource<'input>,
    pub(crate) max_line_length: Option<usize>,
}

impl CheckSingleLineSuitability<'_, '_> {
    pub(crate) fn suitable_as_single_line(&self, node: NodeRef<'_>) -> bool {
        !self.too_long(node) && !self.comment_within(node) && self.safe_to_split(node)
    }

    pub(crate) fn too_long(&self, node: NodeRef<'_>) -> bool {
        self.max_line_length.is_some_and(|max| {
            let lines = self.processed_source.lines_slice(
                node.first_line().saturating_sub(1),
                node.last_line() - node.first_line() + 1,
            );
            self.to_single_line(&lines.join("\n")).chars().count() > max
        })
    }

    pub(crate) fn to_single_line(&self, source: &str) -> String {
        let mut result = Regex::new("\" *\\\\\n\\s*'")
            .expect("static regex")
            .replace_all(source, "\" + '")
            .into_owned();
        result = Regex::new("' *\\\\\n\\s*\"")
            .expect("static regex")
            .replace_all(&result, "' + \"")
            .into_owned();
        result = Regex::new("\" *\\\\\n\\s*\"")
            .expect("static regex")
            .replace_all(&result, "")
            .into_owned();
        result = Regex::new("' *\\\\\n\\s*'")
            .expect("static regex")
            .replace_all(&result, "")
            .into_owned();
        result = Regex::new(r"\n\s*(&?\.\w)")
            .expect("static regex")
            .replace_all(&result, "$1")
            .into_owned();
        Regex::new(r"\s*\\?\n\s*")
            .expect("static regex")
            .replace_all(&result, " ")
            .into_owned()
    }

    pub(crate) fn comment_within(&self, node: NodeRef<'_>) -> bool {
        self.processed_source
            .comments()
            .iter()
            .any(|comment| (node.first_line()..=node.last_line()).contains(&comment.line))
    }

    pub(crate) fn safe_to_split(&self, node: NodeRef<'_>) -> bool {
        node.each_descendant(&["if", "case", "kwbegin", "any_def", "rescue", "ensure"])
            .is_empty()
            && !node
                .each_descendant(&["dstr", "str"])
                .into_iter()
                .any(|string| {
                    string.heredoc()
                        || string
                            .string_child(0)
                            .is_some_and(|value| value.contains('\n'))
                })
            && !node
                .each_descendant(&["begin", "sym"])
                .into_iter()
                .any(NodeRef::multiline)
    }
}
