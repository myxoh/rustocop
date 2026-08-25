use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(ArrayFirstLast),
        Box::new(RedundantArrayFlatten),
        Box::new(RedundantSortBy),
    ]
}

struct ArrayFirstLast;

impl Cop for ArrayFirstLast {
    fn name(&self) -> &'static str {
        "Style/ArrayFirstLast"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        macro_rules! check_index_write {
            ($cast:ident) => {
                if let Some(write) = node.$cast() {
                    let Some(arguments) = write.arguments() else {
                        return;
                    };
                    let arguments = arguments.arguments();
                    if arguments.len() != 1 {
                        return;
                    }
                    let argument = arguments.iter().next().expect("one index argument");
                    let Some(value) = integer_value(&argument) else {
                        return;
                    };
                    let preferred = match value {
                        0 => "first",
                        -1 => "last",
                        _ => return,
                    };
                    let opening = write.opening_loc();
                    let closing = write.closing_loc();
                    let offense = opening.start_offset()..closing.end_offset();
                    context.replace(
                        self.name(),
                        format!("Use `{preferred}`."),
                        offense.clone(),
                        offense,
                        format!(".{preferred}"),
                    );
                    return;
                }
            };
        }
        check_index_write!(as_index_operator_write_node);
        check_index_write!(as_index_or_write_node);
        check_index_write!(as_index_and_write_node);

        let Some(call) = node.as_call_node() else {
            return;
        };
        let Some(argument) = only_argument(&call) else {
            return;
        };
        if call_name(&call) != b"[]" || chained_bracket_call(&call, ancestors) {
            return;
        }
        let Some(value) = integer_value(&argument) else {
            return;
        };
        let preferred = match value {
            0 => "first",
            -1 => "last",
            _ => return,
        };

        let call_location = call.location();
        let (start, replacement) = if let Some(selector) = call.message_loc() {
            if call.call_operator_loc().is_some() {
                (selector.start_offset(), preferred)
            } else {
                (
                    call.receiver().map_or(selector.start_offset(), |receiver| {
                        receiver.location().end_offset()
                    }),
                    if preferred == "first" {
                        ".first"
                    } else {
                        ".last"
                    },
                )
            }
        } else {
            return;
        };
        let offense = start..call_location.end_offset();
        context.replace(
            self.name(),
            format!("Use `{preferred}`."),
            offense.clone(),
            offense,
            replacement,
        );
    }
}

fn integer_value(node: &Node<'_>) -> Option<i32> {
    node.as_integer_node()
        .and_then(|integer| TryInto::<i32>::try_into(integer.value()).ok())
}

fn chained_bracket_call(call: &CallNode<'_>, ancestors: &[Node<'_>]) -> bool {
    if receiver_call(call).is_some_and(|receiver| call_name(&receiver) == b"[]") {
        return true;
    }

    let parent = ancestors
        .iter()
        .rev()
        .find(|ancestor| ancestor.as_arguments_node().is_none());
    parent.is_some_and(|parent| {
        parent.as_call_node().is_some_and(|parent| {
            !parent.is_safe_navigation() && matches!(call_name(&parent), b"[]" | b"[]=")
        })
            || parent.as_index_operator_write_node().is_some()
            || parent.as_index_or_write_node().is_some()
            || parent.as_index_and_write_node().is_some()
    })
}

struct RedundantArrayFlatten;

impl Cop for RedundantArrayFlatten {
    fn name(&self) -> &'static str {
        "Style/RedundantArrayFlatten"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if call_name(node) != b"join" || !join_without_separator(node) {
            return;
        }
        let Some(flatten) = receiver_call(node) else {
            return;
        };
        if call_name(&flatten) != b"flatten"
            || flatten.receiver().is_none()
            || flatten
                .arguments()
                .is_some_and(|arguments| arguments.arguments().len() > 1)
        {
            return;
        }
        let Some(operator) = flatten.call_operator_loc() else {
            return;
        };
        let offense = operator.start_offset()..flatten.location().end_offset();
        context.remove(
            self.name(),
            "Remove the redundant `flatten`.",
            offense.clone(),
            offense,
        );
    }
}

fn join_without_separator(call: &CallNode<'_>) -> bool {
    call.arguments().is_none_or(|arguments| {
        let values = arguments.arguments();
        values.is_empty()
            || values.len() == 1
                && values
                    .first()
                    .is_some_and(|value| value.as_nil_node().is_some())
    })
}

struct RedundantSortBy;

impl Cop for RedundantSortBy {
    fn name(&self) -> &'static str {
        "Style/RedundantSortBy"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if !match_call(node)
            .named(b"sort_by")
            .with_receiver()
            .without_arguments()
            .matches()
        {
            return;
        }
        let Some(block) = node.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        let Some(identity) = identity_block(&block) else {
            return;
        };
        let Some(selector) = node.message_loc() else {
            return;
        };
        let offense = selector.start_offset()..block.location().end_offset();
        let description = match identity {
            IdentityBlock::Named(name) => format!("{{ |{name}| {name} }}"),
            IdentityBlock::Numbered => "{ _1 }".to_string(),
            IdentityBlock::It => "{ it }".to_string(),
        };
        context.replace(
            self.name(),
            format!("Use `sort` instead of `sort_by {description}`."),
            offense.clone(),
            offense,
            "sort",
        );
    }
}

enum IdentityBlock {
    Named(String),
    Numbered,
    It,
}

fn identity_block(block: &ruby_prism::BlockNode<'_>) -> Option<IdentityBlock> {
    let body = single_block_expression(block.body()?)?;
    let parameters = block.parameters()?;

    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return (numbered.maximum() == 1
            && body
                .as_local_variable_read_node()
                .is_some_and(|read| read.name().as_slice() == b"_1"))
        .then_some(IdentityBlock::Numbered);
    }
    if parameters.as_it_parameters_node().is_some() {
        return body
            .as_it_local_variable_read_node()
            .is_some()
            .then_some(IdentityBlock::It);
    }

    let block_parameters = parameters.as_block_parameters_node()?;
    let parameters = block_parameters.parameters()?;
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters
            .rest()
            .is_some_and(|rest| rest.as_implicit_rest_node().is_none())
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    let parameter = parameters
        .requireds()
        .first()?
        .as_required_parameter_node()?;
    let name = parameter.name().as_slice();
    let read = body.as_local_variable_read_node()?;
    (read.name().as_slice() == name)
        .then(|| IdentityBlock::Named(String::from_utf8_lossy(name).into_owned()))
}

fn single_block_expression(node: Node<'_>) -> Option<Node<'_>> {
    if let Some(statements) = node.as_statements_node() {
        let body = statements.body();
        return (body.len() == 1).then(|| body.first()).flatten();
    }
    Some(node)
}
