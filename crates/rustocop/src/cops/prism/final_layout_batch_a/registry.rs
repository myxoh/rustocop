use super::super::catalog_cop::{custom, replace};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(EmptyLineAfterGuardClause),
        custom(
            "Layout/LineEndStringConcatenationIndentation",
            align_continuation,
        ),
        Box::new(SpaceInsideBlockBraces),
        custom(
            "Layout/SpaceInsideHashLiteralBraces",
            space_inside_hash_literal_braces,
        ),
        replace(
            "Layout/LineContinuationLeadingSpace",
            "\n .",
            "\n.",
            "Line continuation should not have leading space.",
        ),
    ]
}

fn space_inside_hash_literal_braces(context: &mut CopContext<'_, '_>) {
    #[derive(Default)]
    struct HashBraces(Vec<(usize, usize)>);

    impl<'pr> Visit<'pr> for HashBraces {
        fn visit_hash_node(&mut self, node: &ruby_prism::HashNode<'pr>) {
            self.0.push((
                node.opening_loc().start_offset(),
                node.closing_loc().start_offset(),
            ));
            ruby_prism::visit_hash_node(self, node);
        }

        fn visit_hash_pattern_node(&mut self, node: &ruby_prism::HashPatternNode<'pr>) {
            if let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) {
                if opening.as_slice() == b"{" {
                    self.0
                        .push((opening.start_offset(), closing.start_offset()));
                }
            }
            ruby_prism::visit_hash_pattern_node(self, node);
        }
    }

    let mut braces = HashBraces::default();
    braces.visit(&parse(context.source().as_bytes()).node());
    for (opening, closing) in braces.0 {
        enforce_hash_brace_spacing(context, opening, closing);
    }
}

fn enforce_hash_brace_spacing(context: &mut CopContext<'_, '_>, opening: usize, closing: usize) {
    let source = context.source();
    if opening >= closing || closing >= source.len() {
        return;
    }
    let inside = &source[opening + 1..closing];
    if inside.trim().is_empty() {
        let style = context
            .config_value("EnforcedStyleForEmptyBraces")
            .unwrap_or("no_space");
        if style == "no_space" && !inside.is_empty() {
            context.remove(
                "Space inside empty hash literal braces detected.",
                opening + 1..closing,
                opening + 1..closing,
            );
        } else if style == "space" && inside.is_empty() {
            context.insert(
                "Space inside empty hash literal braces missing.",
                opening..opening + 1,
                opening + 1,
                " ",
            );
        }
        return;
    }

    let style = context.policy().enforced_style("space");
    let left_end = opening
        + 1
        + inside
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
    let right_start = closing
        - inside
            .bytes()
            .rev()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .count();
    let left_newline = source[opening + 1..left_end].contains('\n')
        || source.as_bytes().get(opening + 1) == Some(&b'\n');
    let right_newline = source[right_start..closing].contains('\n')
        || source.as_bytes().get(closing.wrapping_sub(1)) == Some(&b'\n');
    let compact_left = style == "compact" && source.as_bytes().get(left_end) == Some(&b'{');
    let compact_right = style == "compact"
        && right_start > 0
        && source.as_bytes().get(right_start - 1) == Some(&b'}');
    let want_left = style != "no_space" && !compact_left;
    let want_right = style != "no_space" && !compact_right;

    if !left_newline && source.as_bytes().get(left_end) != Some(&b'#') {
        report_hash_side(context, opening, left_end, want_left, true);
    }
    if !right_newline {
        report_hash_side(context, right_start, closing, want_right, false);
    }
}

fn report_hash_side(
    context: &mut CopContext<'_, '_>,
    whitespace_start: usize,
    brace: usize,
    want_space: bool,
    opening: bool,
) {
    let has_space = if opening {
        brace > whitespace_start + 1
    } else {
        brace > whitespace_start
    };
    let symbol = if opening { "{" } else { "}" };
    if want_space && !has_space {
        let offense = if opening {
            whitespace_start..whitespace_start + 1
        } else {
            brace..brace + 1
        };
        context.insert(
            format!("Space inside {symbol} missing."),
            offense,
            if opening { whitespace_start + 1 } else { brace },
            " ",
        );
    } else if !want_space && has_space {
        let range = if opening {
            whitespace_start + 1..brace
        } else {
            whitespace_start..brace
        };
        context.remove(
            format!("Space inside {symbol} detected."),
            range.clone(),
            range,
        );
    }
}
