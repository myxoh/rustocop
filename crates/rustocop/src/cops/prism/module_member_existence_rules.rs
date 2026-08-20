use ruby_prism::{CallNode, Node};

use super::*;

define_cops! {
    ModuleMemberExistenceCheck => "Style/ModuleMemberExistenceCheck" => rubocop_callbacks(
        ModuleMemberExistenceCheckRule,
        [on_send restrict [
            b"class_variables",
            b"instance_methods",
            b"private_instance_methods",
            b"protected_instance_methods",
            b"public_instance_methods",
        ]]
    ),
}

impl ModuleMemberExistenceCheckRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some(parent) = self.parent().and_then(Node::as_call_node) else {
            return;
        };
        return_unless!(matches!(parent.name().as_slice(), b"include?" | b"member?"));
        return_unless!(parent.receiver().is_some_and(|receiver| {
            receiver.location().start_offset() == node.location().start_offset()
                && receiver.location().end_offset() == node.location().end_offset()
        }));
        return_unless!(simple_method_argument(node) && simple_method_argument(&parent));
        return_unless!(node.name().as_slice() != b"class_variables" || node.first_argument().is_none());

        let Some(argument) = parent.first_argument() else {
            return;
        };
        let replacement_method = match node.name().as_slice() {
            b"class_variables" => "class_variable_defined?",
            b"instance_methods" => "method_defined?",
            b"private_instance_methods" => "private_method_defined?",
            b"protected_instance_methods" => "protected_method_defined?",
            b"public_instance_methods" => "public_method_defined?",
            _ => return,
        };
        let argument_source = self.source_file().node(&argument);
        let inherit = node.first_argument();
        let replacement = match inherit {
            None => format!("{replacement_method}({argument_source})"),
            Some(value)
                if node.name().as_slice() == b"class_variables" || value.as_true_node().is_some() =>
            {
                format!("{replacement_method}({argument_source})")
            }
            Some(value) => format!(
                "{replacement_method}({argument_source}, {})",
                self.source_file().node(&value)
            ),
        };
        let Some(selector) = node.message_loc() else {
            return;
        };
        let offense = selector.start_offset()..parent.location().end_offset();
        add_offense!(self, offense.clone(), message: format!("Use `{replacement}` instead."), |corrector| {
            corrector.replace(offense, replacement);
        });
    }
}

fn simple_method_argument(node: &CallNode<'_>) -> bool {
    if node.block().is_some() || argument_count(node) > 1 {
        return false;
    }
    node.first_argument().is_none_or(|argument| {
        argument.as_splat_node().is_none()
            && argument.as_block_argument_node().is_none()
            && argument.as_hash_node().is_none()
            && argument.as_keyword_hash_node().is_none()
    })
}
