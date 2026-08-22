use ruby_prism::{BlockNode, ForNode, Node};

use super::*;

define_cops! {
    SafeNavigationConsistency => "Lint/SafeNavigationConsistency" => source(safe_navigation_consistency),
    CombinableDefined => "Style/CombinableDefined" => source(combinable_defined),
    For => "Style/For" => rubocop_callbacks(ForRule, [on_for, on_block]),
    ClassAndModuleChildren => "Style/ClassAndModuleChildren" => source(class_module_children),
    SafeNavigationChain => "Lint/SafeNavigationChain" => source(safe_navigation_chain),
    BlockDelimiters => "Style/BlockDelimiters" => source(block_delimiters),
    RedundantSafeNavigation => "Lint/RedundantSafeNavigation" => source(redundant_safe_navigation),
    AndOr => "Style/AndOr" => source(and_or),
}

fn safe_navigation_consistency(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(safe) = line.find("&.") else {
            continue;
        };
        let chain_end = line[safe + 2..]
            .find(|character: char| {
                character.is_whitespace() || matches!(character, '&' | '|' | ',')
            })
            .map_or(line.len(), |at| safe + 2 + at);
        if let Some(dot) = line[safe + 2..chain_end].find('.').map(|at| safe + 2 + at) {
            context.replace(
                "Use safe navigation consistently.",
                offset + dot..offset + dot + 1,
                offset + dot..offset + dot + 1,
                "&.",
            );
        }
    }
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
        for (offense, edit) in offenses.into_iter().rev() {
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

fn class_module_children(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("nested") != "nested" {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        let keyword = if trimmed.starts_with("class ") {
            "class "
        } else if trimmed.starts_with("module ") {
            "module "
        } else {
            continue;
        };
        let name = trimmed.trim_start_matches(keyword).trim();
        if !name.contains("::") || name.starts_with("::") || name.contains(['<', '(']) {
            continue;
        }
        let indent = line.len() - trimmed.len();
        context.report(
            "Use nested module/class definitions instead of a compact namespace.",
            offset + indent..offset + line.len(),
        );
    }
}

fn safe_navigation_chain(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if let Some(and_at) = line.find(" && ") {
            let receiver = line[..and_at].split_whitespace().last().unwrap_or("");
            let rhs = line[and_at + 4..].trim();
            if !receiver.is_empty() && rhs.starts_with(&format!("{receiver}.")) {
                let dot = offset + and_at + 4 + receiver.len();
                context.replace(
                    "Use safe navigation (`&.`) instead of checking for nil.",
                    offset + and_at..dot + 1,
                    offset + and_at..dot + 1,
                    "&.",
                );
            }
        }
    }
}

fn block_delimiters(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("line_count_based") != "line_count_based" {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        if line.contains(" do ") && line.trim_end().ends_with(" end") {
            let start = offset + line.find(" do ").unwrap_or(0);
            let end = offset + line.rfind(" end").unwrap_or(line.len());
            let body_start = start - offset + 4;
            let body_end = end - offset;
            let body = if body_start <= body_end {
                &line[body_start..body_end]
            } else {
                ""
            };
            let message = "Prefer `{...}` over `do...end` for single-line blocks.";
            if body.is_empty() {
                context.report(message, start + 1..start + 3);
                continue;
            }
            context.replace(
                message,
                start..end + 4,
                start..end + 4,
                format!(" {{ {body} }}"),
            );
        }
    }
}

fn redundant_safe_navigation(context: &mut CopContext<'_, '_>) {
    context.replace_code("self&.", "self.", "Redundant safe navigation detected.");
    context.replace_code("[]&.", "[].", "Redundant safe navigation detected.");
    context.replace_code("{}&.", "{}.", "Redundant safe navigation detected.");
}

fn and_or(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if !["if ", "unless ", "while ", "until "]
            .iter()
            .any(|keyword| line.trim_start().starts_with(keyword))
        {
            continue;
        }
        for (old, new, message) in [
            (" and ", " && ", "Use `&&` instead of `and`."),
            (" or ", " || ", "Use `||` instead of `or`."),
        ] {
            if let Some(at) = line.find(old) {
                context.replace(
                    message,
                    offset + at..offset + at + old.len(),
                    offset + at..offset + at + old.len(),
                    new,
                );
            }
        }
    }
}
