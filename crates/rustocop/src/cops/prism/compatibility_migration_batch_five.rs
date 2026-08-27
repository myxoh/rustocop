use super::*;
use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::cop::mixin::range_help::{Side, SurroundingSpace};
use crate::rubocop::cop::mixin::frozen_string_literal::FrozenStringLiteral;

define_cops! {
    NegatedWhileCompatibility => "Style/NegatedWhile" => compatibility_callbacks(NegatedWhileRule, [on_while, on_until]),
    EmptyLambdaParameterCompatibility => "Style/EmptyLambdaParameter" => compatibility_callbacks(EmptyLambdaParameterRule, [on_block]),
    MultilineIfThenCompatibility => "Style/MultilineIfThen" => compatibility_callbacks(MultilineIfThenRule, [on_normal_if_unless]),
    VariableInterpolationCompatibility => "Style/VariableInterpolation" => compatibility_callbacks(VariableInterpolationRule, [on_interpolation]),
    ClassAndModuleCamelCaseCompatibility => "Naming/ClassAndModuleCamelCase" => compatibility_callbacks(ClassAndModuleCamelCaseRule, [on_class, on_module]),
    StripCompatibility => "Style/Strip" => compatibility_callbacks(StripRule, [on_send restrict ["lstrip", "rstrip"]]),
    RedundantCapitalWCompatibility => "Style/RedundantCapitalW" => compatibility_callbacks(RedundantCapitalWRule, [on_array]),
    ArrayIntersectWithSingleElementCompatibility => "Style/ArrayIntersectWithSingleElement" => compatibility_callbacks(ArrayIntersectWithSingleElementRule, [on_send restrict ["intersect?"]]),
    EmptyBlockParameterCompatibility => "Style/EmptyBlockParameter" => compatibility_callbacks(EmptyBlockParameterRule, [on_block]),
    MultipleComparisonCompatibility => "Lint/MultipleComparison" => compatibility_callbacks(MultipleComparisonRule, [on_send restrict ["<", ">", "<=", ">="]]),
    ReverseFindCompatibility => "Style/ReverseFind" => compatibility_callbacks(ReverseFindRule, [on_send restrict ["find", "detect"]]),
    ClassCheckCompatibility => "Style/ClassCheck" => compatibility_callbacks(ClassCheckRule, [on_send restrict ["is_a?", "kind_of?"]]),
    DirEmptyCompatibility => "Style/DirEmpty" => compatibility_callbacks(DirEmptyRule, [on_send restrict ["==", "!=", ">", "empty?", "none?"]]),
    NestedFileDirnameCompatibility => "Style/NestedFileDirname" => compatibility_callbacks(NestedFileDirnameRule, [on_send restrict ["dirname"]]),
    RedundantStringCoercionCompatibility => "Lint/RedundantStringCoercion" => compatibility_callbacks(RedundantStringCoercionRule, [on_send, on_interpolation]),
    DefWithParenthesesCompatibility => "Style/DefWithParentheses" => compatibility_callbacks(DefWithParenthesesRule, [on_def, on_defs]),
    NilLambdaCompatibility => "Style/NilLambda" => compatibility_callbacks(NilLambdaRule, [on_block]),
    PercentSymbolArrayCompatibility => "Lint/PercentSymbolArray" => compatibility_callbacks(PercentSymbolArrayRule, [on_array]),
    SingleArgumentDigCompatibility => "Style/SingleArgumentDig" => compatibility_callbacks(SingleArgumentDigRule, [on_send restrict ["dig"]]),
    BinaryOperatorWithIdenticalOperandsCompatibility => "Lint/BinaryOperatorWithIdenticalOperands" => compatibility_callbacks(BinaryOperatorWithIdenticalOperandsRule, [on_send, on_and, on_or]),
    InterpolationCheckCompatibility => "Lint/InterpolationCheck" => compatibility_callbacks(InterpolationCheckRule, [on_str]),
    RedundantDirGlobSortCompatibility => "Lint/RedundantDirGlobSort" => compatibility_callbacks(RedundantDirGlobSortRule, [on_send restrict ["sort"]]),
    FileEmptyCompatibility => "Style/FileEmpty" => compatibility_callbacks(FileEmptyRule, [on_send restrict [">=", "!=", "==", "zero?", "empty?"]]),
    MapToSetCompatibility => "Style/MapToSet" => compatibility_callbacks(MapToSetRule, [on_send restrict ["to_set"]]),
    OptionalBooleanParameterCompatibility => "Style/OptionalBooleanParameter" => compatibility_callbacks(OptionalBooleanParameterRule, [on_def, on_defs]),
    RedundantInterpolationUnfreezeCompatibility => "Style/RedundantInterpolationUnfreeze" => compatibility_callbacks(RedundantInterpolationUnfreezeRule, [on_dstr]),
    UnpackFirstCompatibility => "Style/UnpackFirst" => compatibility_callbacks(UnpackFirstRule, [on_send restrict ["first", "[]", "slice", "at"]]),
    YAMLFileReadCompatibility => "Style/YAMLFileRead" => compatibility_callbacks(YAMLFileReadRule, [on_send restrict ["load", "safe_load", "parse"]]),
    ExactRegexpMatchCompatibility => "Style/ExactRegexpMatch" => compatibility_callbacks(ExactRegexpMatchRule, [on_send restrict ["=~", "===", "!~", "match", "match?"]]),
    TrailingBodyOnModuleCompatibility => "Style/TrailingBodyOnModule" => compatibility_callbacks(TrailingBodyOnModuleRule, [on_module]),
    TrailingBodyOnClassCompatibility => "Style/TrailingBodyOnClass" => compatibility_callbacks(TrailingBodyOnClassRule, [on_class, on_sclass]),
    IfUnlessModifierOfIfUnlessCompatibility => "Style/IfUnlessModifierOfIfUnless" => compatibility_callbacks(IfUnlessModifierOfIfUnlessRule, [on_if]),
    TrailingBodyOnMethodDefinitionCompatibility => "Style/TrailingBodyOnMethodDefinition" => compatibility_callbacks(TrailingBodyOnMethodDefinitionRule, [on_def, on_defs]),
    MultilineIfModifierCompatibility => "Style/MultilineIfModifier" => compatibility_callbacks(MultilineIfModifierRule, [on_if]),
    InPatternThenCompatibility => "Style/InPatternThen" => compatibility_callbacks(InPatternThenRule, [on_in_pattern]),
    MultilineInPatternThenCompatibility => "Style/MultilineInPatternThen" => compatibility_callbacks(MultilineInPatternThenRule, [on_in_pattern]),
    MultilineWhenThenCompatibility => "Style/MultilineWhenThen" => compatibility_callbacks(MultilineWhenThenRule, [on_when]),
    TrailingMethodEndStatementCompatibility => "Style/TrailingMethodEndStatement" => compatibility_callbacks(TrailingMethodEndStatementRule, [on_def, on_defs]),
    WhileUntilModifierCompatibility => "Style/WhileUntilModifier" => compatibility_callbacks(WhileUntilModifierRule, [on_while, on_until]),
    EmptyLinesAroundBeginBodyCompatibility => "Layout/EmptyLinesAroundBeginBody" => compatibility_callbacks(EmptyLinesAroundBeginBodyRule, [on_kwbegin]),
    InitialIndentationCompatibility => "Layout/InitialIndentation" => compatibility_investigation(InitialIndentationRule, on_new_investigation),
    DoubleCopDisableDirectiveCompatibility => "Style/DoubleCopDisableDirective" => compatibility_investigation(DoubleCopDisableDirectiveRule, on_new_investigation),
    RequireRangeParenthesesCompatibility => "Lint/RequireRangeParentheses" => compatibility_callbacks(RequireRangeParenthesesRule, [on_irange, on_erange]),
    PercentQLiteralsCompatibility => "Style/PercentQLiterals" => compatibility_callbacks(PercentQLiteralsRule, [on_str]),
    RedundantFreezeCompatibility => "Style/RedundantFreeze" => compatibility_callbacks(RedundantFreezeRule, [on_send restrict ["freeze"]]),
    MultilineBlockChainCompatibility => "Style/MultilineBlockChain" => compatibility_callbacks(MultilineBlockChainRule, [on_block, on_numblock, on_itblock]),
    ClassMethodsCompatibility => "Style/ClassMethods" => compatibility_callbacks(ClassMethodsRule, [on_class, on_module]),
    MethodCalledOnDoEndBlockCompatibility => "Style/MethodCalledOnDoEndBlock" => compatibility_callbacks(MethodCalledOnDoEndBlockRule, [on_send]),
    EmptyHeredocCompatibility => "Style/EmptyHeredoc" => compatibility_callbacks(EmptyHeredocRule, [on_str, on_dstr]),
    ConditionPositionCompatibility => "Layout/ConditionPosition" => compatibility_callbacks(ConditionPositionRule, [on_if, on_while, on_until]),
}

define_compatibility_rule!(BinaryOperatorWithIdenticalOperandsRule);
impl BinaryOperatorWithIdenticalOperandsRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(operator) = node.method_name().filter(|operator| {
            ["==", "!=", "===", "<=>", "=~", "&&", "||", ">", ">=", "<", "<=", "|", "^"]
                .contains(operator)
        }) else { return; };
        let (Some(receiver), Some(argument)) = (node.receiver(), node.first_argument()) else { return; };
        if node.arguments().len() == 1 && receiver.structurally_equal(argument) {
            self.report(format!("Binary operator `{operator}` has identical operands."), node);
        }
    }
    fn on_and(&mut self, node: NodeRef<'_>) { self.check_logical(node); }
    fn on_or(&mut self, node: NodeRef<'_>) { self.check_logical(node); }
    fn check_logical(&mut self, node: NodeRef<'_>) {
        let (Some(lhs), Some(rhs), Some(operator)) = (node.lhs(), node.rhs(), node.operator()) else { return; };
        if lhs.structurally_equal(rhs) {
            self.report(format!("Binary operator `{operator}` has identical operands."), node);
        }
    }
}

define_compatibility_rule!(InterpolationCheckRule);
impl InterpolationCheckRule<'_, '_, '_, '_> {
    fn on_str(&mut self, node: NodeRef<'_>) {
        if node.parent().is_some_and(|parent| parent.kind() == "regexp") { return; }
        let Some(source) = node.source().filter(|source| {
            source.starts_with('\'') && source.ends_with('\'') && has_unescaped_interpolation(source) && !source.starts_with("<<")
        }) else { return; };
        let replacement = if source.contains('"') {
            format!("%{{{}}}", &source[1..source.len() - 1])
        } else {
            format!("\"{}\"", &source[1..source.len() - 1])
        };
        let probe = format!("def __rustocop_interpolation_probe__; {replacement}; end");
        let parsed = ruby_prism::parse(probe.as_bytes());
        if parsed.errors().next().is_some() || !contains_interpolated_string(&parsed.node()) { return; }
        let Some(begin) = self.location_range(node, "begin") else { return; };
        let Some(end) = self.location_range(node, "end") else { return; };
        let (opening, closing) = if source.contains('"') { ("%{", "}") } else { ("\"", "\"") };
        add_offense!(self, node, message: "Interpolation in single quoted string detected. Use double quoted strings if you need interpolation.", |corrector| {
            corrector.replace(begin, opening);
            corrector.replace(end, closing);
        });
    }
}

fn has_unescaped_interpolation(source: &str) -> bool {
    source.match_indices("#{").any(|(index, _)| {
        !source[..index].ends_with('\\')
    })
}

fn contains_interpolated_string(root: &ruby_prism::Node<'_>) -> bool {
    struct Finder(bool);
    impl<'pr> ruby_prism::Visit<'pr> for Finder {
        fn visit_interpolated_string_node(&mut self, _node: &ruby_prism::InterpolatedStringNode<'pr>) { self.0 = true; }
    }
    let mut finder = Finder(false);
    ruby_prism::Visit::visit(&mut finder, root);
    finder.0
}

define_compatibility_rule!(RedundantDirGlobSortRule);
impl RedundantDirGlobSortRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(3, 0) { return; }
        let Some(glob) = node.receiver().filter(|receiver| matches!(receiver.method_name(), Some("glob" | "[]"))) else { return; };
        let Some(constant) = glob.receiver().filter(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("Dir")) else { return; };
        let _ = constant;
        if glob.arguments().len() >= 2 || glob.first_argument().is_some_and(|argument| argument.kind() == "splat") { return; }
        let (Some(selector), Some(dot)) = (self.location_range(node, "selector"), self.location_range(node, "dot")) else { return; };
        add_offense!(self, selector, message: "Remove redundant `sort`.", |corrector| {
            corrector.remove(selector);
            corrector.remove(dot);
        });
    }
}

define_compatibility_rule!(FileEmptyRule);
impl FileEmptyRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(2, 4) { return; }
        let Some((constant, argument, negate)) = file_empty_parts(node) else { return; };
        let preferred = format!("{}.empty?({})", constant.source().unwrap_or_default(), argument.source().unwrap_or_default());
        add_offense!(self, node, message: format!("Use `{preferred}` instead."), |corrector| {
            corrector.replace(node, format!("{}{preferred}", if negate { "!" } else { "" }));
        });
    }
}

fn file_call_parts<'ast>(node: NodeRef<'ast>, methods: &[&str]) -> Option<(NodeRef<'ast>, NodeRef<'ast>)> {
    if !methods.contains(&node.method_name()?) || node.arguments().len() != 1 { return None; }
    let constant = node.receiver()?.filter_const("File", "FileTest")?;
    Some((constant, node.first_argument()?))
}

trait CompatibilityConstantFilter<'ast> {
    fn filter_const(self, first: &str, second: &str) -> Option<NodeRef<'ast>>;
}
impl<'ast> CompatibilityConstantFilter<'ast> for NodeRef<'ast> {
    fn filter_const(self, first: &str, second: &str) -> Option<NodeRef<'ast>> {
        (self.kind() == "const" && matches!(self.short_name(), Some(name) if name == first || name == second)).then_some(self)
    }
}

fn file_empty_parts(node: NodeRef<'_>) -> Option<(NodeRef<'_>, NodeRef<'_>, bool)> {
    match node.method_name()? {
        "zero?" => {
            if let Some(parts) = file_call_parts(node, &["zero?"]) { return Some((parts.0, parts.1, false)); }
            let size = node.receiver()?;
            let parts = file_call_parts(size, &["size"])?;
            Some((parts.0, parts.1, false))
        }
        "empty?" => {
            let parts = file_call_parts(node.receiver()?, &["read", "binread"])?;
            Some((parts.0, parts.1, false))
        }
        comparison @ ("==" | "!=" | ">=") => {
            let expected = node.first_argument()?;
            let mut operation = node.receiver()?;
            let mut negated = false;
            if operation.method_name() == Some("!") {
                operation = operation.receiver()?;
                negated = true;
            }
            if operation.method_name() == Some("size") && matches!(comparison, "==" | ">=")
                && expected.kind() == "int" && expected.scalar_value_text().as_deref() == Some("0") {
                let parts = file_call_parts(operation, &["size"])?;
                Some((parts.0, parts.1, negated ^ (comparison == ">=")))
            } else if matches!(operation.method_name(), Some("read" | "binread"))
                && expected.kind() == "str" && expected.str_content() == Some("") {
                let parts = file_call_parts(operation, &["read", "binread"])?;
                Some((parts.0, parts.1, negated ^ (comparison == "!=")))
            } else { None }
        }
        _ => None,
    }
}

define_compatibility_rule!(MapToSetRule);
impl MapToSetRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.block_literal() { return; }
        let Some(receiver) = node.receiver() else { return; };
        let map = if matches!(receiver.kind(), "block" | "numblock" | "itblock") {
            receiver.send_node()
        } else { Some(receiver) };
        let Some(map) = map.filter(|map| matches!(map.method_name(), Some("map" | "collect"))) else { return; };
        let valid = receiver.kind() != "send" || map.last_argument().is_some_and(|argument| {
            argument.kind() == "block_pass" && argument.node_child(0).is_some_and(|value| value.kind() == "sym")
        });
        if !valid { return; }
        let (Some(map_selector), Some(dot), Some(to_set_selector)) = (
            self.location_range(map, "selector"), self.location_range(node, "dot"), self.location_range(node, "selector")
        ) else { return; };
        let method = self.range_source(&map_selector).to_owned();
        let range_help = self.range_help();
        let removal = range_help.range_with_surrounding_space(
            range_help.range_between(dot.begin_pos(), to_set_selector.end_pos()),
            SurroundingSpace { side: Side::Left, ..SurroundingSpace::default() },
        );
        let removal = self.owned_range(removal);
        add_offense!(self, map_selector, message: format!("Pass a block to `to_set` instead of calling `{method}.to_set`."), |corrector| {
            corrector.remove(removal);
            corrector.replace(map_selector, "to_set");
        });
    }
}

define_compatibility_rule!(OptionalBooleanParameterRule);
impl OptionalBooleanParameterRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn check_definition(&mut self, node: NodeRef<'_>) {
        if node.method_name().is_some_and(|name| self.allowed_methods().allowed_method(name)) { return; }
        for argument in node.arguments().into_iter().filter(|argument| argument.kind() == "optarg") {
            let Some(default) = argument.default_value().filter(|value| matches!(value.kind(), "true" | "false")) else { continue; };
            let (Some(name), Some(original)) = (argument.name(), argument.source()) else { continue; };
            let replacement = format!("{name}: {}", default.source().unwrap_or_default());
            self.report(format!("Prefer keyword arguments for arguments with a boolean default value; use `{replacement}` instead of `{original}`."), argument);
        }
    }
}

define_compatibility_rule!(RedundantInterpolationUnfreezeRule);
impl RedundantInterpolationUnfreezeRule<'_, '_, '_, '_> {
    fn on_dstr(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(3, 0) { return; }
        let strings = FrozenStringLiteral::new(self.processed_source(), self.target_ruby_version().as_f64(), None);
        if strings.uninterpolated_string(node) || strings.uninterpolated_heredoc(node) { return; }
        let mut string = node;
        while let Some(parent) = string.parent().filter(|parent| parent.kind() == "dstr") {
            string = parent;
        }
        let Some(parent) = string.parent().filter(|parent| parent.call_type()) else { return; };
        let offense = if matches!(parent.method_name(), Some("+@" | "dup"))
            && parent.receiver() == Some(string) && parent.arguments().is_empty() {
            self.location_range(parent, "selector")
        } else if parent.method_name() == Some("new") && parent.arguments().len() == 1
            && parent.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("String")) {
            let Some(start) = parent.source_range().map(|range| range.start) else { return; };
            let Some(selector) = self.location_range(parent, "selector") else { return; };
            Some(self.range_between(start, selector.end_pos()))
        } else { None };
        let Some(offense) = offense else { return; };
        add_offense!(self, offense, message: "Don't unfreeze interpolated strings as they are already unfrozen.", |corrector| {
            corrector.replace(parent, string.source().unwrap_or_default());
        });
    }
}

define_compatibility_rule!(UnpackFirstRule);
impl UnpackFirstRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(2, 4) { return; }
        let tail = node.method_name().unwrap_or_default();
        let supported = tail == "first" && node.arguments().is_empty()
            || matches!(tail, "[]" | "slice" | "at") && node.arguments().len() == 1
                && node.first_argument().is_some_and(|arg| arg.kind() == "int" && arg.scalar_value_text().as_deref() == Some("0"));
        if !supported { return; }
        let Some(unpack) = node.receiver().filter(|receiver| receiver.method_name() == Some("unpack") && receiver.arguments().len() == 1) else { return; };
        let (Some(selector), Some(expression)) = (self.location_range(unpack, "selector"), self.source_range(node)) else { return; };
        let offense = self.range_between(selector.begin_pos(), expression.end_pos());
        let current = self.range_source(&offense).to_owned();
        let format = unpack.first_argument().and_then(NodeRef::source).unwrap_or_default();
        let removal = self.range_between(unpack.source_range().map_or(selector.end_pos(), |range| range.end), expression.end_pos());
        add_offense!(self, offense, message: format!("Use `unpack1({format})` instead of `{current}`."), |corrector| {
            corrector.remove(removal);
            corrector.replace(selector, "unpack1");
        });
    }
}

define_compatibility_rule!(YAMLFileReadRule);
impl YAMLFileReadRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.method_name() == Some("safe_load") && !self.target_ruby_version().at_least(2, 8) { return; }
        let Some(yaml) = node.receiver().filter(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("YAML")) else { return; };
        let _ = yaml;
        let Some(read) = node.first_argument().filter(|argument| argument.method_name() == Some("read")) else { return; };
        if !read.receiver().is_some_and(|receiver| receiver.kind() == "const" && receiver.short_name() == Some("File")) { return; }
        let Some(path) = read.first_argument() else { return; };
        let rest = node.arguments().into_iter().skip(1).filter_map(NodeRef::source).collect::<Vec<_>>();
        let suffix = if rest.is_empty() { String::new() } else { format!(", {}", rest.join(", ")) };
        let method = node.method_name().unwrap_or_default();
        let prefer = format!("{method}_file({}{suffix})", path.source().unwrap_or_default());
        let (Some(selector), Some(expression)) = (self.location_range(node, "selector"), self.source_range(node)) else { return; };
        let offense = self.range_between(selector.begin_pos(), expression.end_pos());
        add_offense!(self, offense, message: format!("Use `{prefer}` instead."), |corrector| {
            corrector.replace(offense, prefer);
        });
    }
}

define_compatibility_rule!(ExactRegexpMatchRule);
impl ExactRegexpMatchRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let (Some(receiver), Some(regexp)) = (node.receiver(), node.first_argument().filter(|argument| argument.kind() == "regexp")) else { return; };
        if regexp.regexp_options().len() != 0 { return; }
        let source = regexp.source().unwrap_or_default();
        let body = source.strip_prefix('/').and_then(|body| body.strip_suffix('/')).unwrap_or_default();
        let Some(literal) = body.strip_prefix("\\A").and_then(|body| body.strip_suffix("\\z")) else { return; };
        if literal.is_empty() || literal.chars().any(|character| "*+?{}[]()|\\".contains(character)) { return; }
        let operator = if node.method_name() == Some("!~") { "!=" } else { "==" };
        let prefer = format!("{} {operator} '{}'", receiver.source().unwrap_or_default(), literal);
        add_offense!(self, node, message: format!("Use `{prefer}`."), |corrector| {
            corrector.replace(node, prefer);
        });
    }
}

fn first_body_part(mut node: NodeRef<'_>) -> NodeRef<'_> {
    // Malformed or recovery ASTs can contain self-referential body links. RuboCop
    // walks these nodes recursively; keep the same traversal while bounding it
    // so a single project file cannot overflow Rust's stack.
    for _ in 0..64 {
        let next = match node.kind() {
            "begin" => node.child_nodes().first().copied(),
            "rescue" | "ensure" | "kwbegin" => node.body(),
            _ => None,
        };
        let Some(next) = next.filter(|next| next.id() != node.id()) else { break; };
        node = next;
    }
    node
}

fn last_body_part(mut node: NodeRef<'_>) -> NodeRef<'_> {
    for _ in 0..64 {
        let next = match node.kind() {
            "begin" => node.child_nodes().last().copied(),
            "ensure" => node.ensure_branch(),
            "rescue" => node.branches().into_iter().rev().flatten().next(),
            "resbody" | "kwbegin" => node.body(),
            _ => None,
        };
        let Some(next) = next.filter(|next| next.id() != node.id()) else { break; };
        node = next;
    }
    node
}

fn correct_trailing_body(
    context: &mut CompatibilityCopContext<'_, '_, '_>,
    node: NodeRef<'_>,
    header_end: usize,
    body: NodeRef<'_>,
    message: &str,
) {
    let first = first_body_part(body);
    let (Some(first_range), Some(node_range)) = (first.source_range(), node.source_range()) else { return; };
    if node.first_line() != first.first_line() || node.single_line() { return; }
    let indentation_width = context.related_config_value("Layout/IndentationWidth", "Width")
        .and_then(|width| width.parse::<usize>().ok()).unwrap_or(2);
    let mut replacement = context.source_buffer().slice(header_end..first_range.start).to_owned();
    if let Some(semicolon) = replacement.find(';') { replacement.remove(semicolon); }
    replacement.push('\n');
    replacement.push_str(&" ".repeat(node.column() + indentation_width));
    let edit = context.range_between(header_end, first_range.start);
    let trailing_comment = context.processed_source().comments().iter().find(|comment| {
        comment.line == first.first_line() && comment.range.start >= first_range.end
    }).map(|comment| {
        let line_tail = context.source().chars().skip(comment.range.end).take_while(|character| *character != '\n').count();
        (comment.text.trim_end().to_owned(), context.range_between(comment.range.start, comment.range.end + line_tail))
    });
    add_offense!(context, first, message: message, |corrector| {
        corrector.replace(edit, replacement);
        if let Some((text, comment_range)) = trailing_comment {
            corrector.insert_before(node, format!("{text}\n{}", " ".repeat(node.column())));
            corrector.remove(comment_range);
        }
    });
    let _ = node_range;
}

define_compatibility_rule!(TrailingBodyOnModuleRule);
impl TrailingBodyOnModuleRule<'_, '_, '_, '_> {
    fn on_module(&mut self, node: NodeRef<'_>) {
        let (Some(name), Some(body)) = (node.node_child(0), node.body()) else { return; };
        let Some(header_end) = name.source_range().map(|range| range.end) else { return; };
        correct_trailing_body(self.context, node, header_end, body, "Place the first line of module body on its own line.");
    }
}

define_compatibility_rule!(TrailingBodyOnClassRule);
impl TrailingBodyOnClassRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) { self.check_class(node); }
    fn on_sclass(&mut self, node: NodeRef<'_>) { self.check_class(node); }
    fn check_class(&mut self, node: NodeRef<'_>) {
        let (header, body) = if node.kind() == "class" {
            (node.parent_class().or_else(|| node.node_child(0)), node.body())
        } else { (node.node_child(0), node.body()) };
        let (Some(header), Some(body)) = (header, body) else { return; };
        let Some(header_end) = header.source_range().map(|range| range.end) else { return; };
        correct_trailing_body(self.context, node, header_end, body, "Place the first line of class body on its own line.");
    }
}

define_compatibility_rule!(TrailingBodyOnMethodDefinitionRule);
impl TrailingBodyOnMethodDefinitionRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn check_definition(&mut self, node: NodeRef<'_>) {
        if node.endless() || node.single_line() { return; }
        let Some(body) = node.body() else { return; };
        let header_end = node.arguments_node().and_then(NodeRef::source_range).map(|range| range.end)
            .or_else(|| self.location_range(node, "name").map(|range| range.end_pos()));
        let Some(header_end) = header_end else { return; };
        correct_trailing_body(self.context, node, header_end, body, "Place the first line of a multi-line method definition's body on its own line.");
    }
}

define_compatibility_rule!(IfUnlessModifierOfIfUnlessRule);
impl IfUnlessModifierOfIfUnlessRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) {
        if !node.modifier_form() { return; }
        let Some(condition) = node.condition() else { return; };
        let body = node.if_branch().filter(|body| body.kind() == "if").or_else(|| {
            node.child_nodes().into_iter().find(|child| *child != condition && child.kind() == "if")
        });
        let (Some(body), Some(keyword), Some(body_range), Some(condition_range)) = (
            body, node.keyword_name(), body.and_then(NodeRef::source_range), condition.source_range()
        ) else { return; };
        if body.kind() != "if" { return; }
        let Some(keyword_range) = self.location_range(node, "keyword") else { return; };
        let removal = self.range_between(body_range.end, condition_range.end);
        add_offense!(self, keyword_range, message: format!("Avoid modifier `{keyword}` after another conditional."), |corrector| {
            corrector.wrap(body, format!("{keyword} {}\n", condition.source().unwrap_or_default()), "\nend");
            corrector.remove(removal);
        });
    }
}

define_compatibility_rule!(MultilineIfModifierRule);
impl MultilineIfModifierRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) {
        if !node.modifier_form() { return; }
        let (Some(body), Some(condition), Some(keyword)) = (node.if_branch(), node.condition(), node.keyword_name()) else { return; };
        if !body.multiline() || node.ancestors().into_iter().any(|ancestor| ancestor.kind() == "if" && ancestor.modifier_form()) { return; }
        let indentation = " ".repeat(node.column());
        let width = self.related_config_value("Layout/IndentationWidth", "Width").and_then(|value| value.parse::<usize>().ok()).unwrap_or(2);
        let body_indent = format!("{indentation}{}", " ".repeat(width));
        let original_indent = " ".repeat(node.column());
        let rendered_body = body.source().unwrap_or_default().lines().enumerate().map(|(index, line)| {
            let line = if index == 0 { line } else { line.strip_prefix(&original_indent).unwrap_or(line) };
            format!("{body_indent}{line}")
        }).collect::<Vec<_>>().join("\n");
        let replacement = format!("{keyword} {}\n{rendered_body}\n{indentation}end", condition.source().unwrap_or_default());
        add_offense!(self, node, message: format!("Favor a normal {keyword}-statement over a modifier clause in a multiline statement."), |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(InPatternThenRule);
impl InPatternThenRule<'_, '_, '_, '_> {
    fn on_in_pattern(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(2, 7) || node.then_keyword() { return; }
        let (Some(pattern), Some(body), Some(separator)) = (node.pattern(), node.body(), self.location_range(node, "begin")) else { return; };
        if pattern.first_line() != body.first_line() { return; }
        if self.range_source(&separator) != ";" { return; }
        let pattern_source = pattern.source().unwrap_or_default();
        add_offense!(self, separator, message: format!("Do not use `in {pattern_source};`. Use `in {pattern_source} then` instead."), |corrector| {
            corrector.replace(separator, " then");
        });
    }
}

fn remove_multiline_then(context: &mut CompatibilityCopContext<'_, '_, '_>, node: NodeRef<'_>, header: NodeRef<'_>, message: &str) {
    let Some(begin) = context.location_range(node, "begin") else { return; };
    if context.range_source(&begin) != "then" { return; }
    let body = node.body();
    if !header.single_line() || body.is_some_and(|body| header.first_line() == body.first_line()) { return; }
    let range_help = context.range_help();
    let removal = range_help.range_with_surrounding_space(
        range_help.range_between(begin.begin_pos(), begin.end_pos()),
        SurroundingSpace { side: Side::Left, whitespace: false, newlines: false, ..SurroundingSpace::default() },
    );
    let removal = context.owned_range(removal);
    add_offense!(context, begin, message: message, |corrector| { corrector.remove(removal); });
}

define_compatibility_rule!(MultilineInPatternThenRule);
impl MultilineInPatternThenRule<'_, '_, '_, '_> {
    fn on_in_pattern(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(2, 7) { return; }
        let Some(pattern) = node.pattern() else { return; };
        remove_multiline_then(self.context, node, pattern, "Do not use `then` for multiline `in` statement.");
    }
}

define_compatibility_rule!(MultilineWhenThenRule);
impl MultilineWhenThenRule<'_, '_, '_, '_> {
    fn on_when(&mut self, node: NodeRef<'_>) {
        let conditions = node.conditions();
        let (Some(first), Some(last)) = (conditions.first().copied(), conditions.last().copied()) else { return; };
        if first.first_line() != last.last_line() { return; }
        remove_multiline_then(self.context, node, last, "Do not use `then` for multiline `when` statement.");
    }
}

define_compatibility_rule!(TrailingMethodEndStatementRule);
impl TrailingMethodEndStatementRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check_definition(node); }
    fn check_definition(&mut self, node: NodeRef<'_>) {
        if !node.multiline() || node.endless() { return; }
        let (Some(_body), Some(end)) = (node.body(), self.location_range(node, "end")) else { return; };
        let before = self.source().chars().take(end.begin_pos()).collect::<String>();
        if before.rsplit_once('\n').map_or(before.as_str(), |(_, line)| line).trim().is_empty() { return; }
        add_offense!(self, end, message: "Place the end statement of a multi-line method on its own line.", |corrector| {
            corrector.insert_before(end, format!("\n{}", " ".repeat(node.loc_column("keyword").unwrap_or(node.column()))));
        });
    }
}

define_compatibility_rule!(WhileUntilModifierRule);
impl WhileUntilModifierRule<'_, '_, '_, '_> {
    fn on_while(&mut self, node: NodeRef<'_>) { self.check_loop(node); }
    fn on_until(&mut self, node: NodeRef<'_>) { self.check_loop(node); }
    fn check_loop(&mut self, node: NodeRef<'_>) {
        if node.modifier_form() { return; }
        let (Some(body), Some(condition), Some(keyword), Some(keyword_range)) = (node.body(), node.condition(), node.keyword_name(), self.location_range(node, "keyword")) else { return; };
        if !body.single_line() || !condition.single_line() || matches!(body.kind(), "if" | "while" | "until" | "case" | "case_match")
            || condition.each_node(&[]).into_iter().any(|child| matches!(child.kind(), "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn" | "casgn")) { return; }
        let body_source = body.source().unwrap_or_default();
        let condition_source = condition.source().unwrap_or_default();
        let mut replacement = format!("{body_source} {keyword} {condition_source}");
        let max = self.related_config_value("Layout/LineLength", "Max").and_then(|value| value.parse::<usize>().ok());
        if max.is_some_and(|max| node.column() + replacement.chars().count() > max) { return; }
        if let Some(condition_range) = condition.source_range() {
            let suffix = self.source().chars().skip(condition_range.end).take_while(|character| *character != '\n').collect::<String>();
            if let Some(comment) = suffix.find('#') {
                if node.parent().is_some_and(|parent| parent.kind() == "array") { return; }
                replacement.push(' '); replacement.push_str(suffix[comment..].trim_end());
            }
        }
        if node.parent().is_some_and(|parent| parent.kind() != "begin") { replacement = format!("({replacement})"); }
        add_offense!(self, keyword_range, message: format!("Favor modifier `{keyword}` usage when having a single-line body."), |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

fn whole_line_range(context: &CompatibilityCopContext<'_, '_, '_>, start: usize, end: usize) -> CompatibilitySourceRange {
    let chars = context.source().chars().collect::<Vec<_>>();
    let line_start = chars[..start.min(chars.len())].iter().rposition(|character| *character == '\n').map_or(0, |position| position + 1);
    let line_end = chars[end.min(chars.len())..].iter().position(|character| *character == '\n').map_or(chars.len(), |position| end + position + 1);
    context.range_between(line_start, line_end)
}

define_compatibility_rule!(EmptyLinesAroundBeginBodyRule);
impl EmptyLinesAroundBeginBodyRule<'_, '_, '_, '_> {
    fn on_kwbegin(&mut self, node: NodeRef<'_>) {
        if node.single_line() {
            return;
        }
        // EmptyLinesAroundBody indexes the physical lines adjacent to the
        // node boundaries. It deliberately does not inspect the first/last
        // AST child: rescue and ensure clauses are still inside the begin
        // body and must not move either boundary.
        self.check_empty_line(
            node.first_line(),
            "Extra empty line detected at `begin` body beginning.",
        );
        self.check_empty_line(
            node.last_line().saturating_sub(2),
            "Extra empty line detected at `begin` body end.",
        );
    }
    fn check_empty_line(&mut self, zero_based_line: usize, message: &str) {
        if self.processed_source().line(zero_based_line) != Some("") {
            return;
        }
        let start = self.source_buffer().line_start(zero_based_line + 1);
        let offense = whole_line_range(self, start, start);
        add_offense!(self, offense, message: message, |corrector| {
            corrector.remove(offense);
        });
    }
}

define_compatibility_rule!(InitialIndentationRule);
impl InitialIndentationRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let mut offset = 0;
        for (index, line) in self.source().split_inclusive('\n').enumerate() {
            let line = if index == 0 { line.strip_prefix('\u{feff}').unwrap_or(line) } else { line };
            let bom = usize::from(index == 0 && self.source().starts_with('\u{feff}'));
            let visible = line.trim_end_matches(['\r', '\n']);
            let trimmed = visible.trim_start_matches([' ', '\t']);
            if trimmed.is_empty() || trimmed.starts_with('#') { offset += line.chars().count() + bom; continue; }
            let indentation = visible.chars().count() - trimmed.chars().count();
            if indentation == 0 { return; }
            let start = offset + bom + indentation;
            let token_length = trimmed.chars().take_while(|character| !character.is_whitespace()).count().max(1);
            let space = self.range_between(offset + bom, start);
            let offense = self.range_between(start, start + token_length);
            add_offense!(self, offense, message: "Indentation of first line in file detected.", |corrector| { corrector.remove(space); });
            return;
        }
    }
}

define_compatibility_rule!(DoubleCopDisableDirectiveRule);
impl DoubleCopDisableDirectiveRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        for comment in self.processed_source().comments() {
            let count = comment.text.match_indices("# rubocop:disable").count() + comment.text.match_indices("# rubocop:todo").count();
            if count <= 1 { continue; }
            let replacement = comment.text.replace(" # rubocop:disable", ",").replace(" # rubocop:todo", ",");
            add_offense!(self, comment, message: "More than one disable comment on one line.", |corrector| { corrector.replace(comment, replacement); });
        }
    }
}

define_compatibility_rule!(RequireRangeParenthesesRule);
impl RequireRangeParenthesesRule<'_, '_, '_, '_> {
    fn on_irange(&mut self, node: NodeRef<'_>) { self.check_range(node); }
    fn on_erange(&mut self, node: NodeRef<'_>) { self.check_range(node); }
    fn check_range(&mut self, node: NodeRef<'_>) {
        if node.parent().is_some_and(|parent| parent.kind() == "begin") { return; }
        let (Some(begin), Some(end), Some(operator)) = (node.begin(), node.end(), self.location_range(node, "operator")) else { return; };
        let Some(end_range) = end.source_range() else { return; };
        if !self.range_source(&self.range_between(operator.end_pos(), end_range.start)).contains('\n') { return; }
        let range = format!("{}{}", begin.source().unwrap_or_default(), self.range_source(&operator));
        self.report(format!("Wrap the endless range literal `{range}` to avoid precedence ambiguity."), node);
    }
}

define_compatibility_rule!(PercentQLiteralsRule);
impl PercentQLiteralsRule<'_, '_, '_, '_> {
    fn on_str(&mut self, node: NodeRef<'_>) {
        let Some(begin) = self.location_range(node, "begin") else { return; };
        let kind = self.range_source(&begin);
        if !matches!(kind.get(..2), Some("%q" | "%Q")) { return; }
        let style = self.policy().enforced_style("lower_case_q");
        let wrong = style == "lower_case_q" && kind.starts_with("%Q") || style == "upper_case_q" && kind.starts_with("%q");
        if !wrong { return; }
        let source = node.source().unwrap_or_default();
        let all_backslash_runs_even = backslash_runs_even(source);
        if style == "lower_case_q" && (source.contains("#{") || !all_backslash_runs_even)
            || style == "upper_case_q" && (source.contains("#{") || source.contains('\\')) { return; }
        let mut replacement = source.to_owned();
        replacement.replace_range(1..2, if kind.starts_with("%Q") { "q" } else { "Q" });
        let message = if style == "lower_case_q" { "Do not use `%Q` unless interpolation is needed. Use `%q`." } else { "Use `%Q` instead of `%q`." };
        add_offense!(self, begin, message: message, |corrector| { corrector.replace(node, replacement); });
    }
}

fn backslash_runs_even(source: &str) -> bool {
    let mut run = 0;
    for character in source.chars() {
        if character == '\\' { run += 1; }
        else if run % 2 != 0 { return false; }
        else { run = 0; }
    }
    run % 2 == 0
}

define_compatibility_rule!(RedundantFreezeRule);
impl RedundantFreezeRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(mut receiver) = node.receiver() else { return; };
        let parenthesized = receiver.kind() == "begin" && receiver.child_nodes().len() == 1;
        if parenthesized { receiver = receiver.child_nodes()[0]; }
        let frozen_by_default = self.related_config_value("AllCops", "StringLiteralsFrozenByDefault").and_then(|value| match value { "true" => Some(true), "false" => Some(false), _ => None });
        let frozen_string = FrozenStringLiteral::new(self.processed_source(), self.target_ruby_version().as_f64(), frozen_by_default)
            .frozen_string_literal(receiver);
        let call = if matches!(receiver.kind(), "block" | "numblock" | "itblock") { receiver.send_node() } else { Some(receiver) };
        let immutable_operation = call.filter(|call| call.call_type()).is_some_and(|call| {
            let method = call.method_name().unwrap_or_default();
            if matches!(method, "count" | "length" | "size") { return true; }
            if !parenthesized { return false; }
            if matches!(method, "==" | "===" | "!=" | "<=" | ">=" | "<" | ">") { return true; }
            if !matches!(method, "+" | "-" | "*" | "**" | "/" | "%" | "<<") { return false; }
            let left = call.receiver();
            let right = call.first_argument();
            left.is_some_and(|value| matches!(value.kind(), "float" | "int"))
                || right.is_some_and(|value| matches!(value.kind(), "float" | "int"))
                    && !left.is_some_and(|value| matches!(value.kind(), "str" | "array"))
        });
        let immutable = matches!(receiver.kind(), "int" | "float" | "rational" | "complex" | "sym" | "true" | "false" | "nil")
            || self.target_ruby_version().at_least(3, 0) && matches!(receiver.kind(), "regexp" | "irange" | "erange")
            || frozen_string || immutable_operation;
        if !immutable { return; }
        let (Some(dot), Some(selector)) = (self.location_range(node, "dot"), self.location_range(node, "selector")) else { return; };
        add_offense!(self, node, message: "Do not freeze immutable objects, as freezing them has no effect.", |corrector| {
            corrector.remove(dot); corrector.remove(selector);
        });
    }
}

define_compatibility_rule!(MultilineBlockChainRule);
impl MultilineBlockChainRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) { self.check_block(node); }
    fn on_numblock(&mut self, node: NodeRef<'_>) { self.check_block(node); }
    fn on_itblock(&mut self, node: NodeRef<'_>) { self.check_block(node); }
    fn check_block(&mut self, node: NodeRef<'_>) {
        let Some(send) = node.send_node() else { return; };
        for candidate in send.each_node(&["send", "csend"]) {
            let Some(receiver) = candidate.receiver().filter(|receiver| matches!(receiver.kind(), "block" | "numblock" | "itblock") && receiver.multiline()) else { continue; };
            let (Some(block_end), Some(send_range)) = (self.location_range(receiver, "end"), send.source_range()) else { return; };
            let offense = self.range_between(block_end.begin_pos(), send_range.end);
            self.report("Avoid multi-line chains of blocks.", offense);
            break;
        }
    }
}

define_compatibility_rule!(ClassMethodsRule);
impl ClassMethodsRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) { self.check_container(node); }
    fn on_module(&mut self, node: NodeRef<'_>) { self.check_container(node); }
    fn check_container(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.node_child(0) else { return; };
        let Some(body) = node.body() else { return; };
        for definition in body.each_node(&["defs"]) {
            let Some(receiver) = definition.receiver().filter(|receiver| receiver.structurally_equal(name)) else { continue; };
            let method = definition.method_name().unwrap_or_default();
            let class = name.source().unwrap_or_default();
            add_offense!(self, receiver, message: format!("Use `self.{method}` instead of `{class}.{method}`."), |corrector| { corrector.replace(receiver, "self"); });
        }
    }
}

define_compatibility_rule!(MethodCalledOnDoEndBlockRule);
impl MethodCalledOnDoEndBlockRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.block_literal() { return; }
        let Some(receiver) = node.receiver().filter(|receiver| matches!(receiver.kind(), "block" | "numblock" | "itblock") && receiver.loc_is("begin", "do")) else { return; };
        let (Some(block_end), Some(expression_end)) = (self.location_range(receiver, "end"), node.source_range().map(|range| range.end)) else { return; };
        let offense = self.range_between(block_end.begin_pos(), expression_end);
        self.report("Avoid chaining a method call on a do...end block.", offense);
    }
}

define_compatibility_rule!(EmptyHeredocRule);
impl EmptyHeredocRule<'_, '_, '_, '_> {
    fn on_str(&mut self, node: NodeRef<'_>) { self.check_heredoc(node); }
    fn on_dstr(&mut self, node: NodeRef<'_>) { self.check_heredoc(node); }
    fn check_heredoc(&mut self, node: NodeRef<'_>) {
        let (Some(body), Some(end)) = (self.location_range(node, "heredoc_body"), self.location_range(node, "heredoc_end")) else { return; };
        if !self.range_source(&body).is_empty() { return; }
        let replacement = if self.related_config_value("Style/StringLiterals", "EnforcedStyle") == Some("double_quotes") { "\"\"" } else { "''" };
        let end_line = whole_line_range(self.context, end.begin_pos(), end.end_pos());
        add_offense!(self, node, message: "Use an empty string literal instead of heredoc.", |corrector| {
            corrector.replace(node, replacement); corrector.remove(end_line);
        });
    }
}

define_compatibility_rule!(ConditionPositionRule);
impl ConditionPositionRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) { if !node.ternary() { self.check_condition(node); } }
    fn on_while(&mut self, node: NodeRef<'_>) { self.check_condition(node); }
    fn on_until(&mut self, node: NodeRef<'_>) { self.check_condition(node); }
    fn check_condition(&mut self, node: NodeRef<'_>) {
        if node.modifier_form() { return; }
        let (Some(condition), Some(keyword)) = (node.condition(), self.location_range(node, "keyword")) else { return; };
        if condition.first_line() == node.first_line() { return; }
        let Some(condition_range) = condition.source_range() else { return; };
        let removal = whole_line_range(self.context, condition_range.start, condition_range.end);
        let condition_source = condition.source().unwrap_or_default().to_owned();
        let keyword_name = node.keyword_name().unwrap_or_default();
        add_offense!(self, condition, message: format!("Place the condition on the same line as `{keyword_name}`."), |corrector| {
            corrector.insert_after(keyword, format!(" {condition_source}")); corrector.remove(removal);
        });
    }
}

define_compatibility_rule!(NegatedWhileRule);
impl NegatedWhileRule<'_, '_, '_, '_> {
    fn on_while(&mut self, node: NodeRef<'_>) { self.check_negative_conditional(node); }
    fn on_until(&mut self, node: NodeRef<'_>) { self.check_negative_conditional(node); }

    fn check_negative_conditional(&mut self, node: NodeRef<'_>) {
        if node.post_condition_loop() { return; }
        let Some(condition) = node.condition() else { return; };
        let Some(negative) = single_negative(condition) else { return; };
        let Some(receiver) = negative.receiver() else { return; };
        let Some(keyword) = self.location_range(node, "keyword") else { return; };
        let Some(selector) = self.location_range(negative, "selector") else { return; };
        let current = self.range_source(&keyword);
        let inverse = if current == "while" { "until" } else { "while" };
        let message = format!("Favor `{inverse}` over `{current}` for negative conditions.");
        let removal = if self.range_source(&selector) == "not" {
            let receiver_start = receiver.source_range().map_or(selector.end_pos(), |range| range.start);
            self.range_between(selector.begin_pos(), receiver_start)
        } else {
            selector
        };
        add_offense!(self, node, message: message, |corrector| {
            corrector.replace(keyword, inverse);
            corrector.remove(removal);
        });
    }
}

fn single_negative(node: NodeRef<'_>) -> Option<NodeRef<'_>> {
    if node.kind() == "begin" {
        return node.child_nodes().last().copied().and_then(single_negative);
    }
    (node.call_type()
        && node.method_name() == Some("!")
        && node.receiver().is_some_and(|receiver| receiver.method_name() != Some("!")))
        .then_some(node)
}

define_compatibility_rule!(EmptyLambdaParameterRule);
impl EmptyLambdaParameterRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        let Some(send_node) = node.send_node().filter(|send| send.lambda_literal()) else { return; };
        let Some(arguments) = node.node_child(1).filter(|args| {
            args.kind() == "args" && args.children().is_empty() && args.source_range().is_some()
        }) else { return; };
        let (Some(arguments_range), Some(selector)) = (
            arguments.source_range(),
            self.location_range(send_node, "selector"),
        ) else { return; };
        let offense = self.owned_character_range(arguments_range.clone());
        let removal = self.range_between(selector.end_pos(), arguments_range.end);
        add_offense!(self, offense, message: "Omit parentheses for the empty lambda parameters.", |corrector| {
            corrector.remove(removal);
        });
    }
}

define_compatibility_rule!(MultilineIfThenRule);
impl MultilineIfThenRule<'_, '_, '_, '_> {
    fn on_normal_if_unless(&mut self, node: NodeRef<'_>) {
        let Some(begin) = self.location_range(node, "begin") else { return; };
        let begin_source_range = self.range_help().range_between(begin.begin_pos(), begin.end_pos());
        if self.range_source(&begin) != "then"
            || node.if_branch().is_some_and(|branch| branch.first_line() == begin_source_range.line())
        {
            return;
        }
        let keyword = node.keyword_name().unwrap_or("if");
        let removal = self.range_help().range_with_surrounding_space(
            begin_source_range,
            SurroundingSpace { side: Side::Left, whitespace: true, ..SurroundingSpace::default() },
        );
        let removal = self.owned_range(removal);
        add_offense!(self, begin, message: format!("Do not use `then` for multi-line `{keyword}`."), |corrector| {
            corrector.remove(removal);
        });
    }
}

define_compatibility_rule!(VariableInterpolationRule);
impl VariableInterpolationRule<'_, '_, '_, '_> {
    fn on_interpolation(&mut self, node: NodeRef<'_>) {
        // Explicit `#{...}` interpolation uses the same Parser `begin` shape;
        // this cop only handles the implicit `#@var`/`#$var` form.
        if node.source().is_some_and(|source| source.starts_with("#{")) { return; }
        for variable in node.child_nodes().into_iter().filter(|child| child.variable() || child.reference()) {
            let source = variable.source().unwrap_or_default();
            add_offense!(self, variable, message: format!("Replace interpolated variable `{source}` with expression `#{{{source}}}`."), |corrector| {
                corrector.replace(variable, format!("{{{source}}}"));
            });
        }
    }
}

define_compatibility_rule!(ClassAndModuleCamelCaseRule);
impl ClassAndModuleCamelCaseRule<'_, '_, '_, '_> {
    fn on_class(&mut self, node: NodeRef<'_>) { self.check_name(node); }
    fn on_module(&mut self, node: NodeRef<'_>) { self.check_name(node); }

    fn check_name(&mut self, node: NodeRef<'_>) {
        let Some(name) = node.first_node() else { return; };
        let source = name.source().unwrap_or_default();
        if !source.contains('_') { return; }
        let remaining = self.config_values("AllowedNames").iter()
            .fold(source.to_owned(), |value, allowed| value.replace(allowed, ""));
        if remaining.contains('_') {
            self.report("Use CamelCase for classes and modules.", name);
        }
    }
}

define_compatibility_rule!(StripRule);
impl StripRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let outer = node.method_name().unwrap_or_default();
        let Some(inner) = node.receiver().filter(|receiver| receiver.call_type()) else { return; };
        if !node.arguments().is_empty() || !inner.arguments().is_empty()
            || !matches!((inner.method_name(), outer), (Some("rstrip"), "lstrip") | (Some("lstrip"), "rstrip"))
        {
            return;
        }
        let (Some(first_selector), Some(expression)) = (self.location_range(inner, "selector"), self.source_range(node)) else { return; };
        let range = self.range_between(first_selector.begin_pos(), expression.end_pos());
        let methods = self.range_source(&range).to_owned();
        add_offense!(self, range, message: format!("Use `strip` instead of `{methods}`."), |corrector| {
            corrector.replace(range, "strip");
        });
    }
}

define_compatibility_rule!(RedundantCapitalWRule);
impl RedundantCapitalWRule<'_, '_, '_, '_> {
    fn on_array(&mut self, node: NodeRef<'_>) {
        let Some(begin) = self.location_range(node, "begin") else { return; };
        if !self.range_source(&begin).starts_with("%W") { return; }
        let requires_interpolation = node.source().is_some_and(|source| source.contains("#{") || source.contains('\\'));
        if requires_interpolation { return; }
        let replacement = self.range_source(&begin).replacen('W', "w", 1);
        add_offense!(self, node, message: "Do not use `%W` unless interpolation is needed. If not, use `%w`.", |corrector| {
            corrector.replace(begin, replacement);
        });
    }
}

define_compatibility_rule!(ArrayIntersectWithSingleElementRule);
impl ArrayIntersectWithSingleElementRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.kind() != "send" { return; }
        let Some(array) = node.arguments().first().copied().filter(|array| {
            node.arguments().len() == 1 && array.kind() == "array" && array.child_nodes().len() == 1
        }) else { return; };
        let element = array.child_nodes()[0];
        if element.kind() == "splat" { return; }
        let (Some(selector), Some(expression)) = (self.location_range(node, "selector"), self.source_range(node)) else { return; };
        let offense = self.range_between(selector.begin_pos(), expression.end_pos());
        let replacement = if array.percent_literal(None) {
            match element.kind() {
                "sym" => format!(":{}", element.scalar_value_text().unwrap_or_default()),
                "str" => format!("{:?}", element.scalar_value_text().unwrap_or_default()),
                _ => element.source().unwrap_or_default().to_owned(),
            }
        } else {
            element.source().unwrap_or_default().to_owned()
        };
        add_offense!(self, offense, message: "Use `include?(element)` instead of `intersect?([element])`.", |corrector| {
            corrector.replace(selector, "include?");
            corrector.replace(array, replacement);
        });
    }
}

define_compatibility_rule!(EmptyBlockParameterRule);
impl EmptyBlockParameterRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        let Some(send_node) = node.send_node() else { return; };
        if send_node.send_type() && send_node.lambda_literal() { return; }
        let Some(arguments) = node.node_child(1).filter(|args| {
            args.kind() == "args" && args.children().is_empty() && args.source_range().is_some()
        }) else { return; };
        let (Some(arguments_range), Some(block_begin)) = (arguments.source_range(), self.location_range(node, "begin")) else { return; };
        let offense = self.owned_character_range(arguments_range.clone());
        let removal = self.range_between(block_begin.end_pos(), arguments_range.end);
        add_offense!(self, offense, message: "Omit pipes for the empty block parameters.", |corrector| {
            corrector.remove(removal);
        });
    }
}

define_compatibility_rule!(MultipleComparisonRule);
impl MultipleComparisonRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(inner) = node.receiver().filter(|receiver| {
            receiver.call_type() && ["<", ">", "<=", ">="].contains(&receiver.method_name().unwrap_or_default())
        }) else { return; };
        let Some(center) = inner.first_argument() else { return; };
        if center.call_type() && ["&", "|", "^"].contains(&center.method_name().unwrap_or_default()) { return; }
        let center_source = center.source().unwrap_or_default();
        add_offense!(self, node, message: "Use the `&&` operator to compare multiple values.", |corrector| {
            corrector.replace(center, format!("{center_source} && {center_source}"));
        });
    }
}

define_compatibility_rule!(ReverseFindRule);
impl ReverseFindRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(4, 0) { return; }
        let Some(reverse) = node.receiver().filter(|receiver| {
            receiver.call_type()
                && ["reverse", "reverse_each"].contains(&receiver.method_name().unwrap_or_default())
                && receiver.arguments().is_empty()
                && receiver.receiver().is_some()
        }) else { return; };
        let (Some(reverse_selector), Some(find_selector)) = (self.location_range(reverse, "selector"), self.location_range(node, "selector")) else { return; };
        let range = self.range_between(reverse_selector.begin_pos(), find_selector.end_pos());
        add_offense!(self, range, message: "Use `rfind` instead.", |corrector| {
            corrector.replace(range, "rfind");
        });
    }
}

define_compatibility_rule!(ClassCheckRule);
impl ClassCheckRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let current = node.method_name().unwrap_or_default();
        if self.policy().enforced_style("is_a?") == current { return; }
        let preferred = if current == "is_a?" { "kind_of?" } else { "is_a?" };
        let Some(selector) = self.location_range(node, "selector") else { return; };
        add_offense!(self, selector, message: format!("Prefer `Object#{preferred}` over `Object#{current}`."), |corrector| {
            corrector.replace(selector, preferred);
        });
    }
}

define_compatibility_rule!(DirEmptyRule);
impl DirEmptyRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(2, 4) { return; }
        let Some((enumeration, argument, negative)) = dir_empty_match(node) else { return; };
        let Some(dir) = enumeration.receiver().filter(|receiver| compatibility_root_constant(*receiver, "Dir")) else { return; };
        let replacement = format!("{}{}.empty?({})", if negative { "!" } else { "" }, dir.source().unwrap_or_default(), argument.source().unwrap_or_default());
        add_offense!(self, node, message: format!("Use `{replacement}` instead."), |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

fn dir_empty_match(node: NodeRef<'_>) -> Option<(NodeRef<'_>, NodeRef<'_>, bool)> {
    match node.method_name() {
        Some("empty?" | "none?") if node.arguments().is_empty() => {
            let enumeration = node.receiver().filter(|receiver| receiver.call_type())?;
            let expected = if node.method_name() == Some("empty?") { "children" } else { "each_child" };
            (enumeration.method_name() == Some(expected))
                .then(|| enumeration.first_argument().map(|argument| (enumeration, argument, false)))
                .flatten()
        }
        Some("==" | "!=" | ">") if node.arguments().len() == 1 => {
            let size = node.receiver().filter(|receiver| receiver.call_type() && receiver.method_name() == Some("size") && receiver.arguments().is_empty())?;
            let enumeration = size.receiver().filter(|receiver| receiver.call_type())?;
            let expected = node.first_argument()?.scalar_value_text()?.parse::<i64>().ok()?;
            let eligible = enumeration.method_name() == Some("entries") && expected == 2
                || enumeration.method_name() == Some("children") && expected == 0;
            eligible.then(|| enumeration.first_argument().map(|argument| {
                (enumeration, argument, matches!(node.method_name(), Some("!=" | ">")))
            })).flatten()
        }
        _ => None,
    }
}

define_compatibility_rule!(NestedFileDirnameRule);
impl NestedFileDirnameRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(3, 1) || !file_dirname(node)
            || node.parent().is_some_and(file_dirname)
        { return; }
        let Some(mut path) = node.first_argument() else { return; };
        let mut level = 1;
        while file_dirname(path) {
            level += 1;
            let Some(next) = path.first_argument() else { break; };
            path = next;
        }
        if level < 2 { return; }
        let Some(selector) = self.location_range(node, "selector") else { return; };
        let Some(expression) = node.source_range() else { return; };
        let range = self.range_between(selector.begin_pos(), expression.end);
        let path_source = path.source().unwrap_or_default();
        let replacement = format!("dirname({path_source}, {level})");
        add_offense!(self, range, message: format!("Use `{replacement}` instead."), |corrector| {
            corrector.replace(range, replacement);
        });
    }
}

fn file_dirname(node: NodeRef<'_>) -> bool {
    node.call_type() && node.method_name() == Some("dirname") && node.arguments().len() == 1
        && node.receiver().is_some_and(|receiver| compatibility_root_constant(receiver, "File"))
}

fn compatibility_root_constant(node: NodeRef<'_>, name: &str) -> bool {
    node.kind() == "const" && node.short_name() == Some(name)
        && node.namespace().is_none_or(|namespace| namespace.kind() == "cbase")
}

define_compatibility_rule!(RedundantStringCoercionRule);
impl RedundantStringCoercionRule<'_, '_, '_, '_> {
    fn on_interpolation(&mut self, begin_node: NodeRef<'_>) {
        let Some(final_node) = begin_node.child_nodes().last().copied() else { return; };
        if final_node.call_type() && final_node.method_name() == Some("to_s") && final_node.arguments().is_empty() {
            self.register_offense(final_node, "interpolation");
        }
    }
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.receiver().is_some() || !["print", "puts", "warn"].contains(&node.method_name().unwrap_or_default()) { return; }
        let context = format!("`{}`", node.method_name().unwrap_or_default());
        for child in node.child_nodes().into_iter().filter(|child| child.call_type() && child.method_name() == Some("to_s") && child.arguments().is_empty()) {
            self.register_offense(child, &context);
        }
    }
    fn register_offense(&mut self, node: NodeRef<'_>, context: &str) {
        let Some(selector) = self.location_range(node, "selector") else { return; };
        let (message, replacement) = if let Some(receiver) = node.receiver() {
            (format!("Redundant use of `Object#to_s` in {context}."), receiver.source().unwrap_or_default().to_owned())
        } else {
            (format!("Use `self` instead of `Object#to_s` in {context}."), "self".to_owned())
        };
        add_offense!(self, selector, message: message, |corrector| { corrector.replace(node, replacement); });
    }
}

define_compatibility_rule!(DefWithParenthesesRule);
impl DefWithParenthesesRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn on_defs(&mut self, node: NodeRef<'_>) { self.check(node); }
    fn check(&mut self, node: NodeRef<'_>) {
        let Some(arguments) = node.arguments_node().filter(|arguments| arguments.argument_list().is_empty()) else { return; };
        let Some(range) = arguments.source_range().filter(|range| range.end > range.start) else { return; };
        if node.single_line() && !node.endless() || self.range_source(&self.range_between(range.end, range.end.saturating_add(1))) == "=" { return; }
        let range = self.owned_character_range(range);
        add_offense!(self, range, message: "Omit the parentheses in defs when the method doesn't accept any arguments.", |corrector| { corrector.remove(range); });
    }
}

define_compatibility_rule!(NilLambdaRule);
impl NilLambdaRule<'_, '_, '_, '_> {
    fn on_block(&mut self, node: NodeRef<'_>) {
        if !node.lambda_or_proc() { return; }
        let Some(body) = node.body().filter(|body| nil_return(*body)) else { return; };
        let callable = if node.lambda() { "lambda" } else { "proc" };
        let Some(body_range) = self.source_range(body) else { return; };
        let removal = if node.single_line() {
            self.range_help().range_with_surrounding_space(body_range, SurroundingSpace { whitespace: true, ..SurroundingSpace::default() })
        } else {
            self.range_help().range_by_whole_lines(body_range, true)
        };
        let removal = self.owned_range(removal);
        add_offense!(self, node, message: format!("Use an empty {callable} instead of always returning nil."), |corrector| { corrector.remove(removal); });
    }
}

fn nil_return(node: NodeRef<'_>) -> bool {
    node.kind() == "nil" || matches!(node.kind(), "return" | "next" | "break")
        && node.arguments().len() == 1 && node.first_argument().is_some_and(|argument| argument.kind() == "nil")
}

define_compatibility_rule!(PercentSymbolArrayRule);
impl PercentSymbolArrayRule<'_, '_, '_, '_> {
    fn on_array(&mut self, node: NodeRef<'_>) {
        let Some(begin) = self.location_range(node, "begin") else { return; };
        if !matches!(self.range_source(&begin), source if source.starts_with("%i") || source.starts_with("%I")) { return; }
        let Some(expression) = self.source_range(node) else { return; };
        let source = expression.source();
        let opening_len = self.range_source(&begin).len();
        if source.len() <= opening_len + 1 { return; }
        let content = &source[opening_len..source.len() - 1];
        let unwanted = content.char_indices().any(|(index, character)| {
            let previous = index.checked_sub(1).and_then(|at| content.as_bytes().get(at));
            character == ':' && (index == 0 || previous.is_some_and(u8::is_ascii_whitespace))
                || character == ',' && previous != Some(&b'$')
        });
        if !unwanted { return; }
        let clean = content.char_indices().filter_map(|(index, character)| {
            let previous = index.checked_sub(1).and_then(|at| content.as_bytes().get(at));
            (!(character == ':' && (index == 0 || previous.is_some_and(u8::is_ascii_whitespace))
                || character == ',' && previous != Some(&b'$'))).then_some(character)
        }).collect::<String>();
        let replacement = format!("{}{}{}", self.range_source(&begin), clean, &source[source.len() - 1..]);
        add_offense!(self, node, message: "Within `%i`/`%I`, ':' and ',' are unnecessary and may be unwanted in the resulting symbols.", |corrector| { corrector.replace(node, replacement); });
    }
}

define_compatibility_rule!(SingleArgumentDigRule);
impl SingleArgumentDigRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        if node.kind() != "send" || node.receiver().is_none() || node.arguments().len() != 1 { return; }
        let argument = node.first_argument().unwrap();
        if matches!(argument.kind(), "splat" | "block_pass" | "forwarded_restarg" | "forwarded_args" | "hash") { return; }
        let receiver = node.receiver().unwrap();
        let dig_chain_enabled = self.related_config_value("Style/DigChain", "Enabled") == Some("true");
        if dig_chain_enabled && (receiver.method_name() == Some("dig") || node.parent().is_some_and(|parent| parent.method_name() == Some("dig"))) { return; }
        let receiver_source = receiver.source().unwrap_or_default();
        let argument_source = argument.source().unwrap_or_default();
        let original = node.source().unwrap_or_default();
        let replacement = format!("{receiver_source}[{argument_source}]");
        let message = format!("Use `{replacement}` instead of `{original}`.");
        if node.ancestors().into_iter().any(|ancestor| ancestor.method_name() == Some("dig")) {
            self.report(message, node);
        } else {
            add_offense!(self, node, message: message, |corrector| { corrector.replace(node, replacement); });
        }
    }
}
