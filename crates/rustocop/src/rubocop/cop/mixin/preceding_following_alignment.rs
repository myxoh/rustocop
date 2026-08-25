// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/preceding_following_alignment.rb
// Source SHA-256: ed9b43e4539ae5d9251f24620958994a5f4b9af6450bc72074fd94c780541d6c

use std::collections::BTreeSet;
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AlignmentRange {
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) source: String,
}

impl AlignmentRange {
    fn last_column(&self) -> usize {
        self.column + self.source.len()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AlignmentToken {
    pub(crate) kind: String,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) source: String,
    pub(crate) begin_pos: usize,
}

impl AlignmentToken {
    fn equal_sign(&self) -> bool {
        matches!(
            self.kind.as_str(),
            "tEQL" | "tEQ" | "tEQQ" | "tNEQ" | "tLEQ" | "tGEQ" | "tOP_ASGN"
        )
    }
    fn last_column(&self) -> usize {
        self.column + self.source.len()
    }
}

pub(crate) struct PrecedingFollowingAlignment<'a> {
    lines: &'a [&'a str],
    tokens: &'a [AlignmentToken],
    comment_lines: &'a [usize],
    allow_for_alignment: bool,
}

impl<'a> PrecedingFollowingAlignment<'a> {
    pub(crate) fn new(
        lines: &'a [&'a str],
        tokens: &'a [AlignmentToken],
        comment_lines: &'a [usize],
        allow_for_alignment: bool,
    ) -> Self {
        Self {
            lines,
            tokens,
            comment_lines,
            allow_for_alignment,
        }
    }

    fn allow_for_alignment(&self) -> bool {
        self.allow_for_alignment
    }

    fn aligned_with_something(&self, range: &AlignmentRange) -> bool {
        self.aligned_with_adjacent_line(range, false)
    }

    fn aligned_with_operator(&self, range: &AlignmentRange) -> bool {
        self.aligned_with_adjacent_line(range, true)
    }

    fn aligned_with_preceding_equals_operator(&self, token: &AlignmentToken) -> AlignmentResult {
        self.aligned_with_equals_sign(token, (1..=token.line).rev().collect())
    }

    fn aligned_with_subsequent_equals_operator(&self, token: &AlignmentToken) -> AlignmentResult {
        self.aligned_with_equals_sign(token, (token.line..=self.lines.len()).collect())
    }

    fn aligned_with_adjacent_line(&self, range: &AlignmentRange, operator_only: bool) -> bool {
        let preceding = (1..range.line).rev().collect::<Vec<_>>();
        let following = (range.line + 1..=self.lines.len()).collect::<Vec<_>>();
        self.aligned_with_any_line_range(&[preceding, following], range, operator_only)
    }

    fn aligned_with_any_line_range(
        &self,
        line_ranges: &[Vec<usize>],
        range: &AlignmentRange,
        operator_only: bool,
    ) -> bool {
        if self.aligned_with_any_line(line_ranges, range, None, operator_only) {
            return true;
        }
        let base_indent = self.line_indentation(range.line);
        self.aligned_with_any_line(line_ranges, range, Some(base_indent), operator_only)
    }

    fn aligned_with_any_line(
        &self,
        line_ranges: &[Vec<usize>],
        range: &AlignmentRange,
        indentation: Option<usize>,
        operator_only: bool,
    ) -> bool {
        line_ranges
            .iter()
            .any(|lines| self.aligned_with_line(lines, range, indentation, operator_only))
    }

    fn aligned_with_line(
        &self,
        line_numbers: &[usize],
        range: &AlignmentRange,
        indentation: Option<usize>,
        operator_only: bool,
    ) -> bool {
        for &line_number in line_numbers {
            if self.aligned_comment_lines().contains(&line_number) {
                continue;
            }
            let Some(line) = self.lines.get(line_number.saturating_sub(1)) else {
                continue;
            };
            let Some(index) = line.find(|character: char| !character.is_whitespace()) else {
                continue;
            };
            if indentation.is_some_and(|expected| expected != index) {
                continue;
            }
            let aligned = if operator_only {
                self.aligned_operator(range, line, line_number)
            } else {
                self.aligned_token(range, line, line_number)
            };
            if aligned {
                return true;
            }
        }
        false
    }

    fn aligned_comment_lines(&self) -> BTreeSet<usize> {
        self.comment_lines.iter().copied().collect()
    }

    fn aligned_token(&self, range: &AlignmentRange, line: &str, line_number: usize) -> bool {
        self.aligned_words(range, line) || self.aligned_equals_operator(range, line_number)
    }

    fn aligned_operator(&self, range: &AlignmentRange, line: &str, line_number: usize) -> bool {
        self.aligned_identical(range, line) || self.aligned_equals_operator(range, line_number)
    }

    fn aligned_words(&self, range: &AlignmentRange, line: &str) -> bool {
        if range.column > 0
            && line
                .get(range.column - 1..=range.column)
                .is_some_and(|pair| {
                    pair.starts_with(char::is_whitespace) && !pair.ends_with(char::is_whitespace)
                })
        {
            return true;
        }
        line.get(range.column..range.column + range.source.len()) == Some(range.source.as_str())
    }

    fn aligned_equals_operator(&self, range: &AlignmentRange, line_number: usize) -> bool {
        let token = self.tokens.iter().find(|token| {
            token.line == line_number
                && matches!(
                    token.kind.as_str(),
                    "tEQL" | "tEQ" | "tEQQ" | "tNEQ" | "tLEQ" | "tGEQ" | "tOP_ASGN" | "tLSHFT"
                )
        });
        token.is_some_and(|token| {
            self.aligned_with_preceding_equals(range, token)
                || self.aligned_with_append_operator(range, token)
        })
    }

    fn aligned_with_preceding_equals(
        &self,
        range: &AlignmentRange,
        token: &AlignmentToken,
    ) -> bool {
        range.source.ends_with('=') && range.last_column() == token.last_column()
    }

    fn aligned_with_append_operator(&self, range: &AlignmentRange, token: &AlignmentToken) -> bool {
        ((range.source == "<<" && token.equal_sign())
            || (range.source.ends_with('=') && token.kind == "tLSHFT"))
            && range.last_column() == token.last_column()
    }

    fn aligned_identical(&self, range: &AlignmentRange, line: &str) -> bool {
        line.get(range.column..range.column + range.source.len()) == Some(range.source.as_str())
    }

    fn aligned_with_equals_sign(
        &self,
        token: &AlignmentToken,
        line_range: Vec<usize>,
    ) -> AlignmentResult {
        let token_indent = self.line_indentation(token.line);
        let assignment_lines = self.relevant_assignment_lines(&line_range);
        let Some(&relevant_line) = assignment_lines.get(1) else {
            return AlignmentResult::None;
        };
        if self.line_indentation(relevant_line) < token_indent
            || self.lines.get(relevant_line.saturating_sub(1)).is_none()
        {
            return AlignmentResult::None;
        }
        let range = AlignmentRange {
            line: token.line,
            column: token.column,
            source: token.source.clone(),
        };
        if self.aligned_equals_operator(&range, relevant_line) {
            AlignmentResult::Yes
        } else {
            AlignmentResult::No
        }
    }

    fn assignment_lines(&self) -> Vec<usize> {
        self.assignment_tokens()
            .iter()
            .map(|token| token.line)
            .collect()
    }

    fn assignment_tokens(&self) -> Vec<AlignmentToken> {
        let mut seen = BTreeSet::new();
        self.tokens
            .iter()
            .filter(|token| token.equal_sign())
            .filter(|token| seen.insert(token.line))
            .cloned()
            .collect()
    }

    fn relevant_assignment_lines(&self, line_range: &[usize]) -> Vec<usize> {
        let Some(&first) = line_range.first() else {
            return Vec::new();
        };
        let original_indent = self.line_indentation(first);
        let assignments = self.assignment_lines();
        let mut result = Vec::new();
        let mut relevant_indent_at_level = true;
        for &line_number in line_range {
            let indent = self.line_indentation(line_number);
            let blank = self
                .lines
                .get(line_number.saturating_sub(1))
                .is_none_or(|line| line.trim().is_empty());
            if (indent < original_indent && !blank) || (relevant_indent_at_level && blank) {
                break;
            }
            if assignments.contains(&line_number) && indent == original_indent {
                result.push(line_number);
            }
            if !blank {
                relevant_indent_at_level = indent == original_indent;
            }
        }
        result
    }

    fn remove_equals_in_def(
        &self,
        assignment_tokens: &[AlignmentToken],
        ignored_positions: &[usize],
    ) -> Vec<AlignmentToken> {
        assignment_tokens
            .iter()
            .filter(|token| !ignored_positions.contains(&token.begin_pos))
            .cloned()
            .collect()
    }

    fn line_indentation(&self, line: usize) -> usize {
        self.lines.get(line.saturating_sub(1)).map_or(0, |source| {
            source
                .chars()
                .take_while(|character| character.is_whitespace())
                .count()
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AlignmentResult {
    None,
    Yes,
    No,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_word_operator_and_assignment_region_alignment() {
        let lines = ["left  = one", "right = two", "next  = three"];
        let tokens = [
            AlignmentToken {
                kind: "tEQL".into(),
                line: 1,
                column: 6,
                source: "=".into(),
                begin_pos: 6,
            },
            AlignmentToken {
                kind: "tEQL".into(),
                line: 2,
                column: 6,
                source: "=".into(),
                begin_pos: 18,
            },
            AlignmentToken {
                kind: "tEQL".into(),
                line: 3,
                column: 6,
                source: "=".into(),
                begin_pos: 30,
            },
        ];
        let helper = PrecedingFollowingAlignment::new(&lines, &tokens, &[], true);
        let range = AlignmentRange {
            line: 2,
            column: 6,
            source: "=".into(),
        };
        assert!(helper.allow_for_alignment());
        assert!(helper.aligned_with_operator(&range));
        assert!(helper.aligned_with_something(&range));
        assert_eq!(helper.assignment_lines(), vec![1, 2, 3]);
        assert_eq!(helper.remove_equals_in_def(&tokens, &[18]).len(), 2);
        assert_eq!(
            helper.relevant_assignment_lines(&(1..=3).collect::<Vec<_>>()),
            vec![1, 2, 3]
        );
    }
}
