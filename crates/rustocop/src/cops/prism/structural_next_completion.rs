use super::*;

mod elsif_conversion;
use elsif_conversion::*;

define_cops! {
    IfInsideElse => "Style/IfInsideElse" => source(if_inside_else),
    MultilineTernaryOperator => "Style/MultilineTernaryOperator" => recovery_rubocop_callbacks(
        MultilineTernaryOperatorRule,
        [on_if]
    ),
    CaseLikeIf => "Style/CaseLikeIf" => source(case_like_if),
}

fn case_like_if(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let minimum = context
        .config_value("MinBranchesCount")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(3);
    let mut literal_ranges = context.source_file().literal_ranges();
    literal_ranges.extend(context.source_file().heredoc_ranges());
    let mut reported = Vec::new();
    let mut index = 0usize;
    while index < lines.len() {
        let (start_offset, first_line) = lines[index];
        let first = first_line.trim_start();
        let first_content = start_offset + first_line.len() - first.len();
        if literal_ranges
            .iter()
            .any(|range| range.start <= first_content && first_content < range.end)
        {
            index += 1;
            continue;
        }
        let Some(condition) = first.strip_prefix("if ") else {
            index += 1;
            continue;
        };
        if let Some((left, right)) = condition.split_once(" =~ ") {
            if !source_regexp(left.trim()) && !source_regexp(right.trim()) {
                index += 1;
                continue;
            }
        }
        if let Some((receiver, argument)) = condition.split_once(".match?(") {
            if !receiver.trim().starts_with('/')
                && !argument.trim_end_matches(')').trim().starts_with('/')
            {
                index += 1;
                continue;
            }
        }
        let Some((subject, value)) = case_comparison(condition) else {
            index += 1;
            continue;
        };
        let indent = &first_line[..first_line.len() - first.len()];
        let mut branches = vec![(index, value)];
        let mut end_index = None;
        let mut has_conditional_else = false;
        let mut cursor = index + 1;
        while cursor < lines.len() {
            let raw_line = lines[cursor].1;
            let line = raw_line.trim_start();
            let line_indent = &raw_line[..raw_line.len() - line.len()];
            if line_indent != indent {
                cursor += 1;
                continue;
            }
            if let Some(condition) = line.strip_prefix("elsif ") {
                let Some((candidate, value)) = case_comparison(condition) else {
                    break;
                };
                if candidate != subject {
                    break;
                }
                branches.push((cursor, value));
            } else if line == "else" || line.starts_with("else ") {
                has_conditional_else = lines
                    .get(cursor + 1)
                    .map(|(_, body)| body.trim())
                    .is_some_and(|body| {
                        body.contains(" ? ") && body.contains(" : ") && !body.contains(" = ")
                    });
            } else if line == "end" || line.starts_with("end ") {
                end_index = Some(cursor);
                break;
            }
            cursor += 1;
        }
        let Some(end_index) = end_index else {
            index += 1;
            continue;
        };
        if branches.len() + usize::from(has_conditional_else) < minimum {
            index += 1;
            continue;
        }
        let mut edits = Vec::new();
        for (branch, value) in &branches {
            let (offset, line) = lines[*branch];
            let replacement = if *branch == index {
                format!("{indent}case {subject}\n{indent}when {value}")
            } else {
                format!("{indent}when {value}")
            };
            edits.push((offset..offset + line.len(), replacement));
        }
        let end_line = lines[end_index].1;
        let end = lines[end_index].0 + end_line.len() - end_line.trim_start().len() + 3;
        let offense = start_offset + indent.len()..end;
        reported.push(offense.clone());
        context.replace_many(
            "Convert `if-elsif` to `case-when`.",
            offense,
            edits,
        );
        index += 1;
    }
    ast_case_like_if(context, minimum, &reported);
}

fn ast_case_like_if(
    context: &mut CopContext<'_, '_>,
    minimum: usize,
    reported: &[std::ops::Range<usize>],
) {
    struct Candidate {
        offense: std::ops::Range<usize>,
        edits: Vec<(std::ops::Range<usize>, String)>,
    }

    struct Finder<'source> {
        file: SourceFile<'source>,
        minimum: usize,
        reported: &'source [std::ops::Range<usize>],
        candidates: Vec<Candidate>,
    }

    impl<'pr> ruby_prism::Visit<'pr> for Finder<'_> {
        fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
            let Some(keyword) = node.if_keyword_loc() else {
                ruby_prism::visit_if_node(self, node);
                return;
            };
            if keyword.as_slice() != b"if" || node.end_keyword_loc().is_none() {
                ruby_prism::visit_if_node(self, node);
                return;
            }

            let offense = node.location().start_offset()..node.location().end_offset();
            if self.reported.contains(&offense) {
                ruby_prism::visit_if_node(self, node);
                return;
            }
            let mut branches = Vec::new();
            let first_condition = self.file.node(&node.predicate());
            if !case_condition_supported(first_condition) {
                ruby_prism::visit_if_node(self, node);
                return;
            }
            let Some((target, value)) = case_comparison(first_condition) else {
                ruby_prism::visit_if_node(self, node);
                return;
            };
            let keyword_start = keyword.start_offset();
            branches.push((keyword, node.predicate().location(), value));
            let mut subsequent = node.subsequent();
            let mut has_conditional_else = false;
            while let Some(branch) = subsequent {
                let Some(elsif) = branch.as_if_node() else {
                    if let Some(else_node) = branch.as_else_node() {
                        has_conditional_else = else_node
                            .statements()
                            .is_some_and(|statements| {
                                let body = self.file.node(&statements.as_node());
                                body.contains(" ? ")
                                    && body.contains(" : ")
                                    && !body.contains(" = ")
                            });
                    }
                    break;
                };
                let Some(elsif_keyword) = elsif.if_keyword_loc() else { break };
                let Some((candidate, value)) = case_comparison(self.file.node(&elsif.predicate()))
                else {
                    branches.clear();
                    break;
                };
                if candidate != target {
                    branches.clear();
                    break;
                }
                branches.push((elsif_keyword, elsif.predicate().location(), value));
                subsequent = elsif.subsequent();
            }
            if branches.len() + usize::from(has_conditional_else) >= self.minimum {
                let indent = " ".repeat(keyword_start - self.file.line_start(keyword_start));
                let mut edits = Vec::new();
                for (index, (branch_keyword, predicate, value)) in branches.into_iter().enumerate() {
                    let replacement = if index == 0 {
                        format!("case {target}\n{indent}when {value}")
                    } else {
                        format!("when {value}")
                    };
                    edits.push((branch_keyword.start_offset()..predicate.end_offset(), replacement));
                }
                candidates_push(
                    &mut self.candidates,
                    offense,
                    edits,
                );
            }
            ruby_prism::visit_if_node(self, node);
        }
    }

    fn candidates_push(
        candidates: &mut Vec<Candidate>,
        offense: std::ops::Range<usize>,
        edits: Vec<(std::ops::Range<usize>, String)>,
    ) {
        candidates.push(Candidate { offense, edits });
    }

    let parsed = ruby_prism::parse(context.source().as_bytes());
    let mut finder = Finder {
        file: context.source_file(),
        minimum,
        reported,
        candidates: Vec::new(),
    };
    finder.visit(&parsed.node());
    for candidate in finder.candidates {
        context.replace_many(
            "Convert `if-elsif` to `case-when`.",
            candidate.offense,
            candidate.edits,
        );
    }
}

fn source_regexp(source: &str) -> bool {
    source.starts_with('/') || source.starts_with("%r")
}

fn case_condition_supported(condition: &str) -> bool {
    if let Some((left, right)) = condition.split_once(" =~ ") {
        return source_regexp(left.trim())
            || source_regexp(right.trim())
            || (!left.contains('(') && !right.contains('('));
    }
    if let Some((receiver, argument)) = condition.split_once(".match?(") {
        let argument = argument.trim_end_matches(')').trim();
        return source_regexp(receiver.trim())
            || source_regexp(argument)
            || (!receiver.contains('(') && !argument.contains('('));
    }
    true
}

fn case_comparison(condition: &str) -> Option<(String, String)> {
    let trailing_comment = condition
        .find('#')
        .map(|start| condition[start..].trim().to_string());
    let condition = condition.split('#').next().unwrap_or(condition).trim();
    if condition.contains(" && ")
        || condition.contains(" and ")
        || condition.contains(" or ")
    {
        return None;
    }
    let condition = if condition.contains(" == ") {
        condition.trim_matches(['(', ')']).trim()
    } else {
        condition
    };
    if condition.contains(" || ") {
        let comparisons = condition
            .split(" || ")
            .map(case_comparison)
            .collect::<Option<Vec<_>>>()?;
        let subject = comparisons.first()?.0.clone();
        if comparisons.iter().any(|comparison| comparison.0 != subject) {
            return None;
        }
        return Some((
            subject,
            comparisons
                .into_iter()
                .map(|comparison| comparison.1)
                .collect::<Vec<_>>()
                .join(", "),
        ));
    }
    if let Some((value, subject)) = condition.split_once(" === ") {
        return Some((subject.trim().to_string(), value.trim().to_string()));
    }
    if let Some((subject, value)) = condition.split_once(" == ") {
        let subject = subject.trim();
        let value = value.trim();
        let terminal_constant = value.rsplit("::").next().unwrap_or(value);
        let constant_like = terminal_constant
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_uppercase);
        if constant_like
            && (terminal_constant.len() == 1
                || (!case_literal(value)
                    && terminal_constant.bytes().any(|byte| byte.is_ascii_lowercase())))
        {
            return None;
        }
        if case_literal(subject) && !case_literal(value) {
            return Some((value.to_string(), subject.to_string()));
        }
        if !case_literal(value) {
            return None;
        }
        let value = trailing_comment
            .map_or_else(|| value.to_string(), |comment| format!("{value} {comment}"));
        return Some((subject.to_string(), value));
    }
    if let Some((subject, class)) = condition.split_once(".is_a?(") {
        return Some((
            subject.trim().to_string(),
            class.trim_end_matches(')').trim().to_string(),
        ));
    }
    if let Some((subject, class)) = condition.split_once(".is_a? ") {
        return Some((subject.trim().to_string(), class.trim().to_string()));
    }
    if let Some((receiver, argument)) = condition.split_once(".match?(") {
        let receiver = receiver.trim();
        let argument = argument.trim_end_matches(')').trim();
        return if receiver.starts_with('/') {
            Some((argument.to_string(), receiver.to_string()))
        } else if argument.starts_with('/') {
            Some((receiver.to_string(), argument.to_string()))
        } else if argument
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        {
            Some((receiver.to_string(), argument.to_string()))
        } else {
            Some((argument.to_string(), receiver.to_string()))
        };
    }
    if let Some((range, argument)) = condition.split_once(".include?(") {
        if range.contains("..") {
            return Some((
                argument.trim_end_matches(')').trim().to_string(),
                range.trim().trim_matches(['(', ')']).to_string(),
            ));
        }
    }
    if let Some((left, right)) = condition.split_once(" =~ ") {
        let left = left.trim();
        let right = right.trim();
        if left.contains("(?<") {
            return None;
        }
        return if left.starts_with('/') {
            Some((right.to_string(), left.to_string()))
        } else {
            Some((left.to_string(), right.to_string()))
        };
    }
    None
}

fn case_literal(value: &str) -> bool {
    let terminal_constant = value.rsplit("::").next().unwrap_or(value);
    value.starts_with(['\'', '"', ':', '/', '[', '{'])
        || value
            .trim_start_matches('-')
            .as_bytes()
            .first()
            .is_some_and(u8::is_ascii_digit)
        || matches!(value, "nil" | "true" | "false")
        || value.contains("::")
            && terminal_constant
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
        || value
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit() || byte == b'_')
}

impl MultilineTernaryOperatorRule<'_, '_, '_> {
    #[allow(clippy::too_many_lines)]
    fn on_if(&mut self, node: &ruby_prism::IfNode<'_>) {
        if node.if_keyword_loc().is_some()
            || node
                .then_keyword_loc()
                .is_none_or(|keyword| keyword.as_slice() != b"?")
        {
            return;
        }
        let location = node.location();
        let source = self.source_file().at(&location);
        if !source.contains('\n') || !source.contains('?') {
            return;
        }
        let Some(question_location) = node.then_keyword_loc() else {
            return;
        };
        let Some(colon_location) = node
            .subsequent()
            .and_then(|branch| branch.as_else_node())
            .map(|branch| branch.else_keyword_loc())
        else {
            return;
        };
        let file = self.source_file();
        let predicate_location = node.predicate().location();
        let predicate_is_multiline_equality = node.predicate().as_call_node().is_some_and(|call| {
            call.name().as_slice() == b"=="
                && !file.same_line(
                    predicate_location.start_offset(),
                    predicate_location.end_offset().saturating_sub(1),
                )
        });
        let truthy_location = node
            .statements()
            .and_then(|statements| statements.body().first())
            .map(|statement| statement.location());
        let falsey_location = node
            .subsequent()
            .and_then(|branch| branch.as_else_node())
            .and_then(|branch| branch.statements())
            .and_then(|statements| statements.body().first())
            .map(|statement| statement.location());
        if file.same_line(
            question_location.start_offset(),
            colon_location.start_offset(),
        ) && truthy_location.as_ref().is_some_and(|truthy| {
            file.same_line(question_location.start_offset(), truthy.start_offset())
                && file.same_line(
                    truthy.start_offset(),
                    truthy.end_offset().saturating_sub(1),
                )
        }) && falsey_location.as_ref().is_some_and(|falsey| {
            file.same_line(colon_location.start_offset(), falsey.start_offset())
                && file.same_line(
                    falsey.start_offset(),
                    falsey.end_offset().saturating_sub(1),
                )
        }) && !predicate_is_multiline_equality {
            return;
        }
        let question = question_location.start_offset() - location.start_offset();
        let colon = colon_location.start_offset() - location.start_offset();
        if !(source[question..].contains('\n')
            || source[..question].contains('\n') && source[..question].contains("=="))
        {
            return;
        }
        let condition = source[..question].trim();
        let truthy = source[question + 1..colon].trim();
        let falsey = source[colon + 1..].trim();
        if condition.is_empty() || truthy.is_empty() || falsey.is_empty() {
            return;
        }
        let single_line = self.parent().is_some_and(|parent| {
            parent.as_return_node().is_some()
                || parent.as_break_node().is_some()
                || parent.as_next_node().is_some()
                || parent
                    .as_call_node()
                    .is_some_and(|call| !call_name(&call).ends_with(b"="))
        });
        let comments = ternary_comments(source);
        let mut replacement = if single_line {
            format!("{condition} ? {truthy} : {falsey}")
        } else {
            multiline_ternary_ast_replacement(node, self.source_file())
                .unwrap_or_else(|| format!("if {condition}\n  {truthy}\nelse\n  {falsey}\nend"))
        };
        if !comments.is_empty() {
            if let Some(clean_replacement) =
                ternary_ast_replacement(node, self.source_file(), single_line)
            {
                replacement = clean_replacement;
            }
        }
        let message = if single_line {
            "Avoid multi-line ternary operators, use single-line instead."
        } else {
            "Avoid multi-line ternary operators, use `if` or `unless` instead."
        };
        let nested = self.ancestors().iter().any(|ancestor| {
            ancestor
                .as_if_node()
                .is_some_and(|ancestor| self.source_file().at(&ancestor.location()).contains('?'))
        });
        if nested {
            self.replace_indirectly(message, &location, &location, replacement);
        } else {
            let parent_start = self
                .ancestors()
                .iter()
                .rev()
                .find(|ancestor| ancestor.as_statements_node().is_none())
                .map_or(location.start_offset(), |parent| {
                    parent.location().start_offset()
                });
            let edit = location.start_offset()..location.end_offset();
            add_offense!(self, edit.clone(), message: message, |corrector| {
                corrector.replace(edit, replacement);
                if !comments.is_empty() {
                    corrector.replace(parent_start..parent_start, comments);
                }
            });
        }
    }
}

fn multiline_ternary_ast_replacement(
    node: &ruby_prism::IfNode<'_>,
    file: SourceFile<'_>,
) -> Option<String> {
    let condition = file.node(&node.predicate());
    let truthy = only_statement(node.statements())?;
    let else_node = node.subsequent()?.as_else_node()?;
    let falsey = only_statement(else_node.statements())?;
    let truthy = file.node(&truthy);
    let falsey_source = file.node(&falsey);
    let falsey = falsey
        .as_if_node()
        .filter(|nested| {
            nested
                .then_keyword_loc()
                .is_some_and(|keyword| keyword.as_slice() == b"?")
                && falsey_source.contains('\n')
        })
        .and_then(|nested| multiline_ternary_ast_replacement(&nested, file))
        .unwrap_or_else(|| falsey_source.to_string());
    Some(format!(
        "if {condition}\n  {truthy}\nelse\n  {falsey}\nend"
    ))
}

fn ternary_comments(source: &str) -> String {
    source
        .lines()
        .filter_map(|line| line.find('#').map(|index| &line[index..]))
        .map(|comment| format!("{comment}\n"))
        .collect()
}

fn ternary_ast_replacement(
    node: &ruby_prism::IfNode<'_>,
    file: SourceFile<'_>,
    single_line: bool,
) -> Option<String> {
    let condition = file.node(&node.predicate());
    let truthy = only_statement(node.statements())?;
    let else_node = node.subsequent()?.as_else_node()?;
    let falsey = only_statement(else_node.statements())?;
    let truthy = file.node(&truthy);
    let falsey = file.node(&falsey);
    Some(if single_line {
        format!("{condition} ? {truthy} : {falsey}")
    } else {
        format!("if {condition}\n  {truthy}\nelse\n  {falsey}\nend")
    })
}
