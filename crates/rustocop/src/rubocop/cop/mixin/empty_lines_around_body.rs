// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/empty_lines_around_body.rb
// Source SHA-256: 72cf05e697e8525fc19d5705f231bbbcd84963e1724666f533767eb9cf39baeb

use crate::rubocop::ast::node::core::NodeRef;

const MSG_EXTRA: &str = "Extra empty line detected at %<kind>s body %<location>s.";
const MSG_MISSING: &str = "Empty line missing at %<kind>s body %<location>s.";

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BodyOffense {
    pub(crate) line: usize,
    pub(crate) insert_empty_line: bool,
    pub(crate) message: String,
}

pub(crate) struct EmptyLinesAroundBody<'a> {
    lines: &'a [&'a str],
    style: &'a str,
    kind: &'a str,
}

impl<'a> EmptyLinesAroundBody<'a> {
    pub(crate) fn new(lines: &'a [&'a str], style: &'a str, kind: &'a str) -> Self {
        Self { lines, style, kind }
    }

    pub(crate) fn check(
        &self,
        body: Option<NodeRef<'_>>,
        first_line: usize,
        last_line: usize,
    ) -> Vec<BodyOffense> {
        if self.valid_body_style(body) || first_line == last_line {
            return Vec::new();
        }
        match self.style {
            "empty_lines_except_namespace" => {
                self.check_empty_lines_except_namespace(body, first_line, last_line)
            }
            "empty_lines_special" => self.check_empty_lines_special(body, first_line, last_line),
            style => self.check_both(style, first_line, last_line),
        }
    }

    fn check_empty_lines_except_namespace(
        &self,
        body: Option<NodeRef<'_>>,
        first_line: usize,
        last_line: usize,
    ) -> Vec<BodyOffense> {
        let style = if body.is_some_and(|node| self.namespace(node, true)) {
            "no_empty_lines"
        } else {
            "empty_lines"
        };
        self.check_both(style, first_line, last_line)
    }

    fn check_empty_lines_special(
        &self,
        body: Option<NodeRef<'_>>,
        first_line: usize,
        last_line: usize,
    ) -> Vec<BodyOffense> {
        let Some(body) = body else { return Vec::new() };
        if self.namespace(body, true) {
            return self.check_both("no_empty_lines", first_line, last_line);
        }
        let mut offenses = if self.first_child_requires_empty_line(body) {
            self.check_beginning("empty_lines", first_line)
        } else {
            let mut result = self.check_beginning("no_empty_lines", first_line);
            result.extend(self.check_deferred_empty_line(body));
            result
        };
        offenses.extend(self.check_ending("empty_lines", last_line));
        offenses
    }

    fn check_both(&self, style: &str, first_line: usize, last_line: usize) -> Vec<BodyOffense> {
        let (beginning, ending) = match style {
            "beginning_only" => ("empty_lines", "no_empty_lines"),
            "ending_only" => ("no_empty_lines", "empty_lines"),
            _ => (style, style),
        };
        let mut offenses = self.check_beginning(beginning, first_line);
        offenses.extend(self.check_ending(ending, last_line));
        offenses
    }

    fn check_beginning(&self, style: &str, first_line: usize) -> Vec<BodyOffense> {
        self.check_source(style, first_line, "beginning")
    }

    fn check_ending(&self, style: &str, last_line: usize) -> Vec<BodyOffense> {
        self.check_source(style, last_line.saturating_sub(2), "end")
    }

    fn check_source(&self, style: &str, line: usize, description: &str) -> Vec<BodyOffense> {
        let message = match style {
            "no_empty_lines" => self.message(MSG_EXTRA, description),
            "empty_lines" => self.message(MSG_MISSING, description),
            _ => return Vec::new(),
        };
        self.check_line(style, line, message)
    }

    fn check_line(&self, style: &str, line: usize, message: String) -> Vec<BodyOffense> {
        let empty = self.lines.get(line).is_some_and(|source| source.is_empty());
        let violation = if style == "no_empty_lines" {
            empty
        } else {
            !empty
        };
        violation
            .then(|| BodyOffense {
                line: line
                    + if style == "empty_lines" && message.contains("end.") {
                        2
                    } else {
                        1
                    },
                insert_empty_line: style == "empty_lines",
                message,
            })
            .into_iter()
            .collect()
    }

    fn check_deferred_empty_line(&self, body: NodeRef<'_>) -> Vec<BodyOffense> {
        let Some(node) = self.first_empty_line_required_child(body) else {
            return Vec::new();
        };
        let line = self.previous_line_ignoring_comments(node.first_line());
        if self.lines.get(line).is_none_or(|source| source.is_empty()) {
            return Vec::new();
        }
        vec![BodyOffense {
            line: line + 2,
            insert_empty_line: true,
            message: self.deferred_message(node),
        }]
    }

    fn constant_definition(&self, node: NodeRef<'_>) -> bool {
        matches!(node.kind(), "class" | "module")
    }

    fn namespace(&self, body: NodeRef<'_>, with_one_child: bool) -> bool {
        if body.kind() == "begin" {
            !with_one_child
                && body
                    .child_nodes()
                    .into_iter()
                    .all(|child| self.constant_definition(child))
        } else {
            self.constant_definition(body)
        }
    }

    fn first_child_requires_empty_line(&self, body: NodeRef<'_>) -> bool {
        let node = if body.kind() == "begin" {
            body.child_nodes().first().copied()
        } else {
            Some(body)
        };
        node.is_some_and(Self::empty_line_required)
    }

    fn first_empty_line_required_child(&self, body: NodeRef<'a>) -> Option<NodeRef<'a>> {
        if body.kind() == "begin" {
            body.child_nodes()
                .into_iter()
                .find(|node| Self::empty_line_required(*node))
        } else {
            Self::empty_line_required(body).then_some(body)
        }
    }

    fn empty_line_required(node: NodeRef<'_>) -> bool {
        matches!(node.kind(), "def" | "defs" | "class" | "module")
            || (node.kind() == "send"
                && node
                    .method_name()
                    .is_some_and(|name| matches!(name, "private" | "protected" | "public")))
    }

    fn previous_line_ignoring_comments(&self, send_line: usize) -> usize {
        (0..send_line.saturating_sub(1))
            .rev()
            .find(|line| {
                self.lines
                    .get(*line)
                    .is_some_and(|source| !source.trim_start().starts_with('#'))
            })
            .unwrap_or(0)
    }

    fn message(&self, template: &str, description: &str) -> String {
        template
            .replace("%<kind>s", self.kind)
            .replace("%<location>s", description)
    }

    fn deferred_message(&self, node: NodeRef<'_>) -> String {
        format!("Empty line missing before first {} definition", node.kind())
    }

    fn valid_body_style(&self, body: Option<NodeRef<'_>>) -> bool {
        body.is_none() && self.style != "no_empty_lines"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_beginning_ending_and_empty_body_rules() {
        let lines = ["class C", "body", "end"];
        let checker = EmptyLinesAroundBody::new(&lines, "empty_lines", "class");
        let offenses = checker.check(None, 1, 3);
        assert!(
            offenses.is_empty(),
            "empty bodies are ignored for empty_lines"
        );

        let checker = EmptyLinesAroundBody::new(&lines, "no_empty_lines", "class");
        assert!(checker.check(None, 1, 3).is_empty());

        let lines = ["class C", "", "body", "", "end"];
        let checker = EmptyLinesAroundBody::new(&lines, "no_empty_lines", "class");
        let offenses = checker.check_both("no_empty_lines", 1, 5);
        assert_eq!(offenses.len(), 2);
        assert!(offenses.iter().all(|offense| !offense.insert_empty_line));
    }
}
