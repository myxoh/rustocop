// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/multiline_expression_indentation.rb
// Source SHA-256: e94106b7e5d3522b3fec5554ec0ddf07ccbd96e10c5ef1b8dfb82248d861efae

use std::ops::Range;

use crate::rubocop::ast::node::core::NodeRef;

const KEYWORD_ANCESTOR_TYPES: [&str; 5] = ["for", "if", "while", "until", "return"];
const UNALIGNED_RHS_TYPES: [&str; 7] =
    ["if", "while", "until", "for", "return", "array", "kwbegin"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct IndentationOffense {
    pub(crate) range: Range<usize>,
    pub(crate) message_tail: String,
}

pub(crate) struct MultilineExpressionIndentation {
    configured_indentation_width: usize,
    keyword_indentation_width: usize,
}

impl MultilineExpressionIndentation {
    pub(crate) fn new(
        configured_indentation_width: usize,
        keyword_indentation_width: usize,
    ) -> Self {
        Self {
            configured_indentation_width,
            keyword_indentation_width,
        }
    }

    fn on_send(&self, node: NodeRef<'_>) -> Option<IndentationOffense> {
        let receiver = node.receiver()?;
        if node.method_name() == Some("[]") || self.not_for_this_cop(node) {
            return None;
        }
        let lhs = self.left_hand_side(receiver);
        let rhs = node.arguments().first().copied()?;
        let range = rhs.source_range()?;
        self.check(Some(range), node, lhs, rhs)
    }

    fn on_csend(&self, node: NodeRef<'_>) -> Option<IndentationOffense> {
        self.on_send(node)
    }

    fn left_hand_side<'ast>(&self, mut lhs: NodeRef<'ast>) -> NodeRef<'ast> {
        while let Some(parent) = lhs.parent() {
            if !matches!(parent.kind(), "send" | "csend")
                || parent.loc("dot").is_none()
                || parent.assignment_method()
            {
                break;
            }
            lhs = parent;
        }
        lhs
    }

    fn correct_indentation(&self, node: NodeRef<'_>) -> usize {
        if self
            .kw_node_with_special_indentation(node)
            .is_some_and(|keyword| !self.postfix_conditional(keyword))
        {
            self.configured_indentation_width + self.keyword_indentation_width
        } else {
            self.configured_indentation_width
        }
    }

    fn check(
        &self,
        range: Option<Range<usize>>,
        node: NodeRef<'_>,
        lhs: NodeRef<'_>,
        rhs: NodeRef<'_>,
    ) -> Option<IndentationOffense> {
        range.map(|range| self.incorrect_style_detected(range, node, lhs, rhs))
    }

    fn incorrect_style_detected(
        &self,
        range: Range<usize>,
        node: NodeRef<'_>,
        _lhs: NodeRef<'_>,
        rhs: NodeRef<'_>,
    ) -> IndentationOffense {
        IndentationOffense {
            range,
            message_tail: self.operation_description(node, rhs),
        }
    }

    fn indentation(&self, node: NodeRef<'_>) -> usize {
        node.source().map_or(0, |source| {
            source
                .lines()
                .next()
                .unwrap_or_default()
                .chars()
                .position(|character| !character.is_whitespace())
                .unwrap_or(0)
        })
    }

    fn operation_description(&self, node: NodeRef<'_>, rhs: NodeRef<'_>) -> String {
        if let Some(keyword) = self.kw_node_with_special_indentation(node) {
            return self.keyword_message_tail(keyword);
        }
        if self.part_of_assignment_rhs(node, Some(rhs)).is_some() {
            return "an expression in an assignment".into();
        }
        "an expression".into()
    }

    fn keyword_message_tail(&self, node: NodeRef<'_>) -> String {
        let keyword = node
            .loc("keyword")
            .map_or(node.kind(), |(_, source)| source.as_str());
        let kind = if keyword == "for" {
            "collection"
        } else {
            "condition"
        };
        let article = if keyword.starts_with(['i', 'u']) {
            "an"
        } else {
            "a"
        };
        format!("a {kind} in {article} `{keyword}` statement")
    }

    fn kw_node_with_special_indentation<'ast>(&self, node: NodeRef<'ast>) -> Option<NodeRef<'ast>> {
        node.ancestors().into_iter().find(|ancestor| {
            KEYWORD_ANCESTOR_TYPES.contains(&ancestor.kind())
                && !(ancestor.kind() == "if" && ancestor.ternary())
                && self
                    .indented_keyword_expression(*ancestor)
                    .is_some_and(|outer| self.within_node(node, outer))
        })
    }

    fn indented_keyword_expression<'ast>(&self, node: NodeRef<'ast>) -> Option<NodeRef<'ast>> {
        if node.kind() == "for" {
            node.collection()
        } else {
            node.node_child(0)
        }
    }

    fn argument_in_method_call<'ast>(
        &self,
        node: NodeRef<'ast>,
        require_parentheses: bool,
    ) -> Option<NodeRef<'ast>> {
        for ancestor in node.ancestors() {
            if matches!(ancestor.kind(), "block" | "numblock" | "itblock") {
                return None;
            }
            if !matches!(ancestor.kind(), "send" | "csend") || ancestor.assignment_method() {
                continue;
            }
            if require_parentheses && !ancestor.parenthesized() {
                continue;
            }
            if ancestor
                .arguments()
                .iter()
                .any(|argument| self.within_node(node, *argument))
            {
                return Some(ancestor);
            }
        }
        None
    }

    fn part_of_assignment_rhs<'ast>(
        &self,
        node: NodeRef<'ast>,
        candidate: Option<NodeRef<'ast>>,
    ) -> Option<NodeRef<'ast>> {
        for ancestor in node.ancestors() {
            if candidate.is_some_and(|value| self.disqualified_rhs(value, ancestor)) {
                break;
            }
            if self.valid_rhs(candidate, ancestor) {
                return Some(ancestor);
            }
        }
        None
    }

    fn disqualified_rhs(&self, candidate: NodeRef<'_>, ancestor: NodeRef<'_>) -> bool {
        UNALIGNED_RHS_TYPES.contains(&ancestor.kind())
            || (matches!(ancestor.kind(), "block" | "numblock" | "itblock")
                && self.part_of_block_body(candidate, ancestor))
    }

    fn valid_rhs(&self, candidate: Option<NodeRef<'_>>, ancestor: NodeRef<'_>) -> bool {
        if matches!(ancestor.kind(), "send" | "csend") {
            candidate.is_some_and(|value| self.valid_method_rhs_candidate(value, ancestor))
        } else if ancestor.assignment() {
            self.valid_rhs_candidate(candidate, self.assignment_rhs(ancestor))
        } else {
            false
        }
    }

    fn valid_method_rhs_candidate(&self, candidate: NodeRef<'_>, node: NodeRef<'_>) -> bool {
        node.assignment_method()
            && self.valid_rhs_candidate(Some(candidate), node.arguments().last().copied())
    }

    fn valid_rhs_candidate(
        &self,
        candidate: Option<NodeRef<'_>>,
        node: Option<NodeRef<'_>>,
    ) -> bool {
        candidate.is_none()
            || candidate
                .zip(node)
                .is_some_and(|(inner, outer)| self.within_node(inner, outer))
    }

    fn part_of_block_body(&self, candidate: NodeRef<'_>, block: NodeRef<'_>) -> bool {
        block
            .body()
            .is_some_and(|body| self.within_node(candidate, body))
    }

    fn assignment_rhs<'ast>(&self, node: NodeRef<'ast>) -> Option<NodeRef<'ast>> {
        match node.kind() {
            "casgn" | "op_asgn" => node.rhs(),
            "send" | "csend" => node.arguments().last().copied(),
            _ => node.child_nodes().last().copied(),
        }
    }

    fn not_for_this_cop(&self, node: NodeRef<'_>) -> bool {
        node.ancestors().into_iter().any(|ancestor| {
            self.grouped_expression(ancestor) || self.inside_arg_list_parentheses(node, ancestor)
        })
    }

    fn grouped_expression(&self, node: NodeRef<'_>) -> bool {
        node.kind() == "begin" && node.loc("begin").is_some()
    }

    fn inside_arg_list_parentheses(&self, node: NodeRef<'_>, ancestor: NodeRef<'_>) -> bool {
        if !matches!(ancestor.kind(), "send" | "csend") || !ancestor.parenthesized() {
            return false;
        }
        let Some(inner) = node.source_range() else {
            return false;
        };
        let (Some((open, _)), Some((close, _))) = (ancestor.loc("begin"), ancestor.loc("end"))
        else {
            return false;
        };
        inner.start > open.start && inner.end < close.end
    }

    fn postfix_conditional(&self, node: NodeRef<'_>) -> bool {
        node.kind() == "if" && node.modifier_form()
    }

    fn within_node(&self, inner: NodeRef<'_>, outer: NodeRef<'_>) -> bool {
        inner
            .source_range()
            .zip(outer.source_range())
            .is_some_and(|(inner, outer)| inner.start >= outer.start && inner.end <= outer.end)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

    #[test]
    fn ports_assignment_rhs_containment_and_keyword_messages() {
        let source = ProcessedSource::new(
            "value = object.\n  call(argument)",
            3.4,
            None,
            ParserEngine::Prism,
        )
        .unwrap();
        let root = source.ast().unwrap();
        let call = root
            .each_descendant(&["send"])
            .into_iter()
            .find(|node| !node.arguments().is_empty())
            .unwrap();
        let argument = call.arguments()[0];
        let helper = MultilineExpressionIndentation::new(2, 2);
        assert!(helper.within_node(argument, call));
        assert!(helper.part_of_assignment_rhs(call, Some(call)).is_some());
        assert_eq!(helper.correct_indentation(call), 2);
        assert_eq!(helper.indentation(call), 0);
        assert!(helper
            .operation_description(call, argument)
            .contains("assignment"));
    }
}
