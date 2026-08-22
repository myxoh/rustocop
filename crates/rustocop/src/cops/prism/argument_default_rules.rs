use ruby_prism::{CallNode, OptionalParameterNode};

use super::*;

define_rule!(RedundantCurrentDirectoryInPathRule);
define_rule!(RedundantArgumentRule);
define_rule!(OptionHashRule);

define_cops! {
    RedundantCurrentDirectoryInPath => "Style/RedundantCurrentDirectoryInPath" => call_rule(
        RedundantCurrentDirectoryInPathRule,
        on_send,
        restrict [b"require_relative"]
    ),
    RedundantArgument => "Style/RedundantArgument" => call_rule(
        RedundantArgumentRule,
        on_send
    ),
    OptionHash => "Style/OptionHash" => node_rule(
        as_optional_parameter_node,
        OptionHashRule,
        on_optarg
    ),
}

impl RedundantCurrentDirectoryInPathRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some(argument) = first_argument(node) else {
            return;
        };
        let Some(string) = argument.as_string_node() else {
            return;
        };
        let path = string.unescaped();
        return_unless!(path.starts_with(b"./"));
        let offense_length = one_current_directory_prefix(path).expect("checked");
        let mut redundant_length = offense_length;
        while let Some(length) = one_current_directory_prefix(&path[redundant_length..]) {
            redundant_length += length;
        }
        let source = self.source_file().node(&argument);
        let Some(source_index) = source.find("./") else {
            return;
        };
        let start = argument.location().start_offset() + source_index;
        add_offense!(self, start..start + offense_length, message: "Remove the redundant current directory path.", |corrector| {
            corrector.remove(start..start + redundant_length);
        });
    }
}

fn one_current_directory_prefix(path: &[u8]) -> Option<usize> {
    path.starts_with(b"./")
        .then(|| 1 + path[1..].iter().take_while(|byte| **byte == b'/').count())
}

impl RedundantArgumentRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let name = String::from_utf8_lossy(node.name().as_slice());
        return_if!(node.receiver().is_none() && !matches!(name.as_ref(), "exit" | "exit!"));
        let Some(argument) = only_argument(node) else {
            return;
        };
        let Some(default) = redundant_argument_default(self, name.as_ref()) else {
            return;
        };
        let invalid_byte_default = default.lines().any(|field| {
            field
                .split_once('=')
                .is_some_and(|(key, value)| key.trim_matches('"') == "$hex" && value == "82")
        });
        return_unless!(argument_matches_default(
            &argument,
            default,
            invalid_byte_default && name == "chomp"
        ));
        let Some(selector) = node.message_loc() else {
            return;
        };
        let argument_source = self.source_file().node(&argument);
        let offense = selector.end_offset()..node.location().end_offset();
        let message =
            format!("Argument {argument_source} is redundant because it is implied by default.");
        add_offense!(self, offense.clone(), message: message, |corrector| {
            corrector.remove(offense);
        });
    }
}

fn redundant_argument_default<'a>(
    context: &'a CopContext<'_, '_>,
    method: &str,
) -> Option<&'a str> {
    if context.config_contains("Methods") {
        return context
            .config_map("Methods")?
            .get(method)
            .map(String::as_str);
    }
    match method {
        "join" => Some(""),
        "sum" => Some("0"),
        "exit" => Some("true"),
        "exit!" => Some("false"),
        "to_i" => Some("10"),
        "split" => Some(" "),
        "chomp" | "chomp!" => Some("\n"),
        _ => None,
    }
}

fn argument_matches_default(
    argument: &Node<'_>,
    default: &str,
    invalid_byte_default: bool,
) -> bool {
    if let Some(string) = argument.as_string_node() {
        if invalid_byte_default {
            return string.unescaped() == [0x82];
        }
        let decoded = if default == r"\n" {
            b"\n".as_slice()
        } else {
            default.as_bytes()
        };
        return string.unescaped() == decoded;
    }
    if let Some(integer) = argument.as_integer_node() {
        return default.parse::<i32>().ok().is_some_and(|expected| {
            TryInto::<i32>::try_into(integer.value()).ok() == Some(expected)
        });
    }
    argument.as_true_node().is_some() && default == "true"
        || argument.as_false_node().is_some() && default == "false"
}

impl OptionHashRule<'_, '_, '_> {
    fn on_optarg(&mut self, node: &OptionalParameterNode<'_>) {
        return_unless!(node
            .value()
            .as_hash_node()
            .is_some_and(|hash| hash.elements().is_empty()));
        let name = String::from_utf8_lossy(node.name().as_slice());
        let suspicious = if self.config_contains("SuspiciousParamNames") {
            self.config_values("SuspiciousParamNames")
                .iter()
                .any(|candidate| candidate == name.as_ref())
        } else {
            matches!(
                name.as_ref(),
                "options" | "opts" | "args" | "params" | "parameters"
            )
        };
        return_unless!(suspicious);
        let definition = self.ancestors().iter().rev().find_map(Node::as_def_node);
        let block = self.ancestors().iter().rev().find_map(Node::as_block_node);
        let parameters = definition
            .as_ref()
            .and_then(|definition| definition.parameters())
            .or_else(|| {
                block
                    .as_ref()
                    .and_then(|block| block.parameters())
                    .and_then(|parameters| parameters.as_block_parameters_node())
                    .and_then(|parameters| parameters.parameters())
            });
        let Some(parameters) = parameters else { return };
        let last_optional = parameters.optionals().iter().last();
        return_unless!(last_optional.is_some_and(|optional| {
            optional.location().start_offset() == node.location().start_offset()
                && optional.location().end_offset() == node.location().end_offset()
        }));
        return_if!(
            parameters.rest().is_some()
                || !parameters.posts().is_empty()
                || !parameters.keywords().is_empty()
                || parameters.keyword_rest().is_some()
                || parameters.block().is_some()
        );
        let method_name = definition
            .as_ref()
            .map(|definition| definition.name().as_slice())
            .or_else(|| {
                self.ancestors()
                    .iter()
                    .rev()
                    .find_map(Node::as_call_node)
                    .map(|call| call.name().as_slice())
            });
        let Some(method_name) = method_name else {
            return;
        };
        let method = String::from_utf8_lossy(method_name);
        return_if!(self
            .config_values("Allowlist")
            .iter()
            .any(|allowed| allowed == method.as_ref()));
        let mut forwarding_super = ForwardingSuperFinder(false);
        if let Some(definition) = definition {
            if let Some(body) = definition.body() {
                forwarding_super.visit(&body);
            }
        } else if let Some(block) = block {
            if let Some(body) = block.body() {
                forwarding_super.visit(&body);
            }
        }
        return_if!(forwarding_super.0);
        self.report(
            "Prefer keyword arguments to options hashes.",
            node.location(),
        );
    }
}

struct ForwardingSuperFinder(bool);

impl<'pr> Visit<'pr> for ForwardingSuperFinder {
    fn visit_forwarding_super_node(&mut self, _node: &ruby_prism::ForwardingSuperNode<'pr>) {
        self.0 = true;
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
}

#[cfg(test)]
mod tests {
    use super::one_current_directory_prefix;

    #[test]
    fn measures_one_current_directory_component() {
        assert_eq!(one_current_directory_prefix(b"./path"), Some(2));
        assert_eq!(one_current_directory_prefix(b".///./../path"), Some(4));
        assert_eq!(one_current_directory_prefix(b"../path"), None);
    }
}
