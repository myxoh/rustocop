use super::*;
use crate::rubocop::ast::node::core::NodeRef;

define_cops! {
    NumberedParameterAssignmentCompatibility => "Lint/NumberedParameterAssignment" => compatibility_callbacks(NumberedParameterAssignmentRule, [on_lvasgn]),
    NoReturnInBeginEndBlocksCompatibility => "Lint/NoReturnInBeginEndBlocks" => compatibility_callbacks(NoReturnInBeginEndBlocksRule, [on_casgn, on_cvasgn, on_gvasgn, on_ivasgn, on_lvasgn, on_op_asgn, on_or_asgn]),
    AddRuntimeDependencyCompatibility => "Gemspec/AddRuntimeDependency" => compatibility_callbacks(AddRuntimeDependencyRule, [on_send restrict ["add_runtime_dependency"]]),
    UselessConstantScopingCompatibility => "Lint/UselessConstantScoping" => compatibility_callbacks(UselessConstantScopingRule, [on_casgn]),
    AccessorMethodNameCompatibility => "Naming/AccessorMethodName" => compatibility_callbacks(AccessorMethodNameRule, [on_def, on_defs]),
    UriEscapeUnescapeCompatibility => "Lint/UriEscapeUnescape" => compatibility_callbacks(UriEscapeUnescapeRule, [on_send restrict ["escape", "encode", "unescape", "decode"]]),
    ConstantNameCompatibility => "Naming/ConstantName" => compatibility_callbacks(ConstantNameRule, [on_casgn]),
    MissingRespondToMissingCompatibility => "Style/MissingRespondToMissing" => compatibility_callbacks(MissingRespondToMissingRule, [on_def, on_defs]),
    CollectionLiteralLengthCompatibility => "Metrics/CollectionLiteralLength" => compatibility_callbacks(CollectionLiteralLengthRule, [on_array, on_hash, on_index, on_send]),
    FileOpenCompatibility => "Style/FileOpen" => compatibility_callbacks(FileOpenRule, [on_send restrict ["open"]]),
    RubyVersionGlobalsUsageCompatibility => "Gemspec/RubyVersionGlobalsUsage" => compatibility_callbacks(RubyVersionGlobalsUsageRule, [on_const]),
    LiteralAssignmentInConditionCompatibility => "Lint/LiteralAssignmentInCondition" => compatibility_callbacks(LiteralAssignmentInConditionRule, [on_if, on_until, on_while]),
    NonLocalExitFromIteratorCompatibility => "Lint/NonLocalExitFromIterator" => compatibility_callbacks(NonLocalExitFromIteratorRule, [on_return]),
    TopLevelMethodDefinitionCompatibility => "Style/TopLevelMethodDefinition" => compatibility_callbacks(TopLevelMethodDefinitionRule, [on_block, on_def, on_defs, on_itblock, on_numblock, on_send]),
    HeredocDelimiterNamingCompatibility => "Naming/HeredocDelimiterNaming" => compatibility_callbacks(HeredocDelimiterNamingRule, [on_heredoc]),
    MethodParameterNameCompatibility => "Naming/MethodParameterName" => compatibility_callbacks(MethodParameterNameRule, [on_def, on_defs]),
    UselessRescueCompatibility => "Lint/UselessRescue" => compatibility_callbacks(UselessRescueRule, [on_rescue]),
}

define_compatibility_rule!(NumberedParameterAssignmentRule);
impl NumberedParameterAssignmentRule<'_, '_, '_, '_> {
    fn on_lvasgn(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.name() else { return; };
        let Some(digits) = name.strip_prefix('_').filter(|digits| !digits.is_empty() && digits.chars().all(|character| character.is_ascii_digit())) else { return; };
        let Ok(number) = digits.parse::<usize>() else { return; };
        let message = if (1..=9).contains(&number) {
            format!("`_{number}` is reserved for numbered parameter; consider another name.")
        } else {
            format!("`_{number}` is similar to numbered parameter; consider another name.")
        };
        self.report(message, node);
    }
}

define_compatibility_rule!(NoReturnInBeginEndBlocksRule);
impl NoReturnInBeginEndBlocksRule<'_, '_, '_, '_> {
    fn on_casgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); }
    fn on_cvasgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); }
    fn on_gvasgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); }
    fn on_ivasgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); }
    fn on_lvasgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); }
    fn on_op_asgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); }
    fn on_or_asgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); }

    fn check_assignment(&mut self, node: NodeRef<'_>) {
        for begin in node.each_descendant(&["kwbegin"]) {
            for return_node in begin.each_descendant(&["return"]) {
                self.report("Do not `return` in `begin..end` blocks in assignment contexts.", return_node);
            }
        }
    }
}

define_compatibility_rule!(AddRuntimeDependencyRule);
impl AddRuntimeDependencyRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let path = self.processed_source().file_path();
        if path != "(string)" && !path.ends_with(".gemspec") || node.arguments().is_empty() { return; }
        let Some(selector) = self.location_range(node, "selector") else { return; };
        add_offense!(self, selector, message: "Use `add_dependency` instead of `add_runtime_dependency`.", |corrector| {
            corrector.replace(selector, "add_dependency");
        });
    }
}

define_compatibility_rule!(UselessConstantScopingRule);
impl UselessConstantScopingRule<'_, '_, '_, '_> {
    fn on_casgn(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.name() else { return; };
        let after_private = node.left_siblings().into_iter().rev().find(|sibling| {
            sibling.kind() != "casgn" && !(sibling.kind() == "send" && sibling.method_name() == Some("private_constant"))
        }).is_some_and(|candidate| candidate.kind() == "send" && candidate.receiver().is_none() && candidate.method_name() == Some("private") && candidate.arguments().is_empty());
        if !after_private { return; }
        let explicitly_private = node.right_siblings().into_iter().filter(|sibling| sibling.kind() == "send" && sibling.receiver().is_none() && sibling.method_name() == Some("private_constant")).flat_map(NodeRef::arguments).any(|argument| argument.scalar_value_text().as_deref() == Some(name));
        if !explicitly_private {
            self.report("Useless `private` access modifier for constant scope.", node);
        }
    }
}

define_compatibility_rule!(UselessElseWithoutRescueRule);
impl UselessElseWithoutRescueRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let diagnostics = self.processed_source().diagnostics().to_vec();
        let mut found = false;
        for diagnostic in diagnostics {
            if diagnostic.message.contains("else") && diagnostic.message.contains("rescue") {
                let range = self.owned_character_range(diagnostic.range);
                self.report("`else` without `rescue` is useless.", range);
                found = true;
            }
        }
        if found { return; }
        let mut scopes = Vec::<(&'static str, bool)>::new();
        let mut position = 0;
        for line in self.source().split_inclusive('\n') {
            let trimmed = line.trim_start();
            let indent = line.len() - trimmed.len();
            let opener = if trimmed.starts_with("begin") { Some("begin") }
                else if trimmed.starts_with("if ") || trimmed.starts_with("unless ") || trimmed.starts_with("case ") || trimmed.starts_with("while ") || trimmed.starts_with("until ") || trimmed.starts_with("for ") { Some("control") }
                else if trimmed.starts_with("def ") || trimmed.starts_with("class ") || trimmed.starts_with("module ") || trimmed.ends_with(" do\n") || trimmed.contains(" do |") { Some("scope") }
                else { None };
            if let Some(kind) = opener {
                scopes.push((kind, false));
            } else if trimmed.starts_with("rescue") && scopes.last().is_some_and(|scope| scope.0 == "begin") {
                if let Some(scope) = scopes.last_mut() { scope.1 = true; }
            } else if trimmed.starts_with("else") && scopes.last() == Some(&("begin", false)) {
                let range = self.range_between(position + indent, position + indent + 4);
                self.report("`else` without `rescue` is useless.", range);
            } else if trimmed.starts_with("end") && !scopes.is_empty() {
                scopes.pop();
            }
            position += line.chars().count();
        }
    }
}

define_compatibility_rule!(AccessorMethodNameRule);
impl AccessorMethodNameRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn check_definition(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.name().filter(|name| !name.ends_with(['!', '?', '='])) else { return; };
        let arguments = node.arguments();
        let message = if name.starts_with("get_") && arguments.is_empty() {
            Some("Do not prefix reader method names with `get_`.")
        } else if name.starts_with("set_") && arguments.len() == 1 && arguments[0].kind() == "arg" {
            Some("Do not prefix writer method names with `set_`.")
        } else { None };
        if let (Some(message), Some(name_range)) = (message, self.location_range(node, "name")) {
            self.report(message, name_range);
        }
    }
}

define_compatibility_rule!(SpaceAfterColonRule);
impl SpaceAfterColonRule<'_, '_, '_, '_> {
    fn on_pair(&mut self, node: NodeRef<'_>) {
        if let Some(colon) = self.location_range(node, "operator").filter(|range| self.range_source(range) == ":").or_else(|| node.key().and_then(|key| self.location_range(key, "end")).filter(|range| self.range_source(range) == ":")) { self.register_offense(colon); }
    }
    fn on_kwoptarg(&mut self, node: NodeRef<'_>) {
        if let Some(mut colon) = self.location_range(node, "operator") {
            if colon.begin_pos() == colon.end_pos() && colon.begin_pos() > 0 { colon = self.range_between(colon.begin_pos() - 1, colon.begin_pos()); }
            self.register_offense(colon);
        }
    }
    fn register_offense(&mut self, colon: CompatibilitySourceRange) {
        let after = self.range_between(colon.end_pos(), colon.end_pos() + 1);
        if self.range_source(&after).chars().next().is_some_and(|character| !character.is_whitespace() && !matches!(character, ',' | '}' | ')' | ']')) {
            add_offense!(self, colon, message: "Space missing after colon.", |corrector| {
                corrector.insert_after(colon, " ");
            });
        }
    }
}

define_compatibility_rule!(UriEscapeUnescapeRule);
impl UriEscapeUnescapeRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(receiver) = node.receiver().filter(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("URI") && receiver.namespace().is_none_or(|namespace| namespace.kind() == "cbase")) else { return; };
        let method = node.method_name().unwrap_or_default();
        let alternatives = if matches!(method, "escape" | "encode") { "`CGI.escape`, `URI.encode_www_form` or `URI.encode_www_form_component`" } else { "`CGI.unescape`, `URI.decode_www_form` or `URI.decode_www_form_component`" };
        self.report(format!("`{}.{method}` method is obsolete and should not be used. Instead, use {alternatives} depending on your specific use case.", receiver.source().unwrap_or_default()), node);
    }
}

define_compatibility_rule!(GlobalVarsRule);
impl GlobalVarsRule<'_, '_, '_, '_> {
    fn on_gvar(&mut self, node: NodeRef<'_>) { self.check_variable(node); }
    fn on_gvasgn(&mut self, node: NodeRef<'_>) { self.check_variable(node); }
    fn check_variable(&mut self, node: NodeRef<'_>) {
        if node.kind() == "gvar" && node.parent().is_some_and(|parent| parent.kind() == "gvasgn") { return; }
        let Some(mut name) = node.name() else { return; };
        if name == "$" { name = node.source().unwrap_or(name); }
        if !GLOBAL_BUILT_INS.contains(&name) && !self.config_values("AllowedVariables").iter().any(|allowed| allowed == name) {
            let offense = self.location_range(node, "name").filter(|range| range.begin_pos() < range.end_pos()).unwrap_or_else(|| self.owned_character_range(node.source_range().unwrap_or_default()));
            self.report("Do not introduce global variables.", offense);
        }
    }
}

define_compatibility_rule!(ConstantNameRule);
impl ConstantNameRule<'_, '_, '_, '_> {
    fn on_casgn(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.name().filter(|name| !name.chars().all(|character| character == '_' || character.is_uppercase() || character.is_ascii_digit())) else { return; };
        let value = node.parent().filter(|parent| parent.kind() == "or_asgn").and_then(NodeRef::expression).or_else(|| node.expression());
        if value.is_some_and(constant_name_allowed_assignment) { return; }
        if let Some(name_range) = self.location_range(node, "name") {
            self.report(format!("Use SCREAMING_SNAKE_CASE for constants. `{name}` should be `{}`.", screaming_snake_case(name)), name_range);
        }
    }
}

fn screaming_snake_case(name: &str) -> String {
    let mut result = String::new();
    for character in name.chars() {
        if character.is_ascii_uppercase() && result.chars().next_back().is_some_and(|previous| previous.is_ascii_lowercase() || previous.is_ascii_digit()) { result.push('_'); }
        result.push(character.to_ascii_uppercase());
    }
    result
}

fn constant_name_allowed_assignment(value: NodeRef<'_>) -> bool {
    if matches!(value.kind(), "block" | "const" | "casgn") { return true; }
    if value.kind() == "if" && value.branches().into_iter().flatten().any(|branch| branch.kind() == "const") { return true; }
    if matches!(value.kind(), "send" | "csend") {
        let receiver = value.receiver();
        if receiver.is_none() || receiver.is_some_and(|receiver| !receiver.recursive_literal()) { return true; }
        if value.method_name() == Some("new") && receiver.is_some_and(|receiver| receiver.kind() == "const" && matches!(receiver.short_name(), Some("Class" | "Struct"))) { return true; }
    }
    false
}

const GLOBAL_BUILT_INS: &[&str] = &[
    "$:", "$LOAD_PATH", "$\"", "$LOADED_FEATURES", "$0", "$PROGRAM_NAME", "$!", "$ERROR_INFO", "$@", "$ERROR_POSITION", "$;", "$FS", "$FIELD_SEPARATOR", "$,", "$OFS", "$OUTPUT_FIELD_SEPARATOR", "$/", "$RS", "$INPUT_RECORD_SEPARATOR", "$\\", "$ORS", "$OUTPUT_RECORD_SEPARATOR", "$.", "$NR", "$INPUT_LINE_NUMBER", "$_", "$LAST_READ_LINE", "$>", "$DEFAULT_OUTPUT", "$<", "$DEFAULT_INPUT", "$$", "$PID", "$PROCESS_ID", "$?", "$CHILD_STATUS", "$~", "$LAST_MATCH_INFO", "$=", "$IGNORECASE", "$*", "$ARGV", "$&", "$MATCH", "$`", "$PREMATCH", "$'", "$POSTMATCH", "$+", "$LAST_PAREN_MATCH", "$stdin", "$stdout", "$stderr", "$DEBUG", "$FILENAME", "$VERBOSE", "$SAFE", "$-0", "$-a", "$-d", "$-F", "$-i", "$-I", "$-l", "$-p", "$-v", "$-w", "$CLASSPATH", "$JRUBY_VERSION", "$JRUBY_REVISION", "$ENV_JAVA",
];

define_compatibility_rule!(MissingRespondToMissingRule);
impl MissingRespondToMissingRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn check_definition(&mut self, node: NodeRef<'_>) {
        if node.method_name() != Some("method_missing") { return; }
        let singleton = node.kind() == "defs";
        let Some(scope) = node.ancestors().into_iter().find(|ancestor| matches!(ancestor.kind(), "class" | "module" | "sclass" | "block" | "numblock" | "itblock")) else { return; };
        let search_scope = scope.ancestors().into_iter().find(|ancestor| matches!(ancestor.kind(), "class" | "module" | "sclass")).unwrap_or(scope);
        let implemented = std::iter::once(search_scope).chain(search_scope.descendants()).any(|candidate| candidate.method_name() == Some("respond_to_missing?") && (candidate.kind() == "defs") == singleton);
        if !implemented { self.report("When using `method_missing`, define `respond_to_missing?`.", node); }
    }
}

define_compatibility_rule!(CollectionLiteralLengthRule);
impl CollectionLiteralLengthRule<'_, '_, '_, '_> {
    fn on_array(&mut self, node: NodeRef<'_>) { self.check_collection(node, node.child_nodes().len()); }
    fn on_hash(&mut self, node: NodeRef<'_>) { self.check_collection(node, node.pairs().len()); }
    fn on_index(&mut self, node: NodeRef<'_>) { self.check_set(node); }
    fn on_send(&mut self, node: NodeRef<'_>) { self.check_set(node); }
    fn check_set(&mut self, node: NodeRef<'_>) {
        if node.method_name() == Some("[]") && node.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("Set")) { self.check_collection(node, node.arguments().len()); }
    }
    fn check_collection(&mut self, node: NodeRef<'_>, count: usize) {
        if count >= self.config_usize("LengthThreshold", 250) { self.report("Avoid hard coding large quantities of data in code. Prefer reading the data from an external source.", node); }
    }
}

define_compatibility_rule!(FileOpenRule);
impl FileOpenRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.kind() != "send" || node.block_literal() || node.arguments().iter().any(|argument| argument.kind() == "block_pass") || !node.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("File") && receiver.namespace().is_none_or(|namespace| namespace.kind() == "cbase")) { return; }
        let unsafe_use = node.parent().is_none_or(|parent| parent.kind() == "lvasgn" || matches!(parent.kind(), "send" | "csend") && parent.receiver().is_some_and(|receiver| receiver.id() == node.id()) || parent.kind() == "begin" && parent.child_nodes().last().is_none_or(|last| last.id() != node.id()));
        if unsafe_use { self.report("`File.open` without a block may leak a file descriptor; use the block form.", node); }
    }
}

define_compatibility_rule!(RubyVersionGlobalsUsageRule);
impl RubyVersionGlobalsUsageRule<'_, '_, '_, '_> {
    fn on_const(&mut self, node: NodeRef<'_>) {
        let path = self.processed_source().file_path();
        if path != "(string)" && !path.ends_with(".gemspec") { return; }
        let source = node.source().unwrap_or_default();
        if matches!(source, "RUBY_VERSION" | "::RUBY_VERSION" | "Ruby::VERSION" | "::Ruby::VERSION") { self.report(format!("Do not use `{source}` in gemspec file."), node); }
    }
}

define_compatibility_rule!(LiteralAssignmentInConditionRule);
impl LiteralAssignmentInConditionRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) { self.check_condition(node); }
    fn on_until(&mut self, node: NodeRef<'_>) { self.check_condition(node); }
    fn on_while(&mut self, node: NodeRef<'_>) { self.check_condition(node); }
    fn check_condition(&mut self, node: NodeRef<'_>) {
        let Some(condition) = node.condition() else { return; };
        for assignment in std::iter::once(condition).chain(condition.descendants()).filter(|candidate| matches!(candidate.kind(), "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn" | "casgn")) {
            let Some(value) = assignment.expression().filter(|value| value.recursive_basic_literal() && !matches!(value.kind(), "dstr" | "xstr")) else { continue; };
            let Some(operator) = self.location_range(assignment, "operator") else { continue; };
            let end = value.source_range().map_or(operator.end_pos(), |range| range.end);
            let offense = self.range_between(operator.begin_pos(), end);
            self.report(format!("Don't use literal assignment `= {}` in conditional, should be `==` or non-literal operand.", value.source().unwrap_or_default()), offense);
        }
    }
}

define_compatibility_rule!(NonLocalExitFromIteratorRule);
impl NonLocalExitFromIteratorRule<'_, '_, '_, '_> {
    fn on_return(&mut self, node: NodeRef<'_>) {
        if !node.arguments().is_empty() { return; }
        for ancestor in node.ancestors() {
            if matches!(ancestor.kind(), "def" | "defs") || ancestor.method_name() == Some("lambda") || matches!(ancestor.method_name(), Some("define_method" | "define_singleton_method")) { break; }
            if matches!(ancestor.kind(), "block" | "numblock" | "itblock") && !ancestor.arguments().is_empty() && ancestor.send_node().and_then(NodeRef::receiver).is_some() {
                if let Some(keyword) = self.location_range(node, "keyword") { self.report("Non-local exit from iterator, without return value. `next`, `break`, `Array#find`, `Array#any?`, etc. is preferred.", keyword); }
                break;
            }
        }
    }
}

define_compatibility_rule!(TopLevelMethodDefinitionRule);
impl TopLevelMethodDefinitionRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_block(&mut self, node: NodeRef<'_>) { self.check_dynamic(node); }
    fn on_numblock(&mut self, node: NodeRef<'_>) { self.check_dynamic(node); }
    fn on_itblock(&mut self, node: NodeRef<'_>) { self.check_dynamic(node); }
    fn on_send(&mut self, _node: NodeRef<'_>) {}
    fn check_dynamic(&mut self, node: NodeRef<'_>) { if node.method_name() == Some("define_method") { self.check(node); } }
    fn check(&mut self, node: NodeRef<'_>) {
        if node.ancestors().into_iter().all(|ancestor| ancestor.kind() == "begin") { self.report("Do not define methods at the top-level.", node); }
    }
}

define_compatibility_rule!(HeredocDelimiterNamingRule);
impl HeredocDelimiterNamingRule<'_, '_, '_, '_> {
    fn on_heredoc(&mut self, node: NodeRef<'_>) {
        let Some(end) = self.location_range(node, "heredoc_end") else { return; };
        let delimiter = self.range_source(&end).trim();
        let forbidden = self.config_values("ForbiddenDelimiters");
        let default_forbidden = delimiter.eq_ignore_ascii_case("END") || delimiter.len() == 3 && delimiter.to_ascii_uppercase().starts_with("EO");
        if delimiter.is_empty() || if forbidden.is_empty() { default_forbidden } else { forbidden.iter().any(|value| value == delimiter) } { self.report("Use meaningful heredoc delimiters.", end); }
    }
}

define_compatibility_rule!(MethodParameterNameRule);
impl MethodParameterNameRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn check_definition(&mut self, node: NodeRef<'_>) {
        let minimum = self.config_usize("MinNameLength", 3);
        let allow_numbers = self.config_bool("AllowNamesEndingInNumbers", false);
        for parameter in node.arguments() {
            if !matches!(parameter.kind(), "arg" | "optarg" | "kwarg" | "kwoptarg") { continue; }
            let Some(name) = parameter.name() else { continue; };
            let normalized = name.trim_start_matches('_');
            if normalized.is_empty() || self.config_values("AllowedNames").iter().any(|allowed| allowed == normalized) { continue; }
            let message = if self.config_values("ForbiddenNames").iter().any(|forbidden| forbidden == normalized) { Some(format!("Do not use {normalized} as a name for a method parameter.")) } else if normalized.chars().count() < minimum { Some(format!("Method parameter must be at least {minimum} characters long.")) } else if normalized.chars().any(char::is_uppercase) { Some("Only use lowercase characters for method parameter.".to_owned()) } else if !allow_numbers && normalized.ends_with(|character: char| character.is_ascii_digit()) { Some("Do not end method parameter with a number.".to_owned()) } else { None };
            if let Some(message) = message {
                let range = parameter.source_range().unwrap_or_default();
                let offense = self.range_between(range.start, range.start + name.chars().count());
                self.report(message, offense);
            }
        }
    }
}

define_compatibility_rule!(UselessRescueRule);
impl UselessRescueRule<'_, '_, '_, '_> {
    fn on_rescue(&mut self, node: NodeRef<'_>) {
        let bodies = node.each_descendant(&["resbody"]);
        let Some(body) = bodies.last().copied().filter(|_| bodies.len() == 1) else { return; };
        let statements = body.body().map_or_else(Vec::new, |body| if body.kind() == "begin" { body.child_nodes() } else { vec![body] });
        let Some(raise) = statements.first().copied().filter(|_| statements.len() == 1).filter(|raise| raise.method_name() == Some("raise") && raise.receiver().is_none()) else { return; };
        if !raise.arguments().is_empty() { return; }
        if body.node_child(1).is_some() && node.ancestors().into_iter().any(|ancestor| ancestor.ensure_node().is_some()) { return; }
        self.report("Useless `rescue` detected.", body);
    }
}

define_compatibility_rule!(CopDirectiveSyntaxRule);
impl CopDirectiveSyntaxRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let comments = self.processed_source().comments().to_vec();
        for comment in comments {
            let Some(marker) = comment.text.find("rubocop:") else { continue; };
            let directive = comment.text[marker + 8..].trim();
            let mut parts = directive.splitn(2, char::is_whitespace);
            let mode = parts.next().unwrap_or_default();
            let names = parts.next().unwrap_or_default().trim();
            let detail = if mode.is_empty() { Some("The mode name is missing.") } else if !matches!(mode, "enable" | "disable" | "todo" | "push" | "pop") { Some("The mode name must be one of `enable`, `disable`, `todo`, `push`, or `pop`.") } else if !matches!(mode, "push" | "pop") && names.is_empty() { Some("The cop name is missing.") } else { None };
            if let Some(detail) = detail { self.report(format!("Malformed directive comment detected. {detail}"), &comment); }
        }
    }
}

define_compatibility_rule!(OpenRule);
impl OpenRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let receiver = match node.receiver() { None => "Kernel#", Some(receiver) if receiver.kind() == "const" && receiver.short_name() == Some("URI") && receiver.namespace().is_none_or(|namespace| namespace.kind() == "cbase") => if receiver.namespace().is_some() { "::URI." } else { "URI." }, _ => return };
        let Some(argument) = node.first_argument() else { return; };
        if argument.kind() == "str" && !argument.str_content().unwrap_or_default().starts_with('|') { return; }
        if let Some(selector) = self.location_range(node, "selector") { self.report(format!("The use of `{receiver}open` is a serious security risk."), selector); }
    }
}

define_compatibility_rule!(RequireParenthesesRule);
impl RequireParenthesesRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_csend(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        if node.arguments().is_empty() || self.location_range(node, "begin").is_some() { return; }
        let ambiguous = node.parent().is_some_and(|parent| parent.kind() == "if" && parent.ternary() || matches!(parent.kind(), "send" | "csend") && parent.receiver().is_none());
        if ambiguous { self.report("Use parentheses in the method call to avoid confusion about precedence.", node); }
    }
}

define_compatibility_rule!(UnexpectedBlockArityRule);
impl UnexpectedBlockArityRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_numblock(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_itblock(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let Some(method) = node.method_name() else { return; };
        let Some(expected) = self.config_map("Methods").and_then(|methods| methods.get(method)).and_then(|arity| arity.parse::<usize>().ok()) else { return; };
        let actual = node.arguments().iter().filter(|argument| !matches!(argument.kind(), "kwarg" | "kwoptarg" | "kwrestarg" | "blockarg")).count();
        if actual < expected && !node.arguments().iter().any(|argument| argument.kind() == "restarg") { self.report(format!("`{method}` expects at least {expected} positional arguments, got {actual}."), node.send_node().unwrap_or(node)); }
    }
}

define_compatibility_rule!(ModuleLengthRule);
impl ModuleLengthRule<'_, '_, '_, '_> {
    fn on_module(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_casgn(&mut self, node: NodeRef<'_>) { if node.expression().is_some_and(|value| value.method_name() == Some("new") && value.receiver().is_some_and(|receiver| receiver.short_name() == Some("Module"))) { self.check(node); } }
    fn check(&mut self, node: NodeRef<'_>) {
        let maximum = self.config_usize("Max", 100);
        let length = node.line_count().saturating_sub(2);
        if length > maximum { self.report(format!("Module has too many lines. [{length}/{maximum}]"), node); }
    }
}

define_compatibility_rule!(ConstantResolutionRule);
impl ConstantResolutionRule<'_, '_, '_, '_> {
    fn on_const(&mut self, node: NodeRef<'_>) {
        if node.namespace().is_some() || node.parent().is_some_and(|parent| matches!(parent.kind(), "class" | "module")) { return; }
        let Some(name) = node.short_name() else { return; };
        let only = self.config_values("Only"); let ignore = self.config_values("Ignore");
        if (!only.is_empty() && !only.iter().any(|allowed| allowed == name)) || ignore.iter().any(|ignored| ignored == name) { return; }
        self.report("Fully qualify this constant to avoid possibly ambiguous resolution.", node);
    }
}

define_compatibility_rule!(NestedPercentLiteralRule);
impl NestedPercentLiteralRule<'_, '_, '_, '_> {
    fn on_array(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_percent_literal(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        if !node.percent_literal(Some("string")) && !node.percent_literal(Some("symbol")) { return; }
        let nested = node.child_nodes().iter().any(|element| element.scalar_value_text().is_some_and(|value| ["%i", "%I", "%q", "%Q", "%r", "%s", "%w", "%W", "%x", "%"].iter().any(|prefix| value.starts_with(prefix))));
        if nested { self.report("Within percent literals, nested percent literals do not function and may be unwanted in the result.", node); }
    }
}

define_compatibility_rule!(ItAssignmentRule);
impl ItAssignmentRule<'_, '_, '_, '_> {
    fn on_arg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_blockarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_kwarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_kwoptarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_kwrestarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_lvasgn(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_optarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_restarg(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) { if node.name() == Some("it") { let offense = self.location_range(node, "name").unwrap_or_else(|| self.owned_character_range(node.source_range().unwrap_or_default())); self.report("`it` is the default block parameter; consider another name.", offense); } }
}

define_compatibility_rule!(EmptyBlockRule);
impl EmptyBlockRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        if node.body().is_some() || self.config_bool("AllowComments", true) && self.processed_source().comments().iter().any(|comment| node.source_range().is_some_and(|range| range.start <= comment.range.start && comment.range.end <= range.end)) { return; }
        if self.config_bool("AllowEmptyLambdas", true) && matches!(node.method_name(), Some("lambda" | "proc" | "new")) { return; }
        self.report("Empty block detected.", node);
    }
}

define_compatibility_rule!(MinMaxRule);
impl MinMaxRule<'_, '_, '_, '_> {
    fn on_array(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_return(&mut self, node: NodeRef<'_>) { for child in node.arguments() { self.check(child); } }
    fn check(&mut self, node: NodeRef<'_>) {
        if node.kind() != "array" || node.child_nodes().len() != 2 { return; }
        let first = node.child_nodes()[0]; let second = node.child_nodes()[1];
        if matches!((first.method_name(), second.method_name()), (Some("min"), Some("max"))) && first.receiver().zip(second.receiver()).is_some_and(|(left, right)| left.structurally_equal(right)) { self.report(format!("Use `{}.minmax` instead of `{}`.", first.receiver().and_then(NodeRef::source).unwrap_or_default(), node.source().unwrap_or_default()), node); }
    }
}
