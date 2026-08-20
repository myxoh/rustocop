use ruby_prism::DefNode;

use super::*;

define_cops! {
    MethodDefParentheses => "Style/MethodDefParentheses" => rubocop_callbacks(MethodDefParenthesesRule, [on_def]),
}

impl MethodDefParenthesesRule<'_, '_, '_> {
    fn on_def(&mut self, node: &DefNode<'_>) {
        let parameters = node.parameters();
        let parenthesized = node.lparen_loc().is_some() && node.rparen_loc().is_some();
        let multiline = parameters.as_ref().is_some_and(|parameters| {
            self.source_file().node(&parameters.as_node()).contains('\n')
        });
        let style = self.policy().enforced_style("require_parentheses");
        let require_parentheses = style == "require_parentheses"
            || style == "require_no_parentheses_except_multiline" && multiline;

        if require_parentheses {
            let Some(parameters) = parameters else { return };
            return_if!(parenthesized);
            let range = parameters.location().start_offset()..parameters.location().end_offset();
            let gap = node.name_loc().end_offset()..range.start;
            add_offense!(self, range.clone(), message: "Use def with parentheses when there are parameters.", |corrector| {
                corrector.remove(gap);
                corrector.replace(range.start..range.start, "(");
                corrector.replace(range.end..range.end, ")");
            });
            return;
        }

        return_unless!(parenthesized);
        return_if!(node.equal_loc().is_some() || forced_parentheses(node, self.source_file()));
        let left = node.lparen_loc().expect("checked parenthesized");
        let right = node.rparen_loc().expect("checked parenthesized");
        let offense = left.start_offset()..right.end_offset();
        add_offense!(self, offense, message: "Use def without parentheses.", |corrector| {
            corrector.replace(left, " ");
            corrector.remove(right);
        });
    }
}

fn forced_parentheses(node: &DefNode<'_>, file: SourceFile<'_>) -> bool {
    let Some(parameters) = node.parameters() else { return false };
    file.node(&parameters.as_node())
        .split(',')
        .map(str::trim)
        .any(|parameter| matches!(parameter, "..." | "*" | "**" | "&"))
}
