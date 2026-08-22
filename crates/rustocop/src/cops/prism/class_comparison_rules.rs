use super::*;

define_cops! {
    ClassEqualityComparison => "Style/ClassEqualityComparison" => call(class_equality_comparison),
}

fn class_equality_comparison(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"==" | b"equal?" | b"eql?") || allowed_method(context) {
        return;
    }
    let (Some(left), Some(right)) = (node.receiver(), only_argument(node)) else {
        return;
    };
    let Some((class_call, representation)) = class_representation(&left) else {
        return;
    };
    let Some(class_selector) = class_call.message_loc() else {
        return;
    };
    if context.source_file().node(&right).contains("#{") {
        return;
    }
    let target = comparison_target(&right, representation, context);
    if target.is_none() && right.as_interpolated_string_node().is_some() {
        return;
    }
    let offense = class_selector.start_offset()..node.location().end_offset();
    let message = target.as_ref().map_or_else(
        || "Use `instance_of?` instead of comparing classes.".to_string(),
        |target| format!("Use `instance_of?({target})` instead of comparing classes."),
    );
    if let Some(target) = target {
        context.replace(
            message,
            offense.clone(),
            offense,
            format!("instance_of?({target})"),
        );
    } else {
        context.report(message, offense);
    }
}

#[derive(Clone, Copy)]
enum Representation {
    Class,
    Name(&'static [u8]),
}

fn class_representation<'pr>(node: &Node<'pr>) -> Option<(CallNode<'pr>, Representation)> {
    let call = node.as_call_node()?;
    if call_name(&call) == b"class" && argument_count(&call) == 0 && call.receiver().is_some() {
        return Some((call, Representation::Class));
    }
    if !matches!(call_name(&call), b"name" | b"to_s" | b"inspect") || argument_count(&call) != 0 {
        return None;
    }
    let class_call = call.receiver()?.as_call_node()?;
    (call_name(&class_call) == b"class"
        && argument_count(&class_call) == 0
        && class_call.receiver().is_some())
    .then(|| {
        let representation = match call_name(&call) {
            b"name" => Representation::Name(b"name"),
            b"to_s" => Representation::Name(b"to_s"),
            _ => Representation::Name(b"inspect"),
        };
        (class_call, representation)
    })
}

fn comparison_target(
    node: &Node<'_>,
    representation: Representation,
    context: &CopContext<'_, '_>,
) -> Option<String> {
    let Representation::Name(method) = representation else {
        return Some(context.source_file().node(node).to_string());
    };
    if let Some(string) = node.as_string_node() {
        let name = String::from_utf8_lossy(string.unescaped());
        let prefix = if context
            .ancestors()
            .iter()
            .any(|ancestor| {
                ancestor.as_class_node().is_some() || ancestor.as_module_node().is_some()
            })
        {
            "::"
        } else {
            ""
        };
        return Some(format!("{prefix}{name}"));
    }
    if let Some(call) = node.as_call_node() {
        if call_name(&call) == method && argument_count(&call) == 0 {
            let receiver = call.receiver()?;
            if constant_path(&receiver).is_some() {
                return Some(context.source_file().node(&receiver).to_string());
            }
            if let Some(class_call) = receiver.as_call_node() {
                if call_name(&class_call) == b"class" && argument_count(&class_call) == 0 {
                    return Some(context.source_file().node(&class_call.as_node()).to_string());
                }
            }
        }
        return None;
    }
    if node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
    {
        return None;
    }
    Some(context.source_file().node(node).to_string())
}

fn allowed_method(context: &CopContext<'_, '_>) -> bool {
    context.ancestors().iter().rev().any(|ancestor| {
        ancestor
            .as_def_node()
            .is_some_and(|definition| context.policy().allows_method(definition.name().as_slice()))
    })
}
