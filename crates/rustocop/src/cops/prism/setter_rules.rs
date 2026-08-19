use std::collections::HashMap;

use super::*;

define_cops! {
    UselessSetterCall => "Lint/UselessSetterCall" => node(as_def_node, useless_setter_call),
}

fn useless_setter_call(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(statements) = node.body().and_then(|body| body.as_statements_node()) else {
        return;
    };
    let Some(last) = statements.body().last() else {
        return;
    };
    let Some(setter) = last.as_call_node() else {
        return;
    };
    if setter.equal_loc().is_none() {
        return;
    }
    let Some(receiver) = setter
        .receiver()
        .and_then(|receiver| receiver.as_local_variable_read_node())
    else {
        return;
    };

    let mut tracker = LocalObjectTracker::default();
    tracker.visit_statements_node(&statements);
    let name = receiver.name().as_slice();
    if !tracker.local.get(name).copied().unwrap_or(false) {
        return;
    }

    let name = String::from_utf8_lossy(name);
    let message = format!("Useless setter call to local variable `{name}`.");
    let indentation = line_indentation(context.source(), last.location().start_offset());
    context.insert(
        message,
        receiver.location(),
        last.location().end_offset(),
        format!("\n{indentation}{name}"),
    );
}

fn line_indentation(source: &str, offset: usize) -> &str {
    let line_start = source[..offset].rfind('\n').map_or(0, |index| index + 1);
    let length = source[line_start..offset]
        .bytes()
        .take_while(u8::is_ascii_whitespace)
        .count();
    &source[line_start..line_start + length]
}

#[derive(Default)]
struct LocalObjectTracker {
    local: HashMap<Vec<u8>, bool>,
}

impl LocalObjectTracker {
    fn assign(&mut self, name: &[u8], value: &Node<'_>) {
        let local = if let Some(read) = value.as_local_variable_read_node() {
            self.local
                .get(read.name().as_slice())
                .copied()
                .unwrap_or(false)
        } else {
            constructed_locally(value)
        };
        self.local.insert(name.to_vec(), local);
    }

    fn assign_target(&mut self, target: &Node<'_>, value: Option<&Node<'_>>) {
        let Some(target) = target.as_local_variable_target_node() else {
            return;
        };
        let local = value.is_none_or(|value| constructed_locally_or_tracked(self, value));
        self.local.insert(target.name().as_slice().to_vec(), local);
    }
}

fn constructed_locally_or_tracked(tracker: &LocalObjectTracker, value: &Node<'_>) -> bool {
    value.as_local_variable_read_node().map_or_else(
        || constructed_locally(value),
        |read| {
            tracker
                .local
                .get(read.name().as_slice())
                .copied()
                .unwrap_or(false)
        },
    )
}

fn constructed_locally(node: &Node<'_>) -> bool {
    node.as_call_node()
        .is_some_and(|call| call_name(&call) == b"new")
        || node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_range_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
}

impl<'pr> Visit<'pr> for LocalObjectTracker {
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.assign(node.name().as_slice(), &node.value());
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        self.assign(node.name().as_slice(), &node.value());
    }

    fn visit_local_variable_and_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        self.assign(node.name().as_slice(), &node.value());
    }

    fn visit_local_variable_operator_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        self.local.insert(node.name().as_slice().to_vec(), true);
    }

    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode<'pr>) {
        let targets = node
            .lefts()
            .iter()
            .chain(node.rest())
            .chain(node.rights().iter())
            .collect::<Vec<_>>();
        let values = node
            .value()
            .as_array_node()
            .map(|array| array.elements().iter().collect::<Vec<_>>());
        for (index, target) in targets.iter().enumerate() {
            self.assign_target(target, values.as_ref().and_then(|values| values.get(index)));
        }
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
}
