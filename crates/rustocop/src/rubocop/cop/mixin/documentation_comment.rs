// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/documentation_comment.rb
// Source SHA-256: 1159160b3003b31dee55d4b9aa7e5e69084bb0baffca5a461d382496bf8e51fe

use regex::Regex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DocumentationLine {
    pub(crate) line: usize,
    pub(crate) source: String,
    pub(crate) text: String,
    pub(crate) comment: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DocumentationComment {
    annotation_keywords: Vec<String>,
}

impl DocumentationComment {
    pub(crate) fn new(annotation_keywords: Vec<String>) -> Self {
        Self {
            annotation_keywords,
        }
    }

    pub(crate) fn documentation_comment(
        &self,
        node: &DocumentationLine,
        associated: &[DocumentationLine],
    ) -> bool {
        let preceding = self.preceding_lines(node, associated);
        if !self.preceding_comment(Some(node), preceding.last().copied()) {
            return false;
        }
        preceding.into_iter().any(|comment| {
            !self.annotation(comment)
                && !self.interpreter_directive_comment(comment)
                && !self.rubocop_directive_comment(comment)
        })
    }

    pub(crate) fn preceding_comment(
        &self,
        node1: Option<&DocumentationLine>,
        node2: Option<&DocumentationLine>,
    ) -> bool {
        node1
            .zip(node2)
            .is_some_and(|(first, second)| self.precede(second, first) && second.comment)
    }

    pub(crate) fn precede(&self, node1: &DocumentationLine, node2: &DocumentationLine) -> bool {
        node2.line.saturating_sub(node1.line) == 1
    }

    pub(crate) fn preceding_lines<'lines>(
        &self,
        node: &DocumentationLine,
        associated: &'lines [DocumentationLine],
    ) -> Vec<&'lines DocumentationLine> {
        associated
            .iter()
            .filter(|line| line.line < node.line)
            .collect()
    }

    pub(crate) fn interpreter_directive_comment(&self, comment: &DocumentationLine) -> bool {
        Regex::new(r"^#\s*(frozen_string_literal|encoding):")
            .expect("static regex")
            .is_match(&comment.text)
    }

    pub(crate) fn rubocop_directive_comment(&self, comment: &DocumentationLine) -> bool {
        Regex::new(r"#\s*rubocop\s*:\s*(disable|enable|todo|push|pop)\b")
            .expect("static regex")
            .is_match(&comment.text)
    }

    pub(crate) fn annotation_keywords(&self) -> &[String] {
        &self.annotation_keywords
    }

    fn annotation(&self, comment: &DocumentationLine) -> bool {
        let body = comment.text.trim_start_matches('#').trim_start();
        self.annotation_keywords.iter().any(|keyword| {
            let Some(rest) = body
                .get(..keyword.len())
                .filter(|prefix| prefix.eq_ignore_ascii_case(keyword))
                .and_then(|_| body.get(keyword.len()..))
            else {
                return false;
            };
            let colon = rest.trim_start().starts_with(':');
            let separated = rest.starts_with(char::is_whitespace);
            let sentence_word = body.starts_with(&capitalize(keyword))
                && !colon
                && separated
                && !rest.trim().is_empty();
            (colon || separated) && !sentence_word
        })
    }
}

fn capitalize(value: &str) -> String {
    let mut chars = value.chars();
    chars.next().map_or_else(String::new, |first| {
        first.to_uppercase().collect::<String>() + chars.as_str()
    })
}
