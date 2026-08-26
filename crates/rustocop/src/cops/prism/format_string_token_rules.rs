use ruby_prism::{Node, StringNode};

use super::*;

define_cops! {
    FormatStringToken => "Style/FormatStringToken" => rubocop_callbacks(FormatStringTokenRule, [on_str]),
}

#[derive(Clone, Copy, PartialEq)]
enum TokenStyle { Annotated, Template, Unannotated }

struct Token {
    style: TokenStyle,
    start: usize,
    end: usize,
    name: Option<(usize, usize)>,
    kind: u8,
}

impl FormatStringTokenRule<'_, '_, '_> {
    fn on_str(&mut self, node: &StringNode<'_>) {
        return_if!(self.ancestors().iter().any(|ancestor| ancestor.as_regular_expression_node().is_some() || ancestor.as_x_string_node().is_some()));
        let content = node.content_loc();
        let source = self.source_file().at(&content);
        return_if!(!source.contains('%') || self.source_file().node(&node.as_node()) == "__FILE__");
        let tokens = parse_tokens(source);
        return_if!(tokens.is_empty());
        let location = (node.location().start_offset(), node.location().end_offset());
        let typical = typical_context(location, self.ancestors());
        let directly_typical = direct_typical_context(location, self.ancestors());
        return_if!(allowed_context(self, location));
        let target = match self.policy().enforced_style("annotated") {
            "template" => TokenStyle::Template,
            "unannotated" => TokenStyle::Unannotated,
            _ => TokenStyle::Annotated,
        };
        let conservative = self.config_value("Mode").is_some_and(|mode| mode == "conservative");
        let nonstandard_delimiter = node.opening_loc().is_some_and(|opening| {
            opening.as_slice().starts_with(b"%") || opening.as_slice().starts_with(b"<<")
        });
        let nested_in_interpolation = self
            .ancestors()
            .iter()
            .any(|ancestor| ancestor.as_interpolated_string_node().is_some());
        let prism_opaque_unannotated = self.related_config_value("AllCops", "ParserEngine")
            == Some("parser_prism")
            && (nonstandard_delimiter || nested_in_interpolation && !directly_typical);
        return_if!(conservative && !typical);
        let max = self.config_usize("MaxUnannotatedPlaceholdersAllowed", 1);
        return_if!(tokens.iter().all(|token| token.style == TokenStyle::Unannotated) && tokens.len() <= max);

        for token in tokens {
            if token.style == target
                || (token.style == TokenStyle::Unannotated
                    && (!typical || prism_opaque_unannotated))
            {
                continue;
            }
            return_if!(target == TokenStyle::Template && token.style != TokenStyle::Template && token.kind != b's');
            let offense = content.start_offset() + token.start..content.start_offset() + token.end;
            let message = format!("Prefer {} over {}.", label(target), label(token.style));
            let correction = if typical {
                correction(source, &token, target)
            } else {
                None
            };
            if let Some(replacement) = correction {
                add_offense!(self, offense.clone(), message: message, |corrector| { corrector.replace(offense, replacement); });
            } else {
                self.report(message, offense);
            }
        }
    }
}

fn parse_tokens(source: &str) -> Vec<Token> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] != b'%' { index += 1; continue; }
        if bytes[index + 1] == b'%' { index += 2; continue; }
        if bytes[index + 1] == b'#' && bytes.get(index + 2) == Some(&b'{') {
            index += 2;
            continue;
        }
        let mut opening = index + 1;
        let mut interpolation = false;
        while opening < bytes.len()
            && matches!(bytes[opening], b'#' | b'0' | b'-' | b'+' | b' ' | b'1'..=b'9' | b'.' | b'*')
        {
            if bytes[opening] == b'#' && bytes.get(opening + 1) == Some(&b'{') {
                interpolation = true;
                break;
            }
            opening += 1;
        }
        if interpolation {
            index += 1;
            continue;
        }
        if bytes.get(opening) == Some(&b'<') {
            let Some(close) = bytes[opening + 1..].iter().position(|byte| *byte == b'>').map(|at| opening + 1 + at) else { index += 1; continue };
            let mut end = close + 1;
            while end < bytes.len() && !bytes[end].is_ascii_alphabetic() { end += 1; }
            if end < bytes.len() {
                tokens.push(Token { style: TokenStyle::Annotated, start: index, end: end + 1, name: Some((opening + 1, close)), kind: bytes[end] });
                index = end + 1;
                continue;
            }
        } else {
            if bytes.get(opening) == Some(&b'{') {
                if let Some(close) = bytes[opening + 1..].iter().position(|byte| *byte == b'}').map(|at| opening + 1 + at) {
                    let name = &bytes[opening + 1..close];
                    if !name.is_empty()
                        && name
                            .iter()
                            .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
                    {
                        tokens.push(Token { style: TokenStyle::Template, start: index, end: close + 1, name: Some((opening + 1, close)), kind: b's' });
                        index = close + 1;
                        continue;
                    }
                }
            }
            {
                let mut end = index + 1;
                while end < bytes.len() && !bytes[end].is_ascii_alphabetic() && bytes[end] != b'%' { end += 1; }
                if end < bytes.len() && format_type(bytes[end]) {
                    tokens.push(Token { style: TokenStyle::Unannotated, start: index, end: end + 1, name: None, kind: bytes[end] });
                    index = end + 1;
                    continue;
                }
            }
        }
        index += 1;
    }
    tokens
}

fn format_type(byte: u8) -> bool { matches!(byte, b'A'|b'B'|b'E'|b'G'|b'X'|b'a'|b'b'|b'c'|b'd'|b'e'|b'f'|b'g'|b'i'|b'o'|b'p'|b's'|b'u'|b'x') }

fn label(style: TokenStyle) -> &'static str {
    match style {
        TokenStyle::Annotated => "annotated tokens (like `%<foo>s`)",
        TokenStyle::Template => "template tokens (like `%{foo}`)",
        TokenStyle::Unannotated => "unannotated tokens (like `%s`)",
    }
}

fn correction(source: &str, token: &Token, target: TokenStyle) -> Option<String> {
    let (name_start, name_end) = token.name?;
    let name = &source[name_start..name_end];
    match target {
        TokenStyle::Annotated => Some(format!("%<{name}>s")),
        TokenStyle::Template => Some(format!("%{{{name}}}")),
        TokenStyle::Unannotated => None,
    }
}

fn typical_context(location: (usize, usize), ancestors: &[Node<'_>]) -> bool {
    let mut subjects = vec![location];
    subjects.extend(ancestors
        .iter()
        .filter(|ancestor| ancestor.as_interpolated_string_node().is_some())
        .map(|ancestor| {
            (ancestor.location().start_offset(), ancestor.location().end_offset())
        }));
    ancestors.iter().filter_map(Node::as_call_node).any(|call| {
        if matches!(call.name().as_slice(), b"format" | b"sprintf" | b"printf") {
            call.arguments().and_then(|args| args.arguments().iter().next()).is_some_and(|argument| subjects.iter().any(|subject| same_location(argument.location(), *subject)))
        } else if call.name().as_slice() == b"%" {
            call.receiver().is_some_and(|receiver| subjects.iter().any(|subject| same_location(receiver.location(), *subject)))
        } else { false }
    })
}

fn direct_typical_context(location: (usize, usize), ancestors: &[Node<'_>]) -> bool {
    ancestors.iter().filter_map(Node::as_call_node).any(|call| {
        if matches!(call.name().as_slice(), b"format" | b"sprintf" | b"printf") {
            call.arguments()
                .and_then(|args| args.arguments().iter().next())
                .is_some_and(|argument| same_location(argument.location(), location))
        } else if call.name().as_slice() == b"%" {
            call.receiver()
                .is_some_and(|receiver| same_location(receiver.location(), location))
        } else {
            false
        }
    })
}

fn same_location(location: ruby_prism::Location<'_>, expected: (usize, usize)) -> bool {
    location.start_offset() == expected.0 && location.end_offset() == expected.1
}

fn allowed_context(context: &FormatStringTokenRule<'_, '_, '_>, location: (usize, usize)) -> bool {
    context
        .ancestors()
        .iter()
        .filter_map(Node::as_call_node)
        .filter(|call| contains(call.location(), location))
        .min_by_key(|call| call.location().end_offset() - call.location().start_offset())
        .is_some_and(|call| context.policy().allows_method(call.name().as_slice()))
}

fn contains(outer: ruby_prism::Location<'_>, inner: (usize, usize)) -> bool {
    outer.start_offset() <= inner.0 && inner.1 <= outer.end_offset()
}
