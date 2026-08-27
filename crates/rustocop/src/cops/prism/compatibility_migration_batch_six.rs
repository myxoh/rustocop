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
    CopDirectiveSyntaxCompatibility => "Lint/CopDirectiveSyntax" => compatibility_investigation(CopDirectiveSyntaxRule, on_new_investigation),
    OpenCompatibility => "Security/Open" => compatibility_callbacks(OpenRule, [on_send restrict ["open"]]),
    RequireParenthesesCompatibility => "Lint/RequireParentheses" => compatibility_callbacks(RequireParenthesesRule, [on_csend, on_send]),
    UnexpectedBlockArityCompatibility => "Lint/UnexpectedBlockArity" => compatibility_callbacks(UnexpectedBlockArityRule, [on_block, on_itblock, on_numblock]),
    ConstantResolutionCompatibility => "Lint/ConstantResolution" => compatibility_callbacks(ConstantResolutionRule, [on_const]),
    NestedPercentLiteralCompatibility => "Lint/NestedPercentLiteral" => compatibility_callbacks(NestedPercentLiteralRule, [on_array, on_percent_literal]),
    ItAssignmentCompatibility => "Style/ItAssignment" => compatibility_callbacks(ItAssignmentRule, [on_arg, on_blockarg, on_kwarg, on_kwoptarg, on_kwrestarg, on_lvasgn, on_optarg, on_restarg]),
    EmptyBlockCompatibility => "Lint/EmptyBlock" => compatibility_callbacks(EmptyBlockRule, [on_block]),
    MinMaxCompatibility => "Style/MinMax" => compatibility_callbacks(MinMaxRule, [on_array, on_return]),
    EmptyClassCompatibility => "Lint/EmptyClass" => compatibility_callbacks(EmptyClassRule, [on_class, on_sclass]),
    BlockNestingCompatibility => "Metrics/BlockNesting" => compatibility_investigation(BlockNestingRule, on_new_investigation),
    KeywordArgumentsMergingCompatibility => "Style/KeywordArgumentsMerging" => compatibility_callbacks(KeywordArgumentsMergingRule, [on_kwsplat]),
    ToEnumArgumentsCompatibility => "Lint/ToEnumArguments" => compatibility_callbacks(ToEnumArgumentsRule, [on_send restrict ["to_enum", "enum_for"]]),
    TripleQuotesCompatibility => "Lint/TripleQuotes" => compatibility_callbacks(TripleQuotesRule, [on_dstr]),
    HashLikeCaseCompatibility => "Style/HashLikeCase" => compatibility_callbacks(HashLikeCaseRule, [on_case]),
    ConstantVisibilityCompatibility => "Style/ConstantVisibility" => compatibility_callbacks(ConstantVisibilityRule, [on_casgn]),
    ScriptPermissionCompatibility => "Lint/ScriptPermission" => compatibility_investigation(ScriptPermissionRule, on_new_investigation),
    FileTouchCompatibility => "Style/FileTouch" => compatibility_callbacks(FileTouchRule, [on_send restrict ["open"]]),
    UselessMethodDefinitionCompatibility => "Lint/UselessMethodDefinition" => compatibility_callbacks(UselessMethodDefinitionRule, [on_def, on_defs]),
    CompoundHashCompatibility => "Security/CompoundHash" => compatibility_callbacks(CompoundHashRule, [on_csend, on_op_asgn, on_send]),
    IneffectiveAccessModifierCompatibility => "Lint/IneffectiveAccessModifier" => compatibility_callbacks(IneffectiveAccessModifierRule, [on_class, on_module]),
    UselessRuby2KeywordsCompatibility => "Lint/UselessRuby2Keywords" => compatibility_callbacks(UselessRuby2KeywordsRule, [on_send restrict ["ruby2_keywords"]]),
    FloatComparisonCompatibility => "Lint/FloatComparison" => compatibility_callbacks(FloatComparisonRule, [on_case, on_csend, on_send]),
    ParameterListsCompatibility => "Metrics/ParameterLists" => compatibility_callbacks(ParameterListsRule, [on_args, on_def, on_defs]),
    MissingSuperCompatibility => "Lint/MissingSuper" => compatibility_callbacks(MissingSuperRule, [on_def, on_defs]),
    SuppressedExceptionCompatibility => "Lint/SuppressedException" => compatibility_callbacks(SuppressedExceptionRule, [on_resbody]),
    CircularArgumentReferenceCompatibility => "Lint/CircularArgumentReference" => compatibility_callbacks(CircularArgumentReferenceRule, [on_kwoptarg, on_optarg]),
    SelfAssignmentCompatibility => "Style/SelfAssignment" => compatibility_callbacks(SelfAssignmentRule, [on_cvasgn, on_ivasgn, on_lvasgn]),
    KeywordParametersOrderCompatibility => "Style/KeywordParametersOrder" => compatibility_callbacks(KeywordParametersOrderRule, [on_kwoptarg]),
    LoopCompatibility => "Lint/Loop" => compatibility_callbacks(LoopRule, [on_until_post, on_while_post]),
    RedundantConstantBaseCompatibility => "Style/RedundantConstantBase" => compatibility_callbacks(RedundantConstantBaseRule, [on_cbase]),
    GlobalStdStreamCompatibility => "Style/GlobalStdStream" => compatibility_callbacks(GlobalStdStreamRule, [on_const]),
    ArrayFirstLastCompatibility => "Style/ArrayFirstLast" => compatibility_callbacks(ArrayFirstLastRule, [on_csend, on_send]),
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
            let text = comment.text.trim_start().strip_prefix('#').unwrap_or(&comment.text).trim_start();
            let Some(directive) = text.strip_prefix("rubocop:").map(str::trim) else { continue; };
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
        if open_safe_argument(argument) { return; }
        if let Some(selector) = self.location_range(node, "selector") { self.report(format!("The use of `{receiver}open` is a serious security risk."), selector); }
    }
}

fn open_safe_argument(node: NodeRef<'_>) -> bool {
    if node.kind() == "str" { return node.str_content().is_some_and(|value| !value.is_empty() && !value.starts_with('|')); }
    if node.kind() == "dstr" { return node.first_node().is_some_and(open_safe_argument); }
    node.method_name() == Some("+") && node.receiver().is_some_and(|receiver| receiver.kind() == "str" && open_safe_argument(receiver))
}

define_compatibility_rule!(RequireParenthesesRule);
impl RequireParenthesesRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_csend(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        if node.arguments().is_empty() || self.location_range(node, "begin").is_some() { return; }
        let first = node.first_argument();
        if first.is_some_and(|argument| argument.kind() == "if" && argument.ternary() && !matches!(node.method_name(), Some("[]")) && !node.assignment_method() && argument.condition().is_some_and(NodeRef::operator_keyword)) {
            let condition = first.and_then(NodeRef::condition).unwrap_or(node);
            let start = node.source_range().map_or(0, |range| range.start);
            let end = condition.source_range().map_or(start, |range| range.end);
            let offense = self.range_between(start, end);
            self.report("Use parentheses in the method call to avoid confusion about precedence.", offense);
        } else if node.predicate_method() && node.last_argument().is_some_and(NodeRef::operator_keyword) {
            self.report("Use parentheses in the method call to avoid confusion about precedence.", node);
        }
    }
}

define_compatibility_rule!(UnexpectedBlockArityRule);
impl UnexpectedBlockArityRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_numblock(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_itblock(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let Some(send) = node.send_node() else { return; };
        let Some(method) = send.method_name() else { return; };
        let Some(expected) = self.config_map("Methods").and_then(|methods| methods.get(method)).and_then(|arity| arity.parse::<usize>().ok()) else { return; };
        if send.receiver().is_none() { return; }
        let arguments = node.arguments();
        if arguments.iter().any(|argument| argument.kind() == "restarg") { return; }
        let actual = if node.kind() == "numblock" {
            (1..=9).rev().find(|number| node.source().unwrap_or_default().contains(&format!("_{number}"))).unwrap_or(0)
        } else if node.kind() == "itblock" { 1 } else {
            arguments.iter().filter(|argument| matches!(argument.kind(), "arg" | "optarg" | "mlhs")).count()
        };
        if actual < expected { self.report(format!("`{method}` expects at least {expected} positional arguments, got {actual}."), node); }
    }
}

define_compatibility_rule!(ModuleLengthRule);
impl ModuleLengthRule<'_, '_, '_, '_> {
    fn on_module(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_casgn(&mut self, node: NodeRef<'_>) { if node.expression().is_some_and(|value| value.method_name() == Some("new") && value.receiver().is_some_and(|receiver| receiver.short_name() == Some("Module"))) { self.check(node); } }
    fn check(&mut self, node: NodeRef<'_>) {
        let maximum = self.config_usize("Max", 100);
        if !self.config_values("CountAsOne").is_empty() || node.descendants().into_iter().any(|child| matches!(child.kind(), "class" | "module")) { return; }
        let count_comments = self.config_bool("CountComments", false);
        let lines = node.source().unwrap_or_default().lines().collect::<Vec<_>>();
        let length = lines.iter().skip(1).take(lines.len().saturating_sub(2)).filter(|line| !line.trim().is_empty() && (count_comments || !line.trim_start().starts_with('#'))).count();
        if length > maximum {
            let offense = if node.kind() == "casgn" {
                let range = node.source_range().unwrap_or_default();
                self.location_range(node, "name").filter(|name| name.begin_pos() < name.end_pos()).unwrap_or_else(|| self.range_between(range.start, range.start + node.name().map_or(0, str::len)))
            } else { self.owned_character_range(node.source_range().unwrap_or_default()) };
            self.report(format!("Module has too many lines. [{length}/{maximum}]"), offense);
        }
    }
}

define_compatibility_rule!(ConstantResolutionRule);
impl ConstantResolutionRule<'_, '_, '_, '_> {
    fn on_const(&mut self, node: NodeRef<'_>) {
        if node.namespace().is_some() || node.parent().is_some_and(|parent| matches!(parent.kind(), "class" | "module") || parent.kind() == "casgn" && parent.expression().is_some_and(|value| value.method_name() == Some("new") && value.receiver().is_some_and(|receiver| matches!(receiver.short_name(), Some("Class" | "Module"))))) { return; }
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
        let nested = node.child_nodes().iter().any(|element| element.scalar_value_text().is_some_and(|value| ["%i", "%I", "%q", "%Q", "%r", "%s", "%w", "%W", "%x", "%"].iter().any(|prefix| value.strip_prefix(prefix).and_then(|rest| rest.chars().next()).is_some_and(|character| !character.is_ascii_alphanumeric() && character != '_'))));
        if nested { self.report("Within percent literals, nested percent literals do not function and may be unwanted in the result.", node); }
    }
}

define_compatibility_rule!(ItAssignmentRule);
impl ItAssignmentRule<'_, '_, '_, '_> {
    fn on_arg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_blockarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_kwarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_kwoptarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_kwrestarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_lvasgn(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_optarg(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_restarg(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        if node.name() != Some("it") { return; }
        let range = node.source_range().unwrap_or_default();
        let offense = if matches!(node.kind(), "restarg" | "kwrestarg" | "blockarg") {
            self.range_between(range.end.saturating_sub(2), range.end)
        } else if let Some(name) = self.location_range(node, "name").filter(|name| name.begin_pos() < name.end_pos()) {
            name
        } else {
            self.range_between(range.start, range.start + 2)
        };
        self.report("`it` is the default block parameter; consider another name.", offense);
    }
}

define_compatibility_rule!(EmptyBlockRule);
impl EmptyBlockRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        if node.body().is_some() || self.config_bool("AllowComments", true) && self.processed_source().comments().iter().any(|comment| node.source_range().is_some_and(|range| range.start <= comment.range.start && (comment.range.end <= range.end || comment.line == node.last_line()))) { return; }
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

define_compatibility_rule!(EmptyClassRule);
impl EmptyClassRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) {
        if node.body().is_none() && node.parent_class().is_none() && !self.body_or_allowed_comment_lines(node) {
            self.report("Empty class detected.", node);
        }
    }

    fn on_sclass(&mut self, node: NodeRef<'_>) {
        if node.body().is_none() && !self.body_or_allowed_comment_lines(node) {
            self.report("Empty metaclass detected.", node);
        }
    }

    fn body_or_allowed_comment_lines(&self, node: NodeRef<'_>) -> bool {
        node.body().is_some()
            || self.config_bool("AllowComments", false)
                && node.source_range().is_some_and(|range| {
                    self.processed_source().comments().iter().any(|comment| {
                        range.start <= comment.range.start && comment.range.end <= range.end
                    })
                })
    }
}

define_compatibility_rule!(BlockNestingRule);
impl BlockNestingRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        if self.source().trim().is_empty() { return; }
        let Some(root) = self.processed_source().ast() else { return; };
        let maximum = self.config_usize("Max", 3);
        self.check_nesting_level(root, maximum, 0, false);
    }

    fn check_nesting_level(&mut self, node: NodeRef<'_>, maximum: usize, mut level: usize, ignored: bool) {
        let considered = matches!(node.kind(), "case" | "case_match" | "if" | "while" | "while_post" | "until" | "until_post" | "for" | "resbody")
            || self.config_bool("CountBlocks", false) && matches!(node.kind(), "block" | "numblock" | "itblock");
        if considered && self.count_if_block(node) { level += 1; }
        let mut ignored = ignored;
        if considered && level > maximum && !ignored {
            self.report(format!("Avoid more than {maximum} levels of block nesting."), node);
            ignored = true;
        }
        for child in node.child_nodes() {
            self.check_nesting_level(child, maximum, level, ignored);
        }
    }

    fn count_if_block(&self, node: NodeRef<'_>) -> bool {
        node.kind() != "if" || !node.elsif() && (!node.modifier_form() || self.config_bool("CountModifierForms", false))
    }
}

define_compatibility_rule!(KeywordArgumentsMergingRule);
impl KeywordArgumentsMergingRule<'_, '_, '_, '_> {
    fn on_kwsplat(&mut self, node: NodeRef<'_>) {
        let Some(hash) = node.parent().filter(|parent| parent.kind() == "hash") else { return; };
        let Some(ancestor) = hash.parent().filter(|parent| matches!(parent.kind(), "send" | "csend")) else { return; };
        if ancestor.arguments().last().is_none_or(|last| last.id() != hash.id()) { return; }
        if hash.child_nodes().first().is_none_or(|first| first.id() != node.id()) { return; }
        let Some(merge) = node.child_nodes().first().copied().filter(|child| child.method_name() == Some("merge")) else { return; };
        let Some(receiver) = merge.receiver() else { return; };
        let arguments = merge.arguments();
        if arguments.is_empty() { return; }
        let mut pieces = vec![format!("**{}", receiver.source().unwrap_or_default())];
        for argument in arguments {
            if argument.kind() == "hash" {
                let source = argument.source().unwrap_or_default();
                pieces.push(if argument.braces() { source.get(1..source.len().saturating_sub(1)).unwrap_or_default().to_owned() } else { source.to_owned() });
            } else {
                pieces.push(format!("**{}", argument.source().unwrap_or_default()));
            }
        }
        let replacement = pieces.join(", ");
        add_offense!(self, merge, message: "Provide additional arguments directly rather than using `merge`.", |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(ToEnumArgumentsRule);
impl ToEnumArgumentsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.receiver().is_some_and(|receiver| receiver.kind() != "self") { return; }
        let Some(definition) = node.each_ancestor(&["def", "defs"]).first().copied() else { return; };
        let arguments = node.arguments();
        let Some(method_node) = arguments.first().copied() else { return; };
        let method_matches = matches!(method_node.method_name(), Some("__method__" | "__callee__"))
            || method_node.kind() == "sym" && method_node.scalar_value_text().as_deref() == definition.method_name();
        if !method_matches || self.arguments_match(&arguments[1..], definition) { return; }
        self.report("Ensure you correctly provided all the arguments.", node);
    }

    fn arguments_match(&self, arguments: &[NodeRef<'_>], definition: NodeRef<'_>) -> bool {
        let mut index = 0;
        for parameter in definition.arguments().into_iter().filter(|argument| argument.kind() != "blockarg") {
            let send_argument = arguments.get(index).copied();
            if matches!(parameter.kind(), "arg" | "restarg" | "optarg") { index += 1; }
            let Some(send_argument) = send_argument else { return false; };
            let name = parameter.name().unwrap_or_default();
            let matches = match parameter.kind() {
                "arg" | "restarg" => send_argument.source() == parameter.source(),
                "optarg" => send_argument.source() == Some(name),
                "kwarg" | "kwoptarg" => send_argument.kind() == "hash" && send_argument.pairs().iter().any(|pair| pair.key().and_then(NodeRef::scalar_value_text).as_deref() == Some(name) && pair.value_node().and_then(NodeRef::name) == Some(name)),
                "kwrestarg" => send_argument.each_child_node(&["kwsplat", "forwarded_kwrestarg"]).iter().any(|child| child.source() == parameter.source()),
                "forward_arg" => matches!(send_argument.kind(), "forward_args" | "forwarded_args"),
                _ => true,
            };
            if !matches { return false; }
        }
        true
    }
}

define_compatibility_rule!(TripleQuotesRule);
impl TripleQuotesRule<'_, '_, '_, '_> {
    fn on_dstr(&mut self, node: NodeRef<'_>) {
        let mut empty_strings = node.each_child_node(&["str"]).into_iter().filter(|child| child.str_content() == Some("")).collect::<Vec<_>>();
        if empty_strings.is_empty() { return; }
        let opening_quotes = node.source().unwrap_or_default().chars().take_while(|character| matches!(character, '\'' | '"')).count();
        if opening_quotes < 3 { return; }
        if empty_strings.len() == node.child_nodes().len() { empty_strings.remove(0); }
        add_offense!(self, node, message: "Delimiting a string with multiple quotes has no effect, use a single quote instead.", |corrector| {
            for string in empty_strings { corrector.remove(string); }
        });
    }
}

define_compatibility_rule!(HashLikeCaseRule);
impl HashLikeCaseRule<'_, '_, '_, '_> {
    fn on_case(&mut self, node: NodeRef<'_>) {
        if node.has_else() || node.condition().is_none() { return; }
        let branches = node.when_branches();
        if branches.len() < self.config_usize("MinBranchesCount", 3).max(2) { return; }
        let mut condition_kind = None;
        let mut body_kind = None;
        for branch in branches {
            let conditions = branch.conditions();
            let Some(condition) = conditions.first().copied().filter(|_| conditions.len() == 1 && matches!(conditions[0].kind(), "str" | "sym")) else { return; };
            let Some(body) = branch.body() else { return; };
            let statements = if body.kind() == "begin" { body.child_nodes() } else { vec![body] };
            let Some(value) = statements.first().copied().filter(|_| statements.len() == 1 && statements[0].recursive_basic_literal()) else { return; };
            if condition_kind.is_some_and(|kind| kind != condition.kind()) || body_kind.is_some_and(|kind| kind != value.kind()) { return; }
            condition_kind = Some(condition.kind());
            body_kind = Some(value.kind());
        }
        self.report("Consider replacing `case-when` with a hash lookup.", node);
    }
}

define_compatibility_rule!(ConstantVisibilityRule);
impl ConstantVisibilityRule<'_, '_, '_, '_> {
    fn on_casgn(&mut self, node: NodeRef<'_>) {
        let Some(parent) = node.parent() else { return; };
        let scope = if parent.kind() == "begin" { parent.parent() } else { Some(parent) };
        if !scope.is_some_and(|scope| matches!(scope.kind(), "class" | "module")) { return; }
        let Some(name) = node.short_name() else { return; };
        if self.visibility_declaration(node, name) { return; }
        if self.config_bool("IgnoreModules", false) && node.expression().is_some_and(NodeRef::class_constructor) { return; }
        self.report(format!("Explicitly make `{name}` public or private using either `#public_constant` or `#private_constant`."), node);
    }

    fn visibility_declaration(&self, node: NodeRef<'_>, name: &str) -> bool {
        let Some(parent) = node.parent() else { return false; };
        parent.child_nodes().into_iter().filter(|child| matches!(child.method_name(), Some("public_constant" | "private_constant")) && child.receiver().is_none()).flat_map(NodeRef::arguments).any(|argument| {
            argument.scalar_value_text().as_deref() == Some(name)
                || argument.kind() == "splat" && argument.descendants().into_iter().any(|child| child.scalar_value_text().as_deref() == Some(name))
        })
    }
}

define_compatibility_rule!(ScriptPermissionRule);
impl ScriptPermissionRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        if !self.source().starts_with("#!") { return; }
        let path = self.processed_source().file_path();
        if matches!(path, "(string)" | "-") { return; }
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            if !std::fs::metadata(path).is_ok_and(|metadata| metadata.permissions().mode() & 0o111 == 0) { return; }
            let file = std::path::Path::new(path).file_name().map_or_else(|| path.to_owned(), |name| name.to_string_lossy().into_owned());
            if let Some(comment) = self.processed_source().comments().first() {
                self.report(format!("Script file {file} doesn't have execute permission."), comment);
            }
        }
    }
}

define_compatibility_rule!(FileTouchRule);
impl FileTouchRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(receiver) = node.receiver().filter(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("File") && receiver.namespace().is_none_or(|namespace| namespace.kind() == "cbase")) else { return; };
        let _ = receiver;
        let arguments = node.arguments();
        let (Some(filename), Some(mode)) = (arguments.first().copied(), arguments.last().copied()) else { return; };
        if arguments.len() < 2 || mode.kind() != "str" || !matches!(mode.str_content(), Some("a" | "a+" | "ab" | "a+b" | "at" | "a+t")) { return; }
        let Some(block) = node.block_node().filter(|block| block.body().is_none()) else { return; };
        let argument = filename.source().unwrap_or_default();
        add_offense!(self, block, message: format!("Use `FileUtils.touch({argument})` instead of `File.open` in append mode with empty block."), |corrector| {
            corrector.replace(block, format!("FileUtils.touch({argument})"));
        });
    }
}

define_compatibility_rule!(UselessMethodDefinitionRule);
impl UselessMethodDefinitionRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check_definition(node); }

    fn check_definition(&mut self, node: NodeRef<'_>) {
        let modifier = node.parent().filter(|parent| parent.kind() == "send");
        if modifier.is_some_and(|parent| !matches!(parent.method_name(), Some("public" | "private" | "protected" | "module_function"))) { return; }
        if node.arguments().iter().any(|argument| matches!(argument.kind(), "restarg" | "optarg" | "kwoptarg")) { return; }
        let Some(body) = node.body() else { return; };
        let delegated = body.kind() == "zsuper" || body.kind() == "super" && body.arguments().iter().map(|argument| argument.source()).eq(node.arguments().iter().map(|argument| argument.source()));
        if !delegated { return; }
        let offense = modifier.unwrap_or(node);
        add_offense!(self, node, message: "Useless method definition detected.", |corrector| {
            corrector.remove(offense);
        });
    }
}

define_compatibility_rule!(CompoundHashRule);
impl CompoundHashRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_csend(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_op_asgn(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let combinator = if node.kind() == "op_asgn" {
            matches!(node.operator(), Some("^" | "+" | "*" | "|"))
        } else {
            matches!(node.method_name(), Some("^" | "+" | "*" | "|")) && node.arguments().len() == 1
        };
        if combinator && node.ancestors().into_iter().any(compound_hash_definition) && node.ancestors().into_iter().all(|ancestor| !matches!(ancestor.method_name(), Some("^" | "+" | "*" | "|"))) {
            self.report("Use `[...].hash` instead of combining hash values manually.", node);
        }
        if node.method_name() != Some("hash") { return; }
        if node.receiver().is_some_and(|receiver| receiver.kind() == "array" && receiver.child_nodes().len() == 1) {
            self.report("Delegate hash directly without wrapping in an array when only using a single value.", node);
        }
        if node.receiver().is_some_and(|receiver| receiver.kind() == "array") && node.receiver().into_iter().flat_map(NodeRef::child_nodes).any(|element| element.method_name() == Some("hash")) {
            for redundant in node.receiver().into_iter().flat_map(NodeRef::child_nodes).filter(|element| element.method_name() == Some("hash")) {
                self.report("Calling .hash on elements of a hashed array is redundant.", redundant);
            }
        }
    }
}

fn compound_hash_definition(node: NodeRef<'_>) -> bool {
    matches!(node.kind(), "def" | "defs") && node.method_name() == Some("hash") && node.arguments().is_empty()
        || matches!(node.kind(), "block" | "numblock" | "itblock")
            && node.send_node().is_some_and(|send| matches!(send.method_name(), Some("define_method" | "define_singleton_method")) && send.first_argument().and_then(NodeRef::scalar_value_text).as_deref() == Some("hash"))
            && node.arguments().is_empty()
}

define_compatibility_rule!(IneffectiveAccessModifierRule);
impl IneffectiveAccessModifierRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) { self.check_scope(node); }
    fn on_module(&mut self, node: NodeRef<'_>) { self.check_scope(node); }
    fn check_scope(&mut self, node: NodeRef<'_>) {
        let Some(body) = node.body().filter(|body| body.kind() == "begin") else { return; };
        let ignored = body.child_nodes().into_iter().filter(|child| child.receiver().is_none() && child.method_name() == Some("private_class_method")).flat_map(NodeRef::arguments).filter_map(NodeRef::scalar_value_text).collect::<Vec<_>>();
        self.check_children(body, &ignored, None);
    }
    fn check_children<'a>(&mut self, node: NodeRef<'a>, ignored: &[String], mut modifier: Option<NodeRef<'a>>) {
        for child in node.child_nodes() {
            if child.kind() == "send" && child.receiver().is_none() && matches!(child.method_name(), Some("public" | "private" | "protected")) && child.arguments().is_empty() {
                modifier = Some(child);
            } else if child.kind() == "defs" {
                let Some(modifier) = modifier.filter(|modifier| modifier.method_name() != Some("public")) else { continue; };
                if ignored.iter().any(|name| Some(name.as_str()) == child.method_name()) { continue; }
                let visibility = modifier.method_name().unwrap_or_default();
                let alternative = if visibility == "private" { "`private_class_method` or `private` inside a `class << self` block" } else { "`protected` inside a `class << self` block" };
                if let Some(keyword) = self.location_range(child, "keyword") {
                    self.report(format!("`{visibility}` (on line {}) does not make singleton methods {visibility}. Use {alternative} instead.", modifier.first_line()), keyword);
                }
            } else if child.kind() == "kwbegin" {
                self.check_children(child, ignored, modifier);
            }
        }
    }
}

define_compatibility_rule!(UselessRuby2KeywordsRule);
impl UselessRuby2KeywordsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(argument) = node.first_argument() else { return; };
        let definition = if matches!(argument.kind(), "def" | "defs") { Some(argument) } else if argument.kind() == "sym" {
            let name = argument.scalar_value_text();
            node.ancestors().into_iter().flat_map(NodeRef::child_nodes).find(|candidate| matches!(candidate.kind(), "def" | "defs") && candidate.method_name() == name.as_deref() || matches!(candidate.kind(), "block" | "numblock" | "itblock") && candidate.method_name() == Some("define_method") && candidate.first_argument().and_then(NodeRef::scalar_value_text) == name)
        } else { None };
        let Some(definition) = definition else { return; };
        let arguments = definition.arguments();
        if arguments.iter().any(|argument| argument.kind() == "restarg") && arguments.iter().all(|argument| !matches!(argument.kind(), "kwarg" | "kwoptarg" | "kwrestarg")) { return; }
        let name = definition.method_name().map(str::to_owned).or_else(|| definition.first_argument().and_then(NodeRef::scalar_value_text)).unwrap_or_default();
        let offense = if matches!(argument.kind(), "def" | "defs") { self.location_range(node, "selector").map_or_else(|| self.owned_character_range(node.source_range().unwrap_or_default()), |range| range) } else { self.owned_character_range(node.source_range().unwrap_or_default()) };
        self.report(format!("`ruby2_keywords` is unnecessary for method `{name}`."), offense);
    }
}

define_compatibility_rule!(FloatComparisonRule);
impl FloatComparisonRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) { self.check_send(node); }
    fn on_csend(&mut self, node: NodeRef<'_>) { self.check_send(node); }
    fn on_case(&mut self, node: NodeRef<'_>) {
        for condition in node.when_branches().into_iter().flat_map(NodeRef::conditions) {
            if self.float_node(condition) && !float_literal_safe(condition) { self.report("Avoid float literal comparisons in case statements as they are unreliable.", condition); }
        }
    }
    fn check_send(&mut self, node: NodeRef<'_>) {
        if !matches!(node.method_name(), Some("==" | "!=" | "eql?" | "equal?")) { return; }
        let arguments = node.arguments();
        if arguments.len() != 1 { return; }
        let lhs = node.receiver(); let rhs = arguments[0];
        if lhs.is_some_and(float_literal_safe) || float_literal_safe(rhs) { return; }
        if lhs.is_some_and(|lhs| self.float_node(lhs)) || self.float_node(rhs) {
            self.report(if node.method_name() == Some("!=") { "Avoid inequality comparisons of floats as they are unreliable." } else { "Avoid equality comparisons of floats as they are unreliable." }, node);
        }
    }
    fn float_node(&self, node: NodeRef<'_>) -> bool {
        if node.kind() == "float" { return true; }
        if node.kind() == "begin" { return node.first_node().is_some_and(|child| self.float_node(child)); }
        if !matches!(node.kind(), "send" | "csend") { return false; }
        if matches!(node.method_name(), Some("to_f" | "Float" | "fdiv")) { return true; }
        if matches!(node.method_name(), Some("+" | "-" | "*" | "/" | "%" | "**")) { return node.receiver().is_some_and(|receiver| self.float_node(receiver)) || node.first_argument().is_some_and(|argument| self.float_node(argument)); }
        if node.receiver().is_some_and(|receiver| receiver.kind() == "float") && matches!(node.method_name(), Some("@-" | "abs" | "magnitude" | "modulo" | "next_float" | "prev_float" | "quo")) { return true; }
        if node.receiver().is_some_and(|receiver| receiver.kind() == "float") && matches!(node.method_name(), Some("ceil" | "floor" | "round" | "truncate")) {
            return node.first_argument().is_some_and(|argument| argument.kind() == "int" && argument.source().and_then(|source| source.parse::<i64>().ok()).is_some_and(|precision| precision > 0));
        }
        false
    }
}

fn float_literal_safe(node: NodeRef<'_>) -> bool {
    node.kind() == "nil" || matches!(node.kind(), "int" | "float") && node.source().and_then(|source| source.parse::<f64>().ok()) == Some(0.0)
}

define_compatibility_rule!(ParameterListsRule);
impl ParameterListsRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check_optional(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check_optional(node); }
    fn on_args(&mut self, node: NodeRef<'_>) {
        let Some(parent) = node.parent() else { return; };
        if parent.method_name() == Some("initialize") && parent.parent().is_some_and(|block| matches!(block.kind(), "block" | "numblock" | "itblock") && matches!(block.method_name(), Some("new" | "define")) && block.receiver().is_some_and(|receiver| matches!(receiver.short_name(), Some("Struct" | "Data")))) { return; }
        if matches!(parent.kind(), "block" | "numblock" | "itblock") && matches!(parent.method_name(), Some("lambda" | "proc")) { return; }
        let count = node.child_nodes().iter().filter(|argument| argument.kind() != "blockarg" && (self.config_bool("CountKeywordArgs", true) || !matches!(argument.kind(), "kwarg" | "kwoptarg"))).count();
        let maximum = self.config_usize("Max", 5);
        if count > maximum { self.report(format!("Avoid parameter lists longer than {maximum} parameters. [{count}/{maximum}]"), node); }
    }
    fn check_optional(&mut self, node: NodeRef<'_>) {
        let count = node.arguments().iter().filter(|argument| argument.kind() == "optarg").count();
        let maximum = self.config_usize("MaxOptionalParameters", 3);
        if count > maximum { self.report(format!("Method has too many optional parameters. [{count}/{maximum}]"), node); }
    }
}

define_compatibility_rule!(MissingSuperRule);
impl MissingSuperRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check(node, false); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check(node, true); }
    fn check(&mut self, node: NodeRef<'_>, singleton: bool) {
        if node.descendants().into_iter().any(|child| matches!(child.kind(), "super" | "zsuper")) { return; }
        let callbacks = ["inherited", "method_added", "method_removed", "method_undefined", "singleton_method_added", "singleton_method_removed", "singleton_method_undefined"];
        let callback = node.method_name().is_some_and(|name| callbacks.contains(&name)) && node.each_ancestor(&["class", "sclass", "module"]).first().is_some();
        if callback { self.report("Call `super` to invoke callback defined in the parent class.", node); return; }
        if singleton || node.method_name() != Some("initialize") { return; }
        let parent = if let Some(block) = node.each_ancestor(&["block", "numblock", "itblock"]).first().copied() {
            if block.method_name() != Some("new") || !block.receiver().is_some_and(|receiver| receiver.short_name() == Some("Class")) { return; }
            let Some(parent) = block.send_node().and_then(NodeRef::first_argument) else { return; };
            parent
        } else {
            let Some(class) = node.each_ancestor(&["class"]).first().copied() else { return; };
            let Some(parent) = class.parent_class() else { return; };
            parent
        };
        let allowed = self.config_values("AllowedParentClasses");
        if matches!(parent.short_name(), Some("Object" | "BasicObject")) || parent.short_name().is_some_and(|name| allowed.iter().any(|allowed| allowed == name)) { return; }
        self.report("Call `super` to initialize state of the parent class.", node);
    }
}

define_compatibility_rule!(SuppressedExceptionRule);
impl SuppressedExceptionRule<'_, '_, '_, '_> {
    fn on_resbody(&mut self, node: NodeRef<'_>) {
        let nil_body = node.body().is_some_and(|body| body.kind() == "nil");
        if node.body().is_some() && !nil_body { return; }
        if self.config_bool("AllowComments", true) {
            let end_line = node.each_ancestor(&["kwbegin", "def", "defs", "block", "numblock", "itblock"]).first().map_or(node.last_line(), |ancestor| ancestor.last_line());
            if self.processed_source().comments().iter().any(|comment| comment.line > node.first_line() && comment.line <= end_line) { return; }
        }
        if self.config_bool("AllowNil", true) && nil_body { return; }
        let mut range = node.source_range().unwrap_or_default();
        if self.range_source(&self.range_between(range.end, range.end + 1)) == ";" { range.end += 1; }
        let offense = self.owned_character_range(range);
        self.report("Do not suppress exceptions.", offense);
    }
}

define_compatibility_rule!(CircularArgumentReferenceRule);
impl CircularArgumentReferenceRule<'_, '_, '_, '_> {
    fn on_kwoptarg(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_optarg(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.name() else { return; };
        let Some(mut value) = node.expression() else { return; };
        if value.kind() == "lvar" && value.name() == Some(name) { self.report(format!("Circular argument reference - `{name}`."), value); return; }
        let mut assigned = Vec::new();
        while value.kind() == "lvasgn" { if let Some(assigned_name) = value.name() { assigned.push(assigned_name); } let Some(next) = value.expression() else { return; }; value = next; }
        if value.kind() == "lvar" && value.name().is_some_and(|value_name| value_name == name || assigned.contains(&value_name)) { self.report(format!("Circular argument reference - `{name}`."), value); }
    }
}

define_compatibility_rule!(SelfAssignmentRule);
impl SelfAssignmentRule<'_, '_, '_, '_> {
    fn on_lvasgn(&mut self, node: NodeRef<'_>) { self.check(node, "lvar"); }
    fn on_ivasgn(&mut self, node: NodeRef<'_>) { self.check(node, "ivar"); }
    fn on_cvasgn(&mut self, node: NodeRef<'_>) { self.check(node, "cvar"); }
    fn check(&mut self, node: NodeRef<'_>, variable_kind: &str) {
        let Some(rhs) = node.expression() else { return; };
        let (operator, target, new_rhs) = if rhs.operator_keyword() { (self.location_range(rhs, "operator").map(|range| self.range_source(&range).to_owned()), rhs.lhs(), rhs.rhs()) } else if rhs.arguments().len() == 1 && matches!(rhs.method_name(), Some("+" | "-" | "*" | "**" | "/" | "%" | "^" | "<<" | ">>" | "|" | "&")) { (rhs.method_name().map(str::to_owned), rhs.receiver(), rhs.first_argument()) } else { return; };
        let Some(operator) = operator else { return; }; let Some(target) = target else { return; }; let Some(new_rhs) = new_rhs else { return; };
        if target.kind() != variable_kind || target.name() != node.name() { return; }
        let Some(assignment_operator) = self.location_range(node, "operator") else { return; };
        add_offense!(self, node, message: format!("Use self-assignment shorthand `{operator}=`."), |corrector| {
            corrector.insert_before(assignment_operator, operator);
            corrector.replace(rhs, new_rhs.source().unwrap_or_default());
        });
    }
}

define_compatibility_rule!(KeywordParametersOrderRule);
impl KeywordParametersOrderRule<'_, '_, '_, '_> {
    fn on_kwoptarg(&mut self, node: NodeRef<'_>) {
        let required = node.right_siblings().into_iter().filter(|sibling| sibling.kind() == "kwarg").collect::<Vec<_>>();
        if required.is_empty() { return; }
        self.report("Place optional keyword parameters at the end of the parameters list.", node);
    }
}

define_compatibility_rule!(LoopRule);
impl LoopRule<'_, '_, '_, '_> {
    fn on_while_post(&mut self, node: NodeRef<'_>) { self.register(node, "unless"); }
    fn on_until_post(&mut self, node: NodeRef<'_>) { self.register(node, "if"); }
    fn register(&mut self, node: NodeRef<'_>, conditional: &str) {
        let (Some(body), Some(condition), Some(keyword)) = (node.body(), node.condition(), self.location_range(node, "keyword")) else { return; };
        let (Some(opening), Some(closing), Some(node_range)) = (self.location_range(body, "begin"), self.location_range(body, "end"), node.source_range()) else { return; };
        let indent = " ".repeat(node.column());
        let remove = self.range_between(closing.end_pos(), node_range.end);
        add_offense!(self, keyword, message: "Use `Kernel#loop` with `break` rather than `begin/end/until`(or `while`).", |corrector| {
            corrector.replace(opening, "loop do");
            corrector.remove(remove);
            corrector.insert_before(closing, format!("break {conditional} {}\n{indent}", condition.source().unwrap_or_default()));
        });
    }
}

define_compatibility_rule!(RedundantConstantBaseRule);
impl RedundantConstantBaseRule<'_, '_, '_, '_> {
    fn on_cbase(&mut self, node: NodeRef<'_>) {
        if self.related_config_value("Lint/ConstantResolution", "Enabled") == Some("true") { return; }
        let nested = node.each_ancestor(&["class", "module"]).into_iter().any(|ancestor| {
            !(ancestor.kind() == "class" && ancestor.parent_class().is_some_and(|parent| parent.descendants().into_iter().any(|descendant| descendant.id() == node.id())))
        });
        if nested { return; }
        add_offense!(self, node, message: "Remove redundant `::`.", |corrector| { corrector.remove(node); });
    }
}

define_compatibility_rule!(GlobalStdStreamRule);
impl GlobalStdStreamRule<'_, '_, '_, '_> {
    fn on_const(&mut self, node: NodeRef<'_>) {
        let Some(name @ ("STDIN" | "STDOUT" | "STDERR")) = node.short_name() else { return; };
        if node.namespace().is_some_and(|namespace| namespace.kind() != "cbase") { return; }
        let global = format!("${}", name.to_ascii_lowercase());
        if node.parent().is_some_and(|parent| parent.kind() == "gvasgn" && parent.name() == Some(&global)) { return; }
        add_offense!(self, node, message: format!("Use `{global}` instead of `{name}`."), |corrector| { corrector.replace(node, global); });
    }
}

define_compatibility_rule!(ArrayFirstLastRule);
impl ArrayFirstLastRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_csend(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, mut node: NodeRef<'_>) {
        if node.method_name() != Some("[]") || node.arguments().len() != 1 { return; }
        let value = node.first_argument().and_then(NodeRef::source).and_then(|source| source.parse::<i64>().ok());
        let Some(value @ (0 | -1)) = value else { return; };
        while node.receiver().is_some_and(|receiver| receiver.method_name() == Some("[]")) { node = node.receiver().unwrap_or(node); }
        if node.ancestors().into_iter().any(|parent| matches!(parent.method_name(), Some("[]" | "[]="))) { return; }
        if node.source_range().and_then(|range| self.source().chars().nth(range.end)).is_some_and(|character| matches!(character, '[' | ']')) { return; }
        let preferred = if value == 0 { "first" } else { "last" };
        let Some(selector) = self.location_range(node, "selector") else { return; };
        let offense = if self.location_range(node, "dot").is_some() { self.range_between(selector.begin_pos(), node.source_range().map_or(selector.end_pos(), |range| range.end)) } else { selector };
        let replacement = if self.location_range(node, "dot").is_some() { preferred.to_owned() } else { format!(".{preferred}") };
        add_offense!(self, offense, message: format!("Use `{preferred}`."), |corrector| { corrector.replace(offense, replacement); });
    }
}
