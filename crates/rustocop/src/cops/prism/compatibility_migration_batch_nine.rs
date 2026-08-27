use super::*;
use crate::rubocop::ast::node::core::NodeRef;

define_cops! {
    InsecureProtocolSourceCompatibility => "Bundler/InsecureProtocolSource" => compatibility_callbacks(InsecureProtocolSourceRule, [on_send restrict ["source"]]),
    DisjunctiveAssignmentInConstructorCompatibility => "Lint/DisjunctiveAssignmentInConstructor" => compatibility_callbacks(DisjunctiveAssignmentInConstructorRule, [on_def]),
    EmptyInPatternCompatibility => "Lint/EmptyInPattern" => compatibility_callbacks(EmptyInPatternRule, [on_in_pattern]),
    InheritExceptionCompatibility => "Lint/InheritException" => compatibility_callbacks(InheritExceptionRule, [on_class, on_send]),
    RaiseExceptionCompatibility => "Lint/RaiseException" => compatibility_callbacks(RaiseExceptionRule, [on_send restrict ["raise", "fail"]]),
    CaseEqualityCompatibility => "Style/CaseEquality" => compatibility_callbacks(CaseEqualityRule, [on_send restrict ["==="]]),
    NumericLiteralPrefixCompatibility => "Style/NumericLiteralPrefix" => compatibility_callbacks(NumericLiteralPrefixRule, [on_int]),
    OneClassPerFileCompatibility => "Style/OneClassPerFile" => compatibility_callbacks(OneClassPerFileRule, [on_class, on_module]),
    PerlBackrefsCompatibility => "Style/PerlBackrefs" => compatibility_callbacks(PerlBackrefsRule, [on_back_ref, on_gvar, on_nth_ref]),
    ReturnNilCompatibility => "Style/ReturnNil" => compatibility_callbacks(ReturnNilRule, [on_return]),
    EmptyLinesCompatibility => "Layout/EmptyLines" => compatibility_investigation(EmptyLinesRule, on_new_investigation),
    EndOfLineCompatibility => "Layout/EndOfLine" => compatibility_investigation(EndOfLineRule, on_new_investigation),
    TrailingEmptyLinesCompatibility => "Layout/TrailingEmptyLines" => compatibility_investigation(TrailingEmptyLinesRule, on_new_investigation),
    LeadingCommentSpaceCompatibility => "Layout/LeadingCommentSpace" => compatibility_investigation(LeadingCommentSpaceRule, on_new_investigation),
    EmptyCommentCompatibility => "Layout/EmptyComment" => compatibility_investigation(EmptyCommentRule, on_new_investigation),
    EmptyLineAfterMagicCommentCompatibility => "Layout/EmptyLineAfterMagicComment" => compatibility_investigation(EmptyLineAfterMagicCommentRule, on_new_investigation),
    IndentationStyleCompatibility => "Layout/IndentationStyle" => compatibility_investigation(IndentationStyleRule, on_new_investigation),
    OrderedMagicCommentsCompatibility => "Lint/OrderedMagicComments" => compatibility_investigation(OrderedMagicCommentsRule, on_new_investigation),
    RequireOrderCompatibility => "Style/RequireOrder" => compatibility_callbacks(RequireOrderRule, [on_send restrict ["require", "require_relative"]]),
    TrailingCommaInBlockArgsCompatibility => "Style/TrailingCommaInBlockArgs" => compatibility_callbacks(TrailingCommaInBlockArgsRule, [on_block]),
}

define_compatibility_rule!(InsecureProtocolSourceRule);
impl InsecureProtocolSourceRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        return_if!(node.receiver().is_some() || node.arguments().len() != 1);
        let argument = node.arguments()[0];
        let symbol_source = argument.source().and_then(|source| source.strip_prefix(':'));
        let (message, insecure) = if matches!(symbol_source, Some("gemcutter" | "rubygems" | "rubyforge")) {
            let source = symbol_source.unwrap_or_default();
            (format!("The source `:{source}` is deprecated because HTTP requests are insecure. Please change your source to 'https://rubygems.org' if possible, or 'http://rubygems.org' if not."), true)
        } else if argument.kind() == "str" && argument.str_content() == Some("http://rubygems.org") {
            ("Use `https://rubygems.org` instead of `http://rubygems.org`.".to_owned(), !self.config_bool("AllowHttpProtocol", true))
        } else {
            return;
        };
        if insecure {
            add_offense!(self, argument, message: message, |corrector| {
                corrector.replace(argument, "'https://rubygems.org'");
            });
        }
    }
}

define_compatibility_rule!(DisjunctiveAssignmentInConstructorRule);
impl DisjunctiveAssignmentInConstructorRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) {
        return_unless!(node.method_name() == Some("initialize"));
        let Some(body) = node.body() else { return; };
        let statements = if body.kind() == "begin" { body.child_nodes() } else { vec![body] };
        for statement in statements {
            if !statement.kind().ends_with("or_asgn") && statement.kind() != "or_asgn" { break; }
            let Some(lhs) = statement.child_nodes().first().copied() else { break; };
            if lhs.kind() != "ivasgn" { break; }
            let Some(operator) = self.location_range(statement, "operator") else { continue; };
            add_offense!(self, operator, message: "Unnecessary disjunctive assignment. Use plain assignment.", |corrector| {
                corrector.replace(operator, "=");
            });
        }
    }
}

define_compatibility_rule!(EmptyInPatternRule);
impl EmptyInPatternRule<'_, '_, '_, '_> {
    fn on_in_pattern(&mut self, node: NodeRef<'_>) {
        return_if!(!self.target_ruby_version().at_least(2, 7) || node.body().is_some_and(|body| body.kind() != "in_pattern"));
        let Some(keyword) = self.location_range(node, "keyword") else { return; };
        let Some(pattern) = node.child_nodes().first().and_then(|node| node.source_range()) else { return; };
        let line = self.source_buffer().line_range(node.first_line());
        let pattern_end = if self.range_source(&self.owned_character_range(line.clone())).contains(" if ") {
            let source = self.range_source(&self.owned_character_range(line.clone()));
            line.start + source.split('#').next().unwrap_or(source).trim_end().chars().count()
        } else { pattern.end };
        let offense = self.range_between(keyword.begin_pos(), pattern_end);
        if self.config_bool("AllowComments", true) {
            let tail = &self.source()[self.source_buffer().byte_position(offense.end_pos()).unwrap_or(self.source().len())..];
            let branch_end = tail.split_inclusive('\n').scan(0usize, |offset, line| { let start = *offset; *offset += line.len(); Some((start, line)) }).skip(1).find_map(|(offset, line)| matches!(line.trim_start(), value if value.starts_with("in ") || value.starts_with("end")).then_some(offset)).unwrap_or(tail.len());
            if tail[..branch_end].contains('#') { return; }
        }
        self.report("Avoid `in` branches without a body.", offense);
    }
}

define_compatibility_rule!(InheritExceptionRule);
impl InheritExceptionRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) {
        let Some(parent) = node.parent_class() else { return; };
        if parent.kind() == "const" && parent.short_name() == Some("Exception") && !parent.absolute() {
            let before = node.source_range().map(|range| &self.source()[..self.source_buffer().byte_position(range.start).unwrap_or(0)]).unwrap_or_default();
            if before.lines().any(|line| line.trim_start().starts_with("class Exception ")) { return; }
        }
        self.check(parent);
    }
    fn on_send(&mut self, node: NodeRef<'_>) {
        return_unless!(node.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("Class")) && node.arguments().len() == 1);
        self.check(node.arguments()[0]);
    }
    fn check(&mut self, exception: NodeRef<'_>) {
        return_unless!(exception.kind() == "const" && exception.short_name() == Some("Exception") && (exception.absolute() || exception.namespace().is_none()));
        let preferred = if self.policy().enforced_style("standard_error") == "runtime_error" { "RuntimeError" } else { "StandardError" };
        add_offense!(self, exception, message: format!("Inherit from `{preferred}` instead of `Exception`."), |corrector| {
            corrector.replace(exception, preferred);
        });
    }
}

define_compatibility_rule!(RaiseExceptionRule);
impl RaiseExceptionRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        return_if!(node.receiver().is_some());
        let Some(argument) = node.first_argument() else { return; };
        let exception = if argument.kind() == "const" && argument.short_name() == Some("Exception") && (argument.absolute() || argument.namespace().is_none()) {
            argument
        } else if argument.method_name() == Some("new") && argument.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("Exception") && (receiver.absolute() || receiver.namespace().is_none())) {
            argument.receiver().unwrap()
        } else { return; };
        if !exception.absolute() && node.ancestors().iter().any(|ancestor| {
            ancestor.kind() == "module"
                && ancestor.identifier().and_then(NodeRef::short_name).is_some_and(|name| self.config_values("AllowedImplicitNamespaces").iter().any(|allowed| allowed == name))
        }) { return; }
        let replacement = if exception.absolute() { "::StandardError" } else { "StandardError" };
        add_offense!(self, exception, message: "Use `StandardError` over `Exception`.", |corrector| {
            corrector.replace(exception, replacement);
        });
    }
}

define_compatibility_rule!(CaseEqualityRule);
impl CaseEqualityRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let (Some(receiver), Some(argument), Some(selector)) = (node.receiver(), node.first_argument(), self.location_range(node, "selector")) else { return; };
        return_if!(matches!(receiver.kind(), "regexp"));
        let receiver_name = receiver.source().unwrap_or_default().rsplit("::").next().unwrap_or_default();
        let constant = receiver_name.chars().next().is_some_and(char::is_uppercase) && receiver_name.chars().any(char::is_lowercase);
        if receiver_name.chars().all(|character| character.is_ascii_uppercase() || character == '_') { return; }
        let self_class = receiver.method_name() == Some("class") && receiver.receiver().is_some_and(|node| node.kind() == "self");
        return_if!(constant && self.config_bool("AllowOnConstant", false) || self_class && self.config_bool("AllowOnSelfClass", false));
        let range_receiver = matches!(receiver.kind(), "irange" | "erange") || receiver.child_nodes().iter().any(|node| matches!(node.kind(), "irange" | "erange"));
        let replacement = if range_receiver {
            Some(format!("{}.include?({})", receiver.source().unwrap_or_default(), argument.source().unwrap_or_default()))
        } else if constant || self_class {
            Some(format!("{}.is_a?({})", argument.source().unwrap_or_default(), receiver.source().unwrap_or_default()))
        } else { None };
        if let Some(replacement) = replacement {
            add_offense!(self, selector, message: "Avoid the use of the case equality operator `===`.", |corrector| { corrector.replace(node, replacement); });
        } else {
            self.report("Avoid the use of the case equality operator `===`.", selector);
        }
    }
}

define_compatibility_rule!(NumericLiteralPrefixRule);
impl NumericLiteralPrefixRule<'_, '_, '_, '_> {
    fn on_int(&mut self, node: NodeRef<'_>) {
        let Some(literal) = node.source() else { return; };
        let zero_only = self.config_value("EnforcedOctalStyle") == Some("zero_only");
        let result = if zero_only && (literal.starts_with("0o") || literal.starts_with("0O")) {
            Some(("Use 0 for octal literals.", format!("0{}", &literal[2..])))
        } else if let Some(digits) = literal.strip_prefix("0X") { Some(("Use 0x for hexadecimal literals.", format!("0x{digits}")))
        } else if let Some(digits) = literal.strip_prefix("0B") { Some(("Use 0b for binary literals.", format!("0b{digits}")))
        } else if let Some(digits) = literal.strip_prefix("0O") { Some(("Use 0o for octal literals.", format!("0o{digits}")))
        } else if literal.starts_with("0d") || literal.starts_with("0D") { Some(("Do not use prefixes for decimal literals.", literal[2..].to_owned()))
        } else if !zero_only && literal.len() > 1 && literal.starts_with('0') && literal[1..].bytes().all(|byte| matches!(byte, b'0'..=b'7')) { Some(("Use 0o for octal literals.", format!("0o{}", &literal[1..])))
        } else { None };
        let Some((message, replacement)) = result else { return; };
        add_offense!(self, node, message: message, |corrector| { corrector.replace(node, replacement); });
    }
}

define_compatibility_rule!(OneClassPerFileRule);
impl OneClassPerFileRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_module(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        return_if!(node.ancestors().iter().any(|ancestor| matches!(ancestor.kind(), "class" | "module" | "sclass")));
        return_unless!(node.parent().is_some_and(|parent| parent.kind() == "begin" && parent.parent().is_none()));
        let Some(root) = self.processed_source().ast() else { return; };
        let definitions = root.child_nodes().into_iter().filter(|candidate| matches!(candidate.kind(), "class" | "module")).collect::<Vec<_>>();
        let allowed = self.config_values("AllowedClasses");
        let count = definitions.iter().take_while(|candidate| candidate.source_range().zip(node.source_range()).is_some_and(|(a, b)| a.start <= b.start)).filter(|candidate| candidate.identifier().and_then(NodeRef::short_name).is_none_or(|name| !allowed.iter().any(|allowed| allowed == name))).count();
        if count <= 1 { return; }
        let Some(identifier) = node.identifier() else { return; };
        let Some(node_range) = node.source_range() else { return; };
        let Some(identifier_range) = identifier.source_range() else { return; };
        let offense = self.range_between(node_range.start, identifier_range.end);
        self.report("Do not define multiple classes/modules at the top level in a single file.", offense);
    }
}

define_compatibility_rule!(PerlBackrefsRule);
impl PerlBackrefsRule<'_, '_, '_, '_> {
    fn on_back_ref(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_gvar(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_nth_ref(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.name().or_else(|| node.source()) else { return; };
        let suffix = match name {
            "$&" | "$MATCH" => "(0)".to_owned(),
            "$`" | "$PREMATCH" => ".pre_match".to_owned(),
            "$'" | "$POSTMATCH" => ".post_match".to_owned(),
            "$+" | "$LAST_PAREN_MATCH" => "(-1)".to_owned(),
            value if value.strip_prefix('$').is_some_and(|digits| !digits.is_empty() && digits != "0" && digits.bytes().all(|byte| byte.is_ascii_digit())) => format!("({})", &value[1..]),
            _ => return,
        };
        let root = if node.ancestors().iter().any(|ancestor| matches!(ancestor.kind(), "class" | "module")) { "::" } else { "" };
        let replacement = format!("{root}Regexp.last_match{suffix}");
        let message = format!("Prefer `{replacement}` over `{name}`.");
        let embedded = node.parent().is_some_and(|parent| parent.kind() == "begin");
        let correction = if embedded { format!("{{{replacement}}}") } else { replacement };
        add_offense!(self, node, message: message, |corrector| { corrector.replace(node, correction); });
    }
}

define_compatibility_rule!(ReturnNilRule);
impl ReturnNilRule<'_, '_, '_, '_> {
    fn on_return(&mut self, node: NodeRef<'_>) {
        for ancestor in node.ancestors() {
            if matches!(ancestor.kind(), "def" | "defs") || ancestor.lambda_literal() || matches!(ancestor.method_name(), Some("define_method" | "define_singleton_method")) { break; }
            if matches!(ancestor.kind(), "block" | "numblock" | "itblock")
                && ancestor.arguments_node().is_some_and(|arguments| !arguments.child_nodes().is_empty())
                && ancestor.send_node().is_some_and(|send| send.receiver().is_some()) { return; }
        }
        let return_nil = self.policy().enforced_style("return") == "return_nil";
        let arguments = node.child_nodes();
        if return_nil && arguments.is_empty() {
            add_offense!(self, node, message: "Use `return nil` instead of `return`.", |corrector| { corrector.replace(node, "return nil"); });
        } else if !return_nil && arguments.len() == 1 && arguments[0].kind() == "nil" {
            add_offense!(self, node, message: "Use `return` instead of `return nil`.", |corrector| { corrector.replace(node, "return"); });
        }
    }
}

fn character_range(context: &CompatibilityCopContext<'_, '_, '_>, range: std::ops::Range<usize>) -> Option<CompatibilitySourceRange> {
    Some(context.range_between(context.source_buffer().character_position(range.start)?, context.source_buffer().character_position(range.end)?))
}

define_compatibility_rule!(EmptyLinesRule);
impl EmptyLinesRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let source = self.source();
        let ruby_end = source.find("\n__END__").map_or(source.len(), |offset| offset + 1);
        let content_end = source[..ruby_end].rfind(|character: char| !character.is_whitespace()).map_or(0, |offset| offset + 1);
        let ignored = super::source_rules_layout::ignored_syntax_ranges_from(
            source,
            self.prism_result(),
        );
        for (start, window) in source.as_bytes()[..content_end].windows(3).enumerate() {
            if window != b"\n\n\n" || ignored.iter().any(|range| range.start <= start + 2 && start + 2 < range.end) { continue; }
            let Some(offense) = character_range(self, start + 2..start + 3) else { continue; };
            add_offense!(self, offense, message: "Extra blank line detected.", |corrector| { corrector.remove(offense); });
        }
    }
}

define_compatibility_rule!(EndOfLineRule);
impl EndOfLineRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let source = self.source();
        let wants_crlf = self.policy().enforced_style("native") == "crlf";
        let data_start = source.find("\n__END__").map_or(source.len(), |offset| offset + 1);
        let mut bad = Vec::new(); let mut line_start = 0;
        for (index, byte) in source.as_bytes().iter().enumerate() {
            if *byte != b'\n' { continue; } if line_start >= data_start { break; }
            if (index > 0 && source.as_bytes()[index - 1] == b'\r') != wants_crlf { bad.push((line_start, index + 1)); }
            line_start = index + 1;
        }
        let (Some(first), Some(last)) = (bad.first(), bad.last()) else { return; };
        let message = if wants_crlf { "Carriage return character missing." } else { "Carriage return character detected." };
        let end = if wants_crlf { first.1 } else if bad.len() == 1 { (last.1 + 1).min(source.len()) } else { last.1 };
        let Some(offense) = character_range(self, first.0..end) else { return; };
        self.report(message, offense);
    }
}

define_compatibility_rule!(TrailingEmptyLinesRule);
impl TrailingEmptyLinesRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let source = self.source();
        if source.is_empty() || source.lines().any(|line| line.trim() == "__END__") { return; }
        let first = source.lines().next().unwrap_or_default().trim_end();
        if first.ends_with('%') || first.ends_with("%Q") || first.ends_with("%q") { return; }
        let content_end = source.trim_end().len(); let trailing = &source[content_end..];
        let newline_count = trailing.matches('\n').count();
        let final_blank = self.policy().enforced_style("final_newline") == "final_blank_line";
        let wanted = if final_blank { 2 } else { 1 }; if newline_count == wanted { return; }
        let message = if final_blank && newline_count == 1 { "Trailing blank line missing.".to_owned() } else if newline_count == 0 { "Final newline missing.".to_owned() } else if final_blank { format!("{} trailing blank lines instead of 1 detected.", newline_count.saturating_sub(1)) } else { format!("{} trailing blank lines detected.", newline_count.saturating_sub(1)) };
        let offense_bytes = if newline_count == 0 || final_blank && newline_count == 1 { source.len()..source.len() } else { content_end + 1..source.len() };
        let (Some(offense), Some(edit)) = (character_range(self, offense_bytes), character_range(self, content_end..source.len())) else { return; };
        add_offense!(self, offense, message: message, |corrector| { corrector.replace(edit, "\n".repeat(wanted)); });
    }
}

define_compatibility_rule!(LeadingCommentSpaceRule);
impl LeadingCommentSpaceRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let shebang = self.source().starts_with("#!"); let path = self.processed_source().file_path();
        let comments = self.processed_source().comments().to_vec();
        for comment in comments {
            if comment.embedded_document { continue; }
            let text = comment.text.as_str(); let content = text.strip_prefix('#').unwrap_or(text);
            let hashes = text.bytes().take_while(|byte| *byte == b'#').count();
            let multiple = hashes > 1 && (hashes == text.len() || text[hashes..].starts_with(char::is_whitespace));
            let first_line = comment.range.start == 0;
            if content.is_empty() || content.starts_with([' ', '\t']) || multiple || content.starts_with('=') || content.starts_with("++") || content.starts_with("--") || shebang && content.starts_with('!') || first_line && path.ends_with("config.ru") && content.starts_with('\\') || self.config_bool("AllowDoxygenCommentStyle", false) && content.starts_with('*') || self.config_bool("AllowGemfileRubyComment", false) && path.ends_with("Gemfile") && (content.starts_with("ruby=") || content.starts_with("ruby-gemset=")) || self.config_bool("AllowRBSInlineAnnotation", false) && content.starts_with(['[', ':', '|']) || self.config_bool("AllowSteepAnnotation", false) && content.starts_with(['$', ':']) { continue; }
            let offense = self.owned_character_range(comment.range.clone()); let insert = self.range_between(offense.begin_pos() + 1, offense.begin_pos() + 1);
            add_offense!(self, offense, message: "Missing space after `#`.", |corrector| { corrector.insert_before(insert, " "); });
        }
    }
}

define_compatibility_rule!(EmptyCommentRule);
impl EmptyCommentRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let source = self.source();
        let lines = source.split_inclusive('\n').scan(0usize, |offset, line| { let start = *offset; *offset += line.len(); Some((start, line.trim_end_matches(['\n', '\r']))) }).collect::<Vec<_>>();
        let comments = self.processed_source().comments().to_vec();
        for (index, (offset, line)) in lines.iter().copied().enumerate() {
            let Some(comment) = comments.iter().find(|comment| !comment.embedded_document && self.source_buffer().byte_position(comment.range.start).is_some_and(|start| offset <= start && start <= offset + line.len())) else { continue; };
            let comment_at = self.source_buffer().byte_position(comment.range.start).unwrap_or(offset) - offset; let trimmed = line.trim_start();
            if !self.config_bool("AllowBorderComment", true) && !trimmed.is_empty() && trimmed.bytes().all(|byte| byte == b'#') {
                let indent = line.len() - trimmed.len(); let offense_bytes = offset + indent..offset + line.len(); let edit_bytes = offset..offset + line.len() + usize::from(source.as_bytes().get(offset + line.len()) == Some(&b'\n'));
                let (Some(offense), Some(edit)) = (character_range(self, offense_bytes), character_range(self, edit_bytes)) else { continue; };
                add_offense!(self, offense, message: "Source code comment is empty.", |corrector| { corrector.remove(edit); }); continue;
            }
            let inline = (!line[..comment_at].trim().is_empty() && line[comment_at + 1..].trim().is_empty()).then_some(comment_at);
            if !matches!(trimmed.trim_end(), "#" | "# ") && inline.is_none() { continue; }
            if inline.is_none() && self.config_bool("AllowBorderComment", true) && self.config_bool("AllowMarginComment", true) {
                let column = line.len() - trimmed.len(); let same = |candidate: &str| candidate.trim_start().starts_with('#') && candidate.len() - candidate.trim_start().len() == column;
                let content = |candidate: &str| { let comment = candidate.trim_start(); comment.starts_with('#') && !matches!(comment.trim_end(), "#" | "# ") };
                let mut first = index; while first > 0 && same(lines[first - 1].1) { first -= 1; } let mut last = index + 1; while last < lines.len() && same(lines[last].1) { last += 1; }
                if lines[first..last].iter().any(|(_, line)| content(line)) { continue; }
            }
            let indent = inline.unwrap_or(line.len() - trimmed.len()); let edit_start = if inline.is_some() { offset + line[..indent].trim_end().len() } else { offset };
            let edit_end = offset + line.len() + usize::from(inline.is_none() && source.as_bytes().get(offset + line.len()) == Some(&b'\n'));
            let (Some(offense), Some(edit)) = (character_range(self, offset + indent..offset + line.len()), character_range(self, edit_start..edit_end)) else { continue; };
            add_offense!(self, offense, message: "Source code comment is empty.", |corrector| { corrector.remove(edit); });
        }
    }
}

fn magic_comment(line: &str) -> bool {
    let lower = line.trim().to_ascii_lowercase();
    lower.starts_with("# frozen_string_literal:") || lower.starts_with("# encoding:") || lower.starts_with("# coding:") || lower.starts_with("# -*-") && (lower.contains(" encoding:") || lower.contains(" coding:")) || lower.starts_with("# warn_indent:") || lower.starts_with("# shareable_constant_value:") || lower.starts_with("# typed:") || matches!(lower.as_str(), "# rbs_inline: enabled" | "# rbs_inline: disabled")
}

define_compatibility_rule!(EmptyLineAfterMagicCommentRule);
impl EmptyLineAfterMagicCommentRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let mut offset = 0usize; let mut last_magic_end = None;
        for (index, line) in self.source().split_inclusive('\n').enumerate() {
            let bare = line.trim_end_matches(['\n', '\r']); let trimmed = bare.trim();
            if index == 0 && trimmed.starts_with("#!") { offset += line.len(); continue; }
            if magic_comment(trimmed) { last_magic_end = Some(offset + line.len()); offset += line.len(); continue; }
            if trimmed.starts_with('#') && last_magic_end.is_some() { offset += line.len(); continue; }
            if trimmed.is_empty() { if last_magic_end.is_some() { break; } offset += line.len(); continue; }
            break;
        }
        let Some(at) = last_magic_end.filter(|at| *at < self.source().len()) else { return; };
        if self.source()[at..].starts_with('\n') || self.source()[at..].starts_with("\r\n") { return; }
        let Some(point) = character_range(self, at..at) else { return; };
        add_offense!(self, point, message: "Add an empty line after magic comments.", |corrector| { corrector.insert_before(point, "\n"); });
    }
}

define_compatibility_rule!(IndentationStyleRule);
impl IndentationStyleRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let spaces = self.policy().enforced_style("spaces") == "spaces"; let width = self.config_usize("IndentationWidth", 2);
        let ignored = super::source_rules_layout::ignored_syntax_ranges_from(
            self.source(),
            self.prism_result(),
        ); let mut offset = 0usize;
        for line in self.source().split_inclusive('\n') {
            let bare = line.trim_end_matches(['\n', '\r']); if bare.trim() == "__END__" { break; }
            let indentation = bare.len() - bare.trim_start_matches([' ', '\t']).len(); if indentation == 0 { offset += line.len(); continue; }
            let leading = &bare[..indentation];
            let end = if spaces && leading.contains('\t') { leading.rfind('\t').map_or(indentation, |tab| tab + 1) } else if !spaces && leading.contains(' ') { let count = leading.bytes().take_while(|byte| *byte == b' ').count(); if count > 0 { count } else { indentation } } else { offset += line.len(); continue; };
            let heredoc_closing = self.source()[..offset].rfind("<<").is_some_and(|marker| { let name = self.source()[marker + 2..].trim_start_matches(['-', '~', '\'', '"', '`']).chars().take_while(|character| character.is_ascii_alphanumeric() || *character == '_').collect::<String>(); !name.is_empty() && bare.trim_matches([' ', '\t', '\'', '"', '`']) == name });
            if !heredoc_closing && ignored.iter().any(|range| range.start <= offset && offset + end <= range.end) { offset += line.len(); continue; }
            let Some(offense) = character_range(self, offset..offset + end) else { offset += line.len(); continue; };
            let (message, replacement) = if spaces { ("Tab detected in indentation.", leading[..end].replace('\t', &" ".repeat(width))) } else { ("Space detected in indentation.", "\t".repeat(end / width)) };
            add_offense!(self, offense, message: message, |corrector| { corrector.replace(offense, replacement); }); offset += line.len();
        }
    }
}

define_compatibility_rule!(OrderedMagicCommentsRule);
impl OrderedMagicCommentsRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let mut offset = 0usize; let mut encoding = None; let mut frozen = None;
        for line in self.source().split_inclusive('\n') {
            let bare = line.trim_end_matches(['\n', '\r']); let trimmed = bare.trim();
            if !(trimmed.is_empty() || trimmed.starts_with('#')) { break; }
            if trimmed.starts_with("# encoding:") || trimmed.starts_with("# coding:") || trimmed.starts_with("# -*- encoding") { encoding = Some((offset, bare.to_owned())); }
            if trimmed.starts_with("# frozen_string_literal:") { frozen = Some((offset, bare.to_owned())); }
            offset += line.len();
        }
        let (Some((encoding_at, encoding_line)), Some((frozen_at, frozen_line))) = (encoding, frozen) else { return; };
        if encoding_at <= frozen_at { return; }
        let end = encoding_at + encoding_line.len(); let (Some(offense), Some(edit)) = (character_range(self, encoding_at..end), character_range(self, frozen_at..end)) else { return; };
        add_offense!(self, offense, message: "The encoding magic comment should precede all other magic comments.", |corrector| { corrector.replace(edit, format!("{encoding_line}\n{frozen_line}")); });
    }
}

define_compatibility_rule!(RequireOrderRule);
impl RequireOrderRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        return_if!(node.receiver().is_some()); let Some(argument) = node.first_argument().filter(|argument| argument.kind() == "str") else { return; };
        let key = self.string_content(argument).unwrap_or_default().into_owned(); let Some(range) = node.source_range() else { return; };
        let method = node.method_name().unwrap_or_default();
        let message = format!("Sort `{method}` in alphabetical order.");
        let lines = self.source().split_inclusive('\n').scan(0usize, |offset, line| { let start = *offset; *offset += line.len(); Some((start, line.trim_end_matches(['\n', '\r']))) }).collect::<Vec<_>>();
        let line_index = lines.iter().position(|(offset, line)| *offset <= self.source_buffer().byte_position(range.start).unwrap_or(0) && self.source_buffer().byte_position(range.start).unwrap_or(0) <= *offset + line.len()).unwrap_or(0);
        let eligible = |line: &str| { let trimmed = line.trim_start(); trimmed.starts_with('#') || trimmed.starts_with(&format!("{method} ")) };
        let mut first = line_index; while first > 0 && eligible(lines[first - 1].1) && !lines[first - 1].1.trim().is_empty() { first -= 1; }
        let mut last = line_index + 1; while last < lines.len() && eligible(lines[last].1) && !lines[last].1.trim().is_empty() { last += 1; }
        let mut older = false;
        for (_, line) in lines[first..line_index].iter().rev() {
            let trimmed = line.trim_start(); if trimmed.starts_with('#') { continue; }
            let quoted = trimmed.strip_prefix(&format!("{method} ")).and_then(|rest| matches!(rest.chars().next(), Some('\'' | '"')).then(|| rest[1..].split(rest.chars().next().unwrap()).next().unwrap_or_default()));
            let Some(previous) = quoted else { break; }; if key.as_str() < previous { older = true; break; }
        }
        if !older { return; }
        let mut pending = Vec::new(); let mut units = Vec::<(String, Vec<String>)>::new();
        for (_, line) in &lines[first..last] { if line.trim_start().starts_with('#') { pending.push((*line).to_owned()); continue; } let key = line.split(['\'', '"']).nth(1).unwrap_or_default().to_owned(); pending.push((*line).to_owned()); units.push((key, std::mem::take(&mut pending))); }
        units.sort_by(|left, right| left.0.cmp(&right.0)); let replacement = units.into_iter().flat_map(|(_, lines)| lines).chain(pending).collect::<Vec<_>>().join("\n");
        let block_start = lines[first].0; let block_end = lines[last - 1].0 + lines[last - 1].1.len();
        let (Some(offense), Some(edit)) = (character_range(self, self.source_buffer().byte_position(range.start).unwrap_or(0)..self.source_buffer().byte_position(range.end).unwrap_or(0)), character_range(self, block_start..block_end)) else { return; };
        add_offense!(self, offense, message: message, |corrector| { corrector.replace(edit, replacement); });
    }
}

define_compatibility_rule!(TrailingCommaInBlockArgsRule);
impl TrailingCommaInBlockArgsRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        let Some(arguments) = node.arguments_node() else { return; }; let Some(source) = arguments.source() else { return; };
        let Some(first) = source.find('|') else { return; }; let Some(last) = source.rfind('|').filter(|last| *last > first) else { return; };
        let inner = &source[first + 1..last]; let count = arguments.child_nodes().iter().filter(|argument| argument.name().is_some()).count();
        let chained_destructuring = inner.trim_start().starts_with('(') || count <= 1 && node.parent().is_some_and(|call| {
            call.receiver().is_some_and(|receiver| receiver.source_range() == node.source_range())
                && call.parent().is_some_and(|block| block.arguments_node().is_some_and(|arguments| arguments.child_nodes().iter().filter(|argument| argument.name().is_some()).count() > 1))
        });
        if count <= 1 && !chained_destructuring || inner.contains(';') || !inner.trim_end().ends_with(',') { return; }
        let Some(range) = arguments.source_range() else { return; }; let comma = range.start + first + 1 + inner.trim_end().len() - 1; let offense = self.range_between(comma, comma + 1);
        add_offense!(self, offense, message: "Useless trailing comma present in block arguments.", |corrector| { corrector.remove(offense); });
    }
}
