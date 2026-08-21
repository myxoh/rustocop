use super::catalog_cop::custom;
use super::*;
use std::collections::HashMap;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops: Vec<Box<dyn Cop>> = vec![
        Box::new(SafeNavigation),
        Box::new(SelectByKind),
        Box::new(SelectByRange),
        custom("Lint/UselessAccessModifier", useless_access_modifier),
        custom("Style/ArgumentsForwarding", arguments_forwarding),
        custom("Lint/Void", void_expression),
    ];
    cops.extend(registry::cops());
    cops
}

struct SelectByRange;

struct SelectByKind;

struct SafeNavigation;

enum RangeBlockParameter {
    Named(Vec<u8>),
    Numbered,
    It,
}

struct RangeSelection {
    pattern: String,
    negated: bool,
}

struct KindSelection<'pr> {
    class: Node<'pr>,
    negated: bool,
}

impl Cop for SelectByKind {
    fn name(&self) -> &'static str {
        "Style/SelectByKind"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let original = call.name().as_slice();
        if !matches!(original, b"select" | b"filter" | b"find_all" | b"reject") {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        if call.receiver().as_ref().is_some_and(range_hash_receiver) {
            return;
        }
        let Some(parameter) = range_block_parameter(&block) else {
            return;
        };
        let Some(body) = block.body().and_then(single_expression) else {
            return;
        };
        let Some(selection) = kind_selection(body, &parameter) else {
            return;
        };
        let selecting = matches!(original, b"select" | b"filter" | b"find_all");
        let replacement = if selecting == selection.negated {
            "grep_v"
        } else {
            "grep"
        };
        let Some(selector) = call.message_loc() else {
            return;
        };
        let original = String::from_utf8_lossy(original);
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(
            format!("Prefer `{replacement}` to `{original}` with a kind check."),
            call.location(),
            selector.start_offset()..block.location().end_offset(),
            format!(
                "{replacement}({})",
                source_at(source, &selection.class.location())
            ),
        );
    }
}

fn kind_selection<'pr>(
    mut body: Node<'pr>,
    parameter: &RangeBlockParameter,
) -> Option<KindSelection<'pr>> {
    let mut negated = false;
    if let Some(negation) = body.as_call_node() {
        if negation.name().as_slice() == b"!" {
            if negation
                .arguments()
                .is_some_and(|arguments| !arguments.arguments().is_empty())
            {
                return None;
            }
            body = negation.receiver()?;
            negated = true;
        }
    }
    let call = body.as_call_node()?;
    if !matches!(call.name().as_slice(), b"is_a?" | b"kind_of?") {
        return None;
    }
    let receiver = call.receiver()?;
    if !is_range_parameter(&receiver, parameter) {
        return None;
    }
    let arguments = call.arguments()?;
    let mut arguments = arguments.arguments().iter();
    let class = arguments.next()?;
    if arguments.next().is_some() {
        return None;
    }
    Some(KindSelection { class, negated })
}

impl Cop for SelectByRange {
    fn name(&self) -> &'static str {
        "Style/SelectByRange"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let original = call.name().as_slice();
        if !matches!(
            original,
            b"select" | b"filter" | b"find_all" | b"reject" | b"find" | b"detect"
        ) {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        if call.receiver().as_ref().is_some_and(range_hash_receiver) {
            return;
        }
        let Some(parameter) = range_block_parameter(&block) else {
            return;
        };
        let Some(body) = block.body().and_then(single_expression) else {
            return;
        };
        let Some(selection) = range_selection(body, &parameter, source) else {
            return;
        };
        let (grep, suffix, display) = if matches!(original, b"find" | b"detect") {
            if selection.negated {
                ("grep_v", ".first", "grep_v(...).first")
            } else {
                ("grep", ".first", "grep(...).first")
            }
        } else {
            let selecting = matches!(original, b"select" | b"filter" | b"find_all");
            let grep = if selecting == selection.negated {
                "grep_v"
            } else {
                "grep"
            };
            (grep, "", grep)
        };
        let Some(selector) = call.message_loc() else {
            return;
        };
        let original = String::from_utf8_lossy(original);
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(
            format!("Prefer `{display}` to `{original}` with a range check."),
            call.location(),
            selector.start_offset()..block.location().end_offset(),
            format!("{grep}({}){suffix}", selection.pattern),
        );
    }
}

fn range_block_parameter(block: &ruby_prism::BlockNode<'_>) -> Option<RangeBlockParameter> {
    let parameters = block.parameters()?;
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return (numbered.maximum() == 1).then_some(RangeBlockParameter::Numbered);
    }
    if parameters.as_it_parameters_node().is_some() {
        return Some(RangeBlockParameter::It);
    }
    let block_parameters = parameters.as_block_parameters_node()?;
    let parameters = block_parameters.parameters()?;
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    let parameter = parameters
        .requireds()
        .first()?
        .as_required_parameter_node()?;
    Some(RangeBlockParameter::Named(
        parameter.name().as_slice().to_vec(),
    ))
}

fn range_selection(
    body: Node<'_>,
    parameter: &RangeBlockParameter,
    source: &str,
) -> Option<RangeSelection> {
    let (body, negated) = unwrap_range_negation(body)?;
    let call = body.as_call_node()?;
    match call.name().as_slice() {
        b"between?" => {
            let receiver = call.receiver()?;
            if !is_range_parameter(&receiver, parameter) {
                return None;
            }
            let arguments = call.arguments()?;
            let arguments = arguments.arguments().iter().collect::<Vec<_>>();
            if arguments.len() != 2 {
                return None;
            }
            Some(RangeSelection {
                pattern: format!(
                    "{}..{}",
                    source_at(source, &arguments[0].location()),
                    source_at(source, &arguments[1].location())
                ),
                negated,
            })
        }
        b"cover?" | b"include?" => {
            let receiver = unwrap_range_literal(call.receiver()?)?;
            let arguments = call.arguments()?;
            let mut arguments = arguments.arguments().iter();
            let argument = arguments.next()?;
            if arguments.next().is_some() || !is_range_parameter(&argument, parameter) {
                return None;
            }
            Some(RangeSelection {
                pattern: source_at(source, &receiver.location()).to_string(),
                negated,
            })
        }
        _ => None,
    }
}

fn unwrap_range_negation(mut node: Node<'_>) -> Option<(Node<'_>, bool)> {
    let mut negated = false;
    if let Some(call) = node.as_call_node() {
        if call.name().as_slice() == b"!" {
            if call.arguments().is_some_and(|arguments| !arguments.arguments().is_empty()) {
                return None;
            }
            node = call.receiver()?;
            negated = true;
        }
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        node = parentheses.body().and_then(single_expression)?;
    }
    Some((node, negated))
}

fn unwrap_range_literal(mut node: Node<'_>) -> Option<Node<'_>> {
    if let Some(parentheses) = node.as_parentheses_node() {
        node = parentheses.body().and_then(single_expression)?;
    }
    node.as_range_node().map(|range| range.as_node())
}

fn is_range_parameter(node: &Node<'_>, parameter: &RangeBlockParameter) -> bool {
    match parameter {
        RangeBlockParameter::Named(name) => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == name),
        RangeBlockParameter::Numbered => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == b"_1"),
        RangeBlockParameter::It => node.as_it_local_variable_read_node().is_some(),
    }
}

fn range_hash_receiver(node: &Node<'_>) -> bool {
    if node.as_hash_node().is_some() || node_is_root_constant(node, b"ENV") {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        matches!(call.name().as_slice(), b"to_h" | b"to_hash")
            || matches!(call.name().as_slice(), b"new" | b"[]")
                && call
                    .receiver()
                    .as_ref()
                    .is_some_and(|receiver| node_is_root_constant(receiver, b"Hash"))
    })
}

fn useless_access_modifier(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if matches!(window[0].1.trim(), "private" | "protected" | "public")
            && window[0].1.trim() == window[1].1.trim()
        {
            context.remove(
                "Useless access modifier.",
                window[1].0..window[1].0 + window[1].1.len(),
                window[1].0..window[1].0 + window[1].1.len() + 1,
            );
        }
    }
}

struct AccessModifierDeclarations;

impl Cop for AccessModifierDeclarations {
    fn name(&self) -> &'static str {
        "Style/AccessModifierDeclarations"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        if call.receiver().is_some()
            || !matches!(
                call_name(&call),
                b"private" | b"protected" | b"public" | b"module_function"
            )
            || argument_count(&call) == 0
            || ancestors.iter().any(|ancestor| ancestor.as_def_node().is_some())
        {
            return;
        }
        let mut context = context.cop_context(self.name(), source, ancestors);
        if context.policy().enforced_style("group") != "group"
            || allowed_inline_modifier(&call, &context)
            || right_sibling_same_inline_modifier(&call, &context)
        {
            return;
        }
        let Some(selector) = call.message_loc() else {
            return;
        };
        let modifier = context.source_file().at(&selector).to_string();
        let message = format!(
            "`{modifier}` should not be inlined in method definitions."
        );
        let argument = first_argument(&call).expect("modifier has arguments");
        if let Some(definition) = argument.as_def_node() {
            let indentation = context
                .source_file()
                .indentation_text(selector.start_offset());
            context.replace(
                message,
                &selector,
                selector.end_offset()..definition.location().start_offset(),
                format!("\n{indentation}"),
            );
        } else {
            context.replace(message, &selector, &selector, modifier);
        }
    }
}

fn right_sibling_same_inline_modifier(
    node: &CallNode<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let start = node.location().start_offset();
    context.ancestors().iter().rev().any(|ancestor| {
        let statements = if let Some(program) = ancestor.as_program_node() {
            Some(program.statements())
        } else if let Some(class) = ancestor.as_class_node() {
            class.body().and_then(|body| body.as_statements_node())
        } else if let Some(module) = ancestor.as_module_node() {
            module.body().and_then(|body| body.as_statements_node())
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            singleton.body().and_then(|body| body.as_statements_node())
        } else if let Some(begin) = ancestor.as_begin_node() {
            begin.statements()
        } else {
            None
        };
        statements.is_some_and(|statements| {
            let direct_child = statements
                .body()
                .iter()
                .any(|child| child.location().start_offset() == start);
            direct_child
                && statements.body().iter().any(|sibling| {
                    let Some(call) = sibling.as_call_node() else {
                        return false;
                    };
                    call.location().start_offset() > start
                        && call.receiver().is_none()
                        && call_name(&call) == call_name(node)
                        && argument_count(&call) > 0
                        && !allowed_inline_modifier(&call, context)
                })
        })
    })
}

fn allowed_inline_modifier(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    if direct_block_parent(context.ancestors()) {
        return true;
    }
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    if context.config_bool("AllowModifiersOnSymbols", true)
        && arguments.iter().all(symbol_or_allowed_splat)
    {
        return true;
    }
    let Some(call) = arguments
        .first()
        .and_then(|argument| argument.as_call_node())
    else {
        return false;
    };
    call.receiver().is_none()
        && (context.config_bool("AllowModifiersOnAttrs", true)
            && matches!(
                call_name(&call),
                b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor"
            )
            || context.config_bool("AllowModifiersOnAliasMethod", true)
                && call_name(&call) == b"alias_method")
}

fn direct_block_parent(ancestors: &[Node<'_>]) -> bool {
    let Some(parent) = ancestors.last() else {
        return false;
    };
    if parent.as_block_node().is_some() {
        return true;
    }
    let Some(statements) = parent.as_statements_node() else {
        return false;
    };
    if statements.body().len() != 1 {
        return false;
    }
    ancestors[..ancestors.len() - 1]
        .iter()
        .rev()
        .find(|ancestor| ancestor.as_statements_node().is_none())
        .is_some_and(|ancestor| ancestor.as_block_node().is_some())
}

fn symbol_or_allowed_splat(argument: &Node<'_>) -> bool {
    if argument.as_symbol_node().is_some() {
        return true;
    }
    argument
        .as_splat_node()
        .and_then(|splat| splat.expression())
        .is_some_and(|expression| {
            expression.as_array_node().is_some()
                || expression.as_constant_read_node().is_some()
                || expression.as_constant_path_node().is_some()
                || expression.as_call_node().is_some()
        })
}

fn arguments_forwarding(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 7) {
        return;
    }
    let source = context.source().to_string();
    let signature = ["*args, **kwargs, &block", "*args, &block"]
        .into_iter()
        .find(|signature| {
            source
                .lines()
                .any(|line| line.trim_start().starts_with("def ") && line.contains(signature))
        });
    let Some(signature) = signature else { return };
    if ["args =", "kwargs =", "block ="].iter().any(|assignment| {
        source
            .lines()
            .any(|line| line.trim_start().starts_with(assignment))
    }) {
        return;
    }
    let forwarding = if signature.contains("**kwargs") {
        "*args, **kwargs, &block"
    } else {
        "*args, &block"
    };
    if source.match_indices(forwarding).count() < 2 {
        return;
    }
    for start in source
        .match_indices(signature)
        .map(|(start, _)| start)
        .collect::<Vec<_>>()
    {
        context.replace(
            "Use shorthand syntax `...` for arguments forwarding.",
            start..start + signature.len(),
            start..start + signature.len(),
            "...",
        );
    }
}

fn void_expression(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        let (offset, line) = window[0];
        let code = line.trim();
        if matches!(code, "nil" | "true" | "false") && window[1].1.trim() != "end" {
            context.report(
                "Literal expression used in void context.",
                offset..offset + line.len(),
            );
        }
    }
}

struct DuplicateMethods;

#[derive(Default)]
struct DuplicateMethodsState {
    definitions: HashMap<String, SourceDefinition>,
    rescue_scopes: HashMap<&'static str, std::collections::HashSet<String>>,
}

struct SourceDefinition {
    path: String,
    line: usize,
}

impl Cop for DuplicateMethods {
    fn name(&self) -> &'static str {
        "Lint/DuplicateMethods"
    }

    fn investigation_state(&self) -> Box<dyn Any> {
        Box::new(DuplicateMethodsState::default())
    }

    fn on_new_investigation(&self, state: &mut dyn Any) {
        *state
            .downcast_mut::<DuplicateMethodsState>()
            .expect("duplicate methods state") = DuplicateMethodsState::default();
    }

    fn on_node_with_state<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
        state: &mut dyn Any,
    ) {
        if ancestors
            .iter()
            .any(|ancestor| ancestor.as_if_node().is_some() || ancestor.as_unless_node().is_some())
        {
            return;
        }
        let state = state
            .downcast_mut::<DuplicateMethodsState>()
            .expect("duplicate methods state");
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if let Some(definition) = node.as_def_node() {
            let name = String::from_utf8_lossy(definition.name().as_slice()).into_owned();
            let Some(method) = duplicate_method_name(&definition, ancestors, source, &name) else {
                return;
            };
            let key = method_key_with_scope_id(&method, ancestors, source);
            let offense = definition.def_keyword_loc().start_offset()
                ..definition.name_loc().end_offset();
            register_method(state, key, method, offense, &mut cop_context);
        } else if let Some(call) = node.as_call_node() {
            register_attribute_methods(&call, ancestors, state, &mut cop_context);
        }
    }
}

fn duplicate_method_name(
    definition: &ruby_prism::DefNode<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    name: &str,
) -> Option<String> {
    match definition.receiver() {
        None => duplicate_instance_method_name(ancestors, source, name),
        Some(receiver) if receiver.as_self_node().is_some() => {
            let scope = rubocop_parent_module_name(ancestors, source)
                .or_else(|| anonymous_class_scope(ancestors, source).map(|scope| scope.0))?;
            Some(format!("{scope}.{name}"))
        }
        Some(receiver)
            if receiver.as_constant_read_node().is_some()
                || receiver.as_constant_path_node().is_some() =>
        {
            let receiver = node_text(&receiver, source).trim_start_matches("::");
            let scope = rubocop_parent_module_name(ancestors, source)?;
            let qualified = if scope == "Object" || receiver.contains("::") {
                receiver.to_string()
            } else {
                format!("{scope}::{receiver}")
            };
            Some(format!("{qualified}.{name}"))
        }
        Some(_) => None,
    }
}

fn duplicate_instance_method_name(
    ancestors: &[Node<'_>],
    source: &str,
    name: &str,
) -> Option<String> {
    if let Some(scope) = rubocop_parent_module_name(ancestors, source) {
        return Some(format!("{}{name}", humanized_method_scope(&scope)));
    }
    if let Some((scope, _scope_id)) = anonymous_class_scope(ancestors, source) {
        let singleton = ancestors.iter().rev().take_while(|ancestor| {
            ancestor.as_block_node().is_none()
        }).any(|ancestor| ancestor.as_singleton_class_node().is_some());
        let scope = if singleton {
            format!("#<Class:{scope}>")
        } else {
            scope
        };
        return Some(format!("{}{name}", humanized_method_scope(&scope)));
    }
    let singleton = ancestors
        .iter()
        .rev()
        .find_map(Node::as_singleton_class_node)?;
    let receiver = singleton.expression().as_call_node()?;
    Some(format!(
        "{}.{}",
        String::from_utf8_lossy(receiver.name().as_slice()),
        name
    ))
}

/// Mirrors rubocop-ast's `Node#parent_module_name`. In particular, an ordinary
/// block makes the lexical owner unknowable; treating its methods as members of
/// an enclosing class is the source of a large class of false duplicates.
fn rubocop_parent_module_name(ancestors: &[Node<'_>], source: &str) -> Option<String> {
    let mut parts = Vec::new();
    for (index, ancestor) in ancestors.iter().enumerate() {
        if let Some(class) = ancestor.as_class_node() {
            append_scope_part(&mut parts, node_text(&class.constant_path(), source));
        } else if let Some(module) = ancestor.as_module_node() {
            append_scope_part(&mut parts, node_text(&module.constant_path(), source));
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            let expression = singleton.expression();
            let name = if expression.as_self_node().is_some() {
                format!("#<Class:{}>", joined_scope(&parts))
            } else if expression.as_constant_read_node().is_some()
                || expression.as_constant_path_node().is_some()
            {
                format!("#<Class:{}>", node_text(&expression, source).trim_start_matches("::"))
            } else {
                return None;
            };
            parts.push(name);
        } else if let Some(write) = ancestor.as_constant_write_node() {
            if class_or_module_new_call(&write.value()) {
                append_scope_part(&mut parts, location_text(&write.name_loc(), source));
            }
        } else if let Some(write) = ancestor.as_constant_path_write_node() {
            if class_or_module_new_call(&write.value()) {
                append_scope_part(&mut parts, location_text(&write.target().location(), source));
            }
        } else if ancestor.as_block_node().is_some() {
            let Some(call) = index.checked_sub(1).and_then(|parent| ancestors[parent].as_call_node())
            else {
                return None;
            };
            if call_name(&call) == b"class_eval" {
                if let Some(receiver) = call.receiver() {
                    if receiver.as_constant_read_node().is_none()
                        && receiver.as_constant_path_node().is_none()
                    {
                        return None;
                    }
                    append_scope_part(&mut parts, node_text(&receiver, source));
                }
            } else if !class_or_module_new_call(&call.as_node())
                || !ancestors.get(index.wrapping_sub(2)).is_some_and(|parent| {
                    parent.as_constant_write_node().is_some()
                        || parent.as_constant_path_write_node().is_some()
                })
            {
                return None;
            }
        }
    }
    Some(if parts.is_empty() {
        "Object".to_string()
    } else {
        joined_scope(&parts)
    })
}

fn class_or_module_new_call(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"new"
            && (root_constant(call.receiver(), b"Class")
                || root_constant(call.receiver(), b"Module"))
    })
}

fn append_scope_part(parts: &mut Vec<String>, raw: &str) {
    let name = raw.trim_start_matches("::");
    if name.contains("::") {
        parts.clear();
    }
    parts.push(name.to_string());
}

fn joined_scope(parts: &[String]) -> String {
    parts.join("::")
}

fn humanized_method_scope(scope: &str) -> String {
    if let Some(start) = scope.find("#<Class:") {
        if let Some(name) = scope[start + 8..].strip_suffix('>') {
            return format!("{name}.");
        }
    }
    format!("{scope}#")
}

fn anonymous_class_scope(ancestors: &[Node<'_>], source: &str) -> Option<(String, Option<String>)> {
    let block_index = ancestors
        .iter()
        .rposition(|ancestor| ancestor.as_block_node().is_some())?;
    let call_index = block_index.checked_sub(1)?;
    let call = ancestors[call_index].as_call_node()?;
    if !class_or_module_new_call(&call.as_node())
        || ancestors.get(call_index.wrapping_sub(1)).is_some_and(|parent| {
            parent.as_local_variable_write_node().is_some()
        })
    {
        return None;
    }
    if ancestors[block_index + 1..].iter().any(|ancestor| {
        ancestor.as_singleton_class_node().is_some_and(|singleton| {
            singleton.expression().as_self_node().is_none()
        })
    }) {
        return None;
    }
    let enclosing = rubocop_parent_module_name(&ancestors[..call_index], source);
    let base = match enclosing.as_deref() {
        Some("Object") => "Object".to_string(),
        Some(enclosing) => format!("{enclosing}::Object"),
        None => "::Object".to_string(),
    };
    let named_scope_id = ancestors[..call_index]
        .iter()
        .rev()
        .find_map(Node::as_call_node)
        .and_then(|parent| {
            parent.receiver().and_then(|receiver| {
                if class_or_module_new_call(&receiver) {
                    return None;
                }
                format!(
                    "{}.{}",
                    node_text(&receiver, source),
                    String::from_utf8_lossy(parent.name().as_slice())
                )
                .into()
            })
        });
    let scope_id = named_scope_id.or_else(|| {
        (duplicate_rescue_scope(&ancestors[..call_index]) != Some("ensure"))
        .then(|| format!("anonymous: {}", call.location().start_offset()))
    });
    Some((base, scope_id))
}

fn node_text<'a>(node: &Node<'_>, source: &'a str) -> &'a str {
    let location = node.location();
    &source[location.start_offset()..location.end_offset()]
}

fn location_text<'a>(location: &ruby_prism::Location<'_>, source: &'a str) -> &'a str {
    &source[location.start_offset()..location.end_offset()]
}

fn method_key_with_scope_id(method: &str, ancestors: &[Node<'_>], source: &str) -> String {
    let mut key = nested_method_key(method, ancestors);
    if rubocop_parent_module_name(ancestors, source).is_none() {
        if let Some(scope_id) = anonymous_class_scope(ancestors, source).and_then(|scope| scope.1) {
            key.push('@');
            key.push_str(&scope_id);
        }
    }
    key
}

fn nested_method_key(method: &str, ancestors: &[Node<'_>]) -> String {
    ancestors
        .iter()
        .rev()
        .find_map(Node::as_def_node)
        .map_or_else(|| method.to_string(), |definition| {
            format!(
                "{}:{method}",
                String::from_utf8_lossy(definition.name().as_slice())
            )
        })
}

fn register_attribute_methods(
    call: &CallNode<'_>,
    ancestors: &[Node<'_>],
    state: &mut DuplicateMethodsState,
    context: &mut CopContext<'_, '_>,
) {
    if call.receiver().is_some() {
        return;
    }
    let call_method = call_name(call);
    let arguments = call
        .arguments()
        .into_iter()
        .flat_map(|arguments| arguments.arguments().iter())
        .collect::<Vec<_>>();
    let mut names = Vec::new();
    if matches!(call_method, b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor") {
        let readable = matches!(call_method, b"attr" | b"attr_reader" | b"attr_accessor");
        let writable = matches!(call_method, b"attr_writer" | b"attr_accessor");
        for argument in &arguments {
            let Some(name) = literal_method_name(argument) else { continue };
            if readable {
                names.push(name.clone());
            }
            if writable {
                names.push(format!("{name}="));
            }
        }
    } else if matches!(call_method, b"def_delegator" | b"def_instance_delegator") {
        if let Some(name) = arguments.get(if arguments.len() >= 3 { 2 } else { 1 })
            .and_then(|argument| literal_method_name(argument))
        {
            names.push(name);
        }
    } else if matches!(call_method, b"def_delegators" | b"def_instance_delegators") {
        names.extend(arguments.iter().skip(1).filter_map(|argument| literal_method_name(argument)));
    } else {
        return;
    }
    for name in names {
        let Some(method) = duplicate_instance_method_name(ancestors, context.source(), &name) else {
            continue;
        };
        let key = method_key_with_scope_id(&method, ancestors, context.source());
        let location = call.location();
        register_method(
            state,
            key,
            method,
            location.start_offset()..location.end_offset(),
            context,
        );
    }
}

fn literal_method_name(node: &Node<'_>) -> Option<String> {
    if let Some(symbol) = node.as_symbol_node() {
        Some(String::from_utf8_lossy(symbol.unescaped()).into_owned())
    } else {
        node.as_string_node()
            .map(|string| String::from_utf8_lossy(string.unescaped()).into_owned())
    }
}

fn register_method(
    state: &mut DuplicateMethodsState,
    key: String,
    method: String,
    offense: std::ops::Range<usize>,
    context: &mut CopContext<'_, '_>,
) {
    let line = context.source()[..offense.start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let path = smart_source_path(context.path());
    if let Some(previous) = state.definitions.get(&key) {
        let rescue_scope = duplicate_rescue_scope(context.ancestors());
        if let Some(rescue_scope) = rescue_scope {
            if state
                .rescue_scopes
                .entry(rescue_scope)
                .or_default()
                .insert(key.clone())
            {
                state.definitions.insert(key, SourceDefinition { path, line });
                return;
            }
        }
        let message = format!(
            "Method `{method}` is defined at both {}:{} and {path}:{line}.",
            previous.path, previous.line
        );
        context.report(message, offense);
    } else {
        state.definitions.insert(key, SourceDefinition { path, line });
    }
}

fn duplicate_rescue_scope(ancestors: &[Node<'_>]) -> Option<&'static str> {
    ancestors.iter().rev().find_map(|ancestor| {
        if ancestor.as_rescue_node().is_some() {
            Some("rescue")
        } else if ancestor.as_begin_node().is_some_and(|begin| begin.ensure_clause().is_some()) {
            // Prism exposes `ensure` through its containing BeginNode rather
            // than retaining EnsureNode in the investigation ancestor stack.
            Some("ensure")
        } else {
            None
        }
    })
}

fn smart_source_path(path: &str) -> String {
    let path = std::path::Path::new(path);
    std::env::current_dir()
        .ok()
        .and_then(|current| path.strip_prefix(current).ok().map(|path| path.to_path_buf()))
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

const SAFE_NAVIGATION_MESSAGE: &str =
    "Use safe navigation (`&.`) instead of checking if an object exists before calling the method.";

impl Cop for SafeNavigation {
    fn name(&self) -> &'static str {
        "Style/SafeNavigation"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if !cop_context.target_ruby_version().at_least(2, 3) {
            return;
        }
        if let Some(conditional) = node.as_if_node() {
            safe_navigation_if(&conditional, &mut cop_context);
        } else if let Some(conditional) = node.as_unless_node() {
            safe_navigation_unless(&conditional, &mut cop_context);
        } else if let Some(and_node) = node.as_and_node() {
            if !ancestors.iter().any(|parent| parent.as_and_node().is_some()) {
                safe_navigation_and(&and_node, &mut cop_context);
            }
        }
    }
}

fn safe_navigation_if(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .if_keyword_loc()
        .as_ref()
        .is_some_and(|keyword| keyword.as_slice() == b"elsif")
    {
        return;
    }
    let ternary = node.if_keyword_loc().is_none()
        && node.then_keyword_loc().is_some()
        && node.end_keyword_loc().is_none();
    let then_branch = node.statements().and_then(|body| body.body().first());
    let else_branch = node
        .subsequent()
        .and_then(|subsequent| subsequent.as_else_node())
        .and_then(|else_node| else_node.statements())
        .and_then(|body| body.body().first());
    let (checked, body) = if ternary {
        let Some(then_branch) = then_branch else { return };
        let Some(else_branch) = else_branch else { return };
        if else_branch.as_nil_node().is_some() {
            if let Some(checked) = non_nil_checked_receiver(&node.predicate()) {
                (checked, then_branch)
            } else if simple_truthy_check(&node.predicate()) {
                (node.predicate(), then_branch)
            } else {
                return;
            }
        } else if then_branch.as_nil_node().is_some() {
            if let Some(checked) = nil_checked_receiver(&node.predicate()) {
                (checked, else_branch)
            } else if let Some(checked) = negated_receiver(&node.predicate()) {
                (checked, else_branch)
            } else {
                return;
            }
        } else {
            return;
        }
    } else {
        if node.subsequent().is_some() {
            return;
        }
        let Some(body) = then_branch else { return };
        if let Some(checked) = non_nil_checked_receiver(&node.predicate()) {
            (checked, body)
        } else if simple_truthy_check(&node.predicate()) {
            (node.predicate(), body)
        } else {
            return;
        }
    };
    safe_navigation_conditional(node.location(), &checked, &body, ternary, context);
}

fn safe_navigation_unless(
    node: &ruby_prism::UnlessNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if node.else_clause().is_some() {
        return;
    }
    let Some(body) = node.statements().and_then(|body| body.body().first()) else {
        return;
    };
    let checked = if let Some(checked) = nil_checked_receiver(&node.predicate()) {
        checked
    } else if let Some(checked) = negated_receiver(&node.predicate()) {
        checked
    } else {
        return;
    };
    // `obj.do_something unless obj` uses the variable only as a negative
    // condition, rather than as the positive existence guard this cop targets.
    safe_navigation_conditional(node.location(), &checked, &body, false, context);
}

fn safe_navigation_conditional(
    offense: ruby_prism::Location<'_>,
    checked: &Node<'_>,
    body: &Node<'_>,
    ternary: bool,
    context: &mut CopContext<'_, '_>,
) {
    let checked_source = context.source_file().node(checked).to_string();
    let Some(chain) = safe_navigation_chain(body, &checked_source, ternary, context) else {
        return;
    };
    let mut replacement = corrected_safe_navigation_chain(body, &checked_source, &chain, context);
    let before = &context.source()[offense.start_offset()..body.location().start_offset()];
    let after = &context.source()[body.location().end_offset()..offense.end_offset()];
    let comments = before
        .lines()
        .chain(after.lines())
        .filter_map(|line| line.find('#').map(|comment| line[comment..].trim()))
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    if !comments.is_empty() {
        replacement = format!("{comments}{replacement}");
    }
    let offense = offense.start_offset()..offense.end_offset();
    context.replace(SAFE_NAVIGATION_MESSAGE, offense.clone(), offense, replacement);
}

fn safe_navigation_and(node: &ruby_prism::AndNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut clauses = Vec::new();
    flatten_safe_navigation_and(node.as_node(), &mut clauses);
    struct Candidate {
        index: usize,
        offense: std::ops::Range<usize>,
        checked_source: String,
    }
    let mut candidates = Vec::new();
    for (index, pair) in clauses.windows(2).enumerate() {
        let lhs = &pair[0];
        let rhs = &pair[1];
        let (checked_source, non_nil) = if let Some(checked) = non_nil_checked_receiver(lhs) {
            (context.source_file().node(&checked).to_string(), true)
        } else if simple_truthy_check(lhs) {
            (context.source_file().node(lhs).to_string(), false)
        } else {
            continue;
        };
        if non_nil && !context.config_bool("ConvertCodeThatCanStartToReturnNil", false) {
            continue;
        }
        let Some(chain) = safe_navigation_chain(rhs, &checked_source, false, context) else {
            continue;
        };
        let _ = chain;
        let mut end = rhs.location().end_offset();
        let between = &context.source()[lhs.location().end_offset()..rhs.location().start_offset()];
        let opening_parentheses = between.bytes().filter(|byte| *byte == b'(').count();
        for _ in 0..opening_parentheses {
            if context.source().as_bytes().get(end) == Some(&b')') {
                end += 1;
            } else {
                break;
            }
        }
        candidates.push(Candidate {
            index,
            offense: lhs.location().start_offset()..end,
            checked_source,
        });
    }
    if candidates.is_empty() {
        safe_navigation_and_with_or(node, context);
        return;
    }

    let mut groups = Vec::<(usize, usize)>::new();
    let mut group_start = 0;
    for index in 1..candidates.len() {
        if candidates[index].index != candidates[index - 1].index + 1 {
            groups.push((group_start, index - 1));
            group_start = index;
        }
    }
    groups.push((group_start, candidates.len() - 1));

    let node_start = node.location().start_offset();
    let node_end = node.location().end_offset();
    let mut edits = Vec::new();
    for (first, last) in groups {
        let candidate = &candidates[first];
        let lhs = &clauses[candidate.index];
        let final_rhs = &clauses[candidates[last].index + 1];
        let Some(chain) = safe_navigation_chain(
            final_rhs,
            &candidate.checked_source,
            false,
            context,
        ) else {
            continue;
        };
        let corrected = corrected_safe_navigation_chain(
            final_rhs,
            &candidate.checked_source,
            &chain,
            context,
        );
        let between =
            &context.source()[lhs.location().end_offset()..clauses[candidate.index + 1].location().start_offset()];
        let preserved = between
            .chars()
            .filter(|character| *character == '(')
            .collect::<String>();
        edits.push((
            lhs.location().start_offset()..final_rhs.location().end_offset(),
            format!("{preserved}{corrected}"),
        ));
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut correction = String::new();
    let mut cursor = node_start;
    for (range, replacement) in edits {
        correction.push_str(&context.source()[cursor..range.start]);
        correction.push_str(&replacement);
        cursor = range.end;
    }
    correction.push_str(&context.source()[cursor..node_end]);

    context.replace(
        SAFE_NAVIGATION_MESSAGE,
        candidates[0].offense.clone(),
        node.location(),
        correction,
    );
    if !context.autocorrect_enabled() {
        for candidate in candidates.iter().skip(1) {
            context.report(SAFE_NAVIGATION_MESSAGE, candidate.offense.clone());
        }
    }
}

fn safe_navigation_and_with_or(
    node: &ruby_prism::AndNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let lhs = node.left();
    if !simple_truthy_check(&lhs) {
        return;
    }
    let checked_source = context.source_file().node(&lhs).to_string();
    let right = unwrap_safe_navigation_parentheses(node.right());
    let Some(or_node) = right.as_or_node() else {
        return;
    };
    let Some(candidate) = first_safe_navigation_and_left(or_node.right()) else {
        return;
    };
    if safe_navigation_chain(&candidate, &checked_source, false, context).is_none() {
        return;
    }
    context.report(
        SAFE_NAVIGATION_MESSAGE,
        lhs.location().start_offset()..candidate.location().end_offset(),
    );
}

fn unwrap_safe_navigation_parentheses(mut node: Node<'_>) -> Node<'_> {
    loop {
        let Some(parentheses) = node.as_parentheses_node() else {
            return node;
        };
        let Some(inner) = parentheses.body().and_then(single_expression) else {
            return node;
        };
        node = inner;
    }
}

fn first_safe_navigation_and_left(node: Node<'_>) -> Option<Node<'_>> {
    let node = unwrap_safe_navigation_parentheses(node);
    if let Some(and_node) = node.as_and_node() {
        return Some(and_node.left());
    }
    let or_node = node.as_or_node()?;
    first_safe_navigation_and_left(or_node.left())
        .or_else(|| first_safe_navigation_and_left(or_node.right()))
}

fn flatten_safe_navigation_and<'pr>(node: Node<'pr>, clauses: &mut Vec<Node<'pr>>) {
    if let Some(parentheses) = node.as_parentheses_node() {
        if let Some(inner) = parentheses.body().and_then(single_expression) {
            flatten_safe_navigation_and(inner, clauses);
            return;
        }
    }
    if let Some(and_node) = node.as_and_node() {
        flatten_safe_navigation_and(and_node.left(), clauses);
        flatten_safe_navigation_and(and_node.right(), clauses);
    } else {
        clauses.push(node);
    }
}

fn safe_navigation_chain<'pr>(
    body: &Node<'pr>,
    checked_source: &str,
    ternary: bool,
    context: &CopContext<'_, '_>,
) -> Option<Vec<CallNode<'pr>>> {
    let mut calls = Vec::new();
    let mut call = body.as_call_node()?;
    loop {
        if call_name(&call) == b"!" {
            return None;
        }
        let receiver = call.receiver()?;
        calls.push(call);
        if safe_navigation_source_matches(
            source_at(context.source(), &receiver.location()),
            checked_source,
        ) {
            break;
        }
        call = receiver.as_call_node()?;
    }
    calls.reverse();
    if calls.len() > context.config_usize("MaxChainLength", 2) {
        return None;
    }
    if calls.len() > 1
        && context.related_config_value("Lint/SafeNavigationChain", "Enabled") == Some("false")
    {
        return None;
    }
    let first = calls.first()?;
    let first_operator = first.call_operator_loc()?;
    if first_operator.as_slice() == b"::"
        || (!ternary && unsafe_safe_navigation_call(first))
    {
        return None;
    }
    if body
        .as_call_node()
        .is_some_and(|call| call_name(&call) == b"empty?")
    {
        return None;
    }
    for call in calls.iter().skip(1) {
        if unsafe_safe_navigation_call(call)
            || safe_navigation_nil_method(call_name(call))
            || safe_navigation_allowed_method(call_name(call), context)
        {
            return None;
        }
    }
    Some(calls)
}

fn safe_navigation_allowed_method(name: &[u8], context: &CopContext<'_, '_>) -> bool {
    matches!(name, b"present?" | b"blank?" | b"presence" | b"try" | b"try!")
        || context.policy().allows_method(name)
}

fn corrected_safe_navigation_chain(
    body: &Node<'_>,
    checked_source: &str,
    calls: &[CallNode<'_>],
    context: &CopContext<'_, '_>,
) -> String {
    let body_start = body.location().start_offset();
    let body_end = body.location().end_offset();
    let mut edits = Vec::new();
    let matched = calls.first().and_then(CallNode::receiver);
    if let Some(matched) = matched {
        let matched_source = context.source_file().node(&matched);
        if checked_source != matched_source {
            edits.push((
                matched.location().start_offset()..matched.location().end_offset(),
                checked_source.to_string(),
            ));
        }
    }
    for call in calls {
        if let Some(operator) = call.call_operator_loc() {
            if operator.as_slice() == b"." {
                edits.push((operator.start_offset()..operator.start_offset(), "&".to_string()));
            }
        }
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut rendered = String::new();
    let mut cursor = body_start;
    for (range, replacement) in edits {
        if range.start < cursor || range.end > body_end {
            continue;
        }
        rendered.push_str(&context.source()[cursor..range.start]);
        rendered.push_str(&replacement);
        cursor = range.end;
    }
    rendered.push_str(&context.source()[cursor..body_end]);
    rendered
}

fn simple_truthy_check(node: &Node<'_>) -> bool {
    node.as_call_node()
        .is_none_or(|call| !matches!(call_name(&call), b"!" | b"nil?" | b"respond_to?"))
        && node.as_and_node().is_none()
        && node.as_or_node().is_none()
}

fn nil_checked_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let call = node.as_call_node()?;
    if call_name(&call) == b"nil?" && argument_count(&call) == 0 {
        call.receiver()
    } else {
        None
    }
}

fn non_nil_checked_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let receiver = negated_receiver(node)?;
    nil_checked_receiver(&receiver)
}

fn negated_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let call = node.as_call_node()?;
    if call_name(&call) == b"!" && argument_count(&call) == 0 {
        call.receiver()
    } else {
        None
    }
}

fn safe_navigation_source_matches(left: &str, right: &str) -> bool {
    normalize_safe_navigation_source(left) == normalize_safe_navigation_source(right)
}

fn normalize_safe_navigation_source(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum Literal {
        Quote(char),
        Percent { open: char, close: char, depth: usize },
    }
    let characters = source.chars().collect::<Vec<_>>();
    let mut normalized = String::new();
    let mut literal = None;
    let mut escaped = false;
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if let Some(state) = literal {
            normalized.push(character);
            if escaped {
                escaped = false;
                index += 1;
                continue;
            }
            if character == '\\' {
                escaped = true;
                index += 1;
                continue;
            }
            match state {
                Literal::Quote(close) if character == close => literal = None,
                Literal::Percent {
                    open,
                    close,
                    mut depth,
                } => {
                    if character == open && open != close {
                        depth += 1;
                        literal = Some(Literal::Percent { open, close, depth });
                    } else if character == close {
                        if depth == 0 {
                            literal = None;
                        } else {
                            literal = Some(Literal::Percent {
                                open,
                                close,
                                depth: depth - 1,
                            });
                        }
                    }
                }
                _ => {}
            }
            index += 1;
            continue;
        }
        if character.is_whitespace() {
            index += 1;
            continue;
        }
        if character == '&' && characters.get(index + 1) == Some(&'.') {
            normalized.push('.');
            index += 2;
            continue;
        }
        if matches!(character, '\'' | '"' | '`' | '/') {
            normalized.push(character);
            literal = Some(Literal::Quote(character));
            index += 1;
            continue;
        }
        if character == '%' {
            let delimiter_index = if characters
                .get(index + 1)
                .is_some_and(|kind| matches!(kind, 'q' | 'Q' | 'r' | 'w' | 'W' | 'i' | 'I' | 'x' | 's'))
            {
                index + 2
            } else {
                index + 1
            };
            if let Some(&open) = characters.get(delimiter_index) {
                if !open.is_alphanumeric() && !open.is_whitespace() {
                    for value in &characters[index..=delimiter_index] {
                        normalized.push(*value);
                    }
                    let close = match open {
                        '(' => ')',
                        '[' => ']',
                        '{' => '}',
                        '<' => '>',
                        other => other,
                    };
                    literal = Some(Literal::Percent {
                        open,
                        close,
                        depth: 0,
                    });
                    index = delimiter_index + 1;
                    continue;
                }
            }
        }
        normalized.push(character);
        index += 1;
    }
    normalized
}

fn unsafe_safe_navigation_call(call: &CallNode<'_>) -> bool {
    let name = call_name(call);
    let assignment = name.ends_with(b"=")
        && !matches!(name, b"==" | b"!=" | b"<=" | b">=" | b"===" | b"=~" | b"!~" | b"<=>");
    assignment
        || call.call_operator_loc().is_none()
        || call.call_operator_loc().is_some_and(|operator| operator.as_slice() == b"::")
}

fn safe_navigation_nil_method(name: &[u8]) -> bool {
    matches!(
        name,
        b"nil?"
            | b"to_s"
            | b"to_i"
            | b"to_f"
            | b"to_a"
            | b"to_h"
            | b"to_c"
            | b"to_r"
            | b"inspect"
            | b"hash"
            | b"object_id"
            | b"class"
            | b"itself"
            | b"freeze"
            | b"frozen?"
    )
}
