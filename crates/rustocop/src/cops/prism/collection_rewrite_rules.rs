use ruby_prism::{BlockNode, CallNode, Node};

use super::*;

define_rule!(PredicateWithKindRule);
define_rule!(ReduceToHashRule);
define_stateful_rule!(PartitionRule, PartitionState);

define_cops! {
    PredicateWithKind => "Style/PredicateWithKind" => call_rule(
        PredicateWithKindRule,
        on_send,
        restrict [b"any?", b"all?", b"none?", b"one?"]
    ),
    ReduceToHash => "Style/ReduceToHash" => call_rule(
        ReduceToHashRule,
        on_send,
        restrict [b"each_with_object", b"inject", b"reduce"]
    ),
    PartitionInsteadOfDoubleSelect => "Style/PartitionInsteadOfDoubleSelect" => stateful_call_rule(
        PartitionRule,
        PartitionState,
        on_send,
        restrict [b"select", b"filter", b"find_all", b"reject"]
    ),
}

impl PredicateWithKindRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some(block) = node.block().and_then(|block| block.as_block_node()) else { return };
        let Some((parameter, body)) = kind_check(&block, self.source_file()) else { return };
        let Some(kind_call) = body.as_call_node() else { return };
        return_unless!(matches!(kind_call.name().as_slice(), b"is_a?" | b"kind_of?" | b"instance_of?"));
        let Some(receiver) = kind_call.receiver() else { return };
        return_unless!(self.source_file().node(&receiver) == parameter);
        let Some(klass) = only_argument(&kind_call) else { return };
        let Some(selector) = node.message_loc() else { return };
        let method = String::from_utf8_lossy(node.name().as_slice());
        let klass_source = self.source_file().node(&klass);
        let replacement = format!("{method}({klass_source})");
        let message = format!("Prefer `{replacement}` to `{method} {{ ... }}` with a kind check.");
        let offense = node.location().start_offset()..block.closing_loc().end_offset();
        let edit = selector.start_offset()..block.closing_loc().end_offset();
        add_offense!(self, offense, message: message, |corrector| {
            corrector.replace(edit, replacement);
        });
    }
}

impl ReduceToHashRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(!self.target_ruby_version().at_least(2, 6));
        let Some(block) = node.block().and_then(|block| block.as_block_node()) else { return };
        let arguments = call_arguments_for_reduce(node);
        return_unless!(arguments.len() == 1 && self.source_file().node(&arguments[0]).trim() == "{}");
        let Some(pattern) = reduce_to_hash_pattern(node, &block, self.source_file()) else { return };
        return_if!(references_name(&pattern.key, &pattern.accumulator, self.source_file())
            || references_name(&pattern.value, &pattern.accumulator, self.source_file()));
        let key = self.source_file().node(&pattern.key);
        let value = self.source_file().node(&pattern.value);
        return_if!(contains_nested_reduce(key) || contains_nested_reduce(value));
        let method = String::from_utf8_lossy(node.name().as_slice());
        let message = format!("Use `to_h {{ ... }}` instead of `{method}`.");
        let Some(selector) = node.message_loc() else { return };
        let numbered = pattern.numbered;
        let key = if numbered && method != "each_with_object" { key.replace("_2", "_1") } else { key.to_string() };
        let value = if numbered && method != "each_with_object" { value.replace("_2", "_1") } else { value.to_string() };
        let body = format!("[{key}, {value}]");
        let braces = block.opening_loc().as_slice() == b"{";
        let replacement = if braces {
            if numbered { format!("to_h {{ {body} }}") } else { format!("to_h {{ |{}| {body} }}", pattern.element) }
        } else {
            let indent = self.source_file().indentation_text(node.location().start_offset());
            let args = if numbered { String::new() } else { format!(" |{}|", pattern.element) };
            format!("to_h do{args}\n{indent}  {body}\n{indent}end")
        };
        let mut edit = selector.start_offset()..block.closing_loc().end_offset();
        let mut replacement = replacement;
        if let Some((outer_edit, outer_replacement)) = self.outer_reduce_replacement(edit.clone(), &replacement) {
            edit = outer_edit;
            replacement = outer_replacement;
        }
        add_offense!(self, selector, message: message, |corrector| {
            corrector.replace(edit, replacement);
        });
    }

    fn outer_reduce_replacement(&self, inner_edit: std::ops::Range<usize>, inner_replacement: &str) -> Option<(std::ops::Range<usize>, String)> {
        for ancestor in self.ancestors().iter().rev() {
            let Some(call) = ancestor.as_call_node() else { continue };
            if !matches!(call.name().as_slice(), b"each_with_object" | b"inject" | b"reduce") { continue; }
            let block = call.block()?.as_block_node()?;
            let pattern = reduce_to_hash_pattern(&call, &block, self.source_file())?;
            let selector = call.message_loc()?;
            let outer_edit = selector.start_offset()..block.closing_loc().end_offset();
            if !(outer_edit.start <= inner_edit.start && inner_edit.end <= outer_edit.end) { continue; }
            let key = self.source_file().node(&pattern.key);
            let value_location = pattern.value.location();
            if !(value_location.start_offset() <= inner_edit.start && inner_edit.end <= value_location.end_offset()) { continue; }
            let mut value = self.source_file().node(&pattern.value).to_string();
            let relative = inner_edit.start - value_location.start_offset()..inner_edit.end - value_location.start_offset();
            value.replace_range(relative, inner_replacement);
            let replacement = format!("to_h {{ |{}| [{key}, {value}] }}", pattern.element);
            return Some((outer_edit, replacement));
        }
        None
    }
}

struct ReducePattern<'pr> {
    element: String,
    accumulator: String,
    key: Node<'pr>,
    value: Node<'pr>,
    numbered: bool,
}

fn reduce_to_hash_pattern<'pr>(node: &CallNode<'pr>, block: &BlockNode<'pr>, file: SourceFile<'_>) -> Option<ReducePattern<'pr>> {
    let method = node.name().as_slice();
    let parameters = block.parameters()?;
    let (first, second, numbered) = if let Some(parameters) = parameters.as_block_parameters_node() {
        let required = parameters.parameters()?.requireds();
        if required.len() != 2 { return None; }
        if required.iter().any(|parameter| parameter.as_required_parameter_node().is_none()) { return None; }
        (file.node(&required.first()?).to_string(), file.node(&required.iter().nth(1)?).to_string(), false)
    } else if parameters.as_numbered_parameters_node().is_some_and(|parameters| parameters.maximum() == 2) {
        ("_1".to_string(), "_2".to_string(), true)
    } else {
        return None;
    };
    let body = block.body()?.as_statements_node()?;
    let statements = body.body().iter().collect::<Vec<_>>();
    let (element, accumulator, assignment) = if method == b"each_with_object" {
        if statements.len() != 1 { return None; }
        (first, second, body.body().iter().next()?)
    } else {
        if statements.len() != 2 || file.node(&statements[1]) != first { return None; }
        (second, first, body.body().iter().next()?)
    };
    let assignment = assignment.as_call_node()?;
    if assignment.name().as_slice() != b"[]=" || assignment.receiver().is_none() { return None; }
    if file.node(&assignment.receiver()?) != accumulator { return None; }
    let arguments = call_arguments_for_reduce(&assignment);
    if arguments.len() != 2 { return None; }
    let mut arguments = arguments.into_iter();
    Some(ReducePattern { element, accumulator, key: arguments.next()?, value: arguments.next()?, numbered })
}

fn call_arguments_for_reduce<'pr>(node: &CallNode<'pr>) -> Vec<Node<'pr>> {
    node.arguments().map(|arguments| arguments.arguments().iter().collect()).unwrap_or_default()
}

fn references_name(node: &Node<'_>, name: &str, file: SourceFile<'_>) -> bool {
    file.node(node).split(|character: char| !character.is_ascii_alphanumeric() && character != '_').any(|part| part == name)
}

fn contains_nested_reduce(source: &str) -> bool {
    [".each_with_object({})", ".inject({})", ".reduce({})"].iter().any(|needle| source.contains(needle))
}

#[derive(Default)]
struct PartitionState {
    previous: Option<PartitionCandidate>,
}

struct PartitionCandidate {
    method: String,
    receiver: String,
    predicate: String,
    symbol_method: Option<String>,
    negated: bool,
    call_source: String,
    selector: std::ops::Range<usize>,
    container: std::ops::Range<usize>,
    full_line: std::ops::Range<usize>,
    local_variable: Option<String>,
}

impl PartitionRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some(candidate) = partition_candidate(node, self) else { return };
        let Some(previous) = self.state.previous.take() else {
            self.state.previous = Some(candidate);
            return;
        };
        if previous.full_line.end != candidate.full_line.start
            || previous.receiver != candidate.receiver
            || !partition_pair_matches(&previous, &candidate)
        {
            self.state.previous = Some(candidate);
            return;
        }
        let message = format!(
            "Use `partition` instead of consecutive `{}` and `{}` calls.",
            previous.method, candidate.method
        );
        let offense = candidate.container.clone();
        let (Some(previous_var), Some(current_var)) = (&previous.local_variable, &candidate.local_variable) else {
            self.report(message, offense);
            self.state.previous = Some(candidate);
            return;
        };
        let previous_select = is_select_method(&previous.method);
        let current_select = is_select_method(&candidate.method);
        let (truthy, falsey, partition) = if previous_select != current_select {
            if previous_select {
                (previous_var, current_var, &previous)
            } else {
                (current_var, previous_var, &candidate)
            }
        } else {
            let (nonnegated_var, negated_var, nonnegated) = if previous.negated {
                (current_var, previous_var, &candidate)
            } else {
                (previous_var, current_var, &previous)
            };
            if previous_select {
                (nonnegated_var, negated_var, nonnegated)
            } else {
                (negated_var, nonnegated_var, nonnegated)
            }
        };
        let partition_call = replace_selector(&partition.call_source, partition.selector.clone(), "partition");
        let replacement = format!("{truthy}, {falsey} = {partition_call}");
        add_offense!(self, offense, message: message, |corrector| {
            corrector.replace(previous.container.clone(), replacement);
            corrector.remove(candidate.full_line.clone());
        });
    }
}

fn partition_candidate(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> Option<PartitionCandidate> {
    let receiver = context.source_file().node(&node.receiver()?).to_string();
    let method = String::from_utf8_lossy(node.name().as_slice()).into_owned();
    let selector = node.message_loc()?;
    let (predicate, symbol_method, negated, call_end) = if let Some(block) = node.block().and_then(|block| block.as_block_node()) {
        let body = block.body().and_then(single_expression)?;
        let mut body_source = context.source_file().node(&body).trim().to_string();
        let negated = body_source.starts_with('!');
        if negated { body_source = body_source[1..].trim().to_string(); }
        let parameter = block_parameter_source(&block, context.source_file());
        let symbol_method = parameter.as_ref().and_then(|parameter| {
            body_source.strip_prefix(&format!("{parameter}.")).filter(|method| method.ends_with('?') || method.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')).map(str::to_string)
        });
        let parameter_key = parameter.clone().unwrap_or_default();
        (format!("{parameter_key}|{body_source}"), symbol_method, negated, block.closing_loc().end_offset())
    } else {
        let argument = node.block().or_else(|| node.arguments().and_then(|arguments| arguments.arguments().iter().last()))?;
        let source = context.source_file().node(&argument);
        let method_name = source.strip_prefix("&:")?.to_string();
        (format!("symbol:{method_name}"), Some(method_name), false, node.location().end_offset())
    };
    let call_start = node.location().start_offset();
    let call_range = call_start..call_end;
    let (container, local_variable) = assignment_container(context.ancestors(), &call_range, context.source_file())
        .unwrap_or((call_range.clone(), None));
    let full_line = context.source_file().full_line_range(container.clone());
    let call_source = context.source()[call_range.clone()].to_string();
    Some(PartitionCandidate {
        method,
        receiver,
        predicate,
        symbol_method,
        negated,
        selector: selector.start_offset() - call_start..selector.end_offset() - call_start,
        call_source,
        container,
        full_line,
        local_variable,
    })
}

fn block_parameter_source(block: &BlockNode<'_>, file: SourceFile<'_>) -> Option<String> {
    let parameters = block.parameters()?;
    if let Some(parameters) = parameters.as_block_parameters_node() {
        let required = parameters.parameters()?.requireds();
        (required.len() == 1).then(|| file.node(&required.first().expect("one parameter")).to_string())
    } else if parameters.as_numbered_parameters_node().is_some() {
        Some("_1".to_string())
    } else if parameters.as_it_parameters_node().is_some() {
        Some("it".to_string())
    } else {
        None
    }
}

fn assignment_container(ancestors: &[Node<'_>], call: &std::ops::Range<usize>, file: SourceFile<'_>) -> Option<(std::ops::Range<usize>, Option<String>)> {
    for ancestor in ancestors.iter().rev() {
        let (location, name) = if let Some(write) = ancestor.as_local_variable_write_node() {
            (write.location(), Some(file.at(&write.name_loc()).to_string()))
        } else if let Some(write) = ancestor.as_instance_variable_write_node() {
            (write.location(), None)
        } else if let Some(write) = ancestor.as_class_variable_write_node() {
            (write.location(), None)
        } else if let Some(write) = ancestor.as_global_variable_write_node() {
            (write.location(), None)
        } else {
            continue;
        };
        if location.start_offset() <= call.start && call.end <= location.end_offset() {
            return Some((location.start_offset()..call.end.max(location.end_offset()), name));
        }
    }
    None
}

fn partition_pair_matches(left: &PartitionCandidate, right: &PartitionCandidate) -> bool {
    let complementary = is_select_method(&left.method) != is_select_method(&right.method)
        && predicates_equivalent(left, right);
    let negated = left.method == right.method && left.negated != right.negated
        && left.predicate == right.predicate;
    complementary || negated
}

fn predicates_equivalent(left: &PartitionCandidate, right: &PartitionCandidate) -> bool {
    left.predicate == right.predicate
        || left.symbol_method.is_some() && left.symbol_method == right.symbol_method
}

fn is_select_method(method: &str) -> bool {
    matches!(method, "select" | "filter" | "find_all")
}

fn replace_selector(source: &str, selector: std::ops::Range<usize>, replacement: &str) -> String {
    let mut source = source.to_string();
    source.replace_range(selector, replacement);
    source
}

fn kind_check<'pr>(block: &BlockNode<'pr>, file: SourceFile<'_>) -> Option<(String, Node<'pr>)> {
    let parameters = block.parameters()?;
    let parameter = if let Some(parameters) = parameters.as_block_parameters_node() {
        let requireds = parameters.parameters()?.requireds();
        if requireds.len() != 1 { return None; }
        file.node(&requireds.first()?).to_string()
    } else if let Some(numbered) = parameters.as_numbered_parameters_node() {
        if numbered.maximum() != 1 { return None; }
        "_1".to_string()
    } else if parameters.as_it_parameters_node().is_some() {
        "it".to_string()
    } else {
        return None;
    };
    let body = block.body().and_then(single_expression)?;
    Some((parameter, body))
}
