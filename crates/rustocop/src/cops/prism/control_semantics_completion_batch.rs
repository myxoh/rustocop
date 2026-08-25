use ruby_prism::{parse, BlockNode, ForNode, Node, Visit};
use std::collections::{HashMap, HashSet};

use super::*;

define_cops! {
    SafeNavigationConsistency => "Lint/SafeNavigationConsistency" => any_node(safe_navigation_consistency),
    CombinableDefined => "Style/CombinableDefined" => source(combinable_defined),
    For => "Style/For" => rubocop_callbacks(ForRule, [on_for, on_block]),
    ClassAndModuleChildren => "Style/ClassAndModuleChildren" => source(class_module_children),
    SafeNavigationChain => "Lint/SafeNavigationChain" => call(safe_navigation_chain),
    BlockDelimiters => "Style/BlockDelimiters" => node(as_block_node, block_delimiters),
    RedundantSafeNavigation => "Lint/RedundantSafeNavigation" => call(redundant_safe_navigation),
    AndOr => "Style/AndOr" => any_node(and_or),
}

fn safe_navigation_consistency(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_and_node().is_none() && node.as_or_node().is_none() {
        return;
    }
    if node.as_and_node().is_some()
        && context
            .ancestors()
            .iter()
            .any(|ancestor| ancestor.as_and_node().is_some())
        || node.as_or_node().is_some()
            && context
                .ancestors()
                .iter()
                .any(|ancestor| ancestor.as_or_node().is_some())
    {
        return;
    }
    if node.as_and_node().is_some() {
        if let Some(or_ancestor) = context
            .ancestors()
            .iter()
            .find(|ancestor| ancestor.as_or_node().is_some())
        {
            let after = &context.source()
                [node.location().end_offset()..or_ancestor.location().end_offset()];
            if !after.contains("&.") {
                return;
            }
        }
    }
    let mut operands = Vec::new();
    collect_navigation_operands(node, None, &mut operands);
    let mut groups: HashMap<String, Vec<NavigationOperand<'_>>> = HashMap::new();
    for operand in operands {
        let key = navigation_receiver_key(&operand.call, context.source_file());
        if !key.is_empty() {
            groups.entry(key).or_default().push(operand);
        }
    }
    for operands in groups.values() {
        let mut safe_and = None;
        let mut safe_or = None;
        let mut send_and = None;
        let mut send_or = None;
        for (index, operand) in operands.iter().enumerate() {
            let safe = navigation_is_safe(&operand.call);
            let non_nilable = !safe && !navigation_nilable(&operand.call, context);
            match (operand.logical, safe, non_nilable) {
                (LogicalKind::And, true, _) => {
                    safe_and.get_or_insert(index);
                }
                (LogicalKind::Or, true, _) => {
                    safe_or.get_or_insert(index);
                }
                (LogicalKind::And, false, true) => {
                    send_and.get_or_insert(index);
                }
                (LogicalKind::Or, false, true) => {
                    send_or.get_or_insert(index);
                }
                _ => {}
            }
        }
        if safe_and.is_some() && safe_or.is_some() && safe_and < safe_or {
            continue;
        }
        let decision = if let Some(csend) = safe_and {
            Some((".", send_and.map_or(csend, |send| send.min(csend)) + 1))
        } else if let (Some(send), Some(csend)) = (send_or, safe_or) {
            if send < csend {
                Some((".", send + 1))
            } else {
                Some(("&.", csend + 1))
            }
        } else if let (Some(send), Some(csend)) = (send_and, safe_or) {
            (send < csend).then_some((".", csend))
        } else {
            None
        };
        let Some((desired, start)) = decision else {
            continue;
        };
        for operand in operands.iter().skip(start) {
            let safe = navigation_is_safe(&operand.call);
            let dot = operand.call.call_operator_loc();
            let appropriate = if desired == "&." {
                safe
            } else {
                !safe && (dot.is_some() || navigation_operator_method(&operand.call))
            };
            if appropriate {
                continue;
            }
            if desired == "." {
                let Some(operator) = dot else { continue };
                context.replace(
                    "Use `.` instead of unnecessary `&.`.",
                    operator.start_offset()..operator.end_offset(),
                    operator.start_offset()..operator.end_offset(),
                    ".",
                );
            } else if navigation_operator_method(&operand.call) || dot.is_none() {
                let location = operand.call.location();
                context.report(
                    "Use `&.` for consistency with safe navigation.",
                    location.start_offset()..location.end_offset(),
                );
            } else {
                let operator = dot.unwrap();
                context.replace(
                    "Use `&.` for consistency with safe navigation.",
                    operator.start_offset()..operator.end_offset(),
                    operator.start_offset()..operator.end_offset(),
                    "&.",
                );
            }
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum LogicalKind {
    And,
    Or,
}

struct NavigationOperand<'pr> {
    call: ruby_prism::CallNode<'pr>,
    logical: LogicalKind,
}

fn collect_navigation_operands<'pr>(
    node: &Node<'pr>,
    logical: Option<LogicalKind>,
    operands: &mut Vec<NavigationOperand<'pr>>,
) {
    if let Some(and) = node.as_and_node() {
        collect_navigation_operands(&and.left(), Some(LogicalKind::And), operands);
        collect_navigation_operands(&and.right(), Some(LogicalKind::And), operands);
    } else if let Some(or) = node.as_or_node() {
        collect_navigation_operands(&or.left(), Some(LogicalKind::Or), operands);
        collect_navigation_operands(&or.right(), Some(LogicalKind::Or), operands);
    } else if let (Some(call), Some(logical)) = (node.as_call_node(), logical) {
        operands.push(NavigationOperand { call, logical });
    }
}

fn navigation_is_safe(call: &ruby_prism::CallNode<'_>) -> bool {
    call.call_operator_loc()
        .is_some_and(|operator| operator.as_slice() == b"&.")
}

fn navigation_receiver_key(call: &ruby_prism::CallNode<'_>, file: SourceFile<'_>) -> String {
    let Some(receiver) = call.receiver() else {
        return String::new();
    };
    file.node(&receiver).to_string()
}

fn navigation_nilable(call: &ruby_prism::CallNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let name = String::from_utf8_lossy(call_name(call));
    matches!(
        name.as_ref(),
        "!" | "!=" | "!~" | "&" | "<=>" | "==" | "===" | "=~" | "^" | "__id__"
            | "__send__" | "class" | "clone" | "define_singleton_method" | "display" | "dup"
            | "enum_for" | "eql?" | "equal?" | "extend" | "freeze" | "frozen?" | "hash"
            | "inspect" | "instance_eval" | "instance_exec" | "instance_of?"
            | "instance_variable_defined?" | "instance_variable_get" | "instance_variable_set"
            | "instance_variables" | "is_a?" | "itself" | "kind_of?" | "method" | "methods"
            | "nil?" | "object_id" | "private_methods" | "protected_methods" | "public_method"
            | "public_methods" | "public_send" | "rationalize" | "remove_instance_variable"
            | "respond_to?" | "send" | "singleton_class" | "singleton_method"
            | "singleton_methods" | "tap" | "then" | "to_a" | "to_c" | "to_enum" | "to_f"
            | "to_h" | "to_i" | "to_r" | "to_s" | "yield_self" | "|" | "to_d"
    )
        || context
            .config_values("AllowedMethods")
            .iter()
            .any(|allowed| allowed.as_str() == name.as_ref())
}

fn navigation_operator_method(call: &ruby_prism::CallNode<'_>) -> bool {
    matches!(
        call_name(call),
        b"+" | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"<<"
            | b">>"
            | b">"
            | b"<"
            | b">="
            | b"<="
            | b"=="
            | b"!="
            | b"==="
            | b"=~"
            | b"!~"
    )
}

fn combinable_defined(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let mut search = 0;
    while let Some(relative) = source[search..].find("defined?(") {
        let chain_start = search + relative;
        let Some(first) = defined_call_at(source, chain_start) else {
            search = chain_start + "defined?".len();
            continue;
        };
        let mut calls = vec![first];
        let mut cursor = calls[0].end;
        while let Some((next_start, next)) = next_defined_call(source, cursor) {
            if let Some(previous) = calls.last_mut() {
                previous.following_operator_end = next_start;
            }
            calls.push(next);
            cursor = calls.last().map_or(cursor, |call| call.end);
            if next_start >= source.len() {
                break;
            }
        }

        let mut offenses = Vec::new();
        for current in 1..calls.len() {
            let Some(ancestor) = (0..current)
                .find(|prior| directly_nested(&calls[*prior].subject, &calls[current].subject))
            else {
                continue;
            };
            let edit = if calls[ancestor].subject.depth < calls[current].subject.depth {
                calls[ancestor].start..calls[ancestor].following_operator_end
            } else {
                calls[current].preceding_operator_start..calls[current].end
            };
            offenses.push((chain_start..calls[current].end, edit));
        }
        for (offense, edit) in offenses {
            context.remove("Combine nested `defined?` calls.", offense, edit);
        }
        search = calls.last().map_or(chain_start + 1, |call| call.end);
    }
}

#[derive(Clone)]
struct DefinedSubject {
    rooted: bool,
    parts: Vec<String>,
    depth: usize,
}

struct DefinedCall {
    start: usize,
    end: usize,
    preceding_operator_start: usize,
    following_operator_end: usize,
    subject: DefinedSubject,
}

fn defined_call_at(source: &str, start: usize) -> Option<DefinedCall> {
    let open = start + "defined?".len();
    let close = super::source_syntax::matching_delimiter(source, open, b'(', b')')?;
    let subject = defined_subject(source.get(open + 1..close)?.trim())?;
    Some(DefinedCall {
        start,
        end: close + 1,
        preceding_operator_start: start,
        following_operator_end: close + 1,
        subject,
    })
}

fn next_defined_call(source: &str, previous_end: usize) -> Option<(usize, DefinedCall)> {
    let tail = source.get(previous_end..)?;
    let leading = tail.len() - tail.trim_start_matches([' ', '\t']).len();
    let operator_start = previous_end + leading;
    let tail = &tail[leading..];
    let (operator, after_operator) = if let Some(after) = tail.strip_prefix("&&") {
        ("&&", after)
    } else if tail.starts_with("and") && tail.as_bytes().get(3).is_none_or(u8::is_ascii_whitespace)
    {
        ("and", &tail[3..])
    } else {
        return None;
    };
    let spacing = after_operator.len() - after_operator.trim_start_matches([' ', '\t']).len();
    let next_start = operator_start + operator.len() + spacing;
    if !source.get(next_start..)?.starts_with("defined?(") {
        return None;
    }
    let mut call = defined_call_at(source, next_start)?;
    call.preceding_operator_start = previous_end;
    call.following_operator_end = next_start;
    Some((next_start, call))
}

fn defined_subject(source: &str) -> Option<DefinedSubject> {
    let rooted = source.starts_with("::");
    let source = source.strip_prefix("::").unwrap_or(source);
    let parts = source
        .split(['.', ':'])
        .filter(|part| !part.is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if parts.is_empty()
        || parts.iter().any(|part| {
            !part
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        })
    {
        return None;
    }
    Some(DefinedSubject {
        rooted,
        depth: parts.len(),
        parts,
    })
}

fn directly_nested(left: &DefinedSubject, right: &DefinedSubject) -> bool {
    if left.rooted != right.rooted || left.depth.abs_diff(right.depth) != 1 {
        return false;
    }
    let shared = left.depth.min(right.depth);
    left.parts[..shared] == right.parts[..shared]
}

impl ForRule<'_, '_, '_> {
    fn on_for(&mut self, node: &ForNode<'_>) {
        return_if!(self.policy().enforced_style("each") != "each");
        let collection = node.collection();
        let variable = node.index();
        let collection_source = self.source_file().node(&collection);
        let variable_source = self.source_file().node(&variable);
        let collection_source = if for_collection_needs_parentheses(&collection, collection_source)
        {
            format!("({collection_source})")
        } else {
            collection_source.to_string()
        };
        let navigation = if collection
            .as_call_node()
            .and_then(|call| call.call_operator_loc())
            .is_some_and(|operator| operator.as_slice() == b"&.")
        {
            "&."
        } else {
            "."
        };
        let replacement = format!("{collection_source}{navigation}each do |{variable_source}|");
        let header_end = node
            .do_keyword_loc()
            .map_or(collection.location().end_offset(), |location| {
                location.end_offset()
            });
        let edit = node.for_keyword_loc().start_offset()..header_end;
        let offense = node.location();
        add_offense!(self, offense, message: "Prefer `each` over `for`.", |corrector| {
            corrector.replace(edit, replacement);
        });
    }

    fn on_block(&mut self, block: &BlockNode<'_>) {
        return_if!(self.policy().enforced_style("each") != "for");
        let Some(each) = self.parent().and_then(Node::as_call_node) else {
            return;
        };
        return_unless!(each.name().as_slice() == b"each" && argument_count(&each) == 0);
        let block_source = self
            .source_file()
            .slice(block.location().start_offset()..block.location().end_offset())
            .unwrap_or_default();
        return_if!(block_source.lines().count() <= 1);
        let Some(receiver) = each.receiver() else {
            return;
        };
        let explicit_parameters = block
            .parameters()
            .and_then(|parameters| parameters.as_block_parameters_node());
        let variable = explicit_parameters
            .as_ref()
            .map(|parameters| {
                self.source_file()
                    .slice(parameters.location().start_offset()..parameters.location().end_offset())
                    .unwrap_or_default()
                    .trim()
                    .trim_matches('|')
                    .trim()
            })
            .filter(|parameter| !parameter.is_empty())
            .unwrap_or("_");
        let receiver_source = self.source_file().node(&receiver);
        let replacement = format!("for {variable} in {receiver_source} do");
        let header_end = explicit_parameters
            .map_or(block.opening_loc().end_offset(), |parameters| {
                parameters.location().end_offset()
            });
        let edit = each.location().start_offset()..header_end;
        let offense = each.location().start_offset()..block.closing_loc().end_offset();
        add_offense!(self, offense, message: "Prefer `for` over `each`.", |corrector| {
            corrector.replace(edit, replacement);
        });
    }
}

fn for_collection_needs_parentheses(node: &Node<'_>, source: &str) -> bool {
    if source.trim_start().starts_with('(') {
        return false;
    }
    node.as_and_node().is_some()
        || node.as_or_node().is_some()
        || node.as_range_node().is_some()
        || node
            .as_call_node()
            .is_some_and(|call| matches!(call.name().as_slice(), b"+" | b"-" | b"*" | b"|" | b"&"))
}

#[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
fn class_module_children(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let definitions = definition_offsets(context.source());
    let mut compact_covered_until = 0usize;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let trimmed = line.trim_start();
        let (keyword, override_key) = if trimmed.starts_with("class ") {
            ("class ", "EnforcedStyleForClasses")
        } else if trimmed.starts_with("module ") {
            ("module ", "EnforcedStyleForModules")
        } else {
            continue;
        };
        let configured = context.config_value(override_key);
        let style = configured
            .filter(|style| !matches!(*style, "nil" | "null" | "~" | ""))
            .or_else(|| context.config_value("EnforcedStyle"))
            .unwrap_or("nested");
        let rest = trimmed.trim_start_matches(keyword);
        let name = rest.split([' ', '<', '#', ';']).next().unwrap_or("");
        if name.is_empty() || name.contains('(') {
            continue;
        }
        let indent = line.len() - trimmed.len();
        let declaration_start = offset + indent;
        if !definitions.all.contains(&declaration_start) {
            continue;
        }
        let name_start = offset + indent + keyword.len();
        let name_range = name_start..name_start + name.len();
        if style == "nested" {
            let path = name.trim_start_matches("::");
            if definitions.directly_nested.contains(&declaration_start)
                || !path.contains("::")
                || path
                    .split("::")
                    .any(|part| !part.as_bytes().first().is_some_and(u8::is_ascii_uppercase))
            {
                continue;
            }
            let parts = path.split("::").collect::<Vec<_>>();
            let end_index = definitions.ends.get(&declaration_start).and_then(|end| {
                let last = end.saturating_sub(1);
                lines.iter().position(|(offset, line)| {
                    *offset <= last && last <= offset + line.len()
                })
            });
            let Some(end_index) = end_index else {
                context.report(
                    "Use nested module/class definitions instead of compact style.",
                    name_range,
                );
                continue;
            };
            let width = context
                .config_value("IndentationWidth")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(2);
            let unit = if lines[index + 1..end_index]
                .iter()
                .any(|(_, line)| line.starts_with('\t'))
            {
                "\t".to_string()
            } else {
                " ".repeat(width)
            };
            let mut replacement = String::new();
            let base = &line[..indent];
            for (part_index, part) in parts.iter().enumerate() {
                let part = if part_index == 0 && name.starts_with("::") {
                    format!("::{part}")
                } else {
                    (*part).to_string()
                };
                let declaration = if part_index + 1 == parts.len() {
                    let suffix = &rest[name.len()..];
                    format!("{}{keyword}{part}{suffix}", unit.repeat(part_index))
                } else {
                    let namespace = &parts[..=part_index].join("::");
                    let namespace_kind = prior_namespace_kind(context.source(), offset, namespace)
                        .unwrap_or("module");
                    format!("{}{namespace_kind} {part}", unit.repeat(part_index))
                };
                replacement.push_str(base);
                replacement.push_str(&declaration);
                replacement.push('\n');
            }
            let added_depth = parts.len() - 1;
            for (_, body_line) in &lines[index + 1..end_index] {
                replacement.push_str(body_line);
                replacement.push('\n');
            }
            replacement.push_str(base);
            replacement.push_str(&unit.repeat(added_depth));
            replacement.push_str(lines[end_index].1.trim_start());
            replacement.push('\n');
            for close_depth in (0..added_depth).rev() {
                replacement.push_str(base);
                replacement.push_str(&unit.repeat(close_depth));
                replacement.push_str("end\n");
            }
            let edit_end = lines
                .get(end_index + 1)
                .map_or(context.source().len(), |(offset, _)| *offset);
            context.replace(
                "Use nested module/class definitions instead of compact style.",
                name_range.clone(),
                offset..edit_end,
                replacement,
            );
            continue;
        }
        if style != "compact" || index < compact_covered_until {
            continue;
        }
        if rest.contains('<') {
            continue;
        }
        let mut end = lines.len();
        let mut direct_children = Vec::new();
        let mut depth = 0usize;
        for (child_index, (_, child)) in lines.iter().enumerate().skip(index + 1) {
            let child = *child;
            let child_trimmed = child.trim_start();
            if child_trimmed.starts_with("class ") || child_trimmed.starts_with("module ") {
                if depth == 0 {
                    direct_children.push(child_index);
                }
                let one_line = child_trimmed
                    .split_once(';')
                    .is_some_and(|(_, suffix)| suffix.trim_start().starts_with("end"));
                if !one_line {
                    depth += 1;
                }
            } else if child_trimmed == "end" {
                if depth == 0 {
                    end = child_index;
                    break;
                }
                depth -= 1;
            }
        }
        if direct_children.len() == 1 {
            let direct_child = direct_children[0];
            let inline_replacement = line_has_inline_end(lines[direct_child].1.trim_start())
                .then(|| {
                    let child = lines[direct_child].1.trim_start();
                    let (child_keyword, child_rest) = child
                        .strip_prefix("class ")
                        .map(|rest| ("class ", rest))
                        .or_else(|| child.strip_prefix("module ").map(|rest| ("module ", rest)))?;
                    let child_name = child_rest.split([' ', ';']).next()?;
                    let base = &line[..indent];
                    let mut replacement =
                        format!("{base}{child_keyword}{name}::{child_name}\n{base}end\n");
                    let edit_end = lines
                        .get(end + 1)
                        .map_or(context.source().len(), |(offset, _)| *offset);
                    if edit_end < context.source().len() && !replacement.ends_with('\n') {
                        replacement.push('\n');
                    }
                    Some((edit_end, replacement))
                })
                .flatten();
            if let Some((edit_end, replacement)) =
                inline_replacement.or_else(|| compact_namespace_replacement(&lines, index))
            {
                context.replace(
                    "Use compact module/class definition instead of nested style.",
                    name_range.clone(),
                    offset..edit_end,
                    replacement,
                );
            }
            compact_covered_until = end;
        }
    }
}

#[derive(Default)]
struct DefinitionOffsets {
    all: HashSet<usize>,
    directly_nested: HashSet<usize>,
    ends: HashMap<usize, usize>,
}

fn definition_offsets(source: &str) -> DefinitionOffsets {
    #[derive(Default)]
    struct Collector {
        definitions: DefinitionOffsets,
    }

    impl Collector {
        fn collect_body(&mut self, body: Option<Node<'_>>) {
            let Some(statements) = body.and_then(|body| body.as_statements_node()) else {
                return;
            };
            if statements.body().len() != 1 {
                return;
            }
            let Some(child) = statements.body().first() else {
                return;
            };
            if child.as_class_node().is_some() || child.as_module_node().is_some() {
                self.definitions
                    .directly_nested
                    .insert(child.location().start_offset());
            }
        }
    }

    impl<'pr> Visit<'pr> for Collector {
        fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
            self.definitions.all.insert(node.location().start_offset());
            self.definitions
                .ends
                .insert(node.location().start_offset(), node.location().end_offset());
            self.collect_body(node.body());
            ruby_prism::visit_class_node(self, node);
        }

        fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
            self.definitions.all.insert(node.location().start_offset());
            self.definitions
                .ends
                .insert(node.location().start_offset(), node.location().end_offset());
            self.collect_body(node.body());
            ruby_prism::visit_module_node(self, node);
        }
    }

    let parsed = parse(source.as_bytes());
    let mut collector = Collector::default();
    collector.visit(&parsed.node());
    collector.definitions
}

fn compact_namespace_replacement(lines: &[(usize, &str)], start: usize) -> Option<(usize, String)> {
    let outer_end = declaration_end(lines, start)?;
    let outer_indent = lines[start].1.len() - lines[start].1.trim_start().len();
    let mut chain = vec![start];
    let mut current = start;
    loop {
        let end = declaration_end(lines, current)?;
        let mut depth = 0usize;
        let mut children = Vec::new();
        for (index, (_, line)) in lines.iter().enumerate().take(end).skip(current + 1) {
            let trimmed = line.trim_start();
            if depth == 0 && (trimmed.starts_with("class ") || trimmed.starts_with("module ")) {
                children.push(index);
            }
            if line_opens_block(trimmed) {
                depth += 1;
            } else if trimmed == "end" || trimmed.starts_with("end #") {
                depth = depth.saturating_sub(1);
            }
        }
        if children.len() != 1 {
            break;
        }
        current = children[0];
        chain.push(current);
    }
    if chain.len() < 2 {
        return None;
    }
    let deepest = *chain.last()?;
    let deepest_end = declaration_end(lines, deepest)?;
    let mut names = Vec::new();
    let mut final_keyword = "module ";
    let mut final_suffix = "";
    for &index in &chain {
        let trimmed = lines[index].1.trim_start();
        let (keyword, rest) = if let Some(rest) = trimmed.strip_prefix("class ") {
            ("class ", rest)
        } else {
            ("module ", trimmed.strip_prefix("module ")?)
        };
        let name = rest.split([' ', '<', '#', ';']).next()?;
        names.push(name);
        final_keyword = keyword;
        final_suffix = rest[name.len()..]
            .split_once(';')
            .map_or(&rest[name.len()..], |(before, _)| before);
    }
    let base = &lines[start].1[..outer_indent];
    let mut replacement = String::new();
    for pair in chain.windows(2) {
        for (_, line) in &lines[pair[0] + 1..pair[1]] {
            if line.trim_start().starts_with('#') {
                replacement.push_str(base);
                replacement.push_str(line.trim_start());
                replacement.push('\n');
            }
        }
    }
    replacement.push_str(base);
    replacement.push_str(final_keyword);
    replacement.push_str(&names.join("::"));
    replacement.push_str(final_suffix);
    replacement.push('\n');
    let deepest_indent = lines[deepest].1.len() - lines[deepest].1.trim_start().len();
    let deepest_prefix = &lines[deepest].1[..deepest_indent];
    let remove_indent = if deepest_prefix.contains('\t') {
        0
    } else {
        deepest_indent.saturating_sub(outer_indent)
    };
    for (_, line) in &lines[deepest + 1..deepest_end] {
        let leading = line.len() - line.trim_start().len();
        if remove_indent > 0 && leading > deepest_indent {
            replacement.push_str(&line[remove_indent..]);
        } else {
            replacement.push_str(line);
        }
        replacement.push('\n');
    }
    replacement.push_str(base);
    replacement.push_str(lines[outer_end].1.trim_start());
    replacement.push('\n');
    let edit_end = lines.get(outer_end + 1).map_or_else(
        || lines[outer_end].0 + lines[outer_end].1.len() + 1,
        |(offset, _)| *offset,
    );
    Some((edit_end, replacement))
}

fn declaration_end(lines: &[(usize, &str)], start: usize) -> Option<usize> {
    if line_has_inline_end(lines[start].1.trim_start()) {
        return Some(start);
    }
    let mut depth = 0usize;
    for (index, (_, line)) in lines.iter().enumerate().skip(start + 1) {
        let trimmed = line.trim_start();
        if line_opens_block(trimmed) {
            depth += 1;
        } else if trimmed == "end" || trimmed.starts_with("end #") {
            if depth == 0 {
                return Some(index);
            }
            depth -= 1;
        }
    }
    None
}

fn line_opens_block(line: &str) -> bool {
    if line_has_inline_end(line) {
        return false;
    }
    [
        "class ", "module ", "def ", "if ", "unless ", "case ", "begin", "while ", "until ", "for ",
    ]
    .iter()
    .any(|keyword| line.starts_with(keyword))
        || line.ends_with(" do")
        || line.contains(" do |")
}

fn line_has_inline_end(line: &str) -> bool {
    line.rsplit_once(';')
        .is_some_and(|(_, tail)| tail.trim_start().starts_with("end"))
}

fn prior_namespace_kind<'a>(source: &'a str, before: usize, namespace: &str) -> Option<&'a str> {
    source[..before].lines().rev().find_map(|line| {
        let line = line.trim_start();
        if line
            .strip_prefix("class ")
            .is_some_and(|name| name.split_whitespace().next() == Some(namespace))
        {
            Some("class")
        } else if line
            .strip_prefix("module ")
            .is_some_and(|name| name.split_whitespace().next() == Some(namespace))
        {
            Some("module")
        } else {
            None
        }
    })
}

#[allow(clippy::too_many_lines)]
fn safe_navigation_chain(node: &ruby_prism::CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .call_operator_loc()
        .is_some_and(|operator| operator.as_slice() == b"&.")
    {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let Some(safe_call) = receiver.as_call_node() else {
        return;
    };
    if safe_call
        .call_operator_loc().is_none_or(|operator| operator.as_slice() != b"&.")
    {
        return;
    }
    let method = call_name(node);
    if matches!(
        method,
        b"!"
            | b"!="
            | b"!~"
            | b"&"
            | b"<=>"
            | b"=="
            | b"==="
            | b"=~"
            | b"^"
            | b"__id__"
            | b"__send__"
            | b"class"
            | b"clone"
            | b"define_singleton_method"
            | b"display"
            | b"dup"
            | b"enum_for"
            | b"eql?"
            | b"equal?"
            | b"extend"
            | b"freeze"
            | b"frozen?"
            | b"hash"
            | b"inspect"
            | b"instance_eval"
            | b"instance_exec"
            | b"instance_of?"
            | b"instance_variable_defined?"
            | b"instance_variable_get"
            | b"instance_variable_set"
            | b"instance_variables"
            | b"is_a?"
            | b"itself"
            | b"kind_of?"
            | b"method"
            | b"methods"
            | b"nil?"
            | b"object_id"
            | b"private_methods"
            | b"protected_methods"
            | b"public_method"
            | b"public_methods"
            | b"public_send"
            | b"rationalize"
            | b"remove_instance_variable"
            | b"respond_to?"
            | b"send"
            | b"singleton_class"
            | b"singleton_method"
            | b"singleton_methods"
            | b"tap"
            | b"then"
            | b"to_a"
            | b"to_c"
            | b"to_enum"
            | b"to_f"
            | b"to_h"
            | b"to_i"
            | b"to_r"
            | b"to_s"
            | b"yield_self"
            | b"|"
            | b"present?"
            | b"blank?"
            | b"presence"
            | b"try"
            | b"try!"
            | b"to_d"
            | b"in?"
            | b"||"
            | b"&&"
            | b"+@"
            | b"-@"
    ) {
        return;
    }
    if context
        .config_values("AllowedMethods")
        .iter()
        .any(|allowed| allowed.as_bytes() == method)
    {
        return;
    }

    let receiver_start = receiver.location().start_offset();
    let receiver_source = context
        .source_file()
        .node(&receiver)
        .split("&.")
        .next()
        .unwrap_or("")
        .trim();
    let line_prefix = &context.source()[context.source()[..receiver_start]
        .rfind('\n')
        .map_or(0, |at| at + 1)..receiver_start];
    if let Some(and_at) = line_prefix.rfind("&&") {
        let lhs = line_prefix[..and_at].trim();
        if lhs.starts_with(receiver_source) && lhs.contains("&.") {
            return;
        }
    }
    let conditionally_unsafe = if let Some(question) = line_prefix.find('?') {
        let condition = line_prefix[..question].trim();
        if !line_prefix[question + 1..].contains(':')
            && condition == context.source_file().node(&receiver)
        {
            return;
        }
        line_prefix[question + 1..].contains(':')
            && condition == context.source_file().node(&receiver)
    } else {
        false
    };

    let receiver_end = receiver.location().end_offset();
    let (offense, primary_edit) = if method == b"[]" {
        let source = &context.source()[receiver_end..node.location().end_offset()];
        let inner = source
            .strip_prefix('[')
            .and_then(|source| source.strip_suffix(']'))
            .unwrap_or(source);
        (
            receiver_end..node.location().end_offset(),
            (
                receiver_end..node.location().end_offset(),
                format!("&.[]({inner})"),
            ),
        )
    } else if method == b"[]=" {
        let source = &context.source()[receiver_end..node.location().end_offset()];
        let (indices, value) = source.split_once('=').unwrap_or((source, ""));
        let indices = indices.trim().trim_start_matches('[').trim_end_matches(']');
        (
            receiver_end..node.location().end_offset(),
            (
                receiver_end..node.location().end_offset(),
                format!("&.[]=({indices}, {})", value.trim()),
            ),
        )
    } else if let Some(operator) = node.call_operator_loc() {
        (
            operator.start_offset()..node.location().end_offset(),
            (
                operator.start_offset()..operator.end_offset(),
                "&.".to_string(),
            ),
        )
    } else {
        (
            receiver_end..node.location().end_offset(),
            (receiver_end..receiver_end, "&.".to_string()),
        )
    };
    let mut edits = vec![primary_edit];
    let mut child_end = node.location().end_offset();
    for ancestor in context
        .ancestors()
        .iter()
        .rev()
        .take_while(|_| method != b"[]")
    {
        let Some(call) = ancestor.as_call_node() else {
            break;
        };
        let Some(parent_receiver) = call.receiver() else {
            break;
        };
        if parent_receiver.location().start_offset() != node.location().start_offset()
            || parent_receiver.location().end_offset() != child_end
        {
            break;
        }
        let Some(operator) = call.call_operator_loc() else {
            break;
        };
        if operator.as_slice() != b"." {
            break;
        }
        edits.push((
            operator.start_offset()..operator.end_offset(),
            "&.".to_string(),
        ));
        child_end = call.location().end_offset();
    }
    let binary_operator = matches!(
        method,
        b">=" | b"<=" | b">" | b"<" | b"-" | b"+" | b"*" | b"/" | b"%"
    );
    let requires_parentheses = binary_operator
        && context.ancestors().iter().rev().take(2).any(|ancestor| {
            ancestor
                .as_and_node()
                .is_some_and(|and| and.operator_loc().as_slice() == b"&&")
                || ancestor
                    .as_or_node()
                    .is_some_and(|or| or.operator_loc().as_slice() == b"||")
                || ancestor.as_array_node().is_some()
                || ancestor.as_hash_node().is_some()
                || ancestor
                    .as_call_node()
                    .is_some_and(|call| matches!(call_name(&call), b"==" | b"!=" | b"===" | b"<=>"))
        });
    if requires_parentheses {
        edits.push((
            node.location().start_offset()..node.location().start_offset(),
            "(".to_string(),
        ));
        edits.push((
            node.location().end_offset()..node.location().end_offset(),
            ")".to_string(),
        ));
    }
    if conditionally_unsafe {
        context.report(
            "Do not chain ordinary method call after safe navigation operator.",
            offense,
        );
    } else {
        context.replace_many(
            "Do not chain ordinary method call after safe navigation operator.",
            offense,
            edits,
        );
    }
}

#[allow(clippy::too_many_lines)]
fn block_delimiters(node: &BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let opening = node.opening_loc();
    let closing = node.closing_loc();
    let braces = opening.as_slice() == b"{";
    let multiline = context.source_file().node(&node.as_node()).contains('\n');
    let call = context.parent().and_then(Node::as_call_node);
    let method = call
        .as_ref()
        .map(|call| String::from_utf8_lossy(call_name(call)).into_owned())
        .unwrap_or_default();
    if context.policy().allows_method(method.as_bytes()) || block_is_ambiguous_argument(context) {
        return;
    }
    let required = context.config_values("BracesRequiredMethods");
    let style = context.policy().enforced_style("line_count_based");
    if !braces
        && !multiline
        && context
            .source_file()
            .node(&node.as_node())
            .contains("; rescue")
    {
        return;
    }
    if block_nested_in_improper_block(context, style)
        || block_precedes_improper_chained_block(node, context, style)
    {
        return;
    }
    let functional_method = context
        .config_values("FunctionalMethods")
        .iter()
        .any(|configured| configured == &method);
    let procedural_method = context
        .config_values("ProceduralMethods")
        .iter()
        .any(|configured| configured == &method);
    let value_used = block_value_is_used(node, context);
    let value_of_scope = block_is_return_value_of_scope(node, context);
    let chained = block_is_chained(context);
    let want_braces = if required.iter().any(|required| required == &method) {
        true
    } else {
        match style {
            "always_braces" => true,
            "semantic" => {
                if braces {
                    functional_method
                        || value_used
                        || value_of_scope
                        || !multiline
                            && context.config_bool("AllowBracesOnProceduralOneLiners", false)
                } else {
                    !procedural_method && value_used
                }
            }
            "braces_for_chaining" => chained || !multiline,
            _ => !multiline,
        }
    };
    if braces == want_braces {
        return;
    }
    let (message, replacement_open, replacement_close) = if want_braces {
        let message = if required.iter().any(|required| required == &method) {
            format!("Brace delimiters `{{...}}` required for '{method}' method.")
        } else {
            match style {
                "semantic" => "Prefer `{...}` over `do...end` for functional blocks.".to_string(),
                "braces_for_chaining" if multiline => {
                    "Prefer `{...}` over `do...end` for multi-line chained blocks.".to_string()
                }
                "always_braces" => "Prefer `{...}` over `do...end` for blocks.".to_string(),
                _ => "Prefer `{...}` over `do...end` for single-line blocks.".to_string(),
            }
        };
        (message, "{", "}")
    } else {
        let message = match style {
            "semantic" => "Prefer `do...end` over `{...}` for procedural blocks.",
            "braces_for_chaining" => "Prefer `do...end` for multi-line blocks without chaining.",
            _ => "Avoid using `{...}` for multi-line blocks.",
        };
        (message.to_string(), "do", "end")
    };
    let unsafe_change = want_braces
        && call.as_ref().is_some_and(|call| {
            !braces
                && call.opening_loc().is_none()
                && call
                    .arguments()
                    .is_some_and(|arguments| !arguments.arguments().is_empty())
        });
    if unsafe_change {
        context.report(message, opening);
    } else {
        let opening_range = opening.start_offset()..opening.end_offset();
        let closing_range = closing.start_offset()..closing.end_offset();
        let source = context.source();
        let leading_space = !want_braces
            && opening_range.start > 0
            && !source.as_bytes()[opening_range.start - 1].is_ascii_whitespace();
        let trailing_space = source.as_bytes().get(opening_range.end) == Some(&b'|');
        let mut corrected_open = String::new();
        if leading_space {
            corrected_open.push(' ');
        }
        corrected_open.push_str(replacement_open);
        if trailing_space {
            corrected_open.push(' ');
        }
        let leading_close_space = !want_braces
            && closing_range.start > 0
            && !source.as_bytes()[closing_range.start - 1].is_ascii_whitespace();
        let corrected_close = if leading_close_space {
            format!(" {replacement_close}")
        } else {
            replacement_close.to_string()
        };
        let mut edits = vec![
            (opening_range.clone(), corrected_open),
            (closing_range.clone(), corrected_close),
        ];
        let block_source = &source[opening_range.end..closing_range.start];
        if want_braces
            && (block_source.contains("\nrescue") || block_source.contains("\nensure"))
            && !block_source.trim_start().starts_with("begin")
        {
            if let Some(first_line_break) = block_source.find('\n') {
                let line_start = opening_range.end + first_line_break + 1;
                let statement_start = line_start
                    + source[line_start..closing_range.start]
                        .bytes()
                        .take_while(|byte| matches!(byte, b' ' | b'\t'))
                        .count();
                edits.push((statement_start..statement_start, "begin\n".to_string()));
                edits.push((
                    closing_range.start..closing_range.start,
                    "end\n".to_string(),
                ));
            }
        }
        if !want_braces {
            let line_end = source[closing_range.end..]
                .find('\n')
                .map_or(source.len(), |at| closing_range.end + at);
            if let Some(relative_comment) = source[closing_range.end..line_end].find('#') {
                let comment_start = closing_range.end + relative_comment;
                let removal_start = source[..comment_start]
                    .rfind(|character: char| !matches!(character, ' ' | '\t'))
                    .map_or(comment_start, |at| at + 1);
                let mut removal_end = line_end;
                if source.as_bytes().get(line_end) == Some(&b'\n')
                    && source.as_bytes().get(line_end + 1) == Some(&b'\n')
                {
                    removal_end += 1;
                }
                let insertion = call
                    .as_ref()
                    .map_or(node.location().start_offset(), |call| {
                        call.location().start_offset()
                    });
                edits.push((
                    insertion..insertion,
                    format!("{}\n", source[comment_start..line_end].trim_end()),
                ));
                edits.push((removal_start..removal_end, String::new()));
            }
        }
        context.replace_many(message, opening_range.clone(), edits);
    }
}

fn block_value_is_used(_node: &BlockNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let Some(call) = context.parent().and_then(Node::as_call_node) else {
        return false;
    };
    for ancestor in context.ancestors().iter().rev().skip(1) {
        if ancestor.as_statements_node().is_some() || ancestor.as_parentheses_node().is_some() {
            continue;
        }
        return block_assignment_node(ancestor)
            || ancestor.as_call_node().is_some_and(|outer| {
                outer.location().start_offset() != call.location().start_offset()
                    || outer.location().end_offset() != call.location().end_offset()
            });
    }
    false
}

fn block_assignment_node(node: &Node<'_>) -> bool {
    node.as_multi_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
        || node.as_local_variable_or_write_node().is_some()
        || node.as_instance_variable_or_write_node().is_some()
        || node.as_class_variable_or_write_node().is_some()
        || node.as_global_variable_or_write_node().is_some()
        || node.as_local_variable_and_write_node().is_some()
        || node.as_instance_variable_and_write_node().is_some()
        || node.as_class_variable_and_write_node().is_some()
        || node.as_global_variable_and_write_node().is_some()
}

fn block_is_return_value_of_scope(node: &BlockNode<'_>, context: &CopContext<'_, '_>) -> bool {
    let Some(call) = context.parent().and_then(Node::as_call_node) else {
        return false;
    };
    if context.ancestors().iter().rev().skip(1).any(|ancestor| {
        ancestor.as_if_node().is_some()
            || ancestor.as_unless_node().is_some()
            || ancestor.as_while_node().is_some()
            || ancestor.as_until_node().is_some()
            || ancestor.as_case_node().is_some()
            || ancestor.as_case_match_node().is_some()
            || ancestor.as_and_node().is_some()
            || ancestor.as_or_node().is_some()
            || ancestor.as_array_node().is_some()
            || ancestor.as_range_node().is_some()
    }) {
        return true;
    }
    for (index, ancestor) in context.ancestors().iter().enumerate().rev().skip(1) {
        if ancestor.as_parentheses_node().is_some() {
            continue;
        }
        if ancestor.as_if_node().is_some()
            || ancestor.as_unless_node().is_some()
            || ancestor.as_and_node().is_some()
            || ancestor.as_or_node().is_some()
            || ancestor.as_array_node().is_some()
            || ancestor.as_range_node().is_some()
        {
            return true;
        }
        if let Some(statements) = ancestor.as_statements_node() {
            let last_is_call = statements.body().last().is_some_and(|last| {
                last.location().start_offset() == call.location().start_offset()
                    && last.location().end_offset() == call.location().end_offset()
            });
            if !last_is_call {
                return false;
            }
            return context.ancestors()[..index].iter().rev().any(|owner| {
                owner.as_block_node().is_some()
                    || owner.as_def_node().is_some()
                    || owner.as_lambda_node().is_some()
            });
        }
        return false;
    }
    let _ = node;
    false
}

fn block_is_chained(context: &CopContext<'_, '_>) -> bool {
    let Some(call) = context.parent().and_then(Node::as_call_node) else {
        return false;
    };
    context.ancestors().iter().rev().skip(1).any(|ancestor| {
        ancestor.as_call_node().is_some_and(|outer| {
            outer.receiver().is_some_and(|receiver| {
                receiver.location().start_offset() == call.location().start_offset()
                    && receiver.location().end_offset() == call.location().end_offset()
            })
        })
    })
}

fn block_is_ambiguous_argument(context: &CopContext<'_, '_>) -> bool {
    let Some(call) = context.parent().and_then(Node::as_call_node) else {
        return false;
    };
    for ancestor in context.ancestors().iter().rev().skip(1) {
        if ancestor.as_parentheses_node().is_some() {
            return false;
        }
        if ancestor.as_hash_node().is_some() {
            return false;
        }
        if ancestor.as_splat_node().is_some() {
            return false;
        }
        if let Some(outer) = ancestor.as_call_node() {
            let single_argument_operator = matches!(
                call_name(&outer),
                b"+" | b"-"
                    | b"*"
                    | b"/"
                    | b"%"
                    | b"**"
                    | b"=="
                    | b"!="
                    | b"==="
                    | b"=~"
                    | b"!~"
                    | b"<"
                    | b">"
                    | b"<="
                    | b">="
                    | b"<=>"
                    | b"<<"
                    | b">>"
                    | b"&"
                    | b"|"
                    | b"^"
            ) && outer
                .arguments()
                .is_some_and(|arguments| arguments.arguments().len() == 1);
            let contains_as_argument = outer.arguments().is_some_and(|arguments| {
                arguments.arguments().iter().any(|argument| {
                    argument.location().start_offset() <= call.location().start_offset()
                        && call.location().end_offset() <= argument.location().end_offset()
                })
            });
            if single_argument_operator && contains_as_argument && block_is_chained(context) {
                return true;
            }
            if outer.opening_loc().is_none()
                && !call_name(&outer).ends_with(b"=")
                && !single_argument_operator
                && contains_as_argument
            {
                return true;
            }
        }
    }
    false
}

fn block_nested_in_improper_block(context: &CopContext<'_, '_>, style: &str) -> bool {
    context
        .ancestors()
        .iter()
        .enumerate()
        .rev()
        .skip(1)
        .any(|(index, ancestor)| {
        let Some(block) = ancestor.as_block_node() else {
            return false;
        };
        let method = index
            .checked_sub(1)
            .and_then(|parent| context.ancestors()[parent].as_call_node())
            .map(|call| call_name(&call));
        if method.is_some_and(|method| context.policy().allows_method(method)) {
            return false;
        }
        let braces = block.opening_loc().as_slice() == b"{";
        let multiline = context.source_file().node(&block.as_node()).contains('\n');
        match style {
            "line_count_based" => braces == multiline,
            "always_braces" => !braces,
            _ => false,
        }
    })
}

fn block_precedes_improper_chained_block(
    node: &BlockNode<'_>,
    context: &CopContext<'_, '_>,
    style: &str,
) -> bool {
    let current = node.location();
    context.ancestors().iter().rev().skip(1).any(|ancestor| {
        let Some(call) = ancestor.as_call_node() else {
            return false;
        };
        let Some(receiver) = call.receiver() else {
            return false;
        };
        if receiver.location().start_offset() > current.start_offset()
            || receiver.location().end_offset() < current.end_offset()
            || context.policy().allows_method(call_name(&call))
        {
            return false;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return false;
        };
        let braces = block.opening_loc().as_slice() == b"{";
        let multiline = context.source_file().node(&block.as_node()).contains('\n');
        match style {
            "line_count_based" => braces == multiline,
            "always_braces" => !braces,
            _ => false,
        }
    })
}

#[allow(clippy::too_many_lines)]
fn redundant_safe_navigation(node: &ruby_prism::CallNode<'_>, context: &mut CopContext<'_, '_>) {
    use crate::rubocop::cop::mixin::allowed_methods::AllowedMethods;

    let Some(operator) = node.call_operator_loc() else {
        return;
    };
    if operator.as_slice() != b"&." {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let receiver_source = context.source_file().node(&receiver);

    if let Some(or_node) = context.ancestors().iter().rev().find_map(Node::as_or_node) {
        if or_node.left().location().start_offset() == node.location().start_offset()
            && or_node.left().location().end_offset() == node.location().end_offset()
        {
            let expected = match call_name(node) {
                b"to_h" => Some("{}"),
                b"to_a" => Some("[]"),
                b"to_i" => Some("0"),
                b"to_f" => Some("0.0"),
                b"to_s" => Some("''"),
                _ => None,
            };
            if expected
                .is_some_and(|expected| context.source_file().node(&or_node.right()) == expected)
            {
                let replacement = context
                    .source_file()
                    .node(&or_node.left())
                    .replacen("&.", ".", 1);
                context.replace(
                    "Redundant safe navigation with default literal detected.",
                    operator.start_offset()..or_node.location().end_offset(),
                    or_node.location(),
                    replacement,
                );
                return;
            }
        }
    }

    if statically_non_nil(&receiver) {
        context.replace(
            "Redundant safe navigation detected, use `.` instead.",
            operator.start_offset()..operator.end_offset(),
            operator.start_offset()..operator.end_offset(),
            ".",
        );
        return;
    }

    let line_start = context.source()[..operator.start_offset()]
        .rfind('\n')
        .map_or(0, |at| at + 1);
    let allowed_methods = AllowedMethods::new(
        context.config_values("AllowedMethods").to_vec(),
        Vec::new(),
        Vec::new(),
    );
    let allowed = std::str::from_utf8(call_name(node))
        .is_ok_and(|method| allowed_methods.allowed_method(method));
    let nil_responds = call_name(node) == b"respond_to?"
        && node.arguments().is_some_and(|arguments| {
            arguments.arguments().iter().next().is_some_and(|argument| {
                matches!(
                    context
                        .source_file()
                        .node(&argument)
                        .trim_start_matches(':'),
                    "to_a" | "class" | "nil?" | "to_s" | "to_i" | "to_f" | "to_h"
                )
            })
        });
    if allowed && safe_navigation_condition_context(node, context) && !nil_responds {
        context.replace(
            "Redundant safe navigation detected, use `.` instead.",
            operator.start_offset()..operator.end_offset(),
            operator.start_offset()..operator.end_offset(),
            ".",
        );
        return;
    }
    if !context.config_bool("InferNonNilReceiver", false) {
        return;
    }
    let invoked_before = inferred_non_nil_from_source(
        context.source(),
        line_start,
        receiver_source,
        context.config_values("AdditionalNilMethods"),
    );
    let proven_by_condition = context.ancestors().iter().rev().any(|ancestor| {
        let Some(condition) = ancestor.as_if_node() else {
            return false;
        };
        let predicate = context.source_file().node(&condition.predicate());
        let in_truthy = condition.statements().is_some_and(|statements| {
            statements.location().start_offset() <= node.location().start_offset()
                && node.location().end_offset() <= statements.location().end_offset()
        });
        let ordinary_method = predicate
            .strip_prefix(&format!("{receiver_source}."))
            .and_then(|rest| {
                rest.split(|character: char| {
                    !character.is_ascii_alphanumeric()
                        && character != '_'
                        && character != '?'
                        && character != '!'
                })
                .next()
            });
        let ordinary_proves_non_nil = ordinary_method.is_some_and(|method| {
            !matches!(method, "nil?" | "to_s" | "to_i" | "to_f" | "to_a" | "to_h")
                && !context
                    .config_values("AdditionalNilMethods")
                    .iter()
                    .any(|nil_method| nil_method == method)
        });
        ordinary_proves_non_nil
            || in_truthy
                && (predicate == receiver_source
                    || predicate.starts_with(&format!("{receiver_source}&.")))
    });
    if invoked_before || proven_by_condition {
        context.replace(
            "Redundant safe navigation on non-nil receiver (detected by analyzing previous code/method invocations).",
            operator.start_offset()..operator.end_offset(),
            operator.start_offset()..operator.end_offset(), ".",
        );
    }
}

fn safe_navigation_condition_context(
    node: &ruby_prism::CallNode<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let same_location = |candidate: &Node<'_>| {
        candidate.location().start_offset() == node.location().start_offset()
            && candidate.location().end_offset() == node.location().end_offset()
    };
    let Some(parent) = context.parent() else {
        return false;
    };
    if parent.as_and_node().is_some() || parent.as_or_node().is_some() {
        return true;
    }
    if parent.as_call_node().is_some_and(|call| {
        call.name().as_slice() == b"!"
            && call.receiver().as_ref().is_some_and(&same_location)
    }) {
        return true;
    }
    parent
        .as_if_node()
        .is_some_and(|conditional| same_location(&conditional.predicate()))
        || parent
            .as_unless_node()
            .is_some_and(|conditional| same_location(&conditional.predicate()))
        || parent
            .as_while_node()
            .is_some_and(|conditional| same_location(&conditional.predicate()))
        || parent
            .as_until_node()
            .is_some_and(|conditional| same_location(&conditional.predicate()))
}

fn inferred_non_nil_from_source(
    source: &str,
    current_line_start: usize,
    receiver: &str,
    additional_nil_methods: &[String],
) -> bool {
    let marker = format!("{receiver}.");
    let lines = source[..current_line_start].lines().collect::<Vec<_>>();
    let mut branch_boundaries = Vec::<usize>::new();
    if let Some(current) = source[current_line_start..].lines().next() {
        let trimmed = current.trim_start();
        if trimmed.starts_with("elsif ")
            || trimmed.starts_with("when ")
            || trimmed.starts_with("in ")
            || matches!(trimmed, "else" | "rescue" | "ensure")
        {
            branch_boundaries.push(current.len() - trimmed.len());
        }
    }
    for line in lines.iter().rev() {
        let trimmed = line.trim_start();
        let indent = line.len() - trimmed.len();
        if matches!(trimmed, "else" | "rescue" | "ensure")
            || trimmed.starts_with("elsif ")
            || trimmed.starts_with("when ")
            || trimmed.starts_with("in ")
        {
            branch_boundaries.push(indent);
        }
        let Some(at) = line.find(&marker) else {
            if trimmed.starts_with("def ") {
                break;
            }
            continue;
        };
        let method = line[at + marker.len()..]
            .split(|character: char| {
                !character.is_ascii_alphanumeric()
                    && character != '_'
                    && character != '?'
                    && character != '!'
            })
            .next()
            .unwrap_or("");
        if matches!(method, "nil?" | "to_s" | "to_i" | "to_f" | "to_a" | "to_h")
            || additional_nil_methods
                .iter()
                .any(|nil_method| nil_method == method)
        {
            continue;
        }
        if let Some(expression) = ["if ", "elsif ", "when ", "case "]
            .iter()
            .find_map(|keyword| trimmed.strip_prefix(keyword))
        {
            return expression.trim_start_matches('(').starts_with(&marker);
        }
        if line[..at].contains("&&") {
            continue;
        }
        if !branch_boundaries.iter().any(|boundary| *boundary < indent) {
            return true;
        }
    }
    if let Some(else_index) = lines.iter().rposition(|line| line.trim() == "else") {
        if let Some(rescue_index) = lines[..else_index]
            .iter()
            .rposition(|line| line.trim_start().starts_with("rescue"))
        {
            if let Some(begin_index) = lines[..rescue_index]
                .iter()
                .rposition(|line| line.trim() == "begin")
            {
                return lines[begin_index + 1..rescue_index]
                    .iter()
                    .any(|line| line.contains(&marker));
            }
        }
    }
    false
}

fn statically_non_nil(node: &Node<'_>) -> bool {
    if node.as_self_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_interpolated_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
    {
        return true;
    }
    if node.as_call_node().is_some_and(|call| {
        call
            .call_operator_loc().is_none_or(|operator| operator.as_slice() != b"&.")
            && matches!(
                call_name(&call),
                b"to_s" | b"to_i" | b"to_f" | b"to_a" | b"to_h"
            )
    }) {
        return true;
    }
    let name = node
        .as_constant_read_node()
        .map(|constant| constant.name().as_slice())
        .or_else(|| {
            node.as_constant_path_node()
                .and_then(|constant| constant.name())
                .map(|name| name.as_slice())
        });
    name.is_some_and(|name| {
        name.first().is_some_and(u8::is_ascii_uppercase)
            && name.iter().any(u8::is_ascii_lowercase)
    })
}

fn and_or(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (operator, old, new, left, right) = if let Some(and) = node.as_and_node() {
        (
            and.operator_loc(),
            b"and".as_slice(),
            "&&",
            and.left(),
            and.right(),
        )
    } else if let Some(or) = node.as_or_node() {
        (
            or.operator_loc(),
            b"or".as_slice(),
            "||",
            or.left(),
            or.right(),
        )
    } else {
        return;
    };
    if operator.as_slice() != old {
        return;
    }
    if context.policy().enforced_style("conditionals") == "conditionals"
        && !context.ancestors().iter().any(|ancestor| {
            ancestor.as_if_node().is_some_and(|condition| {
                condition.predicate().location().start_offset() <= node.location().start_offset()
                    && node.location().end_offset() <= condition.predicate().location().end_offset()
            }) || ancestor.as_unless_node().is_some_and(|condition| {
                condition.predicate().location().start_offset() <= node.location().start_offset()
                    && node.location().end_offset() <= condition.predicate().location().end_offset()
            }) || ancestor.as_while_node().is_some_and(|condition| {
                condition.predicate().location().start_offset() <= node.location().start_offset()
                    && node.location().end_offset() <= condition.predicate().location().end_offset()
            }) || ancestor.as_until_node().is_some_and(|condition| {
                condition.predicate().location().start_offset() <= node.location().start_offset()
                    && node.location().end_offset() <= condition.predicate().location().end_offset()
            })
        })
    {
        return;
    }
    let old = std::str::from_utf8(old).unwrap_or("");
    let mut edits = vec![(
        operator.start_offset()..operator.end_offset(),
        new.to_string(),
    )];
    let needs_parent_group = new == "||"
        && context.parent().is_some_and(|parent| {
            parent
                .as_and_node()
                .is_some_and(|logical| matches!(logical.operator_loc().as_slice(), b"and" | b"&&"))
        });
    if needs_parent_group {
        edits.push((
            node.location().start_offset()..node.location().start_offset(),
            "(".to_string(),
        ));
        edits.push((
            node.location().end_offset()..node.location().end_offset(),
            ")".to_string(),
        ));
    }
    for operand in [&left, &right] {
        if and_or_group_operand(operand, new, context.source_file()) {
            edits.push((
                operand.location().start_offset()..operand.location().start_offset(),
                "(".to_string(),
            ));
            edits.push((
                operand.location().end_offset()..operand.location().end_offset(),
                ")".to_string(),
            ));
        } else {
            and_or_command_call_edits(operand, &mut edits);
        }
    }
    context.replace_many(
        format!("Use `{new}` instead of `{old}`."),
        operator.start_offset()..operator.end_offset(),
        edits,
    );
}

fn and_or_group_operand(node: &Node<'_>, replacement: &str, file: SourceFile<'_>) -> bool {
    if and_or_assignment(node)
        || node
            .as_return_node()
            .is_some_and(|_| file.node(node).trim() != "return")
        || node
            .as_break_node()
            .is_some_and(|_| file.node(node).trim() != "break")
        || node
            .as_next_node()
            .is_some_and(|_| file.node(node).trim() != "next")
        || replacement == "&&"
            && node
                .as_or_node()
                .is_some_and(|logical| logical.operator_loc().as_slice() == b"||")
    {
        return true;
    }
    if file.node(node).trim_start().starts_with("not ") {
        return true;
    }
    if let Some(call) = node.as_call_node() {
        let name = call.name();
        let name = name.as_slice();
        return name.ends_with(b"=") && !matches!(name, b"==" | b"!=" | b">=" | b"<=" | b"===")
            || matches!(name, b"==" | b"!=" | b">" | b"<" | b">=" | b"<=" | b"<=>");
    }
    false
}

fn and_or_assignment(node: &Node<'_>) -> bool {
    node.as_multi_write_node().is_some()
        || node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
}

fn and_or_command_call_edits(node: &Node<'_>, edits: &mut Vec<(std::ops::Range<usize>, String)>) {
    struct Finder<'a> {
        edits: &'a mut Vec<(std::ops::Range<usize>, String)>,
    }

    impl<'pr> Visit<'pr> for Finder<'_> {
        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            if node.opening_loc().is_none() {
                if let Some(arguments) = node.arguments() {
                    if let Some((first, last)) = arguments
                        .arguments()
                        .first()
                        .zip(arguments.arguments().last())
                    {
                        let opening_start = node
                            .message_loc()
                            .map_or(first.location().start_offset(), |message| {
                                message.end_offset()
                            });
                        self.edits.push((
                            opening_start..first.location().start_offset(),
                            "(".to_string(),
                        ));
                        self.edits.push((
                            last.location().end_offset()..last.location().end_offset(),
                            ")".to_string(),
                        ));
                        return;
                    }
                }
            }
            ruby_prism::visit_call_node(self, node);
        }
    }

    Finder { edits }.visit(node);
}
