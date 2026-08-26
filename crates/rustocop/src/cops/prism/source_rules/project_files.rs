use std::collections::HashMap;

use super::*;

pub(super) fn duplicated_gem(source: &str, context: &mut Reporter<'_>) {
    if std::path::Path::new(context.path())
        .file_name()
        .is_none_or(|name| name != "Gemfile")
    {
        return;
    }
    #[derive(Clone)]
    struct Declaration {
        offset: usize,
        line: usize,
        source_length: usize,
        indent: usize,
        conditional: Option<usize>,
    }

    #[derive(Clone, Copy)]
    enum Frame {
        Conditional(usize),
        Other,
    }

    let mut declarations = HashMap::<String, Vec<Declaration>>::new();
    let mut frames = Vec::new();
    let mut next_conditional = 0;
    for (line_index, (offset, line)) in source_lines(source).enumerate() {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            continue;
        }
        if trimmed.trim() == "end" {
            frames.pop();
            continue;
        }
        if trimmed.starts_with("if ")
            || trimmed.starts_with("unless ")
            || trimmed.starts_with("case ")
            || trimmed.trim() == "case"
        {
            next_conditional += 1;
            frames.push(Frame::Conditional(next_conditional));
            continue;
        }
        if !trimmed.starts_with("gem ") && !trimmed.starts_with("gem(") {
            if trimmed.contains(" do") || trimmed.ends_with("do") {
                frames.push(Frame::Other);
            }
            continue;
        }
        let Some(quote) = trimmed.find(['\'', '"']) else {
            continue;
        };
        let delimiter = trimmed.as_bytes()[quote] as char;
        let Some(end_quote) = trimmed[quote + 1..].find(delimiter) else {
            continue;
        };
        let name = &trimmed[quote + 1..quote + 1 + end_quote];
        let indent = line.len() - trimmed.len();
        let conditional = match frames.last() {
            Some(Frame::Conditional(id)) => Some(*id),
            _ => None,
        };
        declarations
            .entry(name.to_string())
            .or_default()
            .push(Declaration {
                offset,
                line: line_index + 1,
                source_length: line.len(),
                indent,
                conditional,
            });
    }

    for (name, matches) in declarations {
        if matches.len() < 2 {
            continue;
        }
        let conditional = matches[0].conditional;
        if conditional.is_some()
            && matches
                .iter()
                .all(|declaration| declaration.conditional == conditional)
        {
            continue;
        }
        let first_line = matches[0].line;
        for declaration in &matches[1..] {
            context.report(
                format!(
                    "Gem `{name}` requirements already given on line {first_line} of the Gemfile."
                ),
                declaration.offset + declaration.indent
                    ..declaration.offset + declaration.source_length,
            );
        }
    }
}
