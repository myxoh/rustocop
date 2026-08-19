use ruby_prism::CallNode;

use super::*;

define_rule!(RedundantSortRule);

const MSG: &str = "Use `{suggestion}` instead of `{sorter}...{accessor}`.";

define_cops! {
    RedundantSort => "Style/RedundantSort" => call_rule(RedundantSortRule, on_send),
}

impl RedundantSortRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some((sort_node, sorter, accessor, argument)) = redundant_sort(node) else {
            return;
        };
        self.register_offense(node, &sort_node, sorter, accessor, argument);
    }

    fn register_offense(
        &mut self,
        node: &CallNode<'_>,
        sort_node: &CallNode<'_>,
        sorter: &str,
        accessor: &str,
        argument: Option<i32>,
    ) {
        let Some(sort_selector) = sort_node.message_loc() else {
            return;
        };
        let Some(accessor_selector) = node.message_loc() else {
            return;
        };
        let suggestion = suggestion(sorter, accessor, argument);
        let accessor_source = self
            .source()
            .get(accessor_selector.start_offset()..node.location().end_offset())
            .unwrap_or(accessor);
        let message = MSG
            .replace("{suggestion}", suggestion)
            .replace("{sorter}", sorter)
            .replace("{accessor}", accessor_source);
        let accessor_start = node
            .call_operator_loc()
            .map_or(accessor_selector.start_offset(), |operator| operator.start_offset());
        let offense = sort_selector.start_offset()..node.location().end_offset();
        let logical_operator = self.parent().and_then(|parent| {
            parent
                .as_or_node()
                .map(|logical| logical.operator_loc())
                .or_else(|| parent.as_and_node().map(|logical| logical.operator_loc()))
        }).map(|operator| {
            let source = self.source_file().at(&operator).to_string();
            (operator, source)
        });

        add_offense!(self, offense, message: message, |corrector| {
            corrector.replace(sort_selector, suggestion);
            corrector.remove(accessor_start..node.location().end_offset());
            if let Some((operator, operator_source)) = logical_operator {
                corrector.replace(
                    sort_node.location().end_offset()..sort_node.location().end_offset(),
                    format!(" {operator_source}"),
                );
                corrector.remove(operator);
            }
        });
    }
}

def_node_matcher! {
    fn redundant_sort<'pr>(
        node: &CallNode<'pr>,
    ) -> Option<(CallNode<'pr>, &'static str, &'static str, Option<i32>)> {
        let accessor = match node.name().as_slice() {
            b"first" if argument_count(node) == 0 => "first",
            b"last" if argument_count(node) == 0 => "last",
            b"[]" if argument_count(node) == 1 => "[]",
            b"at" if argument_count(node) == 1 => "at",
            b"slice" if argument_count(node) == 1 => "slice",
            _ => return None,
        };
        let argument = only_argument(node)
            .and_then(|argument| argument.as_integer_node())
            .and_then(|integer| TryInto::<i32>::try_into(integer.value()).ok());
        if matches!(accessor, "[]" | "at" | "slice") && !matches!(argument, Some(0 | -1)) {
            return None;
        }
        let sort_node = node.receiver()?.as_call_node()?;
        let sorter = match sort_node.name().as_slice() {
            b"sort" if argument_count(&sort_node) == 0 => "sort",
            b"sort_by" if sort_node.block().is_some() || argument_count(&sort_node) > 0 => "sort_by",
            _ => return None,
        };
        Some((sort_node, sorter, accessor, argument))
    }
}

fn suggestion(sorter: &str, accessor: &str, argument: Option<i32>) -> &'static str {
    match (sorter, accessor, argument) {
        ("sort", "first", _) | ("sort", _, Some(0)) => "min",
        ("sort", "last", _) | ("sort", _, Some(-1)) => "max",
        ("sort_by", "first", _) | ("sort_by", _, Some(0)) => "min_by",
        ("sort_by", "last", _) | ("sort_by", _, Some(-1)) => "max_by",
        _ => unreachable!("matcher only returns supported accessors"),
    }
}
