#![allow(clippy::too_many_arguments, clippy::too_many_lines)]
// RuboCop 1.87.0 compatibility for structural and formatting mixins.
// Source: lib/rubocop/cop/mixin/alignment.rb
// Source SHA-256: 4bd1ece511da0159dab1927bdd26d5b9f12e18b25a3627e503f8745bb3213ee6
// Source: lib/rubocop/cop/mixin/check_assignment.rb
// Source SHA-256: 95fa3d54fe986015fde7733d49651d9c0aaa4a147e1401b8825e1b0619db2523
// Source: lib/rubocop/cop/mixin/check_line_breakable.rb
// Source SHA-256: 81f0cffda303555084c6141bdac1e6bc5d02d32bdca6dd61b6f285d32cd8301d
// Source: lib/rubocop/cop/mixin/check_single_line_suitability.rb
// Source SHA-256: e545083edafa2d21eafe80a0a453c02fb065a3e3d668eb424c1eab553ce3e04b
// Source: lib/rubocop/cop/mixin/code_length.rb
// Source SHA-256: 2d44959d429ffbf9ddc5a6f991a5fcea0c727759838388e7975d9755401a3625
// Source: lib/rubocop/cop/mixin/comments_help.rb
// Source SHA-256: 65172714b7ffcf136480b48d5ec620cefe869c3e39498cd04100fb0e0cb6e2f6
// Source: lib/rubocop/cop/mixin/configurable_formatting.rb
// Source SHA-256: 6e14ea072f9fe28dc63c60b0b65d33e38e5125aacf91c5d1b70769936f1f2191
// Source: lib/rubocop/cop/mixin/documentation_comment.rb
// Source SHA-256: 1159160b3003b31dee55d4b9aa7e5e69084bb0baffca5a461d382496bf8e51fe
// Source: lib/rubocop/cop/mixin/empty_lines_around_body.rb
// Source SHA-256: 72cf05e697e8525fc19d5705f231bbbcd84963e1724666f533767eb9cf39baeb
// Source: lib/rubocop/cop/mixin/end_keyword_alignment.rb
// Source SHA-256: 6efdbf9c40254c69d2b1324a87e85024651f4d0564ee03b6287a1c3a1b825491
// Source: lib/rubocop/cop/mixin/endless_method_rewriter.rb
// Source SHA-256: 54fcac7c601a8c1610183b4f00632382dd3adcc33f72a778eca44474246e11ae
// Source: lib/rubocop/cop/mixin/enforce_superclass.rb
// Source SHA-256: cd4ea1f66b0bfb553cde210d2469c01f90d52b08965a910252e4f99b5963a8d0
// Source: lib/rubocop/cop/mixin/first_element_line_break.rb
// Source SHA-256: ee4b82e1a2a47d9bb1e51e10bcf4e9684627f1909fd60b3c91a86b87b37af7e4
// Source: lib/rubocop/cop/mixin/frozen_string_literal.rb
// Source SHA-256: fe4649eb36fb5a56d21b08dca166cd699d3493521b884fb60d470dd334dcdfdc
// Source: lib/rubocop/cop/mixin/gemspec_help.rb
// Source SHA-256: c7f031eb9e1b567251ae339bb37aad793c2c5edd5eda3f7cec587a26ee5bacce
// Source: lib/rubocop/cop/mixin/hash_alignment_styles.rb
// Source SHA-256: 442c96335947a457499dd3c494915eba604953083703fde656d249fb318ab41a
// Source: lib/rubocop/cop/mixin/hash_shorthand_syntax.rb
// Source SHA-256: 590dc7c208d1a2f760315021b4c5a04e8805517fc29218eef2592a01dac7393a
// Source: lib/rubocop/cop/mixin/hash_subset.rb
// Source SHA-256: 35bbdc65c4bad6fda7808b3378d8e4b69ac654c50884f8d638263e24d04cd91e
// Source: lib/rubocop/cop/mixin/hash_transform_method.rb
// Source SHA-256: 1b4984200f14e13817355f80d8d9b5cc5bdfd06358a800a76c65e581c296609a
// Source: lib/rubocop/cop/mixin/hash_transform_method/autocorrection.rb
// Source SHA-256: 043ec952bfceb375fa1fee240b92c7e9b6c69ba331fb5819db25d1a8af6fc450
// Source: lib/rubocop/cop/mixin/heredoc.rb
// Source SHA-256: 6b8b7effa6d0f54f77d37e9b3138345952673f5bc7dd918116eeb9f2a9edd8ac
// Source: lib/rubocop/cop/mixin/interpolation.rb
// Source SHA-256: 71e820e0b68830a67a449d291b417c1bd8d98f7748596c846b758af67ac53fe2
// Source: lib/rubocop/cop/mixin/line_length_help.rb
// Source SHA-256: c7a635a7f78bd438497667dd750a5b45bde021f2d3c1af0f6e2fd430182cd60f
// Source: lib/rubocop/cop/mixin/method_complexity.rb
// Source SHA-256: 6d30cc815605b57f14bc11e488c5c8b9dc871d3a81b363169a72ad2131c8376a
// Source: lib/rubocop/cop/mixin/multiline_element_indentation.rb
// Source SHA-256: 295abb00d6ca7c0595630ed224e06173b6db95687169156d26d2c4bd20523aa4
// Source: lib/rubocop/cop/mixin/multiline_expression_indentation.rb
// Source SHA-256: e94106b7e5d3522b3fec5554ec0ddf07ccbd96e10c5ef1b8dfb82248d861efae
// Source: lib/rubocop/cop/mixin/multiline_literal_brace_layout.rb
// Source SHA-256: 088de502fd25c62152c5466af620c0b433affe56ca1c69680d8bc8560df511e7
// Source: lib/rubocop/cop/mixin/ordered_gem_node.rb
// Source SHA-256: e36806081d47462d7e6ffe159c745568c0ba60abab9a70fcb4d69555c7583f68
// Source: lib/rubocop/cop/mixin/percent_array.rb
// Source SHA-256: ce85c24d9a1b1c26805b379231de174ca2d6016fccff9c88300c664946d7aa42
// Source: lib/rubocop/cop/mixin/preceding_following_alignment.rb
// Source SHA-256: ed9b43e4539ae5d9251f24620958994a5f4b9af6450bc72074fd94c780541d6c
// Source: lib/rubocop/cop/mixin/project_index_help.rb
// Source SHA-256: 14d54a6db9c55125adf6147dff4d8c49933a8a1ac71790a2ab1ea121c6c09d0a
// Source: lib/rubocop/cop/mixin/require_library.rb
// Source SHA-256: 22b39ca5d2c992be6d080c2aaad3763f46d31d972428bcb4b0dd45276af1b203
// Source: lib/rubocop/cop/mixin/rescue_node.rb
// Source SHA-256: f2884787518a4e423e1dd6e877aed6ff68b3341baad1bf99db8b2332140b28bf
// Source: lib/rubocop/cop/mixin/space_after_punctuation.rb
// Source SHA-256: 0831cab3af375e72e96846413e6c1383fba4e8cbdda5b12aeb4974e3bbc6bf49
// Source: lib/rubocop/cop/mixin/space_before_punctuation.rb
// Source SHA-256: 0ddfcde03c0d680243f1bb4084658f1d171e4f7d4a34305f27690943bf129dfc
// Source: lib/rubocop/cop/mixin/statement_modifier.rb
// Source SHA-256: c45a2318b642b5a3336c3591423f074152d95e5b83ca5069a382e5d2db8aa190
// Source: lib/rubocop/cop/mixin/string_help.rb
// Source SHA-256: 5bcea3ba2368161f24a00b875da9dc59465e2554438a9d2f0add3dd308de9d3b
// Source: lib/rubocop/cop/mixin/surrounding_space.rb
// Source SHA-256: 54db61d6bedac50f539d13e8580ef8861079e3de52a122d4d08976e956385610
// Source: lib/rubocop/cop/mixin/trailing_comma.rb
// Source SHA-256: 6da90c46665b3108a509cbc604777f6c4abec47978852cb82fab05e345a3a6d8
// Source: lib/rubocop/cop/mixin/uncommunicative_name.rb
// Source SHA-256: 4050d3508e49cd64552f02caeb6780ce47ca6be4dc9fb3f599184a28bab55c23
// Source: lib/rubocop/cop/mixin/unused_argument.rb
// Source SHA-256: 7bf5d074e5e5f80801550f8b77cac79a58a3ff44d7471c149b2262ef89c3a048
// Source: lib/rubocop/cop/mixin/visibility_help.rb
// Source SHA-256: 130a58e9558787af5b6f3b67bf2bb90129f5d424feeb8c3902dc11aaaea8f68e

use std::collections::{BTreeSet, HashMap, HashSet};
use std::ops::Range;

use regex::Regex;
use unicode_width::UnicodeWidthStr;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::cop::corrector::Corrector;
use crate::rubocop::cop::correctors::RequireLibraryCorrector;
use crate::rubocop::cop::mixin::range_help::RangeHelp;

pub(crate) fn display_column(text: &str, tab_width: usize) -> usize {
    text.split('\t')
        .enumerate()
        .map(|(index, part)| part.width() + usize::from(index > 0) * tab_width)
        .sum()
}
pub(crate) fn indentation(line: &str) -> &str {
    &line[..line.len() - line.trim_start_matches([' ', '\t']).len()]
}
pub(crate) fn within(inner: Range<usize>, outer: Range<usize>) -> bool {
    inner.start >= outer.start && inner.end <= outer.end
}
pub(crate) fn alignment_offset(actual: usize, expected: usize) -> isize {
    expected as isize - actual as isize
}

pub(crate) fn assignment_rhs(node: NodeRef<'_>) -> Option<NodeRef<'_>> {
    match node.kind() {
        "send" | "csend" => node.last_argument(),
        "masgn" | "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn" | "casgn" | "op_asgn" | "or_asgn"
        | "and_asgn" => node.rhs(),
        _ => None,
    }
}
pub(crate) fn check_assignment_target(node: NodeRef<'_>) -> Option<(NodeRef<'_>, NodeRef<'_>)> {
    assignment_rhs(node).map(|rhs| (node, rhs))
}
pub(crate) trait CheckAssignmentRuntime {
    fn check_assignment(&mut self, node: NodeRef<'_>, rhs: NodeRef<'_>);

    fn on_assignment(&mut self, node: NodeRef<'_>) {
        if let Some(rhs) = assignment_rhs(node) {
            self.check_assignment(node, rhs);
        }
    }

    fn on_send_assignment(&mut self, node: NodeRef<'_>) {
        if node.call_type() {
            self.on_assignment(node);
        }
    }
}
pub(crate) fn all_on_same_line(nodes: &[NodeRef<'_>]) -> bool {
    nodes
        .first()
        .zip(nodes.last())
        .is_none_or(|(first, last)| first.first_line() == last.last_line())
}
pub(crate) fn already_on_multiple_lines(node: NodeRef<'_>) -> bool {
    node.multiline()
}
pub(crate) fn within_column_limit(source: &str, start_column: usize, max: usize) -> bool {
    start_column
        + source
            .lines()
            .map(UnicodeWidthStr::width)
            .max()
            .unwrap_or(0)
        <= max
}
pub(crate) fn suitable_as_single_line(source: &str, max: usize, has_comment: bool) -> bool {
    !has_comment && to_single_line(source).chars().count() <= max
}
pub(crate) fn to_single_line(source: &str) -> String {
    let mut result = Regex::new("\" *\\\\\n\\s*'")
        .expect("static regex")
        .replace_all(source, "\" + '")
        .into_owned();
    result = Regex::new("' *\\\\\n\\s*\"")
        .expect("static regex")
        .replace_all(&result, "' + \"")
        .into_owned();
    result = Regex::new("\" *\\\\\n\\s*\"")
        .expect("static regex")
        .replace_all(&result, "")
        .into_owned();
    result = Regex::new("' *\\\\\n\\s*'")
        .expect("static regex")
        .replace_all(&result, "")
        .into_owned();
    result = Regex::new(r"\n\s*(&?\.\w)")
        .expect("static regex")
        .replace_all(&result, "$1")
        .into_owned();
    Regex::new(r"\s*\\?\n\s*")
        .expect("static regex")
        .replace_all(&result, " ")
        .into_owned()
}

pub(crate) fn suitable_as_single_line_node(
    node: NodeRef<'_>,
    processed_source: &ProcessedSource<'_>,
    max_line_length: Option<usize>,
) -> bool {
    let too_long = max_line_length.is_some_and(|max| {
        let source = processed_source
            .lines_slice(
                node.first_line().saturating_sub(1),
                node.last_line() - node.first_line() + 1,
            )
            .join("\n");
        to_single_line(&source).chars().count() > max
    });
    let commented = processed_source
        .comments()
        .iter()
        .any(|comment| (node.first_line()..=node.last_line()).contains(&comment.line));
    let unsafe_structure = !node
        .each_descendant(&["if", "case", "kwbegin", "any_def", "rescue", "ensure"])
        .is_empty()
        || node
            .each_descendant(&["dstr", "str"])
            .into_iter()
            .any(|string| {
                string.heredoc()
                    || string
                        .string_child(0)
                        .is_some_and(|value| value.contains('\n'))
            })
        || node
            .each_descendant(&["begin", "sym"])
            .into_iter()
            .any(NodeRef::multiline);
    !too_long && !commented && !unsafe_structure
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CodeLengthOptions {
    pub(crate) count_comments: bool,
    pub(crate) count_as_one: Vec<String>,
}
pub(crate) fn code_length(source: &str, options: &CodeLengthOptions) -> usize {
    source
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            !trimmed.is_empty() && (options.count_comments || !trimmed.starts_with('#'))
        })
        .count()
}
pub(crate) fn code_length_for_node(
    node: NodeRef<'_>,
    processed_source: &ProcessedSource<'_>,
    options: &CodeLengthOptions,
) -> Result<usize, String> {
    for kind in &options.count_as_one {
        if !matches!(kind.as_str(), "array" | "hash" | "heredoc" | "method_call") {
            return Err(format!(
                "Unknown foldable type: {kind:?}. Valid foldable types are: array, hash, heredoc, method_call."
            ));
        }
    }
    fn measured_lines(node: NodeRef<'_>) -> Vec<usize> {
        let body = match node.kind() {
            "class" | "module" | "sclass" | "block" | "numblock" | "itblock" | "def" | "defs" => {
                node.body()
            }
            "casgn" => node
                .expression()
                .and_then(|expression| match expression.kind() {
                    "block" | "numblock" | "itblock" => expression.body(),
                    _ => Some(expression),
                }),
            _ => Some(node),
        };
        body.map_or_else(Vec::new, |body| {
            (body.first_line()..=body.last_line()).collect()
        })
    }
    if matches!(node.kind(), "class" | "module")
        && node
            .body()
            .is_some_and(|body| matches!(body.kind(), "class" | "module"))
    {
        return Ok(0);
    }
    let mut lines = measured_lines(node);
    if matches!(node.kind(), "class" | "module") {
        let nested = node.each_descendant(&["class", "module"]);
        lines.retain(|line| {
            !nested
                .iter()
                .any(|inner| (inner.first_line()..=inner.last_line()).contains(line))
        });
    }
    let relevant = |line: usize| {
        processed_source
            .lines()
            .get(line.saturating_sub(1))
            .is_some_and(|source| {
                let trimmed = source.trim();
                !trimmed.is_empty() && (options.count_comments || !trimmed.starts_with('#'))
            })
    };
    let mut length = lines.iter().filter(|line| relevant(**line)).count();
    if options.count_as_one.is_empty() {
        return Ok(length);
    }
    let foldable = |candidate: NodeRef<'_>| {
        options.count_as_one.iter().any(|kind| match kind.as_str() {
            "array" => candidate.kind() == "array",
            "hash" => candidate.kind() == "hash",
            "heredoc" => candidate.heredoc(),
            "method_call" => candidate.call_type(),
            _ => false,
        })
    };
    let mut selected = Vec::new();
    fn collect_top_level<'ast>(
        node: NodeRef<'ast>,
        foldable: &impl Fn(NodeRef<'ast>) -> bool,
        selected: &mut Vec<NodeRef<'ast>>,
    ) {
        for child in node.child_nodes() {
            if matches!(child.kind(), "class" | "module") {
                continue;
            }
            if foldable(child) {
                selected.push(child);
            } else {
                collect_top_level(child, foldable, selected);
            }
        }
    }
    collect_top_level(node, &foldable, &mut selected);
    for folded in selected {
        let folded_length = measured_lines(folded)
            .into_iter()
            .filter(|line| relevant(*line))
            .count();
        if folded_length > 1 {
            length = length.saturating_sub(folded_length - 1);
        }
    }
    Ok(length)
}
pub(crate) fn code_length_message(label: &str, length: usize, max: usize) -> String {
    format!("{label} has too many lines. [{length}/{max}]")
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Comment {
    pub(crate) range: Range<usize>,
    pub(crate) line: usize,
    pub(crate) text: String,
}
pub(crate) fn comments_in_range<'a>(
    comments: &'a [Comment],
    range: &Range<usize>,
) -> Vec<&'a Comment> {
    comments
        .iter()
        .filter(|comment| comment.range.start >= range.start && comment.range.end <= range.end)
        .collect()
}
pub(crate) fn contains_comments(comments: &[Comment], range: &Range<usize>) -> bool {
    comments
        .iter()
        .any(|comment| comment.range.start >= range.start && comment.range.start < range.end)
}
pub(crate) fn comments_contain_disables(comments: &[Comment]) -> bool {
    comments.iter().any(|comment| {
        comment.text.contains("rubocop:disable") || comment.text.contains("rubocop:todo")
    })
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum DetectedStyle {
    Style,
    Opposite,
    Mixed,
    Unknown,
}
pub(crate) fn formatting_style(valid: usize, opposing: usize) -> DetectedStyle {
    match (valid > 0, opposing > 0) {
        (true, false) => DetectedStyle::Style,
        (false, true) => DetectedStyle::Opposite,
        (true, true) => DetectedStyle::Mixed,
        _ => DetectedStyle::Unknown,
    }
}
pub(crate) fn valid_formatting_name(name: &str) -> bool {
    name.bytes()
        .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
}

pub(crate) fn documentation_comment(comment: &str, keywords: &[&str]) -> bool {
    let text = comment.trim_start_matches('#').trim_start();
    !text.is_empty()
        && !text.starts_with("rubocop:")
        && !text.starts_with("#!")
        && !keywords.iter().any(|keyword| text.starts_with(keyword))
}
pub(crate) fn preceding_comment(comments: &[Comment], line: usize) -> Option<&Comment> {
    comments
        .iter()
        .rev()
        .find(|comment| comment.line + 1 == line)
}
pub(crate) fn empty_line_required(lines: &[&str], line: usize) -> bool {
    line > 0
        && lines
            .get(line - 1)
            .is_some_and(|source| !source.trim().is_empty())
}
pub(crate) fn valid_body_style(has_beginning: bool, has_ending: bool, style: &str) -> bool {
    match style {
        "empty_lines" => has_beginning && has_ending,
        "no_empty_lines" => !has_beginning && !has_ending,
        "empty_lines_except_namespace" => has_beginning && has_ending,
        _ => false,
    }
}
pub(crate) fn end_keyword_aligned(end_column: usize, base_column: usize) -> bool {
    end_column == base_column
}
pub(crate) fn variable_alignment(
    keyword_column: usize,
    variable_column: usize,
    style: &str,
) -> usize {
    if style == "variable" {
        variable_column
    } else {
        keyword_column
    }
}

pub(crate) fn endless_to_multiline(
    name: &str,
    arguments: &str,
    body: &str,
    column: usize,
) -> String {
    let _ = column;
    format!("def {name}{arguments}\n  {body}\nend")
}
pub(crate) fn endless_method_replacement(node: NodeRef<'_>, missing: &str) -> Option<String> {
    let name = node.method_name()?;
    let arguments = node.arguments_node()?;
    let arguments = if !arguments.child_nodes().is_empty() {
        arguments.source().unwrap_or(missing)
    } else {
        missing
    };
    Some(endless_to_multiline(
        name,
        arguments,
        node.body()?.source()?,
        0,
    ))
}
pub(crate) fn correct_endless_to_multiline<'b, 's>(
    corrector: &mut Corrector<'b, 's>,
    node: NodeRef<'_>,
) {
    let Some(range) = node.source_range() else {
        return;
    };
    let Some(replacement) = endless_method_replacement(node, "") else {
        return;
    };
    corrector.replace(
        SourceRange::new(corrector.source_buffer(), range.start, range.end),
        replacement,
    );
}
pub(crate) fn superclass_allowed(actual: Option<&str>, allowed: &[&str]) -> bool {
    actual.is_some_and(|name| allowed.contains(&name))
}
pub(crate) fn enforced_superclass_offense<'ast>(
    node: NodeRef<'ast>,
    required_superclass: &str,
    base_pattern: impl Fn(NodeRef<'ast>) -> bool,
) -> Option<NodeRef<'ast>> {
    match node.kind() {
        "class" => {
            if node.identifier()?.short_name() == Some(required_superclass) {
                return None;
            }
            node.parent_class().filter(|parent| base_pattern(*parent))
        }
        "send" => {
            if node.method_name() != Some("new")
                || node
                    .receiver()
                    .and_then(|receiver| receiver.const_name())
                    .as_deref()
                    != Some("Class")
            {
                return None;
            }
            let assignment = match node.parent() {
                Some(parent) if parent.kind() == "casgn" => Some(parent),
                Some(parent) if matches!(parent.kind(), "block" | "numblock" | "itblock") => parent
                    .parent()
                    .filter(|ancestor| ancestor.kind() == "casgn"),
                _ => None,
            }?;
            if assignment.short_name() == Some(required_superclass) {
                return None;
            }
            node.arguments()
                .last()
                .copied()
                .filter(|parent| base_pattern(*parent))
        }
        _ => None,
    }
}
pub(crate) struct EnforceSuperclass<'name> {
    pub(crate) superclass: &'name str,
}
impl EnforceSuperclass<'_> {
    pub(crate) fn included(&self) -> &'static str {
        "`RuboCop::Cop::EnforceSuperclass` is deprecated and will be removed in RuboCop 2.0. Please upgrade to RuboCop Rails 2.9 or newer to continue."
    }
    pub(crate) fn on_class<'ast>(
        &self,
        node: NodeRef<'ast>,
        base_pattern: impl Fn(NodeRef<'ast>) -> bool,
    ) -> Option<NodeRef<'ast>> {
        self.class_definition(node, base_pattern)
    }
    pub(crate) fn on_send<'ast>(
        &self,
        node: NodeRef<'ast>,
        base_pattern: impl Fn(NodeRef<'ast>) -> bool,
    ) -> Option<NodeRef<'ast>> {
        self.class_new_definition(node, base_pattern)
    }
    pub(crate) fn class_definition<'ast>(
        &self,
        node: NodeRef<'ast>,
        base_pattern: impl Fn(NodeRef<'ast>) -> bool,
    ) -> Option<NodeRef<'ast>> {
        (node.kind() == "class")
            .then(|| enforced_superclass_offense(node, self.superclass, base_pattern))
            .flatten()
    }
    pub(crate) fn class_new_definition<'ast>(
        &self,
        node: NodeRef<'ast>,
        base_pattern: impl Fn(NodeRef<'ast>) -> bool,
    ) -> Option<NodeRef<'ast>> {
        (node.kind() == "send")
            .then(|| enforced_superclass_offense(node, self.superclass, base_pattern))
            .flatten()
    }
}
pub(crate) fn first_element_needs_line_break(opening_line: usize, first_line: usize) -> bool {
    opening_line == first_line
}
pub(crate) fn first_element_line_break_offense<'ast>(
    node: NodeRef<'ast>,
    children: &[NodeRef<'ast>],
    start: Option<NodeRef<'ast>>,
    ignore_last: bool,
) -> Option<NodeRef<'ast>> {
    let first = children
        .iter()
        .min_by_key(|child| child.first_line())
        .copied()?;
    let start_line = start.unwrap_or(node).first_line();
    if start_line != first.first_line() {
        return None;
    }
    let last_line = children
        .iter()
        .map(|child| {
            if ignore_last {
                child.first_line()
            } else {
                child.last_line()
            }
        })
        .max()?;
    (start_line != last_line).then_some(first)
}
pub(crate) fn first_by_line<'ast>(nodes: &[NodeRef<'ast>]) -> Option<NodeRef<'ast>> {
    nodes.iter().min_by_key(|node| node.first_line()).copied()
}
pub(crate) fn last_line(nodes: &[NodeRef<'_>], ignore_last: bool) -> Option<usize> {
    nodes
        .iter()
        .map(|node| {
            if ignore_last {
                node.first_line()
            } else {
                node.last_line()
            }
        })
        .max()
}
pub(crate) fn check_children_line_break<'ast>(
    node: NodeRef<'ast>,
    children: &[NodeRef<'ast>],
    start: Option<NodeRef<'ast>>,
    ignore_last: bool,
) -> Option<NodeRef<'ast>> {
    first_element_line_break_offense(node, children, start, ignore_last)
}
pub(crate) fn method_uses_parentheses(node: NodeRef<'_>, limit: NodeRef<'_>) -> bool {
    let Some(node_range) = node.source_range() else {
        return false;
    };
    let Some(limit_range) = limit.source_range() else {
        return false;
    };
    if node.first_line() != limit.first_line() || limit_range.start < node_range.start {
        return false;
    }
    let prefix = node.source().unwrap_or("");
    let relative = limit_range
        .start
        .saturating_sub(node_range.start)
        .min(prefix.chars().count());
    prefix
        .chars()
        .take(relative)
        .collect::<String>()
        .trim_end()
        .ends_with('(')
}
pub(crate) fn method_first_element_line_break_offense<'ast>(
    node: NodeRef<'ast>,
    children: &[NodeRef<'ast>],
    ignore_last: bool,
) -> Option<NodeRef<'ast>> {
    let first = children.first().copied()?;
    method_uses_parentheses(node, first)
        .then(|| first_element_line_break_offense(node, children, None, ignore_last))
        .flatten()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum FrozenStringLiteral {
    Enabled,
    Disabled,
    Unspecified,
}
pub(crate) fn frozen_string_literal(source: &str) -> FrozenStringLiteral {
    for line in source.lines() {
        let trimmed = line.trim();
        if !trimmed.is_empty() && !trimmed.starts_with('#') {
            break;
        }
        let normalized = line.trim().trim_start_matches('#').trim();
        if let Some(value) = normalized.strip_prefix("frozen_string_literal:") {
            return match value.trim() {
                "true" => FrozenStringLiteral::Enabled,
                "false" => FrozenStringLiteral::Disabled,
                _ => FrozenStringLiteral::Unspecified,
            };
        }
    }
    FrozenStringLiteral::Unspecified
}
pub(crate) fn uninterpolated_string(node: NodeRef<'_>) -> bool {
    node.kind() == "str"
        || matches!(node.kind(), "dstr")
            && node.child_nodes().iter().all(|child| child.kind() == "str")
}

pub(crate) fn gem_specification_call(receiver: Option<&str>, method: &str) -> bool {
    receiver == Some("Gem::Specification") && method == "new"
}
pub(crate) fn gem_assignment_method(method: &str) -> bool {
    method.starts_with("add_") && method.ends_with("dependency") || method.ends_with('=')
}
pub(crate) fn gemspec_block_variable(node: NodeRef<'_>) -> Option<&str> {
    if !matches!(node.kind(), "block" | "numblock" | "itblock") {
        return None;
    }
    let send = node.send_node()?;
    if send.method_name() != Some("new")
        || send
            .receiver()
            .is_none_or(|receiver| receiver.const_name().as_deref() != Some("Gem::Specification"))
    {
        return None;
    }
    node.arguments()
        .first()
        .and_then(|argument| argument.name())
}
pub(crate) fn gemspec_assignment_declarations<'ast>(
    root: NodeRef<'ast>,
    block_variable: &str,
) -> Vec<NodeRef<'ast>> {
    root.each_descendant(&["send"])
        .into_iter()
        .filter(|send| {
            send.receiver().is_some_and(|receiver| {
                receiver.kind() == "lvar"
                    && receiver
                        .name()
                        .is_some_and(|name| name == block_variable || matches!(name, "_1" | "it"))
            })
        })
        .collect()
}
pub(crate) fn gem_specification<'ast>(root: NodeRef<'ast>) -> Vec<(NodeRef<'ast>, &'ast str)> {
    root.each_node(&["block", "numblock", "itblock"])
        .into_iter()
        .filter_map(|node| gemspec_block_variable(node).map(|name| (node, name)))
        .collect()
}
pub(crate) fn match_block_variable_name(root: NodeRef<'_>, receiver_name: &str) -> bool {
    gem_specification(root)
        .first()
        .is_some_and(|(_, block_name)| *block_name == receiver_name)
}
pub(crate) fn assignment_method_declarations<'ast>(root: NodeRef<'ast>) -> Vec<NodeRef<'ast>> {
    let Some((block, variable)) = gem_specification(root).first().copied() else {
        return Vec::new();
    };
    gemspec_assignment_declarations(block, variable)
}
pub(crate) fn indexed_assignment_method_declarations<'ast>(
    root: NodeRef<'ast>,
) -> Vec<NodeRef<'ast>> {
    let Some((block, variable)) = gem_specification(root).first().copied() else {
        return Vec::new();
    };
    block
        .each_descendant(&["send"])
        .into_iter()
        .filter(|send| {
            send.method_name() == Some("[]=")
                && send
                    .receiver()
                    .filter(|receiver| receiver.kind() == "send")
                    .and_then(NodeRef::receiver)
                    .is_some_and(|receiver| {
                        receiver.kind() == "lvar"
                            && receiver
                                .name()
                                .is_some_and(|name| name == variable || matches!(name, "_1" | "it"))
                    })
        })
        .collect()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HashAlignmentDelta {
    pub(crate) key: isize,
    pub(crate) separator: isize,
    pub(crate) value: isize,
}
pub(crate) fn hash_alignment_delta(
    first: (usize, usize, usize),
    current: (usize, usize, usize),
) -> HashAlignmentDelta {
    HashAlignmentDelta {
        key: first.0 as isize - current.0 as isize,
        separator: first.1 as isize - current.1 as isize,
        value: first.2 as isize - current.2 as isize,
    }
}
pub(crate) fn checkable_hash_layout(pairs: &[(usize, usize)]) -> bool {
    pairs.len() > 1 && pairs.iter().all(|(key, value)| key == value)
}

pub(crate) fn hash_value_omittable(key: &str, value: &str) -> bool {
    key == value && valid_formatting_name(key)
}
pub(crate) fn mixed_hash_shorthand(pairs: &[(String, Option<String>)]) -> bool {
    pairs.iter().any(|(_, v)| v.is_none()) && pairs.iter().any(|(_, v)| v.is_some())
}
pub(crate) fn preferred_hash_subset(method: &str, negated: bool) -> Option<&'static str> {
    Some(match (method, negated) {
        ("select" | "filter", false) => "slice",
        ("reject", false) => "except",
        ("select" | "filter", true) => "except",
        ("reject", true) => "slice",
        _ => return None,
    })
}
pub(crate) fn transformed_hash_method(method: &str) -> Option<&'static str> {
    Some(match method {
        "map" | "collect" => "transform_values",
        "each_with_object" => "transform_values",
        "to_h" => "transform_keys",
        _ => return None,
    })
}
pub(crate) fn hash_transform_correction(
    receiver: &str,
    method: &str,
    argument: &str,
    body: &str,
) -> String {
    format!("{receiver}.{method} {{ |{argument}| {body} }}")
}

pub(crate) fn heredoc_type(delimiter: &str) -> Option<&'static str> {
    if delimiter.starts_with("<<~") {
        Some("squiggly")
    } else if delimiter.starts_with("<<-") {
        Some("dash")
    } else if delimiter.starts_with("<<") {
        Some("bare")
    } else {
        None
    }
}
pub(crate) fn heredoc_delimiter(delimiter: &str) -> &str {
    delimiter
        .trim_start_matches("<<~")
        .trim_start_matches("<<-")
        .trim_matches(['\'', '\"'])
}
pub(crate) fn heredoc_indent(line: &str) -> usize {
    line.chars().take_while(|c| matches!(c, ' ' | '\t')).count()
}
pub(crate) fn heredoc_indent_level(source: &str) -> usize {
    source
        .split_inclusive('\n')
        .filter_map(|line| {
            let whitespace = line
                .chars()
                .take_while(|character| character.is_whitespace())
                .collect::<String>();
            (!whitespace.ends_with('\n')).then_some(whitespace.chars().count())
        })
        .min()
        .unwrap_or(0)
}
pub(crate) fn heredoc_delimiter_string(source: &str) -> String {
    Regex::new(r#"(<<[~-]?)[\x27\x22`]?([^\x27\x22`]+)[\x27\x22`]?"#)
        .expect("static regex")
        .captures(source)
        .and_then(|captures| captures.get(1))
        .map_or_else(String::new, |capture| capture.as_str().to_owned())
}
pub(crate) fn heredoc_type_string(source: &str) -> String {
    Regex::new(r#"(<<[~-]?)[\x27\x22`]?([^\x27\x22`]+)[\x27\x22`]?"#)
        .expect("static regex")
        .captures(source)
        .and_then(|captures| captures.get(2))
        .map_or_else(String::new, |capture| capture.as_str().to_owned())
}
pub(crate) trait HeredocRuntime {
    fn on_heredoc(&mut self, node: NodeRef<'_>);

    fn on_string(&mut self, node: NodeRef<'_>) {
        if node.heredoc() {
            self.on_heredoc(node);
        }
    }
}
pub(crate) fn interpolation_nodes(node: NodeRef<'_>) -> Vec<NodeRef<'_>> {
    if matches!(node.kind(), "dstr" | "dsym" | "regexp" | "xstr") {
        node.child_nodes()
            .into_iter()
            .filter(|child| child.kind() == "begin")
            .collect()
    } else {
        Vec::new()
    }
}
pub(crate) trait InterpolationRuntime {
    fn on_interpolation(&mut self, begin_node: NodeRef<'_>);

    fn on_node_with_interpolations(&mut self, node: NodeRef<'_>) {
        for interpolation in interpolation_nodes(node) {
            self.on_interpolation(interpolation);
        }
    }
}

pub(crate) fn line_length(line: &str, tab_width: usize) -> usize {
    display_column(line.trim_end_matches(['\n', '\r']), tab_width)
}
pub(crate) fn line_length_without_directive(line: &str, tab_width: usize) -> usize {
    let prefix = line.split("# rubocop:").next().unwrap_or(line).trim_end();
    line_length(prefix, tab_width)
}
pub(crate) fn excessive_range(line: &str, max: usize, tab_width: usize) -> Option<Range<usize>> {
    (line_length(line, tab_width) > max).then(|| max..line.chars().count())
}
pub(crate) fn valid_uri(text: &str) -> bool {
    Regex::new(r"\A(?:https?|ftp)://[^\s]+\z")
        .unwrap()
        .is_match(text)
}
pub(crate) fn qualified_name(text: &str) -> bool {
    Regex::new(r"\A(?:::)?[A-Z]\w*(?:::[A-Z]\w*)+\z")
        .unwrap()
        .is_match(text)
}

pub(crate) fn method_complexity(node: NodeRef<'_>, counted_methods: &[&str]) -> usize {
    1 + node
        .descendants()
        .into_iter()
        .filter(|child| {
            matches!(
                child.kind(),
                "if" | "while" | "until" | "for" | "resbody" | "when" | "in_pattern" | "and" | "or"
            ) || child
                .method_name()
                .is_some_and(|name| counted_methods.contains(&name))
        })
        .count()
}
pub(crate) fn define_method_block(node: NodeRef<'_>) -> bool {
    matches!(node.kind(), "block" | "numblock" | "itblock")
        && node.method_name() == Some("define_method")
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum IndentationStyle {
    Special,
    Consistent,
    AlignBraces,
}
pub(crate) fn expected_element_column(
    base: usize,
    width: usize,
    style: IndentationStyle,
    first_column: usize,
) -> usize {
    match style {
        IndentationStyle::Special => base + width,
        IndentationStyle::Consistent => first_column,
        IndentationStyle::AlignBraces => base,
    }
}
pub(crate) fn incorrect_indentation(actual: usize, expected: usize) -> bool {
    actual != expected
}
pub(crate) fn grouped_expression(node: NodeRef<'_>) -> bool {
    node.kind() == "begin" && node.loc("begin").is_some()
}
pub(crate) fn postfix_conditional(node: NodeRef<'_>) -> bool {
    matches!(node.kind(), "if" | "while" | "until") && node.modifier_form()
}

pub(crate) fn closing_brace_on_same_line(last_line: usize, closing_line: usize) -> bool {
    last_line == closing_line
}
pub(crate) fn symmetrical_braces(
    opening_line: usize,
    first_line: usize,
    last_line: usize,
    closing_line: usize,
) -> bool {
    (opening_line == first_line) == (last_line == closing_line)
}
pub(crate) fn ignored_literal(implicit: bool, empty: bool) -> bool {
    implicit || empty
}

pub(crate) fn gem_canonical_name(name: &str, consider_punctuation: bool) -> String {
    let name = if consider_punctuation {
        name.to_owned()
    } else {
        name.replace(['-', '_'], "")
    };
    name.to_lowercase()
}
pub(crate) fn gem_out_of_order(previous: &str, current: &str, consider_punctuation: bool) -> bool {
    gem_canonical_name(previous, consider_punctuation)
        > gem_canonical_name(current, consider_punctuation)
}
pub(crate) fn case_insensitive_out_of_order(
    string_a: &str,
    string_b: &str,
    consider_punctuation: bool,
) -> bool {
    gem_canonical_name(string_a, consider_punctuation)
        < gem_canonical_name(string_b, consider_punctuation)
}
pub(crate) fn find_gem_name(node: NodeRef<'_>) -> Option<String> {
    if node.kind() == "str" {
        node.str_content().map(str::to_owned)
    } else {
        node.receiver().and_then(find_gem_name)
    }
}
pub(crate) fn gem_name(declaration_node: NodeRef<'_>) -> Option<String> {
    declaration_node
        .arguments()
        .first()
        .copied()
        .and_then(find_gem_name)
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct OrderedGemOffense {
    pub(crate) previous: String,
    pub(crate) current: String,
    pub(crate) offense_node_source: String,
}
pub(crate) fn register_offense(
    previous: NodeRef<'_>,
    current: NodeRef<'_>,
) -> Option<OrderedGemOffense> {
    Some(OrderedGemOffense {
        previous: gem_name(current)?,
        current: gem_name(previous)?,
        offense_node_source: current.source()?.to_owned(),
    })
}
pub(crate) fn consecutive_lines(previous: usize, current: usize) -> bool {
    current == previous + 1
}

pub(crate) fn percent_array_message(kind: &str) -> String {
    format!("Use `%{kind}` for an array of words.")
}
pub(crate) fn bracket_array(words: &[&str], quote: char) -> String {
    format!(
        "[{}]",
        words
            .iter()
            .map(|word| format!("{quote}{word}{quote}"))
            .collect::<Vec<_>>()
            .join(", ")
    )
}
pub(crate) fn percent_array_context_valid(has_comments: bool, has_splat: bool) -> bool {
    !has_comments && !has_splat
}

pub(crate) fn aligned_with_any(column: usize, others: &[usize]) -> bool {
    others.contains(&column)
}
pub(crate) fn allow_for_alignment(columns: &[usize]) -> bool {
    columns.windows(2).all(|pair| pair[0] <= pair[1])
}
pub(crate) fn project_index_signature<'a>(
    document_uris: impl IntoIterator<Item = &'a str>,
) -> Vec<String> {
    let mut signatures = document_uris
        .into_iter()
        .filter(|uri| *uri != "rubydex:built-in")
        .map(|uri| {
            let mut path = uri.strip_prefix("file://").unwrap_or(uri);
            if path.starts_with('/')
                && path
                    .as_bytes()
                    .get(1..3)
                    .is_some_and(|prefix| prefix[0].is_ascii_alphabetic() && prefix[1] == b':')
            {
                path = &path[1..];
            }
            let (mtime, size) = std::fs::metadata(path).map_or((0.0, 0), |metadata| {
                let modified = metadata
                    .modified()
                    .ok()
                    .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
                    .map_or(0.0, |duration| duration.as_secs_f64());
                (modified, metadata.len())
            });
            format!("{path}:{mtime}:{size}")
        })
        .collect::<Vec<_>>();
    signatures.sort();
    signatures
}
pub(crate) fn project_index_checksum<'a>(
    document_uris: impl IntoIterator<Item = &'a str>,
) -> String {
    crate::rubocop::ast::processed_source::sha1_hex(
        project_index_signature(document_uris).join("\n").as_bytes(),
    )
}

pub(crate) fn require_library_name(node: NodeRef<'_>) -> Option<&str> {
    let valid_receiver = node.receiver().is_none()
        || node
            .receiver()
            .is_some_and(|receiver| receiver.global_const("Kernel"));
    (node.call_type() && valid_receiver && node.method_name() == Some("require"))
        .then(|| node.first_argument()?.string_child(0))
        .flatten()
}
pub(crate) fn require_any_library(required: &HashSet<String>, alternatives: &[&str]) -> bool {
    alternatives
        .iter()
        .any(|library| required.contains(*library))
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RequireLibrary {
    required_libs: BTreeSet<String>,
}

impl RequireLibrary {
    pub(crate) fn on_new_investigation(&mut self) {
        self.required_libs.clear();
    }

    pub(crate) fn on_send(&mut self, node: NodeRef<'_>) {
        if node.parent().and_then(NodeRef::parent).is_some() {
            return;
        }
        if let Some(name) = require_library_name(node) {
            self.required_libs.insert(name.to_owned());
        }
    }

    pub(crate) fn required_libs(&self) -> &BTreeSet<String> {
        &self.required_libs
    }
}
pub(crate) fn ensure_required(required: &mut BTreeSet<String>, library: &str) -> bool {
    required.insert(library.into())
}
pub(crate) fn track_top_level_required_library(
    required: &mut BTreeSet<String>,
    node: NodeRef<'_>,
) -> Option<String> {
    if node.parent().and_then(NodeRef::parent).is_some() {
        return None;
    }
    let name = require_library_name(node)?.to_owned();
    required.insert(name.clone());
    Some(name)
}
pub(crate) fn ensure_required_library<'b, 's>(
    corrector: &mut Corrector<'b, 's>,
    mut node: NodeRef<'_>,
    library_name: &str,
    required: &BTreeSet<String>,
) {
    while node.parent().and_then(NodeRef::parent).is_some() {
        node = node.parent().unwrap();
    }
    if node.parent().is_some_and(|parent| parent.kind() == "begin") {
        if required.contains(library_name) {
            return;
        }
        let helper = RangeHelp::new(corrector.source_buffer());
        for sibling in node.right_siblings() {
            if require_library_name(sibling) == Some(library_name) {
                if let Some(range) = sibling.source_range() {
                    let range = SourceRange::new(corrector.source_buffer(), range.start, range.end);
                    corrector.remove(helper.range_by_whole_lines(range, true));
                }
            }
        }
    }
    if let Some(range) = node.source_range() {
        RequireLibraryCorrector::correct(
            corrector,
            SourceRange::new(corrector.source_buffer(), range.start, range.end),
            library_name,
        );
    }
}

pub(crate) fn rescued_exceptions(node: NodeRef<'_>) -> Vec<NodeRef<'_>> {
    if node.kind() == "resbody" {
        node.exceptions()
    } else if node.kind() == "rescue" {
        node.branch_nodes()
            .into_iter()
            .flat_map(NodeRef::exceptions)
            .collect()
    } else {
        Vec::new()
    }
}
pub(crate) fn rescue_modifier_locations(
    processed_source: &ProcessedSource<'_>,
) -> Vec<Range<usize>> {
    processed_source
        .tokens()
        .iter()
        .filter(|token| token.rescue_modifier())
        .map(|token| token.range.clone())
        .collect()
}
pub(crate) fn rescue_modifier(node: NodeRef<'_>) -> bool {
    node.kind() == "resbody" && node.loc_is("keyword", "rescue")
}
pub(crate) fn rescue_modifier_at(node: NodeRef<'_>, locations: &[Range<usize>]) -> bool {
    node.kind() == "resbody"
        && node
            .loc("keyword")
            .is_some_and(|keyword| locations.contains(&keyword.0))
}

pub(crate) fn missing_space_after(buffer: &SourceBuffer<'_>, position: usize) -> bool {
    buffer
        .character(position)
        .is_some_and(|c| !c.is_whitespace())
}
pub(crate) fn missing_space_before(buffer: &SourceBuffer<'_>, position: usize) -> bool {
    position > 0
        && buffer
            .character(position - 1)
            .is_some_and(|c| !c.is_whitespace())
}
pub(crate) fn punctuation_allowed(kind: &str) -> bool {
    matches!(kind, "comma" | "semicolon" | "colon")
}
pub(crate) fn missing_space_after_punctuation<'a>(
    tokens: &'a [crate::rubocop::ast::processed_source::SourceToken],
    offset: usize,
    space_style_before_right_curly: &str,
    mut kind: impl FnMut(
        &crate::rubocop::ast::processed_source::SourceToken,
        &crate::rubocop::ast::processed_source::SourceToken,
    ) -> Option<String>,
) -> Vec<(
    &'a crate::rubocop::ast::processed_source::SourceToken,
    String,
)> {
    tokens
        .windows(2)
        .filter_map(|pair| {
            let left = &pair[0];
            let right = &pair[1];
            let kind = kind(left, right)?;
            let missing = left.line == right.line && right.column == left.column + offset;
            let allowed = matches!(right.kind, "tRPAREN" | "tRBRACK" | "tPIPE" | "tSTRING_DEND");
            let right_curly_forbidden =
                right.right_curly_brace() && space_style_before_right_curly == "no_space";
            (missing && !allowed && !right_curly_forbidden).then_some((left, kind))
        })
        .collect()
}
pub(crate) fn spaces_before_punctuation<'a>(
    tokens: &'a [crate::rubocop::ast::processed_source::SourceToken],
    space_style_after_left_curly: &str,
    mut kind: impl FnMut(&crate::rubocop::ast::processed_source::SourceToken) -> Option<String>,
) -> Vec<(
    &'a crate::rubocop::ast::processed_source::SourceToken,
    Range<usize>,
    String,
)> {
    tokens
        .windows(2)
        .filter_map(|pair| {
            let left = &pair[0];
            let right = &pair[1];
            let kind = kind(right)?;
            let has_space = left.line == right.line && right.begin_pos() > left.end_pos();
            let required = left.left_curly_brace() && space_style_after_left_curly == "space";
            (has_space && !required).then_some((right, left.end_pos()..right.begin_pos(), kind))
        })
        .collect()
}
pub(crate) fn each_missing_space<'a>(
    tokens: &'a [crate::rubocop::ast::processed_source::SourceToken],
    space_style_after_left_curly: &str,
    kind: impl FnMut(&crate::rubocop::ast::processed_source::SourceToken) -> Option<String>,
) -> Vec<(
    &'a crate::rubocop::ast::processed_source::SourceToken,
    Range<usize>,
    String,
)> {
    spaces_before_punctuation(tokens, space_style_after_left_curly, kind)
}
pub(crate) fn space_required_after(
    token: &crate::rubocop::ast::processed_source::SourceToken,
    space_style_after_left_curly: &str,
) -> bool {
    token.left_curly_brace() && space_required_after_lcurly(space_style_after_left_curly)
}
pub(crate) fn space_required_after_lcurly(style: &str) -> bool {
    style == "space"
}
pub(crate) fn side_space_range<'b, 's>(
    buffer: &'b SourceBuffer<'s>,
    position: usize,
    left: bool,
) -> SourceRange<'b, 's> {
    let mut begin = position;
    let mut end = position;
    if left {
        while begin > 0
            && buffer
                .character(begin - 1)
                .is_some_and(|c| matches!(c, ' ' | '\t'))
        {
            begin -= 1
        }
    } else {
        while end < buffer.len()
            && buffer
                .character(end)
                .is_some_and(|c| matches!(c, ' ' | '\t'))
        {
            end += 1
        }
    }
    SourceRange::new(buffer, begin, end)
}

pub(crate) fn modifier_form(
    keyword: &str,
    condition: &str,
    body: &str,
    parenthesize: bool,
) -> String {
    let condition = if parenthesize {
        format!("({condition})")
    } else {
        condition.into()
    };
    format!("{body} {keyword} {condition}")
}
pub(crate) fn modifier_fits(source: &str, max: usize) -> bool {
    to_single_line(source).width() <= max
}
pub(crate) fn non_eligible_modifier(body: NodeRef<'_>, condition: NodeRef<'_>) -> bool {
    matches!(body.kind(), "begin" | "kwbegin" | "rescue" | "ensure") || condition.assignment()
}
pub(crate) fn inside_interpolation(node: NodeRef<'_>) -> bool {
    node.ancestors().iter().any(|ancestor| {
        ancestor.kind() == "begin"
            && ancestor
                .parent()
                .is_some_and(|parent| matches!(parent.kind(), "dstr" | "dsym" | "regexp" | "xstr"))
    })
}

pub(crate) const fn treat_comments_as_separators(configured: Option<bool>) -> bool {
    matches!(configured, Some(true))
}

pub(crate) fn get_source_range(
    node: Range<usize>,
    first_comment: Option<Range<usize>>,
    comments_as_separators: bool,
) -> Range<usize> {
    if comments_as_separators {
        node
    } else {
        first_comment.unwrap_or(node)
    }
}

pub(crate) fn space_between(
    buffer: &SourceBuffer<'_>,
    left_end: usize,
    right_begin: usize,
) -> bool {
    buffer
        .slice(left_end..right_begin)
        .chars()
        .any(char::is_whitespace)
}
pub(crate) fn empty_brackets(source: &str) -> bool {
    matches!(source, "[]" | "{}" | "()")
}
pub(crate) fn extra_space(source: &str) -> bool {
    source.chars().filter(|c| c.is_whitespace()).count() > 1
}

pub(crate) fn should_have_trailing_comma(
    multiline: bool,
    style: &str,
    last_item_precedes_newline: bool,
) -> bool {
    should_have_trailing_comma_for(
        style,
        multiline,
        last_item_precedes_newline,
        false,
        last_item_precedes_newline,
    )
}
pub(crate) fn should_have_trailing_comma_for(
    style: &str,
    multiline: bool,
    no_elements_on_same_line: bool,
    method_name_and_arguments_on_same_line: bool,
    last_item_precedes_newline: bool,
) -> bool {
    match style {
        "comma" => multiline && no_elements_on_same_line,
        "consistent_comma" => multiline && !method_name_and_arguments_on_same_line,
        "diff_comma" => multiline && last_item_precedes_newline,
        _ => false,
    }
}
pub(crate) fn trailing_comma_range(source: &str) -> Option<usize> {
    source
        .char_indices()
        .rev()
        .find(|(_, c)| !c.is_whitespace())
        .and_then(|(index, c)| (c == ',').then_some(index))
}
pub(crate) fn any_heredoc(nodes: &[NodeRef<'_>]) -> bool {
    nodes.iter().any(|node| node.heredoc())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NameIssue {
    TooShort,
    Forbidden,
    EndsWithNumber,
    Uppercase,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NamePolicy {
    pub(crate) min_length: usize,
    pub(crate) allowed: HashSet<String>,
    pub(crate) forbidden: HashSet<String>,
    pub(crate) allow_numbers: bool,
    pub(crate) allow_uppercase: bool,
}
pub(crate) fn name_issues(name: &str, policy: &NamePolicy) -> Vec<NameIssue> {
    if policy.allowed.contains(name) {
        return Vec::new();
    }
    let mut issues = Vec::new();
    if name.chars().count() < policy.min_length {
        issues.push(NameIssue::TooShort)
    }
    if policy.forbidden.contains(name) {
        issues.push(NameIssue::Forbidden)
    }
    if !policy.allow_numbers && name.chars().last().is_some_and(|c| c.is_ascii_digit()) {
        issues.push(NameIssue::EndsWithNumber)
    }
    if !policy.allow_uppercase && name.chars().any(|c| c.is_ascii_uppercase()) {
        issues.push(NameIssue::Uppercase)
    }
    issues
}
pub(crate) fn argument_unused(name: &str, references: &HashSet<String>) -> bool {
    !name.starts_with('_') && !references.contains(name)
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Visibility {
    Public,
    Protected,
    Private,
}
pub(crate) fn visibility_from_method(method: &str) -> Option<Visibility> {
    Some(match method {
        "public" => Visibility::Public,
        "protected" => Visibility::Protected,
        "private" => Visibility::Private,
        _ => return None,
    })
}
pub(crate) fn node_visibility(
    method_name: &str,
    inline: Option<Visibility>,
    current: Visibility,
) -> Visibility {
    inline
        .or_else(|| visibility_from_method(method_name))
        .unwrap_or(current)
}
pub(crate) fn visibility_block(node: NodeRef<'_>) -> Option<Visibility> {
    (node.kind() == "send" && node.receiver().is_none() && node.arguments().is_empty())
        .then(|| visibility_from_method(node.method_name()?))
        .flatten()
}
pub(crate) fn visibility_inline_on_def(node: NodeRef<'_>) -> Option<Visibility> {
    (node.kind() == "send"
        && node.receiver().is_none()
        && node
            .first_argument()
            .is_some_and(|argument| argument.kind() == "def"))
    .then(|| visibility_from_method(node.method_name()?))
    .flatten()
}
pub(crate) fn visibility_inline_on_method_name(
    node: NodeRef<'_>,
    method_name: &str,
) -> Option<Visibility> {
    (node.kind() == "send"
        && node.receiver().is_none()
        && node.first_argument().is_some_and(|argument| {
            argument.kind() == "sym" && argument.symbol_child(0) == Some(method_name)
        }))
    .then(|| visibility_from_method(node.method_name()?))
    .flatten()
}
pub(crate) fn exact_node_visibility(node: NodeRef<'_>) -> Visibility {
    if node.kind() == "def" {
        let mut outer = node;
        while outer.parent().is_some_and(|parent| parent.kind() == "defs") {
            outer = outer.parent().unwrap();
        }
        if let Some(visibility) = outer.parent().and_then(visibility_inline_on_def) {
            return visibility;
        }
        if let Some(name) = node.method_name() {
            if let Some(visibility) = node
                .right_siblings()
                .into_iter()
                .rev()
                .find_map(|sibling| visibility_inline_on_method_name(sibling, name))
            {
                return visibility;
            }
        }
    }
    node.left_siblings()
        .into_iter()
        .rev()
        .find_map(visibility_block)
        .unwrap_or(Visibility::Public)
}
pub(crate) fn find_visibility_end(node: NodeRef<'_>) -> Option<NodeRef<'_>> {
    let current = exact_node_visibility(node);
    let right = node.right_siblings();
    right
        .iter()
        .copied()
        .find(|candidate| exact_node_visibility(*candidate) != current)
        .or_else(|| right.last().copied())
}
pub(crate) fn visibility_span(
    nodes: &[(usize, Option<Visibility>)],
    index: usize,
    current: Visibility,
) -> (usize, usize, Visibility) {
    let start = nodes[..=index]
        .iter()
        .rposition(|(_, visibility)| visibility.is_some())
        .unwrap_or(0);
    let end = nodes[index + 1..]
        .iter()
        .position(|(_, visibility)| visibility.is_some())
        .map_or(nodes.len(), |offset| index + 1 + offset);
    (
        nodes[start].0,
        nodes[end.saturating_sub(1)].0,
        nodes[start].1.unwrap_or(current),
    )
}

pub(crate) fn grouped_by_line<'ast>(nodes: &[NodeRef<'ast>]) -> HashMap<usize, Vec<NodeRef<'ast>>> {
    let mut grouped = HashMap::new();
    for node in nodes {
        grouped
            .entry(node.first_line())
            .or_insert_with(Vec::new)
            .push(*node);
    }
    grouped
}
// RuboCop API ownership: lib/rubocop/cop/mixin/enforce_superclass.rb => on_send
// RuboCop API ownership: lib/rubocop/cop/mixin/require_library.rb => on_new_investigation, on_send
// RuboCop API ownership: lib/rubocop/cop/mixin/space_before_punctuation.rb => on_new_investigation
