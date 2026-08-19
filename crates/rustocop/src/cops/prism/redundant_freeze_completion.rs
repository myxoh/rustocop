use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![Box::new(RedundantFreeze), Box::new(StringChars)]
}

struct RedundantFreeze;

impl Cop for RedundantFreeze {
    fn name(&self) -> &'static str {
        "Style/RedundantFreeze"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let Some(receiver) = call.receiver() else {
            return;
        };
        if call_name(&call) != b"freeze"
            || !redundant_freeze_receiver(
                &receiver,
                &context.cop_context(self.name(), source, _ancestors),
            )
        {
            return;
        }
        let location = call.location();
        context.replace(
            self.name(),
            "Do not freeze immutable objects, as freezing them has no effect.",
            &location,
            &location,
            node_source(source, &receiver),
        );
    }
}

fn redundant_freeze_receiver(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses
            .body()
            .as_ref()
            .is_some_and(|body| redundant_freeze_receiver(body, context));
    }
    if let Some(statements) = node.as_statements_node() {
        return statements
            .body()
            .first()
            .is_some_and(|body| redundant_freeze_receiver(&body, context));
    }
    if immutable_literal(node) {
        if (node.as_regular_expression_node().is_some() || node.as_range_node().is_some())
            && !context.target_ruby_version().at_least(3, 0)
        {
            return false;
        }
        return true;
    }
    if node.as_string_node().is_some() {
        if context.source().contains("# frozen_string_literal: false") {
            return false;
        }
        return context.source().contains("# frozen_string_literal: true")
            || context.related_config_value("AllCops", "StringLiteralsFrozenByDefault")
                == Some("true");
    }
    if node.as_interpolated_string_node().is_some() {
        return !context.target_ruby_version().at_least(3, 0)
            && context.source().contains("# frozen_string_literal: true");
    }
    let Some(call) = node.as_call_node() else {
        return false;
    };
    if matches!(call_name(&call), b"count" | b"length" | b"size") {
        return true;
    }
    if matches!(
        call_name(&call),
        b">" | b">=" | b"<" | b"<=" | b"==" | b"!=" | b"<=>"
    ) {
        return true;
    }
    matches!(call_name(&call), b"+" | b"-" | b"*" | b"/" | b"%" | b"**")
        && call.receiver().is_some_and(|receiver| {
            receiver.as_integer_node().is_some() || receiver.as_float_node().is_some()
        })
        && first_argument(&call).is_some_and(|argument| {
            argument.as_integer_node().is_some() || argument.as_float_node().is_some()
        })
}

struct StringChars;

impl Cop for StringChars {
    fn name(&self) -> &'static str {
        "Style/StringChars"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let Some(argument) = first_argument(&call) else {
            return;
        };
        let argument_source = node_source(source, &argument);
        if call_name(&call) != b"split"
            || !matches!(argument_source, "''" | "\"\"" | "//")
            || call
                .arguments()
                .is_none_or(|arguments| arguments.arguments().len() != 1)
        {
            return;
        }
        let Some(selector) = call.message_loc() else {
            return;
        };
        let end = call.closing_loc().map_or_else(
            || argument.location().end_offset(),
            |closing| closing.end_offset(),
        );
        let current = &source[selector.start_offset()..end];
        context.replace(
            self.name(),
            format!("Use `chars` instead of `{current}`."),
            (selector.start_offset(), end),
            (selector.start_offset(), end),
            "chars",
        );
    }
}
