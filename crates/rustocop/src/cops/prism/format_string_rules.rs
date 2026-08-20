use ruby_prism::{CallNode, Node};

use super::*;

define_cops! {
    FormatString => "Style/FormatString" => rubocop_callbacks(FormatStringRule, [on_send restrict [b"format", b"sprintf", b"%"]]),
}

impl FormatStringRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let method = node.name().as_slice();
        let arguments = arguments(node);
        let detected = if method == b"%" {
            let (Some(receiver), Some(argument)) = (node.receiver(), arguments.first()) else { return };
            return_unless!(string_literal(&receiver) || argument.as_array_node().is_some() || argument.as_hash_node().is_some());
            "percent"
        } else {
            return_if!(node.receiver().is_some() || arguments.len() < 2);
            if method == b"format" { "format" } else { "sprintf" }
        };
        let preferred = self.policy().enforced_style("format").to_string();
        return_if!(detected == preferred);
        let Some(selector) = node.message_loc() else { return };
        let selector = selector.start_offset()..selector.end_offset();
        let message = format!("Favor `{}` over `{}`.", display(&preferred), display(detected));

        if (detected == "format" && preferred == "sprintf")
            || (detected == "sprintf" && preferred == "format")
        {
            add_offense!(self, selector.clone(), message: message, |corrector| { corrector.replace(selector, preferred); });
        } else if detected == "percent" {
            let argument = &arguments[0];
            if unsafe_variable_argument(argument) {
                self.report(message, selector);
                return;
            }
            let receiver = node.receiver().expect("percent receiver");
            let args = collection_contents(argument, self.source_file());
            let replacement = format!("{preferred}({}, {args})", self.source_file().node(&receiver));
            add_offense!(self, selector, message: message, |corrector| { corrector.replace(node.location(), replacement); });
        } else {
            let format = self.source_file().node(&arguments[0]);
            let params = &arguments[1..];
            let args = if params.len() == 1 {
                let source = self.source_file().node(&params[0]);
                if params[0].as_keyword_hash_node().is_some() || params[0].as_hash_node().is_some() {
                    format!("{{ {source} }}")
                } else if operator_call_without_parentheses(&params[0]) {
                    format!("({source})")
                } else {
                    source.to_string()
                }
            } else {
                format!("[{}]", params.iter().map(|argument| self.source_file().node(argument)).collect::<Vec<_>>().join(", "))
            };
            add_offense!(self, selector, message: message, |corrector| { corrector.replace(node.location(), format!("{format} % {args}")); });
        }
    }
}

fn display(style: &str) -> &str {
    if style == "percent" { "String#%" } else { style }
}

fn arguments<'pr>(node: &CallNode<'pr>) -> Vec<Node<'pr>> {
    node.arguments().map(|arguments| arguments.arguments().iter().collect()).unwrap_or_default()
}

fn string_literal(node: &Node<'_>) -> bool {
    node.as_string_node().is_some() || node.as_interpolated_string_node().is_some()
}

fn unsafe_variable_argument(node: &Node<'_>) -> bool {
    node.as_local_variable_read_node().is_some()
        || node.as_call_node().is_some_and(|call| !matches!(call.name().as_slice(), b"to_d" | b"to_f" | b"to_h" | b"to_i" | b"to_r" | b"to_s" | b"to_sym"))
}

fn collection_contents<'a>(node: &Node<'_>, file: SourceFile<'a>) -> &'a str {
    let source = file.node(node);
    if node.as_array_node().is_some() {
        source.strip_prefix('[').and_then(|source| source.strip_suffix(']')).unwrap_or(source)
    } else if node.as_hash_node().is_some() || node.as_keyword_hash_node().is_some() {
        source.strip_prefix('{').and_then(|source| source.strip_suffix('}')).unwrap_or(source).trim()
    } else {
        source
    }
}

fn operator_call_without_parentheses(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| call.opening_loc().is_none() && matches!(call.name().as_slice(), b"+" | b"-" | b"*" | b"/" | b"%" | b"**" | b"<<" | b">>"))
}
