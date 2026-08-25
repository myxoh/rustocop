use ruby_prism::{Location, Node, UntilNode, WhileNode};

use super::*;

define_cops! {
    InfiniteLoop => "Style/InfiniteLoop" => rubocop_callbacks(InfiniteLoopRule, [on_while, on_until]),
}

impl InfiniteLoopRule<'_, '_, '_> {
    fn on_while(&mut self, node: &WhileNode<'_>) {
        let predicate = node.predicate();
        return_unless!(truthy_literal(&predicate));
        self.check_loop(
            node.location(),
            node.keyword_loc(),
            node.do_keyword_loc(),
            node.closing_loc(),
            predicate,
        );
    }

    fn on_until(&mut self, node: &UntilNode<'_>) {
        let predicate = node.predicate();
        return_unless!(predicate.as_false_node().is_some() || predicate.as_nil_node().is_some());
        self.check_loop(
            node.location(),
            node.keyword_loc(),
            node.do_keyword_loc(),
            node.closing_loc(),
            predicate,
        );
    }

    fn check_loop(
        &mut self,
        location: Location<'_>,
        keyword: Location<'_>,
        do_keyword: Option<Location<'_>>,
        closing: Option<Location<'_>>,
        predicate: Node<'_>,
    ) {
        let range = location.start_offset()..location.end_offset();
        return_if!(changes_local_scope(self.source(), range.clone()));
        let modifier = keyword.start_offset() > location.start_offset();
        let source = self.source_file().slice(range.clone()).unwrap_or_default();
        let post_condition = modifier && source.trim_start().starts_with("begin");
        let correction = if post_condition {
            let mut suffix_start = keyword.start_offset();
            while suffix_start > location.start_offset()
                && matches!(self.source().as_bytes()[suffix_start - 1], b' ' | b'\t')
            {
                suffix_start -= 1;
            }
            LoopCorrection::PostCondition {
                begin: location.start_offset()..location.start_offset() + "begin".len(),
                suffix: suffix_start..predicate.location().end_offset(),
            }
        } else if modifier {
            let body = self.source()[location.start_offset()..keyword.start_offset()].trim_end();
            let replacement = if body.contains('\n') {
                let width = self
                    .related_config_value("Layout/IndentationWidth", "Width")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(2);
                let indent = " ".repeat(width);
                let body = body
                    .lines()
                    .map(|line| format!("{indent}{line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("loop do\n{body}\nend")
            } else {
                format!("loop {{ {body} }}")
            };
            LoopCorrection::Replace(range, replacement)
        } else {
            let header_end = do_keyword
                .map_or(predicate.location().end_offset(), |location| location.end_offset());
            LoopCorrection::Replace(keyword.start_offset()..header_end, "loop do".to_string())
        };
        let _ = closing;
        add_offense!(self, &keyword, message: "Use `Kernel#loop` for infinite loops.", |corrector| {
            match correction {
                LoopCorrection::Replace(range, replacement) => corrector.replace(range, replacement),
                LoopCorrection::PostCondition { begin, suffix } => {
                    corrector.replace(begin, "loop do");
                    corrector.remove(suffix);
                }
            }
        });
    }
}

enum LoopCorrection {
    Replace(std::ops::Range<usize>, String),
    PostCondition {
        begin: std::ops::Range<usize>,
        suffix: std::ops::Range<usize>,
    },
}

fn truthy_literal(node: &Node<'_>) -> bool {
    node.as_true_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_regular_expression_node().is_some()
}

fn changes_local_scope(source: &str, loop_range: std::ops::Range<usize>) -> bool {
    let loop_source = &source[loop_range.clone()];
    let file = SourceFile::new(source);
    let scope_start = file
        .lines()
        .take_while(|(offset, _)| *offset < loop_range.start)
        .filter(|(_, line)| line.trim_start().starts_with("def "))
        .map(|(offset, _)| offset)
        .last()
        .unwrap_or(0);
    let definition_line = source[scope_start..]
        .lines()
        .next()
        .unwrap_or_default();
    let definition_indent = definition_line.len() - definition_line.trim_start().len();
    let scope_end = file
        .lines()
        .skip_while(|(offset, _)| *offset < loop_range.end)
        .find_map(|(offset, line)| {
            let trimmed = line.trim_start();
            let indentation = line.len() - trimmed.len();
            (trimmed == "end" && indentation == definition_indent).then_some(offset)
        })
        .unwrap_or(source.len());
    let before = &source[scope_start..loop_range.start];
    let after = &source[loop_range.end..scope_end];
    assigned_local_names(loop_source).into_iter().any(|name| {
        !contains_assignment(before, &name) && contains_word(after, &name)
    })
}

fn assigned_local_names(source: &str) -> Vec<String> {
    let mut names = Vec::new();
    let mut block_depth = 0_usize;
    for line in source.lines() {
        let code = line.split('#').next().unwrap_or_default();
        let trimmed = code.trim();
        if trimmed == "end" && block_depth > 0 {
            block_depth -= 1;
            continue;
        }
        let opens_block = trimmed.ends_with(" do")
            || trimmed.contains(" do |")
            || trimmed.contains(" do|");
        if block_depth > 0 {
            block_depth += usize::from(opens_block);
            continue;
        }
        if opens_block {
            block_depth = 1;
            continue;
        }
        let Some((left, right)) = code.split_once('=') else {
            continue;
        };
        if right.starts_with('=') || left.trim_end().ends_with(['!', '<', '>', '=']) {
            continue;
        }
        names.extend(left.split(',').filter_map(assignment_name));
    }
    names
}

fn contains_assignment(source: &str, name: &str) -> bool {
    source.lines().any(|line| {
        line.split_once('=').is_some_and(|(left, right)| {
            !right.starts_with('=')
                && (left.split(',').filter_map(assignment_name).any(|candidate| candidate == name)
                    || right
                        .split_once('=')
                        .and_then(|(nested, _)| assignment_name(nested))
                        .is_some_and(|candidate| candidate == name))
        })
    })
}

fn contains_word(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(offset, _)| {
        let before = source[..offset].chars().next_back();
        let after = source[offset + name.len()..].chars().next();
        !before.is_some_and(|character| {
            character.is_ascii_alphanumeric() || matches!(character, '_' | '.' | ':')
        }) && !after.is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
    })
}

fn assignment_name(source: &str) -> Option<String> {
    let source = source.trim_end();
    let name = source
        .rsplit(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .find(|part| !part.is_empty())?;
    let prefix = &source[..source.len().saturating_sub(name.len())];
    (!prefix.ends_with(['@', '$'])
        && name
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'))
    .then(|| name.to_string())
}
