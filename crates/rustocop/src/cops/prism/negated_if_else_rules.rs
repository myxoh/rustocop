use ruby_prism::{IfNode, Node};

use super::*;

#[derive(Default)]
struct NegatedIfElseState {
    corrected: Vec<std::ops::Range<usize>>,
}

define_cops! {
    NegatedIfElseCondition => "Style/NegatedIfElseCondition" => compatibility_prism_stateful_callbacks(
        NegatedIfElseConditionRule,
        NegatedIfElseState,
        [on_if]
    ),
}

impl NegatedIfElseConditionRule<'_, '_, '_> {
    fn on_if(&mut self, node: &IfNode<'_>) {
        return_if!(node
            .if_keyword_loc()
            .is_some_and(|keyword| keyword.as_slice() == b"elsif"));
        let Some(else_node) = node.subsequent().and_then(|branch| branch.as_else_node()) else {
            return;
        };
        return_if!(else_node.statements().is_none());
        let condition = unwrap_condition(node.predicate());
        let Some(negation) = negated_condition(&condition) else {
            return;
        };
        return_if!(negation.double_negation || negation.argument_count >= 2);

        let ternary = node.if_keyword_loc().is_none();
        let kind = if ternary { "ternary" } else { "if-else" };
        let message = format!("Invert the negated condition and swap the {kind} branches.");
        let offense = node.location().start_offset()..node.location().end_offset();
        let corrected_ancestor = self
            .state
            .corrected
            .iter()
            .any(|ancestor| ancestor.start <= offense.start && offense.end <= ancestor.end);

        let replacement = negation.replacement(self.source_file());
        let if_empty = node
            .statements()
            .is_none_or(|statements| statements.body().is_empty());
        let if_statement = only_statement(node.statements());
        let else_statement = only_statement(else_node.statements());
        let condition_location = condition.location();
        let condition_range = condition_location.start_offset()..condition_location.end_offset();
        let source = self.source().to_string();
        let empty_else_range = if if_empty {
            let keyword = else_node.else_keyword_loc();
            let line_start = self.source_file().line_start(keyword.start_offset());
            let line_end = self.source_file().line_end(keyword.end_offset());
            let remove_end = if source[line_end..].starts_with('\n') {
                line_end + 1
            } else {
                line_end
            };
            Some(line_start..remove_end)
        } else {
            None
        };
        let ternary_ranges = ternary.then(|| {
            let left = if_statement.as_ref().expect("checked").location();
            let right = else_statement.as_ref().expect("else branch").location();
            (
                left.start_offset()..left.end_offset(),
                right.start_offset()..right.end_offset(),
            )
        });
        let block_ranges = (!ternary).then(|| {
            let end_keyword = node.end_keyword_loc().expect("block conditional");
            (
                node.predicate().location().end_offset()
                    ..else_node.else_keyword_loc().start_offset(),
                else_node.else_keyword_loc().end_offset()..end_keyword.start_offset(),
            )
        });
        let block_replacements = block_ranges.as_ref().map(|(left, right)| {
            (
                corrected_branch_source(
                    right,
                    else_statement.as_ref(),
                    &source,
                    self.source_file(),
                ),
                corrected_branch_source(left, if_statement.as_ref(), &source, self.source_file()),
            )
        });
        let correction_spec = NegatedIfElseCorrection {
            condition: condition_range,
            replacement,
            empty_else: empty_else_range,
            ternary: ternary_ranges,
            block: block_ranges,
            block_replacements,
            source: &source,
        };
        if corrected_ancestor {
            let mut correction = CorrectionPlan::default();
            correction_spec.apply(&mut correction);
            self.apply_correction_indirectly(message, offense, correction);
            return;
        }
        add_offense!(self, offense.clone(), message: message, |corrector| {
            correction_spec.apply(corrector);
        });
        self.state.corrected.push(offense);
    }
}

struct NegatedIfElseCorrection<'a> {
    condition: std::ops::Range<usize>,
    replacement: String,
    empty_else: Option<std::ops::Range<usize>>,
    ternary: Option<(std::ops::Range<usize>, std::ops::Range<usize>)>,
    block: Option<(std::ops::Range<usize>, std::ops::Range<usize>)>,
    block_replacements: Option<(String, String)>,
    source: &'a str,
}

impl NegatedIfElseCorrection<'_> {
    fn apply(self, corrector: &mut CorrectionPlan) {
        corrector.replace(self.condition, self.replacement);
        if let Some(range) = self.empty_else {
            corrector.remove(range);
        } else if let Some((left, right)) = self.ternary {
            corrector.swap(self.source, left, right);
        } else if let (Some((left, right)), Some((left_replacement, right_replacement))) =
            (self.block, self.block_replacements)
        {
            corrector.replace(left, left_replacement);
            corrector.replace(right, right_replacement);
        }
    }
}

fn corrected_branch_source(
    range: &std::ops::Range<usize>,
    statement: Option<&Node<'_>>,
    source: &str,
    file: SourceFile<'_>,
) -> String {
    let Some(statement) = statement else {
        return source[range.clone()].to_string();
    };
    let Some(condition_node) = statement.as_if_node() else {
        return source[range.clone()].to_string();
    };
    let Some(else_node) = condition_node
        .subsequent()
        .and_then(|branch| branch.as_else_node())
    else {
        return source[range.clone()].to_string();
    };
    let condition = unwrap_condition(condition_node.predicate());
    let Some(negation) = negated_condition(&condition) else {
        return source[range.clone()].to_string();
    };
    let Some(end_keyword) = condition_node.end_keyword_loc() else {
        return source[range.clone()].to_string();
    };
    let condition_range = condition.location().start_offset()..condition.location().end_offset();
    let left = condition_node.predicate().location().end_offset()
        ..else_node.else_keyword_loc().start_offset();
    let right = else_node.else_keyword_loc().end_offset()..end_keyword.start_offset();
    let edits = vec![
        (condition_range, negation.replacement(file)),
        (left.clone(), source[right.clone()].to_string()),
        (right, source[left].to_string()),
    ];
    apply_local_edits(source[range.clone()].to_string(), range.start, edits)
}

fn apply_local_edits(
    mut source: String,
    base: usize,
    mut edits: Vec<(std::ops::Range<usize>, String)>,
) -> String {
    edits.sort_by_key(|(range, _)| range.start);
    for (range, replacement) in edits.into_iter().rev() {
        source.replace_range(range.start - base..range.end - base, &replacement);
    }
    source
}

struct Negation<'pr> {
    receiver: Node<'pr>,
    argument: Option<Node<'pr>>,
    method: Vec<u8>,
    argument_count: usize,
    double_negation: bool,
}

impl Negation<'_> {
    fn replacement(&self, file: SourceFile<'_>) -> String {
        if self.method == b"!" {
            return file.node(&self.receiver).to_string();
        }
        let operator = if self.method == b"!=" { "==" } else { "=~" };
        format!(
            "{} {operator} {}",
            file.node(&self.receiver),
            file.node(self.argument.as_ref().expect("binary negation"))
        )
    }
}

def_node_matcher! {
    fn negated_condition<'pr>(node: &Node<'pr>) -> Option<Negation<'pr>> {
        let call = node.as_call_node()?;
        let method = call.name().as_slice();
        if !matches!(method, b"!" | b"!=" | b"!~") {
            return None;
        }
        let receiver = call.receiver()?;
        let arguments = arguments(&call);
        let argument_count = arguments.len();
        let double_negation = method == b"!"
            && receiver.as_call_node().is_some_and(|inner| inner.name().as_slice() == b"!");
        Some(Negation {
            receiver,
            argument: arguments.into_iter().next(),
            method: method.to_vec(),
            argument_count,
            double_negation,
        })
    }
}

fn unwrap_condition(mut node: Node<'_>) -> Node<'_> {
    loop {
        if let Some(parentheses) = node.as_parentheses_node() {
            let Some(inner) = parentheses.body().and_then(single_expression) else {
                return node;
            };
            node = inner;
        } else if let Some(begin) = node.as_begin_node() {
            let Some(inner) = only_statement(begin.statements()) else {
                return node;
            };
            node = inner;
        } else {
            return node;
        }
    }
}
