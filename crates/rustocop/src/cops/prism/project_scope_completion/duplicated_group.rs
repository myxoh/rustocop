use crate::rubocop::ast::node::core::NodeRef;

use super::*;

const MSG_PREFIX: &[u8] = b"Gem group `";
const SOURCE_BLOCK_NAMES: &[&str] = &["source", "git", "platforms", "path"];

pub(super) fn on_new_investigation(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    if !relevant_file(context) || context.processed_source().blank() {
        return;
    }

    let Some(ast) = context.processed_source().ast() else {
        return;
    };
    for nodes in duplicated_group_nodes(ast) {
        let line_of_first_occurrence = nodes[0].first_line();
        for node in nodes.iter().skip(1).copied() {
            let mut group_name = Vec::new();
            for argument in node.arguments() {
                let Some(source) = argument.source_bytes() else {
                    continue;
                };
                if !group_name.is_empty() {
                    group_name.extend_from_slice(b", ");
                }
                group_name.extend_from_slice(&source);
            }
            register_offense(context, node, &group_name, line_of_first_occurrence);
        }
    }
}

fn relevant_file(context: &CompatibilityCopContext<'_, '_, '_>) -> bool {
    if context.config_contains("Include") {
        return true;
    }
    let filename = context.path().rsplit('/').next().unwrap_or(context.path());
    matches!(filename, "Gemfile" | "gems.rb") || filename.ends_with(".gemfile")
}

fn group_declarations(node: NodeRef<'_>) -> Vec<NodeRef<'_>> {
    node.each_node(&["send"])
        .into_iter()
        .filter(|declaration| {
            declaration.receiver().is_none() && declaration.method_name() == Some("group")
        })
        .collect()
}

fn duplicated_group_nodes(node: NodeRef<'_>) -> Vec<Vec<NodeRef<'_>>> {
    let mut groups = Vec::<(Vec<u8>, Vec<NodeRef<'_>>)>::new();
    for declaration in group_declarations(node) {
        let source_key = find_source_key(declaration).unwrap_or_default();
        let mut attributes = group_attributes(declaration);
        attributes.sort();
        let mut key = source_key;
        for attribute in attributes {
            key.extend(attribute);
        }
        if let Some((_, nodes)) = groups.iter_mut().find(|(candidate, _)| candidate == &key) {
            nodes.push(declaration);
        } else {
            groups.push((key, vec![declaration]));
        }
    }
    groups
        .into_iter()
        .map(|(_, nodes)| nodes)
        .filter(|nodes| nodes.len() > 1)
        .collect()
}

fn find_source_key(node: NodeRef<'_>) -> Option<Vec<u8>> {
    let source_block = node.each_ancestor(&["block"]).into_iter().find(|block| {
        block
            .method_name()
            .is_some_and(|name| SOURCE_BLOCK_NAMES.contains(&name))
    })?;
    let send = source_block.send_node()?;
    let method = source_block.method_name()?;
    let argument = send
        .first_argument()
        .and_then(NodeRef::source_bytes)
        .unwrap_or_default();
    let mut key = method.as_bytes().to_vec();
    key.extend_from_slice(&argument);
    Some(key)
}

fn group_attributes(node: NodeRef<'_>) -> Vec<Vec<u8>> {
    node.arguments()
        .into_iter()
        .map(|argument| {
            if argument.kind() == "hash" {
                let mut pairs = argument
                    .pairs()
                    .into_iter()
                    .filter_map(NodeRef::source_bytes)
                    .map(|source| source.into_owned())
                    .collect::<Vec<_>>();
                pairs.sort();
                let mut value = Vec::new();
                for (index, pair) in pairs.into_iter().enumerate() {
                    if index > 0 {
                        value.extend_from_slice(b", ");
                    }
                    value.extend(pair);
                }
                value
            } else {
                argument
                    .ruby_value_to_s_bytes()
                    .or_else(|| argument.source_bytes().map(|source| source.into_owned()))
                    .unwrap_or_default()
            }
        })
        .collect()
}

fn register_offense(
    context: &mut CompatibilityCopContext<'_, '_, '_>,
    node: NodeRef<'_>,
    group_name: &[u8],
    line_of_first_occurrence: usize,
) {
    let offense_location = context.owned_range(context.range_help().source_range_columns(
        context.source_buffer(),
        node.first_line(),
        node.column()..node.last_column(),
    ));
    context.report_bytes(
        offense_message(group_name, line_of_first_occurrence),
        offense_location,
    );
}

fn offense_message(group_name: &[u8], line_of_first_occurrence: usize) -> Vec<u8> {
    let mut message = MSG_PREFIX.to_vec();
    message.extend_from_slice(group_name);
    message.extend_from_slice(
        format!("` already defined on line {line_of_first_occurrence} of the Gemfile.").as_bytes(),
    );
    message
}

#[cfg(test)]
mod tests {
    use super::offense_message;

    #[test]
    fn message_preserves_group_name_bytes() {
        let message = offense_message(&[0xff], 3);
        let mut expected = b"Gem group `".to_vec();
        expected.push(0xff);
        expected.extend_from_slice(b"` already defined on line 3 of the Gemfile.");
        assert_eq!(message, expected);
    }
}
