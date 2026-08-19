use super::*;

define_cops!(
    WhileUntilDo => "Style/WhileUntilDo" => any_node(on_while),
);

fn on_while(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let parts = if let Some(loop_node) = node.as_while_node() {
        loop_node.do_keyword_loc().map(|keyword| {
            (
                "while",
                loop_node.predicate().location().end_offset(),
                keyword,
            )
        })
    } else if let Some(loop_node) = node.as_until_node() {
        loop_node.do_keyword_loc().map(|keyword| {
            (
                "until",
                loop_node.predicate().location().end_offset(),
                keyword,
            )
        })
    } else {
        None
    };
    let Some((keyword_name, predicate_end, do_keyword)) = parts else {
        return;
    };
    if !context.source_file().node(node).contains('\n') {
        return;
    }

    let removal = predicate_end..do_keyword.end_offset();
    let message = format!("Do not use `do` with multi-line `{keyword_name}`.");
    context.add_offense(do_keyword, message, |corrector| {
        corrector.remove(removal);
    });
}
