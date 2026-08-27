use ruby_prism::{DefNode, Node, ReturnNode, Visit};

use super::*;

define_rule!(ReturnNilInPredicateMethodDefinitionRule);

const MSG: &str = "Return `false` instead of `nil` in predicate methods.";

define_cops! {
    ReturnNilInPredicateMethodDefinition => "Style/ReturnNilInPredicateMethodDefinition" => compatibility_prism_node_rule(as_def_node, ReturnNilInPredicateMethodDefinitionRule, on_def),
}

impl ReturnNilInPredicateMethodDefinitionRule<'_, '_, '_> {
    fn on_def(&mut self, node: &DefNode<'_>) {
        let method_name = node.name().as_slice();
        return_unless!(method_name.ends_with(b"?"));
        return_if!(self.policy().allows_method(method_name));
        let Some(body) = node.body() else {
            return;
        };

        let mut returns = ReturnCollector::default();
        returns.visit(&body);
        for offense in returns.offenses {
            self.register_offense(offense, "return false");
        }
        self.handle_implicit_return_values(body);
    }

    fn handle_implicit_return_values(&mut self, node: Node<'_>) {
        if let Some(last) = last_statement(&node) {
            self.handle_implicit_return_values(last);
            return;
        }
        if node.as_nil_node().is_some() {
            let location = node.location();
            self.register_offense(location.start_offset()..location.end_offset(), "false");
            return;
        }
        if let Some(if_node) = node.as_if_node() {
            if let Some(statements) = if_node.statements() {
                self.handle_implicit_return_values(statements.as_node());
            }
            if let Some(subsequent) = if_node.subsequent() {
                if let Some(else_node) = subsequent.as_else_node() {
                    if let Some(statements) = else_node.statements() {
                        self.handle_implicit_return_values(statements.as_node());
                    }
                } else if subsequent.as_if_node().is_some() {
                    self.handle_implicit_return_values(subsequent);
                }
            }
        }
    }

    fn register_offense(&mut self, offense: std::ops::Range<usize>, replacement: &str) {
        add_offense!(self, offense.clone(), message: MSG, |corrector| {
            corrector.replace(offense, replacement);
        });
    }
}

def_node_matcher! {
    fn return_nil(node: &ReturnNode<'_>) -> bool {
        let arguments = node.arguments();
        arguments.as_ref().is_none_or(|arguments| arguments.arguments().is_empty())
            || arguments.is_some_and(|arguments| {
                arguments.arguments().len() == 1
                    && arguments.arguments().first().is_some_and(|argument| argument.as_nil_node().is_some())
            })
    }
}

fn last_statement<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    node.as_statements_node()?.body().last()
}

#[derive(Default)]
struct ReturnCollector {
    offenses: Vec<std::ops::Range<usize>>,
}

impl<'pr> Visit<'pr> for ReturnCollector {
    fn visit_return_node(&mut self, node: &ReturnNode<'pr>) {
        if return_nil(node) {
            self.offenses
                .push(node.location().start_offset()..node.location().end_offset());
        }
        ruby_prism::visit_return_node(self, node);
    }

    fn visit_def_node(&mut self, _node: &DefNode<'pr>) {}
}
