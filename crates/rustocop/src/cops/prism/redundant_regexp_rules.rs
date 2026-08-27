use ruby_prism::{CallNode, Node};

use super::*;

define_rule!(RedundantRegexpEscapeRule);
define_rule!(RedundantRegexpConstructorRule);
define_rule!(RedundantRegexpArgumentRule);
define_compatibility_rule!(RedundantRegexpCharacterClassRule);

const ESCAPE_MSG: &str = "Redundant escape inside regexp literal";
const CLASS_MSG: &str =
    "Redundant single-element character class, `{class}` can be replaced with `{element}`.";
const ARGUMENT_MSG: &str = "Use string `{prefer}` as argument instead of regexp `{current}`.";

define_cops! {
    RedundantRegexpEscape => "Style/RedundantRegexpEscape" => node_rule_aliases(
        RedundantRegexpEscapeRule,
        on_regexp => [as_regular_expression_node, as_interpolated_regular_expression_node]
    ),
    RedundantRegexpConstructor => "Style/RedundantRegexpConstructor" => call_rule(
        RedundantRegexpConstructorRule,
        on_send,
        restrict [b"new", b"compile"]
    ),
    RedundantRegexpCharacterClass => "Style/RedundantRegexpCharacterClass" => compatibility_callbacks(RedundantRegexpCharacterClassRule, [on_regexp]),
    RedundantRegexpArgument => "Style/RedundantRegexpArgument" => call_rule(
        RedundantRegexpArgumentRule,
        on_send,
        restrict [b"byteindex", b"byterindex", b"gsub", b"gsub!", b"partition", b"rpartition", b"scan", b"split", b"start_with?", b"sub", b"sub!"]
    ),
}

impl RedundantRegexpConstructorRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        return_unless!(root_constant(node.receiver(), b"Regexp"));
        let Some(regexp) = only_argument(node).filter(|argument| {
            argument.as_regular_expression_node().is_some()
                || argument.as_interpolated_regular_expression_node().is_some()
        }) else {
            return;
        };
        let method = String::from_utf8_lossy(node.name().as_slice());
        let message = format!("Remove the redundant `Regexp.{method}`.");
        let replacement = self.source_file().node(&regexp).to_string();
        add_offense!(self, node.location(), message: message, |corrector| {
            corrector.replace(node.location(), replacement);
        });
    }
}

impl RedundantRegexpEscapeRule<'_, '_, '_> {
    fn on_regexp(&mut self, node: &Node<'_>) {
        let Some(regexp) = RegexpView::new(node, self.source()) else {
            return;
        };
        let mut class_depth = 0;
        for part in &regexp.parts {
            let Some(source) = self.source().get(part.clone()) else {
                continue;
            };
            let (escapes, next_class_depth) = redundant_escapes(
                source,
                regexp.opening_delimiter,
                regexp.closing_delimiter,
                regexp.extended,
                class_depth,
            );
            class_depth = next_class_depth;
            for escape in escapes {
                let location = part.start + escape..part.start + escape + 2;
                add_offense!(self, location.clone(), message: ESCAPE_MSG, |corrector| {
                    corrector.remove(location.start..location.start + 1);
                });
            }
        }
    }
}

impl RedundantRegexpCharacterClassRule<'_, '_, '_, '_> {
    fn on_regexp(&mut self, node: crate::rubocop::ast::node::core::NodeRef<'_>) {
        let Some(regexp) = RegexpView::new_compatibility(node, self.source_buffer()) else {
            return;
        };
        for part in &regexp.parts {
            let Some(source) = self.source().get(part.clone()) else {
                continue;
            };
            for character_class in single_element_character_classes(source, regexp.extended) {
                let location = part.start + character_class.start..part.start + character_class.end;
                let class = &source[character_class.start..character_class.end];
                let element = if character_class.element == "#" {
                    "\\#".to_string()
                } else {
                    character_class.element
                };
                let message = CLASS_MSG
                    .replace("{class}", class)
                    .replace("{element}", &element);
                let Some(location) = crate::rubocop::ast::source::SourceRange::from_byte_range(self.source_buffer(), location) else { continue; };
                let location = self.owned_range(location);
                add_offense!(self, location.clone(), message: message, |corrector| {
                    corrector.replace(location, element);
                });
            }
        }
    }
}

impl RedundantRegexpArgumentRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some(argument) = node
            .arguments()
            .and_then(|arguments| arguments.arguments().iter().next())
        else {
            return;
        };
        let Some(regexp) = argument.as_regular_expression_node() else {
            return;
        };
        let source = self.source_file().node(&argument);
        return_unless!(source.starts_with('/'));
        let content = self.source_file().at(&regexp.content_loc());
        let closing = self.source_file().at(&regexp.closing_loc());
        return_if!(closing.chars().skip(1).any(|character| character.is_ascii_alphabetic()) || content == " ");
        return_unless!(determinist_regexp(content));

        let prefer = self.preferred_argument(content);
        let message = ARGUMENT_MSG
            .replace("{prefer}", &prefer)
            .replace("{current}", source);
        add_offense!(self, argument.location(), message: message, |corrector| {
            corrector.replace(argument.location(), prefer);
        });
    }

    fn preferred_argument(&self, content: &str) -> String {
        let mut replacement = regexp_to_string_content(content);
        let quote;
        if replacement.contains('"') {
            replacement = replacement.replace('\'', "\\'").replace("\\\"", "\"");
            quote = '\'';
        } else if replacement.contains("\\'") || replacement.contains('\'') {
            replacement = escape_single_quotes(&replacement);
            quote = '\'';
        } else if replacement.contains('\\')
            || self.related_config_value("Style/StringLiterals", "EnforcedStyle")
                == Some("double_quotes")
        {
            quote = '"';
        } else {
            quote = '\'';
        }
        format!("{quote}{replacement}{quote}")
    }
}

struct RegexpView {
    parts: Vec<std::ops::Range<usize>>,
    opening_delimiter: char,
    closing_delimiter: char,
    extended: bool,
}

impl RegexpView {
    fn new_compatibility(
        node: crate::rubocop::ast::node::core::NodeRef<'_>,
        buffer: &crate::rubocop::ast::source::SourceBuffer<'_>,
    ) -> Option<Self> {
        let opening = node.loc("begin")?.1.as_str();
        let closing = node.loc("end")?.1.as_str();
        let parts = node.child_nodes().into_iter()
            .filter(|part| part.kind() == "str")
            .filter_map(|part| part.source_range())
            .filter_map(|range| {
                Some(buffer.byte_position(range.start)?..buffer.byte_position(range.end)?)
            })
            .collect();
        Some(Self {
            parts,
            opening_delimiter: opening.chars().last()?,
            closing_delimiter: closing.chars().next()?,
            extended: node.regexp_extended(),
        })
    }

    fn new(node: &Node<'_>, source: &str) -> Option<Self> {
        let (opening, closing, parts) = if let Some(regexp) = node.as_regular_expression_node() {
            let content = regexp.content_loc();
            (
                regexp.opening_loc(),
                regexp.closing_loc(),
                std::iter::once(content.start_offset()..content.end_offset()).collect(),
            )
        } else {
            let regexp = node.as_interpolated_regular_expression_node()?;
            let parts = regexp
                .parts()
                .iter()
                .filter_map(|part| {
                    part.as_string_node().map(|string| {
                        let location = string.location();
                        location.start_offset()..location.end_offset()
                    })
                })
                .collect();
            (regexp.opening_loc(), regexp.closing_loc(), parts)
        };
        let opening_source = source.get(opening.start_offset()..opening.end_offset())?;
        let closing_source = source.get(closing.start_offset()..closing.end_offset())?;
        let opening_delimiter = opening_source.chars().last()?;
        let closing_delimiter = closing_source.chars().next()?;
        Some(Self {
            parts,
            opening_delimiter,
            closing_delimiter,
            extended: closing_source.get(1..).is_some_and(|options| options.contains('x')),
        })
    }
}

fn redundant_escapes(
    source: &str,
    opening_delimiter: char,
    closing_delimiter: char,
    extended: bool,
    initial_class_depth: usize,
) -> (Vec<usize>, usize) {
    let bytes = source.as_bytes();
    let mut offenses = Vec::new();
    let mut class_depth = initial_class_depth;
    let mut class_starts = vec![None; initial_class_depth];
    let mut index = 0;
    let mut comment = false;
    while index < bytes.len() {
        let character = bytes[index] as char;
        if comment {
            if character == '\n' {
                comment = false;
            }
            index += 1;
            continue;
        }
        if extended && class_depth == 0 && character == '#' {
            comment = true;
            index += 1;
            continue;
        }
        if character == '[' {
            class_depth += 1;
            class_starts.push(Some(index));
            index += 1;
            continue;
        }
        if character == ']' && class_depth > 0 {
            class_depth -= 1;
            class_starts.pop();
            index += 1;
            continue;
        }
        if character != '\\' {
            index += 1;
            continue;
        }
        if index + 1 == bytes.len() {
            break;
        }
        let escaped = bytes[index + 1] as char;
        let previous = index.checked_sub(1).and_then(|at| bytes.get(at)).copied().map(char::from);
        let after = bytes.get(index + 2).copied().map(char::from);
        let allowed = escaped.is_ascii_alphanumeric()
            || matches!(escaped, ' ' | '\n' | '[' | ']' | '^' | '\\' | '#')
            || escaped == opening_delimiter
            || escaped == closing_delimiter
            || previous == Some('#') && matches!(escaped, '@' | '$')
            || if class_depth > 0 {
                escaped == '-'
                    && class_starts
                        .last()
                        .is_some_and(|start| start.is_none_or(|start| index != start + 1))
                    && after != Some(']')
            } else {
                ".*+?{}()|$".contains(escaped)
            };
        if !allowed {
            offenses.push(index);
        }
        index += 2;
    }
    (offenses, class_depth)
}

struct CharacterClass {
    start: usize,
    end: usize,
    element: String,
}

fn single_element_character_classes(source: &str, extended: bool) -> Vec<CharacterClass> {
    let bytes = source.as_bytes();
    let mut classes = Vec::new();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += escape_token_length(&source[index..]).max(2);
            continue;
        }
        if extended && bytes[index] == b'#' {
            index = source[index..]
                .find('\n')
                .map_or(bytes.len(), |newline| index + newline + 1);
            continue;
        }
        if bytes[index] != b'[' {
            index += 1;
            continue;
        }
        let start = index;
        let mut depth = 1_usize;
        index += 1;
        while index < bytes.len() && depth > 0 {
            if bytes[index] == b'\\' {
                index += escape_token_length(&source[index..]).max(2);
                continue;
            }
            if bytes[index] == b'[' {
                depth += 1;
            } else if bytes[index] == b']' {
                depth -= 1;
            }
            index += 1;
        }
        if depth != 0 {
            break;
        }
        let end = index;
        let inner = &source[start + 1..end - 1];
        let Some(element) = single_class_element(inner) else {
            continue;
        };
        if extended && element.chars().any(char::is_whitespace)
            || element == "\\b"
            || matches!(element.as_str(), "\\1" | "\\2" | "\\3" | "\\4" | "\\5" | "\\6" | "\\7")
            || ".*+?{}()|$".contains(element.as_str())
        {
            continue;
        }
        classes.push(CharacterClass { start, end, element });
    }
    classes
}

fn single_class_element(inner: &str) -> Option<String> {
    if inner.is_empty()
        || inner.starts_with('^')
        || inner.contains("&&")
        || inner.starts_with("[:")
        || inner.contains("[.")
        || inner.contains("[=")
    {
        return None;
    }
    if inner.starts_with("\\u{")
        && inner
            .strip_prefix("\\u{")
            .and_then(|value| value.strip_suffix('}'))
            .is_some_and(|codepoints| codepoints.split_whitespace().count() != 1)
    {
        return None;
    }
    let length = if inner.starts_with('\\') {
        escape_token_length(inner)
    } else {
        inner.chars().next()?.len_utf8()
    };
    (length == inner.len()).then(|| inner.to_string())
}

fn escape_token_length(source: &str) -> usize {
    let bytes = source.as_bytes();
    if bytes.len() < 2 || bytes[0] != b'\\' {
        return source.chars().next().map_or(0, char::len_utf8);
    }
    if matches!(bytes[1], b'u' | b'p' | b'P') && bytes.get(2) == Some(&b'{') {
        return source.find('}').map_or(2, |at| at + 1);
    }
    if bytes[1] == b'x' {
        return bytes.len().min(4);
    }
    if bytes[1] == b'u' {
        return bytes.len().min(6);
    }
    if matches!(bytes[1], b'0'..=b'7') {
        return 1 + bytes[1..].iter().take(3).take_while(|byte| matches!(byte, b'0'..=b'7')).count();
    }
    2
}

fn determinist_regexp(content: &str) -> bool {
    let bytes = content.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            if index + 1 >= bytes.len() {
                return false;
            }
            if bytes[index + 1].is_ascii_digit()
                || b"AbBdDgGhHkpPRwWXsSzZ".contains(&bytes[index + 1])
            {
                return false;
            }
            index += 2;
            continue;
        }
        if !(bytes[index].is_ascii_alphanumeric()
            || bytes[index] == b'_'
            || bytes[index].is_ascii_whitespace()
            || b"-,\"'!#%&<>=;:`~/".contains(&bytes[index]))
        {
            return false;
        }
        index += 1;
    }
    true
}

fn regexp_to_string_content(content: &str) -> String {
    let bytes = content.as_bytes();
    let mut output = String::with_capacity(content.len());
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'\\' || index + 1 >= bytes.len() {
            output.push(bytes[index] as char);
            index += 1;
            continue;
        }
        let escaped = bytes[index + 1] as char;
        if matches!(escaped, 'a' | 'c' | 'C' | 'e' | 'f' | 'M' | 'n' | '"' | '\'' | '\\' | 't' | 'b' | 'r' | 'u' | 'v' | 'x' | '0'..='7') {
            output.push('\\');
        }
        output.push(escaped);
        index += 2;
    }
    output
}

fn escape_single_quotes(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut slash_run = 0_usize;
    for character in source.chars() {
        if character == '\\' {
            output.push(character);
            slash_run += 1;
            continue;
        }
        if character == '\'' && slash_run.is_multiple_of(2) {
            output.push('\\');
        }
        output.push(character);
        slash_run = 0;
    }
    output
}
