use ruby_prism::{BlockNode, LambdaNode};

use super::*;

define_cops! {
    Lambda => "Style/Lambda" => rubocop_callbacks(LambdaRule, [on_block, on_lambda]),
}

impl LambdaRule<'_, '_, '_> {
    fn on_block(&mut self, node: &BlockNode<'_>) {
        let Some(call) = self.parent().and_then(|parent| parent.as_call_node()) else {
            return;
        };
        return_unless!(
            call.name().as_slice() == b"lambda"
                && call.receiver().is_none()
                && call.opening_loc().is_none()
        );
        let multiline = self.source_file().node(&node.as_node()).contains('\n');
        return_unless!(self.method_form_offends(multiline));

        let selector = call.message_loc().unwrap_or_else(|| call.location());
        let selector = selector.start_offset()..selector.end_offset();
        let parameters = node.parameters().and_then(|parameters| {
            let source = self.source_file().node(&parameters).trim();
            (source.starts_with('|') && source.ends_with('|')).then(|| {
                source.trim_matches('|').trim().to_string()
            })
        });
        let replacement = parameters
            .as_deref()
            .filter(|parameters| !parameters.is_empty())
            .map_or_else(|| "->".to_string(), |parameters| format!("->({parameters})"));
        let message = self.literal_message(multiline);
        let parameter_removal = node.parameters().and_then(|parameters| {
            let source = self.source_file().node(&parameters).trim();
            (source.starts_with('|') && source.ends_with('|')).then(|| {
                node.opening_loc().end_offset()..parameters.location().end_offset()
            })
        });
        add_offense!(self, selector.clone(), message: message, |corrector| {
            corrector.replace(selector, replacement);
            if let Some(range) = parameter_removal {
                corrector.remove(range);
            }
        });
    }

    fn on_lambda(&mut self, node: &LambdaNode<'_>) {
        let multiline = self.source_file().node(&node.as_node()).contains('\n');
        return_unless!(self.literal_form_offends(multiline));

        let operator = node.operator_loc();
        let operator_range = operator.start_offset()..operator.end_offset();
        let opening = node.opening_loc();
        let parameters = node.parameters().and_then(|parameters| {
            (parameters.location().end_offset() <= opening.start_offset()).then(|| {
                self.source_file()
                    .node(&parameters)
                    .trim()
                    .trim_start_matches('(')
                    .trim_end_matches(')')
                    .trim()
                    .to_string()
            })
        });
        let between = operator.end_offset()..opening.start_offset();
        let between_source = self.source_file().slice(between.clone()).unwrap_or_default();
        let opening_source = self.source_file().at(&opening);
        let separator = if opening_source == "do" || between_source.ends_with(char::is_whitespace) {
            " "
        } else {
            ""
        };
        let inserted_parameters = parameters
            .as_deref()
            .filter(|parameters| !parameters.is_empty())
            .map(|parameters| format!(" |{parameters}|"));
        let message = self.method_message(multiline);
        let braces_required = opening_source == "do"
            && self.ancestors().iter().rev().find_map(|ancestor| ancestor.as_call_node())
                .is_some_and(|call| call.opening_loc().is_none());
        let closing = node.closing_loc();
        add_offense!(self, operator_range.clone(), message: message, |corrector| {
            corrector.replace(operator_range, "lambda");
            if parameters.is_some() || opening_source == "do" && between_source.is_empty() {
                corrector.replace(between, separator);
            }
            if let Some(parameters) = inserted_parameters {
                corrector.replace(opening.end_offset()..opening.end_offset(), parameters);
            }
            if braces_required {
                corrector.replace(opening, "{");
                corrector.replace(closing, "}");
            }
        });
    }

    fn method_form_offends(&self, multiline: bool) -> bool {
        match self.policy().enforced_style("line_count_dependent") {
            "literal" => true,
            "line_count_dependent" => !multiline,
            _ => false,
        }
    }

    fn literal_form_offends(&self, multiline: bool) -> bool {
        match self.policy().enforced_style("line_count_dependent") {
            "lambda" => true,
            "line_count_dependent" => multiline,
            _ => false,
        }
    }

    fn literal_message(&self, multiline: bool) -> String {
        format!(
            "Use the `-> {{ ... }}` lambda literal syntax for {} lambdas.",
            self.modifier(multiline)
        )
    }

    fn method_message(&self, multiline: bool) -> String {
        format!("Use the `lambda` method for {} lambdas.", self.modifier(multiline))
    }

    fn modifier(&self, multiline: bool) -> &'static str {
        if self.policy().enforced_style("line_count_dependent") != "line_count_dependent" {
            "all"
        } else if multiline {
            "multiline"
        } else {
            "single line"
        }
    }
}
