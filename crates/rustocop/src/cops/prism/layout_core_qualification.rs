use super::*;
use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;
use unicode_width::UnicodeWidthChar;

define_compatibility_rule!(DotPositionRule);
define_compatibility_rule!(EmptyLineBetweenDefsCompatibilityRule);

define_cops! {
    BlockAlignment => "Layout/BlockAlignment" => compatibility_prism_any_node(block_alignment_node),
    DotPosition => "Layout/DotPosition" => compatibility_callbacks(DotPositionRule, [on_send]),
    EmptyLineBetweenDefs => "Layout/EmptyLineBetweenDefs" => compatibility_callbacks(EmptyLineBetweenDefsCompatibilityRule, [on_begin]),
    EmptyLinesAfterModuleInclusion => "Layout/EmptyLinesAfterModuleInclusion" => compatibility_prism_call(empty_lines_after_module_inclusion),
    EmptyLinesAroundAccessModifier => "Layout/EmptyLinesAroundAccessModifier" => compatibility_prism_call(empty_lines_around_access_modifier),
    FirstArgumentIndentation => "Layout/FirstArgumentIndentation" => compatibility_prism_any_node(first_argument_indentation),
}

impl EmptyLineBetweenDefsCompatibilityRule<'_, '_, '_, '_> {
    fn on_begin(&mut self, node: RubocopNodeRef<'_>) {
        for nodes in node.child_nodes().windows(2) {
            if self.candidate(nodes[0]) && self.candidate(nodes[1]) {
                self.check_defs(nodes[0], nodes[1]);
            }
        }
    }

    fn candidate(&self, node: RubocopNodeRef<'_>) -> bool {
        match node.kind() {
            "def" | "defs" => self.config_bool("EmptyLineBetweenMethodDefs", true),
            "class" => self.config_bool("EmptyLineBetweenClassDefs", true),
            "module" => self.config_bool("EmptyLineBetweenModuleDefs", true),
            "send" => self.macro_candidate(node),
            "block" | "numblock" | "itblock" => node.send_node().is_some_and(|send| self.macro_candidate(send)),
            _ => false,
        }
    }

    fn macro_candidate(&self, node: RubocopNodeRef<'_>) -> bool {
        node.receiver().is_none() && node.method_name().is_some_and(|name| self.config_values("DefLikeMacros").iter().any(|configured| configured == name))
    }

    fn check_defs(&mut self, previous: RubocopNodeRef<'_>, current: RubocopNodeRef<'_>) {
        let lines = self.lines_between_defs(previous, current);
        let count = lines.iter().filter(|line| line.trim().is_empty()).count();
        let (minimum, maximum) = self.empty_line_limits();
        return_if!((minimum..=maximum).contains(&count));
        let blank_start = lines.iter().rposition(|line| line.trim().is_empty());
        let non_blank_end = lines.iter().position(|line| !line.trim().is_empty());
        return_if!(blank_start.zip(non_blank_end).is_some_and(|(blank, code)| blank > code));
        return_if!(previous.single_line() && current.single_line() && self.config_bool("AllowAdjacentOneLineDefs", true));

        let expected = if minimum == maximum {
            format!("{maximum} empty {}", if maximum == 1 { "line" } else { "lines" })
        } else {
            format!("{minimum}..{maximum} empty lines")
        };
        let kind = match current.kind() {
            "def" | "defs" => "method",
            "numblock" | "itblock" => "block",
            other => other,
        };
        let message = format!("Expected {expected} between {kind} definitions; found {count}.");
        let Some(offense) = self.def_location(current) else { return; };
        let Some(current_range) = current.source_range() else { return; };
        let newline_pos = if previous.last_line() == current.first_line() {
            current_range.start.saturating_sub(1)
        } else {
            self.source_buffer().line_range(previous.last_line()).end
        };
        let offense = self.owned_range(offense);
        let removal = (count > maximum).then(|| self.owned_character_range(newline_pos..newline_pos + (count - maximum)));
        let insertion = (count <= maximum).then(|| {
            self.owned_range(crate::rubocop::ast::source::SourceRange::new(
                self.source_buffer(), newline_pos, (newline_pos + 1).min(self.source_buffer().len())
            ))
        });
        add_offense!(self, offense, message: message, |corrector| {
            if let Some(removal) = removal { corrector.remove(removal); }
            if let Some(insertion) = insertion { corrector.insert_after(insertion, "\n".repeat(minimum - count)); }
        });
    }

    fn lines_between_defs(&self, previous: RubocopNodeRef<'_>, current: RubocopNodeRef<'_>) -> Vec<&str> {
        let start = previous.last_line();
        let end = current.first_line().saturating_sub(1);
        if start >= end { Vec::new() } else { self.processed_source().lines()[start..end].iter().map(String::as_str).collect() }
    }

    fn empty_line_limits(&self) -> (usize, usize) {
        let values = self.config_values("NumberOfEmptyLines");
        if values.is_empty() {
            let value = self.config_usize("NumberOfEmptyLines", 1);
            (value, value)
        } else {
            (values.first().and_then(|value| value.parse().ok()).unwrap_or(1), values.last().and_then(|value| value.parse().ok()).unwrap_or(1))
        }
    }

    fn def_location(&self, node: RubocopNodeRef<'_>) -> Option<crate::rubocop::ast::source::SourceRange<'_, '_>> {
        if matches!(node.kind(), "block" | "numblock" | "itblock") {
            self.source_range(node).zip(node.child_nodes().first().and_then(|child| self.source_range(*child))).map(|(node, child)| node.join(child))
        } else if node.send_type() {
            self.source_range(node)
        } else {
            let keyword = node.loc("keyword")?.0.clone();
            let end = node.loc("name").map(|location| location.0.end)
                .or_else(|| node.node_child(0).and_then(|child| child.source_range()).map(|range| range.end))?;
            Some(crate::rubocop::ast::source::SourceRange::new(self.source_buffer(), keyword.start, end))
        }
    }
}

fn block_alignment_node(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(block) = node.as_block_node() {
        block_alignment(&block, context);
    } else if let Some(lambda) = node.as_lambda_node() {
        lambda_alignment(&lambda, context);
    }
}

impl DotPositionRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: RubocopNodeRef<'_>) {
        let (Some(receiver), Some((dot, operator))) = (node.receiver(), node.loc("dot")) else {
            return;
        };
        let Some(selector) = self.selector_range(node) else { return; };
        return_if!(self.proper_dot_position(receiver, selector, dot));

        let style = self.policy().enforced_style("leading").to_string();
        let message = if style == "leading" {
            format!("Place the {operator} on the next line, together with the method name.")
        } else {
            format!(
                "Place the {operator} on the previous line, together with the method call receiver."
            )
        };
        let dot_range = crate::rubocop::ast::source::SourceRange::new(self.source_buffer(), dot.start, dot.end);
        let removal = if self.processed_source().line(dot_range.line().saturating_sub(1)).is_some_and(|line| line.trim() == ".") {
            self.owned_range(self.range_help().range_by_whole_lines(dot_range, true))
        } else {
            self.owned_range(dot_range)
        };
        let selector = self.owned_range(selector);
        let offense = self.owned_character_range(dot.clone());
        add_offense!(self, offense, message: message, |corrector| {
            corrector.remove(removal);
            if style == "leading" { corrector.insert_before(selector, operator); }
            else { corrector.insert_after(receiver, operator); }
        });
    }

    fn proper_dot_position(
        &self,
        receiver: RubocopNodeRef<'_>,
        selector: crate::rubocop::ast::source::SourceRange<'_, '_>,
        dot: &std::ops::Range<usize>,
    ) -> bool {
        if selector.line() == receiver.last_line() { return true; }
        let receiver_end_line = self.receiver_end_line(receiver);
        let dot_line = crate::rubocop::ast::source::SourceRange::new(self.source_buffer(), dot.start, dot.end).line();
        if selector.line().saturating_sub(receiver_end_line.max(dot_line)) > 1 { return true; }
        match self.policy().enforced_style("leading") {
            "leading" => dot_line == selector.line(),
            "trailing" => dot_line != selector.line(),
            _ => true,
        }
    }

    fn receiver_end_line(&self, node: RubocopNodeRef<'_>) -> usize {
        self.last_heredoc_line(node).unwrap_or(node.last_line())
    }

    fn last_heredoc_line(&self, node: RubocopNodeRef<'_>) -> Option<usize> {
        if node.call_type() {
            node.arguments().into_iter().filter(|arg| self.heredoc(*arg))
                .filter_map(|arg| arg.loc("heredoc_end").map(|(range, _)| crate::rubocop::ast::source::SourceRange::new(self.source_buffer(), range.start, range.end).line()))
                .max()
        } else if self.heredoc(node) {
            node.loc("heredoc_end").map(|(range, _)| crate::rubocop::ast::source::SourceRange::new(self.source_buffer(), range.start, range.end).line())
        } else { None }
    }

    fn heredoc(&self, node: RubocopNodeRef<'_>) -> bool {
        matches!(node.kind(), "str" | "dstr" | "xstr") && node.heredoc()
    }

    fn selector_range(&self, node: RubocopNodeRef<'_>) -> Option<crate::rubocop::ast::source::SourceRange<'_, '_>> {
        if node.call_type() {
            node.loc("selector").or_else(|| node.loc("begin"))
                .map(|(range, _)| crate::rubocop::ast::source::SourceRange::new(self.source_buffer(), range.start, range.end))
        } else { self.source_range(node) }
    }
}

fn empty_lines_after_module_inclusion(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.receiver().is_some()
        || node
            .arguments()
            .is_none_or(|arguments| arguments.arguments().is_empty())
        || !matches!(node.name().as_slice(), b"include" | b"extend" | b"prepend")
    {
        return;
    }
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
            || ancestor.as_singleton_class_node().is_some()
            || ancestor.as_program_node().is_some()
        {
            break;
        }
        if ancestor.as_call_node().is_some() || ancestor.as_array_node().is_some() {
            return;
        }
        if let Some(block) = ancestor.as_block_node() {
            if block
                .body()
                .and_then(|body| body.as_statements_node())
                .is_none_or(|statements| statements.body().len() == 1)
            {
                return;
            }
            break;
        }
        let modifier_conditional = ancestor
            .as_if_node()
            .and_then(|conditional| conditional.if_keyword_loc())
            .is_some_and(|keyword| {
                line_index(context.source(), keyword.start_offset())
                    == line_index(context.source(), node.location().start_offset())
            })
            || ancestor.as_unless_node().is_some_and(|conditional| {
                line_index(context.source(), conditional.keyword_loc().start_offset())
                    == line_index(context.source(), node.location().start_offset())
            });
        if modifier_conditional {
            return;
        }
    }

    let source = context.source();
    let current_line = line_index(source, node.location().end_offset());
    let line_end = line_start(source, current_line + 1).min(source.len());
    if source[node.location().end_offset()..line_end]
        .split([';', ' '])
        .any(|token| token.trim() == "end")
    {
        return;
    }
    if line(source, current_line + 1).trim().is_empty()
        || is_enable_directive(line(source, current_line + 1))
            && line(source, current_line + 2).trim().is_empty()
    {
        return;
    }
    let Some(next) = next_code_line(source, current_line + 1) else {
        return;
    };
    let follower = line(source, next).trim_start();
    let follower_call = call_name(follower);
    let closes_scope = follower.strip_prefix("end").is_some_and(|rest| {
        rest.chars()
            .next()
            .is_none_or(|character| !character.is_alphanumeric() && character != '_')
    }) || follower.starts_with('}');
    if closes_scope
        || matches!(follower, "when" | "else" | "ensure" | "rescue")
        || [")", "]", "elsif ", "when "]
            .iter()
            .any(|prefix| follower.starts_with(prefix))
        || follower.starts_with("rescue ")
            && (0..current_line)
                .rev()
                .map(|line_number| line(source, line_number))
                .find(|candidate| {
                    !candidate.trim().is_empty() && !candidate.trim_start().starts_with('#')
                })
                .is_some_and(|previous| {
                    previous.len() - previous.trim_start().len()
                        == line(source, current_line).len()
                            - line(source, current_line).trim_start().len()
                })
        || ["include", "extend", "prepend"].iter().any(|method| {
            follower_call == *method
                || follower_call.ends_with(&format!(".{method}"))
                || follower_call.ends_with(&format!("&.{method}"))
        })
    {
        return;
    }

    let mut insertion_line = current_line + 1;
    if is_enable_directive(line(source, insertion_line)) {
        insertion_line += 1;
    }
    context.insert(
        "Add an empty line after module inclusion.",
        node.location(),
        line_start(source, insertion_line),
        "\n",
    );
}

#[allow(clippy::too_many_lines)]
fn empty_lines_around_access_modifier(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.receiver().is_some()
        || node.arguments().is_some()
        || node.block().is_some()
        || !matches!(
            node.name().as_slice(),
            b"public" | b"protected" | b"private" | b"module_function"
        )
    {
        return;
    }
    if context
        .ancestors()
        .iter()
        .rev()
        .find(|ancestor| {
            ancestor.as_def_node().is_some()
                || ancestor.as_class_node().is_some()
                || ancestor.as_module_node().is_some()
                || ancestor.as_singleton_class_node().is_some()
                || ancestor.as_block_node().is_some()
        })
        .is_some_and(|ancestor| ancestor.as_def_node().is_some())
    {
        return;
    }
    let inside_block = context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_block_node().is_some());
    let assigned_block = context.ancestors().iter().any(|ancestor| {
        ancestor.as_local_variable_write_node().is_some()
            || ancestor.as_instance_variable_write_node().is_some()
            || ancestor.as_class_variable_write_node().is_some()
            || ancestor.as_global_variable_write_node().is_some()
            || ancestor.as_constant_write_node().is_some()
            || ancestor.as_constant_path_write_node().is_some()
    });
    let class_constructor_block = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_block_node)
        .is_some_and(|block| {
            context.ancestors().iter().rev().any(|ancestor| {
                let Some(call) = ancestor.as_call_node() else {
                    return false;
                };
                let same_block = call.block().is_some_and(|call_block| {
                    call_block.location().start_offset() == block.location().start_offset()
                        && call_block.location().end_offset() == block.location().end_offset()
                });
                same_block
                    && call.name().as_slice() == b"new"
                    && call.receiver().is_some_and(|receiver| {
                        matches!(
                            context
                                .source_file()
                                .node(&receiver)
                                .trim_start_matches("::"),
                            "Class" | "Module" | "Struct"
                        )
                    })
            })
        });
    let inside_condition = context.ancestors().iter().any(|ancestor| {
        let predicate = if let Some(condition) = ancestor.as_if_node() {
            Some(condition.predicate())
        } else {
            ancestor
                .as_unless_node()
                .map(|condition| condition.predicate())
        };
        predicate.is_some_and(|predicate| {
            predicate.location().start_offset() <= node.location().start_offset()
                && node.location().end_offset() <= predicate.location().end_offset()
        })
    });
    if inside_condition || inside_block && assigned_block && !class_constructor_block {
        return;
    }
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_def_node().is_some()
            || ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
            || ancestor.as_singleton_class_node().is_some()
            || ancestor.as_block_node().is_some()
        {
            break;
        }
        if ancestor.as_call_node().is_some() {
            return;
        }
    }
    let source = context.source();
    let location = node.location();
    let current_line = line_index(source, location.start_offset());
    if previous_non_comment_line(source, current_line).is_some_and(|previous| {
        line(source, previous).trim_end().ends_with(',')
            || line(source, previous).len() - line(source, previous).trim_start().len()
                > line(source, current_line).len() - line(source, current_line).trim_start().len()
    }) {
        return;
    }
    if !source[location.end_offset()..line_end(source, current_line)]
        .split('#')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_start_matches(';')
        .trim()
        .is_empty()
    {
        return;
    }

    let bounds = enclosing_body_bounds(context);
    let before_ok = bounds.is_some_and(|(opening, _, _)| current_line == opening + 1)
        || previous_non_comment_line(source, current_line)
            .is_none_or(|previous| line(source, previous).trim().is_empty());
    let mut after_ok = bounds
        .is_some_and(|(_, closing, is_block)| !is_block && current_line + 1 == closing)
        || line(source, current_line + 1).trim().is_empty();
    if after_ok
        && bounds.is_some_and(|(_, closing, _)| current_line + 1 == closing)
        && source
            .lines()
            .skip(current_line + 2)
            .find(|line| !line.trim().is_empty())
            .is_some_and(|line| line.trim_start().starts_with("# == Schema Information"))
    {
        after_ok = false;
    }
    let style = context.policy().enforced_style("around").to_string();
    if style == "around" && before_ok && after_ok {
        return;
    }
    if style == "only_before" {
        let special_modifier = matches!(node.name().as_slice(), b"private" | b"protected");
        let next_line_exists = current_line + 1 < source.lines().count();
        if special_modifier {
            if line(source, current_line + 1).trim() == "end"
                || before_ok && (!after_ok || !next_line_exists)
            {
                return;
            }
        } else if before_ok {
            return;
        }
    }

    let modifier = String::from_utf8_lossy(node.name().as_slice());
    let message = if style == "around" {
        if bounds.is_some_and(|(opening, _, _)| current_line == opening + 1) {
            format!("Keep a blank line after `{modifier}`.")
        } else {
            format!("Keep a blank line before and after `{modifier}`.")
        }
    } else if after_ok {
        format!("Remove a blank line after `{modifier}`.")
    } else {
        format!("Keep a blank line before `{modifier}`.")
    };

    let mut edits = Vec::new();
    let denied_block_end = bounds.is_some_and(|(_, closing, is_block)| {
        is_block
            && current_line + 1 == closing
            && context.related_config_value("Layout/EmptyLinesAroundBlockBody", "EnforcedStyle")
                == Some("no_empty_lines")
    });
    if !before_ok {
        let start = line_start(source, current_line);
        edits.push((start..start, "\n".to_string()));
    }
    if style == "around" && !after_ok && !denied_block_end {
        let start = line_start(source, current_line + 1);
        edits.push((start..start, "\n".to_string()));
    } else if style == "only_before"
        && after_ok
        && bounds.is_none_or(|(_, closing, is_block)| is_block || current_line + 1 != closing)
    {
        edits.push((
            line_start(source, current_line + 1)..line_start(source, current_line + 2),
            String::new(),
        ));
    }
    if edits.is_empty() {
        context.report(message, &location);
    } else {
        context.replace_many(message, &location, edits);
    }
}

fn lambda_alignment(node: &ruby_prism::LambdaNode<'_>, context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let file = context.source_file();
    let closing = node.closing_loc();
    let closing_line = line_index(source, closing.start_offset());
    let closing_start = line_start(source, closing_line);
    if !source[closing_start..closing.start_offset()]
        .chars()
        .all(char::is_whitespace)
    {
        return;
    }

    let lambda_start = node.location().start_offset();
    let lambda_line = line_index(source, lambda_start);
    let line_begin = line_start(source, lambda_line);
    let before_lambda = &source[line_begin..lambda_start];
    let assigned = before_lambda
        .rfind('=')
        .is_some_and(|equal| before_lambda.as_bytes().get(equal + 1) != Some(&b'>'));
    let start_offset = if assigned {
        line_begin + before_lambda.len() - before_lambda.trim_start().len()
    } else {
        lambda_start
    };
    let start_column = file.column(start_offset);

    let body_start = node.body().map_or(closing.start_offset(), |body| {
        body.location().start_offset()
    });
    let brace = source[lambda_start..body_start]
        .rfind('{')
        .map_or(lambda_start, |relative| lambda_start + relative);
    let brace_line = line_index(source, brace);
    let brace_line_start = line_start(source, brace_line);
    let brace_column = source[brace_line_start..brace]
        .chars()
        .take_while(|character| character.is_whitespace())
        .count();
    let current_column = file.column(closing.start_offset());
    let style = context
        .config_value("EnforcedStyleAlignWith")
        .unwrap_or("either");
    let aligned = match style {
        "start_of_block" => current_column == brace_column,
        "start_of_line" => current_column == start_column,
        _ => current_column == start_column || current_column == brace_column,
    };
    if aligned {
        return;
    }

    let current = format!("`}}` at {}, {current_column}", closing_line + 1);
    let start = source_line_column(source, lambda_line, start_column, start_offset);
    let block = source_line_column(
        source,
        brace_line,
        brace_column,
        brace_line_start + brace_column,
    );
    let preferred = if style == "start_of_block" {
        &block
    } else {
        &start
    };
    let alternate =
        if style == "either" && (lambda_line != brace_line || start_column != brace_column) {
            format!(" or {block}")
        } else {
            String::new()
        };
    let parenthesized_argument = before_lambda.trim_end().ends_with('(');
    if !assigned && !parenthesized_argument {
        context.report(
            format!("{current} is not aligned with {preferred}{alternate}."),
            &closing,
        );
        return;
    }
    let target = if style == "start_of_block" {
        brace_column
    } else if parenthesized_argument {
        source[line_begin..lambda_start].len() - source[line_begin..lambda_start].trim_start().len()
    } else {
        start_column
    };
    context.replace(
        format!("{current} is not aligned with {preferred}{alternate}."),
        &closing,
        closing_start..closing.start_offset(),
        " ".repeat(target),
    );
}

fn block_alignment(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let opening = node.opening_loc();
    let opening_line = context.line_index(opening.start_offset());
    let block_column = context
        .source_file()
        .indentation(opening.start_offset())
        .len();
    let block_start = context.line_start_at(opening_line) + block_column;
    block_alignment_locations(
        node.closing_loc(),
        opening,
        block_start,
        block_column,
        false,
        context,
    );
}

fn block_alignment_locations(
    closing: ruby_prism::Location<'_>,
    opening: ruby_prism::Location<'_>,
    block_start: usize,
    block_column: usize,
    prefer_block: bool,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    let closing_line = context.line_index(closing.start_offset());
    let closing_start = context.line_start_at(closing_line);
    if !context.source()[closing_start..closing.start_offset()]
        .chars()
        .all(char::is_whitespace)
    {
        return;
    }

    let ancestors = context.ancestors();
    let Some(call_index) = ancestors
        .iter()
        .rposition(|ancestor| ancestor.as_call_node().is_some())
    else {
        return;
    };
    let mut target = ancestors[call_index].location();
    for parent in ancestors[..call_index].iter().rev() {
        if parent.as_arguments_node().is_some() {
            continue;
        }
        if parent.as_statements_node().is_some() || parent.as_arguments_node().is_some() {
            continue;
        }
        let parent_line = context.line_index(parent.location().start_offset());
        let target_line = context.line_index(target.start_offset());
        let mass_assignment = parent.as_multi_write_node().is_some();
        if parent_line != target_line && !mass_assignment {
            break;
        }
        let absorbs_receiver = parent.as_call_node().is_some_and(|call| {
            call.name().as_slice() == b"<<"
                || call.receiver().is_some_and(|receiver| {
                    call.name().as_slice() != b"[]"
                        && receiver.location().start_offset() == target.start_offset()
                        && receiver.location().end_offset() == target.end_offset()
                })
        });
        let prefix = context
            .source()
            .get(parent.location().start_offset()..target.start_offset())
            .unwrap_or_default();
        let absorbs_expression = parent.as_def_node().is_some()
            || parent.as_splat_node().is_some()
            || mass_assignment
            || parent.as_and_node().is_some()
            || parent.as_or_node().is_some()
            || prefix.contains('=') && !prefix.contains("=>");
        if absorbs_receiver || absorbs_expression {
            target = parent.location();
        } else {
            break;
        }
    }
    let start_line = context.line_index(target.start_offset());
    let start_offset = target.start_offset();
    let start_column = file.column(start_offset);
    let opening_line = context.line_index(opening.start_offset());
    let current_column = file.column(closing.start_offset());
    let style = context
        .config_value("EnforcedStyleAlignWith")
        .unwrap_or("either");
    let aligned = match style {
        "start_of_block" => current_column == block_column,
        "start_of_line" => current_column == start_column,
        _ => current_column == start_column || current_column == block_column,
    };
    if aligned {
        return;
    }

    let current = format!(
        "`{}` at {}, {current_column}",
        String::from_utf8_lossy(closing.as_slice()),
        closing_line + 1
    );
    let start = assignment_lhs_target(
        context.source(),
        start_line,
        start_column,
        start_offset,
        target_is_mass_assignment(context.source(), target),
    )
    .unwrap_or_else(|| {
        source_line_column(context.source(), start_line, start_column, start_offset)
    });
    let block = source_line_column(context.source(), opening_line, block_column, block_start);
    let preferred = if style == "start_of_block" || style == "either" && prefer_block {
        &block
    } else {
        &start
    };
    let alternate =
        if style == "either" && (start_line != opening_line || start_column != block_column) {
            if prefer_block {
                format!(" or {start}")
            } else {
                format!(" or {block}")
            }
        } else {
            String::new()
        };
    let target = if style == "start_of_block" {
        block_column
    } else {
        line(context.source(), start_line).len()
            - line(context.source(), start_line).trim_start().len()
    };
    context.replace(
        format!("{current} is not aligned with {preferred}{alternate}."),
        &closing,
        closing_start..closing.start_offset(),
        " ".repeat(target),
    );
}

fn source_line_column(source: &str, line_number: usize, column: usize, start: usize) -> String {
    let content = &source[start.min(line_end(source, line_number))..line_end(source, line_number)];
    format!("`{}` at {}, {column}", content.trim_end(), line_number + 1)
}

fn target_is_mass_assignment(source: &str, target: ruby_prism::Location<'_>) -> bool {
    source[target.start_offset()..line_end(source, line_index(source, target.start_offset()))]
        .split_once(" = ")
        .is_some_and(|(left, _)| left.contains(','))
}

fn assignment_lhs_target(
    source: &str,
    line_number: usize,
    column: usize,
    start: usize,
    mass_assignment: bool,
) -> Option<String> {
    let content = &source[start..line_end(source, line_number)];
    let mut operators = vec![" += ", " -= ", " *= ", " /= ", " %= "];
    if mass_assignment {
        operators.push(" = ");
    }
    let operator = operators
        .iter()
        .filter_map(|operator| content.find(operator))
        .min()?;
    Some(format!(
        "`{}` at {}, {column}",
        content[..operator].trim_end(),
        line_number + 1
    ))
}

#[allow(clippy::too_many_lines)]
fn first_argument_indentation(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (first, call_start, special_eligible, call_node) = if let Some(call) = node.as_call_node() {
        let Some(first) = call
            .arguments()
            .and_then(|arguments| arguments.arguments().first())
        else {
            return;
        };
        let name = call.name().as_slice();
        if name == b"[]"
            || name == b"=~"
            || name.ends_with(b"=")
            || call.call_operator_loc().is_none() && is_operator_name(name)
        {
            return;
        }
        (first, call.location().start_offset(), true, Some(call))
    } else if let Some(call) = node.as_super_node() {
        let Some(first) = call
            .arguments()
            .and_then(|arguments| arguments.arguments().first())
        else {
            return;
        };
        (first, call.keyword_loc().start_offset(), false, None)
    } else {
        return;
    };

    let source = context.source();
    let argument_start = first.location().start_offset();
    if context.line_index(call_start) == context.line_index(argument_start) {
        return;
    }
    let argument_line = context.line_index(argument_start);
    if !source[context.line_start_at(argument_line)..argument_start]
        .chars()
        .all(char::is_whitespace)
    {
        return;
    }
    if context.related_config_value("Layout/ArgumentAlignment", "EnforcedStyle")
        == Some("with_fixed_indentation")
        && context.related_config_value("Layout/FirstMethodArgumentLineBreak", "Enabled")
            != Some("true")
    {
        return;
    }

    let style = context
        .policy()
        .enforced_style("special_for_inner_method_call_in_parentheses")
        .to_string();
    let semantic_parent = context
        .ancestors()
        .iter()
        .rev()
        .find(|ancestor| ancestor.as_arguments_node().is_none());
    let inside_interpolation = context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_interpolated_string_node().is_some())
        || source[..call_start]
            .rfind("#{")
            .is_some_and(|opening| !source[opening + 2..call_start].contains('}'));
    let outer = (!inside_interpolation)
        .then(|| semantic_parent.and_then(|parent| parent.as_call_node()))
        .flatten();
    let special_indentation = style == "consistent_relative_to_receiver"
        || special_eligible
            && style != "consistent"
            && outer.as_ref().is_some_and(|parent| {
                let permitted = style != "special_for_inner_method_call_in_parentheses"
                    || parent.opening_loc().is_some();
                permitted
                    && parent.name().as_slice() != b"[]="
                    && call_start > parent.location().start_offset()
            });
    let width = context
        .config_value("IndentationWidth")
        .and_then(|value| value.parse().ok())
        .or_else(|| {
            context
                .related_config_value("Layout/IndentationWidth", "Width")
                .and_then(|value| value.parse().ok())
        })
        .unwrap_or(2);
    let previous_line = previous_code_line_in(context, argument_line);
    let base_start = semantic_parent
        .filter(|parent| parent.as_splat_node().is_some() || parent.as_assoc_splat_node().is_some())
        .map_or(call_start, |parent| parent.location().start_offset());
    let base_source = source[base_start..argument_start].trim();
    let base = if inside_interpolation {
        let selector_start = call_node
            .as_ref()
            .and_then(|call| call.message_loc())
            .map_or(call_start, |selector| selector.start_offset());
        let selector_line = context.line_index(selector_start);
        context.line_at(selector_line).len() - context.line_at(selector_line).trim_start().len()
    } else if special_indentation {
        if base_source.contains('\n') {
            previous_line
                .map(|number| {
                    context.line_at(number).len() - context.line_at(number).trim_start().len()
                })
                .unwrap_or(0)
        } else {
            display_column_at(context, base_start)
        }
    } else {
        previous_line
            .map(|number| {
                context.line_at(number).len() - context.line_at(number).trim_start().len()
            })
            .unwrap_or(0)
    };
    let expected = base + width;
    let actual = context.source_file().column(argument_start);
    if actual == expected {
        return;
    }
    let correction_overlaps_outer = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_call_node)
        .and_then(|parent| {
            parent
                .arguments()
                .and_then(|arguments| arguments.arguments().first())
                .map(|outer_first| (parent, outer_first))
        })
        .is_some_and(|(parent, outer_first)| {
            let outer_location = outer_first.location();
            let contains = outer_location.start_offset() <= first.location().start_offset()
                && first.location().end_offset() <= outer_location.end_offset()
                && (outer_location.start_offset() != first.location().start_offset()
                    || outer_location.end_offset() != first.location().end_offset());
            if !contains {
                return false;
            }
            let outer_line = context.line_index(outer_location.start_offset());
            if context.line_index(parent.location().start_offset()) == outer_line
                || !source[context.line_start_at(outer_line)..outer_location.start_offset()]
                    .chars()
                    .all(char::is_whitespace)
            {
                return false;
            }
            let outer_base = if style == "consistent_relative_to_receiver" {
                context
                    .source_file()
                    .column(parent.location().start_offset())
            } else {
                previous_code_line_in(context, outer_line)
                    .map(|number| {
                        context.line_at(number).len() - context.line_at(number).trim_start().len()
                    })
                    .unwrap_or(0)
            };
            context.source_file().column(outer_location.start_offset()) != outer_base + width
        });
    if correction_overlaps_outer {
        if !context.autocorrect_enabled() {
            context.report("Bad indentation of the first argument.", first.location());
        }
        return;
    }

    let base_description = if special_indentation && !base_source.contains('\n') {
        format!("`{base_source}`")
    } else if base_source
        .lines()
        .next_back()
        .is_some_and(|line| line.trim_start().starts_with('#'))
    {
        "the start of the previous line (not counting the comment)".to_string()
    } else {
        "the start of the previous line".to_string()
    };
    let message = format!("Indent the first argument one step more than {base_description}.");
    let delta = expected as isize - actual as isize;
    let first_location = first.location();
    let first_line = context.line_index(first_location.start_offset());
    let mut correction_end = first_location.end_offset();
    let inside_parenthesized_argument = call_node.as_ref().is_some_and(|call| {
        context
            .ancestors()
            .iter()
            .filter_map(Node::as_call_node)
            .any(|parent| {
                parent.opening_loc().is_some()
                    && parent.arguments().is_some_and(|arguments| {
                        arguments.location().start_offset() <= call.location().start_offset()
                            && call.location().end_offset() <= arguments.location().end_offset()
                    })
            })
    });
    if style == "special_for_inner_method_call_in_parentheses" && inside_parenthesized_argument {
        if let Some(call) = call_node.as_ref() {
            correction_end = call.location().end_offset();
            for ancestor in context.ancestors().iter().rev() {
                if ancestor.as_call_node().is_some_and(|ancestor| {
                    ancestor.location().start_offset() == call.location().start_offset()
                }) {
                    correction_end = ancestor.location().end_offset();
                }
            }
        }
    }
    let last_line = context.line_index(correction_end);
    let mut previous = None::<(usize, bool)>;
    let edits = (first_line..=last_line)
        .filter_map(|number| {
            let start = context.line_start_at(number);
            let content = context.line_at(number);
            if content.trim().is_empty() {
                return None;
            }
            if delta > 0
                && inside_parenthesized_argument
                && number > first_line
                && content.trim_start().starts_with(')')
            {
                return None;
            }
            let indentation = content.len() - content.trim_start().len();
            let preserve_nested = delta > 0
                && previous.is_some_and(|(previous_indent, opened)| {
                    opened && indentation == previous_indent + width * 2
                });
            let adjusted = if preserve_nested {
                indentation
            } else {
                (indentation as isize + delta).max(0) as usize
            };
            previous = Some((indentation, content.trim_end().ends_with('(')));
            Some((start..start + indentation, " ".repeat(adjusted)))
        })
        .collect();
    context.replace_many(message, &first_location, edits);
}

fn display_column_at(context: &CopContext<'_, '_>, offset: usize) -> usize {
    context.source()[context.line_start_at(context.line_index(offset))..offset]
        .chars()
        .map(|character| character.width().unwrap_or(0))
        .sum()
}

fn is_operator_name(name: &[u8]) -> bool {
    matches!(
        name,
        b"+" | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
            | b"=="
            | b"!="
            | b"<"
            | b">"
            | b"<="
            | b">="
            | b"<=>"
            | b"&"
            | b"|"
            | b"^"
            | b"<<"
            | b">>"
    )
}

fn enclosing_body_bounds(context: &CopContext<'_, '_>) -> Option<(usize, usize, bool)> {
    context.ancestors().iter().rev().find_map(|ancestor| {
        if let Some(class) = ancestor.as_class_node() {
            let opening = class.superclass().map_or_else(
                || line_index(context.source(), class.class_keyword_loc().start_offset()),
                |superclass| line_index(context.source(), superclass.location().end_offset()),
            );
            return Some((
                opening,
                line_index(context.source(), class.end_keyword_loc().start_offset()),
                false,
            ));
        }
        if let Some(module) = ancestor.as_module_node() {
            return Some((
                line_index(context.source(), module.module_keyword_loc().start_offset()),
                line_index(context.source(), module.end_keyword_loc().start_offset()),
                false,
            ));
        }
        if let Some(class) = ancestor.as_singleton_class_node() {
            return Some((
                line_index(context.source(), class.expression().location().end_offset()),
                line_index(context.source(), class.end_keyword_loc().start_offset()),
                false,
            ));
        }
        ancestor.as_block_node().map(|block| {
            (
                line_index(context.source(), block.opening_loc().start_offset()),
                line_index(context.source(), block.closing_loc().start_offset()),
                true,
            )
        })
    })
}

fn previous_non_comment_line(source: &str, line_number: usize) -> Option<usize> {
    (0..line_number)
        .rev()
        .find(|number| !line(source, *number).trim_start().starts_with('#'))
}

fn previous_code_line_in(context: &CopContext<'_, '_>, line_number: usize) -> Option<usize> {
    (0..line_number).rev().find(|number| {
        let candidate = context.line_at(*number).trim();
        !candidate.is_empty() && !candidate.starts_with('#')
    })
}

fn call_name(source: &str) -> &str {
    source
        .split(|character: char| character.is_whitespace() || character == '(')
        .next()
        .unwrap_or_default()
}

fn is_enable_directive(source: &str) -> bool {
    let source = source.trim();
    source.starts_with("# rubocop:enable") || source.starts_with("# rubocop:todo")
}

fn next_code_line(source: &str, mut line_number: usize) -> Option<usize> {
    while line_start(source, line_number) < source.len() {
        let candidate = line(source, line_number).trim_start();
        if !candidate.is_empty() && !candidate.starts_with('#') {
            return Some(line_number);
        }
        line_number += 1;
    }
    None
}

fn line_index(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
}

fn line_start(source: &str, line_number: usize) -> usize {
    if line_number == 0 {
        return 0;
    }
    source
        .match_indices('\n')
        .nth(line_number - 1)
        .map_or(source.len(), |(offset, _)| offset + 1)
}

fn line_end(source: &str, line_number: usize) -> usize {
    let start = line_start(source, line_number);
    source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset)
}

fn line(source: &str, line_number: usize) -> &str {
    let start = line_start(source, line_number);
    let end = line_end(source, line_number);
    source[start..end]
        .strip_suffix('\r')
        .unwrap_or(&source[start..end])
}
