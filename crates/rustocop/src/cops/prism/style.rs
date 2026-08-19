use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(CharacterLiteral),
        Box::new(BeginBlock),
        Box::new(MethodCallWithoutArgsParentheses),
        Box::new(NilComparison),
        Box::new(NotKeyword),
        Box::new(RedundantArrayConstructor),
        Box::new(StringMethods),
    ]
}

define_node_cop!(CharacterLiteral => "Style/CharacterLiteral" => as_string_node => character_literal);

fn character_literal(string: &ruby_prism::StringNode<'_>, context: &mut CopContext<'_, '_>) {
    let location = string.location();
    let text = context.source_file().at(&location);
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
        "Do not use the character literal - use string literal instead.",
        &location,
        &location,
        replacement,
    );
}

struct MethodCallWithoutArgsParentheses;

impl Cop for MethodCallWithoutArgsParentheses {
    fn name(&self) -> &'static str {
        "Style/MethodCallWithoutArgsParentheses"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(node) = node.as_call_node() else {
            return;
        };
        let mut context = context.cop_context(self.name(), source, ancestors);
        let (Some(open), Some(close)) = (node.opening_loc(), node.closing_loc()) else {
            return;
        };
        if context
            .source()
            .as_bytes()
            .get(open.start_offset().saturating_sub(1))
            == Some(&b'.')
        {
            return;
        }
        if context.source()[..open.start_offset()]
            .trim_end()
            .ends_with("not")
        {
            return;
        }
        let name = call_name(&node);
        if node.arguments().is_some()
            || name.is_empty()
            || name.first().is_some_and(u8::is_ascii_uppercase)
            || context.policy().allows_method(name)
        {
            return;
        }
        if name == b"it"
            && node.receiver().is_none()
            && context.ancestors().iter().any(|ancestor| {
                ancestor
                    .as_block_node()
                    .is_some_and(|block| block.parameters().is_none())
            })
        {
            return;
        }
        if node.receiver().is_none() {
            let line_start = context
                .source_file()
                .line_start(node.location().start_offset());
            let before = &context.source()[line_start..node.location().start_offset()];
            let name = String::from_utf8_lossy(name);
            if before.trim_end().ends_with(&format!("{name} ="))
                || before.trim_end().ends_with(&format!("{name} ||="))
            {
                return;
            }
            if before.split_once('=').is_some_and(|(assigned, _)| {
                assigned.split(',').any(|variable| variable.trim() == name)
            }) {
                return;
            }
            if context.source()[..node.location().start_offset()]
                .lines()
                .any(|line| line.trim_start().starts_with(&format!("{name} =")))
            {
                return;
            }
        }
        context.remove(
            "Do not use parentheses for method calls with no arguments.",
            open.start_offset()..close.end_offset(),
            open.start_offset()..close.end_offset(),
        );
    }
}

struct NilComparison;

impl Cop for NilComparison {
    fn name(&self) -> &'static str {
        "Style/NilComparison"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(node) = node.as_call_node() else {
            return;
        };
        let mut context = context.cop_context(self.name(), source, ancestors);
        let Some(receiver) = node.receiver() else {
            return;
        };
        if context.policy().enforced_style("predicate") == "comparison" {
            if call_name(&node) != b"nil?" || argument_count(&node) != 0 {
                return;
            }
            let Some(selector) = node.message_loc() else {
                return;
            };
            let comparison = format!("{} == nil", context.source_file().node(&receiver));
            if let Some(parent) = context
                .parent()
                .and_then(Node::as_call_node)
                .filter(|parent| {
                    call_name(parent) == b"!"
                        && parent.receiver().is_some_and(|parent_receiver| {
                            parent_receiver.location().start_offset()
                                == node.location().start_offset()
                                && parent_receiver.location().end_offset()
                                    == node.location().end_offset()
                        })
                })
            {
                context.replace(
                    "Prefer the use of the `==` comparison.",
                    selector,
                    parent.location(),
                    format!("!({comparison})"),
                );
            } else {
                context.replace(
                    "Prefer the use of the `==` comparison.",
                    selector,
                    node.location(),
                    comparison,
                );
            }
        } else {
            if !matches!(call_name(&node), b"==" | b"===")
                || first_argument(&node).is_none_or(|argument| argument.as_nil_node().is_none())
            {
                return;
            }
            let Some(selector) = node.message_loc() else {
                return;
            };
            context.replace(
                "Prefer the use of the `nil?` predicate.",
                selector,
                receiver.location().end_offset()..node.location().end_offset(),
                ".nil?",
            );
        }
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
                .map(|receiver| node_source(source, &receiver))
                .unwrap_or_default();
            let call_source = node_source(source, &call.as_node());
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
        let arguments = call
            .arguments()
            .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let (offense, replacement) = if call.receiver().is_none()
            && call_name(&call) == b"Array"
            && arguments.len() == 1
            && arguments[0].as_array_node().is_some()
        {
            let Some(selector) = call.message_loc() else {
                return;
            };
            (
                selector.start_offset()..selector.end_offset(),
                node_source(source, &arguments[0]).to_string(),
            )
        } else if call_name(&call) == b"new"
            && root_constant(call.receiver(), b"Array")
            && arguments.len() == 1
            && arguments[0].as_array_node().is_some()
            && call.block().is_none()
        {
            let receiver = call.receiver().expect("checked above");
            let selector = call.message_loc().expect("new selector");
            let offense = receiver.location().start_offset()..selector.end_offset();
            (offense, node_source(source, &arguments[0]).to_string())
        } else if call_name(&call) == b"[]" && root_constant(call.receiver(), b"Array") {
            let receiver = call.receiver().expect("checked above");
            let location = receiver.location();
            let offense = location.start_offset()..location.end_offset();
            let contents = arguments
                .iter()
                .map(|argument| node_source(source, argument))
                .collect::<Vec<_>>()
                .join(", ");
            (offense, format!("[{contents}]"))
        } else {
            return;
        };
        let node_location = call.location();
        context.replace(
            self.name(),
            "Remove the redundant `Array` constructor.",
            offense,
            node_location,
            replacement,
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

define_call_cop!(StringMethods => "Style/StringMethods" => string_methods);

fn string_methods(node: &CallNode<'_>, reporter: &mut CopContext<'_, '_>) {
    if !match_call(node).without_arguments().matches() {
        return;
    }
    let Ok(method) = std::str::from_utf8(call_name(node)) else {
        return;
    };
    let preferred = reporter
        .config_map("PreferredMethods")
        .and_then(|methods| methods.get(method))
        .map(String::as_str)
        .or_else(|| (method == "intern").then_some("to_sym"))
        .map(str::to_string);
    let Some(preferred) = preferred else { return };
    reporter.replace_selector(
        node,
        format!("Prefer `{preferred}` over `{method}`."),
        &preferred,
    );
}
