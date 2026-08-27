use std::collections::HashSet;

use super::*;

mod project_files;
use project_files::*;

declare_source_cops! {
    AddRuntimeDependency => "Gemspec/AddRuntimeDependency" => add_runtime_dependency,
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
    struct ClassMethodVisitor<'source> {
        source: &'source str,
        offenses: Vec<(std::ops::Range<usize>, String, String)>,
    }

    impl ClassMethodVisitor<'_> {
        fn check_scope(&mut self, owner: &Node<'_>, body: Option<Node<'_>>) {
            let Some(statements) = body.and_then(|body| body.as_statements_node()) else {
                return;
            };
            let owner_source = source_at(self.source, &owner.location());
            for statement in statements.body().iter() {
                let Some(definition) = statement.as_def_node() else {
                    continue;
                };
                let Some(receiver) = definition.receiver() else {
                    continue;
                };
                if source_at(self.source, &receiver.location()) != owner_source {
                    continue;
                }
                self.offenses.push((
                    receiver.location().start_offset()..receiver.location().end_offset(),
                    owner_source.to_string(),
                    String::from_utf8_lossy(definition.name().as_slice()).into_owned(),
                ));
            }
        }
    }

    impl<'pr> Visit<'pr> for ClassMethodVisitor<'_> {
        fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
            self.check_scope(&node.constant_path(), node.body());
            ruby_prism::visit_class_node(self, node);
        }

        fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
            self.check_scope(&node.constant_path(), node.body());
            ruby_prism::visit_module_node(self, node);
        }
    }

    let parsed = ruby_prism::parse(source.as_bytes());
    let mut visitor = ClassMethodVisitor {
        source,
        offenses: Vec::new(),
    };
    visitor.visit(&parsed.node());
    for (receiver, owner, method) in visitor.offenses {
        context.replace(
            format!("Use `self.{method}` instead of `{owner}.{method}`."),
            receiver.clone(),
            receiver,
            "self",
        );
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
