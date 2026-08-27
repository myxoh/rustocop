use super::*;
use crate::rubocop::ast::node::core::NodeRef;

define_cops! {
    BeginBlockCompatibility => "Style/BeginBlock" => compatibility_callbacks(BeginBlockRule, [on_preexe]),
    FloatOutOfRangeCompatibility => "Lint/FloatOutOfRange" => compatibility_callbacks(FloatOutOfRangeRule, [on_float]),
    EndBlockCompatibility => "Style/EndBlock" => compatibility_callbacks(EndBlockRule, [on_postexe]),
    DuplicateHashKeyCompatibility => "Lint/DuplicateHashKey" => compatibility_callbacks(DuplicateHashKeyRule, [on_hash]),
    RegexpAsConditionCompatibility => "Lint/RegexpAsCondition" => compatibility_callbacks(RegexpAsConditionRule, [on_match_current_line]),
    DuplicateElsifConditionCompatibility => "Lint/DuplicateElsifCondition" => compatibility_callbacks(DuplicateElsifConditionRule, [on_if]),
    RandOneCompatibility => "Lint/RandOne" => compatibility_callbacks(RandOneRule, [on_send restrict ["rand"]]),
    EvalCompatibility => "Security/Eval" => compatibility_callbacks(EvalRule, [on_send restrict ["eval"]]),
    HashCompareByIdentityCompatibility => "Lint/HashCompareByIdentity" => compatibility_callbacks(HashCompareByIdentityRule, [on_send]),
    SafeNavigationChainLengthCompatibility => "Style/SafeNavigationChainLength" => compatibility_callbacks(SafeNavigationChainLengthRule, [on_csend]),
    EachWithObjectArgumentCompatibility => "Lint/EachWithObjectArgument" => compatibility_callbacks(EachWithObjectArgumentRule, [on_send restrict ["each_with_object"]]),
    SafeNavigationWithEmptyCompatibility => "Lint/SafeNavigationWithEmpty" => compatibility_callbacks(SafeNavigationWithEmptyRule, [on_if]),
    IoMethodsCompatibility => "Security/IoMethods" => compatibility_callbacks(IoMethodsRule, [on_send]),
    RedundantArrayFlattenCompatibility => "Style/RedundantArrayFlatten" => compatibility_callbacks(RedundantArrayFlattenRule, [on_send restrict ["flatten"]]),
    UselessDefinedCompatibility => "Lint/UselessDefined" => compatibility_callbacks(UselessDefinedRule, [on_defined]),
    AutoResourceCleanupCompatibility => "Style/AutoResourceCleanup" => compatibility_callbacks(AutoResourceCleanupRule, [on_send restrict ["open"]]),
    ReturnInVoidContextCompatibility => "Lint/ReturnInVoidContext" => compatibility_callbacks(ReturnInVoidContextRule, [on_return]),
    DataDefineOverrideCompatibility => "Lint/DataDefineOverride" => compatibility_callbacks(DataDefineOverrideRule, [on_send restrict ["define"]]),
    ClassVarsCompatibility => "Style/ClassVars" => compatibility_callbacks(ClassVarsRule, [on_cvasgn, on_send]),
    OptionalArgumentsCompatibility => "Style/OptionalArguments" => compatibility_callbacks(OptionalArgumentsRule, [on_def]),
    BigDecimalNewCompatibility => "Lint/BigDecimalNew" => compatibility_callbacks(BigDecimalNewRule, [on_send restrict ["new"]]),
    EmptyEnsureCompatibility => "Lint/EmptyEnsure" => compatibility_callbacks(EmptyEnsureRule, [on_ensure]),
    LeadingEmptyLinesCompatibility => "Layout/LeadingEmptyLines" => compatibility_investigation(LeadingEmptyLinesRule, on_new_investigation),
    TopLevelReturnWithArgumentCompatibility => "Lint/TopLevelReturnWithArgument" => compatibility_callbacks(TopLevelReturnWithArgumentRule, [on_return]),
    DirCompatibility => "Style/Dir" => compatibility_callbacks(DirRule, [on_send]),
    YamlLoadCompatibility => "Security/YAMLLoad" => compatibility_callbacks(YamlLoadRule, [on_send restrict ["load"]]),
    RedundantCurrentDirectoryInPathCompatibility => "Style/RedundantCurrentDirectoryInPath" => compatibility_callbacks(RedundantCurrentDirectoryInPathRule, [on_send restrict ["require_relative"]]),
    LambdaWithoutLiteralBlockCompatibility => "Lint/LambdaWithoutLiteralBlock" => compatibility_callbacks(LambdaWithoutLiteralBlockRule, [on_send restrict ["lambda"]]),
    MixedRegexpCaptureTypesCompatibility => "Lint/MixedRegexpCaptureTypes" => compatibility_callbacks(MixedRegexpCaptureTypesRule, [on_regexp]),
    DuplicateRescueExceptionCompatibility => "Lint/DuplicateRescueException" => compatibility_callbacks(DuplicateRescueExceptionRule, [on_rescue]),
    RefinementImportMethodsCompatibility => "Lint/RefinementImportMethods" => compatibility_callbacks(RefinementImportMethodsRule, [on_send restrict ["include", "prepend"]]),
    NextWithoutAccumulatorCompatibility => "Lint/NextWithoutAccumulator" => compatibility_callbacks(NextWithoutAccumulatorRule, [on_block]),
    UriRegexpCompatibility => "Lint/UriRegexp" => compatibility_callbacks(UriRegexpRule, [on_send restrict ["regexp"]]),
    EvenOddCompatibility => "Style/EvenOdd" => compatibility_callbacks(EvenOddRule, [on_send restrict ["==", "!="]]),
    NumberedParametersLimitCompatibility => "Style/NumberedParametersLimit" => compatibility_callbacks(NumberedParametersLimitRule, [on_numblock]),
    AsciiCommentsCompatibility => "Style/AsciiComments" => compatibility_investigation(AsciiCommentsRule, on_new_investigation),
    OpenStructUseCompatibility => "Style/OpenStructUse" => compatibility_callbacks(OpenStructUseRule, [on_const]),
    JsonLoadCompatibility => "Security/JSONLoad" => compatibility_callbacks(JsonLoadRule, [on_send restrict ["load", "restore"]]),
    MixinUsageCompatibility => "Style/MixinUsage" => compatibility_callbacks(MixinUsageRule, [on_send restrict ["include", "extend", "prepend"]]),
    StructNewOverrideCompatibility => "Lint/StructNewOverride" => compatibility_callbacks(StructNewOverrideRule, [on_send restrict ["new"]]),
    SharedMutableDefaultCompatibility => "Lint/SharedMutableDefault" => compatibility_callbacks(SharedMutableDefaultRule, [on_send restrict ["new"]]),
    RedundantArrayConstructorCompatibility => "Style/RedundantArrayConstructor" => compatibility_callbacks(RedundantArrayConstructorRule, [on_send restrict ["new", "[]", "Array"]]),
    PreferredHashMethodsCompatibility => "Style/PreferredHashMethods" => compatibility_callbacks(PreferredHashMethodsRule, [on_send, on_csend]),
    CharacterLiteralCompatibility => "Style/CharacterLiteral" => compatibility_callbacks(CharacterLiteralRule, [on_str]),
    ConstantOverwrittenInRescueCompatibility => "Lint/ConstantOverwrittenInRescue" => compatibility_callbacks(ConstantOverwrittenInRescueRule, [on_resbody]),
    RequireRelativeSelfPathCompatibility => "Lint/RequireRelativeSelfPath" => compatibility_callbacks(RequireRelativeSelfPathRule, [on_send restrict ["require_relative"]]),
    BinaryOperatorParameterNameCompatibility => "Naming/BinaryOperatorParameterName" => compatibility_callbacks(BinaryOperatorParameterNameRule, [on_def]),
    AmbiguousAssignmentCompatibility => "Lint/AmbiguousAssignment" => compatibility_callbacks(AmbiguousAssignmentRule, [on_lvasgn, on_ivasgn, on_cvasgn, on_gvasgn, on_casgn]),
    OptionHashCompatibility => "Style/OptionHash" => compatibility_callbacks(OptionHashCompatibilityRule, [on_args]),
    HashNewWithKeywordArgumentsAsDefaultCompatibility => "Lint/HashNewWithKeywordArgumentsAsDefault" => compatibility_callbacks(HashNewWithKeywordArgumentsAsDefaultRule, [on_send restrict ["new"]]),
}

define_compatibility_rule!(BeginBlockRule);
impl BeginBlockRule<'_, '_, '_, '_> {
    fn on_preexe(&mut self, node: NodeRef<'_>) {
        if let Some(keyword) = self.location_range(node, "keyword") {
            self.report("Avoid the use of `BEGIN` blocks.", keyword);
        }
    }
}

define_compatibility_rule!(FloatOutOfRangeRule);
impl FloatOutOfRangeRule<'_, '_, '_, '_> {
    fn on_float(&mut self, node: NodeRef<'_>) {
        let source = node.source().unwrap_or_default();
        let normalized = source.replace('_', "");
        let value = normalized.parse::<f64>().unwrap_or_default();
        if value.is_infinite() || value == 0.0 && source.bytes().any(|byte| matches!(byte, b'1'..=b'9')) {
            self.report("Float out of range.", node);
        }
    }
}

define_compatibility_rule!(EndBlockRule);
impl EndBlockRule<'_, '_, '_, '_> {
    fn on_postexe(&mut self, node: NodeRef<'_>) {
        let Some(keyword) = self.location_range(node, "keyword") else { return; };
        add_offense!(self, keyword, message: "Avoid the use of `END` blocks. Use `Kernel#at_exit` instead.", |corrector| {
            corrector.replace(keyword, "at_exit");
        });
    }
}

define_compatibility_rule!(DuplicateHashKeyRule);
impl DuplicateHashKeyRule<'_, '_, '_, '_> {
    fn on_hash(&mut self, node: NodeRef<'_>) {
        let keys = node
            .keys()
            .into_iter()
            .filter(|key| key.recursive_basic_literal() || key.kind() == "const")
            .collect::<Vec<_>>();
        let mut seen = Vec::new();
        for key in keys {
            if seen.iter().any(|prior: &NodeRef<'_>| prior.structurally_equal(key)) {
                self.report("Duplicated key in hash literal.", key);
            } else {
                seen.push(key);
            }
        }
    }
}

define_compatibility_rule!(RegexpAsConditionRule);
impl RegexpAsConditionRule<'_, '_, '_, '_> {
    fn on_match_current_line(&mut self, node: NodeRef<'_>) {
        if !node.ancestors().into_iter().any(NodeRef::conditional) {
            return;
        }
        let replacement = format!("{} =~ $_", node.source().unwrap_or_default());
        add_offense!(self, node, message: "Do not use regexp literal as a condition. The regexp literal matches `$_` implicitly.", |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(DuplicateElsifConditionRule);
impl DuplicateElsifConditionRule<'_, '_, '_, '_> {
    fn on_if(&mut self, mut node: NodeRef<'_>) {
        let mut previous = Vec::new();
        while node.kind() == "if" && (node.if_keyword() || node.elsif()) {
            let Some(condition) = node.condition() else { break; };
            if previous.iter().any(|prior: &NodeRef<'_>| prior.structurally_equal(condition)) {
                self.report("Duplicate `elsif` condition detected.", condition);
            }
            previous.push(condition);
            let Some(else_branch) = node.else_branch().filter(|branch| branch.kind() == "if") else { break; };
            node = else_branch;
        }
    }
}

define_compatibility_rule!(RandOneRule);
impl RandOneRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !implicit_or_root_constant(node.receiver(), "Kernel") || node.arguments().len() != 1 {
            return;
        }
        let argument = node.arguments()[0];
        let value = argument.source().unwrap_or_default().replace('_', "").parse::<f64>();
        if !value.is_ok_and(|value| value.abs() == 1.0)
            || !matches!(argument.kind(), "int" | "float")
        {
            return;
        }
        let method = node.source().unwrap_or_default();
        self.report(format!("`{method}` always returns `0`. Perhaps you meant `rand(2)` or `rand`?"), node);
    }
}

define_compatibility_rule!(EvalRule);
impl EvalRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let receiver_matches = node.receiver().is_none_or(|receiver| {
            root_constant(receiver, "Kernel")
                || receiver.method_name() == Some("binding") && receiver.receiver().is_none()
        });
        let Some(code) = node.first_argument() else { return; };
        if !receiver_matches
            || code.kind() == "str"
            || code.kind() == "dstr" && code.recursive_literal()
        {
            return;
        }
        if let Some(selector) = self.location_range(node, "selector") {
            self.report("The use of `eval` is a serious security risk.", selector);
        }
    }
}

define_compatibility_rule!(HashCompareByIdentityRule);
impl HashCompareByIdentityRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !["key?", "has_key?", "fetch", "[]", "[]="].contains(&node.method_name().unwrap_or_default()) {
            return;
        }
        let Some(key) = node.first_argument().filter(|key| {
            key.method_name() == Some("object_id") && key.arguments().is_empty()
        }) else { return; };
        let _ = key;
        self.report("Use `Hash#compare_by_identity` instead of using `object_id` for keys.", node);
    }
}

define_compatibility_rule!(SafeNavigationChainLengthRule);
impl SafeNavigationChainLengthRule<'_, '_, '_, '_> {
    fn on_csend(&mut self, node: NodeRef<'_>) {
        let chains = node
            .ancestors()
            .into_iter()
            .take_while(|parent| parent.kind() == "csend")
            .collect::<Vec<_>>();
        let max = self.config_usize("Max", 2);
        if chains.len() < max {
            return;
        }
        self.report(format!("Avoid safe navigation chains longer than {max} calls."), chains[chains.len() - 1]);
    }
}

fn root_constant(node: NodeRef<'_>, name: &str) -> bool {
    node.kind() == "const"
        && node.short_name() == Some(name)
        && node.namespace().is_none_or(|namespace| namespace.kind() == "cbase")
}

fn implicit_or_root_constant(receiver: Option<NodeRef<'_>>, name: &str) -> bool {
    receiver.is_none_or(|receiver| root_constant(receiver, name))
}

define_compatibility_rule!(EachWithObjectArgumentRule);
impl EachWithObjectArgumentRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.arguments().len() == 1 && node.arguments()[0].immutable_literal() {
            self.report("The argument to each_with_object cannot be immutable.", node);
        }
    }
}

define_compatibility_rule!(SafeNavigationWithEmptyRule);
impl SafeNavigationWithEmptyRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) {
        let Some(condition) = node.condition().filter(|condition| {
            condition.kind() == "csend"
                && condition.method_name() == Some("empty?")
                && condition.arguments().is_empty()
                && condition.receiver().is_some_and(|receiver| receiver.kind() == "send")
        }) else { return; };
        let Some(receiver) = condition.receiver() else { return; };
        let receiver_source = receiver.source().unwrap_or_default();
        let replacement = format!("{receiver_source} && {receiver_source}.{}", condition.method_name().unwrap_or_default());
        add_offense!(self, condition, message: "Avoid calling `empty?` with the safe navigation operator in conditionals.", |corrector| {
            corrector.replace(condition, replacement);
        });
    }
}

define_compatibility_rule!(IoMethodsRule);
impl IoMethodsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let method = node.method_name().unwrap_or_default();
        if !["read", "binread", "write", "binwrite", "foreach", "readlines"].contains(&method)
            // RuboCop deliberately compares the literal receiver source here,
            // so `::IO.read` is not equivalent to `IO.read` for this cop.
            || !node.receiver().is_some_and(|receiver| receiver.source() == Some("IO"))
            || node.first_argument().is_some_and(|argument| {
                argument.scalar_value_text().is_some_and(|value| value.trim().starts_with('|'))
            })
        {
            return;
        }
        let Some(receiver) = node.receiver() else { return; };
        add_offense!(self, node, message: format!("`File.{method}` is safer than `IO.{method}`."), |corrector| {
            corrector.replace(receiver, "File");
        });
    }
}

define_compatibility_rule!(RedundantArrayFlattenRule);
impl RedundantArrayFlattenRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(join) = node.parent().filter(|parent| {
            parent.call_type()
                && parent.receiver() == Some(node)
                && parent.method_name() == Some("join")
                && (parent.arguments().is_empty()
                    || parent.arguments().len() == 1 && parent.arguments()[0].kind() == "nil")
        }) else { return; };
        let _ = join;
        if node.receiver().is_none() || node.arguments().len() > 1 {
            return;
        }
        let (Some(dot), Some(expression)) = (self.location_range(node, "dot"), self.source_range(node)) else { return; };
        let offense = self.range_between(dot.begin_pos(), expression.end_pos());
        add_offense!(self, offense, message: "Remove the redundant `flatten`.", |corrector| {
            corrector.remove(offense);
        });
    }
}

define_compatibility_rule!(UselessDefinedRule);
impl UselessDefinedRule<'_, '_, '_, '_> {
    fn on_defined(&mut self, node: NodeRef<'_>) {
        let Some(argument) = node.first_argument() else { return; };
        let kind = match argument.kind() {
            "str" | "dstr" => "string",
            "sym" | "dsym" => "symbol",
            _ => return,
        };
        self.report(format!("Calling `defined?` with a {kind} argument will always return a truthy value."), node);
    }
}

define_compatibility_rule!(AutoResourceCleanupRule);
impl AutoResourceCleanupRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(receiver) = node.receiver().filter(|receiver| {
            root_constant(*receiver, "File") || root_constant(*receiver, "Tempfile")
        }) else { return; };
        if node.arguments().last().is_some_and(|argument| argument.kind() == "block_pass") {
            return;
        }
        if node.parent().is_some_and(|parent| matches!(parent.kind(), "block" | "numblock" | "itblock") || parent.kind() != "lvasgn") {
            return;
        }
        let current = format!("{}.open", receiver.source().unwrap_or_default());
        self.report(format!("Use the block version of `{current}`."), node);
    }
}

define_compatibility_rule!(ReturnInVoidContextRule);
impl ReturnInVoidContextRule<'_, '_, '_, '_> {
    fn on_return(&mut self, node: NodeRef<'_>) {
        if node.descendants().is_empty() {
            return;
        }
        let Some(definition) = node.each_ancestor(&["def", "defs"]).into_iter().next().filter(|definition| definition.void_context()) else { return; };
        if node.each_ancestor(&["block", "numblock", "itblock"]).into_iter().any(|block| {
            ["lambda", "define_method", "define_singleton_method"].contains(&block.method_name().unwrap_or_default())
        }) {
            return;
        }
        let Some(keyword) = self.location_range(node, "keyword") else { return; };
        self.report(format!("Do not return a value in `{}`.", definition.method_name().unwrap_or_default()), keyword);
    }
}

define_compatibility_rule!(DataDefineOverrideRule);
impl DataDefineOverrideRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.receiver().is_some_and(|receiver| root_constant(receiver, "Data")) {
            return;
        }
        for argument in node.arguments() {
            if !matches!(argument.kind(), "sym" | "str") {
                continue;
            }
            let Some(member_name) = argument.scalar_value_text() else { continue; };
            if !super::lint_builtin_overrides::DATA_METHODS.iter().any(|method| *method == member_name.as_bytes()) {
                continue;
            }
            let inspected = argument.source().unwrap_or_default();
            self.report(format!("`{inspected}` member overrides `Data#{member_name}` and it may be unexpected."), argument);
        }
    }
}

define_compatibility_rule!(ClassVarsRule);
impl ClassVarsRule<'_, '_, '_, '_> {
    fn on_cvasgn(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.name() else { return; };
        let offense = self.location_range(node, "name").map_or(node, |_| node);
        if let Some(name_range) = self.location_range(node, "name") {
            self.report(format!("Replace class var {name} with a class instance var."), name_range);
        } else {
            self.report(format!("Replace class var {name} with a class instance var."), offense);
        }
    }
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.method_name() != Some("class_variable_set") {
            return;
        }
        let Some(argument) = node.first_argument() else { return; };
        self.report(format!("Replace class var {} with a class instance var.", argument.source().unwrap_or_default()), argument);
    }
}

define_compatibility_rule!(OptionalArgumentsRule);
impl OptionalArgumentsRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) {
        let arguments = node.arguments();
        let last_required = arguments.iter().rposition(|argument| argument.kind() == "arg");
        let Some(last_required) = last_required else { return; };
        for argument in &arguments[..last_required] {
            if argument.kind() == "optarg" {
                self.report("Optional arguments should appear at the end of the argument list.", *argument);
            }
        }
    }
}

define_compatibility_rule!(BigDecimalNewRule);
impl BigDecimalNewRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(receiver) = node.receiver().filter(|receiver| root_constant(*receiver, "BigDecimal")) else { return; };
        let (Some(selector), Some(dot)) = (self.location_range(node, "selector"), self.location_range(node, "dot")) else { return; };
        add_offense!(self, selector, message: "`BigDecimal.new()` is deprecated. Use `BigDecimal()` instead.", |corrector| {
            corrector.remove(selector);
            corrector.remove(dot);
            if let Some(cbase) = receiver.namespace().filter(|namespace| namespace.kind() == "cbase") {
                corrector.remove(cbase);
            }
        });
    }
}

define_compatibility_rule!(EmptyEnsureRule);
impl EmptyEnsureRule<'_, '_, '_, '_> {
    fn on_ensure(&mut self, node: NodeRef<'_>) {
        if node.branch().is_some() {
            return;
        }
        let Some(keyword) = self.location_range(node, "keyword") else { return; };
        add_offense!(self, keyword, message: "Empty `ensure` block detected.", |corrector| {
            corrector.remove(keyword);
        });
    }
}

define_compatibility_rule!(LeadingEmptyLinesRule);
impl LeadingEmptyLinesRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let Some(token) = self.processed_source().tokens().iter().find(|token| token.kind != "tNL").filter(|token| token.line > 1) else { return; };
        let offense = self.owned_character_range(token.range.clone());
        let leading = self.range_between(0, token.range.start);
        add_offense!(self, offense, message: "Unnecessary blank line at the beginning of the source.", |corrector| {
            corrector.remove(leading);
        });
    }
}

define_compatibility_rule!(TopLevelReturnWithArgumentRule);
impl TopLevelReturnWithArgumentRule<'_, '_, '_, '_> {
    fn on_return(&mut self, node: NodeRef<'_>) {
        if node.arguments().is_empty()
            || !node.each_ancestor(&["block", "numblock", "itblock", "def", "defs"]).is_empty()
        {
            return;
        }
        add_offense!(self, node, message: "Top level return with argument detected.", |corrector| {
            corrector.replace(node, "return");
        });
    }
}

define_compatibility_rule!(DirRule);
impl DirRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(2, 0) || !dir_replacement(node) {
            return;
        }
        add_offense!(self, node, message: "Use `__dir__` to get an absolute path to the current file's directory.", |corrector| {
            corrector.replace(node, "__dir__");
        });
    }
}

fn dir_replacement(node: NodeRef<'_>) -> bool {
    if !node.receiver().is_some_and(|receiver| root_constant(receiver, "File"))
        || node.arguments().len() != 1
    {
        return false;
    }
    let Some(argument) = node.first_argument().filter(|argument| argument.call_type()) else { return false; };
    let file_keyword = |candidate: NodeRef<'_>| candidate.kind() == "__FILE__" || candidate.source() == Some("__FILE__");
    match node.method_name() {
        Some("expand_path") => argument.method_name() == Some("dirname")
            && argument.receiver().is_some_and(|receiver| root_constant(receiver, "File"))
            && argument.arguments().len() == 1
            && argument.first_argument().is_some_and(file_keyword),
        Some("dirname") => argument.method_name() == Some("realpath")
            && argument.receiver().is_some_and(|receiver| root_constant(receiver, "File"))
            && argument.arguments().len() == 1
            && argument.first_argument().is_some_and(file_keyword),
        _ => false,
    }
}

define_compatibility_rule!(YamlLoadRule);
impl YamlLoadRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if self.target_ruby_version().at_least(3, 1)
            || !node.receiver().is_some_and(|receiver| root_constant(receiver, "YAML"))
        {
            return;
        }
        let Some(selector) = self.location_range(node, "selector") else { return; };
        add_offense!(self, selector, message: "Prefer using `YAML.safe_load` over `YAML.load`.", |corrector| {
            corrector.replace(selector, "safe_load");
        });
    }
}

define_compatibility_rule!(RedundantCurrentDirectoryInPathRule);
impl RedundantCurrentDirectoryInPathRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(argument) = node.first_argument() else { return; };
        let Some(path) = argument.scalar_value_text() else { return; };
        if !path.starts_with("./") {
            return;
        }
        let redundant_length = 1 + path[1..].chars().take_while(|character| *character == '/').count();
        let source = argument.source().unwrap_or_default();
        let Some(index) = source.find("./") else { return; };
        let Some(range) = self.source_range(argument) else { return; };
        let offense = self.range_between(range.begin_pos() + index, range.begin_pos() + index + redundant_length);
        add_offense!(self, offense, message: "Remove the redundant current directory path.", |corrector| {
            corrector.remove(offense);
        });
    }
}

define_compatibility_rule!(LambdaWithoutLiteralBlockRule);
impl LambdaWithoutLiteralBlockRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.parent().is_some_and(|parent| matches!(parent.kind(), "block" | "numblock" | "itblock")) {
            return;
        }
        let Some(argument) = node.first_argument() else { return; };
        if argument.kind() == "block_pass" && argument.node_child(0).is_some_and(|value| value.kind() == "sym") {
            return;
        }
        let replacement = argument.source().unwrap_or_default().replace('&', "");
        add_offense!(self, node, message: "lambda without a literal block is deprecated; use the proc without lambda instead.", |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(MixedRegexpCaptureTypesRule);
impl MixedRegexpCaptureTypesRule<'_, '_, '_, '_> {
    fn on_regexp(&mut self, node: NodeRef<'_>) {
        if node.interpolation() {
            return;
        }
        let content = node.regexp_content();
        let extended = node
            .source()
            .and_then(|source| source.rsplit_once(['/', '}']))
            .is_some_and(|(_, options)| options.contains('x'));
        let (named, numbered) = super::lint_builtin_overrides::capture_types(content.as_bytes(), extended);
        if named && numbered {
            self.report("Do not mix named captures and numbered captures in a Regexp literal.", node);
        }
    }
}

define_compatibility_rule!(DuplicateRescueExceptionRule);
impl DuplicateRescueExceptionRule<'_, '_, '_, '_> {
    fn on_rescue(&mut self, node: NodeRef<'_>) {
        if node.modifier_form() {
            return;
        }
        let mut previous = Vec::new();
        for resbody in node.resbody_branches() {
            for exception in resbody.exceptions() {
                if previous.iter().any(|prior: &NodeRef<'_>| prior.structurally_equal(exception)) {
                    self.report("Duplicate `rescue` exception detected.", exception);
                } else {
                    previous.push(exception);
                }
            }
        }
    }
}

define_compatibility_rule!(RefinementImportMethodsRule);
impl RefinementImportMethodsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(3, 1) || node.receiver().is_some() {
            return;
        }
        let Some(parent) = node.parent().filter(|parent| {
            matches!(parent.kind(), "block" | "numblock" | "itblock")
                && parent.method_name() == Some("refine")
        }) else { return; };
        let _ = parent;
        let Some(selector) = self.location_range(node, "selector") else { return; };
        self.report(
            format!("Use `import_methods` instead of `{}` because it is deprecated in Ruby 3.1.", node.method_name().unwrap_or_default()),
            selector,
        );
    }
}

define_compatibility_rule!(NextWithoutAccumulatorRule);
impl NextWithoutAccumulatorRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        if !matches!(node.method_name(), Some("reduce" | "inject"))
            || node.first_argument().is_some_and(|argument| argument.kind() == "sym")
        {
            return;
        }
        let Some(body) = node.body().filter(|body| body.kind() == "begin") else { return; };
        let Some(void_next) = body.descendants().into_iter().find(|candidate| {
            candidate.kind() == "next"
                && candidate.arguments().is_empty()
                && candidate.each_ancestor(&["block", "numblock", "itblock"]).first() == Some(&node)
        }) else { return; };
        self.report("Use `next` with an accumulator argument in a `reduce`.", void_next);
    }
}

define_compatibility_rule!(UriRegexpRule);
impl UriRegexpRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(receiver) = node.receiver().filter(|receiver| root_constant(*receiver, "URI")) else { return; };
        let parser = if self.target_ruby_version().at_least(3, 4) { "RFC2396_PARSER" } else { "DEFAULT_PARSER" };
        let argument = node.first_argument().map_or_else(String::new, |argument| {
            format!("({})", argument.source().unwrap_or_default())
        });
        let preferred = format!("{}::{parser}.make_regexp{argument}", receiver.source().unwrap_or_default());
        let current = node.source().unwrap_or_default();
        let Some(selector) = self.location_range(node, "selector") else { return; };
        add_offense!(self, selector, message: format!("`{current}` is obsolete and should not be used. Instead, use `{preferred}`."), |corrector| {
            corrector.replace(node, preferred);
        });
    }
}

define_compatibility_rule!(EvenOddRule);
impl EvenOddRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(mut modulo) = node.receiver() else { return; };
        if modulo.kind() == "begin" {
            let children = modulo.child_nodes();
            if children.len() != 1 { return; }
            modulo = children[0];
        }
        let Some(base) = modulo.receiver() else { return; };
        if modulo.method_name() != Some("%")
            || modulo.arguments().len() != 1
            || modulo.arguments()[0].kind() != "int"
            || modulo.arguments()[0].source().is_none_or(|source| source.replace('_', "") != "2")
            || node.arguments().len() != 1
            || node.arguments()[0].kind() != "int"
        {
            return;
        }
        let argument = node.arguments()[0].source().unwrap_or_default().replace('_', "");
        let method = match (argument.as_str(), node.method_name()) {
            ("0", Some("==")) | ("1", Some("!=")) => "even",
            ("1", Some("==")) | ("0", Some("!=")) => "odd",
            _ => return,
        };
        add_offense!(self, node, message: format!("Replace with `Integer#{method}?`."), |corrector| {
            corrector.replace(node, format!("{}.{method}?", base.source().unwrap_or_default()));
        });
    }
}

define_compatibility_rule!(NumberedParametersLimitRule);
impl NumberedParametersLimitRule<'_, '_, '_, '_> {
    fn on_numblock(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(2, 7) {
            return;
        }
        let mut parameters = node
            .descendants()
            .into_iter()
            .filter(|descendant| descendant.kind() == "lvar")
            .filter_map(|descendant| descendant.source())
            .filter(|source| source.len() == 2 && source.starts_with('_') && matches!(source.as_bytes()[1], b'1'..=b'9'))
            .collect::<Vec<_>>();
        parameters.sort_unstable();
        parameters.dedup();
        let count = parameters.len();
        let maximum = self.config_usize("Max", 1).min(9);
        if count <= maximum {
            return;
        }
        let parameter = if maximum > 1 { "parameters" } else { "parameter" };
        self.report(format!("Avoid using more than {maximum} numbered {parameter}; {count} detected."), node);
    }
}

define_compatibility_rule!(AsciiCommentsRule);
impl AsciiCommentsRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let allowed = self.config_values("AllowedChars").to_vec();
        let comments = self.processed_source().comments().to_vec();
        for comment in comments {
            let non_ascii = comment.text.chars().filter(|character| !character.is_ascii()).collect::<Vec<_>>();
            if non_ascii.is_empty()
                || non_ascii.iter().all(|character| allowed.iter().any(|allowed| allowed == &character.to_string()))
            {
                continue;
            }
            let Some((start, run)) = first_non_ascii_run(&comment.text) else { continue; };
            let begin = comment.range.start + comment.text[..start].chars().count();
            let end = begin + run.chars().count();
            let offense = self.range_between(begin, end);
            self.report("Use only ascii symbols in comments.", offense);
        }
    }
}

fn first_non_ascii_run(text: &str) -> Option<(usize, &str)> {
    let start = text.char_indices().find(|(_, character)| !character.is_ascii())?.0;
    let end = text[start..]
        .char_indices()
        .find(|(_, character)| character.is_ascii())
        .map_or(text.len(), |(offset, _)| start + offset);
    Some((start, &text[start..end]))
}

define_compatibility_rule!(OpenStructUseRule);
impl OpenStructUseRule<'_, '_, '_, '_> {
    fn on_const(&mut self, node: NodeRef<'_>) {
        if !root_constant(node, "OpenStruct") {
            return;
        }
        let custom_definition = node.parent().is_some_and(|parent| {
            matches!(parent.kind(), "class" | "module") && node.left_siblings().is_empty()
        });
        if !custom_definition {
            self.report("Avoid using `OpenStruct`; use `Struct`, `Hash`, a class or test doubles instead.", node);
        }
    }
}

define_compatibility_rule!(JsonLoadRule);
impl JsonLoadRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.receiver().is_some_and(|receiver| root_constant(receiver, "JSON"))
            || node.descendants().into_iter().any(|descendant| {
                descendant.kind() == "pair"
                    && descendant.key().and_then(NodeRef::scalar_value_text).as_deref() == Some("create_additions")
            })
        {
            return;
        }
        let method = node.method_name().unwrap_or_default();
        let Some(selector) = self.location_range(node, "selector") else { return; };
        add_offense!(self, selector, message: format!("Prefer `JSON.parse` over `JSON.{method}`."), |corrector| {
            corrector.replace(selector, "parse");
        });
    }
}

define_compatibility_rule!(MixinUsageRule);
impl MixinUsageRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.receiver().is_some()
            || node.arguments().len() != 1
            || node.arguments()[0].kind() != "const"
            || !in_top_level_scope(node)
        {
            return;
        }
        let statement = node.method_name().unwrap_or_default();
        self.report(format!("`{statement}` is used at the top level. Use inside `class` or `module`."), node);
    }
}

fn in_top_level_scope(node: NodeRef<'_>) -> bool {
    node.ancestors().into_iter().all(|ancestor| {
        matches!(ancestor.kind(), "begin" | "kwbegin" | "if" | "def")
    })
}

define_compatibility_rule!(StructNewOverrideRule);
impl StructNewOverrideRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.receiver().is_some_and(|receiver| root_constant(receiver, "Struct")) {
            return;
        }
        for (index, argument) in node.arguments().into_iter().enumerate() {
            if index == 0 && argument.kind() == "str" || !matches!(argument.kind(), "sym" | "str") {
                continue;
            }
            let Some(member_name) = argument.scalar_value_text() else { continue; };
            if !super::lint_builtin_overrides::STRUCT_METHODS.iter().any(|method| *method == member_name.as_bytes()) {
                continue;
            }
            self.report(
                format!("`{}` member overrides `Struct#{member_name}` and it may be unexpected.", argument.source().unwrap_or_default()),
                argument,
            );
        }
    }
}

define_compatibility_rule!(SharedMutableDefaultRule);
impl SharedMutableDefaultRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.receiver().is_some_and(|receiver| root_constant(receiver, "Hash")) {
            return;
        }
        let arguments = node.arguments();
        let Some(argument) = arguments.first().copied() else { return; };
        let capacity_keyword = |candidate: NodeRef<'_>| candidate.kind() == "hash"
            && candidate.pairs().into_iter().any(|pair| {
                pair.key().and_then(NodeRef::scalar_value_text).as_deref() == Some("capacity")
            });
        let mutable = matches!(argument.kind(), "array" | "hash")
            || argument.kind() == "send"
                && argument.method_name() == Some("new")
                && argument.receiver().is_some_and(|receiver| {
                    root_constant(receiver, "Array") || root_constant(receiver, "Hash")
                });
        let offense = arguments.len() == 1 && mutable && !capacity_keyword(argument)
            || arguments.len() == 2
                && argument.kind() == "hash"
                && capacity_keyword(arguments[1]);
        if offense {
            self.report("Do not create a Hash with a mutable default value as the default value can accidentally be changed.", node);
        }
    }
}

define_compatibility_rule!(RedundantArrayConstructorRule);
impl RedundantArrayConstructorRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let method = node.method_name().unwrap_or_default();
        let receiver = node.receiver();
        let (replacement, offense) = match method {
            "new" if receiver.is_some_and(|receiver| root_constant(receiver, "Array"))
                && node.arguments().len() == 1 && node.arguments()[0].kind() == "array" => {
                let (Some(receiver_range), Some(selector)) = (receiver.and_then(|receiver| self.source_range(receiver)), self.location_range(node, "selector")) else { return; };
                (node.arguments()[0].source().unwrap_or_default().to_string(), self.range_between(receiver_range.begin_pos(), selector.end_pos()))
            }
            "[]" if receiver.is_some_and(|receiver| root_constant(receiver, "Array")) => {
                let Some(receiver_range) = receiver.and_then(|receiver| self.source_range(receiver)) else { return; };
                let elements = node.arguments().into_iter().map(|argument| argument.source().unwrap_or_default()).collect::<Vec<_>>().join(", ");
                (format!("[{elements}]"), self.range_between(receiver_range.begin_pos(), receiver_range.end_pos()))
            }
            "Array" if receiver.is_none() && node.arguments().len() == 1 && node.arguments()[0].kind() == "array" => {
                let Some(selector) = self.location_range(node, "selector") else { return; };
                (node.arguments()[0].source().unwrap_or_default().to_string(), self.range_between(selector.begin_pos(), selector.end_pos()))
            }
            _ => return,
        };
        add_offense!(self, offense, message: "Remove the redundant `Array` constructor.", |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(PreferredHashMethodsRule);
impl PreferredHashMethodsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        self.check_hash_method(node);
    }
    fn on_csend(&mut self, node: NodeRef<'_>) {
        self.check_hash_method(node);
    }
    fn check_hash_method(&mut self, node: NodeRef<'_>) {
        if node.arguments().len() != 1 {
            return;
        }
        let current = node.method_name().unwrap_or_default();
        let verbose = self.config_value("EnforcedStyle") == Some("verbose");
        let preferred = match (verbose, current) {
            (false, "has_key?") => "key?",
            (false, "has_value?") => "value?",
            (true, "key?") => "has_key?",
            (true, "value?") => "has_value?",
            _ => return,
        };
        let Some(selector) = self.location_range(node, "selector") else { return; };
        add_offense!(self, selector, message: format!("Use `Hash#{preferred}` instead of `Hash#{current}`."), |corrector| {
            corrector.replace(selector, preferred);
        });
    }
}

define_compatibility_rule!(CharacterLiteralRule);
impl CharacterLiteralRule<'_, '_, '_, '_> {
    fn on_str(&mut self, node: NodeRef<'_>) {
        let Some(source) = node.source().filter(|source| node.character_literal() && (2..=3).contains(&source.len())) else { return; };
        let string = &source[1..];
        let replacement = if string.len() == 2 || string == "'" {
            format!("\"{string}\"")
        } else if string.len() == 1 {
            format!("'{string}'")
        } else {
            return;
        };
        add_offense!(self, node, message: "Do not use the character literal - use string literal instead.", |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(ConstantOverwrittenInRescueRule);
impl ConstantOverwrittenInRescueRule<'_, '_, '_, '_> {
    fn on_resbody(&mut self, node: NodeRef<'_>) {
        let Some(constant) = node.exception_variable().filter(|constant| {
            constant.kind() == "casgn" && node.exceptions().is_empty() && node.body().is_none()
        }) else { return; };
        let (Some(keyword), Some(assoc)) = (self.location_range(node, "keyword"), self.location_range(node, "assoc")) else { return; };
        let removal = self.range_between(keyword.end_pos(), assoc.end_pos());
        add_offense!(self, assoc, message: format!("`{}` is overwritten by `rescue =>`.", constant.source().unwrap_or_default()), |corrector| {
            corrector.remove(removal);
        });
    }
}

define_compatibility_rule!(RequireRelativeSelfPathRule);
impl RequireRelativeSelfPathRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(required_feature) = node.first_argument().and_then(NodeRef::scalar_value_text) else { return; };
        let file_path = self.processed_source().file_path();
        let path = std::path::Path::new(file_path);
        if path.extension().and_then(|extension| extension.to_str()) != Some("rb") {
            return;
        }
        let stem = path.file_stem().and_then(|stem| stem.to_str()).unwrap_or_default();
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or(file_path);
        if file_name != required_feature && stem != required_feature {
            return;
        }
        let Some(expression) = self.source_range(node) else { return; };
        let whole_line = self.range_help().range_by_whole_lines(expression, true);
        let whole_line = self.range_between(whole_line.begin_pos(), whole_line.end_pos());
        add_offense!(self, node, message: "Remove the `require_relative` that requires itself.", |corrector| {
            corrector.remove(whole_line);
        });
    }
}

define_compatibility_rule!(BinaryOperatorParameterNameRule);
impl BinaryOperatorParameterNameRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) {
        let name = node.method_name().unwrap_or_default();
        let operator = (node.operator_method() || matches!(name, "eql?" | "equal?"))
            && !matches!(name, "+@" | "-@" | "[]" | "[]=" | "<<" | "===" | "`" | "=~");
        let arguments = node.arguments();
        if !operator || arguments.len() != 1 || arguments[0].kind() != "arg" {
            return;
        }
        let argument = arguments[0];
        let Some(argument_name) = argument.name().filter(|argument_name| !matches!(*argument_name, "other" | "_other")) else { return; };
        let Some(name_range) = self.location_range(argument, "name") else { return; };
        let occurrences = node.descendants().into_iter().filter(|descendant| {
            matches!(descendant.kind(), "lvar" | "lvasgn") && descendant.name() == Some(argument_name)
        }).filter_map(|descendant| self.location_range(descendant, "name")).collect::<Vec<_>>();
        add_offense!(self, argument, message: format!("When defining the `{name}` operator, name its argument `other`."), |corrector| {
            corrector.replace(name_range, "other");
            for occurrence in occurrences { corrector.replace(occurrence, "other"); }
        });
    }
}

define_compatibility_rule!(AmbiguousAssignmentRule);
impl AmbiguousAssignmentRule<'_, '_, '_, '_> {
    fn on_lvasgn(&mut self, node: NodeRef<'_>) { self.on_asgn(node); }
    fn on_ivasgn(&mut self, node: NodeRef<'_>) { self.on_asgn(node); }
    fn on_cvasgn(&mut self, node: NodeRef<'_>) { self.on_asgn(node); }
    fn on_gvasgn(&mut self, node: NodeRef<'_>) { self.on_asgn(node); }
    fn on_casgn(&mut self, node: NodeRef<'_>) { self.on_asgn(node); }
    fn on_asgn(&mut self, node: NodeRef<'_>) {
        let Some(rhs) = node.value_node() else { return; };
        let (Some(operator), Some(rhs_range)) = (self.location_range(node, "operator"), self.source_range(rhs)) else { return; };
        let range = self.range_between(operator.end_pos().saturating_sub(1), rhs_range.begin_pos() + 1);
        let correction = match self.range_source(&range) {
            "=-" => "-=", "=+" => "+=", "=*" => "*=", "=!" => "!=", _ => return,
        };
        self.report(format!("Suspicious assignment detected. Did you mean `{correction}`?"), range);
    }
}

define_compatibility_rule!(OptionHashCompatibilityRule);
impl OptionHashCompatibilityRule<'_, '_, '_, '_> {
    fn on_args(&mut self, node: NodeRef<'_>) {
        let Some(parent) = node.parent() else { return; };
        if parent.descendants().into_iter().any(|descendant| descendant.kind() == "zsuper")
            || self.config_values("Allowlist").iter().any(|allowed| Some(allowed.as_str()) == parent.method_name())
        {
            return;
        }
        let Some(argument) = node.child_nodes().last().copied().filter(|argument| {
            argument.kind() == "optarg"
                && argument.default_value().is_some_and(|default| default.kind() == "hash" && default.pairs().is_empty())
        }) else { return; };
        let Some(name) = argument.name() else { return; };
        if self.config_values("SuspiciousParamNames").iter().any(|candidate| candidate == name) {
            self.report("Prefer keyword arguments to options hashes.", argument);
        }
    }
}

define_compatibility_rule!(HashNewWithKeywordArgumentsAsDefaultRule);
impl HashNewWithKeywordArgumentsAsDefaultRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.receiver().is_some_and(|receiver| root_constant(receiver, "Hash")) || node.arguments().len() != 1 {
            return;
        }
        let argument = node.arguments()[0];
        if argument.kind() != "hash" || argument.braces() {
            return;
        }
        if argument.pairs().len() == 1
            && argument.pairs()[0].key().and_then(NodeRef::scalar_value_text).as_deref() == Some("capacity")
        {
            return;
        }
        add_offense!(self, argument, message: "Use a hash literal instead of keyword arguments.", |corrector| {
            corrector.wrap(argument, "{", "}");
        });
    }
}
