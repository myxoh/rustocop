use std::collections::HashMap;

use ruby_prism::{CallNode, Node, StringNode};

use super::*;

define_rule!(RedundantFormatRule);

const MSG: &str = "Use `{prefer}` directly instead of `{method_name}`.";

define_cops! {
    RedundantFormat => "Style/RedundantFormat" => compatibility_prism_call_rule(
        RedundantFormatRule,
        on_send,
        restrict [b"format", b"sprintf"]
    ),
}

impl RedundantFormatRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let arguments = node
            .arguments()
            .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
            .unwrap_or_default();
        return_if!(arguments.is_empty());

        if arguments.len() == 1
            && (node.receiver().is_none() || root_constant(node.receiver(), b"Kernel"))
            && single_redundant_argument(&arguments[0])
        {
            let replacement = escape_control_chars(self.source_file().node(&arguments[0]));
            self.register_offense(node, replacement);
            return;
        }
        return_if!(arguments.len() == 1);

        let Some(format_node) = arguments[0].as_string_node() else {
            return;
        };
        return_if!(arguments[1..].iter().any(splatted_argument));
        let Some(formatted) = format_literal_arguments(
            String::from_utf8_lossy(format_node.unescaped()).as_ref(),
            &arguments[1..],
            self.source_file(),
        ) else {
            return;
        };
        let Some(replacement) = quote_formatted_string(&format_node, &formatted, self.source_file())
        else {
            return;
        };
        self.register_offense(node, replacement);
    }

    fn register_offense(&mut self, node: &CallNode<'_>, replacement: String) {
        let method_name = String::from_utf8_lossy(node.name().as_slice());
        let message = MSG
            .replace("{prefer}", &replacement)
            .replace("{method_name}", &method_name);
        add_offense!(self, node.location(), message: message, |corrector| {
            corrector.replace(node.location(), replacement);
        });
    }
}

fn single_redundant_argument(node: &Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_constant_read_node().is_some()
        || node.as_constant_path_node().is_some()
}

fn splatted_argument(node: &Node<'_>) -> bool {
    node.as_splat_node().is_some()
        || node.as_assoc_splat_node().is_some()
        || node.as_forwarding_arguments_node().is_some()
        || node.as_keyword_hash_node().is_some_and(|hash| {
            hash.elements()
                .iter()
                .any(|element| element.as_assoc_splat_node().is_some())
        })
}

#[derive(Clone)]
enum Literal {
    Text { value: String, dynamic: bool },
    Number { value: f64, display: String },
    Boolean(bool),
    Nil,
    Hash(HashMap<String, Literal>),
}

impl Literal {
    fn string_value(&self) -> Option<(String, bool)> {
        match self {
            Self::Text { value, dynamic } => Some((value.clone(), *dynamic)),
            Self::Number { display, .. } => Some((display.clone(), false)),
            Self::Boolean(value) => Some((value.to_string(), false)),
            Self::Nil => Some(("nil".to_string(), false)),
            Self::Hash(_) => None,
        }
    }

    fn integer_value(&self) -> Option<i128> {
        match self {
            Self::Number { value, .. } if value.is_finite() => Some(value.trunc() as i128),
            Self::Text {
                value,
                dynamic: false,
            } => value.parse().ok(),
            _ => None,
        }
    }

    fn float_value(&self) -> Option<f64> {
        match self {
            Self::Number { value, .. } if value.is_finite() => Some(*value),
            Self::Text {
                value,
                dynamic: false,
            } => value.parse().ok(),
            _ => None,
        }
    }
}

struct Formatted {
    value: String,
    dynamic: bool,
}

fn format_literal_arguments(
    template: &str,
    arguments: &[Node<'_>],
    file: SourceFile<'_>,
) -> Option<Formatted> {
    let values = arguments
        .iter()
        .map(|argument| literal_value(argument, file))
        .collect::<Option<Vec<_>>>()?;
    let named = values.iter().find_map(|value| match value {
        Literal::Hash(values) => Some(values),
        _ => None,
    });
    let mut parser = FormatParser::new(template, &values, named);
    parser.render()
}

struct FormatParser<'a> {
    template: &'a str,
    values: &'a [Literal],
    named: Option<&'a HashMap<String, Literal>>,
    next_argument: usize,
    output: String,
    dynamic: bool,
}

impl<'a> FormatParser<'a> {
    fn new(
        template: &'a str,
        values: &'a [Literal],
        named: Option<&'a HashMap<String, Literal>>,
    ) -> Self {
        Self {
            template,
            values,
            named,
            next_argument: 0,
            output: String::new(),
            dynamic: false,
        }
    }

    fn render(&mut self) -> Option<Formatted> {
        let mut index = 0;
        while index < self.template.len() {
            let byte = self.template.as_bytes()[index];
            if byte != b'%' {
                let character = self.template[index..].chars().next()?;
                self.output.push(character);
                index += character.len_utf8();
                continue;
            }
            let (field, end) = ParsedField::parse(self.template, index)?;
            if field.percent {
                return None;
            }
            self.render_field(&field)?;
            index = end;
        }
        Some(Formatted {
            value: std::mem::take(&mut self.output),
            dynamic: self.dynamic,
        })
    }

    fn render_field(&mut self, field: &ParsedField) -> Option<()> {
        if let Some(name) = &field.name {
            let value = self.named?.get(name)?;
            if field.template {
                let (text, dynamic) = value.string_value()?;
                self.output.push_str(&text);
                self.dynamic |= dynamic;
                return Some(());
            }
            return self.render_value(field, value, None, None);
        }

        let width = self.resolve_dimension(field.width)?;
        let precision = self.resolve_dimension(field.precision)?;
        let value_index = field.argument_index.unwrap_or_else(|| {
            let current = self.next_argument;
            self.next_argument += 1;
            current
        });
        let value = self.values.get(value_index)?;
        self.render_value(field, value, width, precision)
    }

    fn resolve_dimension(&mut self, dimension: Dimension) -> Option<Option<i32>> {
        match dimension {
            Dimension::None => Some(None),
            Dimension::Fixed(value) => Some(Some(value)),
            Dimension::Argument(index) => {
                let index = index.unwrap_or_else(|| {
                    let current = self.next_argument;
                    self.next_argument += 1;
                    current
                });
                Some(Some(i32::try_from(self.values.get(index)?.integer_value()?).ok()?))
            }
        }
    }

    fn render_value(
        &mut self,
        field: &ParsedField,
        value: &Literal,
        width: Option<i32>,
        precision: Option<i32>,
    ) -> Option<()> {
        let kind = field.kind?;
        let rendered = match kind {
            's' => {
                let (text, dynamic) = value.string_value()?;
                if dynamic && (width.is_some() || precision.is_some()) {
                    return None;
                }
                self.dynamic |= dynamic;
                format_text(text, width, precision, field.left)
            }
            'd' | 'i' | 'u' => format_integer(
                value.integer_value()?,
                width,
                precision,
                field.left,
                field.zero,
                field.sign,
            ),
            'f' => format_float(
                value.float_value()?,
                width,
                precision,
                field.left,
                field.zero,
                field.sign,
            ),
            _ => return None,
        };
        self.output.push_str(&rendered);
        Some(())
    }
}

#[derive(Clone, Copy)]
enum Dimension {
    None,
    Fixed(i32),
    Argument(Option<usize>),
}

struct ParsedField {
    percent: bool,
    template: bool,
    name: Option<String>,
    argument_index: Option<usize>,
    width: Dimension,
    precision: Dimension,
    left: bool,
    zero: bool,
    sign: Option<char>,
    kind: Option<char>,
}

impl ParsedField {
    fn parse(source: &str, start: usize) -> Option<(Self, usize)> {
        let bytes = source.as_bytes();
        let mut index = start + 1;
        if bytes.get(index) == Some(&b'%') {
            return Some((Self::percent(), index + 1));
        }
        if bytes.get(index) == Some(&b'{') {
            let close = source[index + 1..].find('}')? + index + 1;
            return Some((Self::named(&source[index + 1..close], true), close + 1));
        }

        let mut name = None;
        if bytes.get(index) == Some(&b'<') {
            let close = source[index + 1..].find('>')? + index + 1;
            name = Some(source[index + 1..close].to_string());
            index = close + 1;
        }

        let argument_index = parse_numbered_argument(source, &mut index);
        let mut left = false;
        let mut zero = false;
        let mut sign = None;
        while let Some(byte) = bytes.get(index).copied() {
            match byte {
                b'-' => left = true,
                b'0' => zero = true,
                b'+' => sign = Some('+'),
                b' ' => sign = Some(' '),
                b'#' => {}
                _ => break,
            }
            index += 1;
        }
        let width = parse_dimension(source, &mut index, false)?;
        let precision = if bytes.get(index) == Some(&b'.') {
            index += 1;
            parse_dimension(source, &mut index, true)?
        } else {
            Dimension::None
        };
        let kind = source[index..].chars().next()?;
        index += kind.len_utf8();
        Some((
            Self {
                percent: false,
                template: false,
                name,
                argument_index,
                width,
                precision,
                left,
                zero,
                sign,
                kind: Some(kind),
            },
            index,
        ))
    }

    fn percent() -> Self {
        Self {
            percent: true,
            template: false,
            name: None,
            argument_index: None,
            width: Dimension::None,
            precision: Dimension::None,
            left: false,
            zero: false,
            sign: None,
            kind: None,
        }
    }

    fn named(name: &str, template: bool) -> Self {
        Self {
            percent: false,
            template,
            name: Some(name.to_string()),
            argument_index: None,
            width: Dimension::None,
            precision: Dimension::None,
            left: false,
            zero: false,
            sign: None,
            kind: None,
        }
    }
}

fn parse_numbered_argument(source: &str, index: &mut usize) -> Option<usize> {
    let start = *index;
    while source.as_bytes().get(*index).is_some_and(u8::is_ascii_digit) {
        *index += 1;
    }
    if *index > start && source.as_bytes().get(*index) == Some(&b'$') {
        let number = source[start..*index].parse::<usize>().ok()?;
        *index += 1;
        Some(number.checked_sub(1)?)
    } else {
        *index = start;
        None
    }
}

fn parse_dimension(source: &str, index: &mut usize, precision: bool) -> Option<Dimension> {
    if source.as_bytes().get(*index) == Some(&b'*') {
        *index += 1;
        let numbered = parse_numbered_argument(source, index);
        return Some(Dimension::Argument(numbered));
    }
    let start = *index;
    while source.as_bytes().get(*index).is_some_and(u8::is_ascii_digit) {
        *index += 1;
    }
    if *index > start {
        return source[start..*index]
            .parse::<i32>()
            .ok()
            .map(Dimension::Fixed);
    }
    if precision {
        Some(Dimension::Fixed(0))
    } else {
        Some(Dimension::None)
    }
}

fn literal_value(node: &Node<'_>, file: SourceFile<'_>) -> Option<Literal> {
    if let Some(string) = node.as_string_node() {
        return Some(Literal::Text {
            value: String::from_utf8_lossy(string.unescaped()).into_owned(),
            dynamic: false,
        });
    }
    if let Some(string) = node.as_interpolated_string_node() {
        return Some(Literal::Text {
            value: interpolated_body(file.node(node), string.opening_loc(), string.closing_loc())?,
            dynamic: true,
        });
    }
    if let Some(symbol) = node.as_symbol_node() {
        return Some(Literal::Text {
            value: String::from_utf8_lossy(symbol.unescaped()).into_owned(),
            dynamic: false,
        });
    }
    if let Some(symbol) = node.as_interpolated_symbol_node() {
        return Some(Literal::Text {
            value: interpolated_body(file.node(node), symbol.opening_loc(), symbol.closing_loc())?,
            dynamic: true,
        });
    }
    if node.as_true_node().is_some() {
        return Some(Literal::Boolean(true));
    }
    if node.as_false_node().is_some() {
        return Some(Literal::Boolean(false));
    }
    if node.as_nil_node().is_some() {
        return Some(Literal::Nil);
    }
    if let Some(hash) = node.as_keyword_hash_node() {
        return keyword_hash_value(&hash, file).map(Literal::Hash);
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses.body().and_then(single_expression).and_then(|value| literal_value(&value, file));
    }
    numeric_literal(file.node(node)).map(|(value, display)| Literal::Number { value, display })
}

fn keyword_hash_value(
    hash: &ruby_prism::KeywordHashNode<'_>,
    file: SourceFile<'_>,
) -> Option<HashMap<String, Literal>> {
    hash.elements()
        .iter()
        .map(|element| {
            let pair = element.as_assoc_node()?;
            let key = pair.key().as_symbol_node()?;
            let key = String::from_utf8_lossy(key.unescaped()).into_owned();
            Some((key, literal_value(&pair.value(), file)?))
        })
        .collect()
}

fn interpolated_body(
    source: &str,
    opening: Option<ruby_prism::Location<'_>>,
    closing: Option<ruby_prism::Location<'_>>,
) -> Option<String> {
    let opening = opening?;
    let closing = closing?;
    let opening_len = opening.as_slice().len();
    let closing_len = closing.as_slice().len();
    source
        .get(opening_len..source.len().checked_sub(closing_len)?)
        .map(ToString::to_string)
}

fn numeric_literal(source: &str) -> Option<(f64, String)> {
    let source = source.trim().trim_start_matches('(').trim_end_matches(')');
    let compact = source.replace('_', "");
    if let Some(value) = compact.strip_suffix('i') {
        if let Some((real, imaginary)) = value.split_once('+') {
            let real = parse_ruby_number(real)?;
            let imaginary = parse_ruby_number(imaginary)?;
            return (imaginary == 0.0).then(|| (real, format_number(real)));
        }
        let imaginary = parse_ruby_number(value)?;
        return Some((imaginary, format!("0+{}i", format_number(imaginary))));
    }
    if let Some(value) = compact.strip_suffix('r') {
        if let Some((numerator, denominator)) = value.split_once('/') {
            let numerator = numerator.parse::<f64>().ok()?;
            let denominator = denominator.parse::<f64>().ok()?;
            return Some((
                numerator / denominator,
                format!("{numerator:.0}/{denominator:.0}"),
            ));
        }
        let number = value.parse::<f64>().ok()?;
        return Some((number, format!("{number:.0}/1")));
    }
    let value = compact.parse::<f64>().ok()?;
    Some((value, compact))
}

fn parse_ruby_number(source: &str) -> Option<f64> {
    source
        .strip_suffix('r')
        .unwrap_or(source)
        .parse::<f64>()
        .ok()
}

fn format_number(value: f64) -> String {
    if value.fract() == 0.0 {
        format!("{value:.0}")
    } else {
        value.to_string()
    }
}

fn format_text(mut value: String, width: Option<i32>, precision: Option<i32>, left: bool) -> String {
    if let Some(precision) = precision {
        value = value.chars().take(precision.max(0) as usize).collect();
    }
    pad(value, width, left, ' ')
}

fn format_integer(
    value: i128,
    width: Option<i32>,
    precision: Option<i32>,
    left: bool,
    zero: bool,
    sign: Option<char>,
) -> String {
    let negative = value < 0;
    let mut digits = value.unsigned_abs().to_string();
    if precision == Some(0) && value == 0 {
        digits.clear();
    } else if let Some(precision) = precision {
        let required = precision.max(0) as usize;
        if digits.len() < required {
            digits = format!("{}{}", "0".repeat(required - digits.len()), digits);
        }
    }
    let prefix = if negative {
        "-"
    } else if sign == Some('+') {
        "+"
    } else if sign == Some(' ') {
        " "
    } else {
        ""
    };
    let value = format!("{prefix}{digits}");
    pad(value, width, left, if zero && precision.is_none() { '0' } else { ' ' })
}

fn format_float(
    value: f64,
    width: Option<i32>,
    precision: Option<i32>,
    left: bool,
    zero: bool,
    sign: Option<char>,
) -> String {
    let precision = precision.unwrap_or(6).max(0) as usize;
    let mut value = format!("{value:.precision$}");
    if value.as_bytes().first() != Some(&b'-') {
        if sign == Some('+') {
            value.insert(0, '+');
        } else if sign == Some(' ') {
            value.insert(0, ' ');
        }
    }
    pad(value, width, left, if zero { '0' } else { ' ' })
}

fn pad(value: String, width: Option<i32>, left: bool, fill: char) -> String {
    let Some(width) = width else { return value };
    let left = left || width < 0;
    let width = width.unsigned_abs() as usize;
    if value.chars().count() >= width {
        return value;
    }
    let padding = fill.to_string().repeat(width - value.chars().count());
    if left {
        format!("{value}{padding}")
    } else if fill == '0' && matches!(value.as_bytes().first(), Some(b'+' | b'-' | b' ')) {
        format!("{}{}{}", &value[..1], padding, &value[1..])
    } else {
        format!("{padding}{value}")
    }
}

fn quote_formatted_string(
    format_node: &StringNode<'_>,
    formatted: &Formatted,
    file: SourceFile<'_>,
) -> Option<String> {
    let opening = file.at(&format_node.opening_loc()?).to_string();
    let closing = file.at(&format_node.closing_loc()?).to_string();
    let (opening, closing) = if formatted.dynamic {
        match opening.as_str() {
            "'" => ("\"".to_string(), "\"".to_string()),
            value if value.starts_with("%q") => (value.replacen("%q", "%Q", 1), closing),
            _ => (opening, closing),
        }
    } else {
        (opening, closing)
    };
    Some(format!(
        "{opening}{}{closing}",
        escape_control_chars(&formatted.value)
    ))
}

fn escape_control_chars(source: &str) -> String {
    source
        .chars()
        .map(|character| match character {
            '\u{0007}' => "\\a".to_string(),
            '\u{0008}' => "\\b".to_string(),
            '\t' => "\\t".to_string(),
            '\n' => "\\n".to_string(),
            '\u{000b}' => "\\v".to_string(),
            '\u{000c}' => "\\f".to_string(),
            '\r' => "\\r".to_string(),
            '\u{001b}' => "\\e".to_string(),
            value if value.is_control() => value.escape_default().to_string(),
            value => value.to_string(),
        })
        .collect()
}
