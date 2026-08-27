use super::*;
use std::ops::Range;

define_cops! {
    TrailingUnderscoreVariable => "Style/TrailingUnderscoreVariable" => compatibility_prism_node(as_multi_write_node, trailing_underscore_variable),
}

fn trailing_underscore_variable(
    node: &ruby_prism::MultiWriteNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let mut ranges = Vec::new();
    collect_ranges(&TargetGroup::from_write(node), context, &mut ranges);
    let node_location = node.location();
    let node_start = node_location.start_offset();
    let node_source = context
        .source_file()
        .slice(node_start..node_location.end_offset())
        .unwrap_or_default();

    for range in ranges {
        let mut preferred = node_source.to_string();
        preferred.replace_range(range.start - node_start..range.end - node_start, "");
        let message = format!(
            "Do not use trailing `_`s in parallel assignment. Prefer `{preferred}`."
        );
        context.remove(message, range.clone(), range);
    }
}

fn collect_ranges<'pr>(
    group: &TargetGroup<'pr>,
    context: &CopContext<'_, '_>,
    ranges: &mut Vec<Range<usize>>,
) {
    for variable in &group.variables {
        if let Some(nested) = variable.as_multi_target_node() {
            collect_ranges(&TargetGroup::from_target(&nested), context, ranges);
        }
    }

    let Some(first_offense) = first_offense(group, context) else {
        return;
    };
    let offense_start = group.variables[first_offense].location().start_offset();
    let range = if first_offense == 0 {
        group.start..group.unused_all_end
    } else if group.parenthesized {
        offense_start.saturating_sub(1)..group.end.saturating_sub(1)
    } else {
        offense_start..group.trailing_end
    };
    ranges.push(range);
}

fn first_offense(group: &TargetGroup<'_>, context: &CopContext<'_, '_>) -> Option<usize> {
    let allow_named = context.config_bool("AllowNamedUnderscoreVariables", true);
    let mut first = None;
    for (index, variable) in group.variables.iter().enumerate().rev() {
        let Some(name) = underscore_target_name(variable) else {
            break;
        };
        if !name.starts_with(b"_") || allow_named && name != b"_" {
            break;
        }
        first = Some(index);
    }
    let first = first?;
    if group.variables[..first].iter().any(is_splat) {
        return None;
    }
    Some(first)
}

fn underscore_target_name<'pr>(node: &Node<'pr>) -> Option<&'pr [u8]> {
    if let Some(local) = node.as_local_variable_target_node() {
        return Some(local.name().as_slice());
    }
    node.as_splat_node()?
        .expression()?
        .as_local_variable_target_node()
        .map(|local| local.name().as_slice())
}

fn is_splat(node: &Node<'_>) -> bool {
    node.as_splat_node().is_some()
}

struct TargetGroup<'pr> {
    variables: Vec<Node<'pr>>,
    start: usize,
    end: usize,
    unused_all_end: usize,
    trailing_end: usize,
    parenthesized: bool,
}

impl<'pr> TargetGroup<'pr> {
    fn from_write(node: &ruby_prism::MultiWriteNode<'pr>) -> Self {
        let variables = node
            .lefts()
            .iter()
            .chain(node.rest().filter(|rest| rest.as_splat_node().is_some()))
            .chain(node.rights().iter())
            .collect::<Vec<_>>();
        let start = node
            .lparen_loc()
            .map_or_else(|| variables[0].location().start_offset(), |loc| loc.start_offset());
        let parenthesized = node.lparen_loc().is_some();
        let end = node
            .rparen_loc()
            .map_or_else(|| node.operator_loc().start_offset(), |loc| loc.end_offset());
        Self {
            variables,
            start,
            end,
            unused_all_end: node.value().location().start_offset(),
            trailing_end: node.operator_loc().start_offset(),
            parenthesized,
        }
    }

    fn from_target(node: &ruby_prism::MultiTargetNode<'pr>) -> Self {
        let variables = node
            .lefts()
            .iter()
            .chain(node.rest().filter(|rest| rest.as_splat_node().is_some()))
            .chain(node.rights().iter())
            .collect::<Vec<_>>();
        let location = node.location();
        Self {
            variables,
            start: location.start_offset(),
            end: location.end_offset(),
            unused_all_end: location.end_offset(),
            trailing_end: location.end_offset(),
            parenthesized: node.lparen_loc().is_some(),
        }
    }
}
