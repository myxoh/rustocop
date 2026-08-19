use ruby_prism::{DefNode, Node, ParametersNode};

use super::*;

define_cops! {
    SuperArguments => "Style/SuperArguments" => node(as_super_node, super_arguments),
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum Forwarded {
    Positional(String),
    Rest(Option<String>),
    Keyword(String),
    KeywordRest(Option<String>),
    Block(Option<String>),
    All,
}

fn super_arguments(node: &ruby_prism::SuperNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(definition) = enclosing_definition(context.ancestors()) else {
        return;
    };
    let literal_block = node.block().and_then(|block| block.as_block_node()).is_some();
    let mut definition_arguments = definition_parameters(&definition);
    let Some(forwarded_arguments) = forwarded_arguments(node, context) else {
        return;
    };

    let definition_has_block = definition_arguments
        .last()
        .is_some_and(|argument| matches!(argument, Forwarded::Block(_)));
    if literal_block && definition_has_block {
        definition_arguments.pop();
    }
    if definition_arguments != forwarded_arguments {
        return;
    }
    if let Some(Forwarded::Block(Some(name))) = forwarded_arguments.last() {
        if block_reassigned(&definition, name) {
            return;
        }
    }

    let edit_end = node
        .rparen_loc()
        .map(|location| location.end_offset())
        .or_else(|| {
            node.block()
                .and_then(|block| block.as_block_argument_node())
                .map(|block| block.location().end_offset())
        })
        .or_else(|| {
            node.arguments()
                .and_then(|arguments| arguments.arguments().last())
                .map(|argument| argument.location().end_offset())
        })
        .unwrap_or_else(|| node.keyword_loc().end_offset());
    let offense = node.keyword_loc().start_offset()..edit_end;
    let message = if literal_block && definition_has_block {
        "Call `super` without arguments and parentheses when all positional and keyword arguments are forwarded."
    } else {
        "Call `super` without arguments and parentheses when the signature is identical."
    };
    context.replace(message, offense.clone(), offense, "super");
}

fn enclosing_definition<'pr>(ancestors: &[Node<'pr>]) -> Option<DefNode<'pr>> {
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_block_node().is_some() {
            return None;
        }
        if let Some(definition) = ancestor.as_def_node() {
            return Some(definition);
        }
    }
    None
}

fn definition_parameters(definition: &DefNode<'_>) -> Vec<Forwarded> {
    let Some(parameters) = definition.parameters() else {
        return Vec::new();
    };
    parameter_list(&parameters)
}

fn parameter_list(parameters: &ParametersNode<'_>) -> Vec<Forwarded> {
    let mut result = Vec::new();
    for parameter in parameters.requireds().iter() {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            result.push(Forwarded::Positional(name(parameter.name())));
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            result.push(Forwarded::Positional(name(parameter.name())));
        }
    }
    if let Some(rest) = parameters.rest() {
        if rest.as_forwarding_parameter_node().is_some() {
            result.push(Forwarded::All);
        } else if let Some(rest) = rest.as_rest_parameter_node() {
            result.push(Forwarded::Rest(rest.name().map(name)));
        }
    }
    for parameter in parameters.posts().iter() {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            result.push(Forwarded::Positional(name(parameter.name())));
        }
    }
    for parameter in parameters.keywords().iter() {
        if let Some(parameter) = parameter.as_required_keyword_parameter_node() {
            result.push(Forwarded::Keyword(name(parameter.name())));
        } else if let Some(parameter) = parameter.as_optional_keyword_parameter_node() {
            result.push(Forwarded::Keyword(name(parameter.name())));
        }
    }
    if let Some(rest) = parameters.keyword_rest() {
        if rest.as_forwarding_parameter_node().is_some() {
            if !result.contains(&Forwarded::All) {
                result.push(Forwarded::All);
            }
        } else if let Some(rest) = rest.as_keyword_rest_parameter_node() {
            result.push(Forwarded::KeywordRest(rest.name().map(name)));
        }
    }
    if let Some(block) = parameters.block() {
        result.push(Forwarded::Block(block.name().map(name)));
    }
    result
}

fn forwarded_arguments(
    node: &ruby_prism::SuperNode<'_>,
    context: &CopContext<'_, '_>,
) -> Option<Vec<Forwarded>> {
    let mut result = Vec::new();
    if let Some(arguments) = node.arguments() {
        for argument in arguments.arguments().iter() {
            if let Some(read) = argument.as_local_variable_read_node() {
                result.push(Forwarded::Positional(name(read.name())));
            } else if let Some(splat) = argument.as_splat_node() {
                let forwarded = splat.expression().map(|expression| {
                    expression
                        .as_local_variable_read_node()
                        .map(|read| name(read.name()))
                });
                result.push(Forwarded::Rest(forwarded.flatten()));
            } else if argument.as_forwarding_arguments_node().is_some() {
                result.push(Forwarded::All);
            } else if let Some(hash) = argument.as_keyword_hash_node() {
                for element in hash.elements().iter() {
                    if let Some(assoc) = element.as_assoc_node() {
                        let key = assoc.key().as_symbol_node()?;
                        let key = String::from_utf8_lossy(key.unescaped()).into_owned();
                        let value = assoc.value();
                        let same_value = value
                            .as_local_variable_read_node()
                            .is_some_and(|read| read.name().as_slice() == key.as_bytes());
                        let shorthand = context
                            .source_file()
                            .node(&element)
                            .trim_end()
                            .ends_with(':');
                        if !same_value && !shorthand {
                            return None;
                        }
                        result.push(Forwarded::Keyword(key));
                    } else if let Some(splat) = element.as_assoc_splat_node() {
                        let value = splat.value().map(|value| {
                            value
                                .as_local_variable_read_node()
                                .map(|read| name(read.name()))
                        });
                        result.push(Forwarded::KeywordRest(value.flatten()));
                    } else {
                        return None;
                    }
                }
            } else {
                return None;
            }
        }
    }
    if let Some(block) = node.block().and_then(|block| block.as_block_argument_node()) {
        let value = block.expression().map(|value| {
            value
                .as_local_variable_read_node()
                .map(|read| name(read.name()))
        });
        result.push(Forwarded::Block(value.flatten()));
    }
    Some(result)
}

fn block_reassigned(definition: &DefNode<'_>, block_name: &str) -> bool {
    let mut finder = BlockReassignmentFinder {
        name: block_name.as_bytes(),
        found: false,
    };
    if let Some(body) = definition.body() {
        finder.visit(&body);
    }
    finder.found
}

struct BlockReassignmentFinder<'a> {
    name: &'a [u8],
    found: bool,
}

impl<'pr> Visit<'pr> for BlockReassignmentFinder<'_> {
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.found |= node.name().as_slice() == self.name;
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        self.found |= node.name().as_slice() == self.name;
    }
}

fn name(id: ruby_prism::ConstantId<'_>) -> String {
    String::from_utf8_lossy(id.as_slice()).into_owned()
}
