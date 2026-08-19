use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![Box::new(Semicolon), Box::new(UnlessElse)]
}

struct Semicolon;

impl Cop for Semicolon {
    fn name(&self) -> &'static str {
        "Style/Semicolon"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        if node.as_program_node().is_none() {
            return;
        }
        let allow_separators = {
            let cop_context = context.cop_context(self.name(), source, _ancestors);
            cop_context.config_bool("AllowAsExpressionSeparator", false)
        };
        for offset in semicolon_offsets(source) {
            let line_start = source[..offset].rfind('\n').map_or(0, |at| at + 1);
            let line_end = source[offset..]
                .find('\n')
                .map_or(source.len(), |at| offset + at);
            let prefix = &source[line_start..offset];
            if allow_separators && !source[offset + 1..line_end].trim().is_empty() {
                continue;
            }
            let line_source = &source[line_start..line_end];
            let header_terminated = line_source.trim_start().starts_with("def ")
                && line_source
                    .split_once(';')
                    .is_some_and(|(header, _)| header.split_whitespace().count() == 2);
            if header_terminated {
                continue;
            }
            if prefix.trim_start().starts_with("def ")
                && prefix.trim_end().ends_with(')')
                && prefix.matches('(').count() == 1
                && !prefix.contains(';')
            {
                continue;
            }
            let structural_header = {
                let header = prefix.trim();
                (header.starts_with("module ") || header.starts_with("class "))
                    && header.split_whitespace().count() == 2
            };
            if source[offset + 1..line_end].trim_start().starts_with("end")
                && (source[line_start..line_end].matches(';').count() == 1 || structural_header)
            {
                continue;
            }
            let replacement = if prefix.trim().is_empty()
                || prefix.trim_end().ends_with('{')
                || source[offset + 1..line_end].trim().is_empty()
                || source[offset + 1..line_end].trim_start().starts_with('}')
            {
                ""
            } else {
                "\n"
            };
            if matches!(prefix.trim_end(), value if value.ends_with("..") || value.ends_with("..."))
            {
                let indent = &source
                    [line_start..line_start + line_source.len() - line_source.trim_start().len()];
                let expression = prefix.trim();
                context.replace(
                    self.name(),
                    "Do not use semicolons to terminate expressions.",
                    (offset, offset + 1),
                    (line_start, offset + 1),
                    format!("{indent}({expression})"),
                );
                continue;
            }
            let trimmed_prefix = prefix.trim();
            if !trimmed_prefix.contains('(')
                && trimmed_prefix
                    .split_whitespace()
                    .skip(1)
                    .flat_map(|arguments| arguments.split(','))
                    .map(str::trim)
                    .filter(|argument| !argument.is_empty())
                    .all(|argument| argument.ends_with(':'))
                && trimmed_prefix.split_whitespace().count() > 1
            {
                let (method, arguments) = trimmed_prefix.split_once(' ').expect("count checked");
                let indent = &source
                    [line_start..line_start + line_source.len() - line_source.trim_start().len()];
                context.replace(
                    self.name(),
                    "Do not use semicolons to terminate expressions.",
                    (offset, offset + 1),
                    (line_start, offset + 1),
                    format!("{indent}{method}({arguments})"),
                );
                continue;
            }
            context.replace(
                self.name(),
                "Do not use semicolons to terminate expressions.",
                (offset, offset + 1),
                (offset, offset + 1),
                replacement,
            );
        }
    }
}

fn semicolon_offsets(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut offsets = Vec::new();
    let mut index = 0;
    let mut quote = None;
    let mut escaped = false;
    let mut comment = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if comment {
            if byte == b'\n' {
                comment = false;
            }
            index += 1;
            continue;
        }
        if let Some(delimiter) = quote {
            if delimiter == b'"' && byte == b'#' && bytes.get(index + 1) == Some(&b'{') {
                if let Some(end) = source[index + 2..].find('}') {
                    let expression_end = index + 2 + end;
                    offsets.extend(
                        source[index + 2..expression_end]
                            .match_indices(';')
                            .map(|(relative, _)| index + 2 + relative),
                    );
                    index = expression_end + 1;
                    continue;
                }
            }
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'#' => comment = true,
            b';' => offsets.push(index),
            _ => {}
        }
        index += 1;
    }
    offsets
}

define_node_cop!(UnlessElse => "Style/UnlessElse" => as_unless_node => unless_else);

fn unless_else(node: &ruby_prism::UnlessNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(else_clause) = node.else_clause() else {
        return;
    };
    let location = node.location();
    let message = "Do not use `unless` with `else`. Rewrite these with the positive case first.";
    if context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_unless_node().is_some())
    {
        if !context.autocorrect_enabled() {
            context.report(message, &location);
        }
        return;
    }
    let Some(end_keyword) = node.end_keyword_loc() else {
        context.report(message, &location);
        return;
    };
    let body_start = node.then_keyword_loc().map_or_else(
        || node.predicate().location().end_offset(),
        |keyword| keyword.end_offset(),
    );
    let body = body_start..else_clause.else_keyword_loc().start_offset();
    let alternative = else_clause.else_keyword_loc().end_offset()..end_keyword.start_offset();
    let mut correction = CorrectionPlan::default();
    correction.replace(
        node.keyword_loc().start_offset()..node.keyword_loc().end_offset(),
        "if",
    );
    if correction.swap(context.source(), body, alternative) {
        context.apply_correction(message, &location, correction);
    } else {
        context.report(message, &location);
    }
}
