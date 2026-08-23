use ruby_prism::{IfNode, Node, UnlessNode};

use super::*;

define_cops! {
    IfUnlessModifier => "Style/IfUnlessModifier" => rubocop_callbacks(IfUnlessModifierRule, [on_if, on_unless]),
}

const MODIFIER_MESSAGE: &str = "Favor modifier `{keyword}` usage when having a single-line body. Another good alternative is the usage of control flow `&&`/`||`.";

impl IfUnlessModifierRule<'_, '_, '_> {
    fn on_if(&mut self, node: &IfNode<'_>) {
        let Some(keyword) = node.if_keyword_loc() else { return };
        return_if!(keyword.as_slice() == b"elsif");
        self.check(
            node.location(), keyword, "if", node.predicate(), node.statements().and_then(|body| only_statement_in(&body)),
            node.end_keyword_loc().is_some(), node.subsequent().is_some(),
        );
    }

    fn on_unless(&mut self, node: &UnlessNode<'_>) {
        self.check(
            node.location(), node.keyword_loc(), "unless", node.predicate(), node.statements().and_then(|body| only_statement_in(&body)),
            node.end_keyword_loc().is_some(), node.else_clause().is_some(),
        );
    }

    #[allow(clippy::too_many_arguments)]
    fn check(&mut self, location: ruby_prism::Location<'_>, keyword: ruby_prism::Location<'_>, kind: &str, condition: Node<'_>, body: Option<Node<'_>>, normal: bool, has_else: bool) {
        let source = self.source_file().at(&location);
        let condition_text = self.source_file().node(&condition);
        let keyword_line = self.source_file().slice(
            self.source_file().line_start(keyword.start_offset())..self.source_file().line_end(keyword.end_offset())
        ).unwrap_or_default();
        return_if!(normal && keyword_line.matches("if ").count() + keyword_line.matches("unless ").count() > 1);
        return_if!(has_else || source.contains("\n#{") || condition_excluded(condition_text));
        return_if!(condition_text.contains("defined?") && defined_argument_unsafe(condition_text, &self.source()[..location.start_offset()]));
        return_if!(self.ancestors().iter().any(|ancestor| ancestor.as_interpolated_string_node().is_some()));
        let Some(body) = body else { return };
        let ast_body_source = self.source_file().node(&body).trim();
        let condition_source = self.source_file().node(&condition).trim();
        return_if!(ast_body_source.contains('\n') || body_excluded(&body, ast_body_source));
        return_if!(normal
            && ast_body_source.contains(" = ")
            && ast_body_source.contains(" ? ")
            && ast_body_source.contains(" : "));
        return_if!(!normal
            && condition_source.contains("defined?")
            && !condition_source.contains("defined?(yield)")
            && !condition_source.contains("defined?(super)"));
        let max = self.related_config_value("Layout/LineLength", "Max").and_then(|max| max.parse().ok()).unwrap_or(120);
        let indent = " ".repeat(self.source_file().column(location.start_offset()));
        if normal {
            return_if!(heredoc_source(ast_body_source));
            return_if!(source.lines().skip(1).any(|line| line.contains('#')));
            return_if!([" = ", " += ", " -= ", " *= ", " /= ", " ||= ", " &&= "]
                .iter()
                .any(|operator| condition_source.contains(operator)));
            return_if!(self.parent().and_then(Node::as_call_node).is_some_and(|parent| parent.receiver().is_some_and(|receiver| receiver.location().start_offset() <= location.start_offset() && location.end_offset() <= receiver.location().end_offset())));
            let mut body_source = if source.contains('\n') { source.lines().nth(1).map(str::trim).filter(|body| !body.is_empty()).unwrap_or(ast_body_source).to_owned() } else { ast_body_source.to_owned() };
            if body_source.ends_with(':') && !body_source.contains('(') {
                if let Some(space) = body_source.find(' ') {
                    body_source = format!("{}({})", &body_source[..space], body_source[space + 1..].trim());
                }
            }
            let comment = source.lines().next().and_then(|line| line.find('#').map(|at| line[at..].trim())).unwrap_or("");
            let mut replacement = format!("{body_source} {kind} {condition_source}");
            if !comment.is_empty() { replacement.push(' '); replacement.push_str(comment); }
            let line_start = self.source_file().line_start(keyword.start_offset());
            let line_end = self.source_file().line_end(location.end_offset());
            let prefix = self.source_file().slice(line_start..keyword.start_offset()).unwrap_or_default();
            let suffix = self.source_file().slice(location.end_offset()..line_end).unwrap_or_default();
            return_if!((!comment.is_empty() && !suffix.trim().is_empty()) || suffix.trim_start().starts_with('#'));
            return_if!(conditional_sibling_shares_line(self.source(), &location));
            return_if!(collection_has_shared_conditional_line(self.ancestors(), self.source_file()));
            let parenthesized = source.trim_start().starts_with('(')
                || semantic_parent(self.ancestors()).is_some_and(parent_requires_parentheses);
            if parenthesized { replacement = format!("({replacement})"); }
            let tab_width = self.related_config_value("Layout/IndentationWidth", "Width")
                .and_then(|width| width.parse().ok()).unwrap_or(2);
            return_if!(visual_width(prefix, tab_width) + replacement.chars().count() + suffix.chars().count() > max);
            let message = MODIFIER_MESSAGE.replace("{keyword}", kind);
            let offense = keyword.start_offset()..keyword.end_offset();
            add_offense!(self, offense, message: message, |corrector| { corrector.replace(location, replacement); });
        } else {
            return_if!(!self.source_file().same_line(location.start_offset(), location.end_offset()));
            return_if!(self.related_config_value("Layout/LineLength", "Enabled") == Some("false"));
            let line_start = self.source_file().line_start(location.start_offset());
            let line_end = self.source_file().line_end(location.end_offset());
            let line = self.source_file().slice(line_start..line_end).unwrap_or_default();
            return_if!(line_length_disabled_at(self.source(), location.start_offset()));
            return_if!(line.contains("# rubocop:disable Layout/LineLength"));
            return_if!(line.chars().count() <= max
                || (line.contains("rubocop:")
                    && self.related_config_value("Layout/LineLength", "AllowCopDirectives") != Some("false")
                    && line.split('#').next().unwrap_or(line).trim_end().chars().count() <= max)
                || ((line.contains("://") || max >= 120 && line.contains("http"))
                    && self.related_config_value("Layout/LineLength", "AllowURI") != Some("false")));
            if let Some(comment_at) = line.find('#') {
                let absolute_comment = line_start + comment_at;
                let comment = line[comment_at..].trim_end();
                if line[..comment_at].trim_end().chars().count() <= max {
                    let message = format!("Modifier form of `{kind}` makes the line too long.");
                    let offense = keyword.start_offset()..keyword.end_offset();
                    add_offense!(self, offense, message: message, |corrector| {
                        corrector.replace(line_start..line_start, format!("{indent}{comment}\n"));
                        corrector.remove(absolute_comment.saturating_sub(1)..line_end);
                    });
                    return;
                }
            }
            let before = self.source_file().slice(line_start..location.start_offset()).unwrap_or_default();
            let after = self.source_file().slice(location.end_offset()..line_end).unwrap_or_default();
            return_if!(after.contains(';') || (before.contains(';') && !before.trim().is_empty()));
            let siblings = line.matches(" if ").count() + line.matches(" unless ").count();
            if siblings > 1 && !condition_source.contains(" if ") && !condition_source.contains(" unless ") {
                self.replace_indirectly(
                    format!("Modifier form of `{kind}` makes the line too long."),
                    keyword,
                    location.start_offset()..location.end_offset(),
                    source,
                );
                return;
            }
            if heredoc_source(ast_body_source) {
                if let Some((edit_end, replacement)) = modifier_heredoc_replacement(self.source(), &location, kind, condition_source, ast_body_source, &indent) {
                    let message = format!("Modifier form of `{kind}` makes the line too long.");
                    let offense = keyword.start_offset()..keyword.end_offset();
                    add_offense!(self, offense, message: message, |corrector| { corrector.replace(location.start_offset()..edit_end, replacement); });
                    return;
                }
            }
            let replacement = format!("{kind} {condition_source}\n{indent}  {ast_body_source}\n{indent}end");
            let message = format!("Modifier form of `{kind}` makes the line too long.");
            let offense = keyword.start_offset()..keyword.end_offset();
            add_offense!(self, offense, message: message, |corrector| { corrector.replace(location, replacement); });
        }
    }
}

fn heredoc_source(source: &str) -> bool {
    source.contains("<<~")
        || source.contains("<<-")
        || source.contains("<<'")
        || source.contains("<<\"")
        || source.contains("<<`")
}

fn condition_excluded(source: &str) -> bool {
    source.contains(" in ") || source.contains(" => ") || source.contains("(?<") || source.contains('\n')
}

fn defined_argument_unsafe(condition: &str, before: &str) -> bool {
    if condition.contains("defined?(yield)") || condition.contains("defined?(super)") { return false; }
    let Some(argument) = condition.split("defined?(").nth(1).and_then(|tail| tail.split(')').next()) else { return true };
    if argument.contains("::") || argument.as_bytes().first().is_some_and(u8::is_ascii_uppercase) {
        return false;
    }
    !before.lines().any(|line| line.trim_start().starts_with(&format!("{argument} =")))
}

fn body_excluded(node: &Node<'_>, source: &str) -> bool {
    node.as_if_node().is_some()
        || node.as_unless_node().is_some()
        || source.starts_with("def ") && source.contains(" = ")
        || source.starts_with("begin")
}

fn semantic_parent<'a>(ancestors: &'a [Node<'a>]) -> Option<&'a Node<'a>> {
    ancestors.iter().rev().find(|node| {
        node.as_statements_node().is_none()
            && node.as_arguments_node().is_none()
            && node.as_program_node().is_none()
    })
}

fn parent_requires_parentheses(parent: &Node<'_>) -> bool {
    parent.as_call_node().is_some()
        || parent.as_and_node().is_some()
        || parent.as_or_node().is_some()
        || parent.as_array_node().is_some()
        || parent.as_assoc_node().is_some()
        || parent.as_local_variable_write_node().is_some()
        || parent.as_instance_variable_write_node().is_some()
        || parent.as_class_variable_write_node().is_some()
        || parent.as_global_variable_write_node().is_some()
        || parent.as_constant_write_node().is_some()
        || parent.as_constant_path_write_node().is_some()
}

fn visual_width(source: &str, tab_width: usize) -> usize {
    source.chars().map(|character| if character == '\t' { tab_width } else { 1 }).sum()
}

fn line_length_disabled_at(source: &str, offset: usize) -> bool {
    let before = &source[..offset];
    let disabled = before.rfind("rubocop:disable Layout/LineLength");
    let enabled = before.rfind("rubocop:enable Layout/LineLength");
    disabled.is_some_and(|disabled| enabled.is_none_or(|enabled| disabled > enabled))
}

fn conditional_sibling_shares_line(source: &str, location: &ruby_prism::Location<'_>) -> bool {
    let line_start = source[..location.start_offset()].rfind('\n').map_or(0, |at| at + 1);
    let line_end = source[location.end_offset()..].find('\n').map_or(source.len(), |at| location.end_offset() + at);
    let before = &source[line_start..location.start_offset()];
    let after = &source[location.end_offset()..line_end];
    (before.contains("end,") || before.contains("end),") || before.contains("end), ("))
        || after.split_once(',').is_some_and(|(_, tail)| {
            let tail = tail.trim_start_matches(|character: char| character.is_whitespace() || character == '(');
            tail.starts_with("if ") || tail.starts_with("unless ")
        })
}

fn collection_has_shared_conditional_line(ancestors: &[Node<'_>], file: SourceFile<'_>) -> bool {
    ancestors.iter().rev().find(|node| {
        node.as_array_node().is_some() || node.as_hash_node().is_some() || node.as_call_node().is_some()
    }).is_some_and(|collection| file.at(&collection.location()).lines().any(|line| {
        let Some(end) = line.find("end") else { return false };
        let tail = &line[end + 3..];
        tail.contains("if ") || tail.contains("unless ")
    }))
}

fn modifier_heredoc_replacement(
    source: &str,
    location: &ruby_prism::Location<'_>,
    kind: &str,
    condition: &str,
    body: &str,
    indent: &str,
) -> Option<(usize, String)> {
    let label = body.split("<<~").nth(1)?.trim_start_matches('`')
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_').next()?;
    let first_line_end = source[location.end_offset()..].find('\n').map_or(location.end_offset(), |at| location.end_offset() + at);
    let mut cursor = first_line_end + usize::from(source.as_bytes().get(first_line_end) == Some(&b'\n'));
    let mut tail = String::new();
    let mut end = None;
    for line in source[cursor..].split_inclusive('\n') {
        tail.push_str(&format!("{indent}  {line}"));
        cursor += line.len();
        if line.trim() == label { end = Some(cursor.saturating_sub(usize::from(line.ends_with('\n')))); break; }
    }
    Some((end?, format!("{kind} {condition}\n{indent}  {body}\n{tail}{indent}end")))
}
