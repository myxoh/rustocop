use super::*;

define_cops! {
    SoleNestedConditional => "Style/SoleNestedConditional" => compatibility_prism_any_node(sole_nested_conditional),
}

struct Conditional<'pr> {
    location: ruby_prism::Location<'pr>,
    keyword: ruby_prism::Location<'pr>,
    predicate: Node<'pr>,
    body: Option<Node<'pr>>,
    end_keyword: Option<ruby_prism::Location<'pr>>,
    is_unless: bool,
    modifier: bool,
    has_else: bool,
    is_elsif: bool,
}

impl<'pr> Conditional<'pr> {
    fn from(node: &Node<'pr>, source: &str) -> Option<Self> {
        if let Some(condition) = node.as_if_node() {
            let keyword = condition.if_keyword_loc()?;
            let keyword_source = &source[keyword.start_offset()..keyword.end_offset()];
            let modifier = condition.end_keyword_loc().is_none()
                && keyword.start_offset() != condition.location().start_offset();
            return Some(Self {
                location: condition.location(),
                keyword,
                predicate: condition.predicate(),
                body: only_statement(condition.statements()),
                end_keyword: condition.end_keyword_loc(),
                is_unless: false,
                modifier,
                has_else: condition.subsequent().is_some(),
                is_elsif: keyword_source == "elsif",
            });
        }
        let condition = node.as_unless_node()?;
        let keyword = condition.keyword_loc();
        let modifier = condition.end_keyword_loc().is_none()
            && keyword.start_offset() != condition.location().start_offset();
        Some(Self {
            location: condition.location(),
            keyword,
            predicate: condition.predicate(),
            body: only_statement(condition.statements()),
            end_keyword: condition.end_keyword_loc(),
            is_unless: true,
            modifier,
            has_else: condition.else_clause().is_some(),
            is_elsif: false,
        })
    }
}

fn sole_nested_conditional(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let Some(outer) = Conditional::from(node, context.source()) else {
        return;
    };
    if outer.has_else || outer.is_elsif {
        return;
    }
    let Some(branch) = outer.body.as_ref() else {
        return;
    };
    let Some(inner) = Conditional::from(branch, context.source()) else {
        return;
    };
    if inner.has_else
        || inner.is_elsif
        || (outer.modifier || inner.modifier) && context.config_bool("AllowModifier", false)
        || assigned_condition_reused(&outer.predicate, &inner.predicate, context)
    {
        return;
    }

    let conditional_type = if outer.is_unless { "unless" } else { "if" };
    let message =
        format!("Consider merging nested conditions into outer `{conditional_type}` conditions.");
    let top = context
        .ancestors()
        .iter()
        .find_map(|ancestor| {
            let candidate = Conditional::from(ancestor, context.source())?;
            conditional_chain_contains(&candidate, outer.location.start_offset(), context)
                .then_some(candidate)
        })
        .unwrap_or(outer);
    let (edit, replacement) = merged_conditional_source(&top, context);
    if top.location.start_offset() != node.location().start_offset() {
        context.replace_indirectly(message, inner.keyword, edit, replacement);
    } else {
        context.replace(message, inner.keyword, edit, replacement);
    }
}

fn conditional_chain_contains(
    outer: &Conditional<'_>,
    target_start: usize,
    context: &CopContext<'_, '_>,
) -> bool {
    let Some(node) = outer.body.as_ref() else {
        return false;
    };
    if node.location().start_offset() == target_start {
        return true;
    }
    let Some(next) = Conditional::from(node, context.source()) else {
        return false;
    };
    !next.has_else && !next.is_elsif && conditional_chain_contains(&next, target_start, context)
}

fn merged_conditional_source(
    top: &Conditional<'_>,
    context: &CopContext<'_, '_>,
) -> (std::ops::Range<usize>, String) {
    let mut edits = if top.modifier {
        Vec::new()
    } else {
        correct_header(top, context)
    };
    let mut comments = String::new();
    collect_merge_edits(top, context, &mut edits, &mut comments);
    if !comments.is_empty() {
        edits.push((
            top.keyword.start_offset()..top.keyword.start_offset(),
            comments,
        ));
    }
    edits.sort_by_key(|(range, _)| (range.start, range.end));
    edits.dedup_by(|right, left| left.0 == right.0 && left.1 == right.1);
    let container = top.location.start_offset()
        ..edits
            .iter()
            .map(|(range, _)| range.end)
            .max()
            .unwrap_or(top.location.end_offset())
            .max(top.location.end_offset());
    let mut rendered = context.source()[container.clone()].to_string();
    for (range, replacement) in edits.into_iter().rev() {
        rendered.replace_range(
            range.start - container.start..range.end - container.start,
            &replacement,
        );
    }
    (container, rendered)
}

fn collect_merge_edits(
    outer: &Conditional<'_>,
    context: &CopContext<'_, '_>,
    edits: &mut Vec<(std::ops::Range<usize>, String)>,
    comments: &mut String,
) {
    let Some(branch) = outer.body.as_ref() else {
        return;
    };
    let Some(inner) = Conditional::from(branch, context.source()) else {
        return;
    };
    if inner.has_else
        || inner.is_elsif
        || (outer.modifier || inner.modifier) && context.config_bool("AllowModifier", false)
        || assigned_condition_reused(&outer.predicate, &inner.predicate, context)
    {
        return;
    }
    if outer.modifier {
        edits.extend(correct_header(&inner, context));
        edits.push((
            inner.predicate.location().start_offset()..inner.predicate.location().start_offset(),
            format!("{} && ", chainable(outer, context)),
        ));
        let mut start = outer.keyword.start_offset();
        if start > 0 && context.source().as_bytes()[start - 1].is_ascii_whitespace() {
            start -= 1;
        }
        edits.push((
            start..outer.predicate.location().end_offset(),
            String::new(),
        ));
        return;
    }
    if inner.modifier {
        edits.push((
            outer.predicate.location().end_offset()..outer.predicate.location().end_offset(),
            format!(" && {}", chainable(&inner, context)),
        ));
        let mut start = inner.keyword.start_offset();
        if start > 0 && context.source().as_bytes()[start - 1].is_ascii_whitespace() {
            start -= 1;
        }
        edits.push((
            start..inner.predicate.location().end_offset(),
            String::new(),
        ));
        return;
    }
    comments.push_str(&comments_between(
        outer.predicate.location().end_offset(),
        inner.keyword.start_offset(),
        context.source(),
    ));
    edits.push((
        outer.predicate.location().end_offset()..inner.predicate.location().start_offset(),
        " && ".to_string(),
    ));
    edits.push((
        inner.predicate.location().start_offset()..inner.predicate.location().end_offset(),
        chainable(&inner, context),
    ));
    if let Some(end) = outer.end_keyword.as_ref() {
        let inner_end = inner.end_keyword.as_ref().unwrap_or(&inner.location);
        let removal = if context
            .source_file()
            .same_line(end.start_offset(), inner_end.start_offset())
        {
            end.start_offset()..end.end_offset()
        } else {
            whole_line(end.start_offset(), context.source())
        };
        edits.push((removal, String::new()));
    }
    collect_merge_edits(&inner, context, edits, comments);
}

fn correct_header(
    conditional: &Conditional<'_>,
    context: &CopContext<'_, '_>,
) -> Vec<(std::ops::Range<usize>, String)> {
    let mut edits = Vec::new();
    if conditional.is_unless {
        edits.push((
            conditional.keyword.start_offset()..conditional.keyword.end_offset(),
            "if".to_string(),
        ));
    }
    let replacement = chainable(conditional, context);
    if replacement != context.source_file().node(&conditional.predicate) {
        edits.push((
            conditional.predicate.location().start_offset()
                ..conditional.predicate.location().end_offset(),
            replacement,
        ));
    }
    edits
}

fn chainable(conditional: &Conditional<'_>, context: &CopContext<'_, '_>) -> String {
    let rendered = parenthesized_condition(&conditional.predicate, context);
    if !conditional.is_unless {
        return rendered;
    }
    if conditional.predicate.as_and_node().is_some() {
        format!("!({rendered})")
    } else {
        format!("!{rendered}")
    }
}

fn parenthesized_condition(node: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    let source = context.source_file().node(node);
    if assignment_node(node) || node.as_or_node().is_some() {
        return format!("({source})");
    }
    if let Some(and_node) = node.as_and_node() {
        let left = context.source_file().node(&and_node.left());
        let right = parenthesized_and_clause(&and_node.right(), context);
        let between = &context.source()
            [and_node.left().location().end_offset()..and_node.right().location().start_offset()];
        return format!("{left}{between}{right}");
    }
    let Some(call) = node.as_call_node() else {
        return source.to_string();
    };
    if source.trim_start().starts_with("not ") || call.block().is_some() {
        return format!("({source})");
    }
    let arguments = arguments(&call);
    if arguments.is_empty() {
        return source.to_string();
    }
    if operator_call(call_name(&call)) {
        return format!("({source})");
    }
    if call.opening_loc().is_some() {
        return source.to_string();
    }
    let Some(selector) = call.message_loc() else {
        return format!("({source})");
    };
    let first = arguments[0].location().start_offset();
    let start = node.location().start_offset();
    let end = node.location().end_offset();
    format!(
        "{}({})",
        &context.source()[start..selector.end_offset()],
        &context.source()[first..end]
    )
}

fn parenthesized_and_clause(node: &Node<'_>, context: &CopContext<'_, '_>) -> String {
    if let Some(and_node) = node.as_and_node() {
        let left = context.source_file().node(&and_node.left());
        let right = parenthesized_and_clause(&and_node.right(), context);
        let between = &context.source()
            [and_node.left().location().end_offset()..and_node.right().location().start_offset()];
        format!("{left}{between}{right}")
    } else if assignment_node(node) {
        format!("({})", context.source_file().node(node))
    } else {
        context.source_file().node(node).to_string()
    }
}

fn operator_call(name: &[u8]) -> bool {
    matches!(
        name,
        b"=="
            | b"!="
            | b"<"
            | b">"
            | b"<="
            | b">="
            | b"<=>"
            | b"==="
            | b"=~"
            | b"!~"
            | b"&"
            | b"|"
            | b"^"
            | b"+"
            | b"-"
            | b"*"
            | b"/"
            | b"%"
            | b"**"
            | b"<<"
            | b">>"
            | b"[]"
            | b"!"
    )
}

fn assignment_node(node: &Node<'_>) -> bool {
    node.as_local_variable_write_node().is_some()
        || node.as_instance_variable_write_node().is_some()
        || node.as_class_variable_write_node().is_some()
        || node.as_global_variable_write_node().is_some()
        || node.as_constant_write_node().is_some()
        || node.as_constant_path_write_node().is_some()
        || node.as_multi_write_node().is_some()
        || node.as_local_variable_or_write_node().is_some()
        || node.as_local_variable_and_write_node().is_some()
}

fn assigned_condition_reused(
    outer: &Node<'_>,
    inner: &Node<'_>,
    context: &CopContext<'_, '_>,
) -> bool {
    let mut finder = AssignedNameFinder {
        names: Vec::new(),
        multi_write_depth: 0,
    };
    finder.visit(outer);
    let inner = context.source_file().node(inner).trim();
    finder.names.iter().any(|name| name == inner)
}

struct AssignedNameFinder {
    names: Vec<String>,
    multi_write_depth: usize,
}

impl<'pr> Visit<'pr> for AssignedNameFinder {
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.names
            .push(String::from_utf8_lossy(node.name().as_slice()).into_owned());
        ruby_prism::visit_local_variable_write_node(self, node);
    }

    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode<'pr>) {
        self.multi_write_depth += 1;
        ruby_prism::visit_multi_write_node(self, node);
        self.multi_write_depth -= 1;
    }

    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        if self.multi_write_depth > 0 {
            self.names
                .push(String::from_utf8_lossy(node.name().as_slice()).into_owned());
        }
    }
}

fn whole_line(offset: usize, source: &str) -> std::ops::Range<usize> {
    let start = source[..offset].rfind('\n').map_or(0, |at| at + 1);
    let end = source[offset..]
        .find('\n')
        .map_or(source.len(), |at| offset + at + 1);
    start..end
}

fn comments_between(start: usize, end: usize, source: &str) -> String {
    source[start..end]
        .lines()
        .filter_map(|line| {
            let line = line.trim();
            line.starts_with('#').then(|| format!("{line}\n"))
        })
        .collect()
}
