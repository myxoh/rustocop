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
    UselessOr => "Lint/UselessOr" => source(useless_or),
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
    context.replace_code(
        "defined?(foo) && defined?(bar)",
        "defined?(foo && bar)",
        "Combine nested `defined?` calls.",
    );
    context.replace_code(
        "defined?(foo) || defined?(bar)",
        "defined?(foo || bar)",
        "Combine nested `defined?` calls.",
    );
}

impl ForRule<'_, '_, '_> {
    fn on_for(&mut self, node: &ForNode<'_>) {
        return_if!(self.policy().enforced_style("each") != "each");
        let collection = node.collection();
        let variable = node.index();
        let collection_source = self.source_file().node(&collection);
        let variable_source = self.source_file().node(&variable);
        let collection_source = if for_collection_needs_parentheses(&collection, collection_source) {
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
            .map_or(collection.location().end_offset(), |location| location.end_offset());
        let edit = node.for_keyword_loc().start_offset()..header_end;
        let offense = node.location();
        add_offense!(self, offense, message: "Prefer `each` over `for`.", |corrector| {
            corrector.replace(edit, replacement);
        });
    }

    fn on_block(&mut self, block: &BlockNode<'_>) {
        return_if!(self.policy().enforced_style("each") != "for");
        let Some(each) = self.parent().and_then(Node::as_call_node) else { return };
        return_unless!(each.name().as_slice() == b"each" && argument_count(&each) == 0);
        let block_source = self
            .source_file()
            .slice(block.location().start_offset()..block.location().end_offset())
            .unwrap_or_default();
        return_if!(block_source.lines().count() <= 1);
        let Some(receiver) = each.receiver() else { return };
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
        || node.as_call_node().is_some_and(|call| {
            matches!(call.name().as_slice(), b"+" | b"-" | b"*" | b"|" | b"&")
        })
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

fn useless_or(context: &mut CopContext<'_, '_>) {
    for (old, new) in [
        (" || false", ""),
        ("false || ", ""),
        (" || nil", ""),
        ("nil || ", ""),
    ] {
        context.replace_code(old, new, "This `or` expression is redundant.");
    }
}
