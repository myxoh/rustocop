use ruby_prism::{Node, SymbolNode};

use super::*;

define_rule!(QuotedSymbolsRule);
define_rule!(PercentLiteralDelimitersRule);

define_cops! {
    QuotedSymbols => "Style/QuotedSymbols" => node_rule(
        as_symbol_node,
        QuotedSymbolsRule,
        on_sym
    ),
    PercentLiteralDelimiters => "Style/PercentLiteralDelimiters" => node_rule_aliases(
        PercentLiteralDelimitersRule,
        on_percent_literal => [
            as_array_node,
            as_string_node,
            as_interpolated_string_node,
            as_regular_expression_node,
            as_interpolated_regular_expression_node,
            as_symbol_node,
            as_x_string_node,
            as_interpolated_x_string_node
        ]
    ),
}

impl QuotedSymbolsRule<'_, '_, '_> {
    fn on_sym(&mut self, node: &SymbolNode<'_>) {
        let source = self.source_file().at(&node.location());
        let hash_colon = (source.starts_with('"') || source.starts_with('\'')) && source.ends_with(':');
        let (prefix, quote, body) = if hash_colon {
            ("", source.as_bytes()[0], &source[1..source.len() - 2])
        } else if source.starts_with(":\"") || source.starts_with(":'") {
            (&source[..1], source.as_bytes()[1], &source[2..source.len() - 1])
        } else if source.starts_with('"') || source.starts_with('\'') {
            ("", source.as_bytes()[0], &source[1..source.len() - 1])
        } else {
            return;
        };
        return_if!(body.contains('\n'));
        let style = self.quoted_symbol_style();
        let wrong = match style {
            "double_quotes" => quote == b'\'' && safe_for_double_quotes(body),
            _ => quote == b'"' && safe_for_single_quotes(body),
        };
        return_unless!(wrong);
        let message = if style == "double_quotes" {
            "Prefer double-quoted symbols unless you need single quotes to avoid extra backslashes for escaping."
        } else {
            "Prefer single-quoted symbols when you don't need string interpolation or special symbols."
        };
        let value = String::from_utf8_lossy(node.unescaped());
        let corrected = if style == "double_quotes" {
            format!("{prefix}\"{}\"", escape_symbol(&value, b'"'))
        } else {
            format!("{prefix}'{}'", escape_symbol(&value, b'\''))
        };
        let range = node.location().start_offset()..node.location().end_offset() - usize::from(hash_colon);
        add_offense!(self, range.clone(), message: message, |corrector| {
            corrector.replace(range, corrected);
        });
    }

    fn quoted_symbol_style(&self) -> &str {
        let style = self.policy().enforced_style("same_as_string_literals");
        if style != "same_as_string_literals" { return style; }
        if self.related_config_value("Style/StringLiterals", "Enabled") == Some("false") {
            "single_quotes"
        } else {
            self.related_config_value("Style/StringLiterals", "EnforcedStyle").unwrap_or("single_quotes")
        }
    }
}

fn safe_for_single_quotes(source: &str) -> bool {
    !source.contains('\'') && !contains_double_quote_escape(source) && !source.contains("#{") && !source.contains("#@") && !source.contains("#$")
}

fn safe_for_double_quotes(source: &str) -> bool {
    !source.contains('"') && !contains_double_quote_escape(source)
        && !source.contains("#{") && !source.contains("#@") && !source.contains("#$")
}

fn contains_double_quote_escape(source: &str) -> bool {
    let bytes = source.as_bytes();
    let mut index = 0;
    while index + 1 < bytes.len() {
        if bytes[index] != b'\\' { index += 1; continue; }
        let start = index;
        while index < bytes.len() && bytes[index] == b'\\' { index += 1; }
        if (index - start) % 2 == 1 && index < bytes.len()
            && matches!(bytes[index], b'a' | b'A' | b'b' | b'c' | b'd' | b'e' | b'f' | b'k' | b'M' | b'n' | b'p' | b'r' | b's' | b'S' | b't' | b'u' | b'U' | b'x' | b'z' | b'Z' | b'0'..=b'7')
        { return true; }
    }
    false
}

fn escape_symbol(value: &str, quote: u8) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        if character == '\\' || character as u32 == quote as u32 { escaped.push('\\'); }
        escaped.push(character);
    }
    escaped
}

impl PercentLiteralDelimitersRule<'_, '_, '_> {
    fn on_percent_literal(&mut self, node: &Node<'_>) {
        let percent_opening = if let Some(array) = node.as_array_node() {
            array
                .opening_loc()
                .is_some_and(|opening| opening.as_slice().starts_with(b"%"))
        } else if let Some(string) = node.as_string_node() {
            string
                .opening_loc()
                .is_some_and(|opening| opening.as_slice().starts_with(b"%"))
        } else if let Some(string) = node.as_interpolated_string_node() {
            string
                .opening_loc()
                .is_some_and(|opening| opening.as_slice().starts_with(b"%"))
        } else if let Some(regexp) = node.as_regular_expression_node() {
            regexp.opening_loc().as_slice().starts_with(b"%")
        } else if let Some(regexp) = node.as_interpolated_regular_expression_node() {
            regexp.opening_loc().as_slice().starts_with(b"%")
        } else if let Some(symbol) = node.as_symbol_node() {
            symbol
                .opening_loc()
                .is_some_and(|opening| opening.as_slice().starts_with(b"%"))
        } else if let Some(xstring) = node.as_x_string_node() {
            xstring.opening_loc().as_slice().starts_with(b"%")
        } else if let Some(xstring) = node.as_interpolated_x_string_node() {
            xstring.opening_loc().as_slice().starts_with(b"%")
        } else {
            false
        };
        return_unless!(percent_opening);
        let source = self.source_file().node(node);
        let Some((literal_type, prefix_len)) = percent_literal_type(source) else { return };
        let bytes = source.as_bytes();
        let Some(&used_open) = bytes.get(prefix_len) else { return };
        let used_close = matching_percent_delimiter(used_open);
        let Some(close_at) = source.rfind(used_close as char) else { return };
        let preferred = self.preferred_delimiters(literal_type);
        let preferred_bytes = preferred.as_bytes();
        return_if!(preferred_bytes.len() < 2 || used_open == preferred_bytes[0]);
        let content_start = prefix_len + 1;
        return_if!(close_at < content_start);
        let content = &source[content_start..close_at];
        let literal_content = if let Some(array) = node.as_array_node() {
            array
                .elements()
                .iter()
                .filter(|element| {
                    element.as_string_node().is_some() || element.as_symbol_node().is_some()
                })
                .map(|element| self.source_file().node(&element))
                .collect::<String>()
        } else {
            without_interpolations(content)
        };
        return_if!(literal_content.contains(preferred_bytes[0] as char)
            || literal_content.contains(preferred_bytes[1] as char));
        if matches!(literal_type, "%w" | "%i") {
            return_if!(literal_content.contains(used_open as char) || literal_content.contains(used_close as char));
        }
        let message = format!(
            "`{literal_type}`-literals should be delimited by `{}` and `{}`.",
            preferred_bytes[0] as char, preferred_bytes[1] as char
        );
        let start = node.location().start_offset();
        let opening = start..start + prefix_len + 1;
        let closing = start + close_at..start + close_at + 1;
        let replacement_open = format!("{literal_type}{}", preferred_bytes[0] as char);
        add_offense!(self, node.location(), message: message, |corrector| {
            corrector.replace(opening, replacement_open);
            corrector.replace(closing, (preferred_bytes[1] as char).to_string());
        });
    }

    fn preferred_delimiters(&self, literal_type: &str) -> String {
        self.config_map("PreferredDelimiters")
            .and_then(|map| map.get(literal_type).or_else(|| map.get("default")))
            .cloned()
            .unwrap_or_else(|| "[]".to_string())
    }
}

fn percent_literal_type(source: &str) -> Option<(&'static str, usize)> {
    for literal_type in ["%w", "%W", "%i", "%I", "%r", "%q", "%Q", "%s", "%x"] {
        if source.starts_with(literal_type) { return Some((literal_type, literal_type.len())); }
    }
    source.starts_with('%').then_some(("%", 1))
}

fn matching_percent_delimiter(open: u8) -> u8 {
    match open { b'(' => b')', b'[' => b']', b'{' => b'}', b'<' => b'>', other => other }
}

fn without_interpolations(source: &str) -> String {
    let mut result = String::new();
    let mut index = 0;
    while index < source.len() {
        if source[index..].starts_with("#{") {
            let mut depth = 1;
            index += 2;
            while index < source.len() && depth > 0 {
                match source.as_bytes()[index] { b'{' => depth += 1, b'}' => depth -= 1, _ => {} }
                index += 1;
            }
        } else {
            let Some(character) = source[index..].chars().next() else { break };
            result.push(character);
            index += character.len_utf8();
        }
    }
    result
}
