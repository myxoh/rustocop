// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/line_length_help.rb
// Source SHA-256: c7a635a7f78bd438497667dd750a5b45bde021f2d3c1af0f6e2fd430182cd60f

use std::ops::Range;

use regex::Regex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct LineLengthConfig {
    pub(crate) max_line_length: usize,
    pub(crate) allow_rbs_inline_annotation: bool,
    pub(crate) ignore_cop_directives: Option<bool>,
    pub(crate) allow_cop_directives: bool,
    pub(crate) allow_uri: bool,
    pub(crate) allow_qualified_name: bool,
    pub(crate) tab_indentation_width: Option<usize>,
    pub(crate) configured_indentation_width: usize,
    pub(crate) uri_schemes: Vec<String>,
}

pub(crate) struct LineLengthHelp {
    config: LineLengthConfig,
}

impl LineLengthHelp {
    pub(crate) fn new(config: LineLengthConfig) -> Self {
        Self { config }
    }

    fn allow_rbs_inline_annotation(&self) -> bool {
        self.config.allow_rbs_inline_annotation
    }

    fn rbs_inline_annotation_on_source_line(
        &self,
        line_index: usize,
        first_line: usize,
        comments: &[(usize, &str)],
    ) -> bool {
        let source_line = line_index + first_line;
        comments
            .iter()
            .find(|(line, _)| *line == source_line)
            .is_some_and(|(_, text)| {
                text.starts_with("#:")
                    || Regex::new(r"^#\[.+\]").unwrap().is_match(text)
                    || text.starts_with("#|")
            })
    }

    fn allow_cop_directives(&self) -> bool {
        self.config
            .ignore_cop_directives
            .unwrap_or(self.config.allow_cop_directives)
    }

    fn directive_on_source_line(
        &self,
        line_index: usize,
        first_line: usize,
        comments: &[(usize, &str)],
    ) -> bool {
        let source_line = line_index + first_line;
        comments.iter().any(|(line, text)| {
            *line == source_line
                && Regex::new(r"#\s*rubocop\s*:(?:disable|enable|todo)\b")
                    .unwrap()
                    .is_match(text)
        })
    }

    fn allow_uri(&self) -> bool {
        self.config.allow_uri
    }

    fn allow_qualified_name(&self) -> bool {
        self.config.allow_qualified_name
    }

    fn allowed_position(&self, line: &str, range: &Range<usize>) -> bool {
        range.start < self.config.max_line_length && range.end == self.line_length(line)
    }

    fn line_length(&self, line: &str) -> usize {
        line.chars().count() + self.indentation_difference(line)
    }

    fn find_excessive_range(&self, line: &str, uri: bool) -> Option<Range<usize>> {
        let mut ranges = if uri {
            self.match_uris(line)
        } else {
            self.match_qualified_names(line)
        };
        let mut range = ranges.pop()?;
        range.end = self.extend_end_position(line, range.end);
        let difference = self.indentation_difference(line);
        range.start += difference;
        range.end += difference;
        (range.start >= self.config.max_line_length || range.end >= self.config.max_line_length)
            .then_some(range)
    }

    fn match_uris(&self, string: &str) -> Vec<Range<usize>> {
        self.uri_regexp()
            .find_iter(string)
            .filter(|matched| self.valid_uri(matched.as_str()))
            .map(|matched| matched.range())
            .collect()
    }

    fn match_qualified_names(&self, string: &str) -> Vec<Range<usize>> {
        self.qualified_name_regexp()
            .find_iter(string)
            .map(|matched| matched.range())
            .collect()
    }

    fn indentation_difference(&self, line: &str) -> usize {
        let Some(tab_width) = self.tab_indentation_width() else {
            return 0;
        };
        line.bytes().take_while(|byte| *byte == b'\t').count() * tab_width.saturating_sub(1)
    }

    fn extend_end_position(&self, line: &str, mut end_position: usize) -> usize {
        let tail = line.get(end_position..).unwrap_or_default();
        if line.ends_with('}') && line.contains('{') {
            if let Some(closing) = tail.find('}') {
                end_position += closing + 1;
            }
        }
        let tail = line.get(end_position..).unwrap_or_default();
        // Rust regex deliberately has no look-around; mirror /^\S+(?=\s|$)/ directly.
        end_position += tail.find(char::is_whitespace).unwrap_or(tail.len());
        end_position
    }

    fn tab_indentation_width(&self) -> Option<usize> {
        self.config
            .tab_indentation_width
            .or(Some(self.config.configured_indentation_width))
    }

    fn uri_regexp(&self) -> Regex {
        let schemes = if self.config.uri_schemes.is_empty() {
            vec!["http".to_owned(), "https".to_owned(), "ftp".to_owned()]
        } else {
            self.config.uri_schemes.clone()
        };
        Regex::new(&format!(r"(?:{})://[^\s<>]+", schemes.join("|"))).unwrap()
    }

    fn qualified_name_regexp(&self) -> Regex {
        Regex::new(r"\b(?:[A-Z][A-Za-z0-9_]*::)+[A-Za-z_][A-Za-z0-9_]*\b").unwrap()
    }

    fn valid_uri(&self, candidate: &str) -> bool {
        self.uri_regexp()
            .find(candidate)
            .is_some_and(|matched| matched.start() == 0 && matched.end() == candidate.len())
    }

    fn line_length_without_directive(&self, line: &str) -> usize {
        let before = Regex::new(r"#\s*rubocop\s*:(?:disable|enable|todo)\b")
            .unwrap()
            .find(line)
            .map_or(line, |matched| &line[..matched.start()])
            .trim_end();
        before.chars().count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn helper() -> LineLengthHelp {
        LineLengthHelp::new(LineLengthConfig {
            max_line_length: 20,
            allow_rbs_inline_annotation: true,
            ignore_cop_directives: None,
            allow_cop_directives: true,
            allow_uri: true,
            allow_qualified_name: true,
            tab_indentation_width: Some(4),
            configured_indentation_width: 2,
            uri_schemes: vec!["https".into()],
        })
    }

    #[test]
    fn ports_configuration_comments_ranges_and_tabs() {
        let helper = helper();
        assert!(helper.allow_rbs_inline_annotation());
        assert!(helper.allow_cop_directives());
        assert!(helper.allow_uri() && helper.allow_qualified_name());
        assert!(helper.rbs_inline_annotation_on_source_line(0, 1, &[(1, "#: String")]));
        assert!(helper.directive_on_source_line(0, 1, &[(1, "# rubocop:disable X")]));
        assert_eq!(helper.indentation_difference("\tfoo"), 3);
        assert_eq!(
            helper.line_length_without_directive("foo # rubocop:disable X"),
            3
        );
        assert_eq!(helper.match_qualified_names("x A::B y")[0], 2..6);
        assert!(helper.valid_uri("https://example.test/a"));
        assert!(helper
            .find_excessive_range("                    https://example.test/a", true)
            .is_some());
    }
}
