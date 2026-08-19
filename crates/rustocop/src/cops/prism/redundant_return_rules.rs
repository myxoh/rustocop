use ruby_prism::{Node, ReturnNode};

use super::*;

define_rule!(RedundantReturnRule);

const MSG: &str = "Redundant `return` detected.";
const MULTI_RETURN_MSG: &str = "To return multiple values, use an array.";

define_cops! {
    RedundantReturn => "Style/RedundantReturn" => node_rule_aliases(RedundantReturnRule, on_scope => [as_def_node, as_call_node, as_lambda_node]),
}

impl RedundantReturnRule<'_, '_, '_> {
    fn on_scope(&mut self, node: &Node<'_>) {
        let body = if let Some(definition) = node.as_def_node() {
            definition.body()
        } else if let Some(lambda) = node.as_lambda_node() {
            lambda.body()
        } else if let Some(call) = node.as_call_node() {
            return_unless!(matches!(
                call.name().as_slice(),
                b"define_method" | b"define_singleton_method" | b"lambda"
            ));
            call.block()
                .and_then(|block| block.as_block_node())
                .and_then(|block| block.body())
        } else {
            None
        };
        if let Some(body) = body {
            self.check_branch(body);
        }
    }

    fn check_branch(&mut self, node: Node<'_>) {
        if let Some(statements) = node.as_statements_node() {
            if let Some(last) = statements.body().last() {
                self.check_branch(last);
            }
        } else if let Some(return_node) = node.as_return_node() {
            self.check_return_node(&return_node);
        } else if let Some(if_node) = node.as_if_node() {
            return_if!(if_node.if_keyword_loc().is_none());
            if let Some(statements) = if_node.statements() {
                self.check_branch(statements.as_node());
            }
            if let Some(subsequent) = if_node.subsequent() {
                self.check_branch(subsequent);
            }
        } else if let Some(unless_node) = node.as_unless_node() {
            if let Some(statements) = unless_node.statements() {
                self.check_branch(statements.as_node());
            }
            if let Some(else_node) = unless_node.else_clause() {
                self.check_branch(else_node.as_node());
            }
        } else if let Some(else_node) = node.as_else_node() {
            if let Some(statements) = else_node.statements() {
                self.check_branch(statements.as_node());
            }
        } else if let Some(case_node) = node.as_case_node() {
            for condition in case_node.conditions().iter() {
                self.check_branch(condition);
            }
            if let Some(else_node) = case_node.else_clause() {
                self.check_branch(else_node.as_node());
            }
        } else if let Some(case_node) = node.as_case_match_node() {
            for condition in case_node.conditions().iter() {
                self.check_branch(condition);
            }
            if let Some(else_node) = case_node.else_clause() {
                self.check_branch(else_node.as_node());
            }
        } else if let Some(when_node) = node.as_when_node() {
            if let Some(statements) = when_node.statements() {
                self.check_branch(statements.as_node());
            }
        } else if let Some(in_node) = node.as_in_node() {
            if let Some(statements) = in_node.statements() {
                self.check_branch(statements.as_node());
            }
        } else if let Some(begin_node) = node.as_begin_node() {
            self.check_begin_node(&begin_node);
        } else if let Some(rescue_node) = node.as_rescue_node() {
            self.check_rescue_node(&rescue_node);
        }
    }

    fn check_begin_node(&mut self, node: &ruby_prism::BeginNode<'_>) {
        if let Some(rescue) = node.rescue_clause() {
            self.check_rescue_node(&rescue);
            if let Some(else_node) = node.else_clause() {
                self.check_branch(else_node.as_node());
            } else if let Some(statements) = node.statements() {
                self.check_branch(statements.as_node());
            }
        } else if let Some(statements) = node.statements() {
            self.check_branch(statements.as_node());
        }
    }

    fn check_rescue_node(&mut self, node: &ruby_prism::RescueNode<'_>) {
        let mut branch = Some(node.as_node());
        while let Some(current) = branch {
            let rescue = current.as_rescue_node().expect("rescue chain");
            if let Some(statements) = rescue.statements() {
                self.check_branch(statements.as_node());
            }
            branch = rescue.subsequent().map(|subsequent| subsequent.as_node());
        }
    }

    fn check_return_node(&mut self, node: &ReturnNode<'_>) {
        let arguments = node.arguments();
        let count = arguments
            .as_ref()
            .map_or(0, |arguments| arguments.arguments().len());
        return_if!(self.config_bool("AllowMultipleReturnValues", false) && count > 1);

        let message = if count > 1 {
            format!("{MSG} {MULTI_RETURN_MSG}")
        } else {
            MSG.to_string()
        };
        let replacement = self.replacement(node, count);
        let keyword = node.keyword_loc();
        add_offense!(self, keyword, message: message, |corrector| {
            corrector.replace(node.location(), replacement);
        });
    }

    fn replacement(&self, node: &ReturnNode<'_>, count: usize) -> String {
        let Some(arguments) = node.arguments() else {
            return "nil".to_string();
        };
        if count == 0 {
            return "nil".to_string();
        }
        let values = arguments.arguments().iter().collect::<Vec<_>>();
        if values.len() == 1
            && values[0]
                .as_parentheses_node()
                .is_some_and(|parentheses| parentheses.body().is_none())
        {
            return "nil".to_string();
        }
        let start = values.first().unwrap().location().start_offset();
        let end = values.last().unwrap().location().end_offset();
        let source = self.source().get(start..end).unwrap_or_default();
        if count > 1 {
            return format!("[{source}]");
        }
        let argument = &values[0];
        if let Some(splat) = argument.as_splat_node() {
            return splat
                .expression()
                .map(|expression| self.source_of(&expression).to_string())
                .unwrap_or_default();
        }
        if argument.as_keyword_hash_node().is_some() {
            return format!("{{{source}}}");
        }
        source.to_string()
    }
}
