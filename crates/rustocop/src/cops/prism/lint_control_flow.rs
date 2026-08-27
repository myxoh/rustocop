use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    Vec::new()
}

struct SafeNavigationWithEmpty;

impl Cop for SafeNavigationWithEmpty {
    fn name(&self) -> &'static str {
        "Lint/SafeNavigationWithEmpty"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        if call_name(&call) != b"empty?"
            || !call.is_safe_navigation()
            || call.arguments().is_some()
            || !ancestors
                .iter()
                .rev()
                .any(|ancestor| conditional_predicate_is(ancestor, node))
        {
            return;
        }
        let Some(receiver) = call.receiver() else {
            return;
        };
        let Some(receiver_call) = receiver.as_call_node() else {
            return;
        };
        if receiver_call.is_safe_navigation() {
            return;
        };
        let receiver = source_at(source, &receiver.location());
        let location = call.location();
        context.replace(
            self.name(),
            "Avoid calling `empty?` with the safe navigation operator in conditionals.",
            &location,
            &location,
            format!("{receiver} && {receiver}.empty?"),
        );
    }
}

fn conditional_predicate_is(ancestor: &Node<'_>, candidate: &Node<'_>) -> bool {
    let predicate = if let Some(if_node) = ancestor.as_if_node() {
        Some(if_node.predicate())
    } else {
        ancestor
            .as_unless_node()
            .map(|unless_node| unless_node.predicate())
    };
    predicate.is_some_and(|predicate| same_location(&predicate, candidate))
}

fn same_location(left: &Node<'_>, right: &Node<'_>) -> bool {
    left.location().start_offset() == right.location().start_offset()
        && left.location().end_offset() == right.location().end_offset()
}

struct Loop;

impl Cop for Loop {
    fn name(&self) -> &'static str {
        "Lint/Loop"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        if let Some(while_node) = node
            .as_while_node()
            .filter(|loop_node| loop_node.is_begin_modifier())
        {
            let Some((opening, closing)) = begin_delimiters(while_node.statements()) else {
                return;
            };
            replace_post_loop(
                self.name(),
                PostLoop {
                    location: node.location(),
                    keyword: while_node.keyword_loc(),
                    opening,
                    closing,
                    predicate: while_node.predicate(),
                    conditional: "unless",
                },
                source,
                context,
            );
        } else if let Some(until_node) = node
            .as_until_node()
            .filter(|loop_node| loop_node.is_begin_modifier())
        {
            let Some((opening, closing)) = begin_delimiters(until_node.statements()) else {
                return;
            };
            replace_post_loop(
                self.name(),
                PostLoop {
                    location: node.location(),
                    keyword: until_node.keyword_loc(),
                    opening,
                    closing,
                    predicate: until_node.predicate(),
                    conditional: "if",
                },
                source,
                context,
            );
        }
    }
}

struct PostLoop<'pr> {
    location: ruby_prism::Location<'pr>,
    keyword: ruby_prism::Location<'pr>,
    opening: ruby_prism::Location<'pr>,
    closing: ruby_prism::Location<'pr>,
    predicate: Node<'pr>,
    conditional: &'static str,
}

fn begin_delimiters(
    statements: Option<ruby_prism::StatementsNode<'_>>,
) -> Option<(ruby_prism::Location<'_>, ruby_prism::Location<'_>)> {
    let begin_node = statements?.body().first()?.as_begin_node()?;
    Some((
        begin_node.begin_keyword_loc()?,
        begin_node.end_keyword_loc()?,
    ))
}

fn replace_post_loop<'pr>(
    cop_name: &'static str,
    post_loop: PostLoop<'pr>,
    source: &str,
    context: &mut Context,
) {
    let closing = post_loop.closing;
    let location = post_loop.location;
    let start = location.start_offset();
    let line_start = source[..start].rfind('\n').map_or(0, |offset| offset + 1);
    let indent = &source[line_start..start];
    let body = &source[post_loop.opening.end_offset()..closing.start_offset()];
    let predicate = source_at(source, &post_loop.predicate.location());
    let conditional = post_loop.conditional;
    let replacement = format!("loop do{body}break {conditional} {predicate}\n{indent}end");
    context.replace(
        cop_name,
        "Use `Kernel#loop` with `break` rather than `begin/end/until`(or `while`).",
        post_loop.keyword,
        location,
        replacement,
    );
}
