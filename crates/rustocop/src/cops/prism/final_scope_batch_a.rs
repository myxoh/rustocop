use super::catalog_cop::{custom, report};
use super::*;
use std::collections::HashSet;

mod naming;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        Box::new(ShadowedException) as Box<dyn Cop>,
        Box::new(ConstantDefinitionInBlock),
        Box::new(ShadowingOuterLocalVariable),
        report(
            "Lint/LiteralAssignmentInCondition",
            "if value = 1",
            "Do not use a literal assignment in a condition.",
        ),
        Box::new(HeredocDelimiterCase) as Box<dyn Cop>,
        Box::new(BlockForwarding) as Box<dyn Cop>,
        custom("Lint/AmbiguousAssignment", ambiguous_assignment),
        Box::new(RescuedExceptionsVariableName) as Box<dyn Cop>,
        custom("Lint/ConstantReassignment", constant_reassignment),
    ];
    cops.extend(naming::cops());
    cops
}

define_any_node_cop!(HeredocDelimiterCase => "Naming/HeredocDelimiterCase" => heredoc_case);
define_any_node_cop!(ShadowingOuterLocalVariable => "Lint/ShadowingOuterLocalVariable" => shadowing_outer_local);
define_node_cop!(BlockForwarding => "Naming/BlockForwarding" => as_def_node => block_forwarding);
define_node_cop!(RescuedExceptionsVariableName => "Naming/RescuedExceptionsVariableName" => as_rescue_node => rescued_exception_name);
define_node_cop!(ShadowedException => "Lint/ShadowedException" => as_rescue_node => shadowed_exception);
define_any_node_cop!(ConstantDefinitionInBlock => "Lint/ConstantDefinitionInBlock" => constant_in_block);

fn ambiguous_assignment(context: &mut CopContext<'_, '_>) {
    for (needle, operator) in [("=-", "-"), ("=+", "+"), ("=*", "*"), ("=!", "!")] {
        for start in context.source_file().code_offsets(needle) {
            if !context.source()[..start]
                .as_bytes()
                .last()
                .is_some_and(|byte| byte.is_ascii_whitespace())
            {
                continue;
            }
            if needle == "=!" && context.source().as_bytes().get(start + 2) == Some(&b'!') {
                continue;
            }
            context.report(
                format!("Suspicious assignment detected. Did you mean `{operator}=`?"),
                start..start + needle.len(),
            );
        }
    }
}

fn shadowed_exception(node: &ruby_prism::RescueNode<'_>, context: &mut CopContext<'_, '_>) {
    let current = rescued_exception_names(node, context.source_file());
    let shadows_within_group = current.iter().enumerate().any(|(index, exception)| {
        current[index + 1..]
            .iter()
            .any(|other| exception_shadows(exception, other) || exception_shadows(other, exception))
    });
    let shadows_later_group = node.subsequent().is_some_and(|later| {
        rescue_groups_unsorted(
            &current,
            &rescued_exception_names(&later, context.source_file()),
        )
    });
    if !shadows_within_group && !shadows_later_group {
        return;
    }

    let end = node
        .statements()
        .map(|statements| statements.location().end_offset())
        .or_else(|| {
            node.exceptions()
                .iter()
                .last()
                .map(|exception| exception.location().end_offset())
        })
        .unwrap_or_else(|| node.keyword_loc().end_offset());
    context.report(
        "Do not shadow rescued Exceptions.",
        node.keyword_loc().start_offset()..end,
    );
}

fn rescued_exception_names(node: &ruby_prism::RescueNode<'_>, file: SourceFile<'_>) -> Vec<String> {
    let names = node
        .exceptions()
        .iter()
        .filter_map(|exception| {
            if exception.as_splat_node().is_some() {
                return Some("*".to_string());
            }
            if exception.as_constant_read_node().is_none()
                && exception.as_constant_path_node().is_none()
            {
                return None;
            }
            Some(file.node(&exception).trim_start_matches("::").to_string())
        })
        .collect::<Vec<_>>();
    if names.is_empty() {
        vec!["StandardError".to_string()]
    } else {
        names
    }
}

fn exception_shadows(ancestor: &str, descendant: &str) -> bool {
    if ancestor == descendant && ancestor != "*" {
        return true;
    }
    let descendant_base = descendant.rsplit("::").next().unwrap_or(descendant);
    match ancestor {
        "Exception" => true,
        "StandardError" => {
            descendant.starts_with("Errno::")
                || matches!(descendant, "Psych::Exception" | "Timeout::Error")
                || matches!(
                    descendant_base,
                    "StandardError"
                | "ArgumentError"
                | "EncodingError"
                | "FiberError"
                | "IOError"
                | "EOFError"
                | "IndexError"
                | "KeyError"
                | "StopIteration"
                | "LocalJumpError"
                | "NameError"
                | "NoMethodError"
                | "RangeError"
                | "FloatDomainError"
                | "RegexpError"
                | "RuntimeError"
                | "SystemCallError"
                | "ThreadError"
                | "TypeError"
                        | "ZeroDivisionError"
                )
        }
        "NameError" => descendant_base == "NoMethodError",
        "IndexError" => matches!(descendant_base, "KeyError" | "StopIteration"),
        "RangeError" => descendant_base == "FloatDomainError",
        "ScriptError" => matches!(
            descendant_base,
            "LoadError" | "NotImplementedError" | "SyntaxError"
        ),
        "SignalException" => descendant_base == "Interrupt",
        "EncodingError" => matches!(
            descendant,
            "Encoding::CompatibilityError"
                | "Encoding::ConverterNotFoundError"
                | "Encoding::InvalidByteSequenceError"
                | "Encoding::UndefinedConversionError"
        ),
        "ArgumentError" => descendant == "IPAddr::InvalidAddressError",
        "RuntimeError" => descendant == "Timeout::Error",
        "OpenSSL::PKey::PKeyError" => descendant == "OpenSSL::PKey::RSAError",
        "Psych::Exception" => descendant == "Psych::SyntaxError",
        _ => false,
    }
}

fn rescue_groups_unsorted(current: &[String], later: &[String]) -> bool {
    if current.iter().any(|exception| exception == "Exception") {
        return true;
    }
    if later.iter().any(|exception| exception == "Exception") {
        return false;
    }
    for (current, later) in current.iter().zip(later) {
        if current == later {
            continue;
        }
        return exception_shadows(current, later);
    }
    false
}

fn constant_in_block(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_constant_write_node().is_none()
        && node.as_class_node().is_none()
        && node.as_module_node().is_none()
    {
        return;
    }
    let Some(block) = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_block_node)
    else {
        return;
    };
    let block_start = block.location().start_offset();
    let block_indent = context.source_file().indentation(block_start).len();
    if context.source_file().lines().any(|(offset, line)| {
        offset > node.location().end_offset()
            && offset < block.location().end_offset()
            && line.trim() == "ensure"
            && context.source_file().indentation(offset).len() == block_indent
    }) {
        return;
    }
    for ancestor in context.ancestors().iter().rev() {
        if ancestor
            .as_block_node()
            .is_some_and(|candidate| candidate.location().start_offset() == block_start)
        {
            break;
        }
        if ancestor
            .as_begin_node()
            .is_some_and(|begin| begin.begin_keyword_loc().is_some())
            || ancestor.as_statements_node().is_none() && ancestor.as_begin_node().is_none()
        {
            return;
        }
    }
    let block_method = context.ancestors().iter().rev().find_map(|ancestor| {
        let call = ancestor.as_call_node()?;
        call.block()
            .and_then(|candidate| candidate.as_block_node())
            .filter(|candidate| candidate.location().start_offset() == block_start)?;
        Some(String::from_utf8_lossy(call.name().as_slice()).into_owned())
    });
    let allowed = block_method.as_deref().is_some_and(|method| {
        if context.config_contains("AllowedMethods") {
            context
                .config_values("AllowedMethods")
                .iter()
                .any(|allowed| allowed == method)
        } else {
            method == "enums"
        }
    });
    if allowed {
        return;
    }
    context.report_node(node, "Do not define constants this way within a block.");
}

fn shadowing_outer_local(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(lambda) = node.as_lambda_node() {
        let Some(block_parameters) = lambda
            .parameters()
            .and_then(|parameters| parameters.as_block_parameters_node())
        else {
            return;
        };
        if let Some(parameters) = block_parameters.parameters() {
            report_shadowing_parameters(&parameters, lambda.location().start_offset(), context);
        }
        report_shadowing_block_locals(
            &block_parameters,
            lambda.location().start_offset(),
            context,
        );
        return;
    }
    let Some(node) = node.as_block_node() else {
        return;
    };
    if context
        .parent()
        .and_then(Node::as_call_node)
        .is_some_and(|call| {
            call_name(&call) == b"new"
                && call
                    .receiver()
                    .and_then(|receiver| receiver.as_constant_read_node())
                    .is_some_and(|constant| constant.name().as_slice() == b"Ractor")
        })
    {
        return;
    }
    let Some(block_parameters) = node
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
    else {
        return;
    };
    if let Some(parameters) = block_parameters.parameters() {
        report_shadowing_parameters(&parameters, node.location().start_offset(), context);
    }
    report_shadowing_block_locals(
        &block_parameters,
        node.location().start_offset(),
        context,
    );
}

fn report_shadowing_block_locals(
    block_parameters: &ruby_prism::BlockParametersNode<'_>,
    cutoff: usize,
    context: &mut CopContext<'_, '_>,
) {
    for local in block_parameters
        .locals()
        .iter()
        .filter_map(|local| local.as_block_local_variable_node())
    {
        let name = String::from_utf8_lossy(local.name().as_slice()).into_owned();
        if !name.starts_with('_')
            && outer_scope_has_local(name.as_bytes(), cutoff, context)
        {
            context.report(
                format!("Shadowing outer local variable - `{name}`."),
                local.location(),
            );
        }
    }
}

fn report_shadowing_parameters(
    parameters: &ruby_prism::ParametersNode<'_>,
    cutoff: usize,
    context: &mut CopContext<'_, '_>,
) {
    for (name, range) in shadowing_parameters(parameters) {
        if name.starts_with('_') || !outer_scope_has_local(name.as_bytes(), cutoff, context) {
            continue;
        }
        context.report(format!("Shadowing outer local variable - `{name}`."), range);
    }
}

fn outer_scope_has_local(
    name: &[u8],
    cutoff: usize,
    context: &CopContext<'_, '_>,
) -> bool {
    for scope in context.ancestors().iter().rev() {
        let (parameters, body) = if let Some(block) = scope.as_block_node() {
            (
                block
                    .parameters()
                    .and_then(|parameters| parameters.as_block_parameters_node())
                    .and_then(|parameters| parameters.parameters()),
                block.body(),
            )
        } else if let Some(lambda) = scope.as_lambda_node() {
            (
                lambda
                    .parameters()
                    .and_then(|parameters| parameters.as_block_parameters_node())
                    .and_then(|parameters| parameters.parameters()),
                lambda.body(),
            )
        } else {
            if scope.as_def_node().is_some() {
                break;
            }
            continue;
        };
        if parameters.is_some_and(|parameters| {
            shadowing_parameters(&parameters)
                .iter()
                .any(|(parameter, _)| parameter.as_bytes() == name)
        }) {
            return true;
        }
        let Some(body) = body else { continue };
        let mut outer = OuterLocalDeclarations {
            declarations: Vec::new(),
        };
        ruby_prism::Visit::visit(&mut outer, &body);
        if outer.declarations.iter().any(|(declared, range)| {
            declared.as_slice() == name
                && range.end < cutoff
                && declaration_in_same_conditional_branch(range.start, cutoff, context)
        }) {
            return true;
        }
    }

    let mut collector = OuterLocalDeclarations {
        declarations: Vec::new(),
    };
    let mut lexical_locals = Vec::new();
    if let Some(definition) = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_def_node)
    {
        if definition.parameters().is_some_and(|parameters| {
            shadowing_parameters(&parameters)
                .iter()
                .any(|(parameter, _)| parameter.as_bytes() == name)
        }) {
            return true;
        }
        lexical_locals.extend(
            definition
                .locals()
                .iter()
                .map(|local| local.as_slice().to_vec()),
        );
        if let Some(body) = definition.body() {
            ruby_prism::Visit::visit(&mut collector, &body);
        }
    } else if let Some(class) = context.ancestors().iter().find_map(Node::as_class_node) {
        if let Some(body) = class.body() {
            ruby_prism::Visit::visit(&mut collector, &body);
        }
    } else if let Some(module) = context.ancestors().iter().find_map(Node::as_module_node) {
        if let Some(body) = module.body() {
            ruby_prism::Visit::visit(&mut collector, &body);
        }
    } else if let Some(program) = context.ancestors().iter().find_map(Node::as_program_node) {
        ruby_prism::Visit::visit(&mut collector, &program.statements().as_node());
    }
    let declarations_for_name = collector
        .declarations
        .iter()
        .filter(|(declared, _)| declared.as_slice() == name)
        .collect::<Vec<_>>();
    if lexical_locals.iter().any(|local| local.as_slice() == name)
        && declarations_for_name.is_empty()
    {
        return true; // A method parameter rather than a body assignment.
    }
    declarations_for_name.into_iter().any(|(_, range)| {
        range.start < cutoff
            && range.end < cutoff
            && declaration_in_same_conditional_branch(range.start, cutoff, context)
    })
}

struct OuterLocalDeclarations {
    declarations: Vec<(Vec<u8>, std::ops::Range<usize>)>,
}

impl<'pr> ruby_prism::Visit<'pr> for OuterLocalDeclarations {
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.declarations.push((
            node.name().as_slice().to_vec(),
            node.location().start_offset()..node.location().end_offset(),
        ));
        ruby_prism::visit_local_variable_write_node(self, node);
    }

    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        self.declarations.push((
            node.name().as_slice().to_vec(),
            node.location().start_offset()..node.location().end_offset(),
        ));
    }

    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode<'pr>) {
        let mut targets = OuterTargetNames::default();
        for target in node
            .lefts()
            .iter()
            .chain(node.rest())
            .chain(node.rights().iter())
        {
            ruby_prism::Visit::visit(&mut targets, &target);
        }
        let range = node.location().start_offset()..node.location().end_offset();
        self.declarations.extend(
            targets
                .names
                .into_iter()
                .map(|name| (name, range.clone())),
        );
        ruby_prism::Visit::visit(self, &node.value());
    }

    fn visit_block_node(&mut self, _node: &ruby_prism::BlockNode<'pr>) {}
    fn visit_lambda_node(&mut self, _node: &ruby_prism::LambdaNode<'pr>) {}
    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
    fn visit_class_node(&mut self, _node: &ruby_prism::ClassNode<'pr>) {}
    fn visit_module_node(&mut self, _node: &ruby_prism::ModuleNode<'pr>) {}
}

#[derive(Default)]
struct OuterTargetNames {
    names: Vec<Vec<u8>>,
}

impl<'pr> ruby_prism::Visit<'pr> for OuterTargetNames {
    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        self.names.push(node.name().as_slice().to_vec());
    }
}

fn declaration_in_same_conditional_branch(
    declaration: usize,
    target: usize,
    context: &CopContext<'_, '_>,
) -> bool {
    for ancestor in context.ancestors() {
        let (selected, declaration_branch) = if let Some(conditional) = ancestor.as_if_node() {
            let branches = [
                conditional
                    .statements()
                    .map(|statements| location_offsets(statements.location())),
                conditional
                    .subsequent()
                    .map(|branch| location_offsets(branch.location())),
            ];
            (
                branches
                    .iter()
                    .flatten()
                    .find(|branch| branch.contains(&target))
                    .cloned(),
                branches
                    .iter()
                    .flatten()
                    .find(|branch| branch.contains(&declaration))
                    .cloned(),
            )
        } else if let Some(conditional) = ancestor.as_unless_node() {
            let branches = [
                conditional
                    .statements()
                    .map(|statements| location_offsets(statements.location())),
                conditional
                    .else_clause()
                    .map(|branch| location_offsets(branch.location())),
            ];
            (
                branches
                    .iter()
                    .flatten()
                    .find(|branch| branch.contains(&target))
                    .cloned(),
                branches
                    .iter()
                    .flatten()
                    .find(|branch| branch.contains(&declaration))
                    .cloned(),
            )
        } else if let Some(case) = ancestor.as_case_node() {
            let branches = case
                .conditions()
                .iter()
                .map(|branch| location_offsets(branch.location()))
                .chain(
                    case.else_clause()
                        .map(|branch| location_offsets(branch.location())),
                )
                .collect::<Vec<_>>();
            let selected = branches
                .iter()
                .find(|branch| branch.contains(&target))
                .cloned();
            let declaration_branch = branches
                .iter()
                .find(|branch| branch.contains(&declaration))
                .cloned();
            if context.related_config_value("AllCops", "ParserEngine") == Some("parser_prism") {
                if declaration_branch.is_some() {
                    continue;
                }
                if case
                    .predicate()
                    .is_some_and(|predicate| location_contains(predicate.location(), declaration))
                {
                    return false;
                }
            }
            (selected, declaration_branch)
        } else {
            continue;
        };
        let Some(selected) = selected else { continue };
        if declaration_branch.is_some_and(|branch| {
            branch.start != selected.start || branch.end != selected.end
        }) {
            return false;
        }
    }
    true
}

fn location_contains(location: ruby_prism::Location<'_>, offset: usize) -> bool {
    location.start_offset() <= offset && offset < location.end_offset()
}

fn location_offsets(location: ruby_prism::Location<'_>) -> std::ops::Range<usize> {
    location.start_offset()..location.end_offset()
}

fn shadowing_parameters(
    parameters: &ruby_prism::ParametersNode<'_>,
) -> Vec<(String, std::ops::Range<usize>)> {
    let mut result = Vec::new();
    for parameter in parameters
        .requireds()
        .iter()
        .chain(parameters.posts().iter())
    {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            let location = parameter.location();
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset(),
            ));
        } else if parameter.as_multi_target_node().is_some() {
            let mut targets = ShadowingParameterTargets::default();
            ruby_prism::Visit::visit(&mut targets, &parameter);
            result.extend(targets.parameters);
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            let location = parameter.name_loc();
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset(),
            ));
        }
    }
    for parameter in parameters.keywords().iter() {
        let extracted = if let Some(parameter) = parameter.as_required_keyword_parameter_node() {
            Some((parameter.name().as_slice(), parameter.name_loc()))
        } else if let Some(parameter) = parameter.as_optional_keyword_parameter_node() {
            Some((parameter.name().as_slice(), parameter.name_loc()))
        } else {
            None
        };
        if let Some((name, location)) = extracted {
            result.push((
                String::from_utf8_lossy(name).into_owned(),
                location.start_offset()..location.end_offset(),
            ));
        }
    }
    for parameter in [parameters.rest(), parameters.keyword_rest()] {
        let Some(parameter) = parameter else { continue };
        let extracted = if let Some(rest) = parameter.as_rest_parameter_node() {
            rest.name()
                .map(|name| (name.as_slice(), rest.location()))
        } else if let Some(rest) = parameter.as_keyword_rest_parameter_node() {
            rest.name()
                .map(|name| (name.as_slice(), rest.location()))
        } else {
            None
        };
        if let Some((name, location)) = extracted {
            result.push((
                String::from_utf8_lossy(name).into_owned(),
                location.start_offset()..location.end_offset(),
            ));
        }
    }
    if let Some(parameter) = parameters.block() {
        if let Some(name) = parameter.name() {
            let location = parameter.location();
            result.push((
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                location.start_offset()..location.end_offset(),
            ));
        }
    }
    result
}

#[derive(Default)]
struct ShadowingParameterTargets {
    parameters: Vec<(String, std::ops::Range<usize>)>,
}

impl<'pr> ruby_prism::Visit<'pr> for ShadowingParameterTargets {
    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode<'pr>) {
        let location = node.location();
        self.parameters.push((
            String::from_utf8_lossy(node.name().as_slice()).into_owned(),
            location.start_offset()..location.end_offset(),
        ));
    }

    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        let location = node.location();
        self.parameters.push((
            String::from_utf8_lossy(node.name().as_slice()).into_owned(),
            location.start_offset()..location.end_offset(),
        ));
    }
}

fn heredoc_case(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let uppercase = context.policy().enforced_style("uppercase") == "uppercase";
    let locations = if let Some(string) = node.as_string_node() {
        string.opening_loc().zip(string.closing_loc())
    } else if let Some(string) = node.as_interpolated_string_node() {
        string.opening_loc().zip(string.closing_loc())
    } else if let Some(string) = node.as_x_string_node() {
        Some((string.opening_loc(), string.closing_loc()))
    } else {
        node.as_interpolated_x_string_node()
            .map(|string| (string.opening_loc(), string.closing_loc()))
    };
    let Some((opening, closing)) = locations else {
        return;
    };
    if !opening.as_slice().starts_with(b"<<") {
        return;
    }
    let closing_source = String::from_utf8_lossy(closing.as_slice());
    let delimiter = closing_source.trim();
    let wrong_case = if uppercase {
        delimiter.bytes().any(|byte| byte.is_ascii_lowercase())
    } else {
        delimiter.bytes().any(|byte| byte.is_ascii_uppercase())
    };
    if delimiter.is_empty() || !wrong_case {
        return;
    }
    let replacement = if uppercase {
        delimiter.to_ascii_uppercase()
    } else {
        delimiter.to_ascii_lowercase()
    };
    let opening_source = String::from_utf8_lossy(opening.as_slice());
    let Some(relative) = opening_source.rfind(delimiter) else {
        return;
    };
    let opening_range =
        opening.start_offset() + relative..opening.start_offset() + relative + delimiter.len();
    let closing_end = closing.end_offset()
        - closing
            .as_slice()
            .iter()
            .rev()
            .take_while(|byte| matches!(byte, b'\n' | b'\r'))
            .count();
    let closing_range = closing.start_offset()..closing_end;
    let closing_edit = closing_end.saturating_sub(delimiter.len())..closing_end;
    context.replace_many(
        if uppercase {
            "Use uppercase heredoc delimiters."
        } else {
            "Use lowercase heredoc delimiters."
        },
        closing_range.clone(),
        vec![
            (opening_range, replacement.clone()),
            (closing_edit, replacement),
        ],
    );
}

fn rescued_exception_name(node: &ruby_prism::RescueNode<'_>, context: &mut CopContext<'_, '_>) {
    let preferred = context
        .config_value("PreferredName")
        .unwrap_or("e")
        .to_string();
    let Some(reference) = node.reference() else {
        return;
    };
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_rescue_node().is_some_and(|rescue| {
            rescue.statements().is_some_and(|statements| {
                statements.location().start_offset() <= node.keyword_loc().start_offset()
                    && node.keyword_loc().start_offset() < statements.location().end_offset()
            })
        })
    }) {
        return;
    }
    let range = reference.location().start_offset()..reference.location().end_offset();
    let actual = context
        .source_file()
        .slice(range.clone())
        .unwrap_or_default();
    if actual.is_empty() || actual.contains('.') {
        return;
    }
    let expected = if actual.starts_with('_') && !preferred.starts_with('_') {
        format!("_{preferred}")
    } else {
        preferred
    };
    let rescue_start = node.keyword_loc().start_offset();
    let assignment_scope_end = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_begin_node)
        .and_then(|begin| begin.begin_keyword_loc())
        .map(|keyword| keyword.start_offset())
        .or_else(|| {
            context
                .ancestors()
                .iter()
                .filter_map(Node::as_rescue_node)
                .map(|rescue| rescue.keyword_loc().start_offset())
                .min()
        })
        .unwrap_or(rescue_start);
    if identifier_assigned(&context.source()[..assignment_scope_end], &expected) {
        return;
    }
    if node.statements().is_some_and(|statements| {
        identifier_assigned(context.source_file().node(&statements.as_node()), &expected)
    }) {
        return;
    }
    if actual != expected {
        let mut edits = vec![(range.clone(), expected.clone())];
        let scope_end = context
            .ancestors()
            .iter()
            .rev()
            .find_map(Node::as_def_node)
            .map_or(context.source().len(), |definition| {
                definition.location().end_offset()
            });
        let search_start = node
            .statements()
            .map_or(range.end, |statements| statements.location().start_offset());
        let assignment =
            first_identifier_assignment(context.source(), actual, search_start, scope_end);
        let search_end = assignment
            .and_then(|at| {
                node.statements().and_then(|statements| {
                    statements
                        .body()
                        .iter()
                        .find(|statement| {
                            statement.location().start_offset() <= at
                                && at < statement.location().end_offset()
                        })
                        .map(|statement| statement.location().end_offset())
                })
            })
            .unwrap_or_else(|| {
                assignment.map_or(scope_end, |at| context.source_file().line_end(at))
            });
        let assignment_equal = assignment.and_then(|at| {
            context.source()[at..search_end]
                .find('=')
                .map(|relative| at + relative)
        });
        {
            let mut search = search_start;
            while let Some(relative) = context.source()[search..search_end].find(actual) {
                let start = search + relative;
                let before = context.source().as_bytes().get(start.wrapping_sub(1));
                let after = context.source().as_bytes().get(start + actual.len());
                if assignment
                    .zip(assignment_equal)
                    .is_some_and(|(assignment, equal)| assignment <= start && start <= equal)
                {
                    search = start + actual.len();
                    continue;
                }
                if after == Some(&b':') {
                    let value = context.source()[start + actual.len() + 1..search_end].trim_start();
                    if value.chars().next().is_some_and(|c| matches!(c, ',' | ')')) {
                        edits.push((
                            start + actual.len() + 1..start + actual.len() + 1,
                            format!(" {expected}"),
                        ));
                    }
                    search = start + actual.len();
                    continue;
                }
                if !before.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                    && !after.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                {
                    edits.push((start..start + actual.len(), expected.clone()));
                }
                search = start + actual.len();
            }
        }
        context.replace_many(
            format!("Use `{expected}` instead of `{actual}`."),
            range,
            edits,
        );
    }
}

fn first_identifier_assignment(
    source: &str,
    name: &str,
    start: usize,
    end: usize,
) -> Option<usize> {
    source[start..end]
        .match_indices(name)
        .find_map(|(relative, _)| {
            let at = start + relative;
            let before = source.as_bytes().get(at.wrapping_sub(1));
            let after = source.as_bytes().get(at + name.len());
            if before.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                || after.is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            {
                return None;
            }
            let line_start = source[..at].rfind('\n').map_or(0, |line| line + 1);
            let line_end = source[at..end].find('\n').map_or(end, |line| at + line);
            assignment_equal(&source[line_start..line_end])
                .map(|equal| line_start + equal)
                .filter(|equal| at < *equal)
                .map(|_| at)
        })
}

fn identifier_assigned(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(start, _)| {
        let before = source[..start].bytes().next_back();
        let after = source[start + name.len()..].trim_start();
        !before.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && after.starts_with('=')
            && !after.starts_with("==")
            && !after.starts_with("=>")
    })
}

fn block_forwarding(definition: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("anonymous");
    if style == "explicit" {
        explicit_block_forwarding(definition, context);
        return;
    }
    if style != "anonymous" {
        return;
    }
    if !context.target_ruby_version().at_least(3, 1) {
        return;
    }
    let Some(parameter) = definition
        .parameters()
        .and_then(|parameters| parameters.block())
    else {
        return;
    };
    let Some(name) = parameter.name() else { return };
    if definition
        .parameters()
        .is_some_and(|parameters| !parameters.keywords().is_empty())
    {
        return;
    }
    let mut usage = BlockForwardingUsage {
        name: name.as_slice(),
        forwarded: Vec::new(),
        other_use: false,
        nested_block_depth: 0,
        allow_nested: context.target_ruby_version().at_least(3, 4),
    };
    if let Some(body) = definition.body() {
        ruby_prism::Visit::visit(&mut usage, &body);
    }
    if usage.other_use {
        return;
    }
    let range = parameter.location().start_offset()..parameter.location().end_offset();
    let mut edits = usage
        .forwarded
        .iter()
        .cloned()
        .map(|forwarded| (forwarded, "&".to_string()))
        .collect::<Vec<_>>();
    edits.push((range.clone(), "&".to_string()));
    let mut parenthesize = Vec::new();
    for target in edits
        .iter()
        .map(|(range, _)| range.clone())
        .collect::<Vec<_>>()
    {
        let line_start = context.source_file().line_start(target.start);
        let prefix = &context.source()[line_start..target.start];
        if prefix.contains('(') {
            continue;
        }
        if let Some(space) = unparenthesized_call_separator(prefix) {
            let at = line_start + space;
            parenthesize.push((at..at + 1, "(".to_string()));
            parenthesize.push((target.end..target.end, ")".to_string()));
        }
    }
    edits.extend(parenthesize);
    for (index, forwarded) in usage.forwarded.iter().enumerate() {
        if index == 0 {
            context.replace_many(
                "Use anonymous block forwarding.",
                forwarded.clone(),
                edits.clone(),
            );
        } else {
            context.replace(
                "Use anonymous block forwarding.",
                forwarded.clone(),
                forwarded.start..forwarded.start,
                "",
            );
        }
    }
    if usage.forwarded.is_empty() {
        context.replace("Use anonymous block forwarding.", range.clone(), range, "&");
    } else {
        context.replace(
            "Use anonymous block forwarding.",
            range.clone(),
            range.start..range.start,
            "",
        );
    }
}

fn unparenthesized_call_separator(prefix: &str) -> Option<usize> {
    let indentation = prefix.len() - prefix.trim_start().len();
    let content = &prefix[indentation..];
    let selector_start = if content.starts_with("def ") { 4 } else { 0 };
    content[selector_start..]
        .find(char::is_whitespace)
        .map(|at| indentation + selector_start + at)
}

struct BlockForwardingUsage<'a> {
    name: &'a [u8],
    forwarded: Vec<std::ops::Range<usize>>,
    other_use: bool,
    nested_block_depth: usize,
    allow_nested: bool,
}

impl<'pr> ruby_prism::Visit<'pr> for BlockForwardingUsage<'_> {
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode<'pr>) {
        if node
            .expression()
            .and_then(|value| value.as_local_variable_read_node())
            .is_some_and(|read| read.name().as_slice() == self.name)
        {
            if self.nested_block_depth > 0 && !self.allow_nested {
                self.other_use = true;
            }
            self.forwarded
                .push(node.location().start_offset()..node.location().end_offset());
            return;
        }
        ruby_prism::visit_block_argument_node(self, node);
    }

    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
    }

    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
        ruby_prism::visit_local_variable_write_node(self, node);
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
        ruby_prism::visit_local_variable_or_write_node(self, node);
    }

    fn visit_local_variable_and_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
        ruby_prism::visit_local_variable_and_write_node(self, node);
    }

    fn visit_local_variable_operator_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
        ruby_prism::visit_local_variable_operator_write_node(self, node);
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        self.nested_block_depth += 1;
        ruby_prism::visit_block_node(self, node);
        self.nested_block_depth -= 1;
    }
}

fn explicit_block_forwarding(
    definition: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if !context.target_ruby_version().at_least(3, 1) {
        return;
    }
    let Some(parameter) = definition
        .parameters()
        .and_then(|parameters| parameters.block())
    else {
        return;
    };
    if parameter.name().is_some() {
        return;
    }
    let name = context
        .config_value("BlockForwardingName")
        .unwrap_or("block")
        .to_string();
    let in_use = definition.body().is_some_and(|body| {
        context
            .source_file()
            .node(&body)
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .any(|word| word == name)
    });
    let mut ranges = Vec::new();
    if let Some(body) = definition.body() {
        let mut finder = AnonymousBlockForwarding { ranges: Vec::new() };
        ruby_prism::Visit::visit(&mut finder, &body);
        ranges = finder.ranges;
    }
    ranges.push(parameter.location().start_offset()..parameter.location().end_offset());
    for range in ranges {
        if in_use {
            context.report("Use explicit block forwarding.", range);
        } else {
            context.replace(
                "Use explicit block forwarding.",
                range.clone(),
                range,
                format!("&{name}"),
            );
        }
    }
}

struct AnonymousBlockForwarding {
    ranges: Vec<std::ops::Range<usize>>,
}

impl<'pr> ruby_prism::Visit<'pr> for AnonymousBlockForwarding {
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode<'pr>) {
        if node.expression().is_none() {
            self.ranges
                .push(node.location().start_offset()..node.location().end_offset());
            return;
        }
        ruby_prism::visit_block_argument_node(self, node);
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
}

fn constant_reassignment(context: &mut CopContext<'_, '_>) {
    #[derive(Clone)]
    enum Scope {
        Namespace(String),
        Opaque,
    }

    let mut assigned = HashSet::new();
    let mut scopes = Vec::<Scope>::new();
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim();
        let namespace = scopes
            .iter()
            .filter_map(|scope| match scope {
                Scope::Namespace(name) => Some(name.as_str()),
                Scope::Opaque => None,
            })
            .last()
            .unwrap_or("")
            .to_string();
        let opaque = scopes.iter().any(|scope| matches!(scope, Scope::Opaque));

        if let Some(rest) = trimmed
            .strip_prefix("class ")
            .or_else(|| trimmed.strip_prefix("module "))
        {
            let raw_name = rest.split([' ', '<', ';']).next().unwrap_or("").trim();
            if !opaque && constant_path(raw_name) {
                let full_name = resolve_constant_path(raw_name, &namespace);
                assigned.insert(full_name.clone());
                if !trimmed.contains("; end") && !trimmed.ends_with(";end") {
                    scopes.push(Scope::Namespace(full_name));
                }
            } else if !trimmed.contains("; end") && !trimmed.ends_with(";end") {
                scopes.push(Scope::Opaque);
            }
            continue;
        }

        if trimmed == "end" || trimmed.starts_with("end ") {
            scopes.pop();
            continue;
        }

        if ["if ", "unless ", "case ", "begin", "def "]
            .iter()
            .any(|keyword| trimmed.starts_with(keyword))
            || trimmed.ends_with(" do")
            || trimmed.contains(" do |")
        {
            scopes.push(Scope::Opaque);
            continue;
        }
        if opaque || line.contains("||=") || line.contains("&&=") {
            continue;
        }

        if let Some(argument) = trimmed
            .strip_prefix("remove_const ")
            .or_else(|| trimmed.strip_prefix("self.remove_const "))
        {
            let name = argument.trim_matches([':', '\'', '"']);
            assigned.remove(&resolve_constant_path(name, &namespace));
            continue;
        }

        let Some(equal) = assignment_equal(line) else {
            continue;
        };
        if line[equal + 1..].contains(" unless ") || line[equal + 1..].contains(" if ") {
            continue;
        }
        let before_equal = &line[..equal];
        let multiple = before_equal.contains(',');
        for raw in before_equal.split(',') {
            let candidate = raw.trim();
            let candidate = candidate
                .rsplit_once(|character: char| {
                    !(character.is_ascii_alphanumeric() || matches!(character, '_' | ':'))
                })
                .map_or(candidate, |(_, tail)| tail);
            if !constant_path(candidate) {
                continue;
            }
            let full_name = resolve_constant_path(candidate, &namespace);
            if assigned.insert(full_name) {
                continue;
            }
            let display = candidate
                .strip_prefix("self::")
                .unwrap_or(candidate)
                .trim_start_matches("::");
            let start_in_line = line.find(candidate).unwrap_or(0);
            let end_in_line = if multiple {
                start_in_line + candidate.len()
            } else {
                line.trim_end().trim_end_matches(',').len()
            };
            context.report(
                format!("Constant `{display}` is already assigned in this namespace."),
                offset + start_in_line..offset + end_in_line,
            );
        }
    }
}

fn assignment_equal(line: &str) -> Option<usize> {
    line.char_indices().find_map(|(index, character)| {
        if character != '=' {
            return None;
        }
        let before = line.as_bytes().get(index.wrapping_sub(1)).copied();
        let after = line.as_bytes().get(index + 1).copied();
        (!matches!(before, Some(b'=' | b'!' | b'<' | b'>' | b'|' | b'&'))
            && !matches!(after, Some(b'=' | b'>')))
        .then_some(index)
    })
}

fn constant_path(candidate: &str) -> bool {
    let candidate = candidate
        .strip_prefix("self::")
        .unwrap_or(candidate)
        .trim_start_matches("::");
    !candidate.is_empty()
        && candidate.split("::").all(|part| {
            part.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
                && part
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
}

fn resolve_constant_path(candidate: &str, namespace: &str) -> String {
    if let Some(candidate) = candidate.strip_prefix("::") {
        return candidate.to_string();
    }
    let candidate = candidate.strip_prefix("self::").unwrap_or(candidate);
    if namespace.is_empty() {
        candidate.to_string()
    } else {
        format!("{namespace}::{candidate}")
    }
}
