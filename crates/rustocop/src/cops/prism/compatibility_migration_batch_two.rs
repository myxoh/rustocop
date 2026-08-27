use super::*;
use crate::rubocop::ast::node::core::NodeRef;

define_cops! {
    SpaceBeforeCommentCompatibility => "Layout/SpaceBeforeComment" => compatibility_investigation(SpaceBeforeCommentRule, on_new_investigation),
    SpaceAfterMethodNameCompatibility => "Layout/SpaceAfterMethodName" => compatibility_callbacks(SpaceAfterMethodNameRule, [on_def, on_defs]),
    SpaceAfterNotCompatibility => "Layout/SpaceAfterNot" => compatibility_callbacks(SpaceAfterNotRule, [on_send restrict ["!"]]),
    SpaceBeforeBracketsCompatibility => "Layout/SpaceBeforeBrackets" => compatibility_callbacks(SpaceBeforeBracketsRule, [on_send restrict ["[]", "[]="]]),
    FlipFlopCompatibility => "Lint/FlipFlop" => compatibility_callbacks(FlipFlopRule, [on_iflipflop, on_eflipflop]),
    RescueExceptionCompatibility => "Lint/RescueException" => compatibility_callbacks(RescueExceptionRule, [on_resbody]),
    DuplicateCaseConditionCompatibility => "Lint/DuplicateCaseCondition" => compatibility_callbacks(DuplicateCaseConditionRule, [on_case]),
    EmptyExpressionCompatibility => "Lint/EmptyExpression" => compatibility_callbacks(EmptyExpressionRule, [on_begin]),
    UnifiedIntegerCompatibility => "Lint/UnifiedInteger" => compatibility_callbacks(UnifiedIntegerRule, [on_const]),
    OrAssignmentToConstantCompatibility => "Lint/OrAssignmentToConstant" => compatibility_callbacks(OrAssignmentToConstantRule, [on_or_asgn]),
    EmptyInterpolationCompatibility => "Lint/EmptyInterpolation" => compatibility_callbacks(EmptyInterpolationRule, [on_interpolation]),
    BooleanSymbolCompatibility => "Lint/BooleanSymbol" => compatibility_callbacks(BooleanSymbolRule, [on_sym]),
    IdentityComparisonCompatibility => "Lint/IdentityComparison" => compatibility_callbacks(IdentityComparisonRule, [on_send restrict ["==", "!="]]),
    MarshalLoadCompatibility => "Security/MarshalLoad" => compatibility_callbacks(MarshalLoadRule, [on_send restrict ["load", "restore"]]),
    SymbolLiteralCompatibility => "Style/SymbolLiteral" => compatibility_callbacks(SymbolLiteralRule, [on_sym]),
    SendCompatibility => "Style/Send" => compatibility_callbacks(SendRule, [on_send restrict ["send"]]),
    ImplicitRuntimeErrorCompatibility => "Style/ImplicitRuntimeError" => compatibility_callbacks(ImplicitRuntimeErrorRule, [on_send restrict ["raise", "fail"]]),
    SuperWithArgsParenthesesCompatibility => "Style/SuperWithArgsParentheses" => compatibility_callbacks(SuperWithArgsParenthesesRule, [on_super]),
    StringMethodsCompatibility => "Style/StringMethods" => compatibility_callbacks(StringMethodsRule, [on_send]),
    ColonMethodDefinitionCompatibility => "Style/ColonMethodDefinition" => compatibility_callbacks(ColonMethodDefinitionRule, [on_defs]),
    InlineCommentCompatibility => "Style/InlineComment" => compatibility_investigation(InlineCommentRule, on_new_investigation),
    WhenThenCompatibility => "Style/WhenThen" => compatibility_callbacks(WhenThenRule, [on_when]),
    ProcCompatibility => "Style/Proc" => compatibility_callbacks(ProcRule, [on_block]),
    ArrayJoinCompatibility => "Style/ArrayJoin" => compatibility_callbacks(ArrayJoinRule, [on_send restrict ["*"]]),
    StringCharsCompatibility => "Style/StringChars" => compatibility_callbacks(StringCharsRule, [on_send restrict ["split"]]),
    RedundantFileExtensionInRequireCompatibility => "Style/RedundantFileExtensionInRequire" => compatibility_callbacks(RedundantFileExtensionInRequireRule, [on_send restrict ["require", "require_relative"]]),
    UnlessElseCompatibility => "Style/UnlessElse" => compatibility_callbacks(UnlessElseRule, [on_if]),
    StderrPutsCompatibility => "Style/StderrPuts" => compatibility_callbacks(StderrPutsRule, [on_send restrict ["puts"]]),
    EnvHomeCompatibility => "Style/EnvHome" => compatibility_callbacks(EnvHomeRule, [on_send restrict ["[]", "fetch"]]),
    WhileUntilDoCompatibility => "Style/WhileUntilDo" => compatibility_callbacks(WhileUntilDoRule, [on_while, on_until]),
}

fn root_constant(node: NodeRef<'_>, name: &str) -> bool {
    node.kind() == "const"
        && node.short_name() == Some(name)
        && node
            .namespace()
            .is_none_or(|namespace| namespace.kind() == "cbase")
}

fn call_on_root_constant(node: NodeRef<'_>, receiver: &str, method: &str) -> bool {
    node.method_name() == Some(method) && node.receiver().is_some_and(|node| root_constant(node, receiver))
}

fn literal_source(node: NodeRef<'_>) -> &str {
    node.source().unwrap_or_default()
}

define_compatibility_rule!(SpaceBeforeCommentRule);
impl SpaceBeforeCommentRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        for pair in self.processed_source().sorted_tokens().windows(2) {
            let (token1, token2) = (pair[0], pair[1]);
            if !token2.comment()
                || token1.line != token2.line
                || token1.end_pos() != token2.begin_pos()
            {
                continue;
            }
            let offense = self.owned_character_range(token2.range.clone());
            add_offense!(self, offense.clone(), message: "Put a space before an end-of-line comment.", |corrector| {
                corrector.insert_before(offense, " ");
            });
        }
    }
}

define_compatibility_rule!(SpaceAfterMethodNameRule);
impl SpaceAfterMethodNameRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let Some(args) = node.arguments_node() else { return; };
        if !args.parenthesized_call() { return; }
        let Some(expr) = self.source_range(args) else { return; };
        if expr.begin_pos() == 0 { return; }
        let before = self.range_between(expr.begin_pos() - 1, expr.begin_pos());
        if !self.range_source(&before).starts_with(' ') { return; }
        add_offense!(self, before, message: "Do not put a space between a method name and the opening parenthesis.", |corrector| {
            corrector.remove(before);
        });
    }
}

define_compatibility_rule!(SpaceAfterNotRule);
impl SpaceAfterNotRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let (Some(receiver), Some(selector), Some(expression)) = (
            node.receiver(), self.location_range(node, "selector"), self.source_range(node)
        ) else { return; };
        let Some(receiver_range) = self.source_range(receiver) else { return; };
        if self.range_source(&selector) != "!" || receiver_range.begin_pos() <= expression.begin_pos() + 1 { return; }
        let whitespace = self.range_between(selector.end_pos(), receiver_range.begin_pos());
        add_offense!(self, node, message: "Do not leave space between `!` and its argument.", |corrector| {
            corrector.remove(whitespace);
        });
    }
}

define_compatibility_rule!(SpaceBeforeBracketsRule);
impl SpaceBeforeBracketsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.loc("dot").is_some() { return; }
        let (Some(receiver), Some(selector)) = (node.receiver(), self.location_range(node, "selector")) else { return; };
        let Some(receiver_range) = self.source_range(receiver) else { return; };
        if receiver_range.end_pos() >= selector.begin_pos() { return; }
        let range = self.range_between(receiver_range.end_pos(), selector.begin_pos());
        add_offense!(self, range, message: "Remove the space before the opening brackets.", |corrector| {
            corrector.remove(range);
        });
    }
}

define_compatibility_rule!(FlipFlopRule);
impl FlipFlopRule<'_, '_, '_, '_> {
    fn on_iflipflop(&mut self, node: NodeRef<'_>) { self.report("Avoid the use of flip-flop operators.", node); }
    fn on_eflipflop(&mut self, node: NodeRef<'_>) { self.report("Avoid the use of flip-flop operators.", node); }
}

define_compatibility_rule!(RescueExceptionRule);
impl RescueExceptionRule<'_, '_, '_, '_> {
    fn on_resbody(&mut self, node: NodeRef<'_>) {
        let targets_exception = node
            .first_node()
            .into_iter()
            .flat_map(NodeRef::child_nodes)
            .any(|exception| exception.const_name().as_deref() == Some("Exception"));
        if targets_exception {
            self.report("Avoid rescuing the `Exception` class. Perhaps you meant to rescue `StandardError`?", node);
        }
    }
}

define_compatibility_rule!(DuplicateCaseConditionRule);
impl DuplicateCaseConditionRule<'_, '_, '_, '_> {
    fn on_case(&mut self, node: NodeRef<'_>) {
        let mut previous = Vec::new();
        for condition in node.when_branches().into_iter().flat_map(NodeRef::conditions) {
            if previous.iter().any(|prior: &NodeRef<'_>| prior.structurally_equal(condition)) {
                self.report("Duplicate `when` condition detected.", condition);
            } else {
                previous.push(condition);
            }
        }
    }
}

define_compatibility_rule!(EmptyExpressionRule);
impl EmptyExpressionRule<'_, '_, '_, '_> {
    fn on_begin(&mut self, node: NodeRef<'_>) {
        if node.children().is_empty() { self.report("Avoid empty expressions.", node); }
    }
}

define_compatibility_rule!(UnifiedIntegerRule);
impl UnifiedIntegerRule<'_, '_, '_, '_> {
    fn on_const(&mut self, node: NodeRef<'_>) {
        let Some(klass @ ("Fixnum" | "Bignum")) = node.short_name() else { return; };
        if node.namespace().is_some_and(|namespace| namespace.kind() != "cbase") { return; }
        let message = format!("Use `Integer` instead of `{klass}`.");
        if let Some(name) = self.location_range(node, "name") {
            let correct = self.target_ruby_version().at_least(2, 4);
            add_offense!(self, node, message: message, |corrector| {
                if correct { corrector.replace(name, "Integer"); }
            });
        }
    }
}

define_compatibility_rule!(OrAssignmentToConstantRule);
impl OrAssignmentToConstantRule<'_, '_, '_, '_> {
    fn on_or_asgn(&mut self, node: NodeRef<'_>) {
        if node.lhs().is_none_or(|lhs| lhs.kind() != "casgn") { return; }
        let Some(operator) = self.location_range(node, "operator") else { return; };
        let in_method = node.each_ancestor(&["def", "defs"]).into_iter().next().is_some();
        add_offense!(self, operator, message: "Avoid using or-assignment with constants.", |corrector| {
            if !in_method { corrector.replace(operator, "="); }
        });
    }
}

define_compatibility_rule!(EmptyInterpolationRule);
impl EmptyInterpolationRule<'_, '_, '_, '_> {
    fn on_interpolation(&mut self, node: NodeRef<'_>) {
        if node.each_ancestor(&["array"]).into_iter().any(|array| array.percent_literal(None)) { return; }
        let empty = node.child_nodes().into_iter().all(|child| {
            child.kind() == "nil" || child.basic_literal() && child.scalar_value_text().is_some_and(|value| value.is_empty())
        });
        if empty {
            add_offense!(self, node, message: "Empty interpolation detected.", |corrector| {
                corrector.remove(node);
            });
        }
    }
}

define_compatibility_rule!(BooleanSymbolRule);
impl BooleanSymbolRule<'_, '_, '_, '_> {
    fn on_sym(&mut self, node: NodeRef<'_>) {
        let boolean_value = node.scalar_value_text();
        let Some(boolean @ ("true" | "false")) = boolean_value.as_deref() else { return; };
        if node.parent().is_some_and(|parent| parent.kind() == "array" && parent.percent_literal(Some("symbol"))) { return; }
        let message = format!("Symbol with a boolean name - you probably meant to use `{boolean}`.");
        let source = literal_source(node);
        let pair = node.parent().filter(|parent| parent.kind() == "pair" && parent.loc_is("operator", ":") && parent.first_node() == Some(node));
        let operator = pair.and_then(|parent| self.location_range(parent, "operator"));
        let replacement = if pair.is_some() { format!("{source} =>") } else { source.replace(':', "") };
        add_offense!(self, node, message: message, |corrector| {
            if let Some(operator) = operator { corrector.remove(operator); }
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(IdentityComparisonRule);
impl IdentityComparisonRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(comparison @ ("==" | "!=")) = node.method_name() else { return; };
        let (Some(left), Some(right)) = (node.receiver(), node.first_argument()) else { return; };
        if left.method_name() != Some("object_id") || right.method_name() != Some("object_id") { return; }
        let (Some(receiver), Some(argument)) = (left.receiver(), right.receiver()) else { return; };
        let bang = if comparison == "==" { "" } else { "!" };
        let message = format!("Use `{bang}equal?` instead of `{comparison}` when comparing `object_id`.");
        let replacement = format!("{bang}{}.equal?({})", literal_source(receiver), literal_source(argument));
        add_offense!(self, node, message: message, |corrector| { corrector.replace(node, replacement); });
    }
}

define_compatibility_rule!(MarshalLoadRule);
impl MarshalLoadRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(method @ ("load" | "restore")) = node.method_name() else { return; };
        if !call_on_root_constant(node, "Marshal", method) || node.arguments().len() != 1 { return; }
        if node.first_argument().is_some_and(|argument| call_on_root_constant(argument, "Marshal", "dump")) { return; }
        if let Some(selector) = self.location_range(node, "selector") {
            self.report(format!("Avoid using `Marshal.{method}`."), selector);
        }
    }
}

define_compatibility_rule!(SymbolLiteralRule);
impl SymbolLiteralRule<'_, '_, '_, '_> {
    fn on_sym(&mut self, node: NodeRef<'_>) {
        let source = literal_source(node);
        let quoted = (source.starts_with(":\"") && source.ends_with('"')) || (source.starts_with(":'") && source.ends_with('\''));
        let body = source.get(2..source.len().saturating_sub(1)).unwrap_or_default();
        let word_like = body.chars().next().is_some_and(|first| first == '_' || first.is_ascii_alphabetic())
            && body.chars().all(|character| character == '_' || character.is_ascii_alphanumeric());
        if !quoted || !word_like { return; }
        let replacement = source.replace(['\'', '"'], "");
        add_offense!(self, node, message: "Do not use strings for word-like symbol literals.", |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(SendRule);
impl SendRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.has_arguments() { return; }
        if let Some(selector) = self.location_range(node, "selector") {
            self.report("Prefer `Object#__send__` or `Object#public_send` to `send`.", selector);
        }
    }
}

define_compatibility_rule!(ImplicitRuntimeErrorRule);
impl ImplicitRuntimeErrorRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(method @ ("raise" | "fail")) = node.method_name() else { return; };
        let arguments = node.arguments();
        if node.receiver().is_some() || arguments.len() != 1 || !matches!(arguments[0].kind(), "str" | "dstr") { return; }
        self.report(format!("Use `{method}` with an explicit exception class and message, rather than just a message."), node);
    }
}

define_compatibility_rule!(SuperWithArgsParenthesesRule);
impl SuperWithArgsParenthesesRule<'_, '_, '_, '_> {
    fn on_super(&mut self, node: NodeRef<'_>) {
        if node.parenthesized() || !node.has_arguments() { return; }
        let (Some(keyword), Some(first), Some(last)) = (
            self.location_range(node, "keyword"), node.first_argument(), node.last_argument()
        ) else { return; };
        let Some(first_range) = self.source_range(first) else { return; };
        let between = self.range_between(keyword.end_pos(), first_range.begin_pos());
        add_offense!(self, node, message: "Use parentheses for `super` with arguments.", |corrector| {
            corrector.replace(between, "(");
            corrector.insert_after(last, ")");
        });
    }
}

define_compatibility_rule!(StringMethodsRule);
impl StringMethodsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(current) = node.method_name() else { return; };
        let Some(preferred) = self.config_map("PreferredMethods").and_then(|methods| methods.get(current)).cloned() else { return; };
        let Some(selector) = self.location_range(node, "selector") else { return; };
        let message = format!("Prefer `{preferred}` over `{current}`.");
        add_offense!(self, selector, message: message, |corrector| { corrector.replace(selector, preferred); });
    }
}

define_compatibility_rule!(ColonMethodDefinitionRule);
impl ColonMethodDefinitionRule<'_, '_, '_, '_> {
    fn on_defs(&mut self, node: NodeRef<'_>) {
        if !node.loc_is("operator", "::") { return; }
        let Some(operator) = self.location_range(node, "operator") else { return; };
        add_offense!(self, operator, message: "Do not use `::` for defining class methods.", |corrector| {
            corrector.replace(operator, ".");
        });
    }
}

define_compatibility_rule!(InlineCommentRule);
impl InlineCommentRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        for comment in self.processed_source().comments() {
            let standalone = self.processed_source().line(comment.line.saturating_sub(1))
                .is_some_and(|line| line.trim_start().starts_with('#'));
            let directive = comment.text.starts_with("# rubocop:enable") || comment.text.starts_with("# rubocop:disable");
            if !standalone && !directive {
                let offense = self.owned_character_range(comment.range.clone());
                self.report("Avoid trailing inline comments.", offense);
            }
        }
    }
}

define_compatibility_rule!(WhenThenRule);
impl WhenThenRule<'_, '_, '_, '_> {
    fn on_when(&mut self, node: NodeRef<'_>) {
        if node.multiline() || node.then_keyword() || node.body().is_none() { return; }
        let Some(separator) = self.location_range(node, "begin") else { return; };
        let expression = node.conditions().into_iter().map(literal_source).collect::<Vec<_>>().join(", ");
        let message = format!("Do not use `when {expression};`. Use `when {expression} then` instead.");
        add_offense!(self, separator, message: message, |corrector| { corrector.replace(separator, " then"); });
    }
}

define_compatibility_rule!(ProcRule);
impl ProcRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        let Some(send) = node.send_node() else { return; };
        if !call_on_root_constant(send, "Proc", "new") { return; }
        add_offense!(self, send, message: "Use `proc` instead of `Proc.new`.", |corrector| {
            corrector.replace(send, "proc");
        });
    }
}

define_compatibility_rule!(ArrayJoinRule);
impl ArrayJoinRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let (Some(array), Some(argument), Some(selector)) = (node.receiver(), node.first_argument(), self.location_range(node, "selector")) else { return; };
        if array.kind() != "array" || argument.kind() != "str" || node.arguments().len() != 1 { return; }
        let replacement = format!("{}.join({})", literal_source(array), literal_source(argument));
        add_offense!(self, selector, message: "Favor `Array#join` over `Array#*`.", |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(StringCharsRule);
impl StringCharsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let arguments = node.arguments();
        if arguments.len() != 1 || !matches!(literal_source(arguments[0]), "//" | "''" | "\"\"") { return; }
        let (Some(selector), Some(expression)) = (self.location_range(node, "selector"), self.source_range(node)) else { return; };
        let range = self.range_between(selector.begin_pos(), expression.end_pos());
        let message = format!("Use `chars` instead of `{}`.", self.range_source(&range));
        add_offense!(self, range, message: message, |corrector| { corrector.replace(range, "chars"); });
    }
}

define_compatibility_rule!(RedundantFileExtensionInRequireRule);
impl RedundantFileExtensionInRequireRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.receiver().is_some() || node.arguments().len() != 1 { return; }
        let name = node.arguments()[0];
        if name.kind() != "str" || !name.scalar_value_text().is_some_and(|value| value.ends_with(".rb")) { return; }
        let Some(range) = self.source_range(name) else { return; };
        if range.end_pos() < 4 { return; }
        let extension = self.range_between(range.end_pos() - 4, range.end_pos() - 1);
        add_offense!(self, extension, message: "Redundant `.rb` file extension detected.", |corrector| {
            corrector.remove(extension);
        });
    }
}

define_compatibility_rule!(UnlessElseRule);
impl UnlessElseRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) {
        if !node.unless_keyword() || !node.has_else() { return; }
        let (Some(keyword), Some(else_keyword), Some(end_keyword), Some(condition)) = (
            self.location_range(node, "keyword"), self.location_range(node, "else"), self.location_range(node, "end"), node.condition()
        ) else { return; };
        let Some(condition_range) = self.source_range(condition) else { return; };
        let body_start = self.location_range(node, "begin").map_or(condition_range.end_pos(), |begin| begin.end_pos());
        let body = self.range_between(body_start, else_keyword.begin_pos());
        let alternative = self.range_between(else_keyword.end_pos(), end_keyword.begin_pos());
        add_offense!(self, node, message: "Do not use `unless` with `else`. Rewrite these with the positive case first.", |corrector| {
            corrector.replace(keyword, "if");
            corrector.swap(body, alternative);
        });
    }
}

define_compatibility_rule!(StderrPutsRule);
impl StderrPutsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.kind() == "csend" || !node.has_arguments() { return; }
        let Some(receiver) = node.receiver() else { return; };
        if literal_source(receiver) != "$stderr" && !root_constant(receiver, "STDERR") { return; }
        let (Some(receiver_range), Some(selector)) = (self.source_range(receiver), self.location_range(node, "selector")) else { return; };
        let range = self.range_between(receiver_range.begin_pos(), selector.end_pos());
        let message = format!("Use `warn` instead of `{}.puts` to allow such output to be disabled.", literal_source(receiver));
        add_offense!(self, range, message: message, |corrector| { corrector.replace(range, "warn"); });
    }
}

define_compatibility_rule!(EnvHomeRule);
impl EnvHomeRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !node.receiver().is_some_and(|receiver| root_constant(receiver, "ENV")) { return; }
        let arguments = node.arguments();
        if arguments.is_empty() || arguments[0].scalar_value_text().as_deref() != Some("HOME") { return; }
        if arguments.len() == 2 && arguments[1].kind() != "nil" { return; }
        add_offense!(self, node, message: "Use `Dir.home` instead.", |corrector| { corrector.replace(node, "Dir.home"); });
    }
}

define_compatibility_rule!(WhileUntilDoRule);
impl WhileUntilDoRule<'_, '_, '_, '_> {
    fn on_while(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_until(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        if !node.multiline() || !node.do_keyword() { return; }
        let (Some(condition), Some(begin)) = (node.condition(), self.location_range(node, "begin")) else { return; };
        let Some(condition_range) = self.source_range(condition) else { return; };
        let range = self.range_between(condition_range.end_pos(), begin.end_pos());
        let keyword = node.keyword_name().unwrap_or_default();
        let message = format!("Do not use `do` with multi-line `{keyword}`.");
        add_offense!(self, begin, message: message, |corrector| { corrector.remove(range); });
    }
}
