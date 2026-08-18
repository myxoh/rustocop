use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(CharacterLiteral),
        Box::new(BeginBlock),
        Box::new(DefWithParentheses),
        Box::new(MethodCallWithoutArgsParentheses),
        Box::new(NilComparison),
        Box::new(NotKeyword),
        Box::new(RedundantArrayConstructor),
        Box::new(RedundantFreeze),
        Box::new(StringChars),
        Box::new(StringMethods),
    ]
}

struct CharacterLiteral;

impl Cop for CharacterLiteral {
    fn name(&self) -> &'static str {
        "Style/CharacterLiteral"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(string) = node.as_string_node() else {
            return;
        };
        let location = string.location();
        let text = source_at(source, &location);
        if !text.starts_with('?') || !(2..=3).contains(&text.len()) {
            return;
        }
        let content = &text[1..];
        let replacement = if content.len() == 1 && content != "'" {
            format!("'{content}'")
        } else {
            format!("\"{content}\"")
        };
        context.replace(
            self.name(),
            "Do not use the character literal - use string literal instead.",
            &location,
            &location,
            replacement,
        );
    }
}

struct DefWithParentheses;

impl Cop for DefWithParentheses {
    fn name(&self) -> &'static str {
        "Style/DefWithParentheses"
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
        let (Some(open), Some(close)) = (definition.lparen_loc(), definition.rparen_loc()) else {
            return;
        };
        if definition.parameters().is_some() || definition.operator_loc().is_some() {
            return;
        }
        context.remove(
            self.name(),
            "Omit the parentheses in defs when the method doesn't accept any arguments.",
            (open.start_offset(), close.end_offset()),
            (open.start_offset(), close.end_offset()),
        );
    }
}

struct MethodCallWithoutArgsParentheses;

impl Cop for MethodCallWithoutArgsParentheses {
    fn name(&self) -> &'static str {
        "Style/MethodCallWithoutArgsParentheses"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        let (Some(open), Some(close)) = (node.opening_loc(), node.closing_loc()) else {
            return;
        };
        if node.arguments().is_some() || call_name(node).first().is_some_and(u8::is_ascii_uppercase)
        {
            return;
        }
        context.remove(
            self.name(),
            "Do not use parentheses for method calls with no arguments.",
            (open.start_offset(), close.end_offset()),
            (open.start_offset(), close.end_offset()),
        );
    }
}

struct NilComparison;

impl Cop for NilComparison {
    fn name(&self) -> &'static str {
        "Style/NilComparison"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        let Some(receiver) = node.receiver() else {
            return;
        };
        if !matches!(call_name(node), b"==" | b"===")
            || first_argument(node).is_none_or(|argument| argument.as_nil_node().is_none())
        {
            return;
        }
        let Some(selector) = node.message_loc() else {
            return;
        };
        context.replace(
            self.name(),
            "Prefer the use of the `nil?` predicate.",
            selector,
            (
                receiver.location().end_offset(),
                node.location().end_offset(),
            ),
            ".nil?",
        );
    }
}

struct NotKeyword;

impl Cop for NotKeyword {
    fn name(&self) -> &'static str {
        "Style/Not"
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
        let Some(selector) = call.message_loc() else {
            return;
        };
        if call_name(&call) == b"!" && selector.as_slice() == b"not" {
            let receiver_source = call
                .receiver()
                .map(|receiver| source_at(source, &receiver.location()))
                .unwrap_or_default();
            let call_source = source_at(source, &call.location());
            let replacement = if call_source.starts_with("not(") {
                format!("!({})", receiver_source)
            } else if let Some((left, right)) = receiver_source.split_once(" < ") {
                format!("{} >= {}", left, right)
            } else if [" >> ", " && ", " || ", " ? "]
                .iter()
                .any(|operator| receiver_source.contains(operator))
            {
                format!("!({})", receiver_source)
            } else {
                format!("!{}", receiver_source)
            };
            let location = call.location();
            context.replace(
                self.name(),
                "Use `!` instead of `not`.",
                selector,
                location,
                replacement,
            );
        }
    }
}

struct RedundantArrayConstructor;

impl Cop for RedundantArrayConstructor {
    fn name(&self) -> &'static str {
        "Style/RedundantArrayConstructor"
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
        if call.receiver().is_some()
            || call_name(&call) != b"Array"
            || argument.as_array_node().is_none()
            || call
                .arguments()
                .is_none_or(|arguments| arguments.arguments().len() != 1)
        {
            return;
        }
        let Some(selector) = call.message_loc() else {
            return;
        };
        let node_location = call.location();
        context.replace(
            self.name(),
            "Remove the redundant `Array` constructor.",
            selector,
            node_location,
            source_at(source, &argument.location()),
        );
    }
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
        if call_name(&call) != b"freeze" || !immutable_literal(&receiver) {
            return;
        }
        let location = call.location();
        context.replace(
            self.name(),
            "Do not freeze immutable objects, as freezing them has no effect.",
            &location,
            &location,
            source_at(source, &receiver.location()),
        );
    }
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
        let argument_source = source_at(source, &argument.location());
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
        let end = call.location().end_offset();
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

struct BeginBlock;

impl Cop for BeginBlock {
    fn name(&self) -> &'static str {
        "Style/BeginBlock"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(pre_execution) = node.as_pre_execution_node() else {
            return;
        };
        context.report(
            self.name(),
            "Avoid the use of `BEGIN` blocks.",
            pre_execution.keyword_loc(),
        );
    }
}

struct StringMethods;

impl Cop for StringMethods {
    fn name(&self) -> &'static str {
        "Style/StringMethods"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if !CallMatcher::new(node)
            .named(b"intern")
            .without_arguments()
            .matches()
        {
            return;
        }
        let Some(selector) = node.message_loc() else {
            return;
        };
        context.replace(
            self.name(),
            "Prefer `to_sym` over `intern`.",
            &selector,
            &selector,
            "to_sym",
        );
    }
}
