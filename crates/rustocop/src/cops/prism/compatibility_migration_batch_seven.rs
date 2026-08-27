use super::*;
use crate::rubocop::ast::node::core::NodeRef;

define_cops! {
    NotCompatibility => "Style/Not" => compatibility_callbacks(NotRule, [on_send restrict ["!"]]),
    NilComparisonCompatibility => "Style/NilComparison" => compatibility_callbacks(NilComparisonRule, [on_send]),
    RedundantExceptionCompatibility => "Style/RedundantException" => compatibility_callbacks(RedundantExceptionRule, [on_send restrict ["raise", "fail"]]),
    ComparableBetweenCompatibility => "Style/ComparableBetween" => compatibility_callbacks(ComparableBetweenRule, [on_and]),
    SendWithMixinArgumentCompatibility => "Lint/SendWithMixinArgument" => compatibility_callbacks(SendWithMixinArgumentRule, [on_send]),
    DataInheritanceCompatibility => "Style/DataInheritance" => compatibility_callbacks(DataInheritanceRule, [on_class]),
    DeprecatedAttributeAssignmentCompatibility => "Gemspec/DeprecatedAttributeAssignment" => compatibility_callbacks(DeprecatedAttributeAssignmentRule, [on_block]),
    RedundantSelfAssignmentBranchCompatibility => "Style/RedundantSelfAssignmentBranch" => compatibility_callbacks(RedundantSelfAssignmentBranchRule, [on_lvasgn]),
    ConstantDefinitionInBlockCompatibility => "Lint/ConstantDefinitionInBlock" => compatibility_callbacks(ConstantDefinitionInBlockRule, [on_casgn, on_class, on_module]),
    IncompatibleIoSelectCompatibility => "Lint/IncompatibleIoSelectWithFiberScheduler" => compatibility_callbacks(IncompatibleIoSelectRule, [on_send restrict ["select"]]),
    ArrayCoercionCompatibility => "Style/ArrayCoercion" => compatibility_callbacks(ArrayCoercionRule, [on_array, on_if]),
    DisableDirectiveCompatibility => "Style/DisableCopsWithinSourceCodeDirective" => compatibility_investigation(DisableDirectiveRule, on_new_investigation),
    OrAssignmentCompatibility => "Style/OrAssignment" => compatibility_callbacks(OrAssignmentRule, [on_cvasgn, on_gvasgn, on_if, on_ivasgn, on_lvasgn]),
    AttrCompatibility => "Style/Attr" => compatibility_callbacks(AttrRule, [on_send restrict ["attr"]]),
    PercentStringArrayCompatibility => "Lint/PercentStringArray" => compatibility_callbacks(PercentStringArrayRule, [on_array, on_percent_literal]),
    LambdaCallCompatibility => "Style/LambdaCall" => compatibility_callbacks(LambdaCallRule, [on_send]),
    BarePercentLiteralsCompatibility => "Style/BarePercentLiterals" => compatibility_callbacks(BarePercentLiteralsRule, [on_dstr, on_str]),
}

define_compatibility_rule!(NotRule);
impl NotRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(selector) = self.location_range(node, "selector").filter(|range| self.range_source(range) == "not") else { return; };
        let Some(receiver) = node.receiver() else { return; };
        let end = self.source()[self.source_buffer().byte_position(selector.end_pos()).unwrap_or(0)..].chars().take_while(|character| character.is_whitespace()).count();
        let range = self.range_between(selector.begin_pos(), selector.end_pos() + end);
        let opposite = match receiver.method_name() { Some("==") => Some("!="), Some("!=") => Some("=="), Some("<=") => Some(">"), Some(">") => Some("<="), Some("<") => Some(">="), Some(">=") => Some("<"), _ => None };
        let receiver_operator = self.location_range(receiver, "selector");
        add_offense!(self, selector, message: "Use `!` instead of `not`.", |corrector| {
            if let Some(opposite) = opposite {
                corrector.remove(range);
                if let Some(operator) = receiver_operator { corrector.replace(operator, opposite); }
            } else if receiver.operator_keyword() || receiver.operator_method() || receiver.ternary() {
                corrector.replace(range, "!("); corrector.insert_after(node, ")");
            } else { corrector.replace(range, "!"); }
        });
    }
}

define_compatibility_rule!(NilComparisonRule);
impl NilComparisonRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(receiver) = node.receiver() else { return; };
        let comparison = matches!(node.method_name(), Some("==" | "===")) && node.first_argument().is_some_and(|argument| argument.kind() == "nil");
        let predicate = node.method_name() == Some("nil?") && node.arguments().is_empty();
        let prefer_comparison = self.policy().enforced_style("predicate") == "comparison";
        if prefer_comparison && !predicate || !prefer_comparison && !comparison { return; }
        let Some(selector) = self.location_range(node, "selector") else { return; };
        let message = if prefer_comparison { "Prefer the use of the `==` comparison." } else { "Prefer the use of the `nil?` predicate." };
        let replacement_range = if prefer_comparison {
            let start = self.location_range(node, "dot").map_or(selector.begin_pos(), |dot| dot.begin_pos());
            Some(self.range_between(start, selector.end_pos()))
        } else {
            node.source_range().map(|range| self.range_between(receiver.source_range().map_or(range.start, |range| range.end), range.end))
        };
        add_offense!(self, selector, message: message, |corrector| {
            if let Some(range) = replacement_range {
                corrector.replace(range, if prefer_comparison { " == nil" } else { ".nil?" });
            }
        });
    }
}

define_compatibility_rule!(RedundantExceptionRule);
impl RedundantExceptionRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.receiver().is_some() { return; }
        let arguments = node.arguments();
        if arguments.len() == 2 && runtime_error_constant(arguments[0]) {
            let message = arguments[1]; let rendered = if matches!(message.kind(), "str" | "dstr") { message.source().unwrap_or_default().to_owned() } else { format!("{}.to_s", message.source().unwrap_or_default()) };
            let replacement = if self.location_range(node, "begin").is_some() { format!("{}({rendered})", node.method_name().unwrap_or_default()) } else { format!("{} {rendered}", node.method_name().unwrap_or_default()) };
            add_offense!(self, node, message: "Redundant `RuntimeError` argument can be removed.", |corrector| { corrector.replace(node, replacement); });
        } else if arguments.len() == 1 && arguments[0].method_name() == Some("new") && arguments[0].receiver().is_some_and(runtime_error_constant) && arguments[0].arguments().len() == 1 {
            let call = arguments[0]; let message = call.first_argument().unwrap_or(call); let replacement = if matches!(message.kind(), "str" | "dstr") { message.source().unwrap_or_default().to_owned() } else { format!("{}.to_s", message.source().unwrap_or_default()) };
            add_offense!(self, node, message: "Redundant `RuntimeError.new` call can be replaced with just the message.", |corrector| { corrector.replace(call, replacement); });
        }
    }
}
fn runtime_error_constant(node: NodeRef<'_>) -> bool { node.kind() == "const" && node.short_name() == Some("RuntimeError") && node.namespace().is_none_or(|namespace| namespace.kind() == "cbase") }

define_compatibility_rule!(ComparableBetweenRule);
impl ComparableBetweenRule<'_, '_, '_, '_> {
    fn on_and(&mut self, node: NodeRef<'_>) {
        let (Some(left), Some(right)) = (node.lhs(), node.rhs()) else { return; };
        let Some((left_value, left_bound, left_min)) = comparison_parts(left) else { return; };
        let Some((right_value, right_bound, right_min)) = comparison_parts(right) else { return; };
        if !left_value.structurally_equal(right_value) || left_min == right_min { return; }
        let (minimum, maximum) = if left_min { (left_bound, right_bound) } else { (right_bound, left_bound) };
        let prefer = format!("{}.between?({}, {})", left_value.source().unwrap_or_default(), minimum.source().unwrap_or_default(), maximum.source().unwrap_or_default());
        add_offense!(self, node, message: format!("Prefer `{prefer}` over logical comparison."), |corrector| { corrector.replace(node, prefer); });
    }
}
fn comparison_parts(node: NodeRef<'_>) -> Option<(NodeRef<'_>, NodeRef<'_>, bool)> {
    let receiver = node.receiver()?; let argument = node.first_argument()?;
    match node.method_name()? {
        ">=" => Some((receiver, argument, true)), "<=" => Some((receiver, argument, false)),
        _ => None,
    }
}

define_compatibility_rule!(SendWithMixinArgumentRule);
impl SendWithMixinArgumentRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !matches!(node.method_name(), Some("send" | "public_send" | "__send__")) || !node.receiver().is_some_and(|receiver| receiver.kind() == "const") { return; }
        let arguments = node.arguments(); let Some(method) = arguments.first().and_then(|argument| argument.scalar_value_text()).filter(|method| matches!(method.as_str(), "include" | "prepend" | "extend")) else { return; };
        if arguments.len() < 2 || arguments[1..].iter().any(|argument| argument.kind() != "const") { return; }
        let modules = arguments[1..].iter().filter_map(|argument| argument.source()).collect::<Vec<_>>().join(", ");
        let Some(selector) = self.location_range(node, "selector") else { return; }; let end = node.source_range().map_or(selector.end_pos(), |range| range.end); let offense = self.range_between(selector.begin_pos(), end); let bad = self.range_source(&offense).to_owned();
        add_offense!(self, offense, message: format!("Use `{method} {modules}` instead of `{bad}`."), |corrector| { corrector.replace(offense, format!("{method} {modules}")); });
    }
}

define_compatibility_rule!(DataInheritanceRule);
impl DataInheritanceRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(3, 2) { return; }
        let Some(parent) = node.parent_class().filter(|parent| parent.method_name() == Some("define") && parent.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("Data"))) else { return; };
        let Some(keyword) = self.location_range(node, "keyword") else { return; }; let Some(operator) = self.location_range(node, "operator") else { return; };
        add_offense!(self, parent, message: "Don't extend an instance initialized by `Data.define`. Use a block to customize the class.", |corrector| {
            corrector.remove(keyword); corrector.replace(operator, "=");
            if node.body().is_some() { corrector.insert_after(parent, " do"); }
        });
    }
}

define_compatibility_rule!(DeprecatedAttributeAssignmentRule);
impl DeprecatedAttributeAssignmentRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        if !self.processed_source().file_path().ends_with(".gemspec") { return; }
        if node.method_name() != Some("new") || !node.receiver().is_some_and(|receiver| receiver.source().is_some_and(|source| matches!(source, "Gem::Specification" | "::Gem::Specification"))) { return; }
        let Some(parameter) = node.arguments().first().and_then(|argument| argument.name()) else { return; };
        for assignment in node.descendants() {
            let (target, attribute) = if matches!(assignment.kind(), "op_asgn") { (assignment.lhs(), assignment.lhs().and_then(NodeRef::method_name)) } else { (Some(assignment), assignment.method_name().and_then(|method| method.strip_suffix('='))) };
            let Some(attribute @ ("test_files" | "date" | "specification_version" | "rubygems_version")) = attribute else { continue; };
            if !target.is_some_and(|target| target.receiver().and_then(NodeRef::source) == Some(parameter)) { continue; }
            add_offense!(self, assignment, message: format!("Do not set `{attribute}` in gemspec."), |corrector| { corrector.remove(assignment); });
            break;
        }
    }
}

define_compatibility_rule!(RedundantSelfAssignmentBranchRule);
impl RedundantSelfAssignmentBranchRule<'_, '_, '_, '_> {
    fn on_lvasgn(&mut self, node: NodeRef<'_>) {
        let Some(expression) = node.expression().filter(|expression| expression.kind() == "if") else { return; };
        let (Some(if_branch), Some(else_branch), Some(condition)) = (expression.if_branch(), expression.else_branch(), expression.condition()) else { return; };
        if if_branch.kind() == "begin" || else_branch.kind() == "begin" || else_branch.elsif() { return; }
        let Some(name) = node.name() else { return; };
        let (offense, other, keyword) = if if_branch.source() == Some(name) { (if_branch, else_branch, "unless") } else if else_branch.source() == Some(name) { (else_branch, if_branch, "if") } else { return; };
        let replacement = format!("{} {keyword} {}", other.source().unwrap_or("nil"), condition.source().unwrap_or_default());
        add_offense!(self, offense, message: "Remove the self-assignment branch.", |corrector| { corrector.replace(expression, replacement); });
    }
}

define_compatibility_rule!(ConstantDefinitionInBlockRule);
impl ConstantDefinitionInBlockRule<'_, '_, '_, '_> {
    fn on_casgn(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_class(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_module(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        if node.kind() == "casgn" && node.namespace().is_some() { return; }
        let Some(block) = directly_enclosing_block(node) else { return; };
        if self.config_values("AllowedMethods").iter().any(|allowed| Some(allowed.as_str()) == block.method_name()) { return; }
        self.report("Do not define constants this way within a block.", node);
    }
}

fn directly_enclosing_block(node: NodeRef<'_>) -> Option<NodeRef<'_>> {
    let parent = node.parent()?;
    if matches!(parent.kind(), "block" | "numblock" | "itblock") { return Some(parent); }
    if parent.kind() != "begin" { return None; }
    parent.parent().filter(|ancestor| matches!(ancestor.kind(), "block" | "numblock" | "itblock"))
}

define_compatibility_rule!(IncompatibleIoSelectRule);
impl IncompatibleIoSelectRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("IO") && receiver.namespace().is_none_or(|namespace| namespace.kind() == "cbase")) { return; }
        let arguments = node.arguments(); let read = arguments.first().copied(); let write = arguments.get(1).copied(); let excepts = arguments.get(2).copied(); let timeout = arguments.get(3).copied();
        if excepts.is_some_and(|excepts| excepts.kind() == "array" && !excepts.child_nodes().is_empty()) { return; }
        let compatible = |one: Option<NodeRef<'_>>, other: Option<NodeRef<'_>>| one.is_some_and(|one| one.kind() == "array" && one.child_nodes().len() == 1) && other.is_none_or(|other| other.kind() == "nil" || other.kind() == "array" && other.child_nodes().is_empty());
        let (io, method) = if compatible(read, write) { (read.and_then(|array| array.child_nodes().first().copied()), "wait_readable") } else if compatible(write, read) { (write.and_then(|array| array.child_nodes().first().copied()), "wait_writable") } else { return; };
        let Some(io) = io else { return; }; let suffix = timeout.map_or_else(String::new, |timeout| format!("({})", timeout.source().unwrap_or_default())); let preferred = format!("{}.{method}{suffix}", io.source().unwrap_or_default()); let current = node.source().unwrap_or_default();
        add_offense!(self, node, message: format!("Use `{preferred}` instead of `{current}`."), |corrector| { if !node.parent().is_some_and(|parent| parent.kind().ends_with("asgn")) { corrector.replace(node, preferred); } });
    }
}

define_compatibility_rule!(ArrayCoercionRule);
impl ArrayCoercionRule<'_, '_, '_, '_> {
    fn on_array(&mut self, node: NodeRef<'_>) {
        let children = node.child_nodes(); let Some(splat) = children.first().copied().filter(|_| children.len() == 1 && node.braces() && children[0].kind() == "splat") else { return; }; let Some(argument) = splat.child_nodes().first().copied() else { return; }; let source = argument.source().unwrap_or_default();
        add_offense!(self, node, message: format!("Use `Array({source})` instead of `[*{source}]`."), |corrector| { corrector.replace(node, format!("Array({source})")); });
    }
    fn on_if(&mut self, node: NodeRef<'_>) {
        let Some(condition) = node.condition().filter(|condition| condition.method_name() == Some("is_a?") && condition.first_argument().is_some_and(|argument| argument.short_name() == Some("Array"))) else { return; };
        if node.if_branch().is_some() { return; }
        let Some(variable) = condition.receiver().filter(|receiver| receiver.kind() == "lvar") else { return; }; let Some(assignment) = node.else_branch().filter(|branch| branch.kind() == "lvasgn" && branch.name() == variable.name()) else { return; }; let Some(array) = assignment.expression().filter(|array| array.kind() == "array" && array.child_nodes().len() == 1 && array.child_nodes()[0].name() == variable.name()) else { return; }; let _ = array; let name = variable.name().unwrap_or_default();
        add_offense!(self, node, message: format!("Use `Array({name})` instead of explicit `Array` check."), |corrector| { corrector.replace(node, format!("{name} = Array({name})")); });
    }
}

define_compatibility_rule!(DisableDirectiveRule);
impl DisableDirectiveRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let comments = self.processed_source().comments().to_vec();
        for comment in comments {
            let text = comment.text.strip_prefix('#').unwrap_or_default().trim_start();
            let Some(rest) = text.strip_prefix("rubocop").map(str::trim_start).and_then(|rest| rest.strip_prefix(':')).map(str::trim_start) else { continue; };
            let mut pieces = rest.splitn(2, char::is_whitespace); let mode = pieces.next().unwrap_or_default(); if !matches!(mode, "enable" | "disable" | "todo") { continue; } let cops = pieces.next().unwrap_or_default().split(',').map(str::trim).filter(|cop| !cop.is_empty()).collect::<Vec<_>>(); let allowed = self.config_values("AllowedCops"); let disallowed = cops.iter().filter(|cop| !allowed.iter().any(|allowed| allowed == **cop)).copied().collect::<Vec<_>>(); if disallowed.is_empty() { continue; }
            let message = if allowed.is_empty() { "RuboCop disable/enable directives are not permitted.".to_owned() } else { format!("RuboCop disable/enable directives for `{}` are not permitted.", disallowed.join("`, `")) };
            self.report(message, &comment);
        }
    }
}

define_compatibility_rule!(OrAssignmentRule);
impl OrAssignmentRule<'_, '_, '_, '_> {
    fn on_lvasgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); } fn on_ivasgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); } fn on_cvasgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); } fn on_gvasgn(&mut self, node: NodeRef<'_>) { self.check_assignment(node); }
    fn on_if(&mut self, node: NodeRef<'_>) {
        let Some(condition) = node.condition().filter(|condition| matches!(condition.kind(), "lvar" | "ivar" | "cvar" | "gvar")) else { return; }; if node.if_branch().is_some() { return; } let Some(assignment) = node.else_branch().filter(|branch| matches!(branch.kind(), "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn") && branch.name() == condition.name()) else { return; }; let Some(value) = assignment.expression() else { return; }; let name = assignment.name().unwrap_or_default();
        add_offense!(self, node, message: "Use the double pipe equals operator `||=` instead.", |corrector| { corrector.replace(node, format!("{name} ||= {}", value.source().unwrap_or_default())); });
    }
    fn check_assignment(&mut self, node: NodeRef<'_>) {
        let Some(expression) = node.expression().filter(|expression| expression.kind() == "if") else { return; }; let (Some(condition), Some(if_branch), Some(else_branch)) = (expression.condition(), expression.if_branch(), expression.else_branch()) else { return; }; if condition.name() != node.name() || if_branch.name() != node.name() || else_branch.kind() == "if" { return; } let name = node.name().unwrap_or_default();
        add_offense!(self, node, message: "Use the double pipe equals operator `||=` instead.", |corrector| { corrector.replace(node, format!("{name} ||= {}", else_branch.source().unwrap_or_default())); });
    }
}

define_compatibility_rule!(AttrRule);
impl AttrRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.receiver().is_some() || node.arguments().is_empty() || self.location_range(node, "begin").is_some() { return; }
        let setter = node.last_argument().filter(|argument| matches!(argument.kind(), "true" | "false")); let replacement = if setter.is_some_and(|setter| setter.kind() == "true") { "attr_accessor" } else { "attr_reader" }; let Some(selector) = self.location_range(node, "selector") else { return; };
        let setter_range = setter.map(|setter| { let start = setter.left_sibling().and_then(NodeRef::source_range).map_or_else(|| setter.source_range().map_or(0, |range| range.start), |range| range.end); let end = setter.source_range().map_or(start, |range| range.end); self.range_between(start, end) });
        add_offense!(self, selector, message: format!("Do not use `attr`. Use `{replacement}` instead."), |corrector| { corrector.replace(selector, replacement); if let Some(range) = setter_range { corrector.remove(range); } });
    }
}

define_compatibility_rule!(PercentStringArrayRule);
impl PercentStringArrayRule<'_, '_, '_, '_> {
    fn on_array(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_percent_literal(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        if !node.percent_literal(Some("string")) { return; } let values = node.child_nodes(); if !values.iter().any(|value| value.scalar_value_text().is_some_and(|text| text.chars().any(char::is_alphanumeric) && (text.ends_with(',') || text.len() > 1 && (text.starts_with('\'') && text.ends_with('\'') || text.starts_with('"') && text.ends_with('"'))))) { return; }
        add_offense!(self, node, message: "Within `%w`/`%W`, quotes and ',' are unnecessary and may be unwanted in the resulting strings.", |_corrector| {});
    }
}

define_compatibility_rule!(LambdaCallRule);
impl LambdaCallRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let Some(receiver) = node.receiver() else { return; }; if node.method_name() != Some("call") { return; } let implicit = self.location_range(node, "selector").is_none_or(|selector| self.range_source(&selector).is_empty()); let braces = self.policy().enforced_style("call") == "braces"; if braces == implicit { return; } let arguments = node.arguments().iter().filter_map(|argument| argument.source()).collect::<Vec<_>>().join(", "); let dot = self.location_range(node, "dot").map_or(".", |range| self.range_source(&range)); let preferred = if braces { format!("{}{dot}({arguments})", receiver.source().unwrap_or_default()) } else { format!("{}{dot}call{}", receiver.source().unwrap_or_default(), if arguments.is_empty() { String::new() } else { format!("({arguments})") }) };
        add_offense!(self, node, message: format!("Prefer the use of `{preferred}` over `{}`.", node.source().unwrap_or_default()), |corrector| { corrector.replace(node, preferred); });
    }
}

define_compatibility_rule!(BarePercentLiteralsRule);
impl BarePercentLiteralsRule<'_, '_, '_, '_> {
    fn on_dstr(&mut self, node: NodeRef<'_>) { self.check(node); } fn on_str(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let Some(opening) = self.location_range(node, "begin") else { return; }; let source = self.range_source(&opening); if source.starts_with("<<") { return; } let percent_q = self.policy().enforced_style("bare_percent") == "percent_q"; let wrong = if percent_q { source.starts_with('%') && !source.starts_with("%Q") && source.chars().nth(1).is_some_and(|character| !character.is_alphanumeric()) } else { source.starts_with("%Q") }; if !wrong { return; } let (good, bad) = if percent_q { ("Q", "") } else { ("", "Q") }; let replacement = if source.starts_with("%Q") { source.replacen("%Q", "%", 1) } else { source.replacen('%', "%Q", 1) };
        add_offense!(self, opening, message: format!("Use `%{good}` instead of `%{bad}`."), |corrector| { corrector.replace(opening, replacement); });
    }
}
