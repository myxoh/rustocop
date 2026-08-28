use crate::rubocop::ast::node::core::NodeRef;

use super::*;

pub(super) fn duplicated_gem(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    if !relevant_file(context) || context.processed_source().blank() {
        return;
    }

    let Some(ast) = context.processed_source().ast() else {
        return;
    };
    for nodes in duplicated_gem_nodes(ast) {
        let line_of_first_occurrence = nodes[0].first_line();
        for node in nodes.iter().skip(1).copied() {
            let Some(gem_name) = node.first_argument().and_then(NodeRef::str_content_bytes) else {
                continue;
            };
            register_offense(context, node, gem_name, line_of_first_occurrence);
        }
    }
}

fn relevant_file(context: &CompatibilityCopContext<'_, '_, '_>) -> bool {
    if context.config_contains("Include") {
        // The engine and reporter have already applied the resolved cop path
        // policy. Re-matching Include here would duplicate that selection and
        // can only narrow RuboCop's glob semantics.
        return true;
    }
    let filename = context.path().rsplit('/').next().unwrap_or(context.path());
    matches!(filename, "Gemfile" | "gems.rb") || filename.ends_with(".gemfile")
}

fn gem_declarations(node: NodeRef<'_>) -> Vec<NodeRef<'_>> {
    node.each_node(&["send"])
        .into_iter()
        .filter(|declaration| {
            declaration.receiver().is_none()
                && declaration.method_name() == Some("gem")
                && declaration
                    .first_argument()
                    .is_some_and(|argument| argument.kind() == "str")
        })
        .collect()
}

fn duplicated_gem_nodes(node: NodeRef<'_>) -> Vec<Vec<NodeRef<'_>>> {
    let mut groups: Vec<(NodeRef<'_>, Vec<NodeRef<'_>>)> = Vec::new();
    for declaration in gem_declarations(node) {
        let first_argument = declaration
            .first_argument()
            .expect("gem_declarations requires a first argument");
        if let Some((_, nodes)) = groups.iter_mut().find(|(argument, _)| {
            argument.rubocop_hash_equivalent(first_argument)
                && argument.structurally_equal(first_argument)
        }) {
            nodes.push(declaration);
        } else {
            groups.push((first_argument, vec![declaration]));
        }
    }
    groups
        .into_iter()
        .map(|(_, nodes)| nodes)
        .filter(|nodes| nodes.len() > 1 && !conditional_declaration(nodes))
        .collect()
}

fn conditional_declaration(nodes: &[NodeRef<'_>]) -> bool {
    let Some(parent) = nodes[0]
        .ancestors()
        .into_iter()
        .find(|ancestor| ancestor.kind() != "begin")
    else {
        return false;
    };
    if !matches!(parent.kind(), "if" | "when") {
        return false;
    }
    let Some(root_conditional_node) = (parent.kind() == "if")
        .then_some(parent)
        .or_else(|| parent.parent())
    else {
        return false;
    };
    nodes
        .iter()
        .all(|node| within_conditional(*node, root_conditional_node))
}

fn within_conditional(node: NodeRef<'_>, conditional_node: NodeRef<'_>) -> bool {
    conditional_node
        .branches()
        .into_iter()
        .flatten()
        .any(|branch| {
            branch.structurally_equal(node)
                || branch
                    .child_nodes()
                    .into_iter()
                    .any(|child| child.structurally_equal(node))
        })
}

fn register_offense(
    context: &mut CompatibilityCopContext<'_, '_, '_>,
    node: NodeRef<'_>,
    gem_name: &[u8],
    line_of_first_occurrence: usize,
) {
    let offense_location = context.owned_range(context.range_help().source_range_columns(
        context.source_buffer(),
        node.first_line(),
        node.column()..node.last_column(),
    ));
    context.report_bytes(offense_message(gem_name, line_of_first_occurrence), offense_location);
}

fn offense_message(gem_name: &[u8], line_of_first_occurrence: usize) -> Vec<u8> {
    let mut message = b"Gem `".to_vec();
    message.extend_from_slice(gem_name);
    message.extend_from_slice(
        format!(
            "` requirements already given on line {line_of_first_occurrence} of the Gemfile."
        )
        .as_bytes(),
    );
    message
}

#[cfg(test)]
mod tests {
    use super::offense_message;

    #[test]
    fn duplicated_gem_message_preserves_the_exact_ruby_string_bytes() {
        let message = offense_message(&[0xff], 1);
        let mut expected = b"Gem `".to_vec();
        expected.push(0xff);
        expected.extend_from_slice(b"` requirements already given on line 1 of the Gemfile.");
        assert_eq!(message, expected);
        assert!(std::str::from_utf8(&message).is_err());
    }
}
