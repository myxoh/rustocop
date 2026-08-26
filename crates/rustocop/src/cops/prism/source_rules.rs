use std::collections::HashSet;

use super::*;

mod project_files;
use project_files::*;

declare_source_cops! {
    AddRuntimeDependency => "Gemspec/AddRuntimeDependency" => add_runtime_dependency,
    ClassAndModuleCamelCase => "Naming/ClassAndModuleCamelCase" => camel_case,
    ClassMethods => "Style/ClassMethods" => class_methods,
    DuplicateElsifCondition => "Lint/DuplicateElsifCondition" => duplicate_elsif,
    DuplicatedGem => "Bundler/DuplicatedGem" => duplicated_gem,
}

fn add_runtime_dependency(source: &str, context: &mut Reporter<'_>) {
    if !context.path().ends_with("(string)") && !context.path().ends_with(".gemspec") {
        return;
    }
    const METHOD: &str = "add_runtime_dependency";
    for (offset, line) in source_lines(source) {
        let Some(dot) = line.find(&format!(".{METHOD}")) else {
            continue;
        };
        let start = dot + 1;
        let end = start + METHOD.len();
        let after = line[end..].trim_start();
        let has_argument = if let Some(arguments) = after.strip_prefix('(') {
            arguments
                .split_once(')')
                .is_some_and(|(arguments, _)| !arguments.trim().is_empty())
        } else {
            !after.is_empty() && !after.starts_with('#')
        };
        if has_argument {
            context.replace(
                "Use `add_dependency` instead of `add_runtime_dependency`.",
                offset + start..offset + end,
                offset + start..offset + end,
                "add_dependency",
            );
        }
    }
}

fn camel_case(source: &str, context: &mut Reporter<'_>) {
    let file = SourceFile::new(source);
    let class_offsets = file.code_offsets("class");
    let module_offsets = file.code_offsets("module");
    let heredocs = file.heredoc_ranges();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let keyword = if trimmed.starts_with("class ") {
            "class "
        } else if trimmed.starts_with("module ") {
            "module "
        } else {
            continue;
        };
        let leading = line.len() - trimmed.len();
        let lexical = if keyword == "class " {
            &class_offsets
        } else {
            &module_offsets
        };
        let keyword_offset = offset + leading;
        if lexical.binary_search(&keyword_offset).is_err()
            || heredocs
                .iter()
                .any(|heredoc| heredoc.start <= keyword_offset && keyword_offset < heredoc.end)
        {
            continue;
        }
        let name_start = leading + keyword.len();
        let name = line[name_start..]
            .split_whitespace()
            .next()
            .unwrap_or_default();
        let invalid = name.split("::").any(|part| {
            part.contains('_') && !matches!(part, "module_parent" | "getter_class" | "setter_class")
        });
        if invalid {
            context.report(
                "Use CamelCase for classes and modules.",
                offset + name_start..offset + name_start + name.len(),
            );
        }
    }
}

fn class_methods(source: &str, context: &mut Reporter<'_>) {
    let mut owner: Option<String> = None;
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim();
        if let Some(name) = trimmed
            .strip_prefix("class ")
            .or_else(|| trimmed.strip_prefix("module "))
        {
            owner = Some(
                name.split_whitespace()
                    .next()
                    .unwrap_or_default()
                    .to_string(),
            );
            continue;
        }
        let Some(owner_name) = owner.as_deref() else {
            continue;
        };
        let needle = format!("def {owner_name}.");
        if let Some(start) = line.find(&needle) {
            let receiver = offset + start + 4..offset + start + 4 + owner_name.len();
            let method = line[start + needle.len()..]
                .split_whitespace()
                .next()
                .unwrap_or_default()
                .split('(')
                .next()
                .unwrap_or_default();
            context.replace(
                format!("Use `self.{method}` instead of `{owner_name}.{method}`."),
                receiver.clone(),
                receiver,
                "self",
            );
        }
    }
}

fn duplicate_elsif(source: &str, context: &mut Reporter<'_>) {
    struct DuplicateElsifVisitor<'source> {
        source: &'source str,
        offenses: Vec<std::ops::Range<usize>>,
    }
    impl<'pr> Visit<'pr> for DuplicateElsifVisitor<'_> {
        fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
            if node
                .if_keyword_loc()
                .is_some_and(|keyword| keyword.as_slice() == b"if")
            {
                let predicate = node.predicate().location();
                let mut seen = HashSet::from([self.source
                    [predicate.start_offset()..predicate.end_offset()]
                    .to_string()]);
                let mut subsequent = node.subsequent();
                while let Some(elsif) = subsequent.and_then(|branch| branch.as_if_node()) {
                    if !elsif
                        .if_keyword_loc()
                        .is_some_and(|keyword| keyword.as_slice() == b"elsif")
                    {
                        break;
                    }
                    let predicate = elsif.predicate().location();
                    let source = self.source[predicate.start_offset()..predicate.end_offset()]
                        .to_string();
                    if !seen.insert(source) {
                        self.offenses
                            .push(predicate.start_offset()..predicate.end_offset());
                    }
                    subsequent = elsif.subsequent();
                }
            }
            ruby_prism::visit_if_node(self, node);
        }
    }

    let parsed = ruby_prism::parse(source.as_bytes());
    let mut visitor = DuplicateElsifVisitor {
        source,
        offenses: Vec::new(),
    };
    visitor.visit(&parsed.node());
    visitor.offenses.sort_by_key(|range| range.start);
    for range in visitor.offenses {
        context.report("Duplicate `elsif` condition detected.", range);
    }
}

fn source_lines(source: &str) -> impl Iterator<Item = (usize, &str)> {
    source.split_inclusive('\n').scan(0, |offset, line| {
        let start = *offset;
        *offset += line.len();
        Some((start, line.strip_suffix('\n').unwrap_or(line)))
    })
}
