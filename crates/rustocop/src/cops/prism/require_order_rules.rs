use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> { Vec::new() }

fn require_order(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.receiver().is_some()
        || !matches!(call_name(node), b"require" | b"require_relative")
        || argument_count(node) == 0
    {
        return;
    }
    let Some(argument) = first_argument(node).and_then(|argument| argument.as_string_node()) else {
        return;
    };
    let key = argument.unescaped();
    let Some((search_start, search_end, container_ancestors)) =
        search_node(node, context.ancestors())
    else {
        return;
    };
    let Some(siblings) = containing_statements(container_ancestors, search_start) else {
        return;
    };
    let Some(index) = siblings
        .iter()
        .position(|sibling| sibling.location().start_offset() == search_start)
    else {
        return;
    };

    let mut older_sibling = false;
    for sibling in siblings[..index].iter().rev() {
        let Some(previous) = require_from_sibling(sibling) else {
            break;
        };
        if previous.receiver().is_some()
            || call_name(&previous) != call_name(node)
            || argument_count(&previous) == 0
            || context.source()[sibling.location().start_offset()..search_end].contains("\n\n")
        {
            break;
        }
        let Some(previous_key) = first_argument(&previous)
            .and_then(|argument| argument.as_string_node())
            .map(|string| string.unescaped().to_vec())
        else {
            break;
        };
        if key < previous_key.as_slice() {
            older_sibling = true;
            break;
        }
    }
    if !older_sibling {
        return;
    }

    let location = node.location();
    let range = location.start_offset()..location.end_offset();
    let message = format!(
        "Sort `{}` in alphabetical order.",
        String::from_utf8_lossy(call_name(node))
    );
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let line_index = lines
        .iter()
        .position(|(offset, line)| *offset <= range.start && range.start <= *offset + line.len())
        .unwrap_or(0);
    let kind = String::from_utf8_lossy(call_name(node));
    let eligible = |line: &str| {
        let trimmed = line.trim_start();
        trimmed.starts_with('#') || trimmed.starts_with(&format!("{kind} "))
    };
    let mut first = line_index;
    while first > 0 && eligible(lines[first - 1].1) && !lines[first - 1].1.trim().is_empty() {
        first -= 1;
    }
    let mut last = line_index + 1;
    while last < lines.len() && eligible(lines[last].1) && !lines[last].1.trim().is_empty() {
        last += 1;
    }
    let mut pending_comments = Vec::new();
    let mut units = Vec::<(String, Vec<String>)>::new();
    for (_, line) in &lines[first..last] {
        if line.trim_start().starts_with('#') {
            pending_comments.push((*line).to_string());
            continue;
        }
        let key = line
            .split(['\'', '"'])
            .nth(1)
            .unwrap_or_default()
            .to_string();
        pending_comments.push((*line).to_string());
        units.push((key, std::mem::take(&mut pending_comments)));
    }
    units.sort_by(|left, right| left.0.cmp(&right.0));
    let replacement = units
        .into_iter()
        .flat_map(|(_, lines)| lines)
        .chain(pending_comments)
        .collect::<Vec<_>>()
        .join("\n");
    let block_start = lines[first].0;
    let block_end = lines[last - 1].0 + lines[last - 1].1.len();
    context.replace(message, range, block_start..block_end, replacement);
}

fn search_node<'pr>(
    node: &CallNode<'pr>,
    ancestors: &'pr [Node<'pr>],
) -> Option<(usize, usize, &'pr [Node<'pr>])> {
    let parent = ancestors.last()?;
    if let Some(conditional) = parent.as_if_node() {
        let body = conditional.statements()?;
        if body.body().len() == 1 {
            let keyword = conditional.if_keyword_loc()?;
            if keyword.start_offset() < node.location().start_offset() {
                return None;
            }
            let location = conditional.location();
            return Some((
                location.start_offset(),
                location.end_offset(),
                &ancestors[..ancestors.len() - 1],
            ));
        }
    }
    if let Some(conditional) = parent.as_unless_node() {
        let body = conditional.statements()?;
        if body.body().len() == 1 {
            let keyword = conditional.keyword_loc();
            if keyword.start_offset() < node.location().start_offset() {
                return None;
            }
            let location = conditional.location();
            return Some((
                location.start_offset(),
                location.end_offset(),
                &ancestors[..ancestors.len() - 1],
            ));
        }
    }
    let location = node.location();
    Some((location.start_offset(), location.end_offset(), ancestors))
}

fn containing_statements<'pr>(ancestors: &[Node<'pr>], target: usize) -> Option<Vec<Node<'pr>>> {
    ancestors.iter().rev().find_map(|ancestor| {
        let statements = if let Some(program) = ancestor.as_program_node() {
            Some(program.statements())
        } else if let Some(class) = ancestor.as_class_node() {
            class.body().and_then(|body| body.as_statements_node())
        } else if let Some(module) = ancestor.as_module_node() {
            module.body().and_then(|body| body.as_statements_node())
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            singleton.body().and_then(|body| body.as_statements_node())
        } else if let Some(definition) = ancestor.as_def_node() {
            definition.body().and_then(|body| body.as_statements_node())
        } else if let Some(block) = ancestor.as_block_node() {
            block.body().and_then(|body| body.as_statements_node())
        } else if let Some(begin) = ancestor.as_begin_node() {
            begin.statements()
        } else if let Some(conditional) = ancestor.as_if_node() {
            conditional.statements()
        } else if let Some(conditional) = ancestor.as_unless_node() {
            conditional.statements()
        } else if let Some(branch) = ancestor.as_else_node() {
            branch.statements()
        } else if let Some(rescue) = ancestor.as_rescue_node() {
            rescue.statements()
        } else {
            None
        };
        statements.and_then(|statements| {
            let body = statements.body().iter().collect::<Vec<_>>();
            body.iter()
                .any(|statement| statement.location().start_offset() == target)
                .then_some(body)
        })
    })
}

fn require_from_sibling<'pr>(node: &Node<'pr>) -> Option<CallNode<'pr>> {
    if let Some(call) = node.as_call_node() {
        return Some(call);
    }
    if let Some(conditional) = node.as_if_node() {
        let keyword = conditional.if_keyword_loc()?;
        let body = conditional.statements()?;
        let statement = only_statement(Some(body))?;
        if keyword.start_offset() > statement.location().start_offset() {
            return statement.as_call_node();
        }
        return None;
    }
    if let Some(conditional) = node.as_unless_node() {
        let body = conditional.statements()?;
        let statement = only_statement(Some(body))?;
        if conditional.keyword_loc().start_offset() > statement.location().start_offset() {
            return statement.as_call_node();
        }
    }
    None
}
