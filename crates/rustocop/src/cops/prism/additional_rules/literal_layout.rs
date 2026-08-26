use super::*;

pub(super) fn empty_heredoc(source: &str, reporter: &mut Reporter<'_>) {
    for (offset, line) in source_lines(source) {
        let Some(marker) = line.find("<<") else {
            continue;
        };
        let token = line[marker + 2..].trim_start_matches(['~', '-']);
        let identifier = token
            .bytes()
            .take_while(|byte| identifier_byte(*byte))
            .count();
        if identifier == 0 {
            continue;
        }
        let label = &token[..identifier];
        let header_end = marker
            + 2
            + usize::from(
                line.as_bytes()
                    .get(marker + 2)
                    .is_some_and(|b| matches!(b, b'~' | b'-')),
            )
            + identifier;
        let body_start = offset + line.len() + 1;
        let closing_line = source[body_start..].lines().next().unwrap_or_default();
        if closing_line.trim() == label {
            let full_end = body_start
                + closing_line.len()
                + usize::from(
                    source.as_bytes().get(body_start + closing_line.len()) == Some(&b'\n'),
                );
            let quotes = if reporter.related_config_value("Style/StringLiterals", "EnforcedStyle")
                == Some("double_quotes")
            {
                "\"\""
            } else {
                "''"
            };
            let replacement = format!("{quotes}{}\n", &line[header_end..]);
            reporter.replace(
                "Use an empty string literal instead of heredoc.",
                offset + marker..offset + header_end,
                offset + marker..full_end,
                replacement,
            );
        }
    }
}

pub(super) fn space_after_method_name(source: &str, reporter: &mut Reporter<'_>) {
    #[derive(Default)]
    struct SpaceAfterMethodNameVisitor {
        offenses: Vec<std::ops::Range<usize>>,
    }
    impl<'pr> Visit<'pr> for SpaceAfterMethodNameVisitor {
        fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
            if let Some(opening) = node.lparen_loc() {
                let name = node.name_loc();
                if name.end_offset() < opening.start_offset() {
                    self.offenses
                        .push(name.end_offset()..opening.start_offset());
                }
            }
            ruby_prism::visit_def_node(self, node);
        }
    }

    let parsed = ruby_prism::parse(source.as_bytes());
    let mut visitor = SpaceAfterMethodNameVisitor::default();
    visitor.visit(&parsed.node());
    for range in visitor.offenses {
        reporter.remove(
            "Do not put a space between a method name and the opening parenthesis.",
            range.clone(),
            range,
        );
    }
}
