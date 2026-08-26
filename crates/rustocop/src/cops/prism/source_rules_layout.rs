use super::*;

declare_source_cops! {
    EmptyLines => "Layout/EmptyLines" => empty_lines,
    SpaceBeforeComment => "Layout/SpaceBeforeComment" => space_before_comment,
    SpaceAfterSemicolon => "Layout/SpaceAfterSemicolon" => space_after_semicolon,
    SpaceAfterComma => "Layout/SpaceAfterComma" => space_after_comma,
    SpaceBeforeSemicolon => "Layout/SpaceBeforeSemicolon" => space_before_semicolon,
    SpaceBeforeComma => "Layout/SpaceBeforeComma" => space_before_comma,
}

fn empty_lines(source: &str, context: &mut Reporter<'_>) {
    let literals = SourceFile::new(source).literal_ranges();
    let ruby_end = source
        .lines()
        .scan(0usize, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line))
        })
        .find_map(|(offset, line)| (line.trim() == "__END__").then_some(offset))
        .unwrap_or(source.len());
    let content_end = source[..ruby_end]
        .rfind(|character: char| !character.is_whitespace())
        .map_or(0, |offset| offset + 1);
    for (start, window) in source.as_bytes()[..content_end].windows(3).enumerate() {
        if window == b"\n\n\n"
            && !literals
                .iter()
                .any(|range| range.start <= start + 2 && start + 2 < range.end)
        {
            context.remove(
                "Extra blank line detected.",
                start + 2..start + 3,
                start + 2..start + 3,
            );
        }
    }
}

fn space_before_comment(source: &str, context: &mut Reporter<'_>) {
    let parsed = ruby_prism::parse(source.as_bytes());
    for comment in parsed.comments() {
        if comment.type_() != ruby_prism::CommentType::InlineComment {
            continue;
        }
        let location = comment.location();
        let hash = location.start_offset();
        let line_start = source[..hash].rfind('\n').map_or(0, |at| at + 1);
        if hash == line_start || source.as_bytes()[hash - 1].is_ascii_whitespace() {
            continue;
        }
        context.insert(
            "Put a space before an end-of-line comment.",
            hash..location.end_offset(),
            hash,
            " ",
        );
    }
}

fn space_after_semicolon(source: &str, context: &mut Reporter<'_>) {
    spacing_after(source, context, b';', "Space missing after semicolon.");
}

fn space_after_comma(source: &str, context: &mut Reporter<'_>) {
    spacing_after(source, context, b',', "Space missing after comma.");
}

fn spacing_after(source: &str, context: &mut Reporter<'_>, token: u8, message: &'static str) {
    use crate::rubocop::ast::processed_source::SourceToken;
    use crate::rubocop::cop::mixin::space_after_punctuation::SpaceAfterPunctuation;

    let bytes = source.as_bytes();
    let ignored = ignored_syntax_ranges(source);
    let interpolation_closings = interpolation_closing_offsets(source);
    let punctuation_with_space = SpaceAfterPunctuation {
        space_style_before_rcurly: "space".to_string(),
    };
    let punctuation_without_space = SpaceAfterPunctuation {
        space_style_before_rcurly: "no_space".to_string(),
    };
    for index in 0..bytes.len() {
        if bytes[index] != token {
            continue;
        }
        let Some(next) = bytes.get(index + 1).copied() else {
            continue;
        };
        let no_space_inside_braces = token == b';'
            && next == b'}'
            && context.related_config_value("Layout/SpaceInsideBlockBraces", "EnforcedStyle")
                == Some("no_space");
        let closing_brace_requires_space = next == b'}'
            && ((token == b','
                && context.related_config_value(
                    "Layout/SpaceInsideHashLiteralBraces",
                    "EnforcedStyle",
                ) == Some("space"))
                || (token == b';'
                    && (bytes.get(index.wrapping_sub(1)) == Some(&b' ')
                        || context.related_config_value(
                            "Layout/SpaceInsideBlockBraces",
                            "EnforcedStyle",
                        ) == Some("space"))));
        let punctuation = if closing_brace_requires_space {
            &punctuation_with_space
        } else {
            &punctuation_without_space
        };
        let current_token = SourceToken {
            kind: if token == b',' { "tCOMMA" } else { "tSEMI" },
            text: String::new(),
            range: index..index + 1,
            line: 1,
            column: index,
        };
        let next_token = SourceToken {
            kind: match next {
                b')' => "tRPAREN",
                b']' => "tRBRACK",
                b'}' => "tRCURLY",
                b'|' => "tPIPE",
                _ => "tIDENTIFIER",
            },
            text: String::new(),
            range: index + 1..index + 2,
            line: 1,
            column: index + 1,
        };
        if next == b'\n'
            || next == b' '
            || next == token
            || (token == b';' && next == b'}' && interpolation_closings.contains(&(index + 1)))
            || no_space_inside_braces
            || !punctuation.space_missing(&current_token, &next_token)
            || !punctuation.space_required_before(&next_token)
            || ignored.iter().any(|range| range.start <= index && index < range.end)
        {
            continue;
        }
        context.insert(message, index..index + 1, index + 1, " ");
    }
}

fn interpolation_closing_offsets(source: &str) -> Vec<usize> {
    #[derive(Default)]
    struct Closings(Vec<usize>);
    impl<'pr> Visit<'pr> for Closings {
        fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode<'pr>) {
            self.0.push(node.closing_loc().start_offset());
            ruby_prism::visit_embedded_statements_node(self, node);
        }
    }
    let parsed = ruby_prism::parse(source.as_bytes());
    let mut closings = Closings::default();
    closings.visit(&parsed.node());
    closings.0
}

fn space_before_semicolon(source: &str, context: &mut Reporter<'_>) {
    spacing_before(source, context, b';', "Space found before semicolon.");
}

fn space_before_comma(source: &str, context: &mut Reporter<'_>) {
    spacing_before(source, context, b',', "Space found before comma.");
}

fn spacing_before(source: &str, context: &mut Reporter<'_>, token: u8, message: &'static str) {
    let bytes = source.as_bytes();
    let ignored = ignored_syntax_ranges(source);
    for index in 1..bytes.len() {
        if bytes[index] != token
            || bytes[index - 1] != b' '
            || ignored.iter().any(|range| range.start <= index && index < range.end)
        {
            continue;
        }
        let line_start = source[..index].rfind('\n').map_or(0, |offset| offset + 1);
        if source[line_start..index].trim().is_empty() {
            continue;
        }
        let start = source[..index].trim_end_matches(' ').len();
        if token == b';'
            && source.as_bytes().get(start.wrapping_sub(1)) == Some(&b'{')
            && context.related_config_value("Layout/SpaceInsideBlockBraces", "EnforcedStyle")
                == Some("space")
        {
            continue;
        }
        context.remove(message, start..index, start..index);
    }
}

fn ignored_syntax_ranges(source: &str) -> Vec<std::ops::Range<usize>> {
    #[derive(Default)]
    struct EmbeddedRuby {
        ranges: Vec<std::ops::Range<usize>>,
        opaque_interpolated_strings: Vec<std::ops::Range<usize>>,
    }
    impl<'pr> Visit<'pr> for EmbeddedRuby {
        fn visit_interpolated_string_node(&mut self, node: &ruby_prism::InterpolatedStringNode<'pr>) {
            if node.opening_loc().is_none() {
                self.opaque_interpolated_strings
                    .push(node.location().start_offset()..node.location().end_offset());
            }
            ruby_prism::visit_interpolated_string_node(self, node);
        }

        fn visit_embedded_statements_node(&mut self, node: &ruby_prism::EmbeddedStatementsNode<'pr>) {
            self.ranges.push(node.location().start_offset()..node.location().end_offset());
            ruby_prism::visit_embedded_statements_node(self, node);
        }

        fn visit_embedded_variable_node(&mut self, node: &ruby_prism::EmbeddedVariableNode<'pr>) {
            self.ranges.push(node.location().start_offset()..node.location().end_offset());
            ruby_prism::visit_embedded_variable_node(self, node);
        }
    }

    let file = SourceFile::new(source);
    let parsed = ruby_prism::parse(source.as_bytes());
    let mut embedded = EmbeddedRuby::default();
    embedded.visit(&parsed.node());
    let mut ranges = file.literal_ranges();
    let heredocs = file.heredoc_ranges();
    for range in &mut ranges {
        if heredocs.contains(range) {
            range.start = file.line_end(range.start).saturating_add(1).min(range.end);
        }
    }
    ranges.extend(lexical_heredoc_body_ranges(source));
    for ruby in embedded.ranges {
        if embedded
            .opaque_interpolated_strings
            .iter()
            .any(|literal| {
                literal.start <= ruby.start
                    && ruby.end <= literal.end
                    && !matches!(source.as_bytes().get(literal.start), Some(b'\'' | b'"' | b'`'))
            })
        {
            continue;
        }
        ranges = ranges
            .into_iter()
            .flat_map(|range| {
                if !(range.start <= ruby.start && ruby.end < range.end) {
                    vec![range]
                } else {
                    let mut pieces = Vec::new();
                    if range.start < ruby.start {
                        pieces.push(range.start..ruby.start);
                    }
                    if ruby.end < range.end {
                        pieces.push(ruby.end..range.end);
                    }
                    pieces
                }
            })
            .collect();
    }
    ranges.extend(file.comment_ranges());
    if let Some(start) = file.data_section_start() {
        ranges.push(start..source.len());
    }
    ranges
}

pub(super) fn lexical_heredoc_body_ranges(source: &str) -> Vec<std::ops::Range<usize>> {
    let lines = SourceFile::new(source).lines().collect::<Vec<_>>();
    let mut ranges = Vec::new();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let mut cursor = 0;
        while let Some(relative_marker) = line[cursor..].find("<<") {
            let marker = cursor + relative_marker;
            if !outside_line_quotes(line, marker) {
                cursor = marker + 2;
                continue;
            }
            let marker_tail = &line[marker + 2..];
            let indentation = marker_tail.len() - marker_tail.trim_start_matches(['-', '~']).len();
            let marker_tail = &marker_tail[indentation..];
            let whitespace = marker_tail.len() - marker_tail.trim_start().len();
            if whitespace > 0 {
                cursor = marker + 2 + indentation + whitespace;
                continue;
            }
            let mut tail = &marker_tail[whitespace..];
            let label_start = offset + marker + 2 + indentation + whitespace;
            let (label, consumed) = if tail.starts_with(['\'', '"', '`']) {
                let quote = tail.as_bytes()[0] as char;
                tail = &tail[1..];
                let Some(end) = tail.find(quote) else { break };
                ranges.push(label_start..label_start + end + 2);
                (&tail[..end], end + 2)
            } else {
                let end = tail
                    .find(|character: char| !(character.is_alphanumeric() || character == '_'))
                    .unwrap_or(tail.len());
                (&tail[..end], end)
            };
            cursor = marker + 2 + indentation + whitespace + consumed;
            if label.is_empty() {
                continue;
            }
            let Some((closing_offset, closing_line)) = lines[index + 1..]
                .iter()
                .copied()
                .find(|(_, candidate)| candidate.trim() == label)
            else {
                continue;
            };
            let body_start = offset + line.len()
                + usize::from(source.as_bytes().get(offset + line.len()) == Some(&b'\n'));
            ranges.push(body_start..closing_offset + closing_line.len());
        }
    }
    ranges
}

fn outside_line_quotes(line: &str, end: usize) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for byte in line.as_bytes()[..end].iter().copied() {
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if quote == Some(byte) {
            quote = None;
        } else if quote.is_none() && matches!(byte, b'\'' | b'"' | b'`') {
            quote = Some(byte);
        }
    }
    quote.is_none()
}
