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
        for offset in semicolon_offsets(source) {
            context.replace(
                self.name(),
                "Do not use semicolons to terminate expressions.",
                (offset, offset + 1),
                (offset, offset + 1),
                "\n",
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

struct UnlessElse;

impl Cop for UnlessElse {
    fn name(&self) -> &'static str {
        "Style/UnlessElse"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(unless_node) = node.as_unless_node() else {
            return;
        };
        if unless_node.else_clause().is_none() {
            return;
        }
        let location = unless_node.location();
        context.replace(
            self.name(),
            "Do not use `unless` with `else`. Rewrite these with the positive case first.",
            &location,
            &location,
            correct_unless_else(source_at(source, &location)),
        );
    }
}

fn correct_unless_else(source: &str) -> String {
    let Some((before_else, after_else)) = source.split_once("\nelse\n") else {
        return source.replacen("unless", "if", 1);
    };
    let Some((header, body)) = before_else.split_once('\n') else {
        return source.replacen("unless", "if", 1);
    };
    let else_body = after_else.strip_suffix("\nend").unwrap_or(after_else);
    format!(
        "{}\n{}\nelse\n{}\nend",
        header.replacen("unless", "if", 1),
        else_body,
        body
    )
}
