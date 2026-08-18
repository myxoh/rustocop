use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(BooleanSymbol),
        Box::new(EmptyExpression),
        Box::new(FlipFlop),
        Box::new(FloatComparison),
        Box::new(IdentityComparison),
        Box::new(SelfAssignment),
        Box::new(ToJson),
    ]
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
        context.add_offense_offsets(
            self.name(),
            format!("Use `{bang}equal?` instead of `{operator}` when comparing `object_id`."),
            location.start_offset(),
            location.end_offset(),
            Some((
                location.start_offset(),
                location.end_offset(),
                format!(
                    "{bang}{}.equal?({})",
                    source_at(source, &left_receiver.location()),
                    source_at(source, &right_receiver.location())
                ),
            )),
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
        _source: &str,
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
        context.add_offense_offsets(
            self.name(),
            format!(
                "Symbol with a boolean name - you probably meant to use `{}`.",
                String::from_utf8_lossy(value)
            ),
            location.start_offset(),
            location.end_offset(),
            Some((
                location.start_offset(),
                location.end_offset(),
                String::from_utf8_lossy(value).into_owned(),
            )),
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
            context.add_offense(
                self.name(),
                "Avoid empty expressions.".to_string(),
                parentheses.location(),
                None,
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
        context.add_offense(
            self.name(),
            "Avoid the use of flip-flop operators.".to_string(),
            flip_flop.location(),
            None,
        );
    }
}

struct FloatComparison;

impl Cop for FloatComparison {
    fn name(&self) -> &'static str {
        "Lint/FloatComparison"
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
        context.add_offense(self.name(), message.to_string(), node.location(), None);
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
        let Some(write) = node.as_local_variable_write_node() else {
            return;
        };
        let value = write.value();
        if source_at(source, &write.name_loc()) == source_at(source, &value.location()) {
            context.add_offense(
                self.name(),
                "Self-assignment detected.".to_string(),
                write.location(),
                None,
            );
        }
    }
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
        context.add_offense_offsets(
            self.name(),
            "`#to_json` requires an optional argument to be parsable via JSON.generate(obj)."
                .to_string(),
            location.start_offset(),
            location.end_offset(),
            Some((name.end_offset(), name.end_offset(), "(*_args)".to_string())),
        );
    }
}
