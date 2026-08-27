use super::*;

define_compatibility_rule!(RedundantLineContinuationRule);

const MSG: &str = "Redundant line continuation.";

define_cops! {
    RedundantLineContinuation => "Style/RedundantLineContinuation" => compatibility_source(on_new_investigation),
}

fn on_new_investigation(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    RedundantLineContinuationRule::new(context).on_new_investigation();
}

impl RedundantLineContinuationRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let source = self.source().to_string();
        let ruby_end = source.find("\n__END__\n").map_or(source.len(), |at| at + 1);
        let ruby_source = &source[..ruby_end];
        let logical_end = ruby_source.trim_end_matches(['\n', '\r']).len();
        let literal_ranges = self.literal_ranges();
        let comment_ranges = self.comment_ranges();
        for (slash, _) in ruby_source.match_indices("\\\n") {
            if slash + 2 >= logical_end {
                continue;
            }
            let range = slash..slash + 2;
            if self.require_line_continuation(&source, slash, &literal_ranges, &comment_ranges)
                || !self.redundant_line_continuation(&source, slash)
            {
                continue;
            }
            add_offense!(self, range, message: MSG, |corrector| {
                corrector.remove(slash..slash + 1);
            });
        }
        self.inspect_end_of_ruby_code_line_continuation(ruby_source);
    }

    fn require_line_continuation(
        &self,
        source: &str,
        slash: usize,
        literal_ranges: &[std::ops::Range<usize>],
        comment_ranges: &[std::ops::Range<usize>],
    ) -> bool {
        let line_start = source[..slash].rfind('\n').map_or(0, |at| at + 1);
        let line = &source[line_start..slash];
        let next_start = slash + 2;
        let next_end = source[next_start..]
            .find('\n')
            .map_or(source.len(), |at| next_start + at);
        let next = &source[next_start..next_end];
        let before = line.trim_end();
        let inside_literal = literal_ranges
            .iter()
            .any(|literal| literal.start <= slash && slash < literal.end);
        let interpolation_boundary =
            inside_literal && interpolation_begins_next_line(source, slash) && line.contains('(');
        let has_comment = comment_ranges
            .iter()
            .any(|comment| line_start <= comment.start && comment.start < slash);
        if has_comment
            || string_concatenation(line)
            || inside_heredoc(source, slash)
            || (inside_literal && !interpolation_boundary)
        {
            return true;
        }
        if before.ends_with([',', '(', '[', '{', ':', '.'])
            || before.ends_with("&&")
            || before.ends_with("||")
        {
            return false;
        }
        if before.ends_with('=') && next.contains('{') && next.trim_end().ends_with(['+', '-']) {
            return true;
        }
        starts_with_arithmetic_operator(next)
            || starts_with_required_operator(next)
            || (!interpolation_boundary && method_with_argument(line, next))
            || leading_dot_method_chain_with_blank_line(line, next)
    }

    fn redundant_line_continuation(&self, source: &str, slash: usize) -> bool {
        let line_start = source[..slash].rfind('\n').map_or(0, |at| at + 1);
        let before = source[line_start..slash].trim_end();
        if before.ends_with([',', '(', '[', '{', ':', '.'])
            || before.ends_with("&&")
            || before.ends_with("||")
        {
            return true;
        }
        let mut candidate = source.to_string();
        candidate.remove(slash);
        let parsed = ruby_prism::parse(candidate.as_bytes());
        let valid = parsed.errors().next().is_none();
        valid
    }

    fn inspect_end_of_ruby_code_line_continuation(&mut self, source: &str) {
        let trimmed = source.trim_end_matches(['\n', '\r']);
        return_unless!(trimmed.ends_with('\\'));
        let slash = trimmed.len() - 1;
        let line_start = trimmed[..slash].rfind('\n').map_or(0, |at| at + 1);
        return_if!(line_has_comment(&trimmed[line_start..slash]));
        add_offense!(self, slash..slash + 1, message: MSG, |corrector| {
            corrector.remove(slash..slash + 1);
        });
    }
}

fn interpolation_begins_next_line(source: &str, slash: usize) -> bool {
    source[slash + 2..].starts_with("#{")
}

fn line_has_comment(line: &str) -> bool {
    let mut quote = None;
    let mut escaped = false;
    for character in line.chars() {
        if escaped {
            escaped = false;
            continue;
        }
        if character == '\\' {
            escaped = true;
        } else if quote == Some(character) {
            quote = None;
        } else if quote.is_none() && matches!(character, '\'' | '"') {
            quote = Some(character);
        } else if quote.is_none() && character == '#' {
            return true;
        }
    }
    false
}

fn string_concatenation(line: &str) -> bool {
    line.trim_end().ends_with(['\'', '"'])
}

fn starts_with_arithmetic_operator(line: &str) -> bool {
    matches!(
        line.trim_start().chars().next(),
        Some('+' | '-' | '*' | '/' | '%')
    )
}

fn starts_with_required_operator(line: &str) -> bool {
    let line = line.trim_start();
    [
        "&&", "||", "==", "===", "!=", ">=", "<=", "<=>", "=~", "!~", "&", "|", "^",
    ]
    .iter()
    .any(|operator| line.starts_with(operator))
}

fn method_with_argument(line: &str, next: &str) -> bool {
    let previous = line.trim_end();
    let next = next.trim_start();
    if next.is_empty()
        || matches!(next, "end" | "else" | "elsif" | "ensure" | "rescue")
        || next.starts_with(['&', ')', ']', '}', ','])
        || next.starts_with('.') && !next.starts_with("..")
    {
        return false;
    }
    if [
        "class ", "module ", "def ", "if ", "unless ", "while ", "until ",
    ]
    .iter()
    .any(|keyword| previous.starts_with(keyword))
        || previous.ends_with(" do")
        || ["&& ", "|| "]
            .iter()
            .any(|operator| previous.trim_start().starts_with(operator))
    {
        return false;
    }
    let last = previous
        .split(|character: char| character.is_whitespace() || "()[]{}.,;".contains(character))
        .rfind(|part| !part.is_empty())
        .unwrap_or_default();
    let flow = matches!(
        last,
        "break" | "next" | "return" | "super" | "yield" | "defined?"
    );
    let identifier = last
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_alphabetic() || character == '_')
        && last
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || "_?!".contains(character));
    (flow || identifier)
        && !previous.ends_with(['.', ',', '(', '[', '{', ':'])
        && !previous.ends_with("&&")
        && !previous.ends_with("||")
}

fn leading_dot_method_chain_with_blank_line(line: &str, next: &str) -> bool {
    matches!(line.trim_start().get(..1), Some(".")) && next.trim().is_empty()
        || line.trim_start().starts_with("&.") && next.trim().is_empty()
}

fn inside_heredoc(source: &str, offset: usize) -> bool {
    let mut markers = std::collections::VecDeque::<String>::new();
    let mut line_start = 0;
    for line in source.split_inclusive('\n') {
        let line_end = line_start + line.len();
        if let Some(marker) = markers.front() {
            if line.trim() == marker {
                markers.pop_front();
            } else if (line_start..line_end).contains(&offset) {
                return true;
            }
        } else {
            markers.extend(heredoc_markers(line));
        }
        if line_end > offset {
            return false;
        }
        line_start = line_end;
    }
    false
}

fn heredoc_markers(line: &str) -> Vec<String> {
    line.match_indices("<<")
        .filter_map(|(start, _)| {
            let mut tail = &line[start + 2..];
            tail = tail.strip_prefix(['-', '~']).unwrap_or(tail);
            let first = tail.as_bytes().first().copied()?;
            if matches!(first, b'\'' | b'"' | b'`') {
                let quote = char::from(first);
                let rest = &tail[1..];
                let end = rest.find(quote)?;
                return (!rest[..end].is_empty()).then(|| rest[..end].to_string());
            }
            if !(first.is_ascii_alphabetic() || first == b'_') {
                return None;
            }
            let end = tail
                .find(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                .unwrap_or(tail.len());
            Some(tail[..end].to_string())
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_line_continuations_in_heredoc_bodies() {
        let source = "value = <<~MESSAGE\n  first \\\n  second \\\nMESSAGE\n";
        for (offset, _) in source.match_indices("\\\n") {
            assert!(inside_heredoc(source, offset));
        }
    }

    #[test]
    fn recognizes_multiple_heredocs_before_the_target() {
        let source = "one = <<~MESSAGE\nfirst\nMESSAGE\ntwo = <<~MESSAGE\nsecond\nMESSAGE\nthree = <<~MESSAGE\n  target \\\n  tail\nMESSAGE\n";
        let offset = source.find("\\\n").unwrap();
        assert!(inside_heredoc(source, offset));
    }
}
