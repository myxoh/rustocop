use super::*;
use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;
use crate::rubocop::ast::prism::convert as convert_rubocop_ast;
use std::collections::HashSet;

define_cops! {
    UnusedMethodArgument => "Lint/UnusedMethodArgument" => node(as_def_node, unused_method_argument),
    UselessMethodDefinition => "Lint/UselessMethodDefinition" => node(as_def_node, useless_method_definition),
    ConstantOverwrittenInRescue => "Lint/ConstantOverwrittenInRescue" => node(as_rescue_node, constant_overwritten_in_rescue),
    RedundantAssignment => "Style/RedundantAssignment" => source(redundant_assignment),
    ConstantResolution => "Lint/ConstantResolution" => any_node(constant_resolution),
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
        !(argument.name.starts_with('_')
            || usage.reads.contains(&argument.name)
            || argument.keyword && allow_keywords
            || argument.block && usage.yield_seen)
    }).collect::<Vec<_>>();
    let relevant = arguments
        .iter()
        .filter(|argument| {
            !(argument.name.starts_with('_') || argument.keyword && allow_keywords)
        })
        .count();
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

fn constant_overwritten_in_rescue(
    node: &ruby_prism::RescueNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    // RuboCop's resbody matcher accepts only an implicit exception list whose
    // assignment target is a constant. Prism exposes that target as reference.
    if !node.exceptions().is_empty() || node.statements().is_some() {
        return;
    }
    let Some(reference) = node.reference() else {
        return;
    };
    if reference.as_constant_target_node().is_none()
        && reference.as_constant_path_target_node().is_none()
    {
        return;
    }

    let keyword = node.keyword_loc();
    let between = keyword.end_offset()..reference.location().start_offset();
    let Some(relative_arrow) = context
        .source_file()
        .slice(between.clone())
        .and_then(|source| source.find("=>"))
    else {
        return;
    };
    let arrow_start = between.start + relative_arrow;
    let constant = context.source_file().node(&reference);
    context.replace(
        format!("`{constant}` is overwritten by `rescue =>`."),
        arrow_start..arrow_start + 2,
        keyword.end_offset()..arrow_start + 2,
        "",
    );
}

fn redundant_assignment(context: &mut CopContext<'_, '_>) {
    let parsed = ruby_prism::parse(context.source().as_bytes());
    let (ast, root) = convert_rubocop_ast(context.source(), &parsed.node());
    let Some(root) = root.map(|root| ast.node(root)) else { return };
    for definition in root.each_node(&["def", "defs"]) {
        check_redundant_assignment_branch(definition.body(), context);
    }
}

fn check_redundant_assignment_branch(
    node: Option<RubocopNodeRef<'_>>,
    context: &mut CopContext<'_, '_>,
) {
    let Some(node) = node else { return };
    match node.kind() {
        "case" | "case_match" => {
            for branch in node.branches() {
                check_redundant_assignment_branch(branch, context);
            }
        }
        "if" if !node.modifier_form() && !node.ternary() => {
            check_redundant_assignment_branch(node.if_branch(), context);
            check_redundant_assignment_branch(node.else_branch(), context);
        }
        "rescue" | "resbody" => {
            for child in node.child_nodes() {
                check_redundant_assignment_branch(Some(child), context);
            }
        }
        "ensure" => check_redundant_assignment_branch(node.ensure_branch(), context),
        "begin" | "kwbegin" => check_redundant_assignment_begin(node, context),
        _ => {}
    }
}

fn check_redundant_assignment_begin(
    node: RubocopNodeRef<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let children = node.child_nodes();
    if let [.., assignment, returned] = children.as_slice() {
        let same_name = assignment.kind() == "lvasgn"
            && returned.kind() == "lvar"
            && assignment.symbol_child(0) == returned.symbol_child(0);
        if same_name {
            let Some(expression) = assignment.expression() else { return };
            let Some(assignment_chars) = assignment.source_range() else { return };
            let Some(returned_chars) = returned.source_range() else { return };
            let Some(expression_source) = expression.source() else { return };
            let assignment_range = semantic_character_range_to_byte(context.source(), assignment_chars);
            let returned_range = semantic_character_range_to_byte(context.source(), returned_chars);
            context.replace_many(
                "Redundant assignment before returning detected.",
                assignment_range.clone(),
                vec![
                    (assignment_range, expression_source.to_string()),
                    (returned_range, String::new()),
                ],
            );
            return;
        }
    }
    check_redundant_assignment_branch(children.last().copied(), context);
}

fn semantic_character_range_to_byte(
    source: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let byte = |offset| {
        source
            .char_indices()
            .nth(offset)
            .map_or(source.len(), |(byte, _)| byte)
    };
    byte(range.start)..byte(range.end)
}
fn constant_resolution(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(constant) = node.as_constant_read_node() else {
        return;
    };
    let location = constant.location();
    let direct_defined_module = context
        .parent()
        .is_some_and(|parent| {
            parent.as_class_node().is_some()
                || parent.as_module_node().is_some()
                || parent
                    .as_constant_path_write_node()
                    .is_some_and(|write| module_constructor(&write.value()))
        });
    let ancestors = context.ancestors();
    let prism_single_body_defined_module = ancestors.last().is_some_and(|parent| {
        parent
            .as_statements_node()
            .is_some_and(|statements| statements.body().len() == 1)
    }) && ancestors
        .get(ancestors.len().saturating_sub(2))
        .is_some_and(|grandparent| {
            grandparent.as_class_node().is_some() || grandparent.as_module_node().is_some()
        });
    if direct_defined_module || prism_single_body_defined_module {
        return;
    }
    let name = String::from_utf8_lossy(constant.name().as_slice());
    let only = context.config_values("Only");
    let ignore = context.config_values("Ignore");
    if !only.is_empty() && !only.iter().any(|allowed| allowed == name.as_ref())
        || ignore.iter().any(|ignored| ignored == name.as_ref())
    {
        return;
    }
    context.report(
        "Fully qualify this constant to avoid possibly ambiguous resolution.",
        location,
    );
}

fn module_constructor(node: &Node<'_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return false;
    };
    call.name().as_slice() == b"new"
        && call.receiver().is_some_and(|receiver| {
            receiver.as_constant_read_node().is_some_and(|constant| {
                matches!(constant.name().as_slice(), b"Class" | b"Module")
            })
        })
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
