use ruby_prism::{Location, Node, StatementsNode};

use super::*;

define_rule!(NextRule);

define_cops! {
    Next => "Style/Next" => any_node_rule(NextRule, on_iteration),
}

impl NextRule<'_, '_, '_> {
    fn on_iteration(&mut self, node: &Node<'_>) {
        let body = if let Some(block) = node.as_block_node() {
            let Some(call) = self.ancestors().iter().rev().find_map(Node::as_call_node) else {
                return;
            };
            return_unless!(enumerator_method(call.name().as_slice()));
            block.body().and_then(|body| body.as_statements_node())
        } else if let Some(loop_node) = node.as_while_node() {
            loop_node.statements()
        } else if let Some(loop_node) = node.as_until_node() {
            loop_node.statements()
        } else if let Some(loop_node) = node.as_for_node() {
            loop_node.statements()
        } else {
            return;
        };
        let Some(body) = body else { return };
        self.check(&body);
    }

    fn check(&mut self, body: &StatementsNode<'_>) {
        let statements = body.body().iter().collect::<Vec<_>>();
        let Some(candidate) = statements.last() else {
            return;
        };
        let Some(condition) = next_condition(candidate) else {
            return;
        };
        return_if!(condition.has_else || condition.ternary || condition.body.is_empty());
        return_if!(condition.contains_if_else);
        return_if!(condition.body.len() == 1 && exit_statement(&condition.body[0]));
        let modifier = condition.modifier;
        let style = self.policy().enforced_style("skip_modifier_ifs");
        return_if!(modifier && style == "skip_modifier_ifs");
        let minimum = self.config_usize("MinBodyLength", 1);
        return_if!(!modifier && condition.body.len() < minimum);
        if self.config_bool("AllowConsecutiveConditionals", false) && statements.len() >= 2 {
            return_if!(next_condition(&statements[statements.len() - 2]).is_some());
        }

        if modifier {
            self.correct_modifier(&condition);
        } else {
            self.correct_block(&condition);
        }
    }

    fn correct_modifier(&mut self, condition: &NextCondition<'_>) {
        let body = &condition.body[0];
        let indent = self
            .source_file()
            .indentation_text(condition.location.start_offset());
        let replacement = format!(
            "next {} {}\n{indent}{}",
            condition.inverse_keyword,
            self.source_file().node(&condition.predicate),
            self.source_file().node(body)
        );
        let location = condition.location.start_offset()..condition.location.end_offset();
        add_offense!(self, location.clone(), message: "Use `next` to skip iteration.", |corrector| {
            corrector.replace(location, replacement);
        });
    }

    fn correct_block(&mut self, condition: &NextCondition<'_>) {
        let predicate = self.source_file().node(&condition.predicate);
        let next_code = format!("next {} {predicate}", condition.inverse_keyword);
        let condition_end = condition_header_end(condition, self.source());
        let condition_range = condition.location.start_offset()..condition_end;
        let end_keyword = condition.end_keyword.as_ref().expect("block conditional");
        let end_line_end = self.source_file().line_end(end_keyword.end_offset());
        let end_remove_end = if self.source()[end_keyword.end_offset()..end_line_end]
            .trim()
            .is_empty()
            && self.source()[end_line_end..].starts_with('\n')
        {
            end_line_end + 1
        } else {
            end_keyword.end_offset()
        };
        let end_range = self.source_file().line_start(end_keyword.start_offset())..end_remove_end;
        let offense =
            condition.location.start_offset()..condition.predicate.location().end_offset();
        let reindent = reindent_ranges(
            self.source(),
            self.source_file(),
            condition.predicate.location().start_offset(),
            condition.predicate.location().end_offset(),
            end_keyword.start_offset(),
        );
        if condition.body.iter().any(contains_plain_conditional) {
            let replacement = render_next_with_nested(
                condition,
                self.source(),
                self.source_file(),
                &next_code,
                condition_range,
                &reindent,
            );
            let edit = condition.location.start_offset()..condition.location.end_offset();
            add_offense!(self, offense, message: "Use `next` to skip iteration.", |corrector| {
                corrector.replace(edit, replacement);
            });
            return;
        }
        add_offense!(self, offense, message: "Use `next` to skip iteration.", |corrector| {
            corrector.replace(condition.location.start_offset()..condition.location.start_offset(), next_code);
            corrector.remove(condition_range);
            corrector.remove(end_range);
            for range in reindent {
                corrector.remove(range);
            }
        });
    }
}

fn contains_plain_conditional(node: &Node<'_>) -> bool {
    struct Finder(bool);
    impl<'pr> Visit<'pr> for Finder {
        fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
            if node.end_keyword_loc().is_some() && node.subsequent().is_none() {
                self.0 = true;
            }
            ruby_prism::visit_if_node(self, node);
        }

        fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
            if node.end_keyword_loc().is_some() && node.else_clause().is_none() {
                self.0 = true;
            }
            ruby_prism::visit_unless_node(self, node);
        }
    }
    let mut finder = Finder(false);
    finder.visit(node);
    finder.0
}

fn render_next_with_nested(
    condition: &NextCondition<'_>,
    source: &str,
    file: SourceFile<'_>,
    next_code: &str,
    condition_range: std::ops::Range<usize>,
    reindent: &[std::ops::Range<usize>],
) -> String {
    let base = condition.location.start_offset();
    let end_keyword = condition.end_keyword.as_ref().expect("block conditional");
    let mut rendered = source[base..condition.location.end_offset()].to_string();
    let local_end_start = file
        .line_start(end_keyword.start_offset())
        .saturating_sub(1)
        .max(base)
        - base;
    let mut edits = vec![
        (0..0, next_code.to_string()),
        (
            condition_range.start - base..condition_range.end - base,
            String::new(),
        ),
        (
            local_end_start..end_keyword.end_offset() - base,
            String::new(),
        ),
    ];
    edits.extend(reindent.iter().filter_map(|range| {
        (range.start >= base && range.end <= condition.location.end_offset())
            .then_some((range.start - base..range.end - base, String::new()))
    }));
    edits.sort_by_key(|(range, _)| range.start);
    for (range, replacement) in edits.into_iter().rev() {
        rendered.replace_range(range, &replacement);
    }
    let mut rendered = rewrite_nested_conditionals(rendered);
    if rendered.ends_with('\n') {
        rendered.pop();
    }
    rendered
}

fn rewrite_nested_conditionals(mut source: String) -> String {
    loop {
        let mut lines = source
            .split_inclusive('\n')
            .map(str::to_string)
            .collect::<Vec<_>>();
        let Some((start, keyword, condition)) =
            lines.iter().enumerate().rev().find_map(|(index, line)| {
                let trimmed = line.trim_start();
                if let Some(condition) = trimmed.strip_prefix("if ") {
                    Some((
                        index,
                        "unless",
                        condition.trim_end_matches('\n').to_string(),
                    ))
                } else {
                    trimmed.strip_prefix("unless ").map(|condition| {
                        (index, "if", condition.trim_end_matches('\n').to_string())
                    })
                }
            })
        else {
            return source;
        };
        let mut depth = 0usize;
        let mut matching_end = None;
        let mut has_else = false;
        for (index, line) in lines.iter().enumerate().skip(start + 1) {
            let trimmed = line.trim();
            if trimmed == "else" && depth == 0 {
                has_else = true;
            }
            if block_opener(trimmed) {
                depth += 1;
            } else if trimmed.starts_with("end") {
                if depth == 0 {
                    matching_end = Some(index);
                    break;
                }
                depth -= 1;
            }
        }
        let Some(end) = matching_end else {
            return source;
        };
        if has_else {
            let prefix = 2.min(lines[start].len());
            lines[start].replace_range(..prefix, "__");
            source = lines.concat();
            continue;
        }
        let indent = lines[start].len() - lines[start].trim_start().len();
        lines[start] = format!("{}next {keyword} {condition}\n", " ".repeat(indent));
        let body_indent = lines[start + 1..end]
            .iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| line.len() - line.trim_start().len())
            .min()
            .unwrap_or(indent);
        let delta = body_indent.saturating_sub(indent);
        for line in &mut lines[start + 1..end] {
            if line.as_bytes().iter().take(delta).all(|byte| *byte == b' ') {
                line.replace_range(..delta, "");
            }
        }
        lines.remove(end);
        source = lines.concat();
    }
}

fn block_opener(line: &str) -> bool {
    line.starts_with("if ")
        || line.starts_with("unless ")
        || line.starts_with("while ")
        || line.starts_with("until ")
        || line.starts_with("for ")
        || line.ends_with(" do")
        || line.contains(" do |")
}

struct NextCondition<'pr> {
    location: Location<'pr>,
    predicate: Node<'pr>,
    body: Vec<Node<'pr>>,
    end_keyword: Option<Location<'pr>>,
    inverse_keyword: &'static str,
    modifier: bool,
    ternary: bool,
    has_else: bool,
    contains_if_else: bool,
}

fn next_condition<'pr>(node: &Node<'pr>) -> Option<NextCondition<'pr>> {
    if let Some(condition) = node.as_if_node() {
        let keyword = condition.if_keyword_loc()?;
        let modifier = condition.end_keyword_loc().is_none()
            && condition.subsequent().is_none()
            && keyword.start_offset() > condition.location().start_offset();
        let body: Vec<Node<'pr>> = condition
            .statements()
            .map(|statements| statements.body().iter().collect())
            .unwrap_or_default();
        let contains_if_else = body.iter().any(contains_if_with_else);
        return Some(NextCondition {
            location: condition.location(),
            predicate: condition.predicate(),
            body,
            end_keyword: condition.end_keyword_loc(),
            inverse_keyword: "unless",
            modifier,
            ternary: keyword.as_slice() == b"?",
            has_else: condition.subsequent().is_some(),
            contains_if_else,
        });
    }
    let condition = node.as_unless_node()?;
    let keyword = condition.keyword_loc();
    let modifier = condition.end_keyword_loc().is_none()
        && condition.else_clause().is_none()
        && keyword.start_offset() > condition.location().start_offset();
    let body: Vec<Node<'pr>> = condition
        .statements()
        .map(|statements| statements.body().iter().collect())
        .unwrap_or_default();
    let contains_if_else = body.iter().any(contains_if_with_else);
    Some(NextCondition {
        location: condition.location(),
        predicate: condition.predicate(),
        body,
        end_keyword: condition.end_keyword_loc(),
        inverse_keyword: "if",
        modifier,
        ternary: false,
        has_else: condition.else_clause().is_some(),
        contains_if_else,
    })
}

fn contains_if_with_else(node: &Node<'_>) -> bool {
    struct Finder(bool);
    impl<'pr> Visit<'pr> for Finder {
        fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
            if node.subsequent().is_some() {
                self.0 = true;
            }
            ruby_prism::visit_if_node(self, node);
        }

        fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
            if node.else_clause().is_some() {
                self.0 = true;
            }
            ruby_prism::visit_unless_node(self, node);
        }
    }
    let mut finder = Finder(false);
    finder.visit(node);
    finder.0
}

fn exit_statement(node: &Node<'_>) -> bool {
    node.as_break_node().is_some() || node.as_return_node().is_some()
}

fn enumerator_method(name: &[u8]) -> bool {
    matches!(
        name,
        b"loop"
            | b"each"
            | b"each_with_object"
            | b"each_with_index"
            | b"downto"
            | b"upto"
            | b"times"
            | b"map"
            | b"collect"
            | b"select"
            | b"select!"
            | b"reject"
            | b"reject!"
    ) || name.starts_with(b"each_")
}

fn condition_header_end(condition: &NextCondition<'_>, source: &str) -> usize {
    let predicate_end = condition.predicate.location().end_offset();
    let first_body = condition
        .body
        .first()
        .map_or(condition.location.end_offset(), |body| {
            body.location().start_offset()
        });
    source[predicate_end..first_body]
        .find("then")
        .map_or(predicate_end, |offset| predicate_end + offset + 4)
}

fn reindent_ranges(
    source: &str,
    file: SourceFile<'_>,
    condition_start: usize,
    predicate_end: usize,
    end_start: usize,
) -> Vec<std::ops::Range<usize>> {
    let target_indent = file.indentation_text(condition_start).len();
    let end_line_start = file.line_start(end_start);
    let mut lines = file
        .lines()
        .filter(|(offset, line)| {
            *offset > file.line_start(predicate_end)
                && *offset < end_line_start
                && !line.trim().is_empty()
        })
        .collect::<Vec<_>>();
    let heredoc_lines = heredoc_body_offsets(&lines);
    lines.retain(|(offset, _)| !heredoc_lines.contains(offset));
    let Some(actual_indent) = lines
        .iter()
        .map(|(_, line)| line.len() - line.trim_start().len())
        .min()
    else {
        return Vec::new();
    };
    let delta = actual_indent.saturating_sub(target_indent);
    if delta == 0 {
        return Vec::new();
    }
    lines
        .into_iter()
        .filter_map(|(offset, line)| {
            let indent = line.len() - line.trim_start().len();
            (indent >= delta
                && source[offset..offset + delta]
                    .bytes()
                    .all(|byte| byte == b' '))
            .then_some(offset..offset + delta)
        })
        .collect()
}

fn heredoc_body_offsets(lines: &[(usize, &str)]) -> std::collections::HashSet<usize> {
    let mut skipped = std::collections::HashSet::new();
    let mut terminator = None::<String>;
    for (offset, line) in lines {
        if let Some(expected) = terminator.as_deref() {
            if line.trim() == expected {
                terminator = None;
            } else {
                skipped.insert(*offset);
            }
            continue;
        }
        if let Some(marker) = line.find("<<-").or_else(|| line.find("<<~")) {
            let token = line[marker + 3..]
                .trim()
                .trim_matches(['\'', '"'])
                .split_whitespace()
                .next()
                .unwrap_or("");
            if !token.is_empty() {
                terminator = Some(token.to_string());
            }
        }
    }
    skipped
}
