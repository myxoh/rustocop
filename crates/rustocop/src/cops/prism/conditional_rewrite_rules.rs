use ruby_prism::Node;

use super::*;

define_rule!(RedundantConditionalRule);
define_rule!(ParenthesesAroundConditionRule);
define_rule!(RedundantConditionRule);

define_cops! {
    ParenthesesAroundCondition => "Style/ParenthesesAroundCondition" => compatibility_prism_node_rule_aliases(
        ParenthesesAroundConditionRule,
        on_conditional => [as_if_node, as_unless_node, as_while_node, as_until_node]
    ),
    RedundantCondition => "Style/RedundantCondition" => compatibility_prism_node_rule_aliases(
        RedundantConditionRule,
        on_if => [as_if_node, as_unless_node]
    ),
}

impl RedundantConditionalRule<'_, '_, '_> {
    fn on_if(&mut self, node: &Node<'_>) {
        let (location, keyword, condition, statements, subsequent, unless, ternary) =
            if let Some(condition) = node.as_if_node() {
                let ternary = condition.if_keyword_loc().is_none();
                let keyword = condition.if_keyword_loc().or_else(|| condition.then_keyword_loc());
                (condition.location(), keyword, condition.predicate(), condition.statements(), condition.subsequent(), false, ternary)
            } else if let Some(condition) = node.as_unless_node() {
                (condition.location(), Some(condition.keyword_loc()), condition.predicate(), condition.statements(), condition.else_clause().map(|node| node.as_node()), true, false)
            } else {
                return;
            };
        let Some(keyword) = keyword else { return };
        return_if!(!ternary && keyword.start_offset() != location.start_offset());
        return_unless!(comparison_condition(&condition));
        let Some(if_branch) = only_statement(statements) else { return };
        let Some(else_branch) = subsequent
            .and_then(|branch| branch.as_else_node())
            .and_then(|branch| only_statement(branch.statements())) else { return };
        let branches_inverted = if_branch.as_false_node().is_some() && else_branch.as_true_node().is_some();
        return_unless!((if_branch.as_true_node().is_some() && else_branch.as_false_node().is_some()) || branches_inverted);
        let inverted = branches_inverted ^ unless;

        let condition = self.source_file().node(&condition);
        let expression = if inverted { format!("!({condition})") } else { condition.to_string() };
        let elsif = keyword.as_slice() == b"elsif";
        let replacement = if elsif {
            format!("else\n{}  {}", self.source_file().indentation_text(keyword.start_offset()), expression)
        } else {
            expression.clone()
        };
        let display = if elsif { format!("\n{replacement}") } else { expression };
        let message = format!("This conditional expression can just be replaced by `{display}`.");
        let offense = if elsif { location.start_offset()..else_branch.location().end_offset() } else { location.start_offset()..location.end_offset() };
        add_offense!(self, offense.clone(), message: message, |corrector| {
            corrector.replace(offense, replacement);
        });
    }
}

fn comparison_condition(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        matches!(
            call.name().as_slice(),
            b"==" | b"!=" | b"===" | b"<" | b">" | b"<=" | b">="
        )
    })
}

impl ParenthesesAroundConditionRule<'_, '_, '_> {
    fn on_conditional(&mut self, node: &Node<'_>) {
        let Some((keyword, predicate, ternary, loop_condition)) = conditional_parts(node) else { return };
        return_if!(ternary);
        let Some(parentheses) = predicate.as_parentheses_node() else { return };
        return_if!(keyword.end_offset() == parentheses.opening_loc().start_offset());
        let Some(body) = parentheses.body().and_then(single_expression) else { return };
        return_if!(parentheses.is_multiple_statements());
        return_if!(modifier_expression(&body));
        return_if!(safe_assignment(&body) && self.config_bool("AllowSafeAssignment", true));
        return_if!(self.source_file().node(&predicate).contains('\n') && self.config_bool("AllowInMultilineConditions", false));
        return_if!(loop_condition && do_end_block(&body, self.source_file()));

        let keyword_name = String::from_utf8_lossy(keyword.as_slice());
        let article = if keyword_name == "while" { "a" } else { "an" };
        let message = format!("Don't use parentheses around the condition of {article} `{keyword_name}`.");
        let replacement = self.source_file().node(&body).to_string();
        add_offense!(self, parentheses.location(), message: message, |corrector| {
            corrector.replace(parentheses.location(), replacement);
        });
    }
}

impl RedundantConditionRule<'_, '_, '_> {
    fn on_if(&mut self, node: &Node<'_>) {
        if let Some(condition) = node.as_unless_node() {
            self.on_unless(&condition);
            return;
        }
        let Some(condition) = node.as_if_node() else { return };
        let location = condition.location();
        let ternary = condition.if_keyword_loc().is_none();
        if let Some(keyword) = condition.if_keyword_loc() {
            return_if!(keyword.as_slice() == b"elsif" || keyword.start_offset() != location.start_offset());
        }
        let Some(if_branch) = only_statement(condition.statements()) else { return };
        let else_node = condition.subsequent().and_then(|branch| branch.as_else_node());
        let else_branch = else_node.as_ref().and_then(|branch| only_statement(branch.statements()));
        return_if!(condition.subsequent().is_some() && else_node.is_none());
        return_if!(else_node.is_some() && else_branch.is_none());
        return_if!(else_branch.as_ref().is_some_and(|branch| branch.as_if_node().is_some()));
        return_if!(else_branch.as_ref().is_some_and(|branch| branch.as_index_operator_write_node().is_some()));
        return_if!(else_branch.as_ref().is_some_and(|branch| {
            branch
                .as_call_node()
                .is_some_and(|call| call.name().as_slice() == b"[]=")
        }));

        let condition_source = self.source_file().node(&condition.predicate());
        let if_source = self.source_file().node(&if_branch);
        let simple = condition_source == if_source;
        let true_predicate = if_branch.as_true_node().is_some()
            && else_branch.is_some()
            && predicate_call(&condition.predicate(), self);
        let assignment = else_branch.as_ref().and_then(|else_branch| {
            matching_assignments(&if_branch, else_branch, condition_source, self.source_file())
        });
        let method = else_branch.as_ref().and_then(|else_branch| {
            matching_method_calls(&if_branch, else_branch, condition_source, self.source_file())
        });
        return_unless!(simple || true_predicate || assignment.is_some() || method.is_some());

        let redundant = else_branch.is_none();
        let message = if redundant { "This condition is not needed." } else { "Use double pipes `||` instead." };
        let comments = self.source_file().node(node).contains('#');
        let offense = redundant_condition_offense(&condition, ternary, method.is_some());
        if comments {
            self.report(message, offense);
            return;
        }
        let replacement = if redundant {
            if_source.to_string()
        } else {
            let else_branch = else_branch.as_ref().expect("checked above");
            let left = if true_predicate { render_predicate(&condition.predicate(), self.source_file()) } else { condition_source.to_string() };
            if let Some((target, else_value)) = assignment {
                format!("{target} = {left} || {}", render_or_operand(&else_value, self.source_file(), false))
            } else if let Some(ref method) = method {
                render_matching_method(method, &left, self.source_file())
            } else {
                format!("{left} || {}", render_or_operand(else_branch, self.source_file(), false))
            }
        };
        let replacement = if !ternary && semantic_conditional_parent(self.ancestors()).is_some_and(|parent| parent.as_call_node().is_some()) {
            format!("({replacement})")
        } else { replacement };
        if ternary && method.is_none() && !redundant {
            let question = condition.then_keyword_loc().expect("ternary");
            let colon = else_node.expect("else").else_keyword_loc();
            let edit = question.start_offset()..colon.end_offset();
            let wrap_else = else_branch.as_ref().is_some_and(|branch| branch.as_range_node().is_some());
            add_offense!(self, offense, message: message, |corrector| {
                corrector.replace(edit, "||");
                if wrap_else {
                    let branch = else_branch.as_ref().expect("checked");
                    corrector.replace(branch.location().start_offset()..branch.location().start_offset(), "(");
                    corrector.replace(branch.location().end_offset()..branch.location().end_offset(), ")");
                }
            });
        } else {
            let edit = location.start_offset()..location.end_offset();
            add_offense!(self, offense, message: message, |corrector| {
                corrector.replace(edit, replacement);
            });
        }
    }

    fn on_unless(&mut self, condition: &ruby_prism::UnlessNode<'_>) {
        return_if!(condition.keyword_loc().start_offset() != condition.location().start_offset());
        let Some(if_branch) = only_statement(condition.statements()) else { return };
        let Some(else_node) = condition.else_clause() else { return };
        let Some(else_branch) = only_statement(else_node.statements()) else { return };
        let predicate = self.source_file().node(&condition.predicate());
        return_unless!(predicate == self.source_file().node(&else_branch));
        let replacement = format!("{predicate} || {}", render_or_operand(&if_branch, self.source_file(), false));
        let offense = condition.location().start_offset()..condition.location().end_offset();
        if self.source_file().node(&condition.as_node()).contains('#') {
            self.report("Use double pipes `||` instead.", offense);
        } else {
            add_offense!(self, offense.clone(), message: "Use double pipes `||` instead.", |corrector| {
                corrector.replace(offense, replacement);
            });
        }
    }
}

fn predicate_call(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    let Some(call) = node.as_call_node() else { return false };
    if call.block().is_some() { return false; }
    let name = String::from_utf8_lossy(call.name().as_slice());
    if !name.ends_with('?') { return false; }
    let allowed = context.config_values("AllowedMethods");
    if allowed.is_empty() { !matches!(name.as_ref(), "infinite?" | "nonzero?") } else { !allowed.iter().any(|method| method == name.as_ref()) }
}

fn redundant_condition_offense(condition: &ruby_prism::IfNode<'_>, ternary: bool, branches_have_method: bool) -> std::ops::Range<usize> {
    if ternary && !branches_have_method {
        let question = condition.then_keyword_loc().expect("ternary");
        let colon = condition.subsequent().and_then(|branch| branch.as_else_node()).expect("ternary else").else_keyword_loc();
        question.start_offset()..colon.end_offset()
    } else {
        condition.location().start_offset()..condition.location().end_offset()
    }
}

fn assignment_parts<'pr>(node: &Node<'pr>, file: SourceFile<'_>) -> Option<(String, Node<'pr>)> {
    let (name, value) = if let Some(write) = node.as_local_variable_write_node() {
        (file.at(&write.name_loc()).to_string(), write.value())
    } else if let Some(write) = node.as_instance_variable_write_node() {
        (file.at(&write.name_loc()).to_string(), write.value())
    } else if let Some(write) = node.as_class_variable_write_node() {
        (file.at(&write.name_loc()).to_string(), write.value())
    } else if let Some(write) = node.as_global_variable_write_node() {
        (file.at(&write.name_loc()).to_string(), write.value())
    } else if let Some(write) = node.as_constant_write_node() {
        (file.at(&write.name_loc()).to_string(), write.value())
    } else { return None };
    Some((name, value))
}

fn matching_assignments<'pr>(left: &Node<'pr>, right: &Node<'pr>, condition: &str, file: SourceFile<'_>) -> Option<(String, Node<'pr>)> {
    let (left_target, left_value) = assignment_parts(left, file)?;
    let (right_target, right_value) = assignment_parts(right, file)?;
    (left_target == right_target && file.node(&left_value) == condition).then_some((left_target, right_value))
}

struct MatchingMethod<'pr> {
    left_source: String,
    left_start: usize,
    left_argument_start: usize,
    left_argument_end: usize,
    right_argument: Node<'pr>,
    operator: bool,
}

fn matching_method_calls<'pr>(left: &Node<'pr>, right: &Node<'pr>, condition: &str, file: SourceFile<'_>) -> Option<MatchingMethod<'pr>> {
    let left_call = left.as_call_node()?;
    let right_call = right.as_call_node()?;
    if left_call.name().as_slice() == b"[]" || left_call.name().as_slice() != right_call.name().as_slice() { return None; }
    let left_receiver = left_call.receiver().map(|node| file.node(&node).to_string());
    let right_receiver = right_call.receiver().map(|node| file.node(&node).to_string());
    if left_receiver != right_receiver { return None; }
    let left_argument = only_argument(&left_call)?;
    let right_argument = only_argument(&right_call)?;
    if invalid_redundant_condition_argument(&left_argument)
        || invalid_redundant_condition_argument(&right_argument)
    {
        return None;
    }
    (file.node(&left_argument) == condition).then(|| MatchingMethod {
        left_source: file.node(left).to_string(),
        left_start: left.location().start_offset(),
        left_argument_start: left_argument.location().start_offset(),
        left_argument_end: left_argument.location().end_offset(),
        right_argument,
        operator: matches!(left_call.name().as_slice(), b"+" | b"-" | b"*" | b"/" | b"%" | b"**"),
    })
}

fn invalid_redundant_condition_argument(node: &Node<'_>) -> bool {
    node.as_splat_node().is_some()
        || node.as_assoc_splat_node().is_some()
        || node.as_forwarding_arguments_node().is_some()
        || node.as_block_argument_node().is_some()
        || node.as_keyword_hash_node().is_some_and(|hash| {
            hash.elements()
                .iter()
                .next()
                .is_some_and(|element| element.as_assoc_splat_node().is_some())
        })
}

fn render_matching_method(method: &MatchingMethod<'_>, left: &str, file: SourceFile<'_>) -> String {
    let source = &method.left_source;
    let relative_start = method.left_argument_start - method.left_start;
    let relative_end = method.left_argument_end - method.left_start;
    let combined = format!("{left} || {}", render_or_operand(&method.right_argument, file, true));
    if method.operator {
        format!("{}({combined}){}", &source[..relative_start], &source[relative_end..])
    } else {
        format!("{}{combined}{}", &source[..relative_start], &source[relative_end..])
    }
}

fn render_predicate(node: &Node<'_>, file: SourceFile<'_>) -> String {
    let Some(call) = node.as_call_node() else { return file.node(node).to_string() };
    if call.opening_loc().is_some() || call.arguments().is_none() { return file.node(node).to_string(); }
    let arguments = call.arguments().expect("checked");
    let first = arguments.arguments().first().expect("predicate has arguments");
    let prefix = &file.node(node)[..first.location().start_offset() - node.location().start_offset()];
    let args = file.at(&arguments.location());
    format!("{}({})", prefix.trim_end(), args)
}

fn render_or_operand(node: &Node<'_>, file: SourceFile<'_>, method_argument: bool) -> String {
    let source = file.node(node);
    if node.as_range_node().is_some() || node.as_rescue_modifier_node().is_some()
        || node.as_and_node().is_some() || node.as_or_node().is_some()
        || node.as_if_node().is_some() || node.as_unless_node().is_some()
        || node.as_while_node().is_some() || node.as_until_node().is_some()
    {
        return format!("({source})");
    }
    if method_argument && node.as_keyword_hash_node().is_some() && !source.trim_start().starts_with('{') {
        return format!("{{ {source} }}");
    }
    if !method_argument {
        if let Some(call) = node.as_call_node() {
            if call.opening_loc().is_none() && call.arguments().is_some_and(|arguments| !arguments.arguments().is_empty())
                && !matches!(call.name().as_slice(), b"+" | b"-" | b"*" | b"/" | b"%" | b"**")
            {
                let arguments = call.arguments().expect("checked");
                let first = arguments.arguments().first().expect("nonempty");
                let prefix = &source[..first.location().start_offset() - node.location().start_offset()];
                return format!("{}({})", prefix.trim_end(), file.at(&arguments.location()));
            }
        }
    }
    source.to_string()
}

fn semantic_conditional_parent<'a, 'pr>(ancestors: &'a [Node<'pr>]) -> Option<&'a Node<'pr>> {
    ancestors.iter().rev().find(|node| node.as_statements_node().is_none() && node.as_else_node().is_none())
}

fn conditional_parts<'pr>(node: &Node<'pr>) -> Option<(ruby_prism::Location<'pr>, Node<'pr>, bool, bool)> {
    if let Some(condition) = node.as_if_node() {
        let keyword = condition.if_keyword_loc()?;
        let ternary = keyword.as_slice() == b"?";
        return Some((keyword, condition.predicate(), ternary, false));
    }
    if let Some(condition) = node.as_unless_node() {
        return Some((condition.keyword_loc(), condition.predicate(), false, false));
    }
    if let Some(condition) = node.as_while_node() {
        return Some((condition.keyword_loc(), condition.predicate(), false, true));
    }
    let condition = node.as_until_node()?;
    Some((condition.keyword_loc(), condition.predicate(), false, true))
}

fn modifier_expression(node: &Node<'_>) -> bool {
    ModifierConditional::from_node(node).is_some() || node.as_rescue_modifier_node().is_some()
}

fn safe_assignment(node: &Node<'_>) -> bool {
    node.as_multi_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_constant_write_node().is_some()
        || node.as_call_operator_write_node().is_some()
        || node.as_index_operator_write_node().is_some()
        || node.as_call_node().is_some_and(|call| {
            call.name().as_slice().ends_with(b"=")
                && !matches!(call.name().as_slice(), b"==" | b"!=" | b"===" | b"<=" | b">=" | b"<=>" | b"=~" | b"!~")
        })
}

fn do_end_block(node: &Node<'_>, file: SourceFile<'_>) -> bool {
    node.as_call_node().is_some_and(|call| call.block().is_some() && file.node(node).split_whitespace().any(|part| part == "do"))
}
