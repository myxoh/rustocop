// RuboCop 1.87.0
// Source: lib/rubocop/cop/correctors/percent_literal_corrector.rb
// Source SHA-256: 3130997767a9c6092f5728b27bed958a545292ba01e9cb4dbf49dbc26d698eb6

use std::collections::BTreeMap;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::source::SourceRange;
use crate::rubocop::cop::corrector::Corrector;
use crate::rubocop::cop::framework::{escape_string, needs_escaping};
use crate::rubocop::cop::mixin::preferred_delimiters::PreferredDelimiters;

pub(crate) struct PercentLiteralCorrector {
    config: BTreeMap<String, String>,
    preferred_delimiters: Option<BTreeMap<String, String>>,
}

impl PercentLiteralCorrector {
    pub(crate) fn config(&self) -> &BTreeMap<String, String> {
        &self.config
    }

    pub(crate) fn preferred_delimiters(&self) -> Option<&BTreeMap<String, String>> {
        self.preferred_delimiters.as_ref()
    }

    pub(crate) fn initialize(
        config: BTreeMap<String, String>,
        preferred_delimiters: Option<BTreeMap<String, String>>,
    ) -> Self {
        Self {
            config,
            preferred_delimiters,
        }
    }

    pub(crate) fn correct<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
        node: NodeRef<'_>,
        character: char,
    ) {
        let escape = self.escape_words(node);
        let character = if escape {
            character.to_ascii_uppercase()
        } else {
            character
        };
        let Some(delimiters) = self.delimiters_for(&format!("%{character}")) else {
            return;
        };
        let contents = self.new_contents(node, escape, delimiters);
        self.wrap_contents(corrector, node, &contents, character, delimiters);
    }

    pub(crate) fn wrap_contents<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
        node: NodeRef<'_>,
        contents: &str,
        character: char,
        delimiters: (char, char),
    ) {
        let Some(range) = node.source_range() else {
            return;
        };
        corrector.replace(
            SourceRange::new(corrector.source_buffer(), range.start, range.end),
            format!("%{character}{}{contents}{}", delimiters.0, delimiters.1),
        );
    }

    pub(crate) fn escape_words(&self, node: NodeRef<'_>) -> bool {
        node.child_nodes().into_iter().any(|word| {
            word.string_child(0)
                .or_else(|| word.symbol_child(0))
                .is_some_and(needs_escaping)
        })
    }

    pub(crate) fn delimiters_for(&self, type_name: &str) -> Option<(char, char)> {
        let delimiters = PreferredDelimiters::initialize(
            type_name,
            self.config.clone(),
            self.preferred_delimiters.clone(),
        )
        .delimiters()
        .ok()?;
        (delimiters.len() == 2).then(|| (delimiters[0], delimiters[1]))
    }

    pub(crate) fn new_contents(
        &self,
        node: NodeRef<'_>,
        escape: bool,
        delimiters: (char, char),
    ) -> String {
        if node.multiline() {
            self.autocorrect_multiline_words(node, escape, delimiters)
        } else {
            self.autocorrect_words(node, escape, delimiters)
        }
    }

    pub(crate) fn autocorrect_multiline_words(
        &self,
        node: NodeRef<'_>,
        escape: bool,
        delimiters: (char, char),
    ) -> String {
        let mut contents = self.process_multiline_words(node, escape, delimiters);
        if let Some(end) = self.end_content(node.source().unwrap_or_default()) {
            contents.push_str(&end);
        }
        contents
    }

    pub(crate) fn autocorrect_words(
        &self,
        node: NodeRef<'_>,
        escape: bool,
        delimiters: (char, char),
    ) -> String {
        node.child_nodes()
            .into_iter()
            .map(|word| self.fix_escaped_content(word, escape, delimiters))
            .collect::<Vec<_>>()
            .join(" ")
    }

    pub(crate) fn process_multiline_words(
        &self,
        node: NodeRef<'_>,
        escape: bool,
        delimiters: (char, char),
    ) -> String {
        let base_line_num = node.first_line();
        let mut previous_line_num = base_line_num;
        let source = node.source().unwrap_or_default();
        let mut contents = String::new();
        for (index, word_node) in node.child_nodes().into_iter().enumerate() {
            contents.push_str(&self.line_breaks(
                word_node,
                source,
                previous_line_num,
                base_line_num,
                index,
            ));
            previous_line_num = word_node.last_line();
            contents.push_str(&self.fix_escaped_content(word_node, escape, delimiters));
        }
        contents
    }

    pub(crate) fn line_breaks(
        &self,
        node: NodeRef<'_>,
        source: &str,
        previous_line_num: usize,
        base_line_num: usize,
        node_index: usize,
    ) -> String {
        if self.first_line(node, previous_line_num) {
            if node_index == 0 && node.first_line() == base_line_num {
                String::new()
            } else {
                " ".into()
            }
        } else {
            self.process_lines(node, previous_line_num, base_line_num, source)
        }
    }

    pub(crate) fn first_line(&self, node: NodeRef<'_>, previous_line_num: usize) -> bool {
        node.first_line() == previous_line_num
    }

    pub(crate) fn process_lines(
        &self,
        node: NodeRef<'_>,
        previous_line_num: usize,
        base_line_num: usize,
        source: &str,
    ) -> String {
        let begin = previous_line_num.saturating_sub(base_line_num) + 1;
        let end = node.first_line().saturating_sub(base_line_num) + 1;
        let lines = source.split('\n').collect::<Vec<_>>();
        let between = lines
            .get(begin.min(lines.len())..end.min(lines.len()))
            .unwrap_or_default()
            .join("\n");
        let prefix = node
            .source()
            .and_then(|word| between.split(word).next())
            .unwrap_or_default();
        format!("\n{prefix}")
    }

    pub(crate) fn fix_escaped_content(
        &self,
        word_node: NodeRef<'_>,
        escape: bool,
        delimiters: (char, char),
    ) -> String {
        let Some(word) = word_node
            .string_child(0)
            .or_else(|| word_node.symbol_child(0))
        else {
            return String::new();
        };
        let content = if escape {
            escape_string(word)
        } else {
            word.to_owned()
        };
        self.substitute_escaped_delimiters(content, delimiters)
    }

    pub(crate) fn substitute_escaped_delimiters(
        &self,
        mut content: String,
        delimiters: (char, char),
    ) -> String {
        if delimiters.0 != delimiters.1
            && content.matches(delimiters.0).count() == content.matches(delimiters.1).count()
        {
            return content;
        }
        content = content.replace(delimiters.0, &format!("\\{}", delimiters.0));
        if delimiters.1 != delimiters.0 {
            content = content.replace(delimiters.1, &format!("\\{}", delimiters.1));
        }
        content
    }

    pub(crate) fn end_content(&self, source: &str) -> Option<String> {
        let last = source.split('\n').next_back()?;
        let whitespace = last
            .chars()
            .take_while(|character| character.is_whitespace())
            .collect::<String>();
        last[whitespace.len()..]
            .starts_with(']')
            .then(|| format!("\n{whitespace}"))
    }
}

#[cfg(test)]
mod spec;
