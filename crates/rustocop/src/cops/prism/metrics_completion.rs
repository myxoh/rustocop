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
        let Some(argument) = first_argument(&call).filter(|argument| {
            argument.as_symbol_node().is_some() || argument.as_string_node().is_some()
        }) else {
            return;
        };
        let name = node_source(context.source(), &argument)
            .trim_start_matches(':')
            .trim_matches(['\'', '"'])
            .as_bytes()
            .to_vec();
        (
            name,
            block.body(),
            call.location(),
            Some(block.closing_loc().start_offset()),
            block
                .parameters()
                .map(|parameters| parameters.location().end_offset())
                .or(Some(block.opening_loc().end_offset())),
        )
    } else {
        return;
    };
    if context.policy().allows_method(&name) {
        return;
    }
    let maximum = context.config_usize("Max", 10);
    let count = body.map_or(0, |body| {
        let body = body
            .as_statements_node()
            .filter(|statements| statements.body().len() == 1)
            .and_then(|statements| statements.body().first())
            .unwrap_or(body);
        if direct_heredoc(&body) {
            return 1;
        }
        let body_location = body.location();
        let source_start = header_end
            .filter(|header_end| body_location.start_offset() <= *header_end)
            .map(|header_end| {
                let line_end = context.source_file().line_end(header_end);
                context.source()[header_end..line_end]
                    .find(';')
                    .map_or_else(
                        || {
                            context.source()[header_end..line_end]
                                .find(|character: char| !character.is_whitespace())
                                .map_or_else(
                                    || {
                                        line_end
                                            + usize::from(
                                                context.source().as_bytes().get(line_end)
                                                    == Some(&b'\n'),
                                            )
                                    },
                                    |content| header_end + content,
                                )
                        },
                        |semicolon| header_end + semicolon + 1,
                    )
            })
            .unwrap_or_else(|| body_location.start_offset());
        let mut source_end = body_end
            .map(|end| context.source_file().line_start(end))
            .filter(|end| *end > source_start)
            .unwrap_or_else(|| body_location.end_offset());
        if let Some(heredoc_end) = descendant_source_end_with_heredoc(&body) {
            source_end = heredoc_end;
        }
        let source = if source_end > source_start {
            &context.source()[source_start..source_end]
        } else {
            context.source_file().at(&body_location)
        };
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
        let message = format!("Method has too many lines. [{count}/{maximum}]");
        let definition_line =
            &context.source()[context.source_file().line_range(location.start_offset())];
        if (definition_line.contains("rubocop:disable")
            || definition_line.contains("rubocop:todo"))
            && definition_line.contains("Metrics/MethodLength")
        {
            let end = location.end_offset();
            context.replace(message, location, end..end, "");
        } else {
            context.report(message, location);
        }
    }
}

fn direct_heredoc(node: &Node<'_>) -> bool {
    node.as_string_node()
        .and_then(|string| string.opening_loc())
        .or_else(|| {
            node.as_interpolated_string_node()
                .and_then(|string| string.opening_loc())
        })
        .is_some_and(|opening| opening.as_slice().starts_with(b"<<"))
}

fn direct_heredoc_end(node: &Node<'_>) -> Option<usize> {
    node.as_string_node()
        .and_then(|string| {
            string
                .opening_loc()
                .filter(|opening| opening.as_slice().starts_with(b"<<"))
                .and_then(|_| string.closing_loc())
        })
        .or_else(|| {
            node.as_interpolated_string_node().and_then(|string| {
                string
                    .opening_loc()
                    .filter(|opening| opening.as_slice().starts_with(b"<<"))
                    .and_then(|_| string.closing_loc())
            })
        })
        .map(|closing| closing.end_offset())
}

fn descendant_source_end_with_heredoc(node: &Node<'_>) -> Option<usize> {
    struct DescendantEnd {
        first: bool,
        found_heredoc: bool,
        end: usize,
        root_block_end: Option<usize>,
    }

    impl DescendantEnd {
        fn record(&mut self, node: &Node<'_>) {
            if std::mem::replace(&mut self.first, false) {
                return;
            }
            // Prism models `else` and statement lists as structural wrapper
            // nodes. Parser/RuboCop does not expose those wrappers as AST
            // descendants, so their ranges must not extend the metric body.
            let prism_only_wrapper = node.as_else_node().is_some()
                || node.as_statements_node().is_some()
                || node.as_rescue_node().is_some()
                || node.as_ensure_node().is_some()
                || node
                    .as_begin_node()
                    .is_some_and(|begin| begin.begin_keyword_loc().is_none())
                || node.as_block_node().is_some_and(|block| {
                    self.root_block_end == Some(block.location().end_offset())
                })
                || node.as_if_node().is_some_and(|conditional| {
                    conditional
                        .if_keyword_loc()
                        .is_some_and(|keyword| keyword.as_slice() == b"elsif")
                });
            if !prism_only_wrapper {
                self.end = self.end.max(node.location().end_offset());
            }
            let closing = node
                .as_string_node()
                .and_then(|string| {
                    string
                        .opening_loc()
                        .filter(|opening| opening.as_slice().starts_with(b"<<"))
                        .and_then(|_| string.closing_loc())
                })
                .or_else(|| {
                    node.as_interpolated_string_node().and_then(|string| {
                        string
                            .opening_loc()
                            .filter(|opening| opening.as_slice().starts_with(b"<<"))
                            .and_then(|_| string.closing_loc())
                    })
                });
            if let Some(closing) = closing {
                self.found_heredoc = true;
                self.end = self.end.max(closing.end_offset());
            }
        }
    }

    impl<'pr> Visit<'pr> for DescendantEnd {
        fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
            self.record(&node);
        }

        fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
            self.record(&node);
        }
    }

    let mut finder = DescendantEnd {
        first: true,
        found_heredoc: false,
        end: node.location().start_offset(),
        root_block_end: node
            .as_call_node()
            .and_then(|call| call.block())
            .map(|block| block.location().end_offset()),
    };
    finder.visit(node);
    finder.found_heredoc.then_some(finder.end)
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
        let body = body
            .as_statements_node()
            .filter(|statements| statements.body().len() == 1)
            .and_then(|statements| statements.body().first())
            .unwrap_or(body);
        let location = body.location();
        let implicit_begin = body
            .as_begin_node()
            .filter(|begin| begin.begin_keyword_loc().is_none());
        let start = implicit_begin
            .as_ref()
            .and_then(|begin| begin.statements())
            .map_or_else(|| location.start_offset(), |statements| statements.location().start_offset());
        let end = direct_heredoc_end(&body)
            .or_else(|| descendant_source_end_with_heredoc(&body))
            .unwrap_or_else(|| {
                implicit_begin.map_or_else(
                    || location.end_offset(),
                    |_| node.closing_loc().start_offset(),
                )
            });
        let source = &context.source()[start..end];
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
    let message = format!(
        "Assignment Branch Condition size for `{name}` is too high. [<{assignments}, {branches}, {conditions}> {score}/{maximum}]"
    );
    let definition_line =
        &context.source()[context.source_file().line_range(location.start_offset())];
    if (definition_line.contains("rubocop:disable")
        || definition_line.contains("rubocop:todo"))
        && definition_line.contains("Metrics/AbcSize")
    {
        let end = location.end_offset();
        context.replace(message, location, end..end, "");
    } else {
        context.report(message, location);
    }
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
        if node.block().is_some_and(|block| abc_rubocop_counted_block(&block))
            && abc_iterating_method(name)
        {
            self.condition();
        }
    }

    #[allow(clippy::too_many_lines)] // Exhaustive Prism node dispatch is clearer in one source-shaped match.
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
            if conditional.if_keyword_loc().is_some()
                && conditional.subsequent().is_some_and(|branch| branch.as_else_node().is_some())
            {
                self.condition();
            }
        } else if node.as_unless_node().is_some()
            || node.as_while_node().is_some_and(|loop_node| !loop_node.is_begin_modifier())
            || node.as_until_node().is_some_and(|loop_node| !loop_node.is_begin_modifier())
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

    fn visit_in_node(&mut self, node: &ruby_prism::InNode<'pr>) {
        self.condition();
        let pattern = node.pattern();
        if let Some(guard) = pattern.as_if_node() {
            self.visit(&guard.predicate());
            if let Some(statements) = guard.statements() {
                self.visit_statements_node(&statements);
            }
        } else if let Some(guard) = pattern.as_unless_node() {
            self.visit(&guard.predicate());
            if let Some(statements) = guard.statements() {
                self.visit_statements_node(&statements);
            }
        } else {
            self.visit(&pattern);
        }
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        }
    }
}

fn abc_rubocop_counted_block(node: &Node<'_>) -> bool {
    let Some(block) = node.as_block_node() else {
        return node.as_block_argument_node().is_some();
    };
    block.parameters().is_none_or(|parameters| {
        parameters.as_numbered_parameters_node().is_none()
            && parameters.as_it_parameters_node().is_none()
    })
}

fn abc_iterating_method(name: &[u8]) -> bool {
    matches!(name,
        b"all?"|b"any?"|b"chain"|b"chunk"|b"chunk_while"|b"collect"|b"collect_concat"|b"count"|b"cycle"|
        b"detect"|b"drop"|b"drop_while"|b"each"|b"each_cons"|b"each_entry"|b"each_slice"|b"each_with_index"|
        b"each_with_object"|b"entries"|b"filter"|b"filter_map"|b"find"|b"find_all"|b"find_index"|b"flat_map"|
        b"grep"|b"grep_v"|b"group_by"|b"inject"|b"lazy"|b"map"|b"max"|b"max_by"|b"min"|b"min_by"|b"minmax"|
        b"minmax_by"|b"none?"|b"one?"|b"partition"|b"reduce"|b"reject"|b"reverse_each"|b"select"|b"slice_after"|
        b"slice_before"|b"slice_when"|b"sort"|b"sort_by"|b"sum"|b"take"|b"take_while"|b"tally"|b"to_h"|b"uniq"|
        b"zip"|b"with_index"|b"with_object"|b"bsearch"|b"bsearch_index"|b"collect!"|b"combination"|b"d_permutation"|
        b"delete_if"|b"each_index"|b"keep_if"|b"map!"|b"permutation"|b"product"|b"reject!"|b"repeat"|
        b"repeated_combination"|b"select!"|b"sort!"|b"each_key"|b"each_pair"|b"each_value"|b"fetch"|
        b"fetch_values"|b"has_key?"|b"merge"|b"merge!"|b"transform_keys"|b"transform_keys!"|
        b"transform_values"|b"transform_values!"
    )
}
