use super::*;
use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::cop::mixin::range_help::{Side, SurroundingSpace};

define_compatibility_rule!(TrailingCommaInAttributeDeclarationRule);
define_compatibility_callback_rule_cop!(
    TrailingCommaInAttributeDeclaration
        => "Lint/TrailingCommaInAttributeDeclaration"
        => TrailingCommaInAttributeDeclarationRule
        [on_send restrict ["attr_reader", "attr_writer", "attr_accessor", "attr"]]
);
declare_cops!(TrailingCommaInAttributeDeclaration);

const MSG: &str = "Avoid leaving a trailing comma in attribute declarations.";

impl TrailingCommaInAttributeDeclarationRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        return_unless!(
            node.attribute_accessor()
                && node
                    .last_argument()
                    .is_some_and(|argument| argument.kind() == "def")
        );

        let Some(trailing_comma) = self.trailing_comma_range(node) else {
            return;
        };

        add_offense!(self, trailing_comma.clone(), message: MSG, |corrector| {
            corrector.remove(trailing_comma);
        });
    }

    fn trailing_comma_range(
        &self,
        node: NodeRef<'_>,
    ) -> Option<CompatibilitySourceRange> {
        let arguments = node.arguments();
        let penultimate = arguments.get(arguments.len().checked_sub(2)?)?;
        Some(self.owned_range(
            self.range_help()
                .range_with_surrounding_space(
                    self.source_range(*penultimate)?,
                    SurroundingSpace {
                        side: Side::Right,
                        ..SurroundingSpace::default()
                    },
                )
                .end()
                .resize(1),
        ))
    }
}
