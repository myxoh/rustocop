use ruby_prism::StringNode;

use super::*;

define_rule!(RedundantStringEscapeRule);

const MSG: &str = "Redundant escape of {char} inside string literal.";

define_cops! {
    RedundantStringEscape => "Style/RedundantStringEscape" => node_rule(as_string_node, RedundantStringEscapeRule, on_str),
}

impl RedundantStringEscapeRule<'_, '_, '_> {
    fn on_str(&mut self, node: &StringNode<'_>) {
        return_if!(self.parent().is_some_and(|parent| {
            parent.as_regular_expression_node().is_some()
                || parent.as_interpolated_regular_expression_node().is_some()
                || parent.as_x_string_node().is_some()
                || parent.as_interpolated_x_string_node().is_some()
        }));

        let Some(literal) = StringLiteral::new(node, self) else {
            return;
        };
        return_if!(literal.character_literal || !literal.interpolation_enabled);

        let content = self
            .source()
            .get(literal.content.clone())
            .unwrap_or_default();
        let mut characters = content.char_indices().peekable();
        while let Some((relative, character)) = characters.next() {
            if character != '\\' {
                continue;
            }
            let Some((_, escaped)) = characters.next() else {
                break;
            };
            let start = literal.content.start + relative;
            let end = start + 1 + escaped.len_utf8();
            if self.allowed_escape(&literal, start, end, escaped) {
                continue;
            }
            let message = MSG.replace("{char}", &escaped.to_string());
            add_offense!(self, start..end, message: message, |corrector| {
                corrector.replace(start..end, escaped.to_string());
            });
        }
    }

    fn allowed_escape(
        &self,
        literal: &StringLiteral,
        start: usize,
        end: usize,
        escaped: char,
    ) -> bool {
        if escaped == '\n' || escaped == '\\' || escaped.is_alphanumeric() {
            return true;
        }
        if escaped == ' ' && (literal.percent_array || literal.heredoc) {
            return true;
        }
        if disabling_interpolation(self.source(), start, end, escaped) {
            return true;
        }
        !literal.heredoc
            && (Some(escaped) == literal.opening_delimiter
                || Some(escaped) == literal.closing_delimiter)
    }
}

struct StringLiteral {
    content: std::ops::Range<usize>,
    interpolation_enabled: bool,
    percent_array: bool,
    heredoc: bool,
    character_literal: bool,
    opening_delimiter: Option<char>,
    closing_delimiter: Option<char>,
}

impl StringLiteral {
    fn new(node: &StringNode<'_>, context: &CopContext<'_, '_>) -> Option<Self> {
        let content = node.content_loc();
        let (opening, closing) = if let Some(opening) = node.opening_loc() {
            (
                context.source_file().at(&opening).to_string(),
                node.closing_loc()
                    .map(|closing| context.source_file().at(&closing).to_string())
                    .unwrap_or_default(),
            )
        } else if let Some((opening, closing)) = context
            .ancestors()
            .iter()
            .rev()
            .find_map(|ancestor| {
                let string = ancestor.as_interpolated_string_node()?;
                Some((string.opening_loc()?, string.closing_loc()?))
            })
        {
            (
                context.source_file().at(&opening).to_string(),
                context.source_file().at(&closing).to_string(),
            )
        } else if let Some(array) = context
            .ancestors()
            .iter()
            .rev()
            .find_map(ruby_prism::Node::as_array_node)
        {
            let opening = array.opening_loc()?;
            let closing = array.closing_loc()?;
            (
                context.source_file().at(&opening).to_string(),
                context.source_file().at(&closing).to_string(),
            )
        } else {
            return None;
        };

        let heredoc = opening.starts_with("<<");
        let percent_array = opening.starts_with("%w") || opening.starts_with("%W");
        let interpolation_enabled = !(opening == "'"
            || opening.starts_with("%q")
            || opening.starts_with("%w")
            || heredoc && opening.contains('\''));
        Some(Self {
            content: content.start_offset()..content.end_offset(),
            interpolation_enabled,
            percent_array,
            heredoc,
            character_literal: opening == "?",
            opening_delimiter: (!heredoc).then(|| opening.chars().last()).flatten(),
            closing_delimiter: (!heredoc).then(|| closing.chars().next()).flatten(),
        })
    }
}

fn disabling_interpolation(source: &str, start: usize, end: usize, escaped: char) -> bool {
    let after = source.get(end..).unwrap_or_default();
    if escaped == '#' && after.starts_with(['{', '$', '@']) {
        return true;
    }
    if escaped == '#' && after.starts_with("\\{") {
        return true;
    }
    if matches!(escaped, '{' | '$' | '@') && start >= 2 {
        let prefix = source.as_bytes();
        return prefix.get(start - 1) == Some(&b'#') && prefix.get(start - 2) != Some(&b'\\');
    }
    false
}
