use ruby_prism::RescueNode;

use super::*;

define_rule!(RescueStandardErrorRule);

const MSG_IMPLICIT: &str = "Omit the error class when rescuing `StandardError` by itself.";
const MSG_EXPLICIT: &str = "Avoid rescuing without specifying an error class.";

define_cops! {
    RescueStandardError => "Style/RescueStandardError" => compatibility_prism_node_rule(as_rescue_node, RescueStandardErrorRule, on_resbody),
}

impl RescueStandardErrorRule<'_, '_, '_> {
    fn on_resbody(&mut self, node: &RescueNode<'_>) {
        match self.policy().enforced_style("explicit") {
            "implicit" => {
                if let Some(error) = rescue_standard_error(node) {
                    self.offense_for_implicit_enforced_style(node, &error);
                }
            }
            "explicit" if node.exceptions().is_empty() => {
                self.offense_for_explicit_enforced_style(node);
            }
            _ => {}
        }
    }

    fn offense_for_implicit_enforced_style(&mut self, node: &RescueNode<'_>, error: &Node<'_>) {
        let keyword = node.keyword_loc();
        let offense = keyword.start_offset()..error.location().end_offset();
        let removal = keyword.end_offset()..error.location().end_offset();
        add_offense!(self, offense, message: MSG_IMPLICIT, |corrector| {
            corrector.remove(removal);
        });
    }

    fn offense_for_explicit_enforced_style(&mut self, node: &RescueNode<'_>) {
        let keyword = node.keyword_loc();
        let insertion = keyword.end_offset();
        add_offense!(self, keyword, message: MSG_EXPLICIT, |corrector| {
            corrector.replace(insertion..insertion, " StandardError");
        });
    }
}

def_node_matcher! {
    fn rescue_standard_error<'pr>(node: &RescueNode<'pr>) -> Option<Node<'pr>> {
        let mut exceptions = node.exceptions().iter();
        let error = exceptions.next()?;
        if exceptions.next().is_some() || !node_is_root_constant(&error, b"StandardError") {
            return None;
        }
        Some(error)
    }
}
