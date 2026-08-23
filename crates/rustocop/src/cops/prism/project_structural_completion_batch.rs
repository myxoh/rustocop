use super::*;

define_cops! {
    RequiredRubyVersion => "Gemspec/RequiredRubyVersion" => source(required_ruby_version),
    ClassStructure => "Layout/ClassStructure" => any_node(class_structure),
    ModuleLength => "Metrics/ModuleLength" => any_node(module_length),
    EmptyLineAfterMultilineCondition => "Layout/EmptyLineAfterMultilineCondition" => any_node(empty_after_multiline_condition),
    DeprecatedOpenSSLConstant => "Lint/DeprecatedOpenSSLConstant" => call(deprecated_openssl),
}

fn required_ruby_version(context: &mut CopContext<'_, '_>) {
    if !context.path().ends_with("(string)") && !context.path().ends_with(".gemspec") {
        return;
    }
    let source = context.source();
    if !source.contains("required_ruby_version") {
        if context.path().ends_with(".gemspec") {
            context.report("`required_ruby_version` should be specified.", 0..0);
        }
        return;
    }
    let target = context.target_ruby_version();
    let target_text = format!("{}.{}", target.major(), target.minor());
    for (offset, line) in context.source_file().lines() {
        if !line.contains("required_ruby_version") {
            continue;
        }
        let assigned = line.split_once('=').map_or("", |(_, value)| value.trim());
        if assigned.starts_with('[') && assigned != "[]" && !assigned.contains(['\'', '"']) {
            continue;
        }
        if requirement_includes_target(line, target.major(), target.minor()) {
            continue;
        }
        if let Some(start) = line
            .find("Gem::Requirement.new")
            .or_else(|| line.find('['))
            .or_else(|| line.find(['\'', '"']))
        {
            let end = if line[start..].starts_with("Gem::Requirement.new") {
                line[start..]
                    .find(')')
                    .map_or(line.len(), |at| start + at + 1)
            } else if line.as_bytes().get(start) == Some(&b'[') {
                line.rfind(']').map_or(line.len(), |at| at + 1)
            } else {
                line[start + 1..]
                    .find(['\'', '"'])
                    .map_or(line.len(), |at| start + at + 2)
            };
            context.report(format!("`required_ruby_version` and `TargetRubyVersion` ({target_text}, which may be specified in .rubocop.yml) should be equal."), offset + start..offset + end);
        }
    }
}

fn requirement_includes_target(line: &str, target_major: u16, target_minor: u16) -> bool {
    let mut rest = line;
    while let Some(open) = rest.find(['\'', '"']) {
        let quote = rest.as_bytes()[open];
        let after = &rest[open + 1..];
        let Some(close) = after.as_bytes().iter().position(|byte| *byte == quote) else {
            break;
        };
        let requirement = after[..close].trim();
        let version = requirement
            .strip_prefix(">=")
            .or_else(|| requirement.strip_prefix("~>"))
            .unwrap_or(requirement)
            .trim();
        if !requirement.starts_with('<') && version_matches(version, target_major, target_minor) {
            return true;
        }
        rest = &after[close + 1..];
    }
    false
}

fn version_matches(version: &str, target_major: u16, target_minor: u16) -> bool {
    let mut components = version.split('.');
    let major = components
        .next()
        .and_then(|value| value.parse::<u16>().ok());
    let minor = components
        .next()
        .and_then(|value| value.parse::<u16>().ok());
    major == Some(target_major) && minor == Some(target_minor)
}

fn class_structure(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (body, class_end) = if let Some(class) = node.as_class_node() {
        (class.body(), class.end_keyword_loc().start_offset())
    } else if let Some(singleton) = node.as_singleton_class_node() {
        (singleton.body(), singleton.end_keyword_loc().start_offset())
    } else {
        return;
    };
    let Some(statements) = body.and_then(|body| body.as_statements_node()) else {
        return;
    };
    let expected = context.config_values("ExpectedOrder").to_vec();
    if expected.is_empty() {
        return;
    }

    struct Element<'pr> {
        node: Node<'pr>,
        category: String,
        correctable: bool,
    }

    let mut visibility = "public";
    let mut elements = Vec::<Element<'_>>::new();
    let mut private_constants = Vec::<String>::new();
    for statement in statements.body().iter() {
        if let Some(call) = statement.as_call_node() {
            if call_name(&call) == b"private_constant" {
                private_constants.extend(structural_call_arguments(&call).into_iter().filter_map(
                    |argument| {
                        argument
                            .as_symbol_node()
                            .map(|symbol| String::from_utf8_lossy(symbol.unescaped()).into_owned())
                    },
                ));
            }
        }
    }

    for statement in statements.body().iter() {
        let mut category = None;
        let mut correctable = true;
        if let Some(definition) = statement.as_def_node() {
            let name = definition.name().as_slice();
            category = Some(if name == b"initialize" {
                "initializer".to_string()
            } else if definition.receiver().is_some() {
                "public_class_methods".to_string()
            } else {
                format!("{visibility}_methods")
            });
        } else if let Some(call) = statement.as_call_node() {
            let method = String::from_utf8_lossy(call_name(&call)).into_owned();
            if matches!(method.as_str(), "public" | "protected" | "private") {
                let arguments = structural_call_arguments(&call);
                if arguments.is_empty() {
                    visibility = Box::leak(method.into_boxed_str());
                    continue;
                }
                if let Some(definition) = arguments.first().and_then(Node::as_def_node) {
                    category = Some(if definition.name().as_slice() == b"initialize" {
                        "initializer".to_string()
                    } else {
                        format!("{method}_methods")
                    });
                } else {
                    for argument in arguments {
                        let Some(symbol) = argument.as_symbol_node() else {
                            continue;
                        };
                        let name = symbol.unescaped();
                        if let Some(element) = elements.iter_mut().rev().find(|element| {
                            element
                                .node
                                .as_def_node()
                                .is_some_and(|definition| definition.name().as_slice() == name)
                        }) {
                            element.category = format!("{method}_methods");
                        }
                    }
                    continue;
                }
            } else if matches!(method.as_str(), "include" | "extend" | "prepend") {
                category = Some("module_inclusion".to_string());
            } else if matches!(
                method.as_str(),
                "attr_accessor" | "attr_reader" | "attr_writer"
            ) {
                let qualified = format!("{visibility}_attribute_macros");
                category = Some(if expected.contains(&qualified) {
                    qualified
                } else {
                    "attribute_macros".to_string()
                });
            } else if method == "delegate" {
                let qualified = format!("{visibility}_delegate");
                category = Some(if expected.contains(&qualified) {
                    qualified
                } else {
                    "delegate".to_string()
                });
            } else if matches!(method.as_str(), "validates" | "validate") {
                category = Some("macros".to_string());
            }
        } else {
            let constant = if let Some(write) = statement.as_constant_write_node() {
                Some((
                    String::from_utf8_lossy(write.name().as_slice()).into_owned(),
                    write.value(),
                ))
            } else if let Some(write) = statement.as_constant_path_write_node() {
                let location = write.target().location();
                Some((
                    context.source()[location.start_offset()..location.end_offset()].to_string(),
                    write.value(),
                ))
            } else {
                None
            };
            if let Some((name, value)) = constant {
                if private_constants.iter().any(|private| private == &name) {
                    continue;
                }
                category = Some(if expected.contains(&"all_constants".to_string()) {
                    "all_constants".to_string()
                } else {
                    "constants".to_string()
                });
                correctable = value.as_call_node().is_none_or(|call| {
                    call_name(&call) == b"freeze"
                        && call.receiver().is_some_and(|receiver| {
                            receiver.as_string_node().is_some()
                                || receiver.as_array_node().is_some()
                                || receiver.as_hash_node().is_some()
                        })
                });
            }
        }
        let Some(category) = category else {
            continue;
        };
        if expected.contains(&category) {
            elements.push(Element {
                node: statement,
                category,
                correctable,
            });
        }
    }

    let mut previous = None::<usize>;
    for (position, element) in elements.iter().enumerate() {
        let index = expected
            .iter()
            .position(|category| category == &element.category)
            .expect("filtered expected category");
        if let Some(previous_index) = previous.filter(|previous| index < *previous) {
            let message = format!(
                "`{}` is supposed to appear before `{}`.",
                element.category, expected[previous_index]
            );
            let location = element.node.location();
            let offense = if element.node.as_constant_write_node().is_some()
                || element.node.as_constant_path_write_node().is_some()
            {
                let line = context.source_file().line_range(location.start_offset());
                let indentation = context.source()[line.clone()].len()
                    - context.source()[line.clone()].trim_start().len();
                let end = line.end.saturating_sub(usize::from(
                    context.source().as_bytes().get(line.end.saturating_sub(1)) == Some(&b'\n'),
                ));
                line.start + indentation..end
            } else {
                location.start_offset()..location.end_offset()
            };
            if element.correctable {
                let inline_modifier_group = elements.len() >= 4
                    && elements.iter().all(|element| {
                        let source = context.source_file().node(&element.node);
                        source.contains("def ") && !source.contains('\n')
                    });
                if inline_modifier_group {
                    let mut ordered = elements.iter().collect::<Vec<_>>();
                    ordered.sort_by_key(|element| {
                        expected
                            .iter()
                            .position(|category| category == &element.category)
                            .unwrap_or(usize::MAX)
                    });
                    let body_start = elements
                        .iter()
                        .map(|element| {
                            class_structure_unit_start(
                                context.source_file(),
                                element.node.location().start_offset(),
                            )
                        })
                        .min()
                        .unwrap_or(class_end);
                    let mut replacement = String::new();
                    for (ordered_index, ordered_element) in ordered.iter().enumerate() {
                        let line = context
                            .source_file()
                            .line_range(ordered_element.node.location().start_offset());
                        replacement.push_str(context.source()[line].trim_end_matches('\n'));
                        let next = ordered.get(ordered_index + 1);
                        let repeated_private = next.is_some_and(|next| {
                            ordered_element.category == "private_methods"
                                && next.category == "private_methods"
                        });
                        replacement.push_str(if repeated_private {
                            "\n\n\n"
                        } else if next.is_none() {
                            "\n\n"
                        } else {
                            "\n"
                        });
                    }
                    context.replace(message, offense, body_start..class_end, replacement);
                    previous = Some(index);
                    continue;
                }
                let previous_element = &elements[position - 1];
                let left_start = class_structure_unit_start(
                    context.source_file(),
                    previous_element.node.location().start_offset(),
                );
                let right_start = class_structure_unit_start(
                    context.source_file(),
                    element.node.location().start_offset(),
                );
                let mut run_end = position + 1;
                while run_end < elements.len()
                    && elements[run_end].category == element.category
                    && elements[run_end].correctable
                {
                    run_end += 1;
                }
                let right_end = if run_end < elements.len() {
                    class_structure_unit_start(
                        context.source_file(),
                        elements[run_end].node.location().start_offset(),
                    )
                } else {
                    class_end
                };
                let left_source = context.source()[left_start..right_start].to_string();
                let right_source = context.source()[right_start..right_end].to_string();
                let replacement = if element.category == "constants" && right_source.contains("<<")
                {
                    format!(
                        "{}\n\n{}\n",
                        right_source.trim_end_matches('\n'),
                        left_source.trim_end_matches('\n')
                    )
                } else {
                    format!("{right_source}{left_source}")
                };
                context.replace_many(message, offense, vec![(left_start..right_end, replacement)]);
            } else {
                context.report(message, offense);
            }
        }
        previous = Some(index);
    }
}

fn class_structure_unit_start(file: SourceFile<'_>, offset: usize) -> usize {
    let mut start = file.line_start(offset);
    while start > 0 {
        let previous_end = start - 1;
        let previous_start = file.line_start(previous_end);
        let previous = file
            .slice(previous_start..previous_end)
            .unwrap_or_default()
            .trim_start();
        if !previous.starts_with('#') {
            break;
        }
        start = previous_start;
    }
    start
}

fn structural_call_arguments<'pr>(call: &CallNode<'pr>) -> Vec<Node<'pr>> {
    call.arguments()
        .map(|arguments| arguments.arguments().iter().collect())
        .unwrap_or_default()
}

fn module_length(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (container, body, offense) = if let Some(module) = node.as_module_node() {
        (module.location(), module.body(), module.location())
    } else {
        let (value, name) = if let Some(write) = node.as_constant_write_node() {
            (write.value(), write.name_loc())
        } else if let Some(write) = node.as_constant_path_write_node() {
            (write.value(), write.target().location())
        } else {
            return;
        };
        let Some(call) = value.as_call_node() else {
            return;
        };
        if call_name(&call) != b"new" || !root_constant(call.receiver(), b"Module") {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        (node.location(), block.body(), name)
    };

    #[derive(Default)]
    struct NestedRanges {
        excluded: Vec<std::ops::Range<usize>>,
        folded: Vec<std::ops::Range<usize>>,
        fold_arrays: bool,
    }

    impl<'pr> Visit<'pr> for NestedRanges {
        fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
            let location = node.location();
            self.excluded
                .push(location.start_offset()..location.end_offset());
        }

        fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
            let location = node.location();
            self.excluded
                .push(location.start_offset()..location.end_offset());
        }

        fn visit_array_node(&mut self, node: &ruby_prism::ArrayNode<'pr>) {
            if self.fold_arrays {
                let location = node.location();
                self.folded
                    .push(location.start_offset()..location.end_offset());
            } else {
                ruby_prism::visit_array_node(self, node);
            }
        }
    }

    let mut ranges = NestedRanges {
        fold_arrays: context
            .config_values("CountAsOne")
            .iter()
            .any(|value| value == "array"),
        ..NestedRanges::default()
    };
    if let Some(body) = body {
        ranges.visit(&body);
    }

    let file = context.source_file();
    let first_line = file.line_range(container.start_offset()).end;
    let last_line = file
        .line_range(container.end_offset().saturating_sub(1))
        .start;
    let count_comments = context.config_bool("CountComments", false);
    let relevant = |offset: usize, line: &str| {
        offset >= first_line
            && offset < last_line
            && !line.trim().is_empty()
            && (count_comments || !line.trim_start().starts_with('#'))
            && !ranges.excluded.iter().any(|range| {
                offset < range.end && offset + line.len().saturating_add(1) > range.start
            })
    };
    let mut count = file
        .lines()
        .filter(|(offset, line)| relevant(*offset, line))
        .count();
    for fold in &ranges.folded {
        if ranges
            .excluded
            .iter()
            .any(|excluded| fold.start >= excluded.start && fold.end <= excluded.end)
        {
            continue;
        }
        let folded_lines = file
            .lines()
            .filter(|(offset, line)| {
                relevant(*offset, line)
                    && *offset < fold.end
                    && *offset + line.len().saturating_add(1) > fold.start
            })
            .count();
        count = count.saturating_sub(folded_lines.saturating_sub(1));
    }

    let max = context.config_usize("Max", 100);
    if count > max {
        context.report(
            format!("Module has too many lines. [{count}/{max}]"),
            offense,
        );
    }
}

fn empty_after_multiline_condition(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(condition) = node.as_if_node() {
        if condition.if_keyword_loc().is_none() {
            return;
        }
        let predicate = condition.predicate();
        let modifier = condition
            .if_keyword_loc()
            .is_some_and(|keyword| keyword.start_offset() != condition.location().start_offset());
        if !modifier || modifier_has_following_statement(node, context) {
            check_multiline_condition(&predicate, predicate_range(&predicate), true, context);
        }
    } else if let Some(condition) = node.as_unless_node() {
        let predicate = condition.predicate();
        let modifier =
            condition.keyword_loc().start_offset() != condition.location().start_offset();
        if !modifier || modifier_has_following_statement(node, context) {
            check_multiline_condition(&predicate, predicate_range(&predicate), true, context);
        }
    } else if let Some(condition) = node.as_while_node() {
        let predicate = condition.predicate();
        if !condition.is_begin_modifier() || modifier_has_following_statement(node, context) {
            check_multiline_condition(&predicate, predicate_range(&predicate), true, context);
        }
    } else if let Some(condition) = node.as_until_node() {
        let predicate = condition.predicate();
        if !condition.is_begin_modifier() || modifier_has_following_statement(node, context) {
            check_multiline_condition(&predicate, predicate_range(&predicate), true, context);
        }
    } else if let Some(branch) = node.as_when_node() {
        let conditions = branch.conditions().iter().collect::<Vec<_>>();
        if let (Some(first), Some(last)) = (conditions.first(), conditions.last()) {
            if !context.source_file().same_line(
                first.location().start_offset(),
                last.location().end_offset(),
            ) {
                let location = branch.location();
                check_multiline_condition(
                    last,
                    location.start_offset()..location.end_offset(),
                    false,
                    context,
                );
            }
        }
    } else if let Some(rescue) = node.as_rescue_node() {
        let exceptions = rescue.exceptions().iter().collect::<Vec<_>>();
        if exceptions.len() > 1 {
            let first = &exceptions[0];
            let last = exceptions.last().expect("multiple rescue exceptions");
            if !context.source_file().same_line(
                first.location().start_offset(),
                last.location().end_offset(),
            ) {
                let start = rescue.keyword_loc().start_offset();
                let end = rescue.statements().map_or_else(
                    || rescue.location().end_offset(),
                    |statements| statements.location().end_offset(),
                );
                check_multiline_condition(last, start..end, false, context);
            }
        }
    }
}

fn modifier_has_following_statement(node: &Node<'_>, context: &CopContext<'_, '_>) -> bool {
    if has_right_sibling(node, context.ancestors()) {
        return true;
    }
    let file = context.source_file();
    let end = node.location().end_offset();
    let line_end = file.line_range(end.saturating_sub(1)).end;
    if !context.source()[end..line_end].trim().is_empty() {
        return false;
    }
    context.source()[node.location().end_offset()..]
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty() && !line.starts_with('#'))
        .is_some_and(|line| {
            !matches!(
                line.split_whitespace().next(),
                Some("end" | "elsif" | "rescue" | "ensure" | "when")
            )
        })
}

fn predicate_range(node: &Node<'_>) -> std::ops::Range<usize> {
    let location = node.location();
    location.start_offset()..location.end_offset()
}

fn check_multiline_condition(
    condition_end: &Node<'_>,
    offense: std::ops::Range<usize>,
    require_multiline_node: bool,
    context: &mut CopContext<'_, '_>,
) {
    let location = condition_end.location();
    let file = context.source_file();
    if require_multiline_node && file.same_line(location.start_offset(), location.end_offset()) {
        return;
    }
    let condition_source = file.node(condition_end);
    let continuation_lines = condition_source.lines().skip(1).filter(|line| !line.trim().is_empty());
    if continuation_lines
        .clone()
        .next()
        .is_some_and(|_| continuation_lines.clone().all(|line| {
            let line = line.trim_start();
            line.starts_with('.') || line.starts_with("&.")
        }))
    {
        return;
    }
    let condition_line = file.line_range(location.end_offset().saturating_sub(1));
    if condition_line.end >= context.source().len()
        || file.line(condition_line.end).trim().is_empty()
    {
        return;
    }
    context.insert(
        "Use empty line after multiline condition.",
        offense,
        condition_line.end,
        "\n",
    );
}

fn has_right_sibling(node: &Node<'_>, ancestors: &[Node<'_>]) -> bool {
    ancestors.iter().rev().any(|ancestor| {
        ancestor.as_statements_node().is_some_and(|statements| {
            let body = statements.body().iter().collect::<Vec<_>>();
            body.iter()
                .position(|sibling| {
                    sibling.location().start_offset() == node.location().start_offset()
                        && sibling.location().end_offset() == node.location().end_offset()
                })
                .is_some_and(|index| index + 1 < body.len())
        })
    })
}

fn deprecated_openssl(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !matches!(call_name(node), b"new" | b"digest") {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    let Some(path) = constant_path(&receiver) else {
        return;
    };
    if path.len() != 3
        || path[0] != b"OpenSSL"
        || !matches!(path[1], b"Cipher" | b"Digest")
        || path[1] == b"Digest" && path[2] == b"Digest"
        || rejected_openssl_argument(node)
    {
        return;
    }

    let algorithm = String::from_utf8_lossy(path[2]);
    let method = String::from_utf8_lossy(call_name(node));
    let replacement_args = if path[1] == b"Cipher" {
        cipher_replacement_args(node, &algorithm, context)
    } else {
        let mut arguments = vec![format!("'{algorithm}'")];
        if let Some(call_arguments) = node.arguments() {
            arguments.extend(
                call_arguments
                    .arguments()
                    .iter()
                    .map(|argument| context.source_file().node(&argument).to_string()),
            );
        }
        arguments.join(", ")
    };
    let parent = String::from_utf8_lossy(path[1]);
    let replacement = format!("OpenSSL::{parent}.{method}({replacement_args})");
    let original = context.source_file().node(&node.as_node());
    context.replace_call(
        node,
        format!("Use `{replacement}` instead of `{original}`."),
        replacement,
    );
}

fn rejected_openssl_argument(node: &CallNode<'_>) -> bool {
    node.arguments().is_some_and(|arguments| {
        arguments.arguments().iter().any(|argument| {
            argument.as_local_variable_read_node().is_some()
                || argument.as_instance_variable_read_node().is_some()
                || argument.as_class_variable_read_node().is_some()
                || argument.as_global_variable_read_node().is_some()
                || argument.as_call_node().is_some()
                || constant_path(&argument).is_some()
        })
    })
}

fn cipher_replacement_args(
    node: &CallNode<'_>,
    algorithm: &str,
    context: &CopContext<'_, '_>,
) -> String {
    if algorithm == "Cipher" {
        return first_argument(node)
            .map(|argument| context.source_file().node(&argument).to_string())
            .unwrap_or_default();
    }

    let no_argument_algorithm = matches!(algorithm, "BF" | "DES" | "IDEA" | "RC4");
    let mut parts = if no_argument_algorithm {
        vec![algorithm.to_lowercase()]
    } else {
        algorithm
            .as_bytes()
            .chunks(3)
            .map(|part| String::from_utf8_lossy(part).to_lowercase())
            .collect::<Vec<_>>()
    };
    let no_arguments = argument_count(node) == 0;
    if let Some(arguments) = node.arguments() {
        for argument in arguments.arguments().iter() {
            let source = if let Some(string) = argument.as_string_node() {
                String::from_utf8_lossy(string.unescaped()).into_owned()
            } else {
                context.source_file().node(&argument).to_string()
            };
            parts.extend(
                source
                    .replace([':', '\''], "")
                    .split('-')
                    .map(str::to_lowercase),
            );
        }
    }
    if no_arguments && !no_argument_algorithm {
        parts.push("cbc".to_string());
    }
    parts.truncate(3);
    format!("'{}'", parts.join("-"))
}
