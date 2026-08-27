use super::*;

define_cops! {
    ItWithoutArgumentsInBlock => "Lint/ItWithoutArgumentsInBlock" => node(as_it_local_variable_read_node, it_without_arguments_in_block),
}

fn it_without_arguments_in_block(
    node: &ruby_prism::ItLocalVariableReadNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if context.target_ruby_version().at_least(3, 4)
        || !context.ancestors().iter().rev().any(|ancestor| {
            ancestor.as_block_node().is_some_and(|block| {
                block
                    .parameters()
                    .is_some_and(|parameters| parameters.as_it_parameters_node().is_some())
            })
        })
    {
        return;
    }
    context.report(
        "`it` calls without arguments will refer to the first block param in Ruby 3.4; use `it()` or `self.it`.",
        node.location(),
    );
}

fn numbered_parameters_limit(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 7)
        || node
            .parameters()
            .is_none_or(|parameters| parameters.as_numbered_parameters_node().is_none())
    {
        return;
    }
    let mut counter = NumberedParameterCounter::default();
    if let Some(body) = node.body() {
        counter.visit(&body);
    }
    let count = counter.seen.iter().filter(|seen| **seen).count();
    let maximum = context.config_usize("Max", 1).min(9);
    if count <= maximum {
        return;
    }
    let parameter = if maximum == 1 {
        "parameter"
    } else {
        "parameters"
    };
    let message =
        format!("Avoid using more than {maximum} numbered {parameter}; {count} detected.");
    let offense = context
        .parent()
        .filter(|parent| parent.as_call_node().is_some())
        .map_or_else(|| node.location(), Node::location);
    context.report(message, offense);
}

#[derive(Default)]
struct NumberedParameterCounter {
    seen: [bool; 9],
}

impl<'pr> Visit<'pr> for NumberedParameterCounter {
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        let name = node.name().as_slice();
        if node.depth() == 0 && name.len() == 2 && name[0] == b'_' && name[1].is_ascii_digit() {
            let number = usize::from(name[1] - b'0');
            if (1..=9).contains(&number) {
                self.seen[number - 1] = true;
            }
        }
    }

    fn visit_block_node(&mut self, _node: &ruby_prism::BlockNode<'pr>) {}
}
