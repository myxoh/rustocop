use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![Box::new(SpaceAfterColon)]
}

struct SpaceAfterColon;

impl Cop for SpaceAfterColon {
    fn name(&self) -> &'static str {
        "Layout/SpaceAfterColon"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        if let Some(association) = node.as_assoc_node() {
            if let Some(colon) = association
                .operator_loc()
                .filter(|operator| operator.as_slice() == b":")
            {
                self.check_colon(source, colon.start_offset(), context);
            } else if let Some(key) = association.key().as_symbol_node() {
                self.check_colon(
                    source,
                    key.location().end_offset().saturating_sub(1),
                    context,
                );
            }
        } else if let Some(parameter) = node.as_optional_keyword_parameter_node() {
            self.check_colon(
                source,
                parameter.name_loc().end_offset().saturating_sub(1),
                context,
            );
        }
    }
}

impl SpaceAfterColon {
    fn check_colon(&self, source: &str, offset: usize, context: &mut Context) {
        if source.as_bytes().get(offset) != Some(&b':') {
            return;
        }
        let Some(next) = source.as_bytes().get(offset + 1) else {
            return;
        };
        if next.is_ascii_whitespace() || matches!(next, b',' | b'}' | b')' | b']') {
            return;
        }

        context.add_offense_offsets(
            self.name(),
            "Space missing after colon.".to_string(),
            offset,
            offset + 1,
            Some((offset + 1, offset + 1, " ".to_string())),
        );
    }
}
