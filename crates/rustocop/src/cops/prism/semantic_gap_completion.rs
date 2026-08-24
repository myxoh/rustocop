use super::*;
use std::collections::HashSet;

define_cops! {
    UnusedMethodArgument => "Lint/UnusedMethodArgument" => node(as_def_node, unused_method_argument),
    UselessMethodDefinition => "Lint/UselessMethodDefinition" => node(as_def_node, useless_method_definition),
    ConstantOverwrittenInRescue => "Lint/ConstantOverwrittenInRescue" => source(constant_overwritten_in_rescue),
    RedundantAssignment => "Style/RedundantAssignment" => node(as_def_node, redundant_assignment),
    ConstantResolution => "Lint/ConstantResolution" => source(constant_resolution),
    ReturnInVoidContext => "Lint/ReturnInVoidContext" => node(as_return_node, return_in_void_context),
    AmbiguousEndlessMethodDefinition => "Style/AmbiguousEndlessMethodDefinition" => node(as_def_node, ambiguous_endless_method_definition),
    NestedMethodDefinition => "Lint/NestedMethodDefinition" => node(as_def_node, nested_method_definition),
    UselessConstantScoping => "Lint/UselessConstantScoping" => source(useless_constant_scoping),
}

struct MethodArgument<'pr> {
    name: String,
    location: ruby_prism::Location<'pr>,
    keyword: bool,
    block: bool,
}

fn unused_method_argument(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(parameters) = node.parameters() else { return };
    let mut arguments = Vec::new();
    for parameter in parameters.requireds().iter().chain(parameters.posts().iter()) {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            arguments.push(MethodArgument { name: String::from_utf8_lossy(parameter.name().as_slice()).into_owned(), location: parameter.location(), keyword: false, block: false });
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            arguments.push(MethodArgument { name: String::from_utf8_lossy(parameter.name().as_slice()).into_owned(), location: parameter.name_loc(), keyword: false, block: false });
        }
    }
    if let Some(parameter) = parameters.rest().and_then(|parameter| parameter.as_rest_parameter_node()) {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            arguments.push(MethodArgument { name: String::from_utf8_lossy(name.as_slice()).into_owned(), location, keyword: false, block: false });
        }
    }
    for parameter in parameters.keywords().iter() {
        if let Some(parameter) = parameter.as_required_keyword_parameter_node() {
            arguments.push(MethodArgument { name: String::from_utf8_lossy(parameter.name().as_slice()).into_owned(), location: parameter.name_loc(), keyword: true, block: false });
        } else if let Some(parameter) = parameter.as_optional_keyword_parameter_node() {
            arguments.push(MethodArgument { name: String::from_utf8_lossy(parameter.name().as_slice()).into_owned(), location: parameter.name_loc(), keyword: true, block: false });
        }
    }
    if let Some(parameter) = parameters.keyword_rest().and_then(|parameter| parameter.as_keyword_rest_parameter_node()) {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            arguments.push(MethodArgument { name: String::from_utf8_lossy(name.as_slice()).into_owned(), location, keyword: true, block: false });
        }
    }
    if let Some(parameter) = parameters.block() {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            arguments.push(MethodArgument { name: String::from_utf8_lossy(name.as_slice()).into_owned(), location, keyword: false, block: true });
        }
    }
    if arguments.is_empty() { return; }

    if node.body().is_none() && context.config_bool("IgnoreEmptyMethods", false) { return; }
    let body_source = node.body().map_or("", |body| context.source_file().node(&body));
    if context.config_bool("IgnoreNotImplementedMethods", false) {
        let exceptions = context.config_values("NotImplementedExceptions");
        let not_implemented = body_source.contains("fail") || if exceptions.is_empty() {
            body_source.contains("NotImplementedError")
        } else {
            exceptions.iter().any(|exception| body_source.contains(exception))
        };
        if not_implemented { return; }
    }
    let mut usage = MethodArgumentUsage { reads: HashSet::new(), forwarding_super: false, binding: false, yield_seen: false };
    if let Some(body) = node.body() { usage.visit(&body); }
    if usage.forwarding_super || usage.binding { return; }
    let allow_keywords = context.config_bool("AllowUnusedKeywordArguments", false);
    let unused = arguments.iter().filter(|argument| {
        !argument.name.starts_with('_')
            && !(argument.keyword && allow_keywords)
            && !usage.reads.contains(&argument.name)
            && !(argument.block && usage.yield_seen)
    }).collect::<Vec<_>>();
    let relevant = arguments.iter().filter(|argument| !argument.name.starts_with('_') && !(argument.keyword && allow_keywords)).count();
    let all_unused = unused.len() == relevant;
    for argument in unused {
        let location = argument.location.start_offset()..argument.location.start_offset() + argument.name.len();
        if argument.keyword {
            context.report(format!("Unused method argument - `{}`.", argument.name), location);
            continue;
        }
        let mut message = format!("Unused method argument - `{0}`. If it's necessary, use `_` or `_{0}` as an argument name to indicate that it won't be used. If it's unnecessary, remove it.", argument.name);
        if all_unused {
            let method = String::from_utf8_lossy(node.name().as_slice());
            message.push_str(&format!(" You can also write as `{method}(*)` if you want the method to accept any arguments but don't care about them."));
        }
        if argument.block {
            let parameter_source = context.source_file().at(&parameters.location());
            let relative = argument.location.start_offset() - parameters.location().start_offset();
            let edit_start = parameter_source[..relative].rfind(',').map_or(argument.location.start_offset().saturating_sub(1), |comma| parameters.location().start_offset() + comma);
            context.remove(message, location, edit_start..argument.location.end_offset());
        } else {
            context.replace(message, location.clone(), location, format!("_{}", argument.name));
        }
    }
}

struct MethodArgumentUsage {
    reads: HashSet<String>,
    forwarding_super: bool,
    binding: bool,
    yield_seen: bool,
}

impl<'pr> Visit<'pr> for MethodArgumentUsage {
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        self.reads.insert(String::from_utf8_lossy(node.name().as_slice()).into_owned());
    }
    fn visit_forwarding_super_node(&mut self, _node: &ruby_prism::ForwardingSuperNode<'pr>) { self.forwarding_super = true; }
    fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode<'pr>) { self.yield_seen = true; ruby_prism::visit_yield_node(self, node); }
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if node.receiver().is_none() && node.name().as_slice() == b"binding" && node.arguments().is_none() { self.binding = true; }
        ruby_prism::visit_call_node(self, node);
    }
    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
}

fn useless_method_definition(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let line_start = context
        .source_file()
        .line_start(node.location().start_offset());
    let prefix = context.source()[line_start..node.location().start_offset()].trim();
    if !prefix.is_empty()
        && !matches!(
            prefix,
            "public" | "private" | "protected" | "module_function"
        )
        && !prefix.ends_with('{')
    {
        return;
    }
    let Some(body) = node.body().and_then(single_expression) else {
        return;
    };
    let forwarding = body
        .as_forwarding_super_node()
        .is_some_and(|super_node| super_node.block().is_none());
    if node.parameters().is_some_and(|parameters| {
        context
            .source_file()
            .at(&parameters.location())
            .bytes()
            .any(|byte| matches!(byte, b'*' | b'=' | b':'))
    })
    {
        return;
    }
    let explicit = body.as_super_node().is_some_and(|super_node| {
        if super_node.block().is_some() {
            return false;
        }
        let parameters = node.parameters().map(|parameters| {
            context
                .source_file()
                .at(&parameters.location())
                .trim_matches(['(', ')'])
                .replace(' ', "")
        });
        let arguments = context.source_file().at(&super_node.location());
        let rendered = arguments
            .trim_start_matches("super")
            .trim_matches(['(', ')'])
            .replace(' ', "");
        rendered.is_empty() && parameters.as_deref().is_none_or(str::is_empty)
            || parameters.is_some_and(|parameters| rendered == parameters)
    });
    if !forwarding && !explicit {
        return;
    }
    let location = node.location();
    let edit_start = if matches!(
        prefix,
        "public" | "private" | "protected" | "module_function"
    ) {
        line_start
    } else {
        location.start_offset()
    };
    let edit_end = location.end_offset();
    context.remove(
        "Useless method definition detected.",
        &location,
        edit_start..edit_end,
    );
}

fn constant_overwritten_in_rescue(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(marker) = line.find("rescue => ") else {
            continue;
        };
        let value = line[marker + 10..]
            .split('#')
            .next()
            .unwrap_or_default()
            .trim();
        let constant = value.trim_start_matches("::");
        let final_name = constant.rsplit("::").next().unwrap_or_default();
        if constant.is_empty()
            || !final_name
                .as_bytes()
                .first()
                .is_some_and(u8::is_ascii_uppercase)
        {
            continue;
        }
        let arrow = offset + marker + 7;
        context.replace(
            format!("`{value}` is overwritten by `rescue =>`."),
            arrow..arrow + 2,
            arrow..arrow + 3,
            "",
        );
    }
}

fn redundant_assignment(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let location = node.location();
    let ensure_start = context
        .source_file()
        .lines()
        .find(|(offset, line)| {
            location.start_offset() <= *offset
                && *offset < location.end_offset()
                && line.trim_start().starts_with("ensure")
        })
        .map(|(offset, _)| offset);
    let lines = context
        .source_file()
        .lines()
        .filter(|(offset, _)| location.start_offset() <= *offset && *offset < location.end_offset())
        .filter(|(offset, _)| ensure_start.is_none_or(|ensure| *offset > ensure))
        .filter(|(_, line)| !line.trim().is_empty() && !line.trim_start().starts_with('#'))
        .collect::<Vec<_>>();
    for (pair_index, pair) in lines.windows(2).enumerate() {
        let (assignment_start, assignment) = pair[0];
        let (return_start, returned) = pair[1];
        let Some((left, right)) = assignment.trim().split_once(" = ") else {
            continue;
        };
        if assignment.contains(" if ") || assignment.contains(" unless ") {
            continue;
        }
        let assignment_indent = assignment.len() - assignment.trim_start().len();
        let enclosing = lines[..=pair_index].iter().rev().find(|(_, previous)| {
            previous.len() - previous.trim_start().len() < assignment_indent
        });
        if enclosing.is_some_and(|(_, previous)| {
            previous.trim_end().ends_with(" do") || previous.contains(" do |")
        }) {
            continue;
        }
        if returned.trim() != left
            || left.is_empty()
            || !left
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            continue;
        }
        if lines
            .get(pair_index + 2)
            .is_some_and(|(_, following)| following.trim_start().starts_with('.'))
        {
            continue;
        }
        let indent = assignment.len() - assignment.trim_start().len();
        let offense = assignment_start + indent..assignment_start + assignment.len();
        let returned_start = return_start + returned.len() - returned.trim_start().len();
        context.replace_many(
            "Redundant assignment before returning detected.",
            offense.clone(),
            vec![
                (offense, right.to_string()),
                (returned_start..returned_start + left.len(), String::new()),
            ],
        );
    }
    for (index, (assignment_start, assignment)) in lines.iter().enumerate() {
        let trimmed = assignment.trim();
        let Some((left, right)) = trimmed.split_once(" = ") else {
            continue;
        };
        if !left
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            continue;
        }
        let mut balance = delimiter_balance(right);
        if balance <= 0 {
            continue;
        }
        let mut end_index = None;
        for (candidate_index, (_, candidate)) in lines.iter().enumerate().skip(index + 1) {
            balance += delimiter_balance(candidate);
            if balance == 0 {
                end_index = Some(candidate_index);
                break;
            }
        }
        let Some(end_index) = end_index else { continue };
        let Some((return_start, returned)) = lines.get(end_index + 1).copied() else {
            continue;
        };
        if returned.trim() != left {
            continue;
        }
        let indentation = assignment.len() - assignment.trim_start().len();
        let expression_start = *assignment_start + assignment.find(" = ").unwrap_or(0) + 3;
        let expression_end = lines[end_index].0 + lines[end_index].1.len();
        let offense = *assignment_start + indentation..expression_end;
        let returned_start = return_start + returned.len() - returned.trim_start().len();
        context.replace_many(
            "Redundant assignment before returning detected.",
            offense.clone(),
            vec![
                (offense, context.source()[expression_start..expression_end].to_string()),
                (returned_start..returned_start + left.len(), String::new()),
            ],
        );
    }
    for (index, (assignment_start, assignment)) in lines.iter().enumerate() {
        let trimmed = assignment.trim();
        let Some((left, right)) = trimmed.split_once(" = ") else { continue };
        if !left.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            || !matches!(right, "if" | "unless" | "case" | "begin")
                && !right.starts_with("if ")
                && !right.starts_with("unless ")
                && !right.starts_with("case ")
        {
            continue;
        }
        let indentation = assignment.len() - assignment.trim_start().len();
        let Some(end_index) = lines.iter().enumerate().skip(index + 1).find_map(|(candidate_index, (_, candidate))| {
            (candidate.trim() == "end" && candidate.len() - candidate.trim_start().len() == indentation).then_some(candidate_index)
        }) else { continue };
        let Some((return_start, returned)) = lines.get(end_index + 1) else { continue };
        if returned.trim() != left { continue; }
        let expression_end = lines[end_index].0 + lines[end_index].1.len();
        let offense = *assignment_start + indentation..expression_end;
        let expression_start = *assignment_start + assignment.find(" = ").unwrap_or(0) + 3;
        let returned_start = *return_start + returned.len() - returned.trim_start().len();
        context.replace_many(
            "Redundant assignment before returning detected.",
            offense.clone(),
            vec![
                (offense, context.source()[expression_start..expression_end].to_string()),
                (returned_start..returned_start + left.len(), String::new()),
            ],
        );
    }
    for (index, (assignment_start, assignment)) in lines.iter().enumerate() {
        let trimmed = assignment.trim();
        let Some((left, _)) = trimmed.split_once(" = ") else {
            continue;
        };
        if !trimmed.ends_with(" do") && !trimmed.contains(" do |") {
            continue;
        }
        let indentation = assignment.len() - assignment.trim_start().len();
        let Some(end_index) = lines.iter().enumerate().skip(index + 1).find_map(
            |(candidate_index, (_, candidate))| {
                (candidate.trim() == "end"
                    && candidate.len() - candidate.trim_start().len() == indentation)
                    .then_some(candidate_index)
            },
        ) else {
            continue;
        };
        let Some((return_start, returned)) = lines.get(end_index + 1).copied() else {
            continue;
        };
        if returned.trim() != left {
            continue;
        }
        let end = lines[end_index].0 + lines[end_index].1.len();
        let offense = *assignment_start + indentation..end;
        let expression_start = *assignment_start + assignment.find(" = ").unwrap_or(0) + 3;
        let returned_start = return_start + returned.len() - returned.trim_start().len();
        context.replace_many(
            "Redundant assignment before returning detected.",
            offense.clone(),
            vec![
                (offense, context.source()[expression_start..end].to_string()),
                (returned_start..returned_start + left.len(), String::new()),
            ],
        );
    }
}

fn delimiter_balance(source: &str) -> isize {
    source.bytes().fold(0, |balance, byte| match byte {
        b'(' | b'[' | b'{' => balance + 1,
        b')' | b']' | b'}' => balance - 1,
        _ => balance,
    })
}

fn constant_resolution(context: &mut CopContext<'_, '_>) {
    let source = context.source().trim();
    if source.is_empty()
        || source.starts_with("::")
        || source.starts_with("module ")
        || source.starts_with("class ")
        || source == "__ENCODING__"
    {
        return;
    }
    let root = source.split("::").next().unwrap_or(source);
    if !root
        .bytes()
        .next()
        .is_some_and(|byte| byte.is_ascii_uppercase())
    {
        return;
    }
    let only = context.config_values("Only");
    let ignore = context.config_values("Ignore");
    if !only.is_empty() && !only.iter().any(|name| name == root)
        || ignore.iter().any(|name| name == root)
    {
        return;
    }
    context.report(
        "Fully qualify this constant to avoid possibly ambiguous resolution.",
        0..root.len(),
    );
}

fn return_in_void_context(node: &ruby_prism::ReturnNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .arguments()
        .is_none_or(|arguments| arguments.arguments().is_empty())
    {
        return;
    }
    let Some(definition) = context.ancestors().iter().rev().find_map(Node::as_def_node) else {
        return;
    };
    if definition.receiver().is_some() {
        return;
    }
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_lambda_node().is_some()
            || context
                .source_file()
                .at(&ancestor.location())
                .trim_start()
                .starts_with("lambda do")
    }) {
        return;
    }
    if context
        .ancestors()
        .iter()
        .filter_map(Node::as_call_node)
        .any(|call| {
            matches!(
                call_name(&call),
                b"define_method" | b"define_singleton_method"
            )
        })
    {
        return;
    }
    let name = String::from_utf8_lossy(definition.name().as_slice());
    if name != "initialize" && !return_void_setter_name(name.as_bytes()) {
        return;
    }
    let start = node.location().start_offset();
    context.report(
        format!("Do not return a value in `{name}`."),
        start..start + 6,
    );
}

fn return_void_setter_name(name: &[u8]) -> bool {
    name.ends_with(b"=") && !matches!(name, b"==" | b"===" | b"!=" | b"<=" | b">=")
}

fn ambiguous_endless_method_definition(
    node: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if !context.target_ruby_version().at_least(3, 0) || node.equal_loc().is_none() {
        return;
    }
    let Some(operation) = context
        .ancestors()
        .iter()
        .rev()
        .find(|ancestor| ancestor.as_statements_node().is_none())
    else {
        return;
    };
    let Some((offense, keyword)) = ambiguous_endless_operation(operation, node) else {
        return;
    };
    let Some(replacement) = multiline_endless_method(node, context) else {
        return;
    };
    context.replace(
        format!("Avoid using `{keyword}` statements with endless methods."),
        offense,
        node.location(),
        replacement,
    );
}

fn ambiguous_endless_operation(
    operation: &Node<'_>,
    definition: &ruby_prism::DefNode<'_>,
) -> Option<(std::ops::Range<usize>, &'static str)> {
    let definition_location = definition.location();
    let direct_definition = |body: Option<Node<'_>>| {
        body.is_some_and(|body| {
            body.as_def_node().is_some_and(|body| {
                body.location().start_offset() == definition_location.start_offset()
                    && body.location().end_offset() == definition_location.end_offset()
            })
        })
    };
    let range = || operation.location().start_offset()..operation.location().end_offset();

    if let Some(conditional) = operation.as_if_node() {
        let keyword = conditional.if_keyword_loc()?;
        if keyword.start_offset() > definition_location.end_offset()
            && conditional.end_keyword_loc().is_none()
            && conditional.subsequent().is_none()
            && direct_definition(only_statement(conditional.statements()))
        {
            return Some((range(), "if"));
        }
    } else if let Some(conditional) = operation.as_unless_node() {
        if conditional.keyword_loc().start_offset() > definition_location.end_offset()
            && conditional.end_keyword_loc().is_none()
            && conditional.else_clause().is_none()
            && direct_definition(only_statement(conditional.statements()))
        {
            return Some((range(), "unless"));
        }
    } else if let Some(logical) = operation.as_and_node() {
        if logical.operator_loc().as_slice() == b"and" && direct_definition(Some(logical.left())) {
            return Some((range(), "and"));
        }
    } else if let Some(logical) = operation.as_or_node() {
        if logical.operator_loc().as_slice() == b"or" && direct_definition(Some(logical.left())) {
            return Some((range(), "or"));
        }
    } else if let Some(loop_node) = operation.as_while_node() {
        if loop_node.keyword_loc().start_offset() > definition_location.end_offset()
            && loop_node.closing_loc().is_none()
            && direct_definition(only_statement(loop_node.statements()))
        {
            return Some((range(), "while"));
        }
    } else if let Some(loop_node) = operation.as_until_node() {
        if loop_node.keyword_loc().start_offset() > definition_location.end_offset()
            && loop_node.closing_loc().is_none()
            && direct_definition(only_statement(loop_node.statements()))
        {
            return Some((range(), "until"));
        }
    }
    None
}

fn multiline_endless_method(
    node: &ruby_prism::DefNode<'_>,
    context: &CopContext<'_, '_>,
) -> Option<String> {
    let file = context.source_file();
    let receiver = node
        .receiver()
        .map(|receiver| format!("{}.", file.node(&receiver)))
        .unwrap_or_default();
    let name = file.at(&node.name_loc());
    let arguments = match (node.lparen_loc(), node.rparen_loc(), node.parameters()) {
        (Some(left), Some(right), _) => file
            .slice(left.start_offset()..right.end_offset())
            .unwrap_or_default()
            .to_string(),
        (_, _, Some(parameters)) => format!(" {}", file.at(&parameters.location())),
        _ => String::new(),
    };
    let body = node.body().and_then(single_expression)?;
    Some(format!(
        "def {receiver}{name}{arguments}\n  {}\nend",
        file.node(&body)
    ))
}

fn nested_method_definition(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_def_node().is_some())
    {
        return;
    }
    if node
        .receiver()
        .is_some_and(|receiver| receiver.as_self_node().is_none())
    {
        return;
    }
    if context.ancestors().iter().any(|ancestor| {
        let source = context.source_file().at(&ancestor.location()).trim_start();
        [
            "Class.new",
            "::Class.new",
            "Module.new",
            "::Module.new",
            "Struct.new",
            "::Struct.new",
            "Data.define",
            "::Data.define",
        ]
        .iter()
        .any(|prefix| source.starts_with(prefix))
            || ancestor.as_singleton_class_node().is_some()
    }) {
        return;
    }
    let allowed = context.config_values("AllowedMethods");
    let patterns = context.config_values("AllowedPatterns");
    if context
        .ancestors()
        .iter()
        .filter_map(Node::as_call_node)
        .any(|call| {
            let name = String::from_utf8_lossy(call.name().as_slice());
            matches!(
                name.as_ref(),
                "instance_eval"
                    | "instance_exec"
                    | "class_eval"
                    | "class_exec"
                    | "module_eval"
                    | "module_exec"
            ) || allowed.iter().any(|allowed| allowed == &name)
                || patterns.iter().any(|pattern| name.contains(pattern))
        })
    {
        return;
    }
    context.report(
        "Method definitions must not be nested. Use `lambda` instead.",
        node.location(),
    );
}

fn useless_constant_scoping(context: &mut CopContext<'_, '_>) {
    let mut private = false;
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim();
        if trimmed == "private" {
            private = true;
            continue;
        }
        if matches!(trimmed, "public" | "protected") {
            private = false;
            continue;
        }
        if !private {
            continue;
        }
        let Some((name, _)) = trimmed.split_once(" = ") else {
            continue;
        };
        if name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            || name.contains(" = ")
        {
            if context.source().lines().any(|candidate| {
                let candidate = candidate.trim();
                candidate.starts_with(&format!("private_constant :{name}"))
                    || candidate.starts_with(&format!("private_constant '{name}'"))
                    || candidate.starts_with(&format!("private_constant \"{name}\""))
            }) {
                continue;
            }
            let start = offset + line.len() - line.trim_start().len();
            context.report(
                "Useless `private` access modifier for constant scope.",
                start..offset + line.len(),
            );
        }
    }
}
