use ruby_prism::{BlockNode, CallNode, Node};

use super::*;

define_rule!(RedundantMinMaxByRule);

const MSG_BLOCK: &str = "Use `{replacement}` instead of `{original} { |{var}| {var} }`.";
const MSG_NUMBLOCK: &str = "Use `{replacement}` instead of `{original} { _1 }`.";
const MSG_ITBLOCK: &str = "Use `{replacement}` instead of `{original} { it }`.";

define_cops! {
    RedundantMinMaxBy => "Style/RedundantMinMaxBy" => call_rule(
        RedundantMinMaxByRule,
        on_block,
        restrict [b"max_by", b"min_by", b"minmax_by"]
    ),
}

impl RedundantMinMaxByRule<'_, '_, '_> {
    fn on_block(&mut self, node: &CallNode<'_>) {
        let Some(block) = node.block().and_then(|candidate| candidate.as_block_node()) else {
            return;
        };
        let Some(identity) = redundant_minmax_by_block(&block) else {
            return;
        };
        let replacement = replacement(node);
        let original = String::from_utf8_lossy(node.name().as_slice());
        let message = match identity {
            Identity::Named(var) => MSG_BLOCK
                .replace("{replacement}", replacement)
                .replace("{original}", &original)
                .replace("{var}", &var),
            Identity::Numbered => MSG_NUMBLOCK
                .replace("{replacement}", replacement)
                .replace("{original}", &original),
            Identity::It => MSG_ITBLOCK
                .replace("{replacement}", replacement)
                .replace("{original}", &original),
        };
        self.register_offense(node, &block, message, replacement);
    }

    fn register_offense(
        &mut self,
        send: &CallNode<'_>,
        block: &BlockNode<'_>,
        message: String,
        replacement: &str,
    ) {
        let Some(selector) = send.message_loc() else {
            return;
        };
        let range = selector.start_offset()..block.closing_loc().end_offset();
        add_offense!(self, range.clone(), message: message, |corrector| {
            corrector.replace(range, replacement);
        });
    }
}

enum Identity {
    Named(String),
    Numbered,
    It,
}

def_node_matcher! {
    fn redundant_minmax_by_block(block: &BlockNode<'_>) -> Option<Identity> {
        let parameters = block.parameters()?;
        let body = block.body().and_then(single_expression)?;
        if let Some(numbered) = parameters.as_numbered_parameters_node() {
            return (numbered.maximum() == 1
                && body.as_local_variable_read_node().is_some_and(|read| read.name().as_slice() == b"_1"))
                .then_some(Identity::Numbered);
        }
        if parameters.as_it_parameters_node().is_some() {
            return body.as_it_local_variable_read_node().is_some().then_some(Identity::It);
        }
        let parameters = parameters.as_block_parameters_node()?.parameters()?;
        if parameters.requireds().len() != 1
            || !parameters.optionals().is_empty()
            || parameters.rest().is_some()
            || !parameters.posts().is_empty()
            || !parameters.keywords().is_empty()
            || parameters.keyword_rest().is_some()
            || parameters.block().is_some()
        {
            return None;
        }
        let parameter = parameters.requireds().first()?.as_required_parameter_node()?;
        let read = body.as_local_variable_read_node()?;
        (read.name().as_slice() == parameter.name().as_slice()).then(|| {
            Identity::Named(String::from_utf8_lossy(parameter.name().as_slice()).into_owned())
        })
    }
}

fn replacement(node: &CallNode<'_>) -> &'static str {
    match node.name().as_slice() {
        b"max_by" => "max",
        b"min_by" => "min",
        b"minmax_by" => "minmax",
        _ => unreachable!("restricted callback"),
    }
}
