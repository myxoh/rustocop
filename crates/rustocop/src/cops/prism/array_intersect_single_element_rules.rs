use super::*;

define_cops! {
    ArrayIntersectWithSingleElement => "Style/ArrayIntersectWithSingleElement" => call(array_intersect_with_single_element),
}

fn array_intersect_with_single_element(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"intersect?"
        || node.call_operator_loc().is_some_and(|operator| operator.as_slice() == b"&.")
    {
        return;
    }
    let Some(arguments) = node.arguments() else { return };
    if arguments.arguments().len() != 1 {
        return;
    }
    let Some(array) = arguments.arguments().iter().next().and_then(|argument| argument.as_array_node()) else {
        return;
    };
    if array.elements().len() != 1 {
        return;
    }
    let Some(element) = array.elements().iter().next() else { return };
    if element.as_splat_node().is_some() {
        return;
    }
    let file = context.source_file();
    let array_source = file.at(&array.location());
    let replacement = if array_source.starts_with("%i") {
        let Some(symbol) = element.as_symbol_node() else { return };
        format!(":{}", String::from_utf8_lossy(symbol.unescaped()))
    } else {
        file.node(&element).to_string()
    };
    let selector = node.message_loc().expect("intersect? selector");
    let offense = selector.start_offset()..node.location().end_offset();
    context.replace_many(
        "Use `include?(element)` instead of `intersect?([element])`.",
        offense,
        vec![
            (selector.start_offset()..selector.end_offset(), "include?".to_string()),
            (array.location().start_offset()..array.location().end_offset(), replacement),
        ],
    );
}
