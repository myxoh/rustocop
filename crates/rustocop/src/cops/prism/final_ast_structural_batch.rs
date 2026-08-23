use super::catalog_cop::custom;
use super::*;
use std::collections::{HashMap, HashSet};

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops: Vec<Box<dyn Cop>> = vec![
        Box::new(SafeNavigation),
        Box::new(SelectByKind),
        Box::new(SelectByRange),
        Box::new(RedundantTypeConversion),
        Box::new(ConditionalAssignment),
        Box::new(Debugger),
        custom("Lint/UselessAccessModifier", useless_access_modifier),
        Box::new(ArgumentsForwarding),
        Box::new(Void),
        custom("Lint/LiteralInInterpolation", literal_in_interpolation),
    ];
    cops.extend(registry::cops());
    cops
}

fn literal_in_interpolation(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    for (opening, closing) in interpolation_ranges(source) {
        let Some((start, end)) = final_interpolation_expression(source, opening + 2, closing)
        else {
            continue;
        };
        let expression = &source[start..end];
        if !literal_interpolation_expression(expression) {
            continue;
        }
        if array_percent_interpolation(source, opening, expression) {
            continue;
        }
        if heredoc_trailing_space_interpolation(source, closing, expression)
            || regexp_array_interpolation(source, opening, expression)
        {
            continue;
        }
        let direct_regexp = {
            let line_start = source[..opening].rfind('\n').map_or(0, |at| at + 1);
            source[line_start..opening].trim_start().starts_with('/')
        };
        let replacement = if direct_regexp {
            decoded_string_literal(expression)
                .map(|value| escape_regexp_slashes(&value))
                .unwrap_or_else(|| {
                    interpolation_literal_value(
                        expression,
                        interpolation_outer_delimiter(source, opening),
                    )
                })
        } else if expression.starts_with('"')
            && expression.ends_with('"')
            && expression
                .as_bytes()
                .windows(2)
                .any(|pair| pair[0] == b'\\' && pair[1].is_ascii_digit())
        {
            expression[1..expression.len() - 1].to_string()
        } else {
            interpolation_literal_value(expression, interpolation_outer_delimiter(source, opening))
        };
        context.replace(
            "Literal interpolation detected.",
            start..end,
            opening..closing + 1,
            replacement,
        );
    }
}

fn interpolation_ranges(source: &str) -> Vec<(usize, usize)> {
    #[derive(Default)]
    struct Interpolations(Vec<(usize, usize)>);

    impl<'pr> Visit<'pr> for Interpolations {
        fn visit_embedded_statements_node(
            &mut self,
            node: &ruby_prism::EmbeddedStatementsNode<'pr>,
        ) {
            let location = node.location();
            if location.as_slice().starts_with(b"#{") && location.as_slice().ends_with(b"}") {
                self.0
                    .push((location.start_offset(), location.end_offset() - 1));
            }
            ruby_prism::visit_embedded_statements_node(self, node);
        }
    }

    let parsed = ruby_prism::parse(source.as_bytes());
    let mut interpolations = Interpolations::default();
    interpolations.visit(&parsed.node());
    interpolations.0
}

fn final_interpolation_expression(
    source: &str,
    content_start: usize,
    content_end: usize,
) -> Option<(usize, usize)> {
    let bytes = source.as_bytes();
    let mut start = content_start;
    let mut quote = None;
    let mut nesting = 0usize;
    let mut index = content_start;
    while index < content_end {
        let byte = bytes[index];
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == delimiter {
                quote = None;
            }
        } else {
            match byte {
                b'\'' | b'"' | b'`' => quote = Some(byte),
                b'(' | b'[' | b'{' => nesting += 1,
                b')' | b']' | b'}' => nesting = nesting.saturating_sub(1),
                b';' if nesting == 0 => start = index + 1,
                _ => {}
            }
        }
        index += 1;
    }
    while start < content_end && bytes[start].is_ascii_whitespace() {
        start += 1;
    }
    let mut end = content_end;
    while end > start && bytes[end - 1].is_ascii_whitespace() {
        end -= 1;
    }
    (start < end).then_some((start, end))
}

fn literal_interpolation_expression(expression: &str) -> bool {
    let expression = expression.trim();
    if expression.contains("#{") || expression.starts_with('`') {
        return false;
    }
    if matches!(expression, "nil" | "true" | "false") {
        return true;
    }
    if expression.starts_with(['\'', '"'])
        && expression.ends_with(expression.as_bytes()[0] as char)
        && interpolation_is_single_string(expression)
    {
        return true;
    }
    if expression.starts_with(':') && !expression.starts_with("::") {
        return expression.len() > 1;
    }
    if expression.starts_with('%') {
        return expression.starts_with("%(")
            || matches!(
                expression.as_bytes().get(1),
                Some(b'q' | b'Q' | b'w' | b'i' | b'I')
            )
            || expression
                .as_bytes()
                .get(1)
                .is_some_and(|delimiter| !delimiter.is_ascii_alphanumeric());
    }
    if ((expression.starts_with('[') && expression.ends_with(']'))
        || (expression.starts_with('{') && expression.ends_with('}')))
        && interpolation_composite_is_literal(expression)
    {
        return true;
    }
    let numeric = expression
        .bytes()
        .all(|byte| byte.is_ascii_digit() || matches!(byte, b'_' | b'.' | b'+' | b'-' | b'e' | b'E' | b'x' | b'o' | b'b' | b'a'..=b'f' | b'A'..=b'F'));
    numeric
        && expression.bytes().any(|byte| byte.is_ascii_digit())
        && expression
            .as_bytes()
            .first()
            .is_some_and(|byte| byte.is_ascii_digit() || matches!(byte, b'+' | b'-'))
}

fn interpolation_is_single_string(expression: &str) -> bool {
    let parsed = ruby_prism::parse(expression.as_bytes());
    if parsed.errors().count() != 0 {
        return false;
    }
    parsed
        .node()
        .as_program_node()
        .and_then(|program| {
            let body = program.statements().body();
            (body.len() == 1).then(|| body.first()).flatten()
        })
        .is_some_and(|node| node.as_string_node().is_some())
}

fn interpolation_composite_is_literal(expression: &str) -> bool {
    #[derive(Default)]
    struct Dynamic(bool);

    impl<'pr> Visit<'pr> for Dynamic {
        fn visit_call_node(&mut self, _node: &ruby_prism::CallNode<'pr>) {
            self.0 = true;
        }

        fn visit_local_variable_read_node(
            &mut self,
            _node: &ruby_prism::LocalVariableReadNode<'pr>,
        ) {
            self.0 = true;
        }

        fn visit_instance_variable_read_node(
            &mut self,
            _node: &ruby_prism::InstanceVariableReadNode<'pr>,
        ) {
            self.0 = true;
        }

        fn visit_class_variable_read_node(
            &mut self,
            _node: &ruby_prism::ClassVariableReadNode<'pr>,
        ) {
            self.0 = true;
        }

        fn visit_global_variable_read_node(
            &mut self,
            _node: &ruby_prism::GlobalVariableReadNode<'pr>,
        ) {
            self.0 = true;
        }

        fn visit_constant_read_node(&mut self, _node: &ruby_prism::ConstantReadNode<'pr>) {
            self.0 = true;
        }

        fn visit_constant_path_node(&mut self, _node: &ruby_prism::ConstantPathNode<'pr>) {
            self.0 = true;
        }

        fn visit_numbered_reference_read_node(
            &mut self,
            _node: &ruby_prism::NumberedReferenceReadNode<'pr>,
        ) {
            self.0 = true;
        }

        fn visit_regular_expression_node(
            &mut self,
            _node: &ruby_prism::RegularExpressionNode<'pr>,
        ) {
            self.0 = true;
        }

        fn visit_interpolated_regular_expression_node(
            &mut self,
            _node: &ruby_prism::InterpolatedRegularExpressionNode<'pr>,
        ) {
            self.0 = true;
        }

        fn visit_self_node(&mut self, _node: &ruby_prism::SelfNode<'pr>) {
            self.0 = true;
        }
    }

    let parsed = ruby_prism::parse(expression.as_bytes());
    if parsed.errors().count() != 0 {
        return false;
    }
    let mut dynamic = Dynamic::default();
    dynamic.visit(&parsed.node());
    !dynamic.0
}

fn array_percent_interpolation(source: &str, opening: usize, expression: &str) -> bool {
    let line_start = source[..opening].rfind('\n').map_or(0, |at| at + 1);
    let prefix = source[line_start..opening].trim_start();
    if !(prefix.starts_with("%W[") || prefix.starts_with("%I[")) {
        return false;
    }
    let value = interpolation_literal_value(expression, None);
    value.is_empty() || value.bytes().any(|byte| byte.is_ascii_whitespace())
}

fn heredoc_trailing_space_interpolation(source: &str, closing: usize, expression: &str) -> bool {
    let value = interpolation_literal_value(expression, None);
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_whitespace()) {
        return false;
    }
    let line_end = source[closing + 1..]
        .find('\n')
        .map_or(source.len(), |at| closing + 1 + at);
    source[closing + 1..line_end].trim().is_empty()
        && source[..closing].lines().any(|line| line.contains("<<"))
}

fn regexp_array_interpolation(source: &str, opening: usize, expression: &str) -> bool {
    if !(expression.starts_with('[') || expression.starts_with("%w")) {
        return false;
    }
    let line_start = source[..opening].rfind('\n').map_or(0, |at| at + 1);
    source[line_start..opening].trim_start().starts_with('/')
}

fn interpolation_literal_value(expression: &str, outer_delimiter: Option<u8>) -> String {
    let expression = expression.trim();
    if expression == "nil" {
        return String::new();
    }
    let string = decoded_string_literal(expression);
    if let Some(value) = string {
        return encode_interpolation_value(&value, outer_delimiter);
    }
    if let Some(symbol) = expression.strip_prefix(':') {
        let symbol = if symbol.len() >= 2
            && matches!(symbol.as_bytes()[0], b'\'' | b'"')
            && symbol.as_bytes()[symbol.len() - 1] == symbol.as_bytes()[0]
        {
            &symbol[1..symbol.len() - 1]
        } else {
            symbol
        };
        return encode_interpolation_value(symbol, outer_delimiter);
    }
    if let Some(number) = interpolation_number(expression) {
        return number;
    }
    if expression.starts_with(['[', '{'])
        || expression.starts_with("%w[")
        || expression.starts_with("%i[")
        || expression.starts_with("%I[")
    {
        if let Some(value) = ruby_literal_inspect(expression) {
            return encode_interpolation_value(&value, outer_delimiter.or(Some(b'"')));
        }
    }
    expression.to_string()
}

fn decoded_string_literal(expression: &str) -> Option<String> {
    if expression.len() >= 2
        && matches!(expression.as_bytes()[0], b'\'' | b'"')
        && expression.as_bytes()[expression.len() - 1] == expression.as_bytes()[0]
    {
        let delimiter = expression.as_bytes()[0];
        Some(decode_interpolated_string(
            &expression[1..expression.len() - 1],
            delimiter == b'"',
        ))
    } else if expression.starts_with("%q(") && expression.ends_with(')') {
        Some(decode_interpolated_string(
            &expression[3..expression.len() - 1],
            false,
        ))
    } else if (expression.starts_with("%(") || expression.starts_with("%Q("))
        && expression.ends_with(')')
    {
        let start = if expression.starts_with("%Q(") { 3 } else { 2 };
        Some(decode_interpolated_string(
            &expression[start..expression.len() - 1],
            true,
        ))
    } else {
        None
    }
}

fn escape_regexp_slashes(value: &str) -> String {
    let mut escaped = String::new();
    let mut backslashes = 0usize;
    for character in value.chars() {
        if character == '/' && backslashes % 2 == 0 {
            escaped.push('\\');
        }
        escaped.push(character);
        if character == '\\' {
            backslashes += 1;
        } else {
            backslashes = 0;
        }
    }
    escaped
}

fn ruby_literal_inspect(source: &str) -> Option<String> {
    let mut parser = LiteralInspectParser {
        source: source.as_bytes(),
        position: 0,
    };
    let value = parser.value()?;
    parser.whitespace();
    (parser.position == parser.source.len()).then_some(value)
}

struct LiteralInspectParser<'a> {
    source: &'a [u8],
    position: usize,
}

impl LiteralInspectParser<'_> {
    fn whitespace(&mut self) {
        while self
            .source
            .get(self.position)
            .is_some_and(u8::is_ascii_whitespace)
        {
            self.position += 1;
        }
    }

    fn value(&mut self) -> Option<String> {
        self.whitespace();
        match self.source.get(self.position).copied()? {
            b'{' => self.hash(),
            b'[' => self.array(),
            b'\'' | b'"' => self.string().map(|value| ruby_inspect_string(&value)),
            b':' => self.symbol(),
            b'%' if self.source.get(self.position + 1).is_some_and(|kind| {
                matches!(kind, b'w' | b'i' | b'I')
                    && self.source.get(self.position + 2) == Some(&b'[')
            }) =>
            {
                self.percent_array()
            }
            _ => self.atom(),
        }
    }

    fn hash(&mut self) -> Option<String> {
        self.position += 1;
        let mut pairs = Vec::new();
        loop {
            self.whitespace();
            if self.source.get(self.position) == Some(&b'}') {
                self.position += 1;
                break;
            }
            let saved = self.position;
            let identifier = self.identifier();
            self.whitespace();
            let key = if identifier.is_some() && self.source.get(self.position) == Some(&b':') {
                self.position += 1;
                format!(":{}", identifier.unwrap())
            } else {
                self.position = saved;
                let key = self.value()?;
                self.whitespace();
                if self.source.get(self.position..self.position + 2) != Some(b"=>") {
                    return None;
                }
                self.position += 2;
                key
            };
            let value = self.value()?;
            pairs.push(format!("{key}=>{value}"));
            self.whitespace();
            match self.source.get(self.position) {
                Some(b',') => self.position += 1,
                Some(b'}') => continue,
                _ => return None,
            }
        }
        Some(format!("{{{}}}", pairs.join(", ")))
    }

    fn array(&mut self) -> Option<String> {
        self.position += 1;
        let mut values = Vec::new();
        loop {
            self.whitespace();
            if self.source.get(self.position) == Some(&b']') {
                self.position += 1;
                break;
            }
            values.push(self.value()?);
            self.whitespace();
            match self.source.get(self.position) {
                Some(b',') => self.position += 1,
                Some(b']') => continue,
                _ => return None,
            }
        }
        Some(format!("[{}]", values.join(", ")))
    }

    fn percent_array(&mut self) -> Option<String> {
        self.position += 3;
        let start = self.position;
        while self.source.get(self.position) != Some(&b']') {
            self.position += 1;
            if self.position >= self.source.len() {
                return None;
            }
        }
        let words = std::str::from_utf8(&self.source[start..self.position])
            .ok()?
            .split_whitespace()
            .map(ruby_inspect_string)
            .collect::<Vec<_>>();
        self.position += 1;
        Some(format!("[{}]", words.join(", ")))
    }

    fn string(&mut self) -> Option<String> {
        let delimiter = *self.source.get(self.position)?;
        self.position += 1;
        let mut raw = String::new();
        while let Some(byte) = self.source.get(self.position).copied() {
            self.position += 1;
            if byte == delimiter {
                return Some(decode_interpolated_string(&raw, delimiter == b'"'));
            }
            if byte == b'\\' {
                raw.push('\\');
                raw.push(*self.source.get(self.position)? as char);
                self.position += 1;
            } else {
                raw.push(byte as char);
            }
        }
        None
    }

    fn symbol(&mut self) -> Option<String> {
        self.position += 1;
        if self
            .source
            .get(self.position)
            .is_some_and(|byte| matches!(byte, b'\'' | b'"'))
        {
            let value = self.string()?;
            if value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            {
                Some(format!(":{value}"))
            } else {
                Some(format!(":{}", ruby_inspect_string(&value)))
            }
        } else {
            self.identifier().map(|name| format!(":{name}"))
        }
    }

    fn identifier(&mut self) -> Option<String> {
        let start = self.position;
        while self
            .source
            .get(self.position)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            self.position += 1;
        }
        (self.position > start)
            .then(|| String::from_utf8_lossy(&self.source[start..self.position]).into_owned())
    }

    fn atom(&mut self) -> Option<String> {
        let start = self.position;
        while self
            .source
            .get(self.position)
            .is_some_and(|byte| !byte.is_ascii_whitespace() && !matches!(byte, b',' | b'}' | b']'))
        {
            self.position += 1;
        }
        let atom = std::str::from_utf8(&self.source[start..self.position]).ok()?;
        interpolation_number(atom).or_else(|| (!atom.is_empty()).then(|| atom.to_string()))
    }
}

fn ruby_inspect_string(value: &str) -> String {
    let mut inspected = String::from("\"");
    for character in value.chars() {
        match character {
            '\\' => inspected.push_str("\\\\"),
            '"' => inspected.push_str("\\\""),
            '\n' => inspected.push_str("\\n"),
            '\r' => inspected.push_str("\\r"),
            '\t' => inspected.push_str("\\t"),
            other => inspected.push(other),
        }
    }
    inspected.push('"');
    inspected
}

fn decode_interpolated_string(value: &str, double_quoted: bool) -> String {
    if !double_quoted {
        return value.replace("\\\\", "\\").replace("\\'", "'");
    }
    let mut decoded = String::new();
    let mut chars = value.chars();
    while let Some(character) = chars.next() {
        if character != '\\' {
            decoded.push(character);
            continue;
        }
        match chars.next() {
            Some('n') => decoded.push('\n'),
            Some('t') => decoded.push('\t'),
            Some('r') => decoded.push('\r'),
            Some(next) => decoded.push(next),
            None => decoded.push('\\'),
        }
    }
    decoded
}

fn encode_interpolation_value(value: &str, outer_delimiter: Option<u8>) -> String {
    let mut encoded = String::new();
    for character in value.chars() {
        match character {
            '\\' => encoded.push_str("\\\\"),
            '"' if outer_delimiter != Some(b'\'') => encoded.push_str("\\\""),
            '\'' if outer_delimiter == Some(b'\'') => encoded.push_str("\\'"),
            other => encoded.push(other),
        }
    }
    encoded
}

fn interpolation_number(expression: &str) -> Option<String> {
    let normalized = expression.replace('_', "");
    let negative = normalized.starts_with('-');
    let unsigned = normalized.strip_prefix('-').unwrap_or(&normalized);
    let parsed_integer = if let Some(digits) = unsigned.strip_prefix("0x") {
        i128::from_str_radix(digits, 16).ok()
    } else if let Some(digits) = unsigned.strip_prefix("0b") {
        i128::from_str_radix(digits, 2).ok()
    } else if let Some(digits) = unsigned.strip_prefix("0o") {
        i128::from_str_radix(digits, 8).ok()
    } else if unsigned.chars().all(|c| c.is_ascii_digit()) {
        unsigned.parse::<i128>().ok()
    } else {
        None
    };
    if let Some(value) = parsed_integer {
        return Some(if negative {
            format!("-{value}")
        } else {
            value.to_string()
        });
    }
    normalized.parse::<f64>().ok().map(|value| {
        if !normalized.contains(['e', 'E']) && normalized.contains('.') && value.fract() == 0.0 {
            format!("{value:.1}")
        } else {
            value.to_string()
        }
    })
}

fn interpolation_outer_delimiter(source: &str, opening: usize) -> Option<u8> {
    let line_start = source[..opening].rfind('\n').map_or(0, |at| at + 1);
    source.as_bytes()[line_start..opening]
        .iter()
        .copied()
        .find(|byte| matches!(byte, b'\'' | b'"'))
}

struct SelectByRange;

struct SelectByKind;

struct SafeNavigation;

struct RedundantTypeConversion;

struct ConditionalAssignment;

struct Void;

struct Debugger;

impl Cop for Debugger {
    fn name(&self) -> &'static str {
        "Lint/Debugger"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        let debugger_methods = debugger_configured_entries(&cop_context, "DebuggerMethods");
        let debugger_requires = debugger_configured_entries(&cop_context, "DebuggerRequires");
        let chained_name = debugger_chained_name(&call, source);
        let debugger_method = debugger_methods
            .iter()
            .any(|method| method == &chained_name);
        let debugger_require = call.name().as_slice() == b"require"
            && call.arguments().is_some_and(|arguments| {
                let values = arguments.arguments().iter().collect::<Vec<_>>();
                values.len() == 1
                    && values[0].as_string_node().is_some_and(|string| {
                        let value = String::from_utf8_lossy(string.unescaped()).to_string();
                        debugger_requires.contains(&value)
                    })
            });
        if !debugger_method && !debugger_require {
            return;
        }
        let no_arguments = call
            .arguments()
            .is_none_or(|arguments| arguments.arguments().is_empty());
        if no_arguments {
            if let Some(parent_call) = ancestors.iter().rev().find_map(Node::as_call_node) {
                let inside_parent_block = parent_call.block().is_some_and(|block| {
                    block.location().start_offset() <= call.location().start_offset()
                        && call.location().end_offset() <= block.location().end_offset()
                });
                let inside_begin_or_lambda = ancestors
                    .iter()
                    .rev()
                    .take_while(|ancestor| {
                        ancestor.location().start_offset() >= parent_call.location().start_offset()
                    })
                    .any(|ancestor| {
                        ancestor.as_begin_node().is_some() || ancestor.as_lambda_node().is_some()
                    });
                if !inside_parent_block && !inside_begin_or_lambda {
                    return;
                }
            }
        }
        let start = call.location().start_offset();
        let mut end = call.block().map_or(call.location().end_offset(), |block| {
            block.location().start_offset()
        });
        while end > start && source.as_bytes()[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        let range = start..end;
        cop_context.report(
            format!("Remove debugger entry point `{}`.", &source[range.clone()]),
            range,
        );
    }
}

fn debugger_configured_entries(context: &CopContext<'_, '_>, key: &str) -> Vec<String> {
    let configured = context.config_values(key);
    if !configured.is_empty() {
        return configured.to_vec();
    }
    context
        .config_map(key)
        .into_iter()
        .flat_map(|groups| groups.values())
        .filter(|value| !matches!(value.as_str(), "" | "~" | "nil" | "false"))
        .flat_map(|value| value.lines().map(str::to_string))
        .collect()
}

fn debugger_chained_name(call: &CallNode<'_>, source: &str) -> String {
    let name = String::from_utf8_lossy(call.name().as_slice()).to_string();
    let Some(receiver) = call.receiver() else {
        return name;
    };
    let receiver = if let Some(receiver_call) = receiver.as_call_node() {
        debugger_chained_name(&receiver_call, source)
    } else {
        source_at(source, &receiver.location())
            .trim_start_matches("::")
            .to_string()
    };
    format!("{receiver}.{name}")
}

impl Cop for Void {
    fn name(&self) -> &'static str {
        "Lint/Void"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(statements) = node.as_statements_node() else {
            return;
        };
        let body = statements.body().iter().collect::<Vec<_>>();
        let direct_parent = ancestors.last();
        let enclosing_definition = direct_parent.and_then(Node::as_def_node).filter(|definition| {
            definition.body().is_some_and(|body| {
                body.location().start_offset() == node.location().start_offset()
                    && body.location().end_offset() == node.location().end_offset()
            })
        });
        let ensure_body = ancestors.iter().rev().any(|ancestor| {
            ancestor.as_ensure_node().is_some_and(|ensure_node| {
                ensure_node.statements().is_some_and(|body| {
                    body.location().start_offset() == node.location().start_offset()
                        && body.location().end_offset() == node.location().end_offset()
                })
            })
        }) || source[..node.location().start_offset()]
            .lines()
            .rev()
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| line.trim() == "ensure");
        let all_expressions = body.len() > 1
            && enclosing_definition.as_ref().is_some_and(|definition| {
                definition.name().as_slice() == b"initialize"
                    || void_setter_name(definition.name().as_slice())
            }) || direct_parent.is_some_and(|parent| {
            parent.as_for_node().is_some() || parent.as_ensure_node().is_some()
        }) || ensure_body
            || direct_parent.and_then(Node::as_block_node).is_some_and(|block| {
                ancestors.iter().rev().any(|ancestor| {
                    ancestor.as_call_node().is_some_and(|call| {
                        call.name().as_slice() == b"tap"
                            && call.block().is_some_and(|candidate| {
                                candidate.location().start_offset()
                                    == block.location().start_offset()
                                    && candidate.location().end_offset()
                                        == block.location().end_offset()
                            })
                    })
                })
            });
        let correctable = !enclosing_definition
            .as_ref()
            .is_some_and(|definition| void_setter_name(definition.name().as_slice()));
        let count = if all_expressions {
            body.len()
        } else {
            body.len().saturating_sub(1)
        };
        for expression in body.iter().take(count) {
            check_void_expression(
                expression,
                ancestors,
                source,
                context,
                self.name(),
                correctable,
            );
        }
    }
}

fn void_setter_name(name: &[u8]) -> bool {
    name.ends_with(b"=") && !matches!(name, b"==" | b"===" | b"!=" | b"<=" | b">=")
}

fn check_void_expression(
    node: &Node<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut Context,
    cop: &'static str,
    correctable: bool,
) {
    if let Some(parentheses) = node.as_parentheses_node() {
        if let Some(expression) = parentheses.body().and_then(single_expression) {
            check_void_expression(&expression, ancestors, source, context, cop, correctable);
        }
        return;
    }
    if let Some(conditional) = node.as_if_node() {
        if let Some(statements) = conditional.statements() {
            check_void_if_body(&statements, ancestors, source, context, cop);
        }
        return;
    }
    if let Some(conditional) = node.as_unless_node() {
        if let Some(statements) = conditional.statements() {
            check_void_if_body(&statements, ancestors, source, context, cop);
        }
        return;
    }
    if let Some(case_node) = node.as_case_node() {
        for branch in case_node.conditions().iter() {
            if let Some(statements) = branch.as_when_node().and_then(|branch| branch.statements()) {
                check_void_branch_tail(&statements, ancestors, source, context, cop);
            }
        }
        if let Some(statements) = case_node
            .else_clause()
            .and_then(|branch| branch.statements())
        {
            check_void_branch_tail(&statements, ancestors, source, context, cop);
        }
        return;
    }
    if let Some(case_node) = node.as_case_match_node() {
        for branch in case_node.conditions().iter() {
            if let Some(statements) = branch.as_in_node().and_then(|branch| branch.statements()) {
                check_void_branch_tail(&statements, ancestors, source, context, cop);
            }
        }
        if let Some(statements) = case_node
            .else_clause()
            .and_then(|branch| branch.statements())
        {
            check_void_branch_tail(&statements, ancestors, source, context, cop);
        }
        return;
    }

    let mut cop_context = context.cop_context(cop, source, ancestors);
    let range = node.location().start_offset()..node.location().end_offset();
    let expression = &source[range.clone()];
    if let Some(call) = node.as_call_node() {
        let method = call.name().as_slice();
        const OPERATORS: &[&[u8]] = &[
            b"*", b"/", b"%", b"+", b"-", b"==", b"===", b"!=", b"<", b">", b"<=", b">=", b"<=>",
            b"+@", b"-@", b"~", b"!",
        ];
        if OPERATORS.contains(&method) {
            let binary = !matches!(method, b"+@" | b"-@" | b"~" | b"!");
            if binary
                && call.call_operator_loc().is_some()
                && call
                    .arguments()
                    .is_none_or(|arguments| arguments.arguments().is_empty())
            {
                return;
            }
            if let Some(selector) = call.message_loc() {
                let selector = selector.start_offset()..selector.end_offset();
                let method = String::from_utf8_lossy(method);
                let message = format!("Operator `{method}` used in void context.");
                if correctable {
                    let replacement = call.receiver().map_or_else(String::new, |receiver| {
                        let mut values = vec![source_at(source, &receiver.location()).to_string()];
                        if !matches!(call.name().as_slice(), b"+@" | b"-@" | b"~" | b"!") {
                            if let Some(arguments) = call.arguments() {
                                if let (Some(opening), Some(closing)) =
                                    (call.opening_loc(), call.closing_loc())
                                {
                                    values.push(
                                        source[opening.start_offset()..closing.end_offset()]
                                            .to_string(),
                                    );
                                } else {
                                    values.extend(arguments.arguments().iter().map(|argument| {
                                        source_at(source, &argument.location()).to_string()
                                    }));
                                }
                            }
                        }
                        values.join("\n")
                    });
                    cop_context.replace(message, selector, range, replacement);
                } else {
                    cop_context.report(message, selector);
                }
            }
            return;
        }
        if cop_context.config_bool("CheckForMethodsWithNoSideEffects", false) {
            let suggestion = match method {
                b"collect" | b"map" => Some("each".to_string()),
                b"capitalize" | b"chomp" | b"chop" | b"compact" | b"delete_prefix"
                | b"delete_suffix" | b"downcase" | b"encode" | b"flatten" | b"gsub" | b"lstrip"
                | b"merge" | b"next" | b"reject" | b"reverse" | b"rotate" | b"rstrip"
                | b"scrub" | b"select" | b"shuffle" | b"slice" | b"sort" | b"sort_by"
                | b"squeeze" | b"strip" | b"sub" | b"succ" | b"swapcase" | b"tr" | b"tr_s"
                | b"transform_values" | b"unicode_normalize" | b"uniq" | b"upcase" => {
                    Some(format!("{}!", String::from_utf8_lossy(method)))
                }
                _ => None,
            };
            if let Some(suggestion) = suggestion {
                let method = String::from_utf8_lossy(method);
                let selector = call.message_loc().unwrap_or_else(|| call.location());
                cop_context.replace(
                    format!(
                        "Method `#{method}` used in void context. Did you mean `#{suggestion}`?"
                    ),
                    range.clone(),
                    selector,
                    suggestion,
                );
                return;
            }
        }
    }
    if void_literal(node) || frozen_void_literal(node) {
        report_void(
            &mut cop_context,
            format!("Literal `{expression}` used in void context."),
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if void_variable(node) {
        report_void(
            &mut cop_context,
            format!("Variable `{expression}` used in void context."),
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if node.as_constant_read_node().is_some() || node.as_constant_path_node().is_some() {
        report_void(
            &mut cop_context,
            format!("Constant `{expression}` used in void context."),
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if node.as_self_node().is_some() {
        report_void(
            &mut cop_context,
            "`self` used in void context.",
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if node.as_defined_node().is_some()
        || node.as_lambda_node().is_some()
        || void_proc_expression(node)
    {
        report_void(
            &mut cop_context,
            format!("`{expression}` used in void context."),
            range.clone(),
            expression.to_string(),
            correctable,
        );
    } else if node.as_source_encoding_node().is_some() {
        report_void(
            &mut cop_context,
            format!("Variable `{expression}` used in void context."),
            range,
            expression.to_string(),
            correctable,
        );
    }
}

fn report_void(
    context: &mut CopContext<'_, '_>,
    message: impl Into<String>,
    range: std::ops::Range<usize>,
    _source: String,
    correctable: bool,
) {
    let message = message.into();
    if correctable {
        let line_start = context.source_file().line_start(range.start);
        let line_end = context.source_file().line_end(range.end);
        let whole_variable_line =
            message.starts_with("Variable `") || message.starts_with("Constant `");
        let indented_line = (line_start < range.start || whole_variable_line)
            && context.source()[line_start..range.start]
                .bytes()
                .all(|byte| matches!(byte, b' ' | b'\t'))
            && context.source()[range.end..line_end].trim().is_empty();
        let edit = if indented_line {
            line_start
                ..context
                    .source()
                    .get(line_end..)
                    .and_then(|tail| tail.strip_prefix("\r\n").map(|_| line_end + 2))
                    .or_else(|| {
                        context
                            .source()
                            .get(line_end..)
                            .and_then(|tail| tail.strip_prefix('\n').map(|_| line_end + 1))
                    })
                    .unwrap_or(line_end)
        } else {
            range.clone()
        };
        context.replace(message, range, edit, "");
    } else {
        context.report(message, range);
    }
}

fn void_proc_expression(node: &Node<'_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return false;
    };
    call.block().is_some()
        && (matches!(call.name().as_slice(), b"lambda" | b"proc")
            || call.name().as_slice() == b"new"
                && call.receiver().is_some_and(|receiver| {
                    receiver
                        .as_constant_read_node()
                        .is_some_and(|constant| constant.name().as_slice() == b"Proc")
                }))
}

fn frozen_void_literal(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.name().as_slice() == b"freeze"
            && call
                .arguments()
                .is_none_or(|arguments| arguments.arguments().is_empty())
            && call.receiver().as_ref().is_some_and(void_literal)
    })
}

fn check_void_branch_tail(
    statements: &ruby_prism::StatementsNode<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut Context,
    cop: &'static str,
) {
    let body = statements.body().iter().collect::<Vec<_>>();
    if body.len() == 1 {
        check_void_expression(&body[0], ancestors, source, context, cop, false);
    }
}

fn check_void_if_body(
    statements: &ruby_prism::StatementsNode<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut Context,
    cop: &'static str,
) {
    let body = statements.body().iter().collect::<Vec<_>>();
    if body.len() == 1
        && body[0].as_if_node().is_none()
        && body[0].as_unless_node().is_none()
        && body[0].as_case_node().is_none()
        && body[0].as_case_match_node().is_none()
    {
        check_void_expression(&body[0], ancestors, source, context, cop, false);
    }
}

fn void_variable(node: &Node<'_>) -> bool {
    node.as_local_variable_read_node().is_some()
        || node.as_instance_variable_read_node().is_some()
        || node.as_class_variable_read_node().is_some()
        || node.as_global_variable_read_node().is_some()
        || node.as_back_reference_read_node().is_some()
        || node.as_numbered_reference_read_node().is_some()
}

fn void_literal(node: &Node<'_>) -> bool {
    node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_interpolated_symbol_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
        || node.as_array_node().is_some_and(|array| {
            array
                .elements()
                .iter()
                .all(|element| void_literal(&element) || frozen_void_literal(&element))
        })
        || node.as_hash_node().is_some_and(|hash| {
            hash.elements().iter().all(|element| {
                element
                    .as_assoc_node()
                    .is_some_and(|pair| void_literal(&pair.key()) && void_literal(&pair.value()))
            })
        })
}

struct ConditionalBranch<'pr> {
    tail: Node<'pr>,
    statements: usize,
}

struct ConditionalAssignmentParts<'pr> {
    lhs: String,
    kind: &'static str,
    value: Node<'pr>,
}

impl Cop for ConditionalAssignment {
    fn name(&self) -> &'static str {
        "Style/ConditionalAssignment"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        let style = cop_context.policy().enforced_style("assign_to_condition");
        let single_line_only = cop_context.config_bool("SingleLineConditionsOnly", true);
        let include_ternary = cop_context.config_bool("IncludeTernaryExpressions", true);

        if style == "assign_inside_condition" {
            let Some(assignment) = conditional_assignment_parts(node, source) else {
                return;
            };
            let Some(branches) = conditional_assignment_branches(&assignment.value, true) else {
                return;
            };
            if !include_ternary && conditional_ternary(&assignment.value)
                || single_line_only && branches.iter().any(|branch| branch.statements > 1)
            {
                return;
            }
            let range = node.location().start_offset()..node.location().end_offset();
            let prefix =
                source[range.start..assignment.value.location().start_offset()].to_string();
            let mut edits = vec![(
                range.start..assignment.value.location().start_offset(),
                String::new(),
            )];
            if let Some(parentheses) = assignment.value.as_parentheses_node() {
                edits.push((
                    parentheses.opening_loc().start_offset()
                        ..parentheses.opening_loc().end_offset(),
                    String::new(),
                ));
                edits.push((
                    parentheses.closing_loc().start_offset()
                        ..parentheses.closing_loc().end_offset(),
                    String::new(),
                ));
            }
            edits.extend(branches.iter().map(|branch| {
                let at = branch.tail.location().start_offset();
                (at..at, prefix.clone())
            }));
            let multiline_branch = source[assignment.value.location().start_offset()
                ..assignment.value.location().end_offset()]
                .contains("<<")
                || branches.iter().any(|branch| {
                    source
                        [branch.tail.location().start_offset()..branch.tail.location().end_offset()]
                        .contains('\n')
                });
            if !multiline_branch {
                for (relative, line) in source[range.clone()].split_inclusive('\n').enumerate() {
                    if relative == 0 || line.is_empty() {
                        continue;
                    }
                    let line_start = range.start
                        + source[range.start..]
                            .split_inclusive('\n')
                            .take(relative)
                            .map(str::len)
                            .sum::<usize>();
                    let leading = line.len() - line.trim_start_matches(' ').len();
                    let trimmed = line.trim_start();
                    let structural = ["else", "elsif", "end", "when", "in "]
                        .iter()
                        .any(|keyword| trimmed.starts_with(keyword));
                    let desired = if structural { 0 } else { 2 };
                    if leading > desired {
                        edits.push((line_start..line_start + leading - desired, String::new()));
                    }
                }
            }
            cop_context.replace_many(
                "Assign variables inside of conditionals.",
                range.clone(),
                edits,
            );
            return;
        }

        if node.as_if_node().is_some_and(|conditional| {
            conditional
                .if_keyword_loc()
                .is_some_and(|keyword| keyword.as_slice() == b"elsif")
        }) {
            return;
        }
        if !include_ternary && conditional_ternary(node) {
            return;
        }
        if node.as_if_node().is_none()
            && node.as_unless_node().is_none()
            && node.as_case_node().is_none()
            && node.as_case_match_node().is_none()
        {
            return;
        }
        let Some(branches) = conditional_assignment_branches(node, false) else {
            return;
        };
        if single_line_only && branches.iter().any(|branch| branch.statements > 1) {
            return;
        }
        let assignments = branches
            .iter()
            .map(|branch| conditional_assignment_parts(&branch.tail, source))
            .collect::<Option<Vec<_>>>();
        let Some(assignments) = assignments else {
            return;
        };
        let Some(first) = assignments.first() else {
            return;
        };
        if first.kind == "multi" {
            return;
        }
        if assignments
            .iter()
            .any(|assignment| assignment.kind != first.kind || assignment.lhs != first.lhs)
        {
            return;
        }
        if let Some(maximum) = cop_context
            .related_config_value("Layout/LineLength", "Max")
            .and_then(|maximum| maximum.parse::<usize>().ok())
        {
            let longest = source[range_for_node(node)]
                .lines()
                .map(|line| {
                    line.trim_start()
                        .strip_prefix(&first.lhs)
                        .map_or(line.len(), |rest| rest.trim_start().len())
                })
                .max()
                .unwrap_or(0);
            if longest + first.lhs.trim_end().len() + 1 > maximum {
                return;
            }
        }
        let range = node.location().start_offset()..node.location().end_offset();
        let prefix = format!("{} ", first.lhs.trim_end());
        let mut edits = vec![(range.start..range.start, prefix.clone())];
        if first.kind == "call" && conditional_ternary(node) {
            edits.push((range.start..range.start, "(".to_string()));
            edits.push((range.end..range.end, ")".to_string()));
        }
        edits.extend(
            assignments
                .iter()
                .zip(&branches)
                .map(|(assignment, branch)| {
                    (
                        branch.tail.location().start_offset()
                            ..assignment.value.location().start_offset(),
                        String::new(),
                    )
                }),
        );
        for (assignment, _branch) in assignments.iter().zip(&branches) {
            if assignment
                .value
                .as_array_node()
                .is_some_and(|array| array.opening_loc().is_none())
            {
                let value = assignment.value.location();
                edits.push((value.start_offset()..value.start_offset(), "[".to_string()));
                edits.push((value.end_offset()..value.end_offset(), "]".to_string()));
            }
        }
        if cop_context.related_config_value("Layout/EndAlignment", "EnforcedStyleAlignWith")
            == Some("keyword")
            && source[range.clone()].contains('\n')
        {
            let end_start = range.end.saturating_sub(3);
            if source.get(end_start..range.end) == Some("end") {
                let line_start = source[..end_start].rfind('\n').map_or(0, |at| at + 1);
                let base_indent =
                    range.start - source[..range.start].rfind('\n').map_or(0, |at| at + 1);
                edits.push((
                    line_start..end_start,
                    " ".repeat(base_indent + prefix.len()),
                ));
            }
        }
        cop_context.replace_many(
            "Use the return of the conditional for variable assignment and comparison.",
            range.clone(),
            edits,
        );
    }
}

fn conditional_assignment_branches<'pr>(
    node: &Node<'pr>,
    allow_missing_else: bool,
) -> Option<Vec<ConditionalBranch<'pr>>> {
    if let Some(parentheses) = node.as_parentheses_node() {
        return parentheses
            .body()
            .and_then(single_expression)
            .and_then(|expression| {
                conditional_assignment_branches(&expression, allow_missing_else)
            });
    }
    if let Some(conditional) = node.as_if_node() {
        let mut branches = Vec::new();
        branches.push(conditional_statement_branch(conditional.statements())?);
        let Some(mut subsequent) = conditional.subsequent() else {
            return (allow_missing_else && branches.len() > 1).then_some(branches);
        };
        loop {
            if let Some(elsif) = subsequent.as_if_node() {
                branches.push(conditional_statement_branch(elsif.statements())?);
                let Some(next) = elsif.subsequent() else {
                    return (allow_missing_else && branches.len() > 1).then_some(branches);
                };
                subsequent = next;
                continue;
            }
            let else_node = subsequent.as_else_node()?;
            branches.push(conditional_statement_branch(else_node.statements())?);
            return Some(branches);
        }
    }
    if let Some(conditional) = node.as_unless_node() {
        let first = conditional_statement_branch(conditional.statements())?;
        return if let Some(else_node) = conditional.else_clause() {
            Some(vec![
                first,
                conditional_statement_branch(else_node.statements())?,
            ])
        } else {
            allow_missing_else.then_some(vec![first])
        };
    }
    if let Some(case_node) = node.as_case_node() {
        let mut branches = case_node
            .conditions()
            .iter()
            .map(|branch| {
                branch
                    .as_when_node()
                    .and_then(|branch| conditional_statement_branch(branch.statements()))
            })
            .collect::<Option<Vec<_>>>()?;
        branches.push(conditional_statement_branch(
            case_node.else_clause()?.statements(),
        )?);
        return Some(branches);
    }
    if let Some(case_node) = node.as_case_match_node() {
        let mut branches = case_node
            .conditions()
            .iter()
            .map(|branch| {
                branch
                    .as_in_node()
                    .and_then(|branch| conditional_statement_branch(branch.statements()))
            })
            .collect::<Option<Vec<_>>>()?;
        branches.push(conditional_statement_branch(
            case_node.else_clause()?.statements(),
        )?);
        return Some(branches);
    }
    None
}

fn range_for_node(node: &Node<'_>) -> std::ops::Range<usize> {
    node.location().start_offset()..node.location().end_offset()
}

fn conditional_statement_branch<'pr>(
    statements: Option<ruby_prism::StatementsNode<'pr>>,
) -> Option<ConditionalBranch<'pr>> {
    let body = statements?.body();
    Some(ConditionalBranch {
        tail: body.last()?,
        statements: body.len(),
    })
}

fn conditional_ternary(node: &Node<'_>) -> bool {
    if let Some(expression) = node
        .as_parentheses_node()
        .and_then(|parentheses| parentheses.body().and_then(single_expression))
    {
        return expression
            .as_if_node()
            .is_some_and(|conditional| conditional.if_keyword_loc().is_none());
    }
    node.as_if_node()
        .is_some_and(|conditional| conditional.if_keyword_loc().is_none())
}

fn conditional_assignment_parts<'pr>(
    node: &Node<'pr>,
    source: &str,
) -> Option<ConditionalAssignmentParts<'pr>> {
    let (kind, value) = if let Some(write) = node.as_local_variable_write_node() {
        ("local", write.value())
    } else if let Some(write) = node.as_instance_variable_write_node() {
        ("instance", write.value())
    } else if let Some(write) = node.as_class_variable_write_node() {
        ("class", write.value())
    } else if let Some(write) = node.as_global_variable_write_node() {
        ("global", write.value())
    } else if let Some(write) = node.as_constant_write_node() {
        ("constant", write.value())
    } else if let Some(write) = node.as_constant_path_write_node() {
        ("constant_path", write.value())
    } else if let Some(write) = node.as_local_variable_operator_write_node() {
        ("local_operator", write.value())
    } else if let Some(write) = node.as_instance_variable_operator_write_node() {
        ("instance_operator", write.value())
    } else if let Some(write) = node.as_class_variable_operator_write_node() {
        ("class_operator", write.value())
    } else if let Some(write) = node.as_global_variable_operator_write_node() {
        ("global_operator", write.value())
    } else if let Some(write) = node.as_constant_operator_write_node() {
        ("constant_operator", write.value())
    } else if let Some(write) = node.as_constant_path_operator_write_node() {
        ("constant_path_operator", write.value())
    } else if let Some(write) = node.as_local_variable_or_write_node() {
        ("local_or", write.value())
    } else if let Some(write) = node.as_local_variable_and_write_node() {
        ("local_and", write.value())
    } else if let Some(write) = node.as_instance_variable_or_write_node() {
        ("instance_or", write.value())
    } else if let Some(write) = node.as_instance_variable_and_write_node() {
        ("instance_and", write.value())
    } else if let Some(write) = node.as_class_variable_or_write_node() {
        ("class_or", write.value())
    } else if let Some(write) = node.as_class_variable_and_write_node() {
        ("class_and", write.value())
    } else if let Some(write) = node.as_global_variable_or_write_node() {
        ("global_or", write.value())
    } else if let Some(write) = node.as_global_variable_and_write_node() {
        ("global_and", write.value())
    } else if let Some(write) = node.as_constant_or_write_node() {
        ("constant_or", write.value())
    } else if let Some(write) = node.as_constant_and_write_node() {
        ("constant_and", write.value())
    } else if let Some(write) = node.as_constant_path_or_write_node() {
        ("constant_path_or", write.value())
    } else if let Some(write) = node.as_constant_path_and_write_node() {
        ("constant_path_and", write.value())
    } else if let Some(write) = node.as_multi_write_node() {
        ("multi", write.value())
    } else if let Some(write) = node.as_index_operator_write_node() {
        ("index_operator", write.value())
    } else if let Some(write) = node.as_index_or_write_node() {
        ("index_or", write.value())
    } else if let Some(write) = node.as_index_and_write_node() {
        ("index_and", write.value())
    } else if let Some(write) = node.as_call_operator_write_node() {
        ("call_operator", write.value())
    } else if let Some(write) = node.as_call_or_write_node() {
        ("call_or", write.value())
    } else if let Some(write) = node.as_call_and_write_node() {
        ("call_and", write.value())
    } else if let Some(call) = node.as_call_node() {
        let name = call.name().as_slice();
        if !(name.ends_with(b"=")
            || matches!(name, b"<<" | b"=~" | b"!~" | b"<=>" | b"<" | b">" | b"!="))
        {
            return None;
        }
        let value = call.arguments()?.arguments().last()?;
        (if name == b"[]=" { "index_call" } else { "call" }, value)
    } else {
        return None;
    };
    let start = node.location().start_offset();
    let value_start = value.location().start_offset();
    if value_start < start {
        return None;
    }
    Some(ConditionalAssignmentParts {
        lhs: source[start..value_start]
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" "),
        kind,
        value,
    })
}

impl Cop for RedundantTypeConversion {
    fn name(&self) -> &'static str {
        "Lint/RedundantTypeConversion"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let method = call.name().as_slice();
        if !matches!(
            method,
            b"to_s"
                | b"to_sym"
                | b"to_i"
                | b"to_f"
                | b"to_d"
                | b"to_r"
                | b"to_c"
                | b"to_a"
                | b"to_h"
                | b"to_set"
        ) {
            return;
        }
        if call
            .arguments()
            .is_some_and(|arguments| !arguments.arguments().is_empty())
        {
            return;
        }
        if matches!(method, b"to_h" | b"to_set") && call.block().is_some() {
            return;
        }
        let Some(receiver) = call.receiver().map(unwrap_redundant_conversion_parentheses) else {
            return;
        };
        if !redundant_conversion_receiver(method, &receiver, source) {
            return;
        }
        let Some(selector) = call.message_loc() else {
            return;
        };
        let selector = selector.start_offset()..selector.end_offset();
        let method = String::from_utf8_lossy(method);
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        let mut edit_start = selector.start;
        if source.as_bytes().get(edit_start.wrapping_sub(1)) == Some(&b'.') {
            edit_start -= 1;
            if source.as_bytes().get(edit_start.wrapping_sub(1)) == Some(&b'&') {
                edit_start -= 1;
            }
        }
        let mut edit_end = selector.end;
        if source[edit_end..].starts_with("()") {
            edit_end += 2;
        }
        cop_context.replace(
            format!("Redundant `{method}` detected."),
            selector.clone(),
            edit_start..edit_end,
            "",
        );
    }
}

fn unwrap_redundant_conversion_parentheses(mut node: Node<'_>) -> Node<'_> {
    loop {
        let Some(parentheses) = node.as_parentheses_node() else {
            return node;
        };
        let Some(inner) = parentheses.body().and_then(single_expression) else {
            return node;
        };
        node = inner;
    }
}

fn redundant_conversion_receiver(method: &[u8], receiver: &Node<'_>, source: &str) -> bool {
    let literal = match method {
        b"to_s" => {
            receiver.as_string_node().is_some() || receiver.as_interpolated_string_node().is_some()
        }
        b"to_sym" => {
            receiver.as_symbol_node().is_some() || receiver.as_interpolated_symbol_node().is_some()
        }
        b"to_i" => receiver.as_integer_node().is_some(),
        b"to_f" => receiver.as_float_node().is_some(),
        b"to_r" => receiver.as_rational_node().is_some(),
        b"to_c" => receiver.as_imaginary_node().is_some(),
        b"to_a" => receiver.as_array_node().is_some(),
        b"to_h" => receiver.as_hash_node().is_some(),
        _ => false,
    };
    if literal {
        return true;
    }
    let Some(receiver_call) = receiver.as_call_node() else {
        return false;
    };
    if receiver_call.name().as_slice() == method {
        return true;
    }
    if method == b"to_s" && matches!(receiver_call.name().as_slice(), b"inspect" | b"to_json") {
        return true;
    }
    if source_at(source, &receiver.location()).contains("exception: false") {
        return false;
    }
    redundant_conversion_constructor(method, &receiver_call)
}

fn redundant_conversion_constructor(method: &[u8], call: &CallNode<'_>) -> bool {
    let (class, kernel_method) = match method {
        b"to_s" => (b"String".as_slice(), b"String".as_slice()),
        b"to_i" => (b"Integer".as_slice(), b"Integer".as_slice()),
        b"to_f" => (b"Float".as_slice(), b"Float".as_slice()),
        b"to_d" => (b"BigDecimal".as_slice(), b"BigDecimal".as_slice()),
        b"to_r" => (b"Rational".as_slice(), b"Rational".as_slice()),
        b"to_c" => (b"Complex".as_slice(), b"Complex".as_slice()),
        b"to_a" => (b"Array".as_slice(), b"Array".as_slice()),
        b"to_h" => (b"Hash".as_slice(), b"Hash".as_slice()),
        b"to_set" => (b"Set".as_slice(), b"Set".as_slice()),
        _ => return false,
    };
    let name = call.name().as_slice();
    if name == kernel_method {
        return call.receiver().is_none() || root_constant(call.receiver(), b"Kernel");
    }
    let allowed_constructor = match method {
        b"to_s" => name == b"new",
        b"to_a" | b"to_h" | b"to_set" => matches!(name, b"new" | b"[]"),
        _ => false,
    };
    allowed_constructor && root_constant(call.receiver(), class)
}

enum RangeBlockParameter {
    Named(Vec<u8>),
    Numbered,
    It,
}

struct RangeSelection {
    pattern: String,
    negated: bool,
}

struct KindSelection<'pr> {
    class: Node<'pr>,
    negated: bool,
}

impl Cop for SelectByKind {
    fn name(&self) -> &'static str {
        "Style/SelectByKind"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let original = call.name().as_slice();
        if !matches!(original, b"select" | b"filter" | b"find_all" | b"reject") {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        if call.receiver().as_ref().is_some_and(range_hash_receiver) {
            return;
        }
        let Some(parameter) = range_block_parameter(&block) else {
            return;
        };
        let Some(body) = block.body().and_then(single_expression) else {
            return;
        };
        let Some(selection) = kind_selection(body, &parameter) else {
            return;
        };
        let selecting = matches!(original, b"select" | b"filter" | b"find_all");
        let replacement = if selecting == selection.negated {
            "grep_v"
        } else {
            "grep"
        };
        let Some(selector) = call.message_loc() else {
            return;
        };
        let original = String::from_utf8_lossy(original);
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(
            format!("Prefer `{replacement}` to `{original}` with a kind check."),
            call.location(),
            selector.start_offset()..block.location().end_offset(),
            format!(
                "{replacement}({})",
                source_at(source, &selection.class.location())
            ),
        );
    }
}

fn kind_selection<'pr>(
    mut body: Node<'pr>,
    parameter: &RangeBlockParameter,
) -> Option<KindSelection<'pr>> {
    let mut negated = false;
    if let Some(negation) = body.as_call_node() {
        if negation.name().as_slice() == b"!" {
            if negation
                .arguments()
                .is_some_and(|arguments| !arguments.arguments().is_empty())
            {
                return None;
            }
            body = negation.receiver()?;
            negated = true;
        }
    }
    let call = body.as_call_node()?;
    if !matches!(call.name().as_slice(), b"is_a?" | b"kind_of?") {
        return None;
    }
    let receiver = call.receiver()?;
    if !is_range_parameter(&receiver, parameter) {
        return None;
    }
    let arguments = call.arguments()?;
    let mut arguments = arguments.arguments().iter();
    let class = arguments.next()?;
    if arguments.next().is_some() {
        return None;
    }
    Some(KindSelection { class, negated })
}

impl Cop for SelectByRange {
    fn name(&self) -> &'static str {
        "Style/SelectByRange"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let original = call.name().as_slice();
        if !matches!(
            original,
            b"select" | b"filter" | b"find_all" | b"reject" | b"find" | b"detect"
        ) {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        if call.receiver().as_ref().is_some_and(range_hash_receiver) {
            return;
        }
        let Some(parameter) = range_block_parameter(&block) else {
            return;
        };
        let Some(body) = block.body().and_then(single_expression) else {
            return;
        };
        let Some(selection) = range_selection(body, &parameter, source) else {
            return;
        };
        let (grep, suffix, display) = if matches!(original, b"find" | b"detect") {
            if selection.negated {
                ("grep_v", ".first", "grep_v(...).first")
            } else {
                ("grep", ".first", "grep(...).first")
            }
        } else {
            let selecting = matches!(original, b"select" | b"filter" | b"find_all");
            let grep = if selecting == selection.negated {
                "grep_v"
            } else {
                "grep"
            };
            (grep, "", grep)
        };
        let Some(selector) = call.message_loc() else {
            return;
        };
        let original = String::from_utf8_lossy(original);
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace(
            format!("Prefer `{display}` to `{original}` with a range check."),
            call.location(),
            selector.start_offset()..block.location().end_offset(),
            format!("{grep}({}){suffix}", selection.pattern),
        );
    }
}

fn range_block_parameter(block: &ruby_prism::BlockNode<'_>) -> Option<RangeBlockParameter> {
    let parameters = block.parameters()?;
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return (numbered.maximum() == 1).then_some(RangeBlockParameter::Numbered);
    }
    if parameters.as_it_parameters_node().is_some() {
        return Some(RangeBlockParameter::It);
    }
    let block_parameters = parameters.as_block_parameters_node()?;
    let parameters = block_parameters.parameters()?;
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    let parameter = parameters
        .requireds()
        .first()?
        .as_required_parameter_node()?;
    Some(RangeBlockParameter::Named(
        parameter.name().as_slice().to_vec(),
    ))
}

fn range_selection(
    body: Node<'_>,
    parameter: &RangeBlockParameter,
    source: &str,
) -> Option<RangeSelection> {
    let (body, negated) = unwrap_range_negation(body)?;
    let call = body.as_call_node()?;
    match call.name().as_slice() {
        b"between?" => {
            let receiver = call.receiver()?;
            if !is_range_parameter(&receiver, parameter) {
                return None;
            }
            let arguments = call.arguments()?;
            let arguments = arguments.arguments().iter().collect::<Vec<_>>();
            if arguments.len() != 2 {
                return None;
            }
            Some(RangeSelection {
                pattern: format!(
                    "{}..{}",
                    source_at(source, &arguments[0].location()),
                    source_at(source, &arguments[1].location())
                ),
                negated,
            })
        }
        b"cover?" | b"include?" => {
            let receiver = unwrap_range_literal(call.receiver()?)?;
            let arguments = call.arguments()?;
            let mut arguments = arguments.arguments().iter();
            let argument = arguments.next()?;
            if arguments.next().is_some() || !is_range_parameter(&argument, parameter) {
                return None;
            }
            Some(RangeSelection {
                pattern: source_at(source, &receiver.location()).to_string(),
                negated,
            })
        }
        _ => None,
    }
}

fn unwrap_range_negation(mut node: Node<'_>) -> Option<(Node<'_>, bool)> {
    let mut negated = false;
    if let Some(call) = node.as_call_node() {
        if call.name().as_slice() == b"!" {
            if call
                .arguments()
                .is_some_and(|arguments| !arguments.arguments().is_empty())
            {
                return None;
            }
            node = call.receiver()?;
            negated = true;
        }
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        node = parentheses.body().and_then(single_expression)?;
    }
    Some((node, negated))
}

fn unwrap_range_literal(mut node: Node<'_>) -> Option<Node<'_>> {
    if let Some(parentheses) = node.as_parentheses_node() {
        node = parentheses.body().and_then(single_expression)?;
    }
    node.as_range_node().map(|range| range.as_node())
}

fn is_range_parameter(node: &Node<'_>, parameter: &RangeBlockParameter) -> bool {
    match parameter {
        RangeBlockParameter::Named(name) => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == name),
        RangeBlockParameter::Numbered => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == b"_1"),
        RangeBlockParameter::It => node.as_it_local_variable_read_node().is_some(),
    }
}

fn range_hash_receiver(node: &Node<'_>) -> bool {
    if node.as_hash_node().is_some() || node_is_root_constant(node, b"ENV") {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        matches!(call.name().as_slice(), b"to_h" | b"to_hash")
            || matches!(call.name().as_slice(), b"new" | b"[]")
                && call
                    .receiver()
                    .as_ref()
                    .is_some_and(|receiver| node_is_root_constant(receiver, b"Hash"))
    })
}

fn useless_access_modifier(context: &mut CopContext<'_, '_>) {
    let parsed = parse(context.source().as_bytes());
    let mut collector = AccessModifierScopeCollector {
        context_creating: context.config_values("ContextCreatingMethods").to_vec(),
        active_support: context.related_config_value("AllCops", "ActiveSupportExtensionsEnabled")
            == Some("true"),
        scopes: Vec::new(),
    };
    collector.visit(&parsed.node());
    for (statements, root) in collector.scopes {
        inspect_access_modifier_scope(&statements, root, context);
    }
}

struct AccessModifierScopeCollector<'pr> {
    context_creating: Vec<String>,
    active_support: bool,
    scopes: Vec<(ruby_prism::StatementsNode<'pr>, bool)>,
}

impl<'pr> Visit<'pr> for AccessModifierScopeCollector<'pr> {
    fn visit_program_node(&mut self, node: &ruby_prism::ProgramNode<'pr>) {
        self.scopes.push((node.statements(), true));
        ruby_prism::visit_program_node(self, node);
    }

    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
        if let Some(statements) = node.body().and_then(|body| body.as_statements_node()) {
            self.scopes.push((statements, false));
        }
        ruby_prism::visit_class_node(self, node);
    }

    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
        if let Some(statements) = node.body().and_then(|body| body.as_statements_node()) {
            self.scopes.push((statements, false));
        }
        ruby_prism::visit_module_node(self, node);
    }

    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode<'pr>) {
        if let Some(statements) = node.body().and_then(|body| body.as_statements_node()) {
            self.scopes.push((statements, false));
        }
        ruby_prism::visit_singleton_class_node(self, node);
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        let qualifies = node.as_node().location();
        let _ = qualifies;
        ruby_prism::visit_block_node(self, node);
    }

    fn visit_call_node(&mut self, node: &CallNode<'pr>) {
        let method = String::from_utf8_lossy(call_name(node));
        let qualifying = matches!(
            method.as_ref(),
            "class_eval" | "instance_eval" | "new" | "define"
        ) || self
            .context_creating
            .iter()
            .any(|configured| configured == method.as_ref())
            || self.active_support && method == "included";
        if qualifying {
            if let Some(block) = node.block().and_then(|block| block.as_block_node()) {
                if let Some(statements) = block.body().and_then(|body| body.as_statements_node()) {
                    self.scopes.push((statements, false));
                }
            }
        }
        ruby_prism::visit_call_node(self, node);
    }
}

fn inspect_access_modifier_scope(
    statements: &ruby_prism::StatementsNode<'_>,
    root: bool,
    context: &mut CopContext<'_, '_>,
) {
    let mut visibility = "public".to_string();
    let mut unused: Option<(String, std::ops::Range<usize>)> = None;
    inspect_access_modifier_statements(statements, root, &mut visibility, &mut unused, context);
    if let Some((method, location)) = unused {
        report_useless_modifier(&method, location, context);
    }
}

fn inspect_access_modifier_statements(
    statements: &ruby_prism::StatementsNode<'_>,
    root: bool,
    visibility: &mut String,
    unused: &mut Option<(String, std::ops::Range<usize>)>,
    context: &mut CopContext<'_, '_>,
) {
    for child in statements.body().iter() {
        if let Some(call) = child.as_call_node() {
            let method = String::from_utf8_lossy(call_name(&call)).into_owned();
            let bare = call.receiver().is_none()
                && argument_count(&call) == 0
                && matches!(method.as_str(), "private" | "protected" | "public");
            let private_class = call.receiver().is_none()
                && argument_count(&call) == 0
                && method == "private_class_method";
            if bare || private_class {
                let location = call.location().start_offset()..call.location().end_offset();
                if root || private_class {
                    report_useless_modifier(&method, location, context);
                    continue;
                }
                if method == *visibility {
                    report_useless_modifier(&method, location, context);
                } else {
                    if let Some((previous, location)) = unused.take() {
                        report_useless_modifier(&previous, location, context);
                    }
                    *visibility = method.clone();
                    *unused = Some((method, location));
                }
                continue;
            }
        }
        if let Some(begin) = child.as_begin_node() {
            if let Some(nested) = begin.statements() {
                inspect_access_modifier_statements(&nested, root, visibility, unused, context);
                continue;
            }
        }
        if let Some(call) = child.as_call_node() {
            if root {
                continue;
            }
            let method = String::from_utf8_lossy(call_name(&call));
            let active_included = method == "included"
                && context.related_config_value("AllCops", "ActiveSupportExtensionsEnabled")
                    == Some("true");
            let new_scope = matches!(
                method.as_ref(),
                "class_eval" | "instance_eval" | "new" | "define"
            ) || context
                .config_values("ContextCreatingMethods")
                .iter()
                .any(|configured| configured == method.as_ref())
                || active_included;
            if !new_scope {
                if let Some(statements) = call
                    .block()
                    .and_then(|block| block.as_block_node())
                    .and_then(|block| block.body())
                    .and_then(|body| body.as_statements_node())
                {
                    inspect_access_modifier_statements(
                        &statements,
                        root,
                        visibility,
                        unused,
                        context,
                    );
                }
            }
        }
        if access_node_defines_instance_method(&child, context) {
            *unused = None;
        }
    }
}

fn access_node_defines_instance_method(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    if let Some(definition) = node.as_def_node() {
        return definition.receiver().is_none();
    }
    if node.as_class_node().is_some()
        || node.as_module_node().is_some()
        || node.as_singleton_class_node().is_some()
    {
        return false;
    }
    if let Some(call) = node.as_call_node() {
        if call.receiver().is_none()
            && (matches!(
                call_name(&call),
                b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor" | b"define_method"
            ) || context
                .config_values("MethodCreatingMethods")
                .iter()
                .any(|method| method.as_bytes() == call_name(&call)))
        {
            return true;
        }
        if call.block().is_some()
            && matches!(
                call_name(&call),
                b"class_eval" | b"instance_eval" | b"new" | b"define"
            )
        {
            return false;
        }
        if call.arguments().is_some_and(|arguments| {
            arguments
                .arguments()
                .iter()
                .any(|argument| access_node_defines_instance_method(&argument, context))
        }) {
            return true;
        }
        if let Some(block) = call.block().and_then(|block| block.as_block_node()) {
            if let Some(body) = block.body() {
                return access_node_defines_instance_method(&body, context);
            }
        }
    }
    if let Some(statements) = node.as_statements_node() {
        return statements
            .body()
            .iter()
            .any(|child| access_node_defines_instance_method(&child, context));
    }
    if let Some(if_node) = node.as_if_node() {
        return if_node
            .statements()
            .is_some_and(|body| access_node_defines_instance_method(&body.as_node(), context))
            || if_node
                .subsequent()
                .is_some_and(|body| access_node_defines_instance_method(&body, context));
    }
    if let Some(unless_node) = node.as_unless_node() {
        return unless_node
            .statements()
            .is_some_and(|body| access_node_defines_instance_method(&body.as_node(), context))
            || unless_node
                .else_clause()
                .is_some_and(|body| access_node_defines_instance_method(&body.as_node(), context));
    }
    false
}

fn report_useless_modifier(
    method: &str,
    location: std::ops::Range<usize>,
    context: &mut CopContext<'_, '_>,
) {
    let line_start = context.source()[..location.start]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let line_end = context.source()[location.end..]
        .find('\n')
        .map_or(context.source().len(), |offset| location.end + offset + 1);
    context.remove(
        format!("Useless `{method}` access modifier."),
        location,
        line_start..line_end,
    );
}

struct AccessModifierDeclarations;

impl Cop for AccessModifierDeclarations {
    fn name(&self) -> &'static str {
        "Style/AccessModifierDeclarations"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        if call.receiver().is_some()
            || !matches!(
                call_name(&call),
                b"private" | b"protected" | b"public" | b"module_function"
            )
            || ancestors
                .iter()
                .any(|ancestor| ancestor.as_def_node().is_some())
        {
            return;
        }
        let mut context = context.cop_context(self.name(), source, ancestors);
        let style = context.policy().enforced_style("group");
        let surrounding_scope = ancestors.iter().any(|ancestor| {
            ancestor.as_class_node().is_some()
                || ancestor.as_module_node().is_some()
                || ancestor.as_singleton_class_node().is_some()
        });
        if !surrounding_scope
            && call.arguments().is_some_and(|arguments| {
                arguments
                    .arguments()
                    .iter()
                    .all(|argument| argument.as_symbol_node().is_some())
            })
        {
            return;
        }
        if style == "inline" {
            if argument_count(&call) != 0 || !right_sibling_definition(&call, &context) {
                return;
            }
            let Some(selector) = call.message_loc() else {
                return;
            };
            let modifier = context.source_file().at(&selector).to_string();
            let line_start = context.source_file().line_start(selector.start_offset());
            let line_end = context.source_file().line_end(selector.end_offset());
            let mut remove_end = source
                .get(line_end..)
                .and_then(|tail| tail.strip_prefix("\r\n").map(|_| line_end + 2))
                .or_else(|| {
                    source
                        .get(line_end..)
                        .and_then(|tail| tail.strip_prefix('\n').map(|_| line_end + 1))
                })
                .unwrap_or(line_end);
            let same_line_tail = &source[selector.end_offset()..line_end];
            let edit_start = if same_line_tail.trim_start().starts_with(';') {
                let semicolon =
                    selector.end_offset() + same_line_tail.find(';').unwrap_or_default();
                remove_end = semicolon + 1;
                while source
                    .as_bytes()
                    .get(remove_end)
                    .is_some_and(|byte| matches!(byte, b' ' | b'\t'))
                {
                    remove_end += 1;
                }
                selector.start_offset()
            } else {
                while source
                    .get(remove_end..)
                    .is_some_and(|tail| tail.starts_with('\n'))
                {
                    remove_end += 1;
                }
                while remove_end < source.len() {
                    let end = source[remove_end..]
                        .find('\n')
                        .map_or(source.len(), |at| remove_end + at);
                    if source[remove_end..end].trim().is_empty() {
                        remove_end = end.saturating_add(1);
                    } else {
                        break;
                    }
                }
                line_start
            };
            if line_start == 0 {
                remove_end += source[remove_end..].len() - source[remove_end..].trim_start().len();
            }
            let mut edits = vec![(edit_start..remove_end, String::new())];
            for at in following_access_definition_starts(&call, &context) {
                edits.push((at..at, format!("{modifier} ")));
            }
            context.replace_many(
                format!("`{modifier}` should be inlined in method definitions."),
                &selector,
                edits,
            );
            return;
        }
        if argument_count(&call) == 0 {
            return;
        }
        if ancestors
            .iter()
            .any(|ancestor| ancestor.as_if_node().is_some())
        {
            return;
        }
        if allowed_inline_modifier(&call, &context)
            || right_sibling_same_inline_modifier(&call, &context)
        {
            return;
        }
        let Some(selector) = call.message_loc() else {
            return;
        };
        let modifier = context.source_file().at(&selector).to_string();
        let message = format!("`{modifier}` should not be inlined in method definitions.");
        let argument = first_argument(&call).expect("modifier has arguments");
        if let Some(definition) = argument.as_def_node() {
            if let Some((edit, replacement)) = grouped_repeated_inline_rewrite(
                source,
                access_modifier_scope(&context),
                selector.start_offset(),
                definition.location().end_offset(),
                &modifier,
            ) {
                context.replace(message, &selector, edit, replacement);
                return;
            }
            let line_start = context.source_file().line_start(selector.start_offset());
            let comment_start = preceding_comment_block_start(source, line_start);
            let comments = source[comment_start..line_start]
                .lines()
                .map(str::trim_start)
                .collect::<Vec<_>>()
                .join("\n");
            let bare = other_bare_modifier_line(
                source,
                access_modifier_scope(&context),
                line_start,
                &modifier,
            );
            let following_conditional = following_conditional_modifier_line(
                source,
                definition.location().end_offset(),
                &modifier,
            );
            let replacement = if let Some((_, bare_line)) = &bare {
                if comments.is_empty() {
                    format!("\n{}\n\n", bare_line.trim_end())
                } else {
                    format!("\n{}\n\n{comments}\n", bare_line.trim_end())
                }
            } else if let Some((_, conditional)) = &following_conditional {
                format!("{}\n{modifier}\n\n", conditional.trim_end())
            } else if comments.is_empty() {
                format!("{modifier}\n\n")
            } else {
                format!("{modifier}\n\n{comments}\n")
            };
            let mut edits = vec![(
                comment_start..definition.location().start_offset(),
                replacement,
            )];
            if let Some((range, _)) = bare {
                edits.push((range, String::new()));
            }
            if let Some((range, _)) = following_conditional {
                edits.push((range, String::new()));
            }
            context.replace_many(message, &selector, edits);
        } else if group_modifier_correctable(&call, &context) {
            if argument.as_call_node().is_some() {
                let line_start = context.source_file().line_start(selector.start_offset());
                context.replace(
                    message,
                    &selector,
                    line_start..argument.location().start_offset(),
                    format!("{modifier}\n\n"),
                );
            } else {
                let names = call
                    .arguments()
                    .map(|arguments| {
                        arguments
                            .arguments()
                            .iter()
                            .filter_map(|argument| literal_method_name(&argument))
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default();
                if let Some(first_def) = first_named_definition_line(source, &names) {
                    let modifier_line = context.source_file().line_start(selector.start_offset());
                    let comment_start = preceding_comment_block_start(source, modifier_line);
                    let mut removal_start = comment_start;
                    if removal_start > 0 {
                        let previous_end = removal_start.saturating_sub(1);
                        let previous_start =
                            source[..previous_end].rfind('\n').map_or(0, |at| at + 1);
                        if source[previous_start..previous_end].trim().is_empty() {
                            removal_start = previous_start;
                        }
                    }
                    let modifier_end = context.source_file().line_end(selector.end_offset());
                    let remove_end = source
                        .get(modifier_end..)
                        .and_then(|tail| tail.strip_prefix('\n').map(|_| modifier_end + 1))
                        .unwrap_or(modifier_end);
                    let comments = source[comment_start..modifier_line]
                        .lines()
                        .map(str::trim_start)
                        .collect::<Vec<_>>()
                        .join("\n");
                    let other_bare = source
                        .split_inclusive('\n')
                        .scan(0usize, |offset, line| {
                            let start = *offset;
                            *offset += line.len();
                            Some((start, line))
                        })
                        .find(|(start, line)| *start != modifier_line && line.trim() == modifier)
                        .map(|(start, line)| {
                            let mut removal_start = start;
                            if start > 0 {
                                let previous_end = start - 1;
                                let previous_start =
                                    source[..previous_end].rfind('\n').map_or(0, |at| at + 1);
                                if source[previous_start..previous_end].trim().is_empty() {
                                    removal_start = previous_start;
                                }
                            }
                            (
                                removal_start..start + line.len(),
                                line.trim_end().to_string(),
                            )
                        });
                    let leading = if names.len() > 1 { "\n" } else { "" };
                    let insertion = if let Some((_, bare_line)) = &other_bare {
                        format!("\n\n{bare_line}\n\n")
                    } else if comments.is_empty() {
                        format!("{leading}{modifier}\n\n")
                    } else {
                        format!("{leading}{modifier}\n\n{comments}\n")
                    };
                    let mut edits = vec![
                        (removal_start..remove_end, String::new()),
                        (first_def..first_def, insertion),
                    ];
                    if let Some((bare_range, _)) = other_bare {
                        edits.push((bare_range, String::new()));
                    }
                    for line_start in named_definition_lines(source, &names) {
                        let token = source[line_start..]
                            .find("def ")
                            .map_or(line_start, |at| line_start + at);
                        if token > line_start {
                            edits.push((line_start..token, String::new()));
                        }
                    }
                    context.replace_many(message, &selector, edits);
                } else {
                    context.replace(message, &selector, &selector, modifier);
                }
            }
        } else {
            context.report(message, &selector);
        }
    }
}

fn grouped_repeated_inline_rewrite(
    source: &str,
    scope: std::ops::Range<usize>,
    selector_start: usize,
    definition_end: usize,
    modifier: &str,
) -> Option<(std::ops::Range<usize>, String)> {
    let current_start = source[..selector_start].rfind('\n').map_or(0, |at| at + 1);
    let previous_end = current_start.checked_sub(1)?;
    let previous_start = source[..previous_end].rfind('\n').map_or(0, |at| at + 1);
    let previous = source[previous_start..previous_end].trim_start();
    let inline_prefix = format!("{modifier} ");
    let previous_definition = previous.strip_prefix(&inline_prefix)?;
    if !previous_definition.starts_with("def ") {
        return None;
    }
    let current_end = source[definition_end..]
        .find('\n')
        .map_or(definition_end, |at| definition_end + at + 1);
    let current = source[current_start..current_end].trim_start();
    let current_definition = current.strip_prefix(&inline_prefix)?.trim_end();
    let scope_end_line = source[..scope.end.saturating_sub(1)]
        .rfind('\n')
        .map_or(scope.end, |at| at + 1);
    if current_end > scope_end_line {
        return None;
    }
    let public_tail = &source[current_end..scope_end_line];
    Some((
        previous_start..scope_end_line,
        format!(
            "{public_tail}{modifier}\n\n{}\n\n{current_definition}\n",
            previous_definition.trim_end()
        ),
    ))
}

fn other_bare_modifier_line(
    source: &str,
    scope: std::ops::Range<usize>,
    excluded_start: usize,
    modifier: &str,
) -> Option<(std::ops::Range<usize>, String)> {
    source[scope.clone()]
        .split_inclusive('\n')
        .scan(scope.start, |offset, line| {
            let start = *offset;
            *offset += line.len();
            Some((start, line))
        })
        .find(|(start, line)| *start != excluded_start && line.trim() == modifier)
        .map(|(start, line)| {
            let mut removal_start = start;
            if start > 0 {
                let previous_end = start - 1;
                let previous_start = source[..previous_end].rfind('\n').map_or(0, |at| at + 1);
                if source[previous_start..previous_end].trim().is_empty() {
                    removal_start = previous_start;
                }
            }
            (
                removal_start..start + line.len(),
                line.trim_end().to_string(),
            )
        })
}

fn access_modifier_scope(context: &CopContext<'_, '_>) -> std::ops::Range<usize> {
    context
        .ancestors()
        .iter()
        .rev()
        .find_map(|ancestor| {
            (ancestor.as_class_node().is_some()
                || ancestor.as_module_node().is_some()
                || ancestor.as_singleton_class_node().is_some())
            .then(|| ancestor.location().start_offset()..ancestor.location().end_offset())
        })
        .unwrap_or(0..context.source().len())
}

fn following_conditional_modifier_line(
    source: &str,
    after: usize,
    modifier: &str,
) -> Option<(std::ops::Range<usize>, String)> {
    let mut offset = source[after..]
        .find('\n')
        .map_or(source.len(), |at| after + at + 1);
    while offset < source.len() {
        let end = source[offset..]
            .find('\n')
            .map_or(source.len(), |at| offset + at + 1);
        let line = &source[offset..end];
        let trimmed = line.trim();
        if trimmed.starts_with(modifier) && trimmed.contains(" if ") {
            return Some((offset..end, line.trim_end().to_string()));
        }
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            return None;
        }
        offset = end;
    }
    None
}

fn preceding_comment_block_start(source: &str, mut line_start: usize) -> usize {
    while line_start > 0 {
        let previous_end = line_start.saturating_sub(1);
        let previous_start = source[..previous_end].rfind('\n').map_or(0, |at| at + 1);
        if source[previous_start..previous_end]
            .trim_start()
            .starts_with('#')
        {
            line_start = previous_start;
        } else {
            break;
        }
    }
    line_start
}

fn named_definition_lines(source: &str, names: &[String]) -> Vec<usize> {
    let mut offset = 0usize;
    source
        .split_inclusive('\n')
        .filter_map(|line| {
            let start = offset;
            offset += line.len();
            let definition = line.trim_start().strip_prefix("def ")?;
            names
                .iter()
                .any(|name| {
                    definition == name
                        || definition
                            .strip_prefix(name)
                            .is_some_and(|tail| tail.starts_with(['(', ';', ' ', '\n']))
                })
                .then_some(start)
        })
        .collect()
}

fn first_named_definition_line(source: &str, names: &[String]) -> Option<usize> {
    named_definition_lines(source, names).into_iter().min()
}

fn following_access_definition_starts(
    call: &CallNode<'_>,
    context: &CopContext<'_, '_>,
) -> Vec<usize> {
    let current = call.location().start_offset();
    let statements = context.ancestors().iter().rev().find_map(|ancestor| {
        if let Some(program) = ancestor.as_program_node() {
            Some(program.statements())
        } else if let Some(class) = ancestor.as_class_node() {
            class.body().and_then(|body| body.as_statements_node())
        } else if let Some(module) = ancestor.as_module_node() {
            module.body().and_then(|body| body.as_statements_node())
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            singleton.body().and_then(|body| body.as_statements_node())
        } else {
            None
        }
    });
    let Some(statements) = statements else {
        return Vec::new();
    };
    let mut after = false;
    let mut starts = Vec::new();
    for sibling in statements.body().iter() {
        if sibling.location().start_offset() == current {
            after = true;
            continue;
        }
        if !after {
            continue;
        }
        if let Some(next) = sibling.as_call_node() {
            if next.receiver().is_none()
                && argument_count(&next) == 0
                && matches!(
                    call_name(&next),
                    b"private" | b"protected" | b"public" | b"module_function"
                )
            {
                break;
            }
        }
        if let Some(definition) = sibling.as_def_node() {
            starts.push(definition.def_keyword_loc().start_offset());
        }
    }
    starts
}

fn group_modifier_correctable(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    if arguments
        .iter()
        .any(|argument| argument.as_def_node().is_some())
    {
        return true;
    }
    if arguments
        .first()
        .and_then(Node::as_call_node)
        .is_some_and(|call| {
            call.receiver().is_none()
                && matches!(
                    call_name(&call),
                    b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor" | b"alias_method"
                )
        })
    {
        return true;
    }
    let mut names = Vec::new();
    for argument in arguments {
        if let Some(name) = literal_method_name(&argument) {
            names.push(name);
        } else if let Some(array) = argument
            .as_splat_node()
            .and_then(|splat| splat.expression())
            .and_then(|expression| expression.as_array_node())
        {
            names.extend(
                array
                    .elements()
                    .iter()
                    .filter_map(|element| literal_method_name(&element)),
            );
        } else {
            return false;
        }
    }
    !names.is_empty()
        && names.iter().all(|name| {
            context.source().lines().any(|line| {
                let line = line.trim_start();
                line.strip_prefix("def ").is_some_and(|definition| {
                    let definition = definition.strip_prefix("self.").unwrap_or(definition);
                    definition == name
                        || definition
                            .strip_prefix(name)
                            .is_some_and(|tail| tail.starts_with(['(', ';', ' ', '\n']))
                })
            })
        })
}

fn right_sibling_definition(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let start = node.location().start_offset();
    context.ancestors().iter().rev().any(|ancestor| {
        let statements = if let Some(program) = ancestor.as_program_node() {
            Some(program.statements())
        } else if let Some(class) = ancestor.as_class_node() {
            class.body().and_then(|body| body.as_statements_node())
        } else if let Some(module) = ancestor.as_module_node() {
            module.body().and_then(|body| body.as_statements_node())
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            singleton.body().and_then(|body| body.as_statements_node())
        } else if let Some(begin) = ancestor.as_begin_node() {
            begin.statements()
        } else {
            None
        };
        statements.is_some_and(|statements| {
            let siblings = statements.body().iter().collect::<Vec<_>>();
            let Some(index) = siblings
                .iter()
                .position(|child| child.location().start_offset() == start)
            else {
                return false;
            };
            siblings[index + 1..]
                .iter()
                .take_while(|sibling| {
                    !sibling.as_call_node().is_some_and(|call| {
                        call.receiver().is_none()
                            && matches!(
                                call_name(&call),
                                b"private" | b"protected" | b"public" | b"module_function"
                            )
                            && argument_count(&call) == 0
                    })
                })
                .any(|sibling| sibling.as_def_node().is_some())
        })
    })
}

fn right_sibling_same_inline_modifier(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let start = node.location().start_offset();
    context.ancestors().iter().rev().any(|ancestor| {
        let statements = if let Some(program) = ancestor.as_program_node() {
            Some(program.statements())
        } else if let Some(class) = ancestor.as_class_node() {
            class.body().and_then(|body| body.as_statements_node())
        } else if let Some(module) = ancestor.as_module_node() {
            module.body().and_then(|body| body.as_statements_node())
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            singleton.body().and_then(|body| body.as_statements_node())
        } else if let Some(begin) = ancestor.as_begin_node() {
            begin.statements()
        } else {
            None
        };
        statements.is_some_and(|statements| {
            let direct_child = statements
                .body()
                .iter()
                .any(|child| child.location().start_offset() == start);
            direct_child
                && statements.body().iter().any(|sibling| {
                    let Some(call) = sibling.as_call_node() else {
                        return false;
                    };
                    call.location().start_offset() > start
                        && call.receiver().is_none()
                        && call_name(&call) == call_name(node)
                        && argument_count(&call) > 0
                        && !allowed_inline_modifier(&call, context)
                })
        })
    })
}

fn allowed_inline_modifier(node: &CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    if direct_block_parent(context.ancestors()) {
        return true;
    }
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    if context.config_bool("AllowModifiersOnSymbols", true)
        && arguments.iter().all(symbol_or_allowed_splat)
    {
        return true;
    }
    let Some(call) = arguments
        .first()
        .and_then(|argument| argument.as_call_node())
    else {
        return false;
    };
    call.receiver().is_none()
        && (context.config_bool("AllowModifiersOnAttrs", true)
            && matches!(
                call_name(&call),
                b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor"
            )
            || context.config_bool("AllowModifiersOnAliasMethod", true)
                && call_name(&call) == b"alias_method")
}

fn direct_block_parent(ancestors: &[Node<'_>]) -> bool {
    let Some(parent) = ancestors.last() else {
        return false;
    };
    if parent.as_block_node().is_some() {
        return true;
    }
    let Some(statements) = parent.as_statements_node() else {
        return false;
    };
    if statements.body().len() != 1 {
        return false;
    }
    ancestors[..ancestors.len() - 1]
        .iter()
        .rev()
        .find(|ancestor| ancestor.as_statements_node().is_none())
        .is_some_and(|ancestor| ancestor.as_block_node().is_some())
}

fn symbol_or_allowed_splat(argument: &Node<'_>) -> bool {
    if argument.as_symbol_node().is_some() {
        return true;
    }
    argument
        .as_splat_node()
        .and_then(|splat| splat.expression())
        .is_some_and(|expression| {
            expression.as_array_node().is_some()
                || expression.as_constant_read_node().is_some()
                || expression.as_constant_path_node().is_some()
                || expression.as_call_node().is_some()
        })
}

struct ArgumentsForwarding;

#[derive(Clone)]
struct ForwardingToken {
    prefix: &'static str,
    name: String,
    range: std::ops::Range<usize>,
    message: &'static str,
}

impl Cop for ArgumentsForwarding {
    fn name(&self) -> &'static str {
        "Style/ArgumentsForwarding"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(definition) = node.as_def_node() else {
            return;
        };
        let mut context = context.cop_context(self.name(), source, ancestors);
        check_arguments_forwarding(&definition, &mut context);
    }
}

fn check_arguments_forwarding(
    definition: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if !context.target_ruby_version().at_least(2, 7) || definition.body().is_none() {
        return;
    }
    let Some(parameters) = definition.parameters() else {
        return;
    };
    if report_anonymous_full_forwarding(definition, &parameters, context) {
        return;
    }
    let mut tokens = Vec::new();
    if let Some(rest) = parameters
        .rest()
        .and_then(|node| node.as_rest_parameter_node())
    {
        if let (Some(name), Some(name_loc)) = (rest.name(), rest.name_loc()) {
            tokens.push(ForwardingToken {
                prefix: "*",
                name: String::from_utf8_lossy(name.as_slice()).into_owned(),
                range: rest.operator_loc().start_offset()..name_loc.end_offset(),
                message: "Use anonymous positional arguments forwarding (`*`).",
            });
        }
    }
    if let Some(rest) = parameters
        .keyword_rest()
        .and_then(|node| node.as_keyword_rest_parameter_node())
    {
        if let (Some(name), Some(name_loc)) = (rest.name(), rest.name_loc()) {
            tokens.push(ForwardingToken {
                prefix: "**",
                name: String::from_utf8_lossy(name.as_slice()).into_owned(),
                range: rest.operator_loc().start_offset()..name_loc.end_offset(),
                message: "Use anonymous keyword arguments forwarding (`**`).",
            });
        }
    }
    if let Some(block) = parameters.block() {
        if let (Some(name), Some(name_loc)) = (block.name(), block.name_loc()) {
            tokens.push(ForwardingToken {
                prefix: "&",
                name: String::from_utf8_lossy(name.as_slice()).into_owned(),
                range: block.operator_loc().start_offset()..name_loc.end_offset(),
                message: "Use anonymous block arguments forwarding (`&`).",
            });
        }
    }
    if tokens.is_empty() {
        return;
    }

    let definition_end = definition.location().end_offset();
    let parameter_end = parameters.location().end_offset();
    let body = &context.source()[parameter_end..definition_end];
    let mut use_collector = ForwardingUseCollector::default();
    if let Some(definition_body) = definition.body() {
        use_collector.visit(&definition_body);
    }
    let occurrences = tokens
        .iter()
        .map(|token| {
            use_collector
                .uses
                .get(&format!("{}{}", token.prefix, token.name))
                .cloned()
                .unwrap_or_default()
        })
        .collect::<Vec<_>>();
    if occurrences.iter().all(Vec::is_empty) {
        return;
    }
    if context.target_ruby_version().at_least(3, 1)
        && !context.target_ruby_version().at_least(3, 4)
        && body.contains(" do")
        && occurrences.iter().any(|uses| !uses.is_empty())
    {
        return;
    }
    let referenced = tokens
        .iter()
        .map(|token| {
            use_collector
                .reads
                .get(&token.name)
                .is_some_and(|reads| {
                    reads
                        .iter()
                        .any(|offset| !use_collector.forwarded_reads.contains(offset))
                })
        })
        .collect::<Vec<_>>();

    let rest = tokens.iter().position(|token| token.prefix == "*");
    let kwrest = tokens.iter().position(|token| token.prefix == "**");
    let block = tokens.iter().position(|token| token.prefix == "&");
    let allow_only_rest = context.config_bool("AllowOnlyRestArgument", true);
    let has_additional_keywords = !parameters.keywords().is_empty();
    let has_additional_positionals = !parameters.requireds().is_empty()
        || !parameters.optionals().is_empty()
        || !parameters.posts().is_empty();
    let invalid_additional_parameters = has_additional_keywords
        || !context.target_ruby_version().at_least(3, 0) && has_additional_positionals
        || !context.target_ruby_version().at_least(3, 1) && !parameters.optionals().is_empty();
    let all_present = if context.target_ruby_version().at_least(3, 2) {
        rest.is_some() && kwrest.is_some() && (block.is_some() || !allow_only_rest)
    } else {
        (rest.is_some() || kwrest.is_some()) && (block.is_some() || !allow_only_rest)
    };
    let all_forwarded = all_present
        && !invalid_additional_parameters
        && tokens
            .iter()
            .all(|token| redundant_forwarding_name(token, context))
        && !referenced.iter().any(|referenced| *referenced)
        && tokens
            .iter()
            .enumerate()
            .all(|(index, _)| !occurrences[index].is_empty())
        && occurrences
            .iter()
            .map(Vec::len)
            .max()
            .is_some_and(|count| occurrences.iter().all(|uses| uses.len() == count));
    if all_forwarded {
        let signature_start = tokens.first().unwrap().range.start;
        let signature_end = tokens.last().unwrap().range.end;
        let signature = &context.source()[signature_start..signature_end];
        let normalized = tokens
            .iter()
            .map(|token| format!("{}{}", token.prefix, token.name))
            .collect::<Vec<_>>()
            .join(", ");
        if normalize_forwarding_sequence(signature) == normalized {
            let mut ranges = vec![signature_start..signature_end];
            let matches = body.match_indices(signature).collect::<Vec<_>>();
            if !context.target_ruby_version().at_least(3, 0)
                && matches
                    .iter()
                    .any(|(start, _)| forwarding_has_leading_call_argument(body, *start))
            {
                return;
            }
            for (start, _) in matches {
                ranges.push(parameter_end + start..parameter_end + start + signature.len());
            }
            if ranges.len() > 1 {
                for range in ranges {
                    let edits = forwarding_replacement_edits(context, range.clone(), "...");
                    context.replace_many(
                        "Use shorthand syntax `...` for arguments forwarding.",
                        range.clone(),
                        edits,
                    );
                }
                return;
            }
        }
    }

    if !context.target_ruby_version().at_least(3, 1) {
        return;
    }
    let use_anonymous = context.config_bool("UseAnonymousForwarding", true);
    if context.target_ruby_version().at_least(3, 2) && !use_anonymous {
        return;
    }
    for (index, token) in tokens.iter().enumerate() {
        if referenced[index] || occurrences[index].is_empty() {
            continue;
        }
        if token.prefix != "&" && (!context.target_ruby_version().at_least(3, 2) || !use_anonymous)
        {
            continue;
        }
        if !redundant_forwarding_name(token, context) {
            continue;
        }
        for range in std::iter::once(token.range.clone()).chain(occurrences[index].iter().cloned())
        {
            let edits = forwarding_replacement_edits(context, range.clone(), token.prefix);
            context.replace_many(token.message, range.clone(), edits);
        }
    }
}

fn report_anonymous_full_forwarding(
    definition: &ruby_prism::DefNode<'_>,
    parameters: &ruby_prism::ParametersNode<'_>,
    context: &mut CopContext<'_, '_>,
) -> bool {
    let range = parameters.location();
    let raw = context.source_file().at(&range);
    let Some(ampersand) = raw.rfind('&') else {
        return false;
    };
    if raw[ampersand + 1..]
        .trim_matches([' ', '\t', '\r', '\n', ')'])
        .is_empty()
        && raw[..ampersand].trim_end().ends_with(',')
    {
        let Some(star) = raw.find('*') else {
            return false;
        };
        let sequence = raw[star..=ampersand].trim();
        if context.target_ruby_version().at_least(3, 2) {
            if sequence.starts_with("**") || !sequence.contains("**") {
                return false;
            }
        }
        let signature_start = range.start_offset() + star;
        let signature_end = signature_start + sequence.len();
        let body_start = range.end_offset();
        let body_end = definition.location().end_offset();
        let body = &context.source()[body_start..body_end];
        if body.contains(" do") {
            return false;
        }
        let matches = body.match_indices(sequence).collect::<Vec<_>>();
        if matches.is_empty() {
            return false;
        }
        for offense in std::iter::once(signature_start..signature_end).chain(
            matches
                .into_iter()
                .map(|(at, _)| body_start + at..body_start + at + sequence.len()),
        ) {
            let edits = forwarding_replacement_edits(context, offense.clone(), "...");
            context.replace_many(
                "Use shorthand syntax `...` for arguments forwarding.",
                offense.clone(),
                edits,
            );
        }
        return true;
    }
    false
}

#[derive(Default)]
struct ForwardingUseCollector {
    uses: HashMap<String, Vec<std::ops::Range<usize>>>,
    reads: HashMap<String, Vec<usize>>,
    forwarded_reads: HashSet<usize>,
}

impl ForwardingUseCollector {
    fn record_reference(&mut self, name: &[u8], offset: usize) {
        let name = String::from_utf8_lossy(name).into_owned();
        self.reads.entry(name).or_default().push(offset);
    }

    fn collect_arguments(&mut self, arguments: Option<ruby_prism::ArgumentsNode<'_>>) {
        let Some(arguments) = arguments else { return };
        for argument in arguments.arguments().iter() {
            if let Some(splat) = argument.as_splat_node() {
                if let Some(read) = splat
                    .expression()
                    .and_then(|value| value.as_local_variable_read_node())
                {
                    self.record("*", read.name().as_slice(), splat.location(), read.location());
                }
            } else if let Some(hash) = argument.as_keyword_hash_node() {
                for element in hash.elements().iter() {
                    let Some(splat) = element.as_assoc_splat_node() else {
                        continue;
                    };
                    let Some(read) = splat
                        .value()
                        .and_then(|value| value.as_local_variable_read_node())
                    else {
                        continue;
                    };
                    self.record("**", read.name().as_slice(), splat.location(), read.location());
                }
            }
        }
    }

    fn collect_block(&mut self, block: Option<Node<'_>>) {
        let Some(argument) = block.and_then(|block| block.as_block_argument_node()) else {
            return;
        };
        let Some(read) = argument
            .expression()
            .and_then(|value| value.as_local_variable_read_node())
        else {
            return;
        };
        self.record("&", read.name().as_slice(), argument.location(), read.location());
    }

    fn record(
        &mut self,
        prefix: &str,
        name: &[u8],
        use_location: ruby_prism::Location<'_>,
        read_location: ruby_prism::Location<'_>,
    ) {
        let name = String::from_utf8_lossy(name).into_owned();
        self.uses.entry(format!("{prefix}{name}")).or_default().push(
            use_location.start_offset()..use_location.end_offset(),
        );
        self.forwarded_reads.insert(read_location.start_offset());
    }
}

impl<'pr> Visit<'pr> for ForwardingUseCollector {
    fn visit_splat_node(&mut self, node: &ruby_prism::SplatNode<'pr>) {
        if let Some(read) = node
            .expression()
            .and_then(|value| value.as_local_variable_read_node())
        {
            self.forwarded_reads.insert(read.location().start_offset());
        }
        ruby_prism::visit_splat_node(self, node);
    }

    fn visit_assoc_splat_node(&mut self, node: &ruby_prism::AssocSplatNode<'pr>) {
        if let Some(read) = node
            .value()
            .and_then(|value| value.as_local_variable_read_node())
        {
            self.forwarded_reads.insert(read.location().start_offset());
        }
        ruby_prism::visit_assoc_splat_node(self, node);
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        self.collect_arguments(node.arguments());
        self.collect_block(node.block());
        ruby_prism::visit_call_node(self, node);
    }

    fn visit_super_node(&mut self, node: &ruby_prism::SuperNode<'pr>) {
        self.collect_arguments(node.arguments());
        self.collect_block(node.block());
        ruby_prism::visit_super_node(self, node);
    }

    fn visit_yield_node(&mut self, node: &ruby_prism::YieldNode<'pr>) {
        self.collect_arguments(node.arguments());
        ruby_prism::visit_yield_node(self, node);
    }

    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        let name = String::from_utf8_lossy(node.name().as_slice()).into_owned();
        self.reads
            .entry(name)
            .or_default()
            .push(node.location().start_offset());
        ruby_prism::visit_local_variable_read_node(self, node);
    }

    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.record_reference(node.name().as_slice(), node.name_loc().start_offset());
        ruby_prism::visit_local_variable_write_node(self, node);
    }

    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        self.record_reference(node.name().as_slice(), node.location().start_offset());
        ruby_prism::visit_local_variable_target_node(self, node);
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        self.record_reference(node.name().as_slice(), node.name_loc().start_offset());
        ruby_prism::visit_local_variable_or_write_node(self, node);
    }

    fn visit_local_variable_and_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        self.record_reference(node.name().as_slice(), node.name_loc().start_offset());
        ruby_prism::visit_local_variable_and_write_node(self, node);
    }

    fn visit_local_variable_operator_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        self.record_reference(node.name().as_slice(), node.name_loc().start_offset());
        ruby_prism::visit_local_variable_operator_write_node(self, node);
    }
}

fn forwarding_replacement_edits(
    context: &CopContext<'_, '_>,
    range: std::ops::Range<usize>,
    replacement: &str,
) -> Vec<(std::ops::Range<usize>, String)> {
    let mut edits = vec![(range.clone(), replacement.to_string())];
    let source = context.source();
    if range.start == 0
        || !source.as_bytes()[range.start - 1].is_ascii_whitespace()
        || source.as_bytes()[range.start - 1] == b'\n'
    {
        return edits;
    }
    let line_start = context.source_file().line_start(range.start);
    let prefix = &source[line_start..range.start];
    let nesting = prefix.bytes().fold(0isize, |depth, byte| match byte {
        b'(' | b'[' => depth + 1,
        b')' | b']' => depth - 1,
        _ => depth,
    });
    if nesting > 0 {
        return edits;
    }
    if prefix.contains('*') || prefix.contains('&') {
        return edits;
    }
    let whitespace = prefix
        .bytes()
        .rev()
        .take_while(u8::is_ascii_whitespace)
        .count();
    let line_end = context.source_file().line_end(range.start);
    edits.push((range.start - whitespace..range.start, "(".to_string()));
    edits.push((line_end..line_end, ")".to_string()));
    edits
}

fn forwarding_has_leading_call_argument(body: &str, forwarding_start: usize) -> bool {
    let line_start = body[..forwarding_start].rfind('\n').map_or(0, |at| at + 1);
    let prefix = &body[line_start..forwarding_start];
    let Some(opening) = prefix.rfind(['(', '[']) else {
        return prefix.trim().contains(' ');
    };
    !prefix[opening + 1..].trim().is_empty()
}

fn normalize_forwarding_sequence(source: &str) -> String {
    source.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn redundant_forwarding_name(token: &ForwardingToken, context: &CopContext<'_, '_>) -> bool {
    let key = match token.prefix {
        "*" => "RedundantRestArgumentNames",
        "**" => "RedundantKeywordRestArgumentNames",
        "&" => "RedundantBlockArgumentNames",
        _ => return false,
    };
    context
        .config_values(key)
        .iter()
        .any(|name| name == &token.name)
}

struct DuplicateMethods;

#[derive(Default)]
struct DuplicateMethodsState {
    definitions: HashMap<String, SourceDefinition>,
    rescue_scopes: HashMap<&'static str, std::collections::HashSet<String>>,
}

struct SourceDefinition {
    path: String,
    line: usize,
}

impl Cop for DuplicateMethods {
    fn name(&self) -> &'static str {
        "Lint/DuplicateMethods"
    }

    fn investigation_state(&self) -> Box<dyn Any> {
        Box::new(DuplicateMethodsState::default())
    }

    fn on_new_investigation(&self, state: &mut dyn Any) {
        *state
            .downcast_mut::<DuplicateMethodsState>()
            .expect("duplicate methods state") = DuplicateMethodsState::default();
    }

    fn on_node_with_state<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
        state: &mut dyn Any,
    ) {
        if ancestors
            .iter()
            .any(|ancestor| ancestor.as_if_node().is_some() || ancestor.as_unless_node().is_some())
        {
            return;
        }
        let state = state
            .downcast_mut::<DuplicateMethodsState>()
            .expect("duplicate methods state");
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if let Some(definition) = node.as_def_node() {
            let name = String::from_utf8_lossy(definition.name().as_slice()).into_owned();
            let Some(method) = duplicate_method_name(&definition, ancestors, source, &name) else {
                return;
            };
            let key = method_key_with_scope_id(&method, ancestors, source);
            let offense =
                definition.def_keyword_loc().start_offset()..definition.name_loc().end_offset();
            register_method(state, key, method, offense, &mut cop_context);
        } else if let Some(alias) = node.as_alias_method_node() {
            let Some(name) = literal_method_name(&alias.new_name()) else {
                return;
            };
            if literal_method_name(&alias.old_name()).as_deref() == Some(name.as_str()) {
                return;
            }
            let Some(method) = duplicate_instance_method_name(ancestors, source, &name) else {
                return;
            };
            let key = method_key_with_scope_id(&method, ancestors, source);
            register_method(
                state,
                key,
                method,
                alias.location().start_offset()..alias.location().end_offset(),
                &mut cop_context,
            );
        } else if let Some(call) = node.as_call_node() {
            register_attribute_methods(&call, ancestors, state, &mut cop_context);
        }
    }
}

fn duplicate_method_name(
    definition: &ruby_prism::DefNode<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    name: &str,
) -> Option<String> {
    match definition.receiver() {
        None => duplicate_instance_method_name(ancestors, source, name),
        Some(receiver) if receiver.as_self_node().is_some() => {
            let scope = rubocop_parent_module_name(ancestors, source)
                .or_else(|| anonymous_class_scope(ancestors, source).map(|scope| scope.0))?;
            Some(format!("{scope}.{name}"))
        }
        Some(receiver)
            if receiver.as_constant_read_node().is_some()
                || receiver.as_constant_path_node().is_some() =>
        {
            let receiver = node_text(&receiver, source).trim_start_matches("::");
            let scope = rubocop_parent_module_name(ancestors, source)?;
            let qualified = if scope == "Object"
                || receiver.contains("::")
                || scope.rsplit("::").next() == Some(receiver)
            {
                receiver.to_string()
            } else {
                format!("{scope}::{receiver}")
            };
            Some(format!("{qualified}.{name}"))
        }
        Some(_) => None,
    }
}

fn duplicate_instance_method_name(
    ancestors: &[Node<'_>],
    source: &str,
    name: &str,
) -> Option<String> {
    if let Some(scope) = rubocop_parent_module_name(ancestors, source) {
        return Some(format!("{}{name}", humanized_method_scope(&scope)));
    }
    if let Some((scope, _scope_id)) = anonymous_class_scope(ancestors, source) {
        let singleton = ancestors
            .iter()
            .rev()
            .take_while(|ancestor| ancestor.as_block_node().is_none())
            .any(|ancestor| ancestor.as_singleton_class_node().is_some());
        let scope = if singleton {
            format!("#<Class:{scope}>")
        } else {
            scope
        };
        return Some(format!("{}{name}", humanized_method_scope(&scope)));
    }
    let singleton = ancestors
        .iter()
        .rev()
        .find_map(Node::as_singleton_class_node)?;
    let receiver = singleton.expression().as_call_node()?;
    Some(format!(
        "{}.{}",
        String::from_utf8_lossy(receiver.name().as_slice()),
        name
    ))
}

/// Mirrors rubocop-ast's `Node#parent_module_name`. In particular, an ordinary
/// block makes the lexical owner unknowable; treating its methods as members of
/// an enclosing class is the source of a large class of false duplicates.
fn rubocop_parent_module_name(ancestors: &[Node<'_>], source: &str) -> Option<String> {
    let mut parts = Vec::new();
    for (index, ancestor) in ancestors.iter().enumerate() {
        if let Some(class) = ancestor.as_class_node() {
            append_scope_part(&mut parts, node_text(&class.constant_path(), source));
        } else if let Some(module) = ancestor.as_module_node() {
            append_scope_part(&mut parts, node_text(&module.constant_path(), source));
        } else if let Some(singleton) = ancestor.as_singleton_class_node() {
            let expression = singleton.expression();
            let name = if expression.as_self_node().is_some() {
                format!("#<Class:{}>", joined_scope(&parts))
            } else if expression.as_constant_read_node().is_some()
                || expression.as_constant_path_node().is_some()
            {
                format!(
                    "#<Class:{}>",
                    node_text(&expression, source).trim_start_matches("::")
                )
            } else {
                return None;
            };
            parts.push(name);
        } else if let Some(write) = ancestor.as_constant_write_node() {
            if class_or_module_new_call(&write.value()) {
                append_scope_part(&mut parts, location_text(&write.name_loc(), source));
            }
        } else if let Some(write) = ancestor.as_constant_path_write_node() {
            if class_or_module_new_call(&write.value()) {
                append_scope_part(
                    &mut parts,
                    location_text(&write.target().location(), source),
                );
            }
        } else if ancestor.as_block_node().is_some() {
            let Some(call) = index
                .checked_sub(1)
                .and_then(|parent| ancestors[parent].as_call_node())
            else {
                return None;
            };
            if call_name(&call) == b"class_eval" {
                if let Some(receiver) = call.receiver() {
                    if receiver.as_constant_read_node().is_none()
                        && receiver.as_constant_path_node().is_none()
                    {
                        return None;
                    }
                    append_scope_part(&mut parts, node_text(&receiver, source));
                }
            } else if !class_or_module_new_call(&call.as_node())
                || !ancestors.get(index.wrapping_sub(2)).is_some_and(|parent| {
                    parent.as_constant_write_node().is_some()
                        || parent.as_constant_path_write_node().is_some()
                })
            {
                return None;
            }
        }
    }
    Some(if parts.is_empty() {
        "Object".to_string()
    } else {
        joined_scope(&parts)
    })
}

fn class_or_module_new_call(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"new"
            && (root_constant(call.receiver(), b"Class")
                || root_constant(call.receiver(), b"Module"))
    })
}

fn append_scope_part(parts: &mut Vec<String>, raw: &str) {
    let name = raw.trim_start_matches("::");
    if name.contains("::") {
        parts.clear();
    }
    parts.push(name.to_string());
}

fn joined_scope(parts: &[String]) -> String {
    parts.join("::")
}

fn humanized_method_scope(scope: &str) -> String {
    if let Some(start) = scope.find("#<Class:") {
        if let Some(name) = scope[start + 8..].strip_suffix('>') {
            return format!("{name}.");
        }
    }
    format!("{scope}#")
}

fn anonymous_class_scope(ancestors: &[Node<'_>], source: &str) -> Option<(String, Option<String>)> {
    let block_index = ancestors
        .iter()
        .rposition(|ancestor| ancestor.as_block_node().is_some())?;
    let call_index = block_index.checked_sub(1)?;
    let call = ancestors[call_index].as_call_node()?;
    if !class_or_module_new_call(&call.as_node())
        || ancestors
            .get(call_index.wrapping_sub(1))
            .is_some_and(|parent| parent.as_local_variable_write_node().is_some())
    {
        return None;
    }
    if ancestors[block_index + 1..].iter().any(|ancestor| {
        ancestor
            .as_singleton_class_node()
            .is_some_and(|singleton| singleton.expression().as_self_node().is_none())
    }) {
        return None;
    }
    let assigned_name = ancestors
        .get(call_index.wrapping_sub(1))
        .and_then(|parent| {
            if let Some(write) = parent.as_constant_write_node() {
                Some(location_text(&write.name_loc(), source).to_string())
            } else {
                parent.as_constant_path_write_node().map(|write| {
                    location_text(&write.target().location(), source).to_string()
                })
            }
        });
    if let Some(name) = assigned_name {
        let name = name.trim_start_matches("::").to_string();
        return Some((
            name,
            Some(format!("constant: {}", call.location().start_offset())),
        ));
    }

    let enclosing = rubocop_parent_module_name(&ancestors[..call_index], source);
    let base = match enclosing.as_deref() {
        Some("Object") => "Object".to_string(),
        Some(enclosing) => format!("{enclosing}::Object"),
        None => "::Object".to_string(),
    };
    let named_scope_id = ancestors[..call_index]
        .iter()
        .rev()
        .take_while(|ancestor| {
            ancestor.as_block_node().is_none() && ancestor.as_begin_node().is_none()
        })
        .find_map(Node::as_call_node)
        .and_then(|parent| {
            if let Some(receiver) = parent.receiver() {
                if class_or_module_new_call(&receiver) {
                    return Some(format!("outer-call: {}", parent.location().start_offset()));
                }
                Some(format!(
                    "{}.{}",
                    node_text(&receiver, source),
                    String::from_utf8_lossy(parent.name().as_slice())
                ))
            } else {
                Some(format!("outer-call: {}", parent.location().start_offset()))
            }
        });
    if named_scope_id.is_none()
        && ancestors[..call_index]
            .iter()
            .rev()
            .take_while(|ancestor| ancestor.as_block_node().is_none())
            .any(|ancestor| {
                ancestor.as_ensure_node().is_some()
                    || ancestor.as_rescue_node().is_some()
                    || ancestor
                        .as_begin_node()
                        .is_some_and(|begin| begin.ensure_clause().is_some())
            })
    {
        // Parser's anonymous-block identity deliberately disappears when the
        // expression itself is a statement below an `ensure`/`rescue` body.
        return Some((base, None));
    }
    let scope_id = named_scope_id.or_else(|| {
        ancestors[..call_index]
            .iter()
            .any(|ancestor| ancestor.as_block_node().is_some())
            .then(|| format!("anonymous: {}", call.location().start_offset()))
    });
    Some((base, scope_id))
}

fn node_text<'a>(node: &Node<'_>, source: &'a str) -> &'a str {
    let location = node.location();
    &source[location.start_offset()..location.end_offset()]
}

fn location_text<'a>(location: &ruby_prism::Location<'_>, source: &'a str) -> &'a str {
    &source[location.start_offset()..location.end_offset()]
}

fn method_key_with_scope_id(method: &str, ancestors: &[Node<'_>], source: &str) -> String {
    let mut key = nested_method_key(method, ancestors);
    if rubocop_parent_module_name(ancestors, source).is_none() {
        if let Some(scope_id) = anonymous_class_scope(ancestors, source).and_then(|scope| scope.1) {
            key.push('@');
            key.push_str(&scope_id);
        }
    }
    key
}

fn nested_method_key(method: &str, ancestors: &[Node<'_>]) -> String {
    ancestors
        .iter()
        .rev()
        .find_map(Node::as_def_node)
        .map_or_else(
            || method.to_string(),
            |definition| {
                format!(
                    "{}:{method}",
                    String::from_utf8_lossy(definition.name().as_slice())
                )
            },
        )
}

fn register_attribute_methods(
    call: &CallNode<'_>,
    ancestors: &[Node<'_>],
    state: &mut DuplicateMethodsState,
    context: &mut CopContext<'_, '_>,
) {
    if call.receiver().is_some() {
        return;
    }
    let call_method = call_name(call);
    let arguments = call
        .arguments()
        .into_iter()
        .flat_map(|arguments| arguments.arguments().iter())
        .collect::<Vec<_>>();
    let mut names = Vec::new();
    if matches!(
        call_method,
        b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor"
    ) {
        let readable = matches!(call_method, b"attr" | b"attr_reader" | b"attr_accessor");
        let writable = matches!(call_method, b"attr_writer" | b"attr_accessor");
        for argument in &arguments {
            let Some(name) = literal_method_name(argument) else {
                continue;
            };
            if readable {
                names.push(name.clone());
            }
            if writable {
                names.push(format!("{name}="));
            }
        }
        if call_method == b"attr"
            && arguments
                .get(1)
                .is_some_and(|argument| argument.as_true_node().is_some())
            && arguments
                .first()
                .and_then(|argument| literal_method_name(argument))
                .is_some()
        {
            names.push(format!(
                "{}=",
                literal_method_name(&arguments[0]).expect("checked literal attr")
            ));
        }
    } else if matches!(call_method, b"def_delegator" | b"def_instance_delegator") {
        if let Some(name) = arguments
            .get(if arguments.len() >= 3 { 2 } else { 1 })
            .and_then(|argument| literal_method_name(argument))
        {
            names.push(name);
        }
    } else if matches!(call_method, b"def_delegators" | b"def_instance_delegators") {
        names.extend(
            arguments
                .iter()
                .skip(1)
                .filter_map(|argument| literal_method_name(argument)),
        );
    } else if call_method == b"alias_method" {
        if let (Some(name), Some(original)) = (
            arguments
                .first()
                .and_then(|argument| literal_method_name(argument)),
            arguments
                .get(1)
                .and_then(|argument| literal_method_name(argument)),
        ) {
            if name != original {
                names.push(name);
            }
        }
    } else if call_method == b"delegate"
        && context.related_config_value("AllCops", "ActiveSupportExtensionsEnabled") == Some("true")
    {
        let methods = arguments
            .iter()
            .take_while(|argument| {
                argument.as_keyword_hash_node().is_none() && argument.as_hash_node().is_none()
            })
            .filter_map(|argument| literal_method_name(argument))
            .collect::<Vec<_>>();
        let pairs = arguments
            .last()
            .into_iter()
            .flat_map(|argument| {
                if let Some(hash) = argument.as_keyword_hash_node() {
                    hash.elements()
                        .iter()
                        .filter_map(|element| element.as_assoc_node())
                        .collect::<Vec<_>>()
                } else if let Some(hash) = argument.as_hash_node() {
                    hash.elements()
                        .iter()
                        .filter_map(|element| element.as_assoc_node())
                        .collect::<Vec<_>>()
                } else {
                    Vec::new()
                }
            })
            .collect::<Vec<_>>();
        let value_for = |key: &[u8]| {
            pairs.iter().find_map(|pair| {
                pair.key()
                    .as_symbol_node()
                    .filter(|symbol| symbol.unescaped() == key)
                    .map(|_| pair.value())
            })
        };
        let target = value_for(b"to").and_then(|value| literal_method_name(&value));
        let prefix = value_for(b"prefix").and_then(|value| {
            if value.as_true_node().is_some() {
                target.clone()
            } else {
                literal_method_name(&value)
            }
        });
        if target.is_none() {
            return;
        }
        names.extend(methods.into_iter().map(|method| {
            prefix
                .as_ref()
                .map_or(method.clone(), |prefix| format!("{prefix}_{method}"))
        }));
    } else {
        return;
    }
    for name in names {
        let Some(method) = duplicate_instance_method_name(ancestors, context.source(), &name)
        else {
            continue;
        };
        let key = method_key_with_scope_id(&method, ancestors, context.source());
        let location = call.location();
        register_method(
            state,
            key,
            method,
            location.start_offset()..location.end_offset(),
            context,
        );
    }
}

fn literal_method_name(node: &Node<'_>) -> Option<String> {
    if let Some(symbol) = node.as_symbol_node() {
        Some(String::from_utf8_lossy(symbol.unescaped()).into_owned())
    } else {
        node.as_string_node()
            .map(|string| String::from_utf8_lossy(string.unescaped()).into_owned())
    }
}

fn register_method(
    state: &mut DuplicateMethodsState,
    key: String,
    method: String,
    offense: std::ops::Range<usize>,
    context: &mut CopContext<'_, '_>,
) {
    let normalized = method.replace("self::", "::");
    let blocks = context
        .ancestors()
        .iter()
        .rev()
        .filter_map(Node::as_block_node)
        .collect::<Vec<_>>();
    let preserve_root_object = normalized.starts_with("::Object")
        && blocks
            .get(1)
            .is_some_and(|block| block.opening_loc().as_slice() == b"do");
    let method = if preserve_root_object {
        normalized
    } else {
        normalized
            .strip_prefix("::Object")
            .map_or(normalized.clone(), |suffix| format!("Object{suffix}"))
    };
    let line = context.source()[..offense.start]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1;
    let path = smart_source_path(context.path());
    if let Some(previous) = state.definitions.get(&key) {
        let rescue_scope = duplicate_rescue_scope(context.ancestors());
        if let Some(rescue_scope) = rescue_scope {
            if state
                .rescue_scopes
                .entry(rescue_scope)
                .or_default()
                .insert(key.clone())
            {
                state
                    .definitions
                    .insert(key, SourceDefinition { path, line });
                return;
            }
        }
        let message = format!(
            "Method `{method}` is defined at both {}:{} and {path}:{line}.",
            previous.path, previous.line
        );
        context.report(message, offense);
    } else {
        state
            .definitions
            .insert(key, SourceDefinition { path, line });
    }
}

fn duplicate_rescue_scope(ancestors: &[Node<'_>]) -> Option<&'static str> {
    ancestors.iter().rev().find_map(|ancestor| {
        if ancestor.as_rescue_node().is_some() {
            Some("rescue")
        } else if ancestor
            .as_begin_node()
            .is_some_and(|begin| begin.ensure_clause().is_some())
        {
            // Prism exposes `ensure` through its containing BeginNode rather
            // than retaining EnsureNode in the investigation ancestor stack.
            Some("ensure")
        } else {
            None
        }
    })
}

fn smart_source_path(path: &str) -> String {
    if let Some(relative) = path.strip_prefix("/path/to/project/root/") {
        return relative.to_string();
    }
    let path = std::path::Path::new(path);
    std::env::current_dir()
        .ok()
        .and_then(|current| {
            path.strip_prefix(current)
                .ok()
                .map(|path| path.to_path_buf())
        })
        .unwrap_or_else(|| path.to_path_buf())
        .to_string_lossy()
        .into_owned()
}

const SAFE_NAVIGATION_MESSAGE: &str =
    "Use safe navigation (`&.`) instead of checking if an object exists before calling the method.";

impl Cop for SafeNavigation {
    fn name(&self) -> &'static str {
        "Style/SafeNavigation"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if !cop_context.target_ruby_version().at_least(2, 3) {
            return;
        }
        if cop_context.related_config_value("AllCops", "DisabledByDefault") == Some("true")
            && safe_navigation_in_call_arguments(node, ancestors)
        {
            return;
        }
        if let Some(conditional) = node.as_if_node() {
            safe_navigation_if(&conditional, &mut cop_context);
        } else if let Some(conditional) = node.as_unless_node() {
            safe_navigation_unless(&conditional, &mut cop_context);
        } else if let Some(and_node) = node.as_and_node() {
            if !safe_navigation_nested_and(ancestors)
                && !safe_navigation_and_in_call_arguments(node, ancestors)
                && !safe_navigation_and_negated(ancestors)
                && !safe_navigation_and_unsafe_outer_call(node, ancestors)
            {
                safe_navigation_and(&and_node, &mut cop_context);
            }
        }
    }
}

fn safe_navigation_in_call_arguments(node: &Node<'_>, ancestors: &[Node<'_>]) -> bool {
    let location = node.location();
    ancestors.iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| {
            call.arguments().is_some_and(|arguments| {
                let arguments = arguments.location();
                arguments.start_offset() <= location.start_offset()
                    && location.end_offset() <= arguments.end_offset()
            })
        })
    })
}

fn safe_navigation_nested_and(ancestors: &[Node<'_>]) -> bool {
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_or_node().is_some() {
            return false;
        }
        if ancestor.as_and_node().is_some() {
            return true;
        }
    }
    false
}

fn safe_navigation_and_in_call_arguments(node: &Node<'_>, ancestors: &[Node<'_>]) -> bool {
    let location = node.location();
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_and_node().is_some()
            || ancestor.as_or_node().is_some()
            || ancestor.as_if_node().is_some()
            || ancestor.as_unless_node().is_some()
        {
            return false;
        }
        if let Some(call) = ancestor.as_call_node() {
            return call.arguments().is_some_and(|arguments| {
                let arguments = arguments.location();
                arguments.start_offset() <= location.start_offset()
                    && location.end_offset() <= arguments.end_offset()
            });
        }
    }
    false
}

fn safe_navigation_and_negated(ancestors: &[Node<'_>]) -> bool {
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_and_node().is_some()
            || ancestor.as_or_node().is_some()
            || ancestor.as_if_node().is_some()
            || ancestor.as_unless_node().is_some()
        {
            return false;
        }
        if ancestor
            .as_call_node()
            .is_some_and(|call| call_name(&call) == b"!")
        {
            return true;
        }
    }
    false
}

fn safe_navigation_and_unsafe_outer_call(node: &Node<'_>, ancestors: &[Node<'_>]) -> bool {
    let location = node.location();
    ancestors.iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|call| {
            call.receiver().is_some_and(|receiver| {
                let receiver = receiver.location();
                receiver.start_offset() <= location.start_offset()
                    && location.end_offset() <= receiver.end_offset()
                    && safe_navigation_nil_method(call_name(&call))
            })
        })
    })
}

fn safe_navigation_if(node: &ruby_prism::IfNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .if_keyword_loc()
        .as_ref()
        .is_some_and(|keyword| keyword.as_slice() == b"elsif")
    {
        return;
    }
    let ternary = node.if_keyword_loc().is_none()
        && node.then_keyword_loc().is_some()
        && node.end_keyword_loc().is_none();
    let then_branch = node.statements().and_then(|body| {
        (body.body().len() == 1)
            .then(|| body.body().first())
            .flatten()
    });
    let else_branch = node
        .subsequent()
        .and_then(|subsequent| subsequent.as_else_node())
        .and_then(|else_node| else_node.statements())
        .and_then(|body| {
            (body.body().len() == 1)
                .then(|| body.body().first())
                .flatten()
        });
    let (checked, body) = if ternary {
        let Some(then_branch) = then_branch else {
            return;
        };
        let Some(else_branch) = else_branch else {
            return;
        };
        if else_branch.as_nil_node().is_some() {
            if let Some(checked) = non_nil_checked_receiver(&node.predicate()) {
                (checked, then_branch)
            } else if simple_truthy_check(&node.predicate()) {
                (node.predicate(), then_branch)
            } else {
                return;
            }
        } else if then_branch.as_nil_node().is_some() {
            if let Some(checked) = nil_checked_receiver(&node.predicate()) {
                (checked, else_branch)
            } else if let Some(checked) = negated_receiver(&node.predicate()) {
                (checked, else_branch)
            } else {
                return;
            }
        } else {
            return;
        }
    } else {
        if node.subsequent().is_some() {
            return;
        }
        let Some(body) = then_branch else { return };
        if let Some(checked) = non_nil_checked_receiver(&node.predicate()) {
            (checked, body)
        } else if simple_truthy_check(&node.predicate()) {
            (node.predicate(), body)
        } else {
            return;
        }
    };
    safe_navigation_conditional(node.location(), &checked, &body, ternary, context);
}

fn safe_navigation_unless(node: &ruby_prism::UnlessNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.else_clause().is_some() {
        return;
    }
    let Some(body) = node.statements().and_then(|body| {
        (body.body().len() == 1)
            .then(|| body.body().first())
            .flatten()
    }) else {
        return;
    };
    let checked = if let Some(checked) = nil_checked_receiver(&node.predicate()) {
        checked
    } else if let Some(checked) = negated_receiver(&node.predicate()) {
        checked
    } else {
        return;
    };
    // `obj.do_something unless obj` uses the variable only as a negative
    // condition, rather than as the positive existence guard this cop targets.
    safe_navigation_conditional(node.location(), &checked, &body, false, context);
}

fn safe_navigation_conditional(
    offense: ruby_prism::Location<'_>,
    checked: &Node<'_>,
    body: &Node<'_>,
    ternary: bool,
    context: &mut CopContext<'_, '_>,
) {
    let strict_project_config = context
        .related_config_value("AllCops", "DisabledByDefault")
        == Some("true");
    if strict_project_config && safe_navigation_in_chained_block(context.ancestors()) {
        return;
    }
    let checked_source = context.source_file().node(checked).to_string();
    let Some(chain) = safe_navigation_chain(body, &checked_source, ternary, context) else {
        return;
    };
    let mut replacement = corrected_safe_navigation_chain(body, &checked_source, &chain, context);
    let before = &context.source()[offense.start_offset()..body.location().start_offset()];
    let after = &context.source()[body.location().end_offset()..offense.end_offset()];
    let comments = before
        .lines()
        .chain(after.lines())
        .filter_map(|line| line.find('#').map(|comment| line[comment..].trim()))
        .map(|line| format!("{line}\n"))
        .collect::<String>();
    if !comments.is_empty() {
        replacement = format!("{comments}{replacement}");
    }
    let offense = offense.start_offset()..offense.end_offset();
    context.replace(
        SAFE_NAVIGATION_MESSAGE,
        offense.clone(),
        offense,
        replacement,
    );
}

fn safe_navigation_and(node: &ruby_prism::AndNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut clauses = Vec::new();
    flatten_safe_navigation_and(node.as_node(), &mut clauses);
    struct Candidate {
        index: usize,
        offense: std::ops::Range<usize>,
        checked_source: String,
    }
    let mut candidates = Vec::new();
    for (index, pair) in clauses.windows(2).enumerate() {
        let lhs = &pair[0];
        let rhs = &pair[1];
        let (checked_source, non_nil) = if let Some(checked) = non_nil_checked_receiver(lhs) {
            (context.source_file().node(&checked).to_string(), true)
        } else if simple_truthy_check(lhs) {
            (context.source_file().node(lhs).to_string(), false)
        } else {
            continue;
        };
        if non_nil && !context.config_bool("ConvertCodeThatCanStartToReturnNil", false) {
            continue;
        }
        let Some(chain) = safe_navigation_chain(rhs, &checked_source, false, context) else {
            continue;
        };
        let _ = chain;
        let mut end = rhs.location().end_offset();
        let between = &context.source()[lhs.location().end_offset()..rhs.location().start_offset()];
        let opening_parentheses = between.bytes().filter(|byte| *byte == b'(').count();
        for _ in 0..opening_parentheses {
            if context.source().as_bytes().get(end) == Some(&b')') {
                end += 1;
            } else {
                break;
            }
        }
        candidates.push(Candidate {
            index,
            offense: lhs.location().start_offset()..end,
            checked_source,
        });
    }
    if candidates.is_empty() {
        safe_navigation_and_with_or(node, context);
        return;
    }
    let strict_project_block = context
        .related_config_value("AllCops", "DisabledByDefault")
        == Some("true")
        && context
            .ancestors()
            .iter()
            .any(|ancestor| ancestor.as_block_node().is_some());
    if strict_project_block
        && (candidates[0].index > 0
            || safe_navigation_in_chained_block(context.ancestors()))
    {
        return;
    }

    let mut groups = Vec::<(usize, usize)>::new();
    let mut group_start = 0;
    for index in 1..candidates.len() {
        if candidates[index].index != candidates[index - 1].index + 1 {
            groups.push((group_start, index - 1));
            group_start = index;
        }
    }
    groups.push((group_start, candidates.len() - 1));

    let node_start = node.location().start_offset();
    let node_end = node.location().end_offset();
    let mut edits = Vec::new();
    for (first, last) in groups {
        let candidate = &candidates[first];
        let lhs = &clauses[candidate.index];
        let final_rhs = &clauses[candidates[last].index + 1];
        let Some(chain) =
            safe_navigation_chain(final_rhs, &candidate.checked_source, false, context)
        else {
            continue;
        };
        let corrected =
            corrected_safe_navigation_chain(final_rhs, &candidate.checked_source, &chain, context);
        let between = &context.source()
            [lhs.location().end_offset()..clauses[candidate.index + 1].location().start_offset()];
        let preserved = between
            .chars()
            .filter(|character| *character == '(')
            .collect::<String>();
        edits.push((
            lhs.location().start_offset()..final_rhs.location().end_offset(),
            format!("{preserved}{corrected}"),
        ));
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut correction = String::new();
    let mut cursor = node_start;
    for (range, replacement) in edits {
        correction.push_str(&context.source()[cursor..range.start]);
        correction.push_str(&replacement);
        cursor = range.end;
    }
    correction.push_str(&context.source()[cursor..node_end]);

    context.replace(
        SAFE_NAVIGATION_MESSAGE,
        candidates[0].offense.clone(),
        node.location(),
        correction,
    );
    if !context.autocorrect_enabled() && !strict_project_block {
        for candidate in candidates.iter().skip(1) {
            context.report(SAFE_NAVIGATION_MESSAGE, candidate.offense.clone());
        }
    }
}

fn safe_navigation_in_chained_block(ancestors: &[Node<'_>]) -> bool {
    let mut saw_block = false;
    let mut calls_outside_block = 0;
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_block_node().is_some() {
            if saw_block {
                break;
            }
            saw_block = true;
        } else if saw_block && ancestor.as_call_node().is_some() {
            calls_outside_block += 1;
        }
    }
    calls_outside_block > 1
}

fn safe_navigation_and_with_or(node: &ruby_prism::AndNode<'_>, context: &mut CopContext<'_, '_>) {
    let lhs = node.left();
    if !simple_truthy_check(&lhs) {
        return;
    }
    let checked_source = context.source_file().node(&lhs).to_string();
    let right = unwrap_safe_navigation_parentheses(node.right());
    let Some(or_node) = right.as_or_node() else {
        return;
    };
    let Some(candidate) = first_safe_navigation_and_left(or_node.right()) else {
        return;
    };
    if safe_navigation_chain(&candidate, &checked_source, false, context).is_none() {
        return;
    }
    context.report(
        SAFE_NAVIGATION_MESSAGE,
        lhs.location().start_offset()..candidate.location().end_offset(),
    );
}

fn unwrap_safe_navigation_parentheses(mut node: Node<'_>) -> Node<'_> {
    loop {
        let Some(parentheses) = node.as_parentheses_node() else {
            return node;
        };
        let Some(inner) = parentheses.body().and_then(single_expression) else {
            return node;
        };
        node = inner;
    }
}

fn first_safe_navigation_and_left(node: Node<'_>) -> Option<Node<'_>> {
    let node = unwrap_safe_navigation_parentheses(node);
    if let Some(and_node) = node.as_and_node() {
        return Some(and_node.left());
    }
    let or_node = node.as_or_node()?;
    first_safe_navigation_and_left(or_node.left())
        .or_else(|| first_safe_navigation_and_left(or_node.right()))
}

fn flatten_safe_navigation_and<'pr>(node: Node<'pr>, clauses: &mut Vec<Node<'pr>>) {
    if let Some(parentheses) = node.as_parentheses_node() {
        if let Some(inner) = parentheses.body().and_then(single_expression) {
            flatten_safe_navigation_and(inner, clauses);
            return;
        }
    }
    if let Some(and_node) = node.as_and_node() {
        flatten_safe_navigation_and(and_node.left(), clauses);
        flatten_safe_navigation_and(and_node.right(), clauses);
    } else {
        clauses.push(node);
    }
}

fn safe_navigation_chain<'pr>(
    body: &Node<'pr>,
    checked_source: &str,
    ternary: bool,
    context: &CopContext<'_, '_>,
) -> Option<Vec<CallNode<'pr>>> {
    let mut calls = Vec::new();
    let mut call = body.as_call_node()?;
    loop {
        if call_name(&call) == b"!" {
            return None;
        }
        let receiver = call.receiver()?;
        calls.push(call);
        if safe_navigation_source_matches(
            source_at(context.source(), &receiver.location()),
            checked_source,
        ) {
            break;
        }
        call = receiver.as_call_node()?;
    }
    calls.reverse();
    if calls.len() > context.config_usize("MaxChainLength", 2) {
        return None;
    }
    let unsafe_chain_length = calls
        .iter()
        .filter(|call| {
            call.call_operator_loc()
                .is_some_and(|operator| operator.as_slice() == b".")
        })
        .count();
    if unsafe_chain_length > 1 {
        let disabled_by_default = context
            .related_config_value("AllCops", "DisabledByDefault")
            == Some("true");
        if disabled_by_default
            || context.related_config_value("Lint/SafeNavigationChain", "Enabled") != Some("true")
        {
            return None;
        }
    }
    let first = calls.first()?;
    let first_operator = first.call_operator_loc()?;
    if first_operator.as_slice() == b"::" || (!ternary && unsafe_safe_navigation_call(first)) {
        return None;
    }
    if body
        .as_call_node()
        .is_some_and(|call| call_name(&call) == b"empty?")
    {
        return None;
    }
    for call in calls.iter().skip(1) {
        if unsafe_safe_navigation_call(call)
            || safe_navigation_nil_method(call_name(call))
            || safe_navigation_allowed_method(call_name(call), context)
        {
            return None;
        }
    }
    Some(calls)
}

fn safe_navigation_allowed_method(name: &[u8], context: &CopContext<'_, '_>) -> bool {
    matches!(
        name,
        b"present?" | b"blank?" | b"presence" | b"try" | b"try!"
    ) || context.policy().allows_method(name)
}

fn corrected_safe_navigation_chain(
    body: &Node<'_>,
    checked_source: &str,
    calls: &[CallNode<'_>],
    context: &CopContext<'_, '_>,
) -> String {
    let body_start = body.location().start_offset();
    let body_end = body.location().end_offset();
    let mut edits = Vec::new();
    let matched = calls.first().and_then(CallNode::receiver);
    if let Some(matched) = matched {
        let matched_source = context.source_file().node(&matched);
        if checked_source != matched_source {
            edits.push((
                matched.location().start_offset()..matched.location().end_offset(),
                checked_source.to_string(),
            ));
        }
    }
    for call in calls {
        if let Some(operator) = call.call_operator_loc() {
            if operator.as_slice() == b"." {
                edits.push((
                    operator.start_offset()..operator.start_offset(),
                    "&".to_string(),
                ));
            }
        }
    }
    edits.sort_by_key(|(range, _)| range.start);
    let mut rendered = String::new();
    let mut cursor = body_start;
    for (range, replacement) in edits {
        if range.start < cursor || range.end > body_end {
            continue;
        }
        rendered.push_str(&context.source()[cursor..range.start]);
        rendered.push_str(&replacement);
        cursor = range.end;
    }
    rendered.push_str(&context.source()[cursor..body_end]);
    rendered
}

fn simple_truthy_check(node: &Node<'_>) -> bool {
    node.as_call_node()
        .is_none_or(|call| !matches!(call_name(&call), b"!" | b"nil?" | b"respond_to?"))
        && node.as_and_node().is_none()
        && node.as_or_node().is_none()
}

fn nil_checked_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let call = node.as_call_node()?;
    if call_name(&call) == b"nil?" && argument_count(&call) == 0 {
        call.receiver()
    } else {
        None
    }
}

fn non_nil_checked_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let receiver = negated_receiver(node)?;
    nil_checked_receiver(&receiver)
}

fn negated_receiver<'pr>(node: &Node<'pr>) -> Option<Node<'pr>> {
    let call = node.as_call_node()?;
    if call_name(&call) == b"!" && argument_count(&call) == 0 {
        call.receiver()
    } else {
        None
    }
}

fn safe_navigation_source_matches(left: &str, right: &str) -> bool {
    normalize_safe_navigation_source(left) == normalize_safe_navigation_source(right)
}

fn normalize_safe_navigation_source(source: &str) -> String {
    #[derive(Clone, Copy)]
    enum Literal {
        Quote(char),
        Percent {
            open: char,
            close: char,
            depth: usize,
        },
    }
    let characters = source.chars().collect::<Vec<_>>();
    let mut normalized = String::new();
    let mut literal = None;
    let mut escaped = false;
    let mut index = 0;
    while index < characters.len() {
        let character = characters[index];
        if let Some(state) = literal {
            normalized.push(character);
            if escaped {
                escaped = false;
                index += 1;
                continue;
            }
            if character == '\\' {
                escaped = true;
                index += 1;
                continue;
            }
            match state {
                Literal::Quote(close) if character == close => literal = None,
                Literal::Percent {
                    open,
                    close,
                    mut depth,
                } => {
                    if character == open && open != close {
                        depth += 1;
                        literal = Some(Literal::Percent { open, close, depth });
                    } else if character == close {
                        if depth == 0 {
                            literal = None;
                        } else {
                            literal = Some(Literal::Percent {
                                open,
                                close,
                                depth: depth - 1,
                            });
                        }
                    }
                }
                _ => {}
            }
            index += 1;
            continue;
        }
        if character.is_whitespace() {
            index += 1;
            continue;
        }
        if character == '&' && characters.get(index + 1) == Some(&'.') {
            normalized.push('.');
            index += 2;
            continue;
        }
        if matches!(character, '\'' | '"' | '`' | '/') {
            normalized.push(character);
            literal = Some(Literal::Quote(character));
            index += 1;
            continue;
        }
        if character == '%' {
            let delimiter_index = if characters.get(index + 1).is_some_and(|kind| {
                matches!(kind, 'q' | 'Q' | 'r' | 'w' | 'W' | 'i' | 'I' | 'x' | 's')
            }) {
                index + 2
            } else {
                index + 1
            };
            if let Some(&open) = characters.get(delimiter_index) {
                if !open.is_alphanumeric() && !open.is_whitespace() {
                    for value in &characters[index..=delimiter_index] {
                        normalized.push(*value);
                    }
                    let close = match open {
                        '(' => ')',
                        '[' => ']',
                        '{' => '}',
                        '<' => '>',
                        other => other,
                    };
                    literal = Some(Literal::Percent {
                        open,
                        close,
                        depth: 0,
                    });
                    index = delimiter_index + 1;
                    continue;
                }
            }
        }
        normalized.push(character);
        index += 1;
    }
    normalized
}

fn unsafe_safe_navigation_call(call: &CallNode<'_>) -> bool {
    let name = call_name(call);
    let assignment = name.ends_with(b"=")
        && !matches!(
            name,
            b"==" | b"!=" | b"<=" | b">=" | b"===" | b"=~" | b"!~" | b"<=>"
        );
    assignment
        || call.call_operator_loc().is_none()
        || call
            .call_operator_loc()
            .is_some_and(|operator| operator.as_slice() == b"::")
}

fn safe_navigation_nil_method(name: &[u8]) -> bool {
    matches!(
        name,
        b"!"
            | b"!="
            | b"!~"
            | b"&"
            | b"<=>"
            | b"=="
            | b"==="
            | b"=~"
            | b"^"
            | b"__id__"
            | b"__send__"
            | b"clone"
            | b"define_singleton_method"
            | b"display"
            | b"dup"
            | b"enum_for"
            | b"eql?"
            | b"equal?"
            | b"extend"
            | b"instance_eval"
            | b"instance_exec"
            | b"instance_of?"
            | b"instance_variable_defined?"
            | b"instance_variable_get"
            | b"instance_variable_set"
            | b"instance_variables"
            | b"is_a?"
            | b"kind_of?"
            | b"method"
            | b"methods"
            | b"nil?"
            | b"private_methods"
            | b"protected_methods"
            | b"public_method"
            | b"public_methods"
            | b"public_send"
            | b"rationalize"
            | b"remove_instance_variable"
            | b"respond_to?"
            | b"send"
            | b"singleton_class"
            | b"singleton_method"
            | b"singleton_methods"
            | b"tap"
            | b"then"
            | b"to_d"
            | b"to_enum"
            | b"yield_self"
            | b"|"
            | b"to_s"
            | b"to_i"
            | b"to_f"
            | b"to_a"
            | b"to_h"
            | b"to_c"
            | b"to_r"
            | b"inspect"
            | b"hash"
            | b"object_id"
            | b"class"
            | b"itself"
            | b"freeze"
            | b"frozen?"
    )
}
