use super::*;

define_rule!(ZeroLengthPredicateRule);

const ZERO_MSG: &str = "Use `empty?` instead of `{current}`.";
const NONZERO_MSG: &str = "Use `!empty?` instead of `{current}`.";

define_cops! {
    ArrayIntersect => "Style/ArrayIntersect" => compatibility_prism_any_node(array_intersect),
    TallyMethod => "Style/TallyMethod" => compatibility_prism_call(tally_method),
    ZeroLengthPredicate => "Style/ZeroLengthPredicate" => compatibility_prism_call_rule(ZeroLengthPredicateRule, on_send, restrict [b"size", b"length"]),
}

fn array_intersect(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 1) { return; }
    if let Some(block) = node.as_block_node() {
        array_intersect_block(&block, context);
        return;
    }
    let Some(call) = node.as_call_node() else { return };
    if call.block().is_some() { return; }
    let active_support = context.related_config_value("AllCops", "ActiveSupportExtensionsEnabled") == Some("true");
    let name = call_name(&call);
    let direct = matches!(name, b"any?" | b"empty?" | b"none?")
        || active_support && matches!(name, b"present?" | b"blank?");
    let (left, right, dot, straight) = if direct {
        let Some(receiver) = call.receiver() else { return };
        let Some((left, right, dot)) = array_intersection_parts(&receiver, context.source_file()) else { return };
        (left, right, dot, matches!(name, b"any?" | b"present?"))
    } else if matches!(name, b">" | b"==" | b"!=") {
        let Some(zero) = first_argument(&call).and_then(|argument| argument.as_integer_node()) else { return };
        if context.source_file().at(&zero.location()) != "0" { return; }
        let Some(size) = call.receiver().and_then(|receiver| receiver.as_call_node()) else { return };
        if !matches!(call_name(&size), b"count" | b"length" | b"size") { return; }
        let Some(intersection) = size.receiver() else { return };
        let Some((left, right, dot)) = array_intersection_parts(&intersection, context.source_file()) else { return };
        (left, right, dot, matches!(name, b">" | b"!="))
    } else if matches!(name, b"zero?" | b"positive?") {
        let Some(size) = call.receiver().and_then(|receiver| receiver.as_call_node()) else { return };
        if !matches!(call_name(&size), b"count" | b"length" | b"size") { return; }
        let Some(intersection) = size.receiver() else { return };
        let Some((left, right, dot)) = array_intersection_parts(&intersection, context.source_file()) else { return };
        (left, right, dot, name == b"positive?")
    } else { return };
    let replacement = format!("{}{left}{dot}intersect?({right})", if straight { "" } else { "!" });
    let location = call.location();
    let existing = context.source_file().at(&location);
    context.replace(
        format!("Use `{replacement}` instead of `{existing}`."),
        location.start_offset()..location.end_offset(),
        location.start_offset()..location.end_offset(),
        replacement,
    );
}

fn array_intersection_parts(node: &Node<'_>, file: SourceFile<'_>) -> Option<(String, String, String)> {
    if let Some(parentheses) = node.as_parentheses_node() {
        let inner = parentheses.body().and_then(single_expression)?;
        return array_intersection_parts(&inner, file);
    }
    let call = node.as_call_node()?;
    if call_name(&call) == b"&" {
        let left = call.receiver()?;
        let right = first_argument(&call)?;
        return Some((file.node(&left).to_string(), file.node(&right).to_string(), ".".to_string()));
    }
    if call_name(&call) != b"intersection" || argument_count(&call) != 1 { return None; }
    let left = call.receiver()?;
    let right = first_argument(&call)?;
    let dot = call.call_operator_loc().map_or(".", |operator| file.at(&operator)).to_string();
    Some((file.node(&left).to_string(), file.node(&right).to_string(), dot))
}

fn array_intersect_block(block: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(call) = context.parent().and_then(Node::as_call_node) else { return };
    if !matches!(call_name(&call), b"any?" | b"none?") { return; }
    let Some(receiver) = call.receiver() else { return };
    let Some(member) = block.body().and_then(|body| body.as_statements_node())
        .and_then(|statements| statements.body().last())
        .and_then(|body| body.as_call_node())
    else { return };
    if call_name(&member) != b"member?" || argument_count(&member) != 1 { return; }
    let Some(argument) = first_argument(&member) else { return };
    let local_name = argument.as_local_variable_read_node().map(|argument| argument.name().as_slice().to_vec());
    let expected = block.parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
        .and_then(|parameters| parameters.parameters())
        .and_then(|parameters| parameters.requireds().first())
        .and_then(|parameter| parameter.as_required_parameter_node())
        .map(|parameter| parameter.name().as_slice().to_vec())
        .or_else(|| (local_name.as_deref() == Some(b"_1")).then(|| b"_1".to_vec()))
        .or_else(|| (context.target_ruby_version().at_least(3, 4) && argument.as_it_local_variable_read_node().is_some()).then(|| b"it".to_vec()));
    let actual = local_name.as_deref().or_else(|| argument.as_it_local_variable_read_node().map(|_| b"it".as_slice()));
    if expected.as_deref() != actual { return; }
    let Some(other) = member.receiver() else { return };
    let dot = call.call_operator_loc().map_or(".", |operator| context.source_file().at(&operator));
    let replacement = format!("{}{}{dot}intersect?({})",
        if call_name(&call) == b"none?" { "!" } else { "" },
        context.source_file().node(&receiver), context.source_file().node(&other));
    let range = call.location().start_offset()..block.location().end_offset();
    let existing = &context.source()[range.clone()];
    context.replace(format!("Use `{replacement}` instead of `{existing}`."), range.clone(), range, replacement);
}

fn tally_method(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 7) {
        return;
    }
    if call_name(node) == b"each_with_object" && each_with_object_tally(node, context) {
        let Some(selector) = node.message_loc() else {
            return;
        };
        context.replace(
            "Use `tally` instead of `each_with_object`.",
            &selector,
            selector.start_offset()..node.location().end_offset(),
            "tally",
        );
    } else if call_name(node) == b"transform_values" && transform_values_tally(node) {
        let Some(group_by) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
            return;
        };
        let Some(selector) = group_by.message_loc() else {
            return;
        };
        context.replace(
            "Use `tally` instead of `group_by` and `transform_values`.",
            &selector,
            selector.start_offset()..node.location().end_offset(),
            "tally",
        );
    }
}

fn each_with_object_tally(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    if argument_count(node) != 1 {
        return false;
    }
    let Some(initializer) = only_argument(node).and_then(|argument| argument.as_call_node()) else {
        return false;
    };
    if call_name(&initializer) != b"new" || argument_count(&initializer) != 1 {
        return false;
    }
    let hash_receiver = initializer
        .receiver()
        .is_some_and(|receiver| matches!(context.source_file().node(&receiver), "Hash" | "::Hash"));
    let zero = only_argument(&initializer)
        .and_then(|argument| argument.as_integer_node())
        .is_some_and(|integer| TryInto::<i32>::try_into(integer.value()).ok() == Some(0));
    let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
        return false;
    };
    let Some(body) = block.body().and_then(single_expression) else {
        return false;
    };
    let Some(write) = body.as_index_operator_write_node() else {
        return false;
    };
    if write.binary_operator().as_slice() != b"+"
        || write
            .value()
            .as_integer_node().is_none_or(|integer| TryInto::<i32>::try_into(integer.value()).ok() != Some(1))
        || write
            .arguments()
            .is_none_or(|arguments| arguments.arguments().len() != 1)
    {
        return false;
    }
    let Some(receiver) = write
        .receiver()
        .and_then(|receiver| receiver.as_local_variable_read_node())
    else {
        return false;
    };
    let Some(key) = write
        .arguments()
        .and_then(|arguments| arguments.arguments().first())
        .and_then(|argument| argument.as_local_variable_read_node())
    else {
        return false;
    };
    hash_receiver && zero && tally_block_parameters(&block, receiver.name().as_slice(), key.name().as_slice())
}

fn tally_block_parameters(block: &ruby_prism::BlockNode<'_>, hash: &[u8], element: &[u8]) -> bool {
    let Some(parameters) = block.parameters() else {
        return false;
    };
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return numbered.maximum() == 2 && hash == b"_2" && element == b"_1";
    }
    let Some(parameters) = parameters
        .as_block_parameters_node()
        .and_then(|parameters| parameters.parameters())
    else {
        return false;
    };
    if parameters.requireds().len() != 2
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return false;
    }
    let Some(first) = parameters
        .requireds()
        .first()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    let Some(second) = parameters
        .requireds()
        .last()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    first.name().as_slice() == element && second.name().as_slice() == hash
}

fn transform_values_tally(node: &CallNode<'_>) -> bool {
    if argument_count(node) != 0 {
        return false;
    }
    let Some(group_by) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return false;
    };
    call_name(&group_by) == b"group_by"
        && argument_count(&group_by) == 0
        && group_by_identity(&group_by)
        && transform_counts(node)
}

fn group_by_identity(node: &CallNode<'_>) -> bool {
    let Some(block) = node.block() else {
        return false;
    };
    if let Some(argument) = block.as_block_argument_node() {
        return argument
            .expression()
            .and_then(|expression| expression.as_symbol_node())
            .is_some_and(|symbol| symbol.unescaped() == b"itself");
    }
    block
        .as_block_node()
        .is_some_and(|block| identity_block(&block))
}

fn identity_block(block: &ruby_prism::BlockNode<'_>) -> bool {
    let (Some(parameters), Some(body)) = (block.parameters(), block.body().and_then(single_expression))
    else {
        return false;
    };
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return numbered.maximum() == 1
            && body
                .as_local_variable_read_node()
                .is_some_and(|read| read.name().as_slice() == b"_1");
    }
    if parameters.as_it_parameters_node().is_some() {
        return body.as_it_local_variable_read_node().is_some();
    }
    let Some(parameters) = parameters
        .as_block_parameters_node()
        .and_then(|parameters| parameters.parameters())
    else {
        return false;
    };
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return false;
    }
    let Some(parameter) = parameters
        .requireds()
        .first()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    body.as_local_variable_read_node()
        .is_some_and(|read| read.name().as_slice() == parameter.name().as_slice())
}

fn transform_counts(node: &CallNode<'_>) -> bool {
    let Some(block) = node.block() else {
        return false;
    };
    if let Some(argument) = block.as_block_argument_node() {
        return argument
            .expression()
            .and_then(|expression| expression.as_symbol_node())
            .is_some_and(|symbol| counting_method(symbol.unescaped()));
    }
    let Some(block) = block.as_block_node() else {
        return false;
    };
    let Some(body) = block.body().and_then(single_expression) else {
        return false;
    };
    let Some(count) = body.as_call_node() else {
        return false;
    };
    if !counting_method(call_name(&count)) || argument_count(&count) != 0 {
        return false;
    }
    block_parameter_is_receiver(&block, count.receiver())
}

fn block_parameter_is_receiver(
    block: &ruby_prism::BlockNode<'_>,
    receiver: Option<Node<'_>>,
) -> bool {
    let (Some(parameters), Some(receiver)) = (block.parameters(), receiver) else {
        return false;
    };
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return numbered.maximum() == 1
            && receiver
                .as_local_variable_read_node()
                .is_some_and(|read| read.name().as_slice() == b"_1");
    }
    if parameters.as_it_parameters_node().is_some() {
        return receiver.as_it_local_variable_read_node().is_some();
    }
    let Some(parameters) = parameters
        .as_block_parameters_node()
        .and_then(|parameters| parameters.parameters())
    else {
        return false;
    };
    if parameters.requireds().len() != 1 {
        return false;
    }
    let Some(parameter) = parameters
        .requireds()
        .first()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    receiver
        .as_local_variable_read_node()
        .is_some_and(|read| read.name().as_slice() == parameter.name().as_slice())
}

fn single_expression(node: Node<'_>) -> Option<Node<'_>> {
    let statements = node.as_statements_node()?;
    (statements.body().len() == 1)
        .then(|| statements.body().first())
        .flatten()
}

fn counting_method(name: &[u8]) -> bool {
    matches!(name, b"count" | b"size" | b"length")
}

impl ZeroLengthPredicateRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_unless!(argument_count(node) == 0 && node.receiver().is_some());
        if call_operator_is(node, b"&.") {
            self.on_csend(node);
            return;
        }
        let Some(parent) = self.parent().and_then(Node::as_call_node) else {
            return;
        };
        self.check_zero_length_predicate(node, &parent);
        self.check_zero_length_comparison(node, &parent);
        self.check_nonzero_length_comparison(node, &parent);
    }

    fn on_csend(&mut self, node: &CallNode<'_>) {
        let Some(parent) = self.parent().and_then(Node::as_call_node) else {
            return;
        };
        self.check_zero_length_predicate(node, &parent);
        self.check_zero_length_comparison(node, &parent);
    }

    fn check_zero_length_predicate(&mut self, node: &CallNode<'_>, parent: &CallNode<'_>) {
        return_unless!(
            call_name(parent) == b"zero?"
                && argument_count(parent) == 0
                && parent
                    .receiver()
                    .is_some_and(|receiver| same_call(&receiver, node))
        );
        return_if!(non_polymorphic_collection(node, self));
        let Some(offense) = self.selector_through(node, parent.location().end_offset()) else {
            return;
        };
        let current = self
            .source_file()
            .slice(offense.clone())
            .unwrap_or_default();
        let message = ZERO_MSG.replace("{current}", current);
        add_offense!(self, offense.clone(), message: message, |corrector| {
            corrector.replace(offense, "empty?");
        });
    }

    fn check_zero_length_comparison(&mut self, node: &CallNode<'_>, parent: &CallNode<'_>) {
        let Some(comparison) = length_comparison(node, parent) else {
            return;
        };
        return_unless!(comparison.zero());
        return_if!(non_polymorphic_collection(node, self));
        report_length_comparison(node, parent, comparison, false, self);
    }

    fn check_nonzero_length_comparison(&mut self, node: &CallNode<'_>, parent: &CallNode<'_>) {
        let Some(comparison) = length_comparison(node, parent) else {
            return;
        };
        return_unless!(comparison.nonzero());
        return_if!(non_polymorphic_collection(node, self));
        report_length_comparison(node, parent, comparison, true, self);
    }
}

#[derive(Clone, Copy)]
struct LengthComparison {
    length_on_left: bool,
    operator: &'static str,
    integer: i32,
}

impl LengthComparison {
    fn zero(self) -> bool {
        matches!(
            (self.length_on_left, self.operator, self.integer),
            (true, "==", 0) | (false, "==", 0) | (true, "<", 1) | (false, ">", 1)
        )
    }

    fn nonzero(self) -> bool {
        matches!(
            (self.length_on_left, self.operator, self.integer),
            (true, ">", 0) | (true, "!=", 0) | (false, "<", 0) | (false, "!=", 0)
        )
    }
}

def_node_matcher! {
    fn length_comparison(node: &CallNode<'_>, parent: &CallNode<'_>) -> Option<LengthComparison> {
        let operator = match call_name(parent) {
            b"==" => "==",
            b"!=" => "!=",
            b"<" => "<",
            b">" => ">",
            _ => return None,
        };
        let (left, right) = (parent.receiver()?, only_argument(parent)?);
        let length_on_left = same_call(&left, node);
        if !length_on_left && !same_call(&right, node) {
            return None;
        }
        let other = if length_on_left { &right } else { &left };
        Some(LengthComparison {
            length_on_left,
            operator,
            integer: integer_value(other)?,
        })
    }
}

fn report_length_comparison(
    node: &CallNode<'_>,
    parent: &CallNode<'_>,
    comparison: LengthComparison,
    nonzero: bool,
    context: &mut CopContext<'_, '_>,
) {
    let Some(replacement) = replacement(node, nonzero, context.source_file()) else {
        return;
    };
    let method = String::from_utf8_lossy(call_name(node));
    let current = if comparison.length_on_left {
        format!(
            "{method} {} {}",
            comparison.operator, comparison.integer
        )
    } else {
        format!(
            "{} {} {method}",
            comparison.integer, comparison.operator
        )
    };
    let message = if nonzero { NONZERO_MSG } else { ZERO_MSG }.replace("{current}", &current);
    add_offense!(context, parent.location(), message: message, |corrector| {
        corrector.replace(parent.location(), replacement);
    });
}

fn replacement(
    node: &CallNode<'_>,
    nonzero: bool,
    file: SourceFile<'_>,
) -> Option<String> {
    let receiver = node.receiver()?;
    let call_operator = node
        .call_operator_loc()
        .map_or(".", |location| file.at(&location));
    Some(format!(
        "{}{receiver_source}{call_operator}empty?",
        if nonzero { "!" } else { "" },
        receiver_source = file.node(&receiver),
    ))
}

fn same_call(node: &Node<'_>, expected: &CallNode<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.location().start_offset() == expected.location().start_offset()
            && call.location().end_offset() == expected.location().end_offset()
    })
}

fn integer_value(node: &Node<'_>) -> Option<i32> {
    TryInto::<i32>::try_into(node.as_integer_node()?.value()).ok()
}

def_node_matcher! {
    fn non_polymorphic_collection(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
        if call_name(node) != b"size" {
            return false;
        }
        let Some(receiver) = node.receiver() else {
            return false;
        };
        let source = context.source_file().node(&receiver);
        let source = source.strip_prefix("::").unwrap_or(source);
        source.starts_with("File.stat(")
            || ["File", "Tempfile", "StringIO"].iter().any(|constant| {
                source.starts_with(&format!("{constant}.new"))
                    || source.starts_with(&format!("{constant}.open"))
            })
    }
}
