use ruby_prism::{CallNode, Location, Node, StatementsNode};

use super::SourceFile;

/// Materializes call arguments when a cop needs to inspect or render the
/// complete argument list. Prefer `first_argument` or `only_argument` for the
/// common zero/one-argument cases.
pub(super) fn arguments<'pr>(node: &CallNode<'pr>) -> Vec<Node<'pr>> {
    node.arguments()
        .map(|arguments| arguments.arguments().iter().collect())
        .unwrap_or_default()
}

pub(super) fn joined_arguments<'source>(
    node: &CallNode<'_>,
    file: SourceFile<'source>,
    separator: &str,
) -> String {
    arguments(node)
        .iter()
        .map(|argument| file.node(argument))
        .collect::<Vec<_>>()
        .join(separator)
}

/// Returns the only statement in an optional Prism statement list.
pub(super) fn only_statement<'pr>(statements: Option<StatementsNode<'pr>>) -> Option<Node<'pr>> {
    statements.and_then(|statements| only_statement_in(&statements))
}

pub(super) fn only_statement_in<'pr>(statements: &StatementsNode<'pr>) -> Option<Node<'pr>> {
    (statements.body().len() == 1)
        .then(|| statements.body().first())
        .flatten()
}

/// Normalizes Prism bodies that may be either a direct expression or a
/// `StatementsNode` containing one expression.
pub(super) fn single_expression(node: Node<'_>) -> Option<Node<'_>> {
    node.as_statements_node()
        .map_or(Some(node), |statements| only_statement_in(&statements))
}

/// A normalized view over Ruby's four statement-modifier node shapes.
/// Consumers can implement rules such as nested modifiers and modifier-form
/// layout without branching over four unrelated Prism APIs.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum ModifierKind {
    If,
    Unless,
    While,
    Until,
}

#[allow(dead_code)]
pub(super) struct ModifierConditional<'pr> {
    pub(super) kind: ModifierKind,
    pub(super) location: Location<'pr>,
    pub(super) keyword: Location<'pr>,
    pub(super) predicate: Node<'pr>,
    pub(super) body: Node<'pr>,
}

impl<'pr> ModifierConditional<'pr> {
    pub(super) fn from_node(node: &Node<'pr>) -> Option<Self> {
        if let Some(condition) = node.as_if_node() {
            let keyword = condition.if_keyword_loc()?;
            if condition.end_keyword_loc().is_some()
                || condition.subsequent().is_some()
                || keyword.start_offset() == condition.location().start_offset()
            {
                return None;
            }
            return Some(Self {
                kind: ModifierKind::If,
                location: condition.location(),
                keyword,
                predicate: condition.predicate(),
                body: only_statement(condition.statements())?,
            });
        }
        if let Some(condition) = node.as_unless_node() {
            let keyword = condition.keyword_loc();
            if condition.end_keyword_loc().is_some()
                || condition.else_clause().is_some()
                || keyword.start_offset() == condition.location().start_offset()
            {
                return None;
            }
            return Some(Self {
                kind: ModifierKind::Unless,
                location: condition.location(),
                keyword,
                predicate: condition.predicate(),
                body: only_statement(condition.statements())?,
            });
        }
        if let Some(condition) = node.as_while_node() {
            let keyword = condition.keyword_loc();
            if condition.closing_loc().is_some()
                || condition.is_begin_modifier()
                || keyword.start_offset() == condition.location().start_offset()
            {
                return None;
            }
            return Some(Self {
                kind: ModifierKind::While,
                location: condition.location(),
                keyword,
                predicate: condition.predicate(),
                body: only_statement(condition.statements())?,
            });
        }
        let condition = node.as_until_node()?;
        let keyword = condition.keyword_loc();
        if condition.closing_loc().is_some()
            || condition.is_begin_modifier()
            || keyword.start_offset() == condition.location().start_offset()
        {
            return None;
        }
        Some(Self {
            kind: ModifierKind::Until,
            location: condition.location(),
            keyword,
            predicate: condition.predicate(),
            body: only_statement(condition.statements())?,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ruby_prism::parse;

    fn first_node(source: &[u8]) -> Node<'_> {
        let parsed = Box::leak(Box::new(parse(source)));
        parsed
            .node()
            .as_program_node()
            .unwrap()
            .statements()
            .body()
            .first()
            .unwrap()
    }

    #[test]
    fn normalizes_the_four_modifier_conditionals() {
        for (source, expected) in [
            (b"work if ready".as_slice(), ModifierKind::If),
            (b"work unless ready".as_slice(), ModifierKind::Unless),
            (b"work while ready".as_slice(), ModifierKind::While),
            (b"work until ready".as_slice(), ModifierKind::Until),
        ] {
            let modifier = ModifierConditional::from_node(&first_node(source)).unwrap();
            assert_eq!(modifier.kind, expected);
            assert_eq!(
                modifier.keyword.as_slice(),
                &source[5..modifier.keyword.end_offset()]
            );
        }
    }

    #[test]
    fn rejects_block_form_conditionals() {
        assert!(ModifierConditional::from_node(&first_node(b"if ready\n  work\nend")).is_none());
        assert!(ModifierConditional::from_node(&first_node(b"while ready\n  work\nend")).is_none());
    }
}
