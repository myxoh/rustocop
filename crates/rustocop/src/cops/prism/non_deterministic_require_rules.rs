use super::*;

define_cops! {
    NonDeterministicRequireOrder => "Lint/NonDeterministicRequireOrder" => call(non_deterministic_require_order),
}

const MESSAGE: &str = "Sort files before requiring them.";

fn non_deterministic_require_order(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if context.target_ruby_version().at_least(3, 0) {
        return;
    }
    if call_name(node) == b"each" {
        inspect_each(node, context);
    } else if call_name(node) == b"glob" && root_constant(node.receiver(), b"Dir") {
        inspect_direct_glob(node, context);
    }
}

fn inspect_each(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(glob) = node.receiver().and_then(|receiver| receiver.as_call_node()) else {
        return;
    };
    if !dir_glob(&glob) || !requires_files(node.block()) {
        return;
    }
    let receiver = glob.location();
    let offense = call_without_attached_block(node);
    context.insert(MESSAGE, offense, receiver.end_offset(), ".sort");
}

fn inspect_direct_glob(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(block) = node.block() else {
        return;
    };
    let block_argument = block.as_block_argument_node();
    if !requires_files(Some(block)) {
        return;
    }
    let offense = call_without_attached_block(node);
    if let Some(block_argument) = block_argument {
        let Some(closing) = node.closing_loc() else {
            return;
        };
        let block_location = block_argument.location();
        let comma = context.source()[..block_location.start_offset()]
            .rfind(',')
            .unwrap_or(block_location.start_offset());
        let block_source = context.source_file().at(&block_location);
        context.replace_many(
            MESSAGE,
            offense,
            vec![
                (comma..block_location.end_offset(), String::new()),
                (
                    closing.end_offset()..closing.end_offset(),
                    format!(".sort.each({block_source})"),
                ),
            ],
        );
    } else {
        let end = node.closing_loc().map_or_else(
            || node.message_loc().unwrap().end_offset(),
            |loc| loc.end_offset(),
        );
        context.insert(MESSAGE, offense, end, ".sort.each");
    }
}

fn dir_glob(node: &CallNode<'_>) -> bool {
    root_constant(node.receiver(), b"Dir") && matches!(call_name(node), b"[]" | b"glob")
}

fn call_without_attached_block(node: &CallNode<'_>) -> std::ops::Range<usize> {
    let location = node.location();
    let end = match node.block() {
        Some(block) if block.as_block_node().is_some() => node.closing_loc().map_or_else(
            || node.message_loc().unwrap().end_offset(),
            |loc| loc.end_offset(),
        ),
        _ => location.end_offset(),
    };
    location.start_offset()..end
}

fn requires_files(block: Option<Node<'_>>) -> bool {
    let Some(block) = block else {
        return false;
    };
    if let Some(argument) = block.as_block_argument_node() {
        return argument
            .expression()
            .and_then(|node| node.as_call_node())
            .is_some_and(|call| {
                call_name(&call) == b"method"
                    && only_argument(&call)
                        .and_then(|argument| argument.as_symbol_node())
                        .is_some_and(|symbol| {
                            matches!(symbol.unescaped(), b"require" | b"require_relative")
                        })
            });
    }
    let Some(body) = block.as_block_node().and_then(|block| block.body()) else {
        return false;
    };
    let mut visitor = RequireVisitor::default();
    visitor.visit(&body);
    visitor.found
}

#[derive(Default)]
struct RequireVisitor {
    found: bool,
}

impl<'pr> Visit<'pr> for RequireVisitor {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if node.receiver().is_none() && matches!(call_name(node), b"require" | b"require_relative")
        {
            self.found = true;
        } else {
            ruby_prism::visit_call_node(self, node);
        }
    }
}
