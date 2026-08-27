use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(FloatComparison),
        Box::new(SelfAssignment),
    ]
}


struct FloatComparison;

impl Cop for FloatComparison {
    fn name(&self) -> &'static str {
        "Lint/FloatComparison"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        if let Some(branch) = node.as_when_node() {
            for condition in branch.conditions().iter() {
                if float_expression(Some(&condition)) && !literal_zero(Some(&condition)) {
                    context.report(
                        self.name(),
                        "Avoid float literal comparisons in case statements as they are unreliable.",
                        condition.location(),
                    );
                }
            }
        } else if let Some(call) = node.as_call_node() {
            self.on_call(&call, context);
        }
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        let method = call_name(node);
        if !matches!(method, b"==" | b"!=" | b"eql?" | b"equal?") {
            return;
        }
        let Some(argument) = first_argument(node) else {
            return;
        };
        if node
            .arguments()
            .is_none_or(|arguments| arguments.arguments().len() != 1)
            || node
                .receiver()
                .as_ref()
                .is_some_and(|receiver| receiver.as_nil_node().is_some())
            || argument.as_nil_node().is_some()
            || literal_zero(node.receiver().as_ref())
            || literal_zero(Some(&argument))
            || (!float_expression(node.receiver().as_ref()) && !float_expression(Some(&argument)))
        {
            return;
        }

        let message = if method == b"!=" {
            "Avoid inequality comparisons of floats as they are unreliable."
        } else {
            "Avoid equality comparisons of floats as they are unreliable."
        };
        context.report(self.name(), message, node.location());
    }
}

struct SelfAssignment;

impl Cop for SelfAssignment {
    fn name(&self) -> &'static str {
        "Lint/SelfAssignment"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        if node.as_program_node().is_some() {
            report_compound_self_assignments(source, context, self.name());
            return;
        }
        let allow_rbs_annotation = {
            let cop_context = context.cop_context(self.name(), source, &[]);
            cop_context.config_bool("AllowRBSInlineAnnotation", false)
        };
        if allow_rbs_annotation {
            let line = SourceFile::new(source).line(node.location().start_offset());
            if line.contains("#:") {
                return;
            }
        }
        let simple = if let Some(write) = node.as_local_variable_write_node() {
            Some((write.name_loc(), write.value(), write.location()))
        } else if let Some(write) = node.as_instance_variable_write_node() {
            Some((write.name_loc(), write.value(), write.location()))
        } else if let Some(write) = node.as_class_variable_write_node() {
            Some((write.name_loc(), write.value(), write.location()))
        } else if let Some(write) = node.as_global_variable_write_node() {
            Some((write.name_loc(), write.value(), write.location()))
        } else { node.as_constant_write_node().map(|write| (write.name_loc(), write.value(), write.location())) };
        if let Some((name, value, location)) = simple {
            if source_at(source, &name) == source_at(source, &value.location()) {
                context.report(self.name(), "Self-assignment detected.", location);
            }
            return;
        }

        let Some(call) = node.as_call_node() else {
            return;
        };
        let Some(receiver) = call.receiver() else {
            return;
        };
        let arguments = call
            .arguments()
            .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let self_assignment = if call_name(&call) == b"[]=" && arguments.len() >= 2 {
            let Some(value_call) = arguments.last().and_then(Node::as_call_node) else {
                return;
            };
            let keys = &arguments[..arguments.len() - 1];
            let value_keys = value_call
                .arguments()
                .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
                .unwrap_or_default();
            call_name(&value_call) == b"[]"
                && same_node_source(source, &receiver, &value_call.receiver())
                && keys.len() == value_keys.len()
                && keys.iter().all(|key| key.as_call_node().is_none())
                && keys.iter().zip(value_keys.iter()).all(|(left, right)| {
                    source_at(source, &left.location()) == source_at(source, &right.location())
                })
        } else if call_name(&call).ends_with(b"=") && arguments.len() == 1 {
            let Some(value_call) = arguments[0].as_call_node() else {
                return;
            };
            let assigned_name = &call_name(&call)[..call_name(&call).len() - 1];
            call_name(&value_call) == assigned_name
                && argument_count(&value_call) == 0
                && same_node_source(source, &receiver, &value_call.receiver())
        } else {
            false
        };
        if self_assignment {
            context.report(self.name(), "Self-assignment detected.", call.location());
        }
    }
}

fn report_compound_self_assignments(source: &str, context: &mut Context, cop: &'static str) {
    let allow_rbs_annotation = {
        let cop_context = context.cop_context(cop, source, &[]);
        cop_context.config_bool("AllowRBSInlineAnnotation", false)
    };
    let file = SourceFile::new(source);
    let mut non_code_ranges = file.literal_ranges();
    non_code_ranges.extend(file.heredoc_ranges());
    non_code_ranges.extend(file.comment_ranges());
    for (offset, line) in file.lines() {
        let code_start = offset + line.len() - line.trim_start().len();
        if non_code_ranges
            .iter()
            .any(|range| range.start <= code_start && code_start < range.end)
        {
            continue;
        }
        if allow_rbs_annotation && line.contains("#:") {
            continue;
        }
        let code = line.split('#').next().unwrap_or_default().trim_end();
        let self_assignment = [" ||= ", " &&= "].iter().any(|operator| {
            code.split_once(operator)
                .is_some_and(|(left, right)| {
                    !left.contains('[') && left.trim() == right.trim()
                })
        }) || code.split_once(" = ").is_some_and(|(left, right)| {
            left.contains(',')
                && left.trim() == right.trim().trim_start_matches('[').trim_end_matches(']')
        });
        if self_assignment {
            context.report(
                cop,
                "Self-assignment detected.",
                offset..offset + code.len(),
            );
        }
    }
}

fn same_node_source(source: &str, left: &Node<'_>, right: &Option<Node<'_>>) -> bool {
    right.as_ref().is_some_and(|right| {
        source_at(source, &left.location()) == source_at(source, &right.location())
    })
}
