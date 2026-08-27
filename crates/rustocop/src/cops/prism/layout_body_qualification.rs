use super::*;
use crate::rubocop::ast::node::core::NodeRef as RubocopNodeRef;

define_compatibility_rule!(EmptyLinesAroundAttributeAccessorRule);

define_cops! {
    EmptyLinesAroundMethodBody => "Layout/EmptyLinesAroundMethodBody" => compatibility_prism_node(as_def_node, empty_method_body),
    EmptyLinesAroundAttributeAccessor => "Layout/EmptyLinesAroundAttributeAccessor" => compatibility_callbacks(EmptyLinesAroundAttributeAccessorRule, [on_send restrict ["attr_reader", "attr_writer", "attr_accessor", "attr"]]),
    EmptyLinesAroundBlockBody => "Layout/EmptyLinesAroundBlockBody" => compatibility_prism_node(as_block_node, empty_block_body),
    EmptyLinesAroundArguments => "Layout/EmptyLinesAroundArguments" => compatibility_prism_call(empty_around_arguments),
    EmptyLinesAroundExceptionHandlingKeywords => "Layout/EmptyLinesAroundExceptionHandlingKeywords" => compatibility_prism_node(as_begin_node, empty_exception_keywords),
    EmptyLinesAroundClassBody => "Layout/EmptyLinesAroundClassBody" => compatibility_prism_any_node(empty_class_body),
    EmptyLinesAroundModuleBody => "Layout/EmptyLinesAroundModuleBody" => compatibility_prism_node(as_module_node, empty_module_body),
}

fn empty_begin_body(node: &ruby_prism::BeginNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(opening), Some(closing)) = (node.begin_keyword_loc(), node.end_keyword_loc()) else {
        return;
    };
    check_body_edges(
        context,
        line_index(context.source(), opening.end_offset()),
        line_index(context.source(), closing.start_offset()),
        "`begin`",
        "no_empty_lines",
    );
}

fn empty_method_body(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(equal) = node.equal_loc() {
        let Some(body) = node.body() else { return };
        let equal_line = line_index(context.source(), equal.start_offset());
        let body_line = line_index(context.source(), body.location().start_offset());
        if body_line > equal_line + 1 && line(context.source(), equal_line + 1).is_empty() {
            remove_blank_line(
                context,
                equal_line + 1,
                "Extra empty line detected at method body beginning.",
            );
        }
        return;
    }

    let Some(closing) = node.end_keyword_loc() else {
        return;
    };
    let opening_line = node.rparen_loc().map_or_else(
        || line_index(context.source(), node.def_keyword_loc().start_offset()),
        |location| line_index(context.source(), location.end_offset()),
    );
    check_body_edges(
        context,
        opening_line,
        line_index(context.source(), closing.start_offset()),
        "method",
        "no_empty_lines",
    );
}

fn empty_block_body(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    let style = context
        .policy()
        .enforced_style("no_empty_lines")
        .to_string();
    if node.body().is_none() && style == "empty_lines" {
        return;
    }
    check_body_edges(
        context,
        line_index(context.source(), node.opening_loc().end_offset()),
        line_index(context.source(), node.closing_loc().start_offset()),
        "block",
        &style,
    );
}

fn empty_around_arguments(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(arguments) = node.arguments() else {
        return;
    };
    let argument_nodes = arguments.arguments();
    if argument_nodes.is_empty() {
        return;
    }
    let source = context.source();
    let location = node.location();
    if context.line_index(location.start_offset()) == context.line_index(location.end_offset()) {
        return;
    }
    if let (Some(receiver), Some(selector)) = (node.receiver(), node.message_loc()) {
        if context.line_index(receiver.location().end_offset())
            != context.line_index(selector.start_offset())
        {
            return;
        }
    }

    let mut starts = argument_nodes
        .iter()
        .map(|argument| argument.location().start_offset())
        .collect::<Vec<_>>();
    if let Some(closing) = node.closing_loc() {
        starts.push(closing.start_offset());
    }
    for start in starts {
        let whitespace_start = source[..start].trim_end_matches(char::is_whitespace).len();
        if source[whitespace_start..start]
            .bytes()
            .filter(|byte| *byte == b'\n')
            .count()
            > 1
        {
            let blank_line = context.line_index(start) - 1;
            remove_blank_line(context, blank_line, "Empty line detected around arguments.");
        }
    }
}

impl EmptyLinesAroundAttributeAccessorRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: RubocopNodeRef<'_>) {
        return_unless!(node.attribute_accessor());
        return_if!(self.next_line_empty(node.last_line()));
        return_if!(self.next_line_empty_or_enable_directive_comment(node.last_line()));

        let next_line_node = self.next_line_node(node);
        return_unless!(self.require_empty_line(next_line_node));

        let Some(node_range) = self.autocorrect_range(node) else { return; };
        add_offense!(self, node, message: "Add an empty line after attribute accessor.", |corrector| {
            corrector.insert_after(node_range, "\n");
        });
    }

    fn autocorrect_range(&self, node: RubocopNodeRef<'_>) -> Option<CompatibilitySourceRange> {
        let node_range = self.range_help().range_by_whole_lines(self.source_range(node)?, false);
        let next_line = node_range.last_line() + 1;
        if self.next_line_enable_directive_comment(next_line) {
            self.processed_source().comment_at_line(next_line)
                .map(|comment| self.owned_character_range(comment.range.clone()))
        } else {
            Some(self.owned_range(node_range))
        }
    }

    fn next_line_empty_or_enable_directive_comment(&self, line: usize) -> bool {
        self.next_line_empty(line)
            || (self.next_line_enable_directive_comment(line + 1)
                && self.next_line_empty(line + 1))
    }

    fn next_line_enable_directive_comment(&self, line: usize) -> bool {
        self.processed_source().comment_at_line(line)
            .is_some_and(|comment| self.directive_comment_enabled(comment))
    }

    fn next_line_empty(&self, line: usize) -> bool {
        self.processed_source().line(line).is_none_or(|next| next.trim().is_empty())
    }

    fn require_empty_line(&self, node: Option<RubocopNodeRef<'_>>) -> bool {
        let Some(node) = node else {
            return false;
        };
        !self.allow_alias(node) && !self.attribute_or_allowed_method(node)
    }

    fn next_line_node<'ast>(&self, node: RubocopNodeRef<'ast>) -> Option<RubocopNodeRef<'ast>> {
        if node.parent().is_some_and(|parent| parent.kind() == "if") { None } else { node.right_sibling() }
    }

    fn allow_alias(&self, node: RubocopNodeRef<'_>) -> bool {
        self.config_bool("AllowAliasSyntax", true) && node.kind() == "alias"
    }

    fn attribute_or_allowed_method(&self, node: RubocopNodeRef<'_>) -> bool {
        node.send_type()
            && (node.attribute_accessor()
                || node.method_name().is_some_and(|name| self.allowed_methods().allowed_method(name)))
    }
}

fn empty_exception_keywords(node: &ruby_prism::BeginNode<'_>, context: &mut CopContext<'_, '_>) {
    let mut keywords = Vec::new();
    let mut rescue = node.rescue_clause();
    while let Some(clause) = rescue {
        keywords.push((clause.keyword_loc(), "rescue"));
        rescue = clause.subsequent();
    }
    if node.rescue_clause().is_some() {
        if let Some(clause) = node.else_clause() {
            keywords.push((clause.else_keyword_loc(), "else"));
        }
    }
    if let Some(clause) = node.ensure_clause() {
        keywords.push((clause.ensure_keyword_loc(), "ensure"));
    }

    let source = context.source();
    if let Some((location, _)) = keywords.last() {
        let last_line = line(source, line_index(source, location.start_offset()));
        if last_line
            .split(|character: char| !character.is_ascii_alphabetic())
            .any(|token| token == "end")
        {
            return;
        }
    }
    let opening_line = line_index(source, node.location().start_offset());
    for (location, keyword) in keywords {
        let index = line_index(source, location.start_offset());
        if index == opening_line {
            continue;
        }
        if index > 0 && line(source, index - 1).trim().is_empty() {
            remove_blank_line(
                context,
                index - 1,
                format!("Extra empty line detected before the `{keyword}`."),
            );
        }
        if line(source, index + 1).trim().is_empty() {
            remove_blank_line(
                context,
                index + 1,
                format!("Extra empty line detected after the `{keyword}`."),
            );
        }
    }
}

fn empty_class_body(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(class) = node.as_class_node() {
        let opening_line = class.superclass().map_or_else(
            || line_index(context.source(), class.class_keyword_loc().start_offset()),
            |superclass| line_index(context.source(), superclass.location().end_offset()),
        );
        check_container_body(
            context,
            opening_line,
            line_index(context.source(), class.end_keyword_loc().start_offset()),
            "class",
            class.body(),
        );
    } else if let Some(class) = node.as_singleton_class_node() {
        check_container_body(
            context,
            line_index(context.source(), class.class_keyword_loc().start_offset()),
            line_index(context.source(), class.end_keyword_loc().start_offset()),
            "class",
            class.body(),
        );
    }
}

fn empty_module_body(node: &ruby_prism::ModuleNode<'_>, context: &mut CopContext<'_, '_>) {
    check_container_body(
        context,
        line_index(context.source(), node.module_keyword_loc().start_offset()),
        line_index(context.source(), node.end_keyword_loc().start_offset()),
        "module",
        node.body(),
    );
}

fn check_container_body(
    context: &mut CopContext<'_, '_>,
    opening_line: usize,
    closing_line: usize,
    kind: &str,
    body: Option<Node<'_>>,
) {
    let style = context
        .policy()
        .enforced_style("no_empty_lines")
        .to_string();
    if body.is_none() && style != "no_empty_lines" {
        return;
    }
    let children = body_children(body);
    let namespace = children.len() == 1
        && (children[0].as_class_node().is_some() || children[0].as_module_node().is_some());
    match style.as_str() {
        "empty_lines_except_namespace" => check_body_edges(
            context,
            opening_line,
            closing_line,
            kind,
            if namespace {
                "no_empty_lines"
            } else {
                "empty_lines"
            },
        ),
        "empty_lines_special" if namespace => {
            check_body_edges(context, opening_line, closing_line, kind, "no_empty_lines")
        }
        "empty_lines_special" => {
            let Some(first) = children.first() else {
                return;
            };
            if requires_special_empty_line(first) {
                check_beginning(context, opening_line, kind, true);
            } else {
                check_beginning(context, opening_line, kind, false);
                if let Some(required) = children
                    .iter()
                    .find(|candidate| requires_special_empty_line(candidate))
                {
                    check_deferred_empty_line(context, required);
                }
            }
            check_ending(context, closing_line, kind, true);
        }
        _ => check_body_edges(context, opening_line, closing_line, kind, &style),
    }
}

fn body_children(body: Option<Node<'_>>) -> Vec<Node<'_>> {
    let Some(body) = body else { return Vec::new() };
    if let Some(statements) = body.as_statements_node() {
        return statements.body().iter().collect();
    }
    if let Some(begin) = body.as_begin_node() {
        return begin
            .statements()
            .map(|statements| statements.body().iter().collect())
            .unwrap_or_default();
    }
    vec![body]
}

fn requires_special_empty_line(node: &Node<'_>) -> bool {
    node.as_def_node().is_some()
        || node.as_class_node().is_some()
        || node.as_module_node().is_some()
        || node.as_call_node().is_some_and(|call| {
            call.receiver().is_none()
                && matches!(
                    call.name().as_slice(),
                    b"private" | b"protected" | b"public"
                )
        })
}

fn check_deferred_empty_line(context: &mut CopContext<'_, '_>, node: &Node<'_>) {
    let node_line = line_index(context.source(), node.location().start_offset());
    if node_line == 0 {
        return;
    }
    let mut previous = node_line - 1;
    while line(context.source(), previous)
        .trim_start()
        .starts_with('#')
        && previous > 0
    {
        previous -= 1;
    }
    if line(context.source(), previous).is_empty() {
        return;
    }
    let node_type = if node.as_def_node().is_some() {
        "def"
    } else if node.as_class_node().is_some() {
        "class"
    } else if node.as_module_node().is_some() {
        "module"
    } else {
        "send"
    };
    insert_blank_line(
        context,
        previous + 1,
        format!("Empty line missing before first {node_type} definition"),
    );
}

fn check_body_edges(
    context: &mut CopContext<'_, '_>,
    opening_line: usize,
    closing_line: usize,
    kind: &str,
    style: &str,
) {
    if closing_line <= opening_line {
        return;
    }
    match style {
        "no_empty_lines" => {
            check_beginning(context, opening_line, kind, false);
            if closing_line - 1 != opening_line + 1 {
                check_ending(context, closing_line, kind, false);
            }
        }
        "empty_lines" => {
            check_beginning(context, opening_line, kind, true);
            check_ending(context, closing_line, kind, true);
        }
        "beginning_only" => {
            check_beginning(context, opening_line, kind, true);
            check_ending(context, closing_line, kind, false);
        }
        "ending_only" => {
            check_beginning(context, opening_line, kind, false);
            check_ending(context, closing_line, kind, true);
        }
        _ => {}
    }
}

fn check_beginning(
    context: &mut CopContext<'_, '_>,
    opening_line: usize,
    kind: &str,
    require_empty: bool,
) {
    let body_line = opening_line + 1;
    if require_empty && !line(context.source(), body_line).is_empty() {
        insert_blank_line(
            context,
            body_line,
            format!("Empty line missing at {kind} body beginning."),
        );
    } else if !require_empty && line(context.source(), body_line).is_empty() {
        remove_blank_line(
            context,
            body_line,
            format!("Extra empty line detected at {kind} body beginning."),
        );
    }
}

fn check_ending(
    context: &mut CopContext<'_, '_>,
    closing_line: usize,
    kind: &str,
    require_empty: bool,
) {
    if closing_line == 0 {
        return;
    }
    let body_line = closing_line - 1;
    if require_empty && !line(context.source(), body_line).is_empty() {
        insert_blank_line(
            context,
            closing_line,
            format!("Empty line missing at {kind} body end."),
        );
    } else if !require_empty && line(context.source(), body_line).is_empty() {
        remove_blank_line(
            context,
            body_line,
            format!("Extra empty line detected at {kind} body end."),
        );
    }
}

fn remove_blank_line(
    context: &mut CopContext<'_, '_>,
    line_number: usize,
    message: impl Into<String>,
) {
    let start = line_start(context.source(), line_number);
    let end = line_start(context.source(), line_number + 1);
    context.remove(message, start..end, start..end);
}

fn insert_blank_line(
    context: &mut CopContext<'_, '_>,
    line_number: usize,
    message: impl Into<String>,
) {
    let start = line_start(context.source(), line_number);
    context.insert(
        message,
        start..(start + 1).min(context.source().len()),
        start,
        "\n",
    );
}

fn line_index(source: &str, offset: usize) -> usize {
    source[..offset.min(source.len())]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
}

fn line_start(source: &str, line_number: usize) -> usize {
    if line_number == 0 {
        return 0;
    }
    source
        .match_indices('\n')
        .nth(line_number - 1)
        .map_or(source.len(), |(offset, _)| offset + 1)
}

fn line(source: &str, line_number: usize) -> &str {
    let start = line_start(source, line_number);
    let end = source[start..]
        .find('\n')
        .map_or(source.len(), |offset| start + offset);
    source[start..end]
        .strip_suffix('\r')
        .unwrap_or(&source[start..end])
}
