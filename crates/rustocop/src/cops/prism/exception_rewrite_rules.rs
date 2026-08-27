use ruby_prism::{CallNode, Node};

use super::*;
use super::source_syntax::{split_arguments, trim_range};

define_rule!(RaiseArgsRule);

define_cops! {
    RaiseArgs => "Style/RaiseArgs" => compatibility_prism_call_rule(
        RaiseArgsRule,
        on_send,
        restrict [b"raise", b"fail"]
    ),
}

impl RaiseArgsRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(node.receiver().is_some());
        if call_arguments(node).is_empty() && self.has_unparsed_command_arguments(node) {
            self.check_unparsed_command(node);
            return;
        }
        match self.policy().enforced_style("exploded") {
            "compact" => self.check_compact(node),
            _ => self.check_exploded(node),
        }
    }

    fn has_unparsed_command_arguments(&self, node: &CallNode<'_>) -> bool {
        let end = node.location().end_offset();
        self.source()
            .as_bytes()
            .get(end)
            .is_some_and(u8::is_ascii_whitespace)
            && end < self.source_file().line_end(end)
    }

    fn check_unparsed_command(&mut self, node: &CallNode<'_>) {
        let start = node.location().start_offset();
        let arguments_start = node.location().end_offset();
        let line_end = self.source_file().line_end(arguments_start);
        let tail = &self.source()[arguments_start..line_end];
        let relative_end = tail.find(" : ").unwrap_or(tail.len());
        let offense_end = arguments_start + relative_end;
        let arguments_range = trim_range(self.source(), arguments_start..offense_end);
        let arguments = split_arguments(self.source(), arguments_range.start, arguments_range.end);
        let method = String::from_utf8_lossy(node.name().as_slice());
        match self.policy().enforced_style("exploded") {
            "compact" if arguments.len() > 1 => {
                let message = format!("Provide an exception object as an argument to `{method}`.");
                if arguments.len() > 2 {
                    self.report(message, start..offense_end);
                    return;
                }
                let class = self.source()[arguments[0].clone()].trim_end_matches(".new");
                let value = self.source()[arguments[1].clone()].trim();
                let replacement = format!("{method}({class}.new({value}))");
                self.replace(message, start..offense_end, start..offense_end, replacement);
            }
            "compact" => {}
            _ => {
                let argument = self.source()[arguments_range.clone()].trim();
                let Some(new_at) = argument.find(".new") else { return };
                let class = &argument[..new_at];
                let constructor_arguments = argument[new_at + 4..].trim();
                let inner = constructor_arguments
                    .strip_prefix('(')
                    .and_then(|value| value.strip_suffix(')'))
                    .unwrap_or(constructor_arguments);
                let pieces = if inner.trim().is_empty() {
                    Vec::new()
                } else {
                    split_arguments(inner, 0, inner.len())
                };
                return_if!(pieces.len() > 1 || pieces.first().is_some_and(|range| acceptable_exploded_source(&inner[range.clone()])));
                let message = format!("Provide an exception class and message as arguments to `{method}`.");
                let suffix = pieces.first().map(|range| format!(", {}", inner[range.clone()].trim())).unwrap_or_default();
                let replacement = format!("{method}({class}{suffix})");
                self.replace(message, start..offense_end, start..offense_end, replacement);
            }
        }
    }

    fn check_compact(&mut self, node: &CallNode<'_>) {
        let arguments = call_arguments(node);
        return_if!(arguments.len() <= 1);
        return_if!(self.source_file().node(&arguments[0]).contains(':'));
        let message = format!(
            "Provide an exception object as an argument to `{}`.",
            String::from_utf8_lossy(node.name().as_slice())
        );
        if arguments.len() > 2 || self.first_argument_is_keyword_hash(&arguments[0]) {
            self.report(message, node.location());
            return;
        }
        let replacement = self.correction_exploded_to_compact(node, &arguments);
        add_offense!(self, node.location(), message: message, |corrector| {
            corrector.replace(node.location(), replacement);
        });
    }

    fn check_exploded(&mut self, node: &CallNode<'_>) {
        let arguments = call_arguments(node);
        return_unless!(arguments.len() == 1);
        let Some(constructor) = arguments[0].as_call_node() else { return };
        return_unless!(constructor.name().as_slice() == b"new" && constructor.receiver().is_some());
        let constructor_arguments = call_arguments(&constructor);
        return_if!(constructor_arguments.len() > 1);
        if let Some(argument) = constructor_arguments.first() {
            return_if!(acceptable_exploded_argument(argument));
        }
        let receiver = constructor.receiver().expect("checked above");
        let exception_type = self.source_file().node(&receiver);
        return_if!(self.config_values("AllowedCompactTypes").iter().any(|allowed| allowed == exception_type));
        let message = format!(
            "Provide an exception class and message as arguments to `{}`.",
            String::from_utf8_lossy(node.name().as_slice())
        );
        let replacement = self.correction_compact_to_exploded(node, &constructor_arguments, &receiver);
        add_offense!(self, node.location(), message: message, |corrector| {
            corrector.replace(node.location(), replacement);
        });
    }

    fn correction_exploded_to_compact(&self, node: &CallNode<'_>, arguments: &[Node<'_>]) -> String {
        let exception = &arguments[0];
        let message = self.source_file().node(&arguments[1]);
        let class = exception
            .as_call_node()
            .and_then(|call| call.receiver())
            .map_or_else(|| self.source_file().node(exception), |receiver| self.source_file().node(&receiver));
        let method = String::from_utf8_lossy(node.name().as_slice());
        let inner = format!("{class}.new({message})");
        if requires_raise_parentheses(self.ancestors()) {
            format!("{method}({inner})")
        } else {
            format!("{method} {inner}")
        }
    }

    fn correction_compact_to_exploded(
        &self,
        node: &CallNode<'_>,
        arguments: &[Node<'_>],
        receiver: &Node<'_>,
    ) -> String {
        let exception = self.source_file().node(receiver);
        let arguments = arguments
            .first()
            .map(|argument| format!(", {}", self.source_file().node(argument)))
            .unwrap_or_default();
        let method = String::from_utf8_lossy(node.name().as_slice());
        if requires_raise_parentheses(self.ancestors()) {
            format!("{method}({exception}{arguments})")
        } else {
            format!("{method} {exception}{arguments}")
        }
    }
    fn first_argument_is_keyword_hash(&self, node: &Node<'_>) -> bool {
        if self.source_file().node(node).contains(':') {
            return true;
        }
        node.as_call_node()
            .and_then(|call| call.arguments())
            .and_then(|arguments| arguments.arguments().first())
            .is_some_and(|argument| {
                argument.as_keyword_hash_node().is_some()
                    || argument.as_hash_node().is_some()
                    || self.source_file().node(&argument).contains(':')
            })
    }
}

fn call_arguments<'pr>(node: &CallNode<'pr>) -> Vec<Node<'pr>> {
    node.arguments()
        .map(|arguments| arguments.arguments().iter().collect())
        .unwrap_or_default()
}

fn acceptable_exploded_argument(node: &Node<'_>) -> bool {
    node.as_keyword_hash_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_splat_node().is_some()
        || node.as_forwarding_arguments_node().is_some()
        || node.as_assoc_splat_node().is_some()
}

fn acceptable_exploded_source(source: &str) -> bool {
    let source = source.trim();
    source.starts_with('*') || source.starts_with("...") || source.contains(':')
}

fn requires_raise_parentheses(ancestors: &[Node<'_>]) -> bool {
    ancestors.iter().rev().any(|parent| {
        parent.as_if_node().is_some_and(|node| node.if_keyword_loc().is_none())
            || parent.as_and_node().is_some()
            || parent.as_or_node().is_some()
            || parent.as_call_node().is_some_and(|call| matches!(call.name().as_slice(), b"&&" | b"||" | b"and" | b"or"))
    })
}
