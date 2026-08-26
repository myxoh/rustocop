// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/heredoc.rb
// Source SHA-256: 6b8b7effa6d0f54f77d37e9b3138345952673f5bc7dd918116eeb9f2a9edd8ac

use regex::Regex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StringNode {
    pub(crate) source: String,
    pub(crate) heredoc: bool,
}

pub(crate) trait Heredoc {
    fn on_heredoc(&mut self, node: &StringNode);

    fn on_str(&mut self, node: &StringNode) {
        if node.heredoc {
            self.on_heredoc(node);
        }
    }

    fn on_dstr(&mut self, node: &StringNode) {
        self.on_str(node);
    }

    fn on_xstr(&mut self, node: &StringNode) {
        self.on_str(node);
    }
}

pub(crate) fn indent_level(value: &str) -> usize {
    let indentations: Vec<&str> = value
        .split_inclusive('\n')
        .map(|line| &line[..line.len() - line.trim_start_matches(char::is_whitespace).len()])
        .filter(|indentation| !indentation.ends_with('\n'))
        .collect();
    indentations
        .iter()
        .map(|indentation| indentation.len())
        .min()
        .unwrap_or(0)
}

fn opening_delimiter(source: &str) -> Option<(String, String)> {
    let captures = Regex::new(r#"(<<[~-]?)[\'\"`]?([^\'\"`]+)[\'\"`]?"#)
        .expect("static regex")
        .captures(source)?;
    Some((captures[1].into(), captures[2].into()))
}

pub(crate) fn delimiter_string(node: &StringNode) -> String {
    opening_delimiter(&node.source).map_or_else(String::new, |capture| capture.1)
}

pub(crate) fn heredoc_type(node: &StringNode) -> String {
    opening_delimiter(&node.source).map_or_else(String::new, |capture| capture.0)
}
