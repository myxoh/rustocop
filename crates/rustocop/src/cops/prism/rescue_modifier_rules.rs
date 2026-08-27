use ruby_prism::{Node, RescueModifierNode};

use super::*;

define_rule!(RescueModifierRule);

const MSG: &str = "Avoid using `rescue` in its modifier form.";

define_cops! {
    RescueModifier => "Style/RescueModifier" => compatibility_prism_node_rule(as_rescue_modifier_node, RescueModifierRule, on_resbody),
}

impl RescueModifierRule<'_, '_, '_> {
    fn on_resbody(&mut self, node: &RescueModifierNode<'_>) {
        return_if!(self.policy().excluded_path(self.path()));

        if !self.target_ruby_version().at_least(2, 7) {
            if let Some(parent) = self
                .ancestors()
                .iter()
                .rev()
                .find_map(Node::as_multi_write_node)
            {
                let parent_location = parent.location();
                let edit = parent_location.start_offset()..parent_location.end_offset();
                let operation = self
                    .source()
                    .get(edit.start..node.keyword_loc().start_offset())
                    .unwrap_or_default()
                    .trim_end();
                let replacement = self.render(operation, self.source_of(&node.rescue_expression()), "");
                add_offense!(self, edit.clone(), message: MSG, |corrector| {
                    corrector.replace(edit, replacement);
                });
                return;
            }
        }

        let parenthesized = self
            .ancestors()
            .iter()
            .rev()
            .find_map(Node::as_parentheses_node);
        let edit = parenthesized
            .as_ref()
            .map_or_else(|| node.location(), |parent| parent.location());
        let offset = self.indentation(edit.start_offset());
        let (operation, operation_end) = self.operation_source(&node.expression());
        let handler = self.source_of(&node.rescue_expression());
        let mut replacement = self.render(&operation, handler, &offset);
        if operation_end > node.expression().location().end_offset() {
            replacement.push('\n');
        }
        let edit = edit.start_offset()..operation_end.max(edit.end_offset());

        add_offense!(self, node.location(), message: MSG, |corrector| {
            corrector.replace(edit, replacement);
        });
    }

    fn operation_source(&self, operation: &Node<'_>) -> (String, usize) {
        let source = if let Some(nested) = operation.as_rescue_modifier_node() {
            self.render(
                &self.operation_source(&nested.expression()).0,
                self.source_of(&nested.rescue_expression()),
                "",
            )
        } else if let Some(array) = operation.as_array_node() {
            if array.opening_loc().is_none() {
                format!("[{}]", self.source_of(operation))
            } else {
                self.source_of(operation).to_string()
            }
        } else {
            self.source_of(operation).to_string()
        };
        let Some(heredoc_end) = last_heredoc_end(operation) else {
            return (source, operation.location().end_offset());
        };
        let tail = self
            .source()
            .get(operation.location().end_offset()..heredoc_end)
            .unwrap_or_default()
            .strip_prefix(
                self.source()
                    .get(operation.location().end_offset()..)
                    .and_then(|suffix| suffix.lines().next())
                    .unwrap_or_default(),
            )
            .unwrap_or_default()
            .strip_prefix('\n')
            .unwrap_or_default()
            .strip_suffix('\n')
            .unwrap_or_default();
        (format!("{source}\n{tail}"), heredoc_end)
    }

    fn render(&self, operation: &str, handler: &str, offset: &str) -> String {
        let node_indentation = format!("{offset}  ");
        format!(
            "begin\n{node_indentation}{operation}\n{offset}rescue\n{node_indentation}{handler}\n{offset}end"
        )
    }

    fn indentation(&self, offset: usize) -> String {
        let line_start = self.source()[..offset].rfind('\n').map_or(0, |at| at + 1);
        " ".repeat(offset - line_start)
    }
}

fn last_heredoc_end(node: &Node<'_>) -> Option<usize> {
    let call = node.as_call_node()?;
    call.arguments()?
        .arguments()
        .iter()
        .filter_map(|argument| {
            argument
                .as_string_node()
                .filter(|string| {
                    string
                        .opening_loc()
                        .is_some_and(|opening| opening.as_slice().starts_with(b"<<"))
                })
                .and_then(|string| string.closing_loc())
                .or_else(|| {
                    argument
                        .as_interpolated_string_node()
                        .filter(|string| {
                            string
                                .opening_loc()
                                .is_some_and(|opening| opening.as_slice().starts_with(b"<<"))
                        })
                        .and_then(|string| string.closing_loc())
                })
                .map(|closing| closing.end_offset())
        })
        .max()
}
