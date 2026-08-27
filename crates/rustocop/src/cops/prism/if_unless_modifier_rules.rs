use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::cop::mixin::statement_modifier::StatementModifier;

use super::*;

define_cops! {
    IfUnlessModifier => "Style/IfUnlessModifier" => compatibility_source(if_unless_modifier),
}

const MODIFIER_MESSAGE: &str = "Favor modifier `{keyword}` usage when having a single-line body. Another good alternative is the usage of control flow `&&`/`||`.";

fn if_unless_modifier(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let source = context.source().to_owned();
    if !if_unless_modifier_has_keyword(&source) {
        return;
    }
    let version = context.target_ruby_version();
    let ruby_version = format!("{}.{}", version.major(), version.minor())
        .parse()
        .unwrap_or(3.4);
    let Ok(processed) = ProcessedSource::new(&source, ruby_version, None, ParserEngine::Whitequark)
    else {
        return;
    };
    let Some(root) = processed.ast() else {
        return;
    };
    let max_line_length = context
        .related_config_value("Layout/LineLength", "Max")
        .and_then(|value| value.parse().ok())
        .or(Some(120));
    let statement_modifier = StatementModifier::new(
        &processed,
        max_line_length,
        "Style/IfUnlessModifier",
    );

    for node in root.each_node(&["if"]) {
        let Some(keyword_range) = node.loc("keyword").map(|location| location.0.clone()) else {
            continue;
        };
        let keyword = node.keyword_name().unwrap_or("if");
        if if_unless_modifier_endless_method(node.body())
            || node.ancestors().iter().any(|ancestor| ancestor.kind() == "dstr")
        {
            continue;
        }
        let Some(condition) = node.condition() else {
            continue;
        };
        if if_unless_modifier_defined_argument_is_undefined(node, condition)
            || condition
                .each_node(&[])
                .iter()
                .any(|candidate| candidate.type_group_is("any_match_pattern"))
        {
            continue;
        }

        if if_unless_modifier_single_line_as_modifier(
            node,
            &statement_modifier,
            context,
            &source,
        ) {
            let replacement = statement_modifier.to_modifier_form(node);
            let Some(node_range) = node.source_range() else {
                continue;
            };
            let message = MODIFIER_MESSAGE.replace("{keyword}", keyword);
            context.replace(
                message,
                if_unless_modifier_character_range_to_byte(&source, keyword_range),
                if_unless_modifier_character_range_to_byte(&source, node_range),
                replacement,
            );
        } else if if_unless_modifier_too_long(node, context, &processed, max_line_length) {
            if_unless_modifier_report_long(
                node,
                keyword,
                keyword_range,
                context,
                &processed,
                &source,
            );
        }
    }
}

fn if_unless_modifier_has_keyword(source: &str) -> bool {
    ["if", "unless"].into_iter().any(|keyword| {
        source.match_indices(keyword).any(|(at, _)| {
            let before = source[..at].chars().next_back();
            let after = source[at + keyword.len()..].chars().next();
            before.is_none_or(|character| !character.is_ascii_alphanumeric() && character != '_')
                && after.is_some_and(|character| character.is_whitespace() || character == '(')
        })
    })
}

fn if_unless_modifier_endless_method(body: Option<RubocopNodeRef<'_>>) -> bool {
    body.is_some_and(|body| matches!(body.kind(), "def" | "defs") && body.endless())
}

fn if_unless_modifier_defined_argument_is_undefined(
    if_node: RubocopNodeRef<'_>,
    condition: RubocopNodeRef<'_>,
) -> bool {
    condition
        .each_node(&["defined?"])
        .into_iter()
        .filter_map(|defined| defined.first_argument())
        .filter(|argument| matches!(argument.kind(), "lvar" | "send"))
        .any(|argument| {
            let name = if argument.kind() == "lvar" {
                argument.symbol_child(0)
            } else if argument.receiver().is_none() {
                argument.method_name()
            } else {
                None
            };
            name.is_none_or(|name| {
                if_node.left_siblings().iter().all(|sibling| {
                    sibling.kind() != "lvasgn" || sibling.symbol_child(0) != Some(name)
                })
            })
        })
}

fn if_unless_modifier_single_line_as_modifier(
    node: RubocopNodeRef<'_>,
    statement_modifier: &StatementModifier<'_, '_>,
    context: &CompatibilityCopContext<'_, '_, '_>,
    source: &str,
) -> bool {
    if node.ternary()
        || node.elsif()
        || node.has_else()
        || node.chained()
        || node.nested_conditional()
        || if_unless_modifier_multiline_inside_collection(node)
        || node
            .condition()
            .is_some_and(|condition| {
                condition
                    .each_node(&["match_with_lvasgn"])
                    .into_iter()
                    .next()
                    .is_some()
                    || condition.method_name() == Some("=~")
                        && condition
                            .receiver()
                            .is_some_and(|receiver| matches!(receiver.kind(), "regexp" | "regopt"))
            })
    {
        return false;
    }
    if !statement_modifier.single_line_as_modifier(node) {
        return false;
    }
    let max = context
        .related_config_value("Layout/LineLength", "Max")
        .and_then(|value| value.parse::<usize>().ok());
    let Some(max) = max else {
        return true;
    };
    let Some(keyword) = node.loc("keyword") else {
        return false;
    };
    let line_start = source[..if_unless_modifier_character_position_to_byte(source, keyword.0.start)]
        .rfind('\n')
        .map_or(0, |at| at + 1);
    let prefix_end = if_unless_modifier_character_position_to_byte(source, keyword.0.start);
    let node_end = node
        .source_range()
        .map(|range| if_unless_modifier_character_position_to_byte(source, range.end))
        .unwrap_or(prefix_end);
    let line_end = source[node_end..]
        .find('\n')
        .map_or(source.len(), |at| node_end + at);
    let candidate = format!(
        "{}{}{}",
        &source[line_start..prefix_end],
        statement_modifier.to_modifier_form(node),
        &source[node_end..line_end]
    );
    let tab_width = context
        .related_config_value("Layout/IndentationWidth", "Width")
        .or_else(|| context.related_config_value("Layout/IndentationStyle", "IndentationWidth"))
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(2);
    candidate
        .chars()
        .map(|character| if character == '\t' { tab_width } else { 1 })
        .sum::<usize>()
        <= max
}

fn if_unless_modifier_multiline_inside_collection(node: RubocopNodeRef<'_>) -> bool {
    if node.modifier_form() {
        return false;
    }
    let Some(collection) = if_unless_modifier_containing_collection(node) else {
        return false;
    };
    collection.child_nodes().into_iter().any(|child| {
        let inner = if child.kind() == "pair" {
            child.node_child(1).unwrap_or(child)
        } else {
            child
        };
        let inner = if inner.kind() == "begin" {
            inner.node_child(0).unwrap_or(inner)
        } else {
            inner
        };
        inner.kind() == "if"
            && !inner.ternary()
            && (inner.first_line() == node.last_line() || inner.last_line() == node.first_line())
    })
}

fn if_unless_modifier_containing_collection(
    node: RubocopNodeRef<'_>,
) -> Option<RubocopNodeRef<'_>> {
    let parent = node.parent()?;
    let ancestor = if parent.kind() == "begin" {
        parent.parent()?
    } else {
        parent
    };
    if matches!(ancestor.kind(), "array" | "send" | "csend") {
        Some(ancestor)
    } else if ancestor.kind() == "pair" {
        ancestor.parent()
    } else {
        None
    }
}

fn if_unless_modifier_too_long(
    node: RubocopNodeRef<'_>,
    context: &CompatibilityCopContext<'_, '_, '_>,
    processed: &ProcessedSource<'_>,
    max_line_length: Option<usize>,
) -> bool {
    let Some(max) = max_line_length else {
        return false;
    };
    if !node.modifier_form()
        || !node.single_line()
        || context.related_config_value("Layout/LineLength", "Enabled") == Some("false")
        || context.related_config_value("AllCops", "DisabledByDefault") == Some("true")
            && !context.related_config_explicit("Layout/LineLength", "Enabled")
    {
        return false;
    }
    let Some(line) = processed.line(node.first_line().saturating_sub(1)) else {
        return false;
    };
    if line_length_disabled_at(processed.raw_source(), node.source_range().map_or(0, |r| r.start))
        || line.contains("rubocop:disable Layout/LineLength")
        || if_unless_modifier_another_statement_same_line(node)
        || line.chars().count() <= max
        || if_unless_modifier_allowed_pattern(context, line)
    {
        return false;
    }
    if line.contains("rubocop:")
        && context.related_config_value("Layout/LineLength", "AllowCopDirectives") != Some("false")
        && processed
            .comments()
            .iter()
            .find(|comment| comment.line == node.first_line())
            .and_then(|comment| line.find(&comment.text))
            .map_or(line, |comment_at| &line[..comment_at])
            .trim_end()
            .chars()
            .count()
            <= max
    {
        return false;
    }
    if context.related_config_value("Layout/LineLength", "AllowURI") != Some("false")
        && if_unless_modifier_uri_is_allowed_excess(line, max)
    {
        return false;
    }
    true
}

fn if_unless_modifier_another_statement_same_line(node: RubocopNodeRef<'_>) -> bool {
    let line = node.last_line();
    let mut child = node;
    while let Some(parent) = child.parent() {
        if parent.kind() == "begin" {
            return child
                .right_sibling()
                .is_some_and(|sibling| sibling.first_line() == line);
        }
        child = parent;
    }
    false
}

fn if_unless_modifier_uri_is_allowed_excess(line: &str, max: usize) -> bool {
    let uri = regex::Regex::new(r"(?:http|https|ftp)://[^\s<>]+").unwrap();
    let Some(found) = uri.find_iter(line).last() else {
        return false;
    };
    let start = line[..found.start()].chars().count();
    let end = if line.ends_with('}') && line.contains('{') {
        line.chars().count()
    } else {
        line[..found.end()].chars().count()
    };
    start < max && end == line.chars().count()
}

fn if_unless_modifier_allowed_pattern(context: &CompatibilityCopContext<'_, '_, '_>, line: &str) -> bool {
    ["AllowedPatterns", "IgnoredPatterns"]
        .into_iter()
        .filter_map(|key| context.related_config_value("Layout/LineLength", key))
        .flat_map(str::lines)
        .any(|pattern| !pattern.is_empty() && regex::Regex::new(pattern).is_ok_and(|re| re.is_match(line)))
}

fn if_unless_modifier_report_long(
    node: RubocopNodeRef<'_>,
    keyword: &str,
    keyword_range: std::ops::Range<usize>,
    context: &mut CompatibilityCopContext<'_, '_, '_>,
    processed: &ProcessedSource<'_>,
    source: &str,
) {
    let Some(node_range) = node.source_range() else {
        return;
    };
    let Some(condition) = node.condition().and_then(RubocopNodeRef::source) else {
        return;
    };
    let Some(body) = node.body().and_then(RubocopNodeRef::source) else {
        return;
    };
    let indentation = " ".repeat(node.column());
    let message = format!("Modifier form of `{keyword}` makes the line too long.");
    let offense_range = if_unless_modifier_character_range_to_byte(source, keyword_range);
    let correction_range = if_unless_modifier_character_range_to_byte(source, node_range.clone());

    if if_unless_modifier_another_modifier_same_line(node)
        || node
            .ancestors()
            .iter()
            .any(|ancestor| ancestor.kind() == "if" && ancestor.modifier_form())
    {
        context.report(message, offense_range);
        return;
    }

    if let Some(comment) = processed
        .comments()
        .iter()
        .find(|comment| comment.line == node.first_line())
    {
        let line = processed.line(node.first_line().saturating_sub(1)).unwrap_or_default();
        let max = context
            .related_config_value("Layout/LineLength", "Max")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(120);
        if max >= line.chars().count().saturating_sub(comment.range.end - comment.range.start) {
            let comment_range = if_unless_modifier_character_range_to_byte(source, comment.range.clone());
            let remove_start = comment_range.start.saturating_sub(
                usize::from(source.as_bytes().get(comment_range.start.saturating_sub(1)) == Some(&b' ')),
            );
            let replacement = format!("{}\n{indentation}{body} {keyword} {condition}", comment.text);
            context.replace_many(
                message,
                offense_range,
                vec![
                    (correction_range, replacement),
                    (remove_start..comment_range.end, String::new()),
                ],
            );
            return;
        }
    }

    if let Some((heredoc_range, replacement)) =
        if_unless_modifier_heredoc_replacement(node, keyword, condition, body, source)
    {
        context.replace(message, offense_range, heredoc_range, replacement);
        return;
    }

    let replacement = format!("{keyword} {condition}\n{indentation}  {body}\n{indentation}end");
    context.replace(message, offense_range, correction_range, replacement);
}

fn if_unless_modifier_another_modifier_same_line(node: RubocopNodeRef<'_>) -> bool {
    let Some(collection) = if_unless_modifier_containing_collection(node) else {
        return false;
    };
    collection
        .each_descendant(&["if"])
        .into_iter()
        .filter(|sibling| {
            sibling.modifier_form() && sibling.first_line() == node.first_line()
        })
        .count()
        > 1
}

fn if_unless_modifier_heredoc_replacement(
    node: RubocopNodeRef<'_>,
    keyword: &str,
    condition: &str,
    body: &str,
    source: &str,
) -> Option<(std::ops::Range<usize>, String)> {
    let marker = body.find("<<")?;
    let marker_tail = body[marker + 2..]
        .trim_start_matches(['~', '-', '\'', '"', '`']);
    let label = marker_tail
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .next()?;
    if label.is_empty() {
        return None;
    }
    let character_range = node.source_range()?;
    let byte_start = if_unless_modifier_character_position_to_byte(source, character_range.start);
    let byte_node_end = if_unless_modifier_character_position_to_byte(source, character_range.end);
    let first_line_end = source[byte_node_end..]
        .find('\n')
        .map_or(byte_node_end, |at| byte_node_end + at);
    let mut cursor = first_line_end + usize::from(source.as_bytes().get(first_line_end) == Some(&b'\n'));
    let mut heredoc_lines = Vec::new();
    let mut replacement_end = None;
    for line in source[cursor..].split_inclusive('\n') {
        let content = line.strip_suffix('\n').unwrap_or(line);
        heredoc_lines.push(content.to_owned());
        cursor += line.len();
        if content.trim() == label {
            replacement_end = Some(cursor.saturating_sub(usize::from(line.ends_with('\n'))));
            break;
        }
    }
    let replacement_end = replacement_end?;
    let indentation = " ".repeat(node.column());
    let mut replacement = format!("{keyword} {condition}\n{indentation}  {body}");
    for line in heredoc_lines {
        replacement.push('\n');
        replacement.push_str(&indentation);
        replacement.push_str("  ");
        replacement.push_str(&line);
    }
    replacement.push('\n');
    replacement.push_str(&indentation);
    replacement.push_str("end");
    Some((byte_start..replacement_end, replacement))
}

fn line_length_disabled_at(source: &str, character_offset: usize) -> bool {
    let byte_offset = source
        .char_indices()
        .nth(character_offset)
        .map_or(source.len(), |(byte, _)| byte);
    let target_line = source[..byte_offset].bytes().filter(|byte| *byte == b'\n').count();
    let mut disabled = false;
    for (line_index, line) in source.lines().enumerate().take(target_line + 1) {
        let Some(comment_at) = line.find('#') else {
            continue;
        };
        let comment = &line[comment_at..];
        let standalone = line[..comment_at].trim().is_empty();
        if comment.contains("rubocop:disable Layout/LineLength") {
            if standalone {
                disabled = true;
            } else if line_index == target_line {
                return true;
            }
        }
        if comment.contains("rubocop:enable Layout/LineLength") && standalone {
            disabled = false;
        }
    }
    disabled
}

fn if_unless_modifier_character_range_to_byte(
    source: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let start = source
        .char_indices()
        .nth(range.start)
        .map_or(source.len(), |(byte, _)| byte);
    let end = source
        .char_indices()
        .nth(range.end)
        .map_or(source.len(), |(byte, _)| byte);
    start..end
}

fn if_unless_modifier_character_position_to_byte(source: &str, position: usize) -> usize {
    source
        .char_indices()
        .nth(position)
        .map_or(source.len(), |(byte, _)| byte)
}
