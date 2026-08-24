use super::*;

define_cops! {
    ArgumentAlignment => "Layout/ArgumentAlignment" => call(argument_alignment),
    FirstArrayElementIndentation => "Layout/FirstArrayElementIndentation" => node(as_array_node, first_array_element_indentation),
    IndentationConsistency => "Layout/IndentationConsistency" => node(as_statements_node, indentation_consistency),
    IndentationWidth => "Layout/IndentationWidth" => node(as_statements_node, indentation_width),
}

const ARGUMENT_ALIGNMENT_MESSAGE: &str =
    "Align the arguments of a method call if they span more than one line.";
const FIXED_ARGUMENT_INDENTATION_MESSAGE: &str =
    "Use one level of indentation for arguments following the first line of a multi-line method call.";

fn argument_alignment(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() == b"[]=" {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let mut arguments = arguments.arguments().iter().collect::<Vec<_>>();
    if arguments.len() < 2
        && arguments.first().is_none_or(|argument| {
            argument
                .as_keyword_hash_node()
                .is_none_or(|hash| hash.elements().len() < 2)
        })
    {
        return;
    }

    let fixed = context.policy().enforced_style("with_first_argument")
        == "with_fixed_indentation";
    let mut items = Vec::new();
    if fixed {
        let last = arguments.pop();
        items.extend(arguments);
        if let Some(last) = last {
            if let Some(hash) = last.as_keyword_hash_node() {
                items.extend(hash.elements().iter());
            } else {
                items.push(last);
            }
        }
    } else if let Some(hash) = arguments
        .first()
        .and_then(|argument| argument.as_keyword_hash_node())
    {
        items.extend(hash.elements().iter());
    } else {
        items.extend(arguments);
    }
    items.retain(|item| item.as_assoc_splat_node().is_none());
    if let Some(block_argument) = node
        .block()
        .filter(|block| block.as_block_argument_node().is_some())
    {
        items.push(block_argument);
    }
    let Some(first) = items.first() else { return };

    let file = context.source_file();
    let base_column = if fixed {
        let base = node
            .message_loc()
            .or_else(|| node.opening_loc())
            .unwrap_or_else(|| node.location());
        file.indentation(base.start_offset()).len() + configured_width(context)
    } else {
        display_column(file, first.location().start_offset())
    };
    let message = if fixed {
        FIXED_ARGUMENT_INDENTATION_MESSAGE
    } else {
        ARGUMENT_ALIGNMENT_MESSAGE
    };

    let mut previous_line = None;
    for item in items {
        let location = item.location();
        let indentation = file.indentation(location.start_offset());
        let line_start = file.line_start(location.start_offset());
        if previous_line == Some(line_start) || indentation.end != location.start_offset() {
            continue;
        }
        previous_line = Some(line_start);
        let actual = display_column(file, location.start_offset());
        if actual != base_column {
            let nested = nested_alignment_offense(context, node);
            align_node(context, &item, base_column, message, nested);
        }
    }
}

fn first_array_element_indentation(
    node: &ruby_prism::ArrayNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let (Some(opening), Some(closing)) = (node.opening_loc(), node.closing_loc()) else {
        return;
    };
    let elements = node.elements();
    let first = elements.iter().next();
    let file = context.source_file();
    let style = context
        .policy()
        .enforced_style("special_inside_parentheses")
        .to_string();
    let parenthesis = enclosing_argument_parenthesis(context, opening.start_offset()).or_else(|| {
        if style != "special_inside_parentheses" {
            return None;
        }
        let line_start = file.line_start(opening.start_offset());
        let prefix = &context.source()[line_start..opening.start_offset()];
        let relative = prefix.rfind('(')?;
        let nested = &prefix[relative + 1..];
        (!nested.contains(')') && nested.contains('{') && !nested.contains(','))
            .then_some(line_start + relative)
    });
    let parent_hash_column = parent_hash_key_column(context, opening.start_offset());
    let base_kind;
    let base_column = if let Some(column) = parent_hash_column {
        base_kind = ArrayIndentBase::ParentHashKey;
        column
    } else if style == "align_brackets" {
        base_kind = ArrayIndentBase::OpeningBracket;
        file.column(opening.start_offset())
    } else if style == "special_inside_parentheses" {
        if let Some(parenthesis) = parenthesis {
            base_kind = ArrayIndentBase::Parenthesis;
            file.column(parenthesis) + 1
        } else {
            base_kind = ArrayIndentBase::LineStart;
            file.indentation(opening.start_offset()).len()
        }
    } else {
        base_kind = ArrayIndentBase::LineStart;
        file.indentation(opening.start_offset()).len()
    };

    let first_begins_later = first.as_ref().is_some_and(|first| {
        !file.same_line(opening.start_offset(), first.location().start_offset())
    });
    if let Some(ref first) = first {
        let location = first.location();
        if first_begins_later {
            let indentation = file.indentation(location.start_offset());
            if indentation.end == location.start_offset() {
                let expected = base_column + configured_width(context);
                if file.column(location.start_offset()) != expected {
                    let message = format!(
                        "Use {} spaces for indentation in an array, relative to {}.",
                        configured_width(context),
                        base_kind.description()
                    );
                    align_node(context, first, expected, &message, false);
                }
            }
        }
    }

    if (first.is_none() || first_begins_later)
        && !file.same_line(opening.start_offset(), closing.start_offset())
    {
        let indentation = file.indentation(closing.start_offset());
        if indentation.end == closing.start_offset() && file.column(closing.start_offset()) != base_column {
            context.replace(
                base_kind.closing_message(),
                &closing,
                indentation,
                " ".repeat(base_column),
            );
        }
    }
}

#[derive(Clone, Copy)]
enum ArrayIndentBase {
    OpeningBracket,
    Parenthesis,
    ParentHashKey,
    LineStart,
}

impl ArrayIndentBase {
    fn description(self) -> &'static str {
        match self {
            Self::OpeningBracket => "the position of the opening bracket",
            Self::Parenthesis => "the first position after the preceding left parenthesis",
            Self::ParentHashKey => "the parent hash key",
            Self::LineStart => "the start of the line where the left square bracket is",
        }
    }

    fn closing_message(self) -> &'static str {
        match self {
            Self::OpeningBracket => "Indent the right bracket the same as the left bracket.",
            Self::Parenthesis => {
                "Indent the right bracket the same as the first position after the preceding left parenthesis."
            }
            Self::ParentHashKey => "Indent the right bracket the same as the parent hash key.",
            Self::LineStart => {
                "Indent the right bracket the same as the start of the line where the left bracket is."
            }
        }
    }
}

fn enclosing_argument_parenthesis(
    context: &CopContext<'_, '_>,
    opening_bracket: usize,
) -> Option<usize> {
    let file = context.source_file();
    context.ancestors().iter().rev().find_map(|ancestor| {
        let call = ancestor.as_call_node()?;
        let opening = call.opening_loc()?;
        if file.at(&opening) != "(" {
            return None;
        }
        let direct_argument = call.arguments().is_some_and(|arguments| {
            arguments.arguments().iter().any(|argument| {
                let location = argument.location();
                let nested_call = context.ancestors().iter().any(|ancestor| {
                    let Some(nested) = ancestor.as_call_node() else {
                        return false;
                    };
                    let nested_location = nested.location();
                    nested_location.start_offset() <= opening_bracket
                        && opening_bracket < nested_location.end_offset()
                        && nested_location.start_offset() >= location.start_offset()
                        && nested_location.end_offset() <= location.end_offset()
                        && (nested_location.start_offset() != call.location().start_offset()
                            || nested_location.end_offset() != call.location().end_offset())
                });
                location.start_offset() <= opening_bracket
                    && opening_bracket < location.end_offset()
                    && !nested_call
            })
        });
        (direct_argument
            && file.same_line(opening.start_offset(), opening_bracket)
            && opening.start_offset() < opening_bracket)
            .then_some(opening.start_offset())
    })
}

fn parent_hash_key_column(
    context: &CopContext<'_, '_>,
    opening_bracket: usize,
) -> Option<usize> {
    let file = context.source_file();
    let pair = context.ancestors().iter().rev().find_map(Node::as_assoc_node)?;
    let value = pair.value();
    if value.location().start_offset() != opening_bracket
        || !file.same_line(pair.key().location().start_offset(), opening_bracket)
    {
        return None;
    }
    let elements = context.ancestors().iter().rev().find_map(|ancestor| {
        ancestor
            .as_keyword_hash_node()
            .map(|hash| hash.elements().iter().collect::<Vec<_>>())
            .or_else(|| {
                ancestor
                    .as_hash_node()
                    .map(|hash| hash.elements().iter().collect::<Vec<_>>())
            })
    })?;
    let position = elements.iter().position(|element| {
        element.location().start_offset() == pair.location().start_offset()
    })?;
    let next = elements.get(position + 1)?;
    if file.same_line(value.location().end_offset(), next.location().start_offset()) {
        return None;
    }
    Some(file.column(pair.key().location().start_offset()))
}

fn indentation_consistency(
    statements: &ruby_prism::StatementsNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let file = context.source_file();
    let children = statements.body().iter().collect::<Vec<_>>();
    if children.len() < 2 {
        return;
    }
    let style = context.policy().enforced_style("normal");
    let first_modifier_column = children
        .first()
        .filter(|node| bare_access_modifier(node))
        .and_then(|node| {
            let column = file.column(node.location().start_offset());
            let Some(parent) = context.parent() else {
                return Some(column);
            };
            if parent.as_program_node().is_some() {
                return Some(column);
            }
            if parent.as_block_node().is_some() {
                return context.nearest_call().and_then(|call| {
                    (call.name().as_slice() == b"class_methods"
                        || (call.name().as_slice() == b"new"
                            && call
                                .receiver()
                                .is_some_and(|receiver| file.node(&receiver) == "Struct")))
                    .then_some(column)
                });
            }
            parent
                .as_block_node()
                .is_none()
                .then(|| file.indentation(parent.location().start_offset()).len())
                .filter(|indent| column > *indent)
                .map(|_| column)
        });
    let groups = if style == "indented_internal_methods" {
        split_at_access_modifiers(children)
    } else {
        vec![children
            .into_iter()
            .filter(|node| !bare_access_modifier(node))
            .collect::<Vec<_>>()]
    };

    for group in groups {
        let mut items = group.into_iter();
        let Some(first) = items.next() else { continue };
        let expected = first_modifier_column
            .unwrap_or_else(|| display_column(file, first.location().start_offset()));
        let mut candidates = Vec::new();
        if first_modifier_column.is_some() {
            candidates.push(first);
        }
        candidates.extend(items.filter(|node| {
            let location = node.location();
            file.indentation(location.start_offset()).end == location.start_offset()
        }));
        let mut previous_line = None;
        for item in candidates {
            let location = item.location();
            let line = file.line_start(location.start_offset());
            if previous_line == Some(line) {
                continue;
            }
            previous_line = Some(line);
            let actual = file.column(location.start_offset());
            if actual != expected {
                let nested = nested_consistency_offense(context, statements);
                align_node(
                    context,
                    &item,
                    expected,
                    "Inconsistent indentation detected.",
                    nested,
                );
            }
        }
    }
}

fn split_at_access_modifiers<'pr>(children: Vec<Node<'pr>>) -> Vec<Vec<Node<'pr>>> {
    let mut groups = vec![Vec::new()];
    for child in children {
        if bare_access_modifier(&child) {
            groups.push(Vec::new());
        } else {
            groups.last_mut().expect("one group exists").push(child);
        }
    }
    groups
}

fn bare_access_modifier(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.receiver().is_none()
            && call.arguments().is_none_or(|arguments| arguments.arguments().is_empty())
            && matches!(
                call.name().as_slice(),
                b"public" | b"protected" | b"private" | b"module_function"
            )
    })
}

fn access_modifier_declaration(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call.receiver().is_none()
            && matches!(
                call.name().as_slice(),
                b"public" | b"protected" | b"private" | b"module_function"
            )
            && call.arguments().is_some_and(|arguments| {
                let arguments = arguments.arguments();
                !arguments.is_empty()
                    && arguments
                        .iter()
                        .all(|argument| argument.as_def_node().is_none())
            })
    })
}
