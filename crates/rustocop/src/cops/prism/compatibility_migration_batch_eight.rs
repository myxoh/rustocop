use super::*;
use crate::rubocop::ast::node::core::NodeRef;
use std::collections::HashSet;

define_cops! {
    SpaceBeforeSemicolonCompatibility => "Layout/SpaceBeforeSemicolon" => compatibility_investigation(SpaceBeforeSemicolonRule, on_new_investigation),
    SpaceBeforeCommaCompatibility => "Layout/SpaceBeforeComma" => compatibility_investigation(SpaceBeforeCommaRule, on_new_investigation),
    SpaceAfterCommaCompatibility => "Layout/SpaceAfterComma" => compatibility_investigation(SpaceAfterCommaRule, on_new_investigation),
    SpaceAfterSemicolonCompatibility => "Layout/SpaceAfterSemicolon" => compatibility_investigation(SpaceAfterSemicolonRule, on_new_investigation),
    ItWithoutArgumentsInBlockCompatibility => "Lint/ItWithoutArgumentsInBlock" => compatibility_investigation(ItWithoutArgumentsInBlockRule, on_new_investigation),
    DuplicateRequireCompatibility => "Lint/DuplicateRequire" => compatibility_investigation(DuplicateRequireRule, on_new_investigation),
    RedundantHeredocDelimiterQuotesCompatibility => "Style/RedundantHeredocDelimiterQuotes" => compatibility_callbacks(RedundantHeredocDelimiterQuotesRule, [on_str, on_dstr]),
    BlockCommentsCompatibility => "Style/BlockComments" => compatibility_investigation(BlockCommentsRule, on_new_investigation),
    EncodingCompatibility => "Style/Encoding" => compatibility_investigation(EncodingRule, on_new_investigation),
    DuplicateMagicCommentCompatibility => "Lint/DuplicateMagicComment" => compatibility_investigation(DuplicateMagicCommentRule, on_new_investigation),
    RedundantSortByCompatibility => "Style/RedundantSortBy" => compatibility_callbacks(RedundantSortByRule, [on_block]),
    StabbyLambdaParenthesesCompatibility => "Style/StabbyLambdaParentheses" => compatibility_callbacks(StabbyLambdaParenthesesRule, [on_send]),
    NestedParenthesizedCallsCompatibility => "Style/NestedParenthesizedCalls" => compatibility_callbacks(NestedParenthesizedCallsRule, [on_send]),
    UselessNumericOperationCompatibility => "Lint/UselessNumericOperation" => compatibility_callbacks(UselessNumericOperationRule, [on_send, on_op_asgn]),
    RedundantRequireStatementCompatibility => "Lint/RedundantRequireStatement" => compatibility_callbacks(RedundantRequireStatementRule, [on_send restrict ["require"]]),
    RedundantWithObjectCompatibility => "Lint/RedundantWithObject" => compatibility_callbacks(RedundantWithObjectRule, [on_block]),
    RedundantWithIndexCompatibility => "Lint/RedundantWithIndex" => compatibility_callbacks(RedundantWithIndexRule, [on_block]),
    FileNullCompatibility => "Style/FileNull" => compatibility_callbacks(FileNullRule, [on_str]),
    AsciiIdentifiersCompatibility => "Naming/AsciiIdentifiers" => compatibility_investigation(AsciiIdentifiersRule, on_new_investigation),
    IpAddressesCompatibility => "Style/IpAddresses" => compatibility_callbacks(IpAddressesRule, [on_str]),
    RedundantConditionalCompatibility => "Style/RedundantConditional" => compatibility_callbacks(RedundantConditionalRule, [on_if]),
    EachForSimpleLoopCompatibility => "Style/EachForSimpleLoop" => compatibility_callbacks(EachForSimpleLoopRule, [on_block]),
    DigChainCompatibility => "Style/DigChain" => compatibility_callbacks(DigChainRule, [on_send restrict ["dig"]]),
    MinMaxComparisonCompatibility => "Style/MinMaxComparison" => compatibility_callbacks(MinMaxComparisonRule, [on_if]),
    YodaExpressionCompatibility => "Style/YodaExpression" => compatibility_callbacks(YodaExpressionRule, [on_send]),
    StructInheritanceCompatibility => "Style/StructInheritance" => compatibility_callbacks(StructInheritanceRule, [on_class]),
    DateTimeCompatibility => "Style/DateTime" => compatibility_callbacks(DateTimeRule, [on_send]),
    ConcatArrayLiteralsCompatibility => "Style/ConcatArrayLiterals" => compatibility_callbacks(ConcatArrayLiteralsRule, [on_send restrict ["concat"]]),
    ObjectThenCompatibility => "Style/ObjectThen" => compatibility_callbacks(ObjectThenRule, [on_block, on_send]),
    SingleLineDoEndBlockCompatibility => "Style/SingleLineDoEndBlock" => compatibility_callbacks(SingleLineDoEndBlockRule, [on_block]),
}

fn check_space_before(context: &mut CompatibilityCopContext<'_, '_, '_>, punctuation: &str, kind: &str) {
    let bytes = context.source().as_bytes();
    let ignored = super::source_rules_layout::ignored_syntax_ranges(context.source());
    let punctuation = punctuation.as_bytes()[0];
    for index in 1..bytes.len() {
        if bytes[index] != punctuation || bytes[index - 1] != b' ' || ignored.iter().any(|range| range.start <= index && index < range.end) { continue; }
        let line_start = context.source()[..index].rfind('\n').map_or(0, |offset| offset + 1);
        if context.source()[line_start..index].trim().is_empty() { continue; }
        let start = context.source()[..index].trim_end_matches(' ').len();
        if punctuation == b';' && bytes.get(start.wrapping_sub(1)) == Some(&b'{') && context.related_config_value("Layout/SpaceInsideBlockBraces", "EnforcedStyle") == Some("space") { continue; }
        let Some(start) = context.source_buffer().character_position(start) else { continue; }; let Some(index) = context.source_buffer().character_position(index) else { continue; }; let range = context.range_between(start, index);
        context.add_offense(range, format!("Space found before {kind}."), |corrector| corrector.remove(range));
    }
}

fn check_space_after(context: &mut CompatibilityCopContext<'_, '_, '_>, punctuation: &str, kind: &str, brace_config: &str, _skip_double_semicolon: bool) {
    let bytes = context.source().as_bytes();
    let ignored = super::source_rules_layout::ignored_syntax_ranges(context.source());
    let interpolation_closings = super::source_rules_layout::interpolation_closing_offsets(context.source());
    let punctuation = punctuation.as_bytes()[0];
    for index in 0..bytes.len() {
        if bytes[index] != punctuation || ignored.iter().any(|range| range.start <= index && index < range.end) { continue; }
        let Some(next) = bytes.get(index + 1).copied() else { continue; };
        if matches!(next, b'\n' | b' ' | b'\t' | b';' | b',' | b')' | b']' | b'|') { continue; }
        if next == b'}' && (punctuation == b';' && interpolation_closings.contains(&(index + 1)) || context.related_config_value(brace_config, "EnforcedStyle").unwrap_or("space") == "no_space") { continue; }
        let Some(begin) = context.source_buffer().character_position(index) else { continue; }; let Some(end) = context.source_buffer().character_position(index + 1) else { continue; }; let token = context.range_between(begin, end);
        context.add_offense(token, format!("Space missing after {kind}."), |corrector| corrector.insert_after(token, " "));
    }
}

fn whole_line_range(context: &CompatibilityCopContext<'_, '_, '_>, start: usize, end: usize) -> CompatibilitySourceRange {
    let characters = context.source().chars().collect::<Vec<_>>();
    let begin = (0..start).rev().find(|&index| characters.get(index) == Some(&'\n')).map_or(0, |index| index + 1);
    let finish = (end..characters.len()).find(|&index| characters.get(index) == Some(&'\n')).map_or(characters.len(), |index| index + 1);
    context.range_between(begin, finish)
}

define_compatibility_rule!(SpaceBeforeSemicolonRule);
impl SpaceBeforeSemicolonRule<'_, '_, '_, '_> { fn on_new_investigation(&mut self) { check_space_before(self, ";", "semicolon"); } }
define_compatibility_rule!(SpaceBeforeCommaRule);
impl SpaceBeforeCommaRule<'_, '_, '_, '_> { fn on_new_investigation(&mut self) { check_space_before(self, ",", "comma"); } }
define_compatibility_rule!(SpaceAfterCommaRule);
impl SpaceAfterCommaRule<'_, '_, '_, '_> { fn on_new_investigation(&mut self) { check_space_after(self, ",", "comma", "Layout/SpaceInsideHashLiteralBraces", false); } }
define_compatibility_rule!(SpaceAfterSemicolonRule);
impl SpaceAfterSemicolonRule<'_, '_, '_, '_> { fn on_new_investigation(&mut self) { check_space_after(self, ";", "semicolon", "Layout/SpaceInsideBlockBraces", true); } }

define_compatibility_rule!(ItWithoutArgumentsInBlockRule);
impl ItWithoutArgumentsInBlockRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        if self.target_ruby_version().at_least(3, 4) { return; }
        let Some(root) = self.processed_source().ast() else { return; };
        let blocks = root.each_node(&["block", "numblock", "itblock"]).into_iter().filter(|block| block.arguments_node().is_none_or(|arguments| arguments.source_range().is_none())).filter_map(|block| block.source_range().map(|range| (block, range))).collect::<Vec<_>>();
        let assignments = root.each_node(&["lvasgn"]).into_iter().filter(|node| node.name() == Some("it")).filter_map(NodeRef::source_range).collect::<Vec<_>>();
        let tokens = self.processed_source().tokens().iter().filter(|token| token.text == "it").cloned().collect::<Vec<_>>();
        let characters = self.source().chars().collect::<Vec<_>>();
        for token in tokens {
            let Some((_, block_range)) = blocks.iter().filter(|(_, range)| range.start <= token.begin_pos() && token.end_pos() <= range.end).min_by_key(|(_, range)| range.end - range.start) else { continue; };
            if assignments.iter().any(|assignment| assignment.start < block_range.start || block_range.start <= assignment.start && assignment.start <= token.begin_pos()) { continue; }
            let previous = characters[..token.begin_pos()].iter().rev().find(|character| !character.is_whitespace()).copied();
            let following = characters[token.end_pos()..].iter().skip_while(|character| character.is_whitespace()).take(2).collect::<String>();
            if matches!(previous, Some('.' | '&')) || following.starts_with('(') || following.starts_with('{') || following.starts_with("do") { continue; }
            self.report("`it` calls without arguments will refer to the first block param in Ruby 3.4; use `it()` or `self.it`.", &token);
        }
    }
}

define_compatibility_rule!(DuplicateRequireRule);
impl DuplicateRequireRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let Some(root) = self.processed_source().ast() else { return; };
        let mut required = HashSet::new();
        for node in root.each_node(&[]) {
            let Some(method @ ("require" | "require_relative")) = node.method_name() else { continue; };
            if node.receiver().is_some_and(|receiver| !(receiver.kind() == "const" && receiver.short_name() == Some("Kernel"))) { continue; }
            let Some(argument) = node.first_argument() else { continue; };
            let parent = node.parent().and_then(NodeRef::source_range).map(|range| (range.start, range.end));
            let key = (parent, method.to_owned(), argument.source().unwrap_or_default().to_owned());
            if required.insert(key) { continue; }
            let whole = node.source_range().map(|range| whole_line_range(self, range.start, range.end));
            add_offense!(self, node, message: format!("Duplicate `{method}` detected."), |corrector| { if let Some(range) = whole { corrector.remove(range); } });
        }
    }
}

define_compatibility_rule!(RedundantHeredocDelimiterQuotesRule);
impl RedundantHeredocDelimiterQuotesRule<'_, '_, '_, '_> {
    fn on_str(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_dstr(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let (Some(opening), Some(body), Some(ending)) = (node.source_range().map(|range| self.owned_character_range(range)), self.location_range(node, "heredoc_body"), self.location_range(node, "heredoc_end")) else { return; };
        let source = self.range_source(&opening); let body_source = self.range_source(&body); let delimiter = self.range_source(&ending).trim();
        if !source.starts_with("<<") { return; }
        let prefix_len = if source.starts_with("<<~") || source.starts_with("<<-") { 3 } else if source.starts_with("<<") { 2 } else { return; };
        let quoted = &source[prefix_len..]; if !(quoted.starts_with('\'') || quoted.starts_with('"')) || body_source.contains('\\') || body_source.contains("#{") || body_source.contains("#@") || body_source.contains("#$") || delimiter.chars().any(|character| !character.is_alphanumeric() && character != '_') { return; }
        let replacement = format!("{}{}", &source[..prefix_len], quoted.trim_matches(['\'', '"']));
        add_offense!(self, opening, message: format!("Remove the redundant heredoc delimiter quotes, use `{replacement}` instead."), |corrector| { corrector.replace(opening, replacement); });
    }
}

define_compatibility_rule!(BlockCommentsRule);
impl BlockCommentsRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let comments = self.processed_source().comments().iter().filter(|comment| comment.embedded_document).cloned().collect::<Vec<_>>();
        for comment in comments {
            let expr = comment.range.clone();
            let begin = self.range_between(expr.start, (expr.start + 7).min(expr.end));
            let end_start = if comment.text.ends_with('\n') { expr.end.saturating_sub(5) } else { expr.end.saturating_sub(8) };
            let end_finish = if comment.text.ends_with('\n') { expr.end } else { expr.end.saturating_sub(2) };
            let ending = self.range_between(end_start, end_finish);
            let contents = self.range_between(begin.end_pos(), ending.begin_pos());
            let source = self.range_source(&contents);
            let replacement = if source.is_empty() { String::new() } else { source.replace("\n\n", "\n#\n").split_inclusive('\n').enumerate().map(|(index, line)| if index > 0 && line.starts_with('#') { line.to_owned() } else { format!("# {line}") }).collect::<String>() };
            add_offense!(self, &comment, message: "Do not use block comments.", |corrector| { corrector.remove(begin); if !source.is_empty() { corrector.replace(contents, replacement); } corrector.remove(ending); });
        }
    }
}

define_compatibility_rule!(EncodingRule);
impl EncodingRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let mut offset = 0usize;
        for line in self.source().split_inclusive('\n') {
            let bare = line.trim_end_matches('\n');
            if bare.starts_with("#!") { offset += line.chars().count(); continue; }
            let lower = bare.to_ascii_lowercase();
            let magic = lower.contains("frozen_string_literal:") || lower.contains("warn_indent:") || lower.contains("shareable_constant_value:") || lower.contains("coding") || lower.contains("encoding") || lower.starts_with("# vim:") || lower.starts_with("# -*-");
            if !magic { break; }
            if lower.contains("utf-8") && (lower.contains("coding") || lower.contains("encoding")) {
                let range = self.range_between(offset, offset + bare.chars().count());
                let replacement = encoding_comment_without_encoding(bare);
                let whole = whole_line_range(self, range.begin_pos(), range.end_pos());
                add_offense!(self, range, message: "Unnecessary utf-8 encoding comment.", |corrector| { if replacement.is_empty() { corrector.remove(whole); } else { corrector.replace(range, replacement); } });
            }
            offset += line.chars().count();
        }
    }
}

fn encoding_comment_without_encoding(line: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if lower.starts_with("# vim:") {
        let kept = line[6..].split(',').map(str::trim).filter(|part| { let part = part.to_ascii_lowercase(); !part.contains("encoding") && !part.contains("coding") }).collect::<Vec<_>>();
        return if kept.is_empty() { String::new() } else { format!("# vim: {}", kept.join(", ")) };
    }
    if lower.starts_with("# -*-") {
        let inner = line.trim_start_matches("# -*-").trim().strip_suffix("-*-").unwrap_or(line).trim();
        let kept = inner.split(';').map(str::trim).filter(|part| { let part = part.to_ascii_lowercase(); !part.starts_with("encoding") && !part.starts_with("coding") && !part.starts_with("fileencoding") }).collect::<Vec<_>>();
        return if kept.is_empty() { String::new() } else { format!("# -*- {} -*-", kept.join("; ")) };
    }
    String::new()
}

define_compatibility_rule!(DuplicateMagicCommentRule);
impl DuplicateMagicCommentRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let mut encoding = false; let mut frozen = false; let mut offset = 0usize;
        for line in self.source().split_inclusive('\n') {
            let bare = line.trim_end_matches('\n'); let lower = bare.to_ascii_lowercase();
            let kind = if lower.contains("coding:") || lower.contains("coding=") { Some(&mut encoding) } else if lower.contains("frozen_string_literal:") { Some(&mut frozen) } else { None };
            let Some(seen) = kind else { if !bare.starts_with("#!") { break; } offset += line.chars().count(); continue; };
            if *seen { let range = self.range_between(offset, offset + bare.chars().count()); let whole = whole_line_range(self, range.begin_pos(), range.end_pos()); add_offense!(self, range, message: "Duplicate magic comment detected.", |corrector| { corrector.remove(whole); }); } else { *seen = true; }
            offset += line.chars().count();
        }
    }
}

define_compatibility_rule!(RedundantSortByRule);
impl RedundantSortByRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        let Some(send) = node.send_node().filter(|send| send.method_name() == Some("sort_by")) else { return; };
        let arguments = node.arguments().into_iter().filter(|argument| argument.name().is_some()).collect::<Vec<_>>(); let Some(mut body) = node.body() else { return; }; if body.kind() == "begin" && body.child_nodes().len() == 1 { body = body.child_nodes()[0]; }
        let message = match node.kind() {
            "block" if arguments.len() == 1 && body.name() == arguments[0].name() => format!("Use `sort` instead of `sort_by {{ |{}| {} }}`.", arguments[0].name().unwrap_or_default(), arguments[0].name().unwrap_or_default()),
            "numblock" if body.name() == Some("_1") => "Use `sort` instead of `sort_by { _1 }`.".to_owned(),
            "itblock" if body.name() == Some("it") => "Use `sort` instead of `sort_by { it }`.".to_owned(),
            _ => return,
        };
        let (Some(selector), Some(ending)) = (self.location_range(send, "selector"), self.location_range(node, "end")) else { return; }; let range = self.range_between(selector.begin_pos(), ending.end_pos());
        add_offense!(self, range, message: message, |corrector| { corrector.replace(range, "sort"); });
    }
}

define_compatibility_rule!(StabbyLambdaParenthesesRule);
impl StabbyLambdaParenthesesRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.lambda_literal() { return; } let Some(block) = node.block_node() else { return; }; let Some(arguments) = block.arguments_node().filter(|arguments| !arguments.child_nodes().is_empty()) else { return; };
        let begin = self.location_range(arguments, "begin"); let end = self.location_range(arguments, "end"); let require = self.policy().enforced_style("require_parentheses") == "require_parentheses"; if require == begin.is_some() { return; }
        let message = if require { "Wrap stabby lambda arguments with parentheses." } else { "Do not wrap stabby lambda arguments with parentheses." };
        add_offense!(self, arguments, message: message, |corrector| { if require { corrector.wrap(arguments, "(", ")"); } else { if let Some(begin) = begin { corrector.remove(begin); } if let Some(end) = end { corrector.remove(end); } } });
    }
}

define_compatibility_rule!(NestedParenthesizedCallsRule);
impl NestedParenthesizedCallsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.parenthesized() { return; }
        for nested in node.arguments().into_iter().filter(|child| matches!(child.kind(), "send" | "csend")) {
            let arguments = nested.arguments(); if arguments.is_empty() || nested.parenthesized() || nested.method_name().is_some_and(|method| method.ends_with('=')) || nested.operator_method() { continue; }
            if node.arguments().len() == 1 && arguments.len() == 1 && self.allowed_methods().allowed_method(nested.method_name().unwrap_or_default()) { continue; }
            let Some(first) = arguments.first().and_then(|argument| argument.source_range()) else { continue; }; let Some(last) = arguments.last().copied() else { continue; }; let mut start = first.start; let chars = self.source().chars().collect::<Vec<_>>(); loop { while start > 0 && chars.get(start - 1).is_some_and(|character| matches!(character, ' ' | '\t')) { start -= 1; } if start >= 2 && chars.get(start - 1) == Some(&'\n') && chars.get(start - 2) == Some(&'\\') { start -= 2; continue; } break; } let leading = self.range_between(start, first.start);
            add_offense!(self, nested, message: format!("Add parentheses to nested method call `{}`.", nested.source().unwrap_or_default()), |corrector| { corrector.replace(leading, "("); corrector.insert_after(last, ")"); });
        }
    }
}

define_compatibility_rule!(UselessNumericOperationRule);
impl UselessNumericOperationRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(operation @ ("+" | "-" | "*" | "/" | "**")) = node.method_name() else { return; }; let Some(variable) = node.receiver().filter(|receiver| receiver.kind() == "send" && receiver.receiver().is_none() && receiver.arguments().is_empty()) else { return; }; let Some(number) = node.first_argument().and_then(integer_value) else { return; }; if !useless_numeric(operation, number) { return; }
        add_offense!(self, node, message: "Do not apply inconsequential numeric operations to variables.", |corrector| { corrector.replace(node, variable.source().unwrap_or_default()); });
    }
    fn on_op_asgn(&mut self, node: NodeRef<'_>) {
        let (Some(variable), Some(number), Some(operation)) = (node.lhs(), node.rhs().and_then(integer_value), node.assignment_operator()) else { return; }; if !useless_numeric(operation, number) { return; } let name = variable.name().or_else(|| variable.source()).unwrap_or_default();
        add_offense!(self, node, message: "Do not apply inconsequential numeric operations to variables.", |corrector| { corrector.replace(node, format!("{name} = {name}")); });
    }
}
fn useless_numeric(operation: &str, number: i64) -> bool { number == 0 && matches!(operation, "+" | "-") || number == 1 && matches!(operation, "*" | "/" | "**") }
fn integer_value(node: NodeRef<'_>) -> Option<i64> { if node.kind() == "int" { node.source()?.replace('_', "").parse().ok() } else { None } }

define_compatibility_rule!(RedundantRequireStatementRule);
impl RedundantRequireStatementRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.receiver().is_some() { return; } let Some(feature) = node.first_argument().and_then(NodeRef::str_content) else { return; }; let version = self.target_ruby_version(); let redundant = feature == "enumerator" || version.at_least(2, 1) && feature == "thread" || version.at_least(2, 2) && matches!(feature, "rational" | "complex") || version.at_least(2, 7) && feature == "ruby2_keywords" || version.at_least(3, 1) && feature == "fiber" || version.at_least(3, 2) && feature == "set" || version.at_least(4, 0) && feature == "pathname"; if !redundant { return; }
        let source_range = node.source_range(); let modifier = node.parent().filter(|parent| parent.modifier_form());
        let whole = source_range.as_ref().map(|range| whole_line_range(self, range.start, range.end));
        let modifier_removal = source_range.map(|range| self.range_between(range.start, (range.end + 1).min(self.source().chars().count())));
        add_offense!(self, node, message: "Remove unnecessary `require` statement.", |corrector| { if let Some(parent) = modifier { corrector.insert_after(parent, "\nend"); if let Some(range) = modifier_removal { corrector.remove(range); } } else if let Some(range) = whole { corrector.remove(range); } });
    }
}

define_compatibility_rule!(RedundantWithObjectRule);
impl RedundantWithObjectRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        let Some(send) = node.send_node().filter(|send| matches!(send.method_name(), Some("each_with_object" | "with_object")) && send.arguments().len() == 1) else { return; }; if !single_implicit_or_explicit_block_argument(node) { return; } let Some(selector) = self.location_range(send, "selector") else { return; }; let end = send.source_range().map_or(selector.end_pos(), |range| range.end); let range = self.range_between(selector.begin_pos(), end); let each = send.method_name() == Some("each_with_object"); let message = if each { "Use `each` instead of `each_with_object`." } else { "Remove redundant `with_object`." };
        let dot = self.location_range(send, "dot"); add_offense!(self, range, message: message, |corrector| { if each { corrector.replace(range, "each"); } else { corrector.remove(range); if let Some(dot) = dot { corrector.remove(dot); } } });
    }
}

define_compatibility_rule!(RedundantWithIndexRule);
impl RedundantWithIndexRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        let Some(send) = node.send_node().filter(|send| matches!(send.method_name(), Some("each_with_index" | "with_index"))) else { return; }; if !single_implicit_or_explicit_block_argument(node) || send.method_name() == Some("with_index") && send.receiver().and_then(NodeRef::receiver).is_none() { return; } let Some(selector) = self.location_range(send, "selector") else { return; }; let end = send.source_range().map_or(selector.end_pos(), |range| range.end); let range = self.range_between(selector.begin_pos(), end); let each = send.method_name() == Some("each_with_index"); let message = if each { "Use `each` instead of `each_with_index`." } else { "Remove redundant `with_index`." };
        let dot = self.location_range(send, "dot"); add_offense!(self, range, message: message, |corrector| { if each { corrector.replace(selector, "each"); } else { corrector.remove(range); if let Some(dot) = dot { corrector.remove(dot); } } });
    }
}

fn single_implicit_or_explicit_block_argument(node: NodeRef<'_>) -> bool { match node.kind() { "block" => node.arguments().len() == 1, "numblock" => node.numbered_arguments().len() == 1, "itblock" => true, _ => false } }

define_compatibility_rule!(FileNullRule);
impl FileNullRule<'_, '_, '_, '_> {
    fn on_str(&mut self, node: NodeRef<'_>) {
        let Some(value) = node.str_content().filter(|value| !value.is_empty()) else { return; }; if node.parent().is_some_and(|parent| matches!(parent.kind(), "array" | "pair")) { return; } let lower = value.to_ascii_lowercase(); if !matches!(lower.as_str(), "/dev/null" | "nul" | "nul:") { return; } if lower == "nul" && !self.processed_source().ast().is_some_and(|root| root.each_node(&["str"]).into_iter().any(|string| string.str_content().is_some_and(|value| value.eq_ignore_ascii_case("/dev/null")))) { return; }
        add_offense!(self, node, message: format!("Use `File::NULL` instead of `{value}`."), |corrector| { corrector.replace(node, "File::NULL"); });
    }
}

define_compatibility_rule!(AsciiIdentifiersRule);
impl AsciiIdentifiersRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let ascii_constants = self.config_bool("AsciiConstants", true); let tokens = self.processed_source().tokens().to_vec(); let literals = literal_source_ranges(self);
        for token in tokens { if token.kind != "tIDENTIFIER" && !(ascii_constants && token.kind == "tCONSTANT") || token.text.is_ascii() || literals.iter().any(|range| range.start <= token.begin_pos() && token.begin_pos() < range.end) { continue; } let Some((offset, sequence)) = first_non_ascii_sequence(&token.text) else { continue; }; let range = self.range_between(token.begin_pos() + offset, token.begin_pos() + offset + sequence); let message = if token.kind == "tIDENTIFIER" { "Use only ascii symbols in identifiers." } else { "Use only ascii symbols in constants." }; self.report(message, range); }
    }
}
fn first_non_ascii_sequence(text: &str) -> Option<(usize, usize)> { let chars = text.chars().collect::<Vec<_>>(); let start = chars.iter().position(|character| !character.is_ascii())?; let len = chars[start..].iter().take_while(|character| !character.is_ascii()).count(); Some((start, len)) }
fn literal_source_ranges(context: &CompatibilityCopContext<'_, '_, '_>) -> Vec<std::ops::Range<usize>> { context.processed_source().ast().map_or_else(Vec::new, |root| root.each_node(&["str", "dstr", "sym", "dsym", "regexp"]).into_iter().flat_map(|node| [node.source_range(), node.loc("heredoc_body").map(|(range, _)| range.clone())].into_iter().flatten()).collect()) }

define_compatibility_rule!(IpAddressesRule);
impl IpAddressesRule<'_, '_, '_, '_> {
    fn on_str(&mut self, node: NodeRef<'_>) {
        if self.location_range(node, "begin").is_none() { return; } let Some(value) = node.str_content().filter(|value| !value.is_empty() && value.len() <= 45 && value.chars().next().is_some_and(|character| character == ':' || character.is_ascii_hexdigit())) else { return; }; let allowed = self.config_values("AllowedAddresses"); if allowed.iter().any(|item| item.eq_ignore_ascii_case(value)) || !self.config_explicit("AllowedAddresses") && value == "::" || value.parse::<std::net::IpAddr>().is_err() { return; } self.report("Do not hardcode IP addresses.", node);
    }
}

define_compatibility_rule!(RedundantConditionalRule);
impl RedundantConditionalRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) {
        if node.modifier_form() { return; } let Some(condition) = node.condition().filter(|condition| condition.comparison_method()) else { return; }; let (Some(if_branch), Some(else_branch)) = (node.if_branch(), node.else_branch()) else { return; }; if !(matches!(if_branch.kind(), "true" | "false") && matches!(else_branch.kind(), "true" | "false") && if_branch.kind() != else_branch.kind()) { return; } let mut inverted = if_branch.kind() == "false" && else_branch.kind() == "true"; if node.unless_keyword() { inverted = !inverted; }
        let expression = if inverted { format!("!({})", condition.source().unwrap_or_default()) } else { condition.source().unwrap_or_default().to_owned() }; let indentation = " ".repeat(node.parent().map_or(node.column(), |parent| parent.column() + 2)); let replacement = if node.elsif() { format!("else\n{indentation}{expression}") } else { expression }; let rendered = if node.elsif() { format!("\n{replacement}") } else { replacement.clone() };
        let offense = if node.elsif() { node.source_range().zip(else_branch.source_range()).map(|(node, branch)| self.range_between(node.start, branch.end)) } else { node.source_range().map(|range| self.owned_character_range(range)) }; let Some(offense) = offense else { return; };
        add_offense!(self, offense, message: format!("This conditional expression can just be replaced by `{rendered}`."), |corrector| { corrector.replace(offense, replacement); });
    }
}

define_compatibility_rule!(EachForSimpleLoopRule);
impl EachForSimpleLoopRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        if node.kind() != "block" || !node.arguments().is_empty() { return; } let Some(send) = node.send_node().filter(|send| send.method_name() == Some("each")) else { return; }; let Some(receiver) = send.receiver() else { return; }; let range = if matches!(receiver.kind(), "irange" | "erange") { receiver } else if receiver.kind() == "begin" { let Some(range) = receiver.child_nodes().first().copied().filter(|range| matches!(range.kind(), "irange" | "erange")) else { return; }; range } else { return; }; let (Some(min), Some(max)) = (range.range_begin().and_then(integer_value), range.range_end().and_then(integer_value)) else { return; }; let count = max - min + i64::from(range.kind() == "irange");
        add_offense!(self, send, message: "Use `Integer#times` for a simple loop which iterates a fixed number of times.", |corrector| { corrector.replace(send, format!("{count}.times")); });
    }
}

define_compatibility_rule!(DigChainRule);
impl DigChainRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if self.location_range(node, "dot").is_none() || !valid_dig(node) || node.parent().is_some_and(|parent| parent.method_name() == Some("dig") && parent.receiver() == Some(node)) { return; } let mut current = node; let mut arguments = node.arguments(); let end = node.source_range().map_or(0, |range| range.end); let mut begin = None;
        while let Some(receiver) = current.receiver().filter(|receiver| valid_dig(*receiver)) { begin = self.location_range(receiver, "selector"); let mut earlier = receiver.arguments(); earlier.append(&mut arguments); arguments = earlier; current = receiver; }
        let Some(begin) = begin else { return; }; let forwarded = arguments.iter().position(|argument| matches!(argument.kind(), "forward_args" | "forwarded_args")); if forwarded.is_some_and(|index| index < arguments.len() - 1) { return; } let range = self.range_between(begin.begin_pos(), end); let replacement = format!("dig({})", arguments.iter().filter_map(|argument| argument.source()).collect::<Vec<_>>().join(", ")); let comments = self.comments_help().comments_in_range(node).into_iter().map(|comment| comment.text.clone()).collect::<Vec<_>>();
        add_offense!(self, range, message: format!("Use `{replacement}` instead of chaining."), |corrector| { corrector.replace(range, replacement); for comment in &comments { corrector.insert_before(node, format!("{comment}\n")); } });
    }
}

fn valid_dig(node: NodeRef<'_>) -> bool { node.method_name() == Some("dig") && !node.arguments().is_empty() && node.arguments().iter().all(|argument| !matches!(argument.kind(), "hash" | "block_pass")) }

define_compatibility_rule!(MinMaxComparisonRule);
impl MinMaxComparisonRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) {
        let Some(condition) = node.condition() else { return; }; let comparison = if condition.kind() == "begin" { condition.child_nodes().first().copied() } else { Some(condition) }; let Some(comparison) = comparison else { return; }; let (Some(lhs), Some(rhs), Some(operator), Some(if_branch), Some(else_branch)) = (comparison.receiver(), comparison.first_argument(), comparison.method_name(), node.if_branch(), node.else_branch()) else { return; }; if !matches!(operator, ">" | ">=" | "<" | "<=") { return; }
        let method = if lhs.structurally_equal(if_branch) && rhs.structurally_equal(else_branch) { if matches!(operator, ">" | ">=") { "max" } else { "min" } } else if lhs.structurally_equal(else_branch) && rhs.structurally_equal(if_branch) { if matches!(operator, "<" | "<=") { "max" } else { "min" } } else { return; }; let replacement = format!("[{}, {}].{method}", lhs.source().unwrap_or_default(), rhs.source().unwrap_or_default());
        if node.elsif() { let Some(parent_else) = node.parent().and_then(|parent| self.location_range(parent, "else")) else { return; }; let Some(own_else) = self.location_range(node, "else") else { return; }; let removal = self.range_between(parent_else.begin_pos(), own_else.begin_pos()); let Some(branch) = node.else_branch() else { return; }; let offense = node.source_range().zip(branch.source_range()).map(|(node, branch)| self.range_between(node.start, branch.end)); let Some(offense) = offense else { return; }; add_offense!(self, offense, message: format!("Use `{replacement}` instead."), |corrector| { corrector.remove(removal); corrector.replace(branch, replacement); }); } else { add_offense!(self, node, message: format!("Use `{replacement}` instead."), |corrector| { corrector.replace(node, replacement); }); }
    }
}

define_compatibility_rule!(YodaExpressionRule);
impl YodaExpressionRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let supported = self.config_values("SupportedOperators"); let Some(_operator) = node.method_name().filter(|operator| supported.iter().any(|item| item == *operator)) else { return; }; let (Some(lhs), Some(rhs)) = (node.receiver(), node.first_argument()) else { return; }; if !matches!(lhs.kind(), "int" | "float" | "rational" | "complex" | "const") || matches!(rhs.kind(), "int" | "float" | "rational" | "complex" | "const") { return; } if node.ancestors().into_iter().any(|ancestor| ancestor.method_name().is_some_and(|operator| supported.iter().any(|item| item == operator)) && ancestor.receiver().is_some_and(|receiver| matches!(receiver.kind(), "int" | "float" | "rational" | "complex" | "const")) && ancestor.first_argument().is_some_and(|argument| !matches!(argument.kind(), "int" | "float" | "rational" | "complex" | "const"))) { return; }
        add_offense!(self, node, message: format!("Non-literal operand (`{}`) should be first.", rhs.source().unwrap_or_default()), |corrector| { corrector.swap(lhs, rhs); });
    }
}

define_compatibility_rule!(StructInheritanceRule);
impl StructInheritanceRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) {
        let Some(parent) = node.parent_class().filter(|parent| parent.method_name() == Some("new") && parent.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("Struct") && receiver.namespace().is_none_or(|namespace| namespace.kind() == "cbase"))) else { return; }; let (Some(keyword), Some(operator)) = (self.location_range(node, "keyword"), self.location_range(node, "operator")) else { return; }; let class_body = node.body().is_some(); let unparenthesized = parent.kind() != "block" && !parent.parenthesized() && !parent.arguments().is_empty(); let args = parent.arguments().iter().filter_map(|argument| argument.source()).collect::<Vec<_>>().join(", ");
        let keyword_with_space = self.range_between(keyword.begin_pos(), (keyword.end_pos() + 1).min(self.source().chars().count())); let unparenthesized_range = self.location_range(parent, "selector").map(|selector| self.range_between(selector.end_pos(), parent.source_range().map_or(selector.end_pos(), |range| range.end))); let parent_end_removal = self.location_range(parent, "end").map(|end| self.range_between(end.begin_pos().saturating_sub(1), end.end_pos())); let empty_class_removal = self.location_range(node, "end").map(|end| if node.single_line() { self.range_between(parent.source_range().map_or(end.begin_pos(), |range| range.end), node.source_range().map_or(end.end_pos(), |range| range.end)) } else { whole_line_range(self, end.begin_pos(), end.end_pos()) });
        add_offense!(self, parent, message: "Don't extend an instance initialized by `Struct.new`. Use a block to customize the struct.", |corrector| { corrector.remove(keyword_with_space); corrector.replace(operator, "="); if parent.kind() == "block" { if let Some(removal) = parent_end_removal { corrector.remove(removal); } } else if !class_body { if let Some(removal) = empty_class_removal { corrector.remove(removal); } } else if unparenthesized { if let Some(range) = unparenthesized_range { corrector.replace(range, format!("({args}) do")); } } else { corrector.insert_after(parent, " do"); } });
    }
}

define_compatibility_rule!(DateTimeRule);
impl DateTimeRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let coercion = node.method_name() == Some("to_datetime") && node.arguments().is_empty(); if coercion && self.config_bool("AllowCoercion", false) { return; } let date_time = node.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("DateTime") && receiver.namespace().is_none_or(|namespace| namespace.kind() == "cbase")); if !coercion && !date_time { return; } if node.arguments().get(1).is_some_and(|argument| argument.kind() == "const" && argument.namespace().is_some_and(|namespace| namespace.short_name() == Some("Date"))) { return; } let message = if coercion { "Do not use `#to_datetime`." } else { "Prefer `Time` over `DateTime`." };
        if coercion { self.report(message, node); return; }
        let receiver_name = node.receiver().and_then(|receiver| self.location_range(receiver, "name")); add_offense!(self, node, message: message, |corrector| { if let Some(receiver_name) = receiver_name { corrector.replace(receiver_name, "Time"); } });
    }
}

define_compatibility_rule!(ConcatArrayLiteralsRule);
impl ConcatArrayLiteralsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let arguments = node.arguments(); if arguments.is_empty() || arguments.iter().any(|argument| argument.kind() != "array") { return; } let Some(selector) = self.location_range(node, "selector") else { return; }; let end = node.source_range().map_or(selector.end_pos(), |range| range.end); let range = self.range_between(selector.begin_pos(), end); let percent = arguments.iter().any(|argument| argument.percent_literal(None)); let basic = arguments.iter().all(|argument| !argument.percent_literal(None) || argument.child_nodes().iter().all(|child| matches!(child.kind(), "str" | "sym"))); let values = arguments.iter().flat_map(|argument| argument.child_nodes()).map(|child| if percent { match child.kind() { "sym" => format!(":{}", child.scalar_value_text().unwrap_or_default()), _ => format!("{:?}", child.scalar_value_text().unwrap_or_else(|| child.source().unwrap_or_default().to_owned())) } } else { child.source().unwrap_or_default().to_owned() }).collect::<Vec<_>>().join(", "); let preferred = format!("push({values})"); let message = if percent && !basic { format!("Use `push` with elements as arguments without array brackets instead of `{}`.", self.range_source(&range)) } else { format!("Use `{preferred}` instead of `{}`.", self.range_source(&range)) };
        let delimiters = arguments.iter().map(|argument| (self.location_range(*argument, "begin"), self.location_range(*argument, "end"))).collect::<Vec<_>>();
        if percent && !basic { self.report(message, range); return; }
        add_offense!(self, range, message: message, |corrector| { if !percent { corrector.replace(selector, "push"); for (begin, end) in delimiters { if let Some(begin) = begin { corrector.remove(begin); } if let Some(end) = end { corrector.remove(end); } } } else { corrector.replace(range, preferred); } });
    }
}

define_compatibility_rule!(ObjectThenRule);
impl ObjectThenRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) { if let Some(send) = node.send_node() { self.check(send); } }
    fn on_send(&mut self, node: NodeRef<'_>) { if node.arguments().len() == 1 && node.first_argument().is_some_and(|argument| argument.kind() == "block_pass") { self.check(node); } }
    fn check(&mut self, node: NodeRef<'_>) { if !self.target_ruby_version().at_least(2, 6) || !matches!(node.method_name(), Some("then" | "yield_self")) { return; } let preferred = self.policy().enforced_style("then").to_owned(); if node.method_name() == Some(preferred.as_str()) { return; } let Some(selector) = self.location_range(node, "selector") else { return; }; let replacement = if preferred == "then" && node.receiver().is_none() { "self.then".to_owned() } else { preferred.clone() }; add_offense!(self, selector, message: format!("Prefer `{preferred}` over `{}`.", node.method_name().unwrap_or_default()), |corrector| { corrector.replace(selector, replacement); }); }
}

define_compatibility_rule!(SingleLineDoEndBlockRule);
impl SingleLineDoEndBlockRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) { let (Some(begin), Some(end)) = (self.location_range(node, "begin"), self.location_range(node, "end")) else { return; }; let inspect_blocks = self.related_config_value("Layout/RedundantLineBreak", "InspectBlocks") == Some("true"); let max = self.related_config_value("Layout/LineLength", "Max").and_then(|value| value.parse::<usize>().ok()).unwrap_or(120); if self.range_source(&begin) != "do" || node.multiline() || inspect_blocks && node.source_length() <= max { return; } let do_line = if node.kind() == "block" && !node.arguments().is_empty() && !node.send_node().is_some_and(|send| send.lambda_literal()) { node.arguments_node().and_then(NodeRef::source_range).map(|range| self.owned_character_range(range)).unwrap_or(begin) } else { begin }; let heredoc_end = node.body().and_then(|body| self.location_range(body, "heredoc_end")); add_offense!(self, node, message: "Prefer multiline `do`...`end` block.", |corrector| { corrector.insert_after(do_line, "\n"); if let Some(heredoc_end) = heredoc_end { corrector.remove(end); corrector.insert_after(heredoc_end, "\nend"); } else { corrector.insert_before(end, "\n"); } }); }
}
