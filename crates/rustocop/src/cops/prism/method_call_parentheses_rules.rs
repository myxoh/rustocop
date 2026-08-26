use regex::Regex;
use ruby_prism::{CallNode, Node, YieldNode};

use super::*;

define_cops! {
    MethodCallWithArgsParentheses => "Style/MethodCallWithArgsParentheses" => rubocop_callbacks(
        MethodCallWithArgsParenthesesRule,
        [on_send, on_yield]
    ),
}

impl MethodCallWithArgsParenthesesRule<'_, '_, '_> {
    fn on_send(&mut self, node: &CallNode<'_>) {
        if self.policy().enforced_style("require_parentheses") == "omit_parentheses" {
            self.omit_parentheses(node);
        } else {
            self.require_parentheses(node);
        }
    }

    fn on_yield(&mut self, node: &YieldNode<'_>) {
        let style = self.policy().enforced_style("require_parentheses");
        if style == "omit_parentheses" {
            let (Some(left), Some(right)) = (node.lparen_loc(), node.rparen_loc()) else { return };
            return_if!(self.config_bool("AllowParenthesesInMultilineCall", false)
                && self.source_file().at(&node.location()).contains('\n'));
            self.add_omit_offense(left, right, node.location());
        } else {
            let Some(arguments) = node.arguments() else { return };
            return_if!(node.lparen_loc().is_some());
            return_if!(ignored_macro_name(self, "yield", node.location()));
            let args = arguments.location();
            let keyword = node.keyword_loc();
            let offense = node.location();
            add_offense!(self, offense, message: "Use parentheses for method calls with arguments.", |corrector| {
                corrector.replace(keyword.end_offset()..args.start_offset(), "(");
                corrector.replace(args.end_offset()..args.end_offset(), ")");
            });
        }
    }

    fn require_parentheses(&mut self, node: &CallNode<'_>) {
        let method = String::from_utf8_lossy(node.name().as_slice()).to_string();
        return_if!(allowed_method(self, &method) || operator_or_setter(&method));
        let arguments = node.arguments();
        let block_argument = node.block().and_then(|block| block.as_block_argument_node());
        return_if!(arguments.is_none() && block_argument.is_none());
        return_if!(node.opening_loc().is_some());
        return_if!(ignored_macro(self, node, &method));
        let args = arguments.as_ref().map_or_else(
            || block_argument.as_ref().expect("argument presence checked").location(),
            |arguments| arguments.location(),
        );
        let Some(selector) = node.message_loc() else { return };
        // Parser represents a block as the send's parent, while Prism stores it
        // on the call node. RuboCop's offense therefore ends at the arguments,
        // not at the end of an attached block.
        let call = node.location();
        let argument_end = block_argument
            .as_ref()
            .map_or(args.end_offset(), |block| {
                block.location().end_offset()
            });
        let offense = call.start_offset()..argument_end;
        let only_parenthesized_argument = arguments.as_ref().is_some_and(|arguments| {
            match arguments.arguments().iter().collect::<Vec<_>>().as_slice() {
                [argument] => argument.as_parentheses_node().is_some(),
                _ => false,
            }
        });
        add_offense!(self, offense, message: "Use parentheses for method calls with arguments.", |corrector| {
            if only_parenthesized_argument {
                corrector.remove(selector.end_offset()..args.start_offset());
            } else {
                corrector.replace(selector.end_offset()..args.start_offset(), "(");
                corrector.replace(argument_end..argument_end, ")");
            }
        });
    }

    fn omit_parentheses(&mut self, node: &CallNode<'_>) {
        let (Some(left), Some(right)) = (node.opening_loc(), node.closing_loc()) else { return };
        return_if!(omit_parentheses_is_unsafe(self, node));
        self.add_omit_offense(left, right, node.location());
    }

    fn add_omit_offense(&mut self, left: ruby_prism::Location<'_>, right: ruby_prism::Location<'_>, _call: ruby_prism::Location<'_>) {
        let offense = left.start_offset()..right.end_offset();
        let opening_line_end = self.source_file().line_end(left.start_offset());
        let multiline_at_end = self.source()
            .get(left.end_offset()..opening_line_end)
            .is_some_and(|tail| tail.bytes().all(|byte| matches!(byte, b' ' | b'\t')));
        let replacement = if multiline_at_end { " \\" } else { " " };
        let left_range = if multiline_at_end {
            let mut end = left.end_offset();
            while end < self.source().len() && matches!(self.source().as_bytes()[end], b' ' | b'\t') { end += 1; }
            left.start_offset()..end
        } else {
            left.start_offset()..left.end_offset()
        };
        add_offense!(self, offense, message: "Omit parentheses for method calls with arguments.", |corrector| {
            corrector.replace(left_range, replacement);
            corrector.remove(right);
        });
    }
}

fn allowed_method(context: &CopContext<'_, '_>, method: &str) -> bool {
    context.config_values("AllowedMethods").iter().any(|allowed| allowed == method)
        || context.config_values("AllowedPatterns").iter().any(|pattern| {
            Regex::new(pattern).is_ok_and(|pattern| pattern.is_match(method))
        })
}

fn operator_or_setter(method: &str) -> bool {
    matches!(method, "+" | "-" | "*" | "/" | "%" | "**" | "==" | "!=" | "===" | "=~" | "!~" | "<" | ">" | "<=" | ">=" | "<=>" | "<<" | ">>" | "&" | "|" | "^" | "[]" | "[]=" | "!" | "~" | "+@" | "-@")
        || method.ends_with('=')
}

fn ignored_macro(context: &CopContext<'_, '_>, node: &CallNode<'_>, method: &str) -> bool {
    if !context.config_bool("IgnoreMacros", true)
        || node.receiver().is_some()
        || !macro_context(context, context.ancestors(), node.location())
    {
        return false;
    }
    let included = context.config_values("IncludedMacros").iter().any(|name| name == method)
        || context.config_values("IncludedMacroPatterns").iter().any(|pattern| Regex::new(pattern).is_ok_and(|pattern| pattern.is_match(method)));
    !included
}

fn ignored_macro_name(
    context: &CopContext<'_, '_>,
    method: &str,
    location: ruby_prism::Location<'_>,
) -> bool {
    if !context.config_bool("IgnoreMacros", true)
        || !macro_context(context, context.ancestors(), location)
    {
        return false;
    }
    let included = context.config_values("IncludedMacros").iter().any(|name| name == method)
        || context.config_values("IncludedMacroPatterns").iter().any(|pattern| Regex::new(pattern).is_ok_and(|pattern| pattern.is_match(method)));
    !included
}

fn macro_context(
    context: &CopContext<'_, '_>,
    ancestors: &[Node<'_>],
    location: ruby_prism::Location<'_>,
) -> bool {
    let mut block_owner = false;
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_statements_node().is_some()
            || ancestor.as_arguments_node().is_some()
            || ancestor.as_parentheses_node().is_some()
            || ancestor.as_program_node().is_some()
        {
            continue;
        }
        if ancestor.as_call_node().is_some_and(|call| class_constructor_call(context, &call)) {
            return true;
        }
        if ancestor.as_block_node().is_some() {
            block_owner = true;
            continue;
        }
        if block_owner {
            if let Some(call) = ancestor.as_call_node() {
                if class_constructor_call(context, &call) {
                    return true;
                }
                block_owner = false;
                continue;
            }
        }
        if ancestor.as_def_node().is_some() {
            return false;
        }
        if ancestor.as_begin_node().is_some_and(|begin| {
            begin.rescue_clause().is_some() || begin.ensure_clause().is_some()
        }) {
            return false;
        }
        if ancestor.as_if_node().is_some_and(|conditional| {
            !range_contains(&conditional.predicate().location(), &location)
        })
            || ancestor.as_unless_node().is_some_and(|conditional| {
                !range_contains(&conditional.predicate().location(), &location)
            })
            || ancestor.as_else_node().is_some()
            || ancestor.as_lambda_node().is_some()
            || ancestor.as_begin_node().is_some()
        {
            continue;
        }
        if ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
            || ancestor.as_singleton_class_node().is_some()
        {
            return true;
        }
        return false;
    }
    true
}

fn range_contains(outer: &ruby_prism::Location<'_>, inner: &ruby_prism::Location<'_>) -> bool {
    outer.start_offset() <= inner.start_offset() && inner.end_offset() <= outer.end_offset()
}

fn class_constructor_call(context: &CopContext<'_, '_>, call: &CallNode<'_>) -> bool {
    let method = call.name().as_slice();
    if method != b"new" && method != b"define" {
        return false;
    }
    let Some(receiver) = call.receiver() else {
        return false;
    };
    let receiver = context.source_file().node(&receiver);
    let receiver = receiver.strip_prefix("::").unwrap_or(receiver);
    (method == b"new" && matches!(receiver, "Class" | "Module" | "Struct"))
        || (method == b"define" && receiver == "Data")
}

fn omit_parentheses_is_unsafe(context: &CopContext<'_, '_>, node: &CallNode<'_>) -> bool {
    let method = String::from_utf8_lossy(node.name().as_slice());
    let arguments = node.arguments().map(|arguments| arguments.arguments().iter().collect::<Vec<_>>()).unwrap_or_default();
    if configured_omit_exception(context, node, &method, &arguments) {
        return true;
    }
    let parent = context.parent();
    if parent.as_ref().is_some_and(|parent| {
        parent.as_array_node().is_some()
            || parent.as_assoc_node().is_some()
            || parent.as_range_node().is_some()
            || parent.as_and_node().is_some()
            || parent.as_or_node().is_some()
            || parent.as_when_node().is_some()
            || parent.as_splat_node().is_some()
            || parent.as_assoc_splat_node().is_some()
            || parent.as_block_argument_node().is_some()
            || parent.as_optional_parameter_node().is_some()
            || parent.as_optional_keyword_parameter_node().is_some()
            || parent.as_constant_path_node().is_some()
    }) {
        return true;
    }
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_splat_node().is_some()
            || ancestor.as_assoc_splat_node().is_some()
            || ancestor.as_block_argument_node().is_some()
            || ancestor.as_yield_node().is_some()
            || ancestor.as_super_node().is_some()
            || ancestor.as_and_node().is_some()
            || ancestor.as_or_node().is_some()
            || ancestor.as_if_node().is_some_and(|conditional| conditional.if_keyword_loc().is_none())
    }) {
        return true;
    }
    if context.ancestors().iter().any(|ancestor| ancestor.as_class_node().is_some_and(|class| {
        !context.source_file().node(&class.as_node()).contains('\n')
    })) {
        return true;
    }
    if let Some(parent_call) = parent.and_then(|parent| parent.as_call_node()) {
        let current = node.location();
        let is_receiver = parent_call.receiver().is_some_and(|receiver| receiver.location().start_offset() == current.start_offset() && receiver.location().end_offset() == current.end_offset());
        let is_argument = parent_call.arguments().is_some_and(|arguments| arguments.arguments().iter().any(|argument| argument.location().start_offset() <= current.start_offset() && current.end_offset() <= argument.location().end_offset()));
        let parent_assignment = String::from_utf8_lossy(parent_call.name().as_slice()).ends_with('=');
        if is_receiver || is_argument && !parent_assignment {
            return true;
        }
    }
    if arguments.iter().any(|argument| {
        argument.as_splat_node().is_some()
            || argument.as_assoc_splat_node().is_some()
            || argument.as_block_argument_node().is_some()
            || argument.as_hash_node().is_some()
            || argument.as_regular_expression_node().is_some_and(|regexp| context.source_file().at(&regexp.opening_loc()) == "/")
    }) {
        return true;
    }
    source_shape_requires_parentheses(context, node, &arguments)
}

fn configured_omit_exception(
    context: &CopContext<'_, '_>,
    node: &CallNode<'_>,
    method: &str,
    arguments: &[Node<'_>],
) -> bool {
    if operator_or_setter(method) || node.message_loc().is_none() || method == "call" {
        return true;
    }
    if method.as_bytes().first().is_some_and(u8::is_ascii_uppercase)
        && (arguments.is_empty() || context.config_bool("AllowParenthesesInCamelCaseMethod", false))
    {
        return true;
    }
    if context.config_bool("AllowParenthesesInMultilineCall", false)
        && context.source_file().node(&node.as_node()).contains('\n')
    {
        return true;
    }
    if context.config_bool("AllowParenthesesInStringInterpolation", false)
        && context.ancestors().iter().any(|ancestor| ancestor.as_interpolated_string_node().is_some())
    {
        return true;
    }
    if context.ancestors().iter().any(|ancestor| ancestor.as_def_node().is_some_and(|definition| definition.equal_loc().is_some()))
        && !arguments.is_empty()
    {
        return true;
    }
    if context.config_bool("AllowParenthesesInChaining", false)
        && receiver_chain_has_parentheses_or_block(node)
    {
        return true;
    }
    if node.block().and_then(|block| block.as_block_node()).is_some_and(|block| context.source_file().at(&block.opening_loc()) == "{") {
        return true;
    }
    false
}

fn source_shape_requires_parentheses(
    context: &CopContext<'_, '_>,
    node: &CallNode<'_>,
    _arguments: &[Node<'_>],
) -> bool {
    let call_source = context.source_file().node(&node.as_node());
    let inner = node.opening_loc().zip(node.closing_loc()).and_then(|(left, right)| {
        context.source_file().slice(left.end_offset()..right.start_offset())
    }).unwrap_or_default();
    if inner.trim_start().starts_with('{')
        || inner.contains(", {")
        || inner.trim_start().starts_with(['*', '&'])
        || inner.contains(" || ")
        || inner.contains(" && ")
        || inner.contains(" ? ")
        || inner.trim_start().starts_with(['+', '-'])
        || call_source.contains("...")
        || inner.contains("**")
        || inner.contains("&")
        || node.block().is_none() && (call_source.contains(" { ") || call_source.contains(" do "))
    {
        return true;
    }
    let trimmed_inner = inner.trim();
    if trimmed_inner.starts_with("..") || trimmed_inner.ends_with("..") || trimmed_inner.ends_with("...") {
        return true;
    }
    let line = context.source_file().line(node.location().start_offset());
    let line_start = context.source_file().line_start(node.location().start_offset());
    let relative_end = node.location().end_offset().saturating_sub(line_start);
    let after = line.get(relative_end..).unwrap_or_default();
    if after.contains(" in ") || after.contains(" => ") {
        return true;
    }
    if (after.trim_start().starts_with("if ") || after.trim_start().starts_with("unless "))
        && hash_value_omission(inner)
    {
        return true;
    }
    let relative_start = node.location().start_offset().saturating_sub(line_start);
    if let Some(operator) = assignment_operator_offset(line) {
        if relative_end <= operator {
            return true;
        }
        if context.ancestors().iter().any(|ancestor| {
            ancestor.as_if_node().is_some()
                || ancestor.as_unless_node().is_some()
                || ancestor.as_case_node().is_some()
                || ancestor.as_when_node().is_some()
        }) {
            return true;
        }
    }
    if hash_value_omission(inner) {
        if line.get(..relative_start).unwrap_or_default().contains(" then ") {
            return true;
        }
        if context.ancestors().iter().any(|ancestor| {
            ancestor.as_if_node().is_some()
                || ancestor.as_unless_node().is_some()
                || ancestor.as_case_node().is_some()
        }) || has_following_statement(context.source(), node.location().end_offset()) {
            return true;
        }
    }
    false
}

fn receiver_chain_has_parentheses_or_block(node: &CallNode<'_>) -> bool {
    let mut receiver = node.receiver().and_then(|receiver| receiver.as_call_node());
    while let Some(call) = receiver {
        if call.opening_loc().is_some() || call.block().is_some() {
            return true;
        }
        receiver = call.receiver().and_then(|receiver| receiver.as_call_node());
    }
    false
}

fn assignment_operator_offset(line: &str) -> Option<usize> {
    [" &&= ", " ||= ", " += ", " -= ", " = "]
        .into_iter()
        .filter_map(|operator| line.find(operator).map(|offset| offset + operator.len() / 2))
        .min()
}

fn hash_value_omission(source: &str) -> bool {
    Regex::new(r"\b[a-zA-Z_]\w*:\s*(?:,|$)")
        .expect("static hash omission regex")
        .is_match(source.trim())
}

fn has_following_statement(source: &str, end: usize) -> bool {
    source.get(end..).is_some_and(|tail| {
        tail.lines().skip(1).map(str::trim).find(|line| !line.is_empty())
            .is_some_and(|line| {
                !matches!(line, "end" | "else" | "ensure")
                    && !line.starts_with("in ")
                    && !line.starts_with("when ")
            })
    })
}
