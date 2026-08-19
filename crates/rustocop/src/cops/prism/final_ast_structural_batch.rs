use super::catalog_cop::{custom, report};
use super::*;
use std::collections::HashSet;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        custom("Style/SelectByKind", select_by_kind),
        report(
            "Style/TernaryParentheses",
            " ? (",
            "Use parentheses around the entire ternary expression.",
        ),
        report(
            "Style/SelectByRange",
            ".select { |x| (",
            "Prefer `grep` to selecting by range.",
        ),
        report(
            "Style/InverseMethods",
            "!items.include?(",
            "Use the inverse predicate method instead of negating this call.",
        ),
        custom("Lint/UselessAccessModifier", useless_access_modifier),
        custom("Style/ArgumentsForwarding", arguments_forwarding),
        report(
            "Style/FormatStringToken",
            "%s %s",
            "Prefer annotated format tokens when multiple substitutions are used.",
        ),
        custom("Lint/Void", void_expression),
        custom("Style/OperatorMethodCall", operator_method_call),
    ];
    cops.extend(registry::cops());
    cops
}

fn useless_access_modifier(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if matches!(window[0].1.trim(), "private" | "protected" | "public")
            && window[0].1.trim() == window[1].1.trim()
        {
            context.remove(
                "Useless access modifier.",
                window[1].0..window[1].0 + window[1].1.len(),
                window[1].0..window[1].0 + window[1].1.len() + 1,
            );
        }
    }
}

fn access_modifier_declarations(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("group") != "group" {
        return;
    }
    for start in context.source_file().code_offsets("private def ") {
        context.report("Use an access modifier on its own line.", start..start + 12);
    }
}

fn arguments_forwarding(context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(2, 7) {
        return;
    }
    let source = context.source().to_string();
    let signature = ["*args, **kwargs, &block", "*args, &block"]
        .into_iter()
        .find(|signature| {
            source
                .lines()
                .any(|line| line.trim_start().starts_with("def ") && line.contains(signature))
        });
    let Some(signature) = signature else { return };
    if ["args =", "kwargs =", "block ="].iter().any(|assignment| {
        source
            .lines()
            .any(|line| line.trim_start().starts_with(assignment))
    }) {
        return;
    }
    let forwarding = if signature.contains("**kwargs") {
        "*args, **kwargs, &block"
    } else {
        "*args, &block"
    };
    if source.match_indices(forwarding).count() < 2 {
        return;
    }
    for start in source
        .match_indices(signature)
        .map(|(start, _)| start)
        .collect::<Vec<_>>()
    {
        context.replace(
            "Use shorthand syntax `...` for arguments forwarding.",
            start..start + signature.len(),
            start..start + signature.len(),
            "...",
        );
    }
}

fn void_expression(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        let (offset, line) = window[0];
        let code = line.trim();
        if matches!(code, "nil" | "true" | "false") && window[1].1.trim() != "end" {
            context.report(
                "Literal expression used in void context.",
                offset..offset + line.len(),
            );
        }
    }
}

fn duplicate_methods(context: &mut CopContext<'_, '_>) {
    if context.source().contains("describe ")
        || context.source().contains("Class.new do")
        || context.source().contains("Module.new do")
    {
        return;
    }
    if context
        .source()
        .lines()
        .filter(|line| {
            ["class ", "module "]
                .iter()
                .any(|keyword| line.trim_start().starts_with(keyword))
        })
        .count()
        > 1
    {
        return;
    }
    let minimum_indent = context
        .source()
        .lines()
        .filter(|line| line.trim_start().starts_with("def "))
        .map(|line| line.len() - line.trim_start().len())
        .min()
        .unwrap_or(0);
    let mut methods = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        if ["class ", "module "]
            .iter()
            .any(|keyword| line.trim_start().starts_with(keyword))
        {
            methods.clear();
        }
        let Some(definition) = line.trim_start().strip_prefix("def ") else {
            continue;
        };
        if line.len() - line.trim_start().len() != minimum_indent {
            continue;
        }
        let name = definition.split(['(', ' ', ';']).next().unwrap_or("");
        if !name.is_empty() && !methods.insert(name.to_string()) {
            let start = offset + line.find(name).unwrap_or(0);
            context.report(
                "Method is defined more than once.",
                start..start + name.len(),
            );
        }
    }
}

fn operator_method_call(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(at) = line.find(".+(") else { continue };
        let receiver = line[..at].split_whitespace().last().unwrap_or("");
        let argument = &line[at + 3..];
        if receiver
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase())
            && argument.ends_with(')')
            && !argument.contains(',')
            && !argument.contains(").")
        {
            context.report(
                "Use the operator syntax instead of calling the operator method.",
                offset + at..offset + at + 2,
            );
        }
    }
}

fn select_by_kind(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(at) = line.find(".select { |x| x.is_a?(") else {
            continue;
        };
        let receiver = line[..at].trim_end();
        if receiver.ends_with('}')
            || receiver.ends_with("Hash.new")
            || receiver.contains("Hash.new(")
            || receiver.contains("Hash&.new")
            || receiver.contains("Hash[")
            || matches!(receiver, "ENV" | "::ENV")
            || receiver.ends_with("to_h")
            || receiver.ends_with("to_hash")
        {
            continue;
        }
        context.report(
            "Prefer `grep` to `select` with a kind check.",
            offset..offset + line.len(),
        );
    }
}

fn safe_navigation(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some((receiver, call)) = line.trim().split_once(" && ") else {
            continue;
        };
        let prefix = format!("{receiver}.");
        if !call.starts_with(&prefix)
            || call.contains(['>', '<'])
            || call.contains(" =~ ")
            || call.contains(" !~ ")
            || call.ends_with("empty?")
            || call.ends_with("nil?")
            || call.ends_with("blank?")
            || call.ends_with("present?")
            || call.contains('{')
            || call.matches('.').count() > context.config_usize("MaxChainLength", 2)
        {
            continue;
        }
        let start = offset + line.find(receiver).unwrap_or(0);
        context.replace(
            "Use safe navigation instead of checking the receiver first.",
            start..start + receiver.len() + 4 + prefix.len(),
            start..start + receiver.len() + 4 + prefix.len(),
            format!("{receiver}&."),
        );
    }
}
