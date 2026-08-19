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
    if let Some(path) = constant_path(node) {
        return Some(render_constant_path(&path));
    }
    let Representation::Name(method) = representation else {
        return None;
    };
    if let Some(string) = node.as_string_node() {
        let name = String::from_utf8_lossy(string.unescaped());
        if !valid_constant_name(&name) {
            return None;
        }
        let prefix = if context
            .ancestors()
            .iter()
            .any(|ancestor| ancestor.as_module_node().is_some())
        {
            "::"
        } else {
            ""
        };
        return Some(format!("{prefix}{name}"));
    }
    let call = node.as_call_node()?;
    if call_name(&call) != method || argument_count(&call) != 0 {
        return None;
    }
    constant_path(&call.receiver()?).map(|parts| render_constant_path(&parts))
}

fn render_constant_path(parts: &[&[u8]]) -> String {
    parts
        .iter()
        .map(|part| String::from_utf8_lossy(part))
        .collect::<Vec<_>>()
        .join("::")
}

fn valid_constant_name(name: &str) -> bool {
    name.split("::").all(|part| {
        part.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
            && part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
    })
}

fn allowed_method(context: &CopContext<'_, '_>) -> bool {
    context.ancestors().iter().rev().any(|ancestor| {
        ancestor
            .as_def_node()
            .is_some_and(|definition| context.policy().allows_method(definition.name().as_slice()))
    })
}
