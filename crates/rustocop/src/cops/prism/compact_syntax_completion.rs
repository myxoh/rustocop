use ruby_prism::{BlockNode, CallNode, Node};

use super::*;

define_rule!(WhileUntilModifierRule);

const WHILE_UNTIL_MODIFIER_MSG: &str =
    "Favor modifier `{keyword}` usage when having a single-line body.";

define_cops! {
    FileRead => "Style/FileRead" => call(file_read),
    FileWrite => "Style/FileWrite" => rubocop_callbacks(FileWriteRule, [on_send restrict [b"open"]]),
}

fn file_read(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"open" {
        return;
    }
    let Some(receiver) = node.receiver() else {
        return;
    };
    if !matches!(context.source_file().node(&receiver), "File" | "::File") {
        return;
    }
    let Some(arguments) = node.arguments() else {
        return;
    };
    let arguments = arguments.arguments().iter().collect::<Vec<_>>();
    let (filename, mode) = match arguments.as_slice() {
        [filename] => (filename, "r"),
        [filename, mode] => {
            let Some(mode) = mode.as_string_node() else {
                return;
            };
            let mode = String::from_utf8_lossy(mode.unescaped());
            if !matches!(mode.as_ref(), "r" | "rt" | "rb" | "r+" | "r+t" | "r+b") {
                return;
            }
            (filename, if mode.ends_with('b') { "rb" } else { "r" })
        }
        _ => return,
    };
    let read_method = if mode.ends_with('b') { "binread" } else { "read" };
    let message = format!("Use `File.{read_method}`.");
    let replacement = format!(
        "{read_method}({})",
        context.source_file().node(filename)
    );
    let Some(selector) = node.message_loc() else {
        return;
    };

    if node.block().is_some_and(read_symbol_block_pass) {
        let location = node.location();
        context.replace(
            message,
            &location,
            selector.start_offset()..location.end_offset(),
            replacement,
        );
        return;
    }

    if let Some(block) = node.block().and_then(|block| block.as_block_node()) {
        if !block_reads_parameter(&block) {
            return;
        }
        let offense = node.location().start_offset()..block.closing_loc().end_offset();
        context.replace(
            message,
            offense.clone(),
            selector.start_offset()..offense.end,
            replacement,
        );
        return;
    }

    let Some(read) = context.parent().and_then(Node::as_call_node) else {
        return;
    };
    if call_name(&read) != b"read"
        || argument_count(&read) != 0
        || read.block().is_some()
        || !read.receiver().is_some_and(|receiver| {
            receiver.location().start_offset() == node.location().start_offset()
                && receiver.location().end_offset() == node.location().end_offset()
        })
    {
        return;
    }
    let location = read.location();
    context.replace(
        message,
        &location,
        selector.start_offset()..location.end_offset(),
        replacement,
    );
}

fn read_symbol_block_pass(node: Node<'_>) -> bool {
    node.as_block_argument_node()
        .and_then(|block| block.expression())
        .and_then(|expression| expression.as_symbol_node())
        .is_some_and(|symbol| symbol.unescaped() == b"read")
}

fn block_reads_parameter(block: &BlockNode<'_>) -> bool {
    let Some(parameters) = block
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
        .and_then(|parameters| parameters.parameters())
    else {
        return false;
    };
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return false;
    }
    let Some(parameter) = parameters
        .requireds()
        .first()
        .and_then(|parameter| parameter.as_required_parameter_node())
    else {
        return false;
    };
    block
        .body()
        .and_then(single_expression)
        .and_then(|body| body.as_call_node())
        .is_some_and(|read| {
            call_name(&read) == b"read"
                && argument_count(&read) == 0
                && read.block().is_none()
                && read
                    .receiver()
                    .and_then(|receiver| receiver.as_local_variable_read_node())
                    .is_some_and(|receiver| receiver.name().as_slice() == parameter.name().as_slice())
        })
}

impl FileWriteRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        let Some(receiver) = node.receiver() else { return };
        return_unless!(matches!(self.source_file().node(&receiver), "File" | "::File"));
        let arguments = node
            .arguments()
            .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
            .unwrap_or_default();
        let [filename, mode] = arguments.as_slice() else { return };
        let mode = self.source_file().node(mode).trim_matches(['\'', '"']);
        return_unless!(matches!(mode, "w" | "wt" | "wb" | "w+" | "w+t" | "w+b"));
        let write_method = if mode.ends_with('b') { "binwrite" } else { "write" };

        if let Some(block) = node.block().and_then(|block| block.as_block_node()) {
            self.register_block_write(node, &block, filename, write_method);
            return;
        }
        let Some(write) = self.parent().and_then(Node::as_call_node) else { return };
        return_unless!(write.name().as_slice() == b"write");
        let Some(content) = only_argument(&write) else { return };
        return_if!(content.as_splat_node().is_some());
        let Some(open_selector) = node.message_loc() else { return };
        let replacement = format!(
            "{write_method}({}, {})",
            self.source_file().node(filename),
            self.source_file().node(&content)
        );
        let edit = open_selector.start_offset()..write.location().end_offset();
        add_offense!(self, write.location(), message: format!("Use `File.{write_method}`."), |corrector| {
            corrector.replace(edit, replacement);
        });
    }

    fn register_block_write(
        &mut self,
        node: &CallNode<'_>,
        block: &BlockNode<'_>,
        filename: &Node<'_>,
        write_method: &str,
    ) {
        let Some(parameters) = block.parameters().and_then(|parameters| parameters.as_block_parameters_node()) else { return };
        let parameter = self
            .source_file()
            .slice(parameters.location().start_offset()..parameters.location().end_offset())
            .unwrap_or_default()
            .trim()
            .trim_matches('|')
            .trim();
        return_if!(parameter.is_empty() || parameter.contains(','));
        let Some(write) = block.body().and_then(single_expression).and_then(|body| body.as_call_node()) else { return };
        return_unless!(write.name().as_slice() == b"write");
        let Some(write_receiver) = write.receiver() else { return };
        return_unless!(self.source_file().node(&write_receiver) == parameter);
        let Some(content) = only_argument(&write) else { return };
        return_if!(content.as_splat_node().is_some());
        let Some(open_selector) = node.message_loc() else { return };
        let mut replacement = format!(
            "{write_method}({}, {})",
            self.source_file().node(filename),
            self.source_file().node(&content)
        );
        if let Some(heredoc) = heredoc_tail(self.source(), &content) {
            replacement.push('\n');
            replacement.push_str(heredoc);
        }
        let edit = open_selector.start_offset()..block.closing_loc().end_offset();
        let offense = node.location().start_offset()..block.closing_loc().end_offset();
        add_offense!(self, offense, message: format!("Use `File.{write_method}`."), |corrector| {
            corrector.replace(edit, replacement);
        });
    }
}

fn heredoc_tail<'a>(source: &'a str, content: &Node<'_>) -> Option<&'a str> {
    let header = source.get(content.location().start_offset()..content.location().end_offset())?;
    let marker = header.trim_start().strip_prefix("<<")?.trim_start_matches(['-', '~']);
    let marker = marker
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .next()?;
    let first_newline = source[content.location().start_offset()..].find('\n')?
        + content.location().start_offset();
    let tail = &source[first_newline + 1..];
    let mut consumed = 0;
    for line in tail.split_inclusive('\n') {
        consumed += line.len();
        if line.trim() == marker {
            return Some(tail[..consumed].trim_end_matches('\n'));
        }
    }
    None
}

struct ModifierForm {
    keyword: &'static str,
    offense: std::ops::Range<usize>,
    replacement: String,
}

impl WhileUntilModifierRule<'_, '_, '_> {
    fn on_while(&mut self, node: &Node<'_>) {
        let Some(form) = single_line_as_modifier(node, self) else {
            return;
        };
        let message = WHILE_UNTIL_MODIFIER_MSG.replace("{keyword}", form.keyword);
        add_offense!(self, form.offense, message: message, |corrector| {
            corrector.replace(node.location(), form.replacement);
        });
    }
}

fn single_line_as_modifier(
    node: &Node<'_>,
    context: &CopContext<'_, '_>,
) -> Option<ModifierForm> {
    let parts = if let Some(loop_node) = node.as_while_node() {
        Some((
            "while",
            loop_node.keyword_loc(),
            loop_node.closing_loc(),
            loop_node.predicate(),
            loop_node.statements(),
        ))
    } else {
        node.as_until_node().map(|loop_node| {
            (
            "until",
            loop_node.keyword_loc(),
            loop_node.closing_loc(),
            loop_node.predicate(),
            loop_node.statements(),
            )
        })
    };
    let Some((keyword, keyword_loc, Some(closing), predicate, Some(statements))) = parts else {
        return None;
    };
    let body_nodes = statements.body().iter().collect::<Vec<_>>();
    return_unless!(body_nodes.len() == 1, None);
    let body = &body_nodes[0];
    return_if!(non_eligible_body(body), None);
    let expression = context.source_file().node(node);
    return_if!(
        expression.lines().filter(|line| !line.trim().is_empty()).count() > 3,
        None
    );
    let body_source = context.source_file().node(body);
    return_if!(
        body_source.trim().is_empty()
            || body_source.contains('\n')
            || source_has_comment(body_source),
        None
    );
    let mut assignment = LocalAssignmentVisitor::default();
    assignment.visit(&predicate);
    return_if!(assignment.found, None);

    let source = context.source();
    let first_line_end = source[keyword_loc.start_offset()..]
        .find('\n')
        .map_or(source.len(), |offset| keyword_loc.start_offset() + offset);
    let header_tail = &source[predicate.location().end_offset()..first_line_end];
    let first_line_comment = header_tail
        .find('#')
        .map(|offset| header_tail[offset..].trim_end());
    let last_line_end = source[closing.end_offset()..]
        .find('\n')
        .map_or(source.len(), |offset| closing.end_offset() + offset);
    let code_after = &source[closing.end_offset()..last_line_end];
    return_if!(
        code_after.trim_start().starts_with('#')
            || first_line_comment.is_some() && !code_after.trim().is_empty(),
        None
    );

    let condition = context.source_file().node(&predicate);
    let replacement = to_modifier_form(
        body_source,
        keyword,
        condition,
        parenthesize_modifier(context.parent()),
        first_line_comment,
    );

    let line_start = context.source_file().line_start(keyword_loc.start_offset());
    let code_before = &source[line_start..keyword_loc.start_offset()];
    let line_length_enabled = context
        .related_config_value("Layout/LineLength", "Enabled")
        != Some("false");
    let maximum = context
        .related_config_value("Layout/LineLength", "Max")
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(120);
    return_if!(
        line_length_enabled
            && format!("{code_before}{replacement}{code_after}").chars().count() > maximum,
        None
    );
    Some(ModifierForm {
        keyword,
        offense: keyword_loc.start_offset()..keyword_loc.end_offset(),
        replacement,
    })
}

fn non_eligible_body(node: &Node<'_>) -> bool {
    node.as_if_node().is_some()
        || node.as_unless_node().is_some()
        || node.as_while_node().is_some()
        || node.as_until_node().is_some()
        || node.as_case_node().is_some()
        || node.as_case_match_node().is_some()
}

fn to_modifier_form(
    body: &str,
    keyword: &str,
    condition: &str,
    parenthesize: bool,
    comment: Option<&str>,
) -> String {
    let mut replacement = format!("{body} {keyword} {condition}");
    if parenthesize {
        replacement = format!("({replacement})");
    }
    if let Some(comment) = comment {
        replacement.push(' ');
        replacement.push_str(comment);
    }
    replacement
}

fn source_has_comment(source: &str) -> bool {
    source
        .lines()
        .any(|line| line.trim_start().starts_with('#') || line.contains(" #"))
}

fn parenthesize_modifier(parent: Option<&Node<'_>>) -> bool {
    parent.is_some_and(|parent| {
        parent.as_local_variable_write_node().is_some()
            || parent.as_instance_variable_write_node().is_some()
            || parent.as_class_variable_write_node().is_some()
            || parent.as_constant_write_node().is_some()
            || parent.as_constant_path_write_node().is_some()
            || parent.as_array_node().is_some()
            || parent.as_assoc_node().is_some()
            || parent.as_and_node().is_some()
            || parent.as_or_node().is_some()
            || parent.as_call_node().is_some()
    })
}

#[derive(Default)]
struct LocalAssignmentVisitor {
    found: bool,
}

impl<'pr> Visit<'pr> for LocalAssignmentVisitor {
    fn visit_local_variable_write_node(&mut self, _node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.found = true;
    }

    fn visit_local_variable_and_write_node(
        &mut self,
        _node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        self.found = true;
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        _node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        self.found = true;
    }

    fn visit_local_variable_operator_write_node(
        &mut self,
        _node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        self.found = true;
    }
}
