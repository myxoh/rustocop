use super::*;
use std::collections::HashSet;

define_cops! {
    MethodLength => "Metrics/MethodLength" => any_node(method_length),
    BlockLength => "Metrics/BlockLength" => node(as_block_node, block_length),
    AbcSize => "Metrics/AbcSize" => any_node(abc_size),
}

fn method_length(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (name, body, location, body_end, header_end) = if let Some(node) = node.as_def_node() {
        (
            node.name().as_slice().to_vec(),
            node.body(),
            node.location(),
            node.end_keyword_loc().map(|end| end.start_offset()),
            node.rparen_loc()
                .map(|closing| closing.end_offset())
                .or_else(|| node.parameters().map(|parameters| parameters.location().end_offset()))
                .or(Some(node.name_loc().end_offset())),
        )
    } else if let Some(block) = node.as_block_node() {
        let Some(call) = context
            .nearest_call()
            .filter(|call| call_name(call) == b"define_method")
        else {
            return;
        };
        let name = first_argument(&call)
            .map(|argument| {
                node_source(context.source(), &argument)
                    .trim_start_matches(':')
                    .as_bytes()
                    .to_vec()
            })
            .unwrap_or_default();
        (
            name,
            block.body(),
            call.location(),
            Some(block.closing_loc().start_offset()),
            None,
        )
    } else {
        return;
    };
    if context.policy().allows_method(&name) {
        return;
    }
    let definition_line = &context.source()[context.source_file().line_range(location.start_offset())];
    if (definition_line.contains("rubocop:disable") || definition_line.contains("rubocop:todo"))
        && definition_line.contains("Metrics/MethodLength")
    {
        return;
    }
    let maximum = context.config_usize("Max", 10);
    let count = body.map_or(0, |body| {
        let body_location = body.location();
        let source_start = header_end
            .filter(|header_end| body_location.start_offset() <= *header_end)
            .map(|header_end| {
                let line_end = context.source_file().line_end(header_end);
                context.source()[header_end..line_end]
                    .find(';')
                    .map_or_else(
                        || {
                            line_end
                                + usize::from(
                                    context.source().as_bytes().get(line_end) == Some(&b'\n'),
                                )
                        },
                        |semicolon| header_end + semicolon + 1,
                    )
            })
            .unwrap_or_else(|| body_location.start_offset());
        let source_end = body_end
            .map(|end| context.source_file().line_start(end))
            .filter(|end| *end > source_start)
            .unwrap_or_else(|| body_location.end_offset());
        let source = if source_end > source_start {
            &context.source()[source_start..source_end]
        } else {
            context.source_file().at(&body_location)
        };
        let mut count = code_lines(source, false, context.config_bool("CountComments", false));
        if source.lines().any(|line| line.contains("<<~") || line.contains("<<-"))
            && source.lines().rev().find(|line| !line.trim().is_empty()).map(str::trim)
                == Some("end")
        {
            count = count.saturating_sub(1);
        }
        if context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "array")
        {
            count = count.saturating_sub(folded_extra_lines(source, '[', ']'));
        }
        if context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "hash")
        {
            count = count.saturating_sub(folded_extra_lines(source, '{', '}'));
        }
        count
    });
    if count > maximum {
        context.report(
            format!("Method has too many lines. [{count}/{maximum}]"),
            location,
        );
    }
}

fn block_length(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let owning_call = context.nearest_call();
    if owning_call.as_ref().is_some_and(|call| {
        matches!(call_name(call), b"define_method" | b"new")
            && (call_name(call) != b"new"
                || root_constant(call.receiver(), b"Class")
                || root_constant(call.receiver(), b"Module")
                || root_constant(call.receiver(), b"Struct")
                || root_constant(call.receiver(), b"Data"))
    }) {
        return;
    }
    if owning_call.as_ref().is_some_and(|call| {
        let method = String::from_utf8_lossy(call_name(call));
        let full_name = block_method_name(call, context.source());
        context.policy().allows_method(call_name(call))
            || ["AllowedMethods", "IgnoredMethods", "ExcludedMethods"]
                .iter()
                .flat_map(|key| context.config_values(key))
                .any(|allowed| allowed == method.as_ref() || allowed == &full_name)
            || ["AllowedPatterns", "IgnoredMethods"]
                .iter()
                .flat_map(|key| context.config_values(key))
                .any(|pattern| {
                    let pattern = pattern.trim_matches(['^', '$']);
                    method.contains(pattern) || full_name.contains(pattern)
                })
    }) {
        return;
    }
    let maximum = context.config_usize("Max", 25);
    let count = node.body().map_or(0, |body| {
        let source = context.source_file().at(&body.location());
        let mut count = code_lines(source, false, context.config_bool("CountComments", false));
        if context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "array")
        {
            count = count.saturating_sub(folded_extra_lines(source, '[', ']'));
        }
        if context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "hash")
        {
            count = count.saturating_sub(folded_extra_lines(source, '{', '}'));
        }
        count
    });
    if count > maximum {
        let offense = owning_call.map_or_else(
            || node.location().start_offset()..node.location().end_offset(),
            |call| call.location().start_offset()..call.location().end_offset(),
        );
        context.report(
            format!("Block has too many lines. [{count}/{maximum}]"),
            offense,
        );
    }
}

fn block_method_name(call: &CallNode<'_>, source: &str) -> String {
    let Some(message) = call.message_loc() else {
        return String::from_utf8_lossy(call_name(call)).into_owned();
    };
    let start = call.receiver().map_or_else(
        || message.start_offset(),
        |receiver| receiver.location().start_offset(),
    );
    source[start..message.end_offset()]
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn code_lines(source: &str, exclude_edges: bool, count_comments: bool) -> usize {
    let lines = source.lines().collect::<Vec<_>>();
    let slice = if exclude_edges && lines.len() >= 2 {
        &lines[1..lines.len() - 1]
    } else {
        lines.as_slice()
    };
    slice
        .iter()
        .filter(|line| {
            let line = line.trim();
            !line.is_empty() && (count_comments || !line.starts_with('#'))
        })
        .count()
}

fn folded_extra_lines(source: &str, open: char, close: char) -> usize {
    let Some(start) = source.find(open) else {
        return 0;
    };
    let Some(end_relative) = source[start..].find(close) else {
        return 0;
    };
    source[start..start + end_relative].matches('\n').count()
}

fn abc_size(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (name, body, location) = if let Some(definition) = node.as_def_node() {
        (
            definition.name().as_slice().to_vec(),
            definition.body(),
            definition.location(),
        )
    } else if let Some(block) = node.as_block_node() {
        let Some(call) = context
            .nearest_call()
            .filter(|call| call_name(call) == b"define_method")
        else {
            return;
        };
        let name = first_argument(&call)
            .map(|argument| {
                node_source(context.source(), &argument)
                    .trim_start_matches(':')
                    .as_bytes()
                    .to_vec()
            })
            .unwrap_or_default();
        (name, block.body(), call.location())
    } else {
        return;
    };
    if context.policy().allows_method(&name) {
        return;
    }
    let definition_line = &context.source()[context.source_file().line_range(location.start_offset())];
    if (definition_line.contains("rubocop:disable") || definition_line.contains("rubocop:todo"))
        && definition_line.contains("Metrics/AbcSize")
    {
        return;
    }
    let Some(body) = body else {
        return;
    };
    let mut counter = AbcCounter {
        source: context.source(),
        count_repeated_attributes: context.config_bool("CountRepeatedAttributes", true),
        ..AbcCounter::default()
    };
    counter.visit(&body);
    let assignments = counter.assignments;
    let branches = counter.branches;
    let conditions = counter.conditions;
    let score =
        ((assignments * assignments + branches * branches + conditions * conditions) as f64).sqrt();
    let maximum = context
        .config_value("Max")
        .and_then(|value| value.parse::<f64>().ok())
        .unwrap_or(17.0);
    if score <= maximum {
        return;
    }
    let name = String::from_utf8_lossy(&name);
    let score = metric_number(score);
    let maximum = metric_number(maximum);
    context.report(
        format!(
            "Assignment Branch Condition size for `{name}` is too high. [<{assignments}, {branches}, {conditions}> {score}/{maximum}]"
        ),
        location,
    );
}

fn metric_number(value: f64) -> String {
    let integer_digits = if value < 1.0 {
        1
    } else {
        value.log10().floor() as usize + 1
    };
    let precision = 4_usize.saturating_sub(integer_digits).min(2);
    let formatted = format!("{value:.precision$}");
    if formatted.contains('.') {
        formatted
            .trim_end_matches('0')
            .trim_end_matches('.')
            .to_string()
    } else {
        formatted
    }
}

#[derive(Default)]
struct AbcCounter<'source> {
    source: &'source str,
    assignments: usize,
    branches: usize,
    conditions: usize,
    count_repeated_attributes: bool,
    attributes: HashSet<String>,
    safe_navigation_receivers: HashSet<Vec<u8>>,
}

impl AbcCounter<'_> {
    fn assignment(&mut self) {
        self.assignments += 1;
    }

    fn condition(&mut self) {
        self.conditions += 1;
    }

    fn call(&mut self, node: &CallNode<'_>) {
        let name = node.name().as_slice();
        if matches!(name, b"==" | b"!=" | b"<=" | b">=" | b"<" | b">" | b"===" | b"=~" | b"!~") {
            self.condition();
            return;
        }
        if name.ends_with(b"=") {
            self.assignment();
        }
        if node.call_operator_loc().is_some_and(|operator| operator.as_slice() == b"&.") {
            let repeated = node.receiver().and_then(|receiver| {
                receiver
                    .as_local_variable_read_node()
                    .map(|receiver| !self.safe_navigation_receivers.insert(receiver.name().as_slice().to_vec()))
            });
            if repeated != Some(true) {
                self.condition();
            }
        }
        if !self.count_repeated_attributes && node.arguments().is_none() && node.block().is_none() {
            let attribute = String::from_utf8_lossy(name).into_owned();
            if !self.attributes.insert(attribute) {
                return;
            }
        }
        self.branches += 1;
        if node.block().is_some()
            && matches!(
                name,
                b"all?"
                    | b"any?"
                    | b"collect"
                    | b"collect!"
                    | b"count"
                    | b"detect"
                    | b"delete_if"
                    | b"drop_while"
                    | b"each"
                    | b"each_cons"
                    | b"each_entry"
                    | b"each_index"
                    | b"each_key"
                    | b"each_pair"
                    | b"each_slice"
                    | b"each_value"
                    | b"each_with_index"
                    | b"each_with_object"
                    | b"filter_map"
                    | b"find_all"
                    | b"find_index"
                    | b"flat_map"
                    | b"group_by"
                    | b"keep_if"
                    | b"with_index"
                    | b"with_object"
                    | b"map"
                    | b"map!"
                    | b"select"
                    | b"select!"
                    | b"reject"
                    | b"reject!"
                    | b"filter"
                    | b"find"
                    | b"reduce"
                    | b"inject"
                    | b"none?"
                    | b"one?"
                    | b"partition"
                    | b"reverse_each"
                    | b"sort_by"
                    | b"sum"
                    | b"take_while"
                    | b"transform_keys"
                    | b"transform_keys!"
                    | b"transform_values"
                    | b"transform_values!"
                    | b"times"
                    | b"upto"
                    | b"downto"
            )
        {
            self.condition();
        }
    }

    fn count_node(&mut self, node: &Node<'_>) {
        if let Some(call) = node.as_call_node() {
            self.call(&call);
            return;
        }
        if node.as_yield_node().is_some() {
            self.branches += 1;
        }
        if let Some(conditional) = node.as_if_node() {
            self.condition();
            if conditional.subsequent().is_some_and(|branch| branch.as_else_node().is_some()) {
                self.condition();
            }
        } else if node.as_unless_node().is_some()
            || node.as_while_node().is_some()
            || node.as_until_node().is_some()
            || node.as_for_node().is_some()
            || node.as_rescue_node().is_some()
            || node.as_rescue_modifier_node().is_some()
            || node.as_when_node().is_some()
            || node.as_in_node().is_some()
            || node.as_and_node().is_some()
            || node.as_or_node().is_some()
            || node.as_flip_flop_node().is_some()
        {
            self.condition();
        }
        if node.as_case_node().is_some_and(|case| case.else_clause().is_some()) {
            self.condition();
        }
        if node.as_for_node().is_some() {
            self.assignment();
        }
        if let Some(parameters) = node.as_block_parameters_node() {
            let location = parameters.location();
            let source = &self.source[location.start_offset()..location.end_offset()];
            self.assignments += source
                .trim_matches(['|', ' '])
                .split([',', ';'])
                .map(str::trim)
                .filter(|parameter| {
                    !parameter.is_empty()
                        && !parameter.trim_start_matches(['*', '&']).starts_with('_')
                })
                .count();
        }

        let local_assignment = node
            .as_local_variable_write_node()
            .map(|write| write.name().as_slice().to_vec())
            .or_else(|| node.as_local_variable_and_write_node().map(|write| write.name().as_slice().to_vec()))
            .or_else(|| node.as_local_variable_or_write_node().map(|write| write.name().as_slice().to_vec()))
            .or_else(|| node.as_local_variable_operator_write_node().map(|write| write.name().as_slice().to_vec()));
        if let Some(name) = local_assignment {
            if !name.starts_with(b"_") {
                self.assignment();
                self.safe_navigation_receivers.remove(&name);
                self.attributes.clear();
            }
        } else if node.as_instance_variable_write_node().is_some()
            || node.as_instance_variable_and_write_node().is_some()
            || node.as_instance_variable_or_write_node().is_some()
            || node.as_instance_variable_operator_write_node().is_some()
            || node.as_class_variable_write_node().is_some()
            || node.as_class_variable_and_write_node().is_some()
            || node.as_class_variable_or_write_node().is_some()
            || node.as_class_variable_operator_write_node().is_some()
            || node.as_global_variable_write_node().is_some()
            || node.as_global_variable_and_write_node().is_some()
            || node.as_global_variable_or_write_node().is_some()
            || node.as_global_variable_operator_write_node().is_some()
            || node.as_constant_write_node().is_some()
            || node.as_constant_and_write_node().is_some()
            || node.as_constant_or_write_node().is_some()
            || node.as_constant_operator_write_node().is_some()
            || node.as_constant_path_write_node().is_some()
            || node.as_constant_path_and_write_node().is_some()
            || node.as_constant_path_or_write_node().is_some()
            || node.as_constant_path_operator_write_node().is_some()
        {
            self.assignment();
        }
        if node.as_local_variable_and_write_node().is_some()
            || node.as_local_variable_or_write_node().is_some()
            || node.as_local_variable_operator_write_node().is_some()
            || node.as_instance_variable_and_write_node().is_some()
            || node.as_instance_variable_or_write_node().is_some()
            || node.as_instance_variable_operator_write_node().is_some()
            || node.as_class_variable_and_write_node().is_some()
            || node.as_class_variable_or_write_node().is_some()
            || node.as_class_variable_operator_write_node().is_some()
            || node.as_global_variable_and_write_node().is_some()
            || node.as_global_variable_or_write_node().is_some()
            || node.as_global_variable_operator_write_node().is_some()
            || node.as_constant_and_write_node().is_some()
            || node.as_constant_or_write_node().is_some()
            || node.as_constant_operator_write_node().is_some()
            || node.as_constant_path_and_write_node().is_some()
            || node.as_constant_path_or_write_node().is_some()
            || node.as_constant_path_operator_write_node().is_some()
        {
            self.assignment();
        }
        if node.as_call_and_write_node().is_some()
            || node.as_call_or_write_node().is_some()
            || node.as_index_and_write_node().is_some()
            || node.as_index_or_write_node().is_some()
        {
            self.assignment();
            self.branches += 1;
        }
        if let Some(write) = node.as_call_operator_write_node() {
            self.assignment();
            self.branches += 1;
            if write.value().as_call_node().is_some() {
                self.assignment();
            }
        }
        if let Some(write) = node.as_index_operator_write_node() {
            self.assignment();
            self.branches += 1;
            if write.value().as_call_node().is_some() {
                self.assignment();
            }
        }
        if node.as_local_variable_and_write_node().is_some()
            || node.as_local_variable_or_write_node().is_some()
            || node.as_instance_variable_and_write_node().is_some()
            || node.as_instance_variable_or_write_node().is_some()
            || node.as_class_variable_and_write_node().is_some()
            || node.as_class_variable_or_write_node().is_some()
            || node.as_global_variable_and_write_node().is_some()
            || node.as_global_variable_or_write_node().is_some()
            || node.as_constant_and_write_node().is_some()
            || node.as_constant_or_write_node().is_some()
            || node.as_constant_path_and_write_node().is_some()
            || node.as_constant_path_or_write_node().is_some()
            || node.as_call_and_write_node().is_some()
            || node.as_call_or_write_node().is_some()
            || node.as_index_and_write_node().is_some()
            || node.as_index_or_write_node().is_some()
        {
            self.condition();
        }
    }
}

impl<'pr> Visit<'pr> for AbcCounter<'_> {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        self.count_node(&node);
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        self.count_node(&node);
    }

    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode<'pr>) {
        self.condition();
        if node.reference().is_some_and(|reference| {
            reference
                .as_local_variable_target_node()
                .is_some_and(|target| !target.name().as_slice().starts_with(b"_"))
        }) {
            self.assignment();
        }
        ruby_prism::visit_rescue_node(self, node);
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
}
