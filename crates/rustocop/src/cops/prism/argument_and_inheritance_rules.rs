use super::*;

define_cops! {
    CircularArgumentReference => "Lint/CircularArgumentReference" => any_node(circular_argument_reference),
    InheritException => "Lint/InheritException" => any_node(inherit_exception),
    NumberedParameterAssignment => "Lint/NumberedParameterAssignment" => node(as_local_variable_write_node, numbered_parameter_assignment),
    RaiseException => "Lint/RaiseException" => call(raise_exception),
    DateTime => "Style/DateTime" => call(date_time),
    RedundantArgument => "Style/RedundantArgument" => call(redundant_argument),
    YAMLFileRead => "Style/YAMLFileRead" => call(yaml_file_read),
}

fn raise_exception(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"raise" | b"fail") || node.receiver().is_some() {
        return;
    }
    let Some(argument) = first_argument(node) else {
        return;
    };
    let exception = if node_is_root_constant(&argument, b"Exception") {
        argument
    } else if let Some(constructor) = argument.as_call_node() {
        if call_name(&constructor) != b"new" {
            return;
        }
        let Some(receiver) = constructor.receiver() else {
            return;
        };
        if !node_is_root_constant(&receiver, b"Exception") {
            return;
        }
        receiver
    } else {
        return;
    };
    let source = context.source_file().node(&exception);
    if !source.starts_with("::") && inside_allowed_namespace(context) {
        return;
    }
    let edit = exception
        .as_constant_path_node()
        .map(|constant| constant.name_loc())
        .unwrap_or_else(|| exception.location());
    context.replace(
        "Use `StandardError` over `Exception`.",
        exception.location(),
        edit,
        "StandardError",
    );
}

fn inside_allowed_namespace(context: &CopContext<'_, '_>) -> bool {
    let allowed = context.config_values("AllowedImplicitNamespaces");
    context.ancestors().iter().rev().any(|ancestor| {
        ancestor.as_module_node().is_some_and(|module| {
            constant_path(&module.constant_path()).is_some_and(|parts| {
                let name = parts
                    .iter()
                    .map(|part| String::from_utf8_lossy(part))
                    .collect::<Vec<_>>()
                    .join("::");
                allowed.iter().any(|allowed| allowed == &name)
            })
        })
    })
}

fn circular_argument_reference(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let parameter = if let Some(parameter) = node.as_optional_parameter_node() {
        Some((parameter.name().as_slice(), parameter.value()))
    } else {
        node.as_optional_keyword_parameter_node()
            .map(|parameter| (parameter.name().as_slice(), parameter.value()))
    };
    let Some((name, mut value)) = parameter else {
        return;
    };
    let mut assigned = Vec::new();
    while let Some(write) = value.as_local_variable_write_node() {
        assigned.push(write.name().as_slice());
        value = write.value();
    }
    let Some(read) = value.as_local_variable_read_node() else {
        return;
    };
    if read.name().as_slice() != name && !assigned.contains(&read.name().as_slice()) {
        return;
    }
    context.report(
        format!(
            "Circular argument reference - `{}`.",
            String::from_utf8_lossy(name)
        ),
        read.location(),
    );
}

fn numbered_parameter_assignment(
    node: &ruby_prism::LocalVariableWriteNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let name = node.name().as_slice();
    let Some(digits) = name.strip_prefix(b"_") else {
        return;
    };
    if digits.is_empty() || !digits.iter().all(u8::is_ascii_digit) {
        return;
    }
    let Ok(number) = String::from_utf8_lossy(digits).parse::<usize>() else {
        return;
    };
    let message = if (1..=9).contains(&number) {
        format!("`_{number}` is reserved for numbered parameter; consider another name.")
    } else {
        format!("`_{number}` is similar to numbered parameter; consider another name.")
    };
    context.report(message, node.location());
}

fn inherit_exception(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let exception = if let Some(class) = node.as_class_node() {
        let parent = class
            .superclass()
            .filter(|parent| node_is_root_constant(parent, b"Exception"));
        if parent
            .as_ref()
            .is_some_and(|parent| parent.as_constant_read_node().is_some())
            && preceding_exception_class(context, &class)
        {
            return;
        }
        parent
    } else if let Some(call) = node.as_call_node() {
        if !match_call(&call)
            .named(b"new")
            .on_root_constant(b"Class")
            .with_argument_count(1)
            .matches()
        {
            return;
        }
        only_argument(&call).filter(|parent| node_is_root_constant(parent, b"Exception"))
    } else {
        None
    };
    let Some(exception) = exception else {
        return;
    };
    let preferred = if context.policy().enforced_style("standard_error") == "runtime_error" {
        "RuntimeError"
    } else {
        "StandardError"
    };
    context.replace_node(
        &exception,
        format!("Inherit from `{preferred}` instead of `Exception`."),
        preferred,
    );
}

fn preceding_exception_class(
    context: &CopContext<'_, '_>,
    current: &ruby_prism::ClassNode<'_>,
) -> bool {
    let Some(statements) = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_statements_node)
    else {
        return false;
    };
    statements
        .body()
        .iter()
        .take_while(|sibling| sibling.location().start_offset() < current.location().start_offset())
        .any(|sibling| {
            sibling
                .as_class_node()
                .is_some_and(|class| node_is_root_constant(&class.constant_path(), b"Exception"))
        })
}

fn date_time(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) == b"to_datetime"
        && node.receiver().is_some()
        && argument_count(node) == 0
        && !context.config_bool("AllowCoercion", false)
    {
        context.report_call(node, "Do not use `#to_datetime`.");
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    if !node_is_root_constant(&receiver, b"DateTime") || historic_date(node) {
        return;
    }
    let edit = receiver
        .as_constant_path_node()
        .map(|path| path.name_loc())
        .unwrap_or_else(|| receiver.location());
    context.replace(
        "Prefer `Time` over `DateTime`.",
        node.location(),
        edit,
        "Time",
    );
}

fn historic_date(node: &CallNode<'_>) -> bool {
    let Some(arguments) = node.arguments() else {
        return false;
    };
    arguments.arguments().iter().skip(1).any(|argument| {
        constant_path(&argument)
            .is_some_and(|path| path.first() == Some(&b"Date".as_slice()) && path.len() >= 2)
    })
}

fn redundant_argument(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let name = String::from_utf8_lossy(call_name(node));
    if node.receiver().is_none() && !matches!(name.as_ref(), "exit" | "exit!") {
        return;
    }
    let Some(argument) = only_argument(node) else {
        return;
    };
    let Some(methods) = context.config_map("Methods") else {
        return;
    };
    let Some(default) = methods.get(name.as_ref()).cloned() else {
        return;
    };
    let invalid_byte_default = methods
        .get("$hex")
        .or_else(|| methods.get("\"$hex\""))
        .is_some_and(|hex| hex == "82");
    let matches_default = if invalid_byte_default && name == "chomp" {
        argument
            .as_string_node()
            .is_some_and(|string| string.unescaped() == [0x82])
    } else if let Some(string) = argument.as_string_node() {
        let decoded = match default.as_str() {
            r"\n" => b"\n".as_slice(),
            _ => default.as_bytes(),
        };
        string.unescaped() == decoded
    } else if let Some(integer) = argument.as_integer_node() {
        default.parse::<i32>().ok().is_some_and(|expected| {
            TryInto::<i32>::try_into(integer.value()).ok() == Some(expected)
        })
    } else {
        argument.as_true_node().is_some() && default == "true"
            || argument.as_false_node().is_some() && default == "false"
    };
    if !matches_default {
        return;
    }
    let Some(selector) = node.message_loc() else {
        return;
    };
    let argument_source = context.source_file().node(&argument).to_string();
    let redundant_range = selector.end_offset()..node.location().end_offset();
    context.remove(
        format!("Argument {argument_source} is redundant because it is implied by default."),
        redundant_range.clone(),
        redundant_range,
    );
}

fn yaml_file_read(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"load" | b"safe_load" | b"parse")
        || !root_constant(node.receiver(), b"YAML")
        || call_name(node) == b"safe_load" && !context.target_ruby_version().at_least(3, 0)
    {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let values = arguments.arguments().iter().collect::<Vec<_>>();
    let Some(read) = values.first().and_then(|argument| argument.as_call_node()) else {
        return;
    };
    if !match_call(&read)
        .named(b"read")
        .on_root_constant(b"File")
        .with_argument_count(1)
        .matches()
    {
        return;
    }
    let Some(path) = only_argument(&read) else {
        return;
    };
    let Some(offense) = offense_range(node) else {
        return;
    };
    let preferred = preferred_yaml_file_read(node, &path, &values[1..], context.source_file());
    let message = format!("Use `{preferred}` instead.");
    context.add_offense(offense.clone(), message, |corrector| {
        corrector.replace(offense, preferred);
    });
}

fn offense_range(node: &CallNode<'_>) -> Option<std::ops::Range<usize>> {
    let selector = node.message_loc()?;
    Some(selector.start_offset()..node.location().end_offset())
}

fn preferred_yaml_file_read(
    node: &CallNode<'_>,
    path: &Node<'_>,
    rest: &[Node<'_>],
    file: SourceFile<'_>,
) -> String {
    let suffix = rest
        .iter()
        .map(|argument| file.node(argument))
        .collect::<Vec<_>>()
        .join(", ");
    let suffix = if suffix.is_empty() {
        String::new()
    } else {
        format!(", {suffix}")
    };
    format!(
        "{}_file({}{})",
        String::from_utf8_lossy(call_name(node)),
        file.node(path),
        suffix
    )
}
