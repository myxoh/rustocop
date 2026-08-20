use super::*;

define_cops! {
    SafeNavigationConsistency => "Lint/SafeNavigationConsistency" => source(safe_navigation_consistency),
    MissingElse => "Style/MissingElse" => source(missing_else),
    CombinableDefined => "Style/CombinableDefined" => source(combinable_defined),
    For => "Style/For" => source(for_loop),
    ClassAndModuleChildren => "Style/ClassAndModuleChildren" => source(class_module_children),
    SafeNavigationChain => "Lint/SafeNavigationChain" => source(safe_navigation_chain),
    BlockDelimiters => "Style/BlockDelimiters" => source(block_delimiters),
    RedundantSafeNavigation => "Lint/RedundantSafeNavigation" => source(redundant_safe_navigation),
    Next => "Style/Next" => source(next_in_loop),
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

fn missing_else(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(3) {
        if window[0].1.trim_start().starts_with("case ")
            && window[1].1.trim_start().starts_with("when ")
            && window[2].1.trim() == "end"
        {
            context.insert(
                "Do not use `case` without an `else`.",
                window[2].0..window[2].0 + 3,
                window[2].0,
                "else\n  nil\n",
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

fn for_loop(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        let Some(body) = trimmed.strip_prefix("for ") else {
            continue;
        };
        let Some((variable, collection)) = body.split_once(" in ") else {
            continue;
        };
        let indent = line.len() - trimmed.len();
        context.replace(
            "Prefer `each` over `for`.",
            offset + indent..offset + line.len(),
            offset + indent..offset + line.len(),
            format!("{}.each do |{}|", collection.trim(), variable.trim()),
        );
    }
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
            context.replace(
                "Prefer `{...}` over `do...end` for a single-line block.",
                start..end + 4,
                start..end + 4,
                format!(" {{ {} }}", &line[start - offset + 4..end - offset]),
            );
        }
    }
}

fn redundant_safe_navigation(context: &mut CopContext<'_, '_>) {
    context.replace_code(
        "self&.",
        "self.",
        "Redundant safe navigation detected.",
    );
    context.replace_code(
        "[]&.",
        "[].",
        "Redundant safe navigation detected.",
    );
    context.replace_code(
        "{}&.",
        "{}.",
        "Redundant safe navigation detected.",
    );
}

fn next_in_loop(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(3) {
        let condition = window[0].1.trim_start().strip_prefix("if ");
        if let Some(condition) = condition {
            if window[2].1.trim() == "end" && !window[1].1.trim().is_empty() {
                let indent = window[0].1.len() - window[0].1.trim_start().len();
                context.replace(
                    "Use `next` to skip iteration.",
                    window[0].0 + indent..window[2].0 + window[2].1.len(),
                    window[0].0 + indent..window[2].0 + window[2].1.len(),
                    format!("next unless {condition}\n{}", window[1].1),
                );
            }
        }
    }
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
