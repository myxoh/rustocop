use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(BooleanSymbol),
        Box::new(EmptyExpression),
        Box::new(FlipFlop),
        Box::new(FloatComparison),
        Box::new(FloatOutOfRange),
        Box::new(IdentityComparison),
        Box::new(RegexpAsCondition),
        Box::new(SelfAssignment),
        Box::new(ToJson),
    ]
}

define_any_node_cop!(RegexpAsCondition => "Lint/RegexpAsCondition" => regexp_as_condition);

fn regexp_as_condition(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if (node.as_match_last_line_node().is_none()
        && node.as_interpolated_match_last_line_node().is_none())
        || !context.ancestors().iter().any(conditional_node)
    {
        return;
    }

    context.insert_after(
        node,
        "Do not use regexp literal as a condition. The regexp literal matches `$_` implicitly.",
        " =~ $_",
    );
}

fn conditional_node(node: &Node<'_>) -> bool {
    node.as_if_node().is_some()
        || node.as_unless_node().is_some()
        || node.as_while_node().is_some()
        || node.as_until_node().is_some()
        || node.as_for_node().is_some()
}

struct FloatOutOfRange;

impl Cop for FloatOutOfRange {
    fn name(&self) -> &'static str {
        "Lint/FloatOutOfRange"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(float) = node.as_float_node() else {
            return;
        };
        let location = float.location();
        let literal = source_at(source, &location);
        let nonzero_mantissa = literal
            .split(['e', 'E'])
            .next()
            .is_some_and(|mantissa| mantissa.bytes().any(|byte| matches!(byte, b'1'..=b'9')));
        if !(float.value().is_infinite() || float.value() == 0.0 && nonzero_mantissa) {
            return;
        }

        context.report(self.name(), "Float out of range.", location);
    }
}

struct IdentityComparison;

impl Cop for IdentityComparison {
    fn name(&self) -> &'static str {
        "Lint/IdentityComparison"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(comparison) = node.as_call_node() else {
            return;
        };
        let operator = call_name(&comparison);
        if !matches!(operator, b"==" | b"!=") {
            return;
        }
        let Some(left) = comparison.receiver().and_then(|node| node.as_call_node()) else {
            return;
        };
        let Some(right) = first_argument(&comparison).and_then(|node| node.as_call_node()) else {
            return;
        };
        if call_name(&left) != b"object_id" || call_name(&right) != b"object_id" {
            return;
        }
        let (Some(left_receiver), Some(right_receiver)) = (left.receiver(), right.receiver())
        else {
            return;
        };
        let bang = if operator == b"!=" { "!" } else { "" };
        let operator = String::from_utf8_lossy(operator);
        let location = comparison.location();
        context.replace(
            self.name(),
            format!("Use `{bang}equal?` instead of `{operator}` when comparing `object_id`."),
            &location,
            &location,
            format!(
                "{bang}{}.equal?({})",
                source_at(source, &left_receiver.location()),
                source_at(source, &right_receiver.location())
            ),
        );
    }
}

struct BooleanSymbol;

impl Cop for BooleanSymbol {
    fn name(&self) -> &'static str {
        "Lint/BooleanSymbol"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(symbol) = node.as_symbol_node() else {
            return;
        };
        let value = symbol.unescaped();
        if !matches!(value, b"true" | b"false") {
            return;
        }

        let location = symbol.location();
        let raw = source_at(source, &location);
        if raw.as_bytes() == value {
            return;
        }
        let value = String::from_utf8_lossy(value);
        let replacement = if raw.ends_with(':') {
            format!("{value} =>")
        } else {
            value.to_string()
        };
        let edit = location.start_offset()..location.end_offset();
        let offense = if raw.ends_with(':') {
            symbol.value_loc().map_or_else(
                || edit.clone(),
                |value| value.start_offset()..value.end_offset(),
            )
        } else {
            edit.clone()
        };
        context.replace(
            self.name(),
            format!("Symbol with a boolean name - you probably meant to use `{value}`."),
            offense,
            edit,
            replacement,
        );
    }
}

struct EmptyExpression;

impl Cop for EmptyExpression {
    fn name(&self) -> &'static str {
        "Lint/EmptyExpression"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(parentheses) = node.as_parentheses_node() else {
            return;
        };
        if parentheses.body().is_none() {
            context.report(
                self.name(),
                "Avoid empty expressions.",
                parentheses.location(),
            );
        }
    }
}

struct FlipFlop;

impl Cop for FlipFlop {
    fn name(&self) -> &'static str {
        "Lint/FlipFlop"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(flip_flop) = node.as_flip_flop_node() else {
            return;
        };
        context.report(
            self.name(),
            "Avoid the use of flip-flop operators.",
            flip_flop.location(),
        );
    }
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
        } else if let Some(write) = node.as_constant_write_node() {
            Some((write.name_loc(), write.value(), write.location()))
        } else {
            None
        };
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
    for (offset, line) in SourceFile::new(source).lines() {
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

struct ToJson;

impl Cop for ToJson {
    fn name(&self) -> &'static str {
        "Lint/ToJSON"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(definition) = node.as_def_node() else {
            return;
        };
        if definition.name().as_slice() != b"to_json" || definition.parameters().is_some() {
            return;
        }

        let name = definition.name_loc();
        let location = definition.location();
        context.insert(
            self.name(),
            "`#to_json` requires an optional argument to be parsable via JSON.generate(obj).",
            location,
            name.end_offset(),
            "(*_args)",
        );
    }
}
