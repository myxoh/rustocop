use ruby_prism::CallNode;

use super::*;

define_cops! {
    HashConversion => "Style/HashConversion" => rubocop_callbacks(HashConversionRule, [on_send restrict [b"[]"]]),
}

impl HashConversionRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_if!(self.parent().and_then(ruby_prism::Node::as_call_node).is_some_and(|parent| {
            parent.name().as_slice() == b"[]"
                && parent.receiver().is_some_and(|receiver| matches!(self.source_file().node(&receiver), "Hash" | "::Hash"))
        }));
        let Some(receiver) = node.receiver() else { return };
        return_unless!(matches!(self.source_file().node(&receiver), "Hash" | "::Hash"));
        let arguments = node
            .arguments()
            .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let offense = node.location();

        if arguments.len() == 1 {
            let argument = &arguments[0];
            if argument.as_splat_node().is_some()
                || argument.as_forwarding_arguments_node().is_some()
            {
                return_if!(self.config_bool("AllowSplatArgument", true));
                self.report("Prefer `array_of_pairs.to_h` to `Hash[*array]`.", offense);
                return;
            }
            if argument.as_keyword_hash_node().is_some() || argument.as_hash_node().is_some() {
                let source = self.source_file().node(argument);
                let replacement = format!("{{{source}}}");
                let parentheses = self.parentheses_for_literal();
                add_offense!(self, offense, message: "Prefer literal hash to `Hash[key: value, ...]`.", |corrector| {
                    corrector.replace(node.location(), replacement);
                    if let Some((open, close)) = parentheses {
                        corrector.replace(open, "(");
                        corrector.replace(close..close, ")");
                    }
                });
                return;
            }
            let source = self.source_file().node(argument);
            let replacement = if source.contains(" || ") || source.contains(" && ")
                || argument.as_call_node().is_some_and(|call| argument_count(&call) > 0 && call.opening_loc().is_none())
            {
                format!("({}).to_h", source.trim_matches(['(', ')']))
            } else if argument.as_call_node().is_some_and(|call| call.name().as_slice() == b"zip" && argument_count(&call) == 0) {
                if source.ends_with("()") {
                    format!("{}([]).to_h", source.trim_end_matches("()"))
                } else {
                    format!("{source}([]).to_h")
                }
            } else {
                format!("{source}.to_h")
            };
            add_offense!(self, offense, message: "Prefer `ary.to_h` to `Hash[ary]`.", |corrector| {
                corrector.replace(node.location(), replacement);
            });
            return;
        }

        let message = "Prefer literal hash to `Hash[arg1, arg2, ...]`.";
        if arguments.len() % 2 == 1 {
            self.report(message, offense);
            return;
        }
        let replacement = arguments
            .chunks(2)
            .map(|pair| format!("{} => {}", self.source_file().node(&pair[0]), self.source_file().node(&pair[1])))
            .collect::<Vec<_>>()
            .join(", ");
        let replacement = format!("{{{replacement}}}");
        let parentheses = self.parentheses_for_literal();
        add_offense!(self, offense, message: message, |corrector| {
            corrector.replace(node.location(), replacement);
            if let Some((open, close)) = parentheses {
                corrector.replace(open, "(");
                corrector.replace(close..close, ")");
            }
        });
    }

    fn parentheses_for_literal(&self) -> Option<(std::ops::Range<usize>, usize)> {
        let parent = self.parent().and_then(ruby_prism::Node::as_call_node)?;
        if parent.opening_loc().is_some() || parent.name().as_slice() == b"to_h" {
            return None;
        }
        Some((parent.message_loc()?.end_offset()..parent.arguments()?.location().start_offset(), parent.location().end_offset()))
    }
}
