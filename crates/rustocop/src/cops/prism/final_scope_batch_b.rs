use super::catalog_cop::{custom, report};
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Naming/MemoizedInstanceVariableName", memoized_variable),
        custom("Naming/FileName", file_name),
        report(
            "Lint/AssignmentInCondition",
            "if value = ",
            "Assignment in condition detected.",
        ),
        custom("Naming/VariableNumber", variable_number),
        custom("Naming/VariableName", variable_name),
        custom("Lint/UselessAssignment", useless_assignment),
        Box::new(SelfAssignment),
        custom("Naming/MethodName", method_name),
        Box::new(PredicateMethod),
    ]
}

struct PredicateMethod;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PredicateReturn {
    Boolean,
    NonBoolean,
    Unknown,
    Super,
}

impl Cop for PredicateMethod {
    fn name(&self) -> &'static str {
        "Naming/PredicateMethod"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(definition) = node.as_def_node() else {
            return;
        };
        let name = definition.name().as_slice();
        let Some(body) = definition.body() else {
            return;
        };
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if name == b"initialize"
            || cop_context.policy().allows_method(name)
            || cop_context
                .config_values("AllowedPatterns")
                .iter()
                .any(|pattern| {
                    let pattern = pattern.strip_prefix("\\A").unwrap_or(pattern);
                    std::str::from_utf8(name).is_ok_and(|name| {
                        name.starts_with(pattern) || pattern.contains(name.trim_end_matches('?'))
                    })
                })
            || predicate_operator_method(name)
            || name.ends_with(b"!") && cop_context.config_bool("AllowBangMethods", false)
        {
            return;
        }

        let wayward = cop_context.config_values("WaywardPredicates");
        let mut collector = PredicateReturnCollector::default();
        ruby_prism::Visit::visit(&mut collector, &body);
        let mut values = Vec::new();
        for value in &collector.returns {
            if let Some(value) = value {
                collect_implicit_predicate_returns(value, &wayward, &mut values);
            } else {
                values.push(PredicateReturn::NonBoolean);
            }
        }
        collect_implicit_predicate_returns(&body, &wayward, &mut values);

        let conservative =
            cop_context.config_value("Mode").unwrap_or("conservative") == "conservative";
        if conservative
            && values
                .iter()
                .any(|value| matches!(value, PredicateReturn::Unknown | PredicateReturn::Super))
        {
            return;
        }
        let predicate_name = name.ends_with(b"?");
        let offense = if predicate_name {
            let has_non_boolean = values.contains(&PredicateReturn::NonBoolean);
            let has_boolean = values.contains(&PredicateReturn::Boolean);
            has_non_boolean && (!conservative || !has_boolean)
        } else {
            let known = values
                .iter()
                .filter(|value| **value != PredicateReturn::Super)
                .collect::<Vec<_>>();
            !known.is_empty()
                && known
                    .into_iter()
                    .all(|value| *value == PredicateReturn::Boolean)
        };
        if !offense {
            return;
        }
        let message = if predicate_name {
            "Non-predicate method names should not end with `?`."
        } else {
            "Predicate method names should end with `?`."
        };
        cop_context.report(message, definition.name_loc());
    }
}

#[derive(Default)]
struct PredicateReturnCollector<'pr> {
    returns: Vec<Option<Node<'pr>>>,
}

impl<'pr> ruby_prism::Visit<'pr> for PredicateReturnCollector<'pr> {
    fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode<'pr>) {
        let arguments = node.arguments();
        let values = arguments
            .as_ref()
            .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
            .unwrap_or_default();
        self.returns
            .push((values.len() == 1).then(|| values.into_iter().next().expect("one value")));
        ruby_prism::visit_return_node(self, node);
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
}

fn collect_implicit_predicate_returns<'pr>(
    node: &Node<'pr>,
    wayward: &[String],
    values: &mut Vec<PredicateReturn>,
) {
    if node.as_return_node().is_some() {
        return;
    }
    if let Some(statements) = node.as_statements_node() {
        if let Some(last) = statements.body().last() {
            collect_implicit_predicate_returns(&last, wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        if let Some(body) = parentheses.body() {
            collect_implicit_predicate_returns(&body, wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    if let Some(conditional) = node.as_if_node() {
        if let Some(statements) = conditional.statements() {
            collect_implicit_predicate_returns(&statements.as_node(), wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        if let Some(subsequent) = conditional.subsequent() {
            collect_implicit_predicate_returns(&subsequent, wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    if let Some(conditional) = node.as_unless_node() {
        if let Some(statements) = conditional.statements() {
            collect_implicit_predicate_returns(&statements.as_node(), wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        if let Some(else_node) = conditional.else_clause() {
            collect_implicit_predicate_returns(&else_node.as_node(), wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    if let Some(else_node) = node.as_else_node() {
        if let Some(statements) = else_node.statements() {
            collect_implicit_predicate_returns(&statements.as_node(), wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    if let Some(logical) = node.as_and_node() {
        collect_implicit_predicate_returns(&logical.left(), wayward, values);
        collect_implicit_predicate_returns(&logical.right(), wayward, values);
        return;
    }
    if let Some(logical) = node.as_or_node() {
        collect_implicit_predicate_returns(&logical.left(), wayward, values);
        collect_implicit_predicate_returns(&logical.right(), wayward, values);
        return;
    }
    if let Some(case_node) = node.as_case_node() {
        for branch in case_node.conditions().iter() {
            if let Some(when_node) = branch.as_when_node() {
                if let Some(statements) = when_node.statements() {
                    collect_implicit_predicate_returns(&statements.as_node(), wayward, values);
                } else {
                    values.push(PredicateReturn::NonBoolean);
                }
            }
        }
        if let Some(else_node) = case_node.else_clause() {
            collect_implicit_predicate_returns(&else_node.as_node(), wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    if let Some(case_node) = node.as_case_match_node() {
        for branch in case_node.conditions().iter() {
            if let Some(in_node) = branch.as_in_node() {
                if let Some(statements) = in_node.statements() {
                    collect_implicit_predicate_returns(&statements.as_node(), wayward, values);
                } else {
                    values.push(PredicateReturn::NonBoolean);
                }
            }
        }
        if let Some(else_node) = case_node.else_clause() {
            collect_implicit_predicate_returns(&else_node.as_node(), wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    if let Some(loop_node) = node.as_while_node() {
        if let Some(statements) = loop_node.statements() {
            collect_implicit_predicate_returns(&statements.as_node(), wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    if let Some(loop_node) = node.as_until_node() {
        if let Some(statements) = loop_node.statements() {
            collect_implicit_predicate_returns(&statements.as_node(), wayward, values);
        } else {
            values.push(PredicateReturn::NonBoolean);
        }
        return;
    }
    values.push(predicate_return_kind(node, wayward));
}

fn predicate_return_kind(node: &Node<'_>, wayward: &[String]) -> PredicateReturn {
    if node.as_true_node().is_some() || node.as_false_node().is_some() {
        return PredicateReturn::Boolean;
    }
    if node.as_super_node().is_some() || node.as_forwarding_super_node().is_some() {
        return PredicateReturn::Super;
    }
    if let Some(call) = node.as_call_node() {
        let name = call.name().as_slice();
        let boolean = matches!(
            name,
            b"==" | b"===" | b"!=" | b"=~" | b"!~" | b"<" | b"<=" | b">" | b">=" | b"<=>" | b"!"
        ) || name.ends_with(b"?")
            && !wayward
                .iter()
                .any(|configured| configured.as_bytes() == name);
        return if boolean {
            PredicateReturn::Boolean
        } else {
            PredicateReturn::Unknown
        };
    }
    if predicate_non_boolean_literal(node) {
        PredicateReturn::NonBoolean
    } else {
        PredicateReturn::Unknown
    }
}

fn predicate_non_boolean_literal(node: &Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_interpolated_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_interpolated_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_rational_node().is_some()
        || node.as_imaginary_node().is_some()
        || node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_interpolated_regular_expression_node().is_some()
        || node.as_x_string_node().is_some()
        || node.as_interpolated_x_string_node().is_some()
        || node.as_range_node().is_some()
}

fn predicate_operator_method(name: &[u8]) -> bool {
    matches!(
        name,
        b"+" | b"-"
            | b"*"
            | b"**"
            | b"/"
            | b"%"
            | b"=="
            | b"==="
            | b"!="
            | b"=~"
            | b"!~"
            | b"<"
            | b"<="
            | b">"
            | b">="
            | b"<=>"
            | b"[]"
            | b"[]="
            | b"<<"
            | b">>"
            | b"&"
            | b"|"
            | b"^"
            | b"~"
            | b"+@"
            | b"-@"
            | b"!"
    )
}

struct SelfAssignment;

#[derive(Clone, Copy)]
enum SelfAssignmentVariable {
    Local,
    Instance,
    Class,
}

impl Cop for SelfAssignment {
    fn name(&self) -> &'static str {
        "Style/SelfAssignment"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let (name, operator, value, variable) =
            if let Some(write) = node.as_local_variable_write_node() {
                (
                    write.name().as_slice(),
                    write.operator_loc(),
                    write.value(),
                    SelfAssignmentVariable::Local,
                )
            } else if let Some(write) = node.as_instance_variable_write_node() {
                (
                    write.name().as_slice(),
                    write.operator_loc(),
                    write.value(),
                    SelfAssignmentVariable::Instance,
                )
            } else if let Some(write) = node.as_class_variable_write_node() {
                (
                    write.name().as_slice(),
                    write.operator_loc(),
                    write.value(),
                    SelfAssignmentVariable::Class,
                )
            } else {
                return;
            };

        let Some((shorthand, new_rhs)) = self_assignment_rhs(&value, name, variable) else {
            return;
        };
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        cop_context.replace_many(
            format!("Use self-assignment shorthand `{shorthand}=`."),
            node.location(),
            vec![
                (
                    operator.start_offset()..operator.start_offset(),
                    shorthand.to_string(),
                ),
                (
                    value.location().start_offset()..value.location().end_offset(),
                    source_at(source, &new_rhs.location()).to_string(),
                ),
            ],
        );
    }
}

fn self_assignment_rhs<'pr>(
    value: &Node<'pr>,
    name: &[u8],
    variable: SelfAssignmentVariable,
) -> Option<(&'static str, Node<'pr>)> {
    if let Some(call) = value.as_call_node() {
        const OPERATORS: &[(&[u8], &str)] = &[
            (b"+", "+"),
            (b"-", "-"),
            (b"*", "*"),
            (b"**", "**"),
            (b"/", "/"),
            (b"%", "%"),
            (b"^", "^"),
            (b"<<", "<<"),
            (b">>", ">>"),
            (b"|", "|"),
            (b"&", "&"),
        ];
        let shorthand = OPERATORS.iter().find_map(|(method, shorthand)| {
            (call.name().as_slice() == *method).then_some(*shorthand)
        })?;
        let receiver = call.receiver()?;
        if !same_assignment_variable(&receiver, name, variable) {
            return None;
        }
        let arguments = call.arguments()?;
        let mut arguments = arguments.arguments().iter();
        let argument = arguments.next()?;
        return arguments.next().is_none().then_some((shorthand, argument));
    }

    if let Some(boolean) = value.as_or_node() {
        return same_assignment_variable(&boolean.left(), name, variable)
            .then(|| ("||", boolean.right()));
    }
    if let Some(boolean) = value.as_and_node() {
        return same_assignment_variable(&boolean.left(), name, variable)
            .then(|| ("&&", boolean.right()));
    }
    None
}

fn same_assignment_variable(
    node: &Node<'_>,
    name: &[u8],
    variable: SelfAssignmentVariable,
) -> bool {
    match variable {
        SelfAssignmentVariable::Local => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == name),
        SelfAssignmentVariable::Instance => node
            .as_instance_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == name),
        SelfAssignmentVariable::Class => node
            .as_class_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == name),
    }
}

fn memoized_variable(context: &mut CopContext<'_, '_>) {
    if context.source().contains("define_method")
        || context.source().contains("define_singleton_method")
    {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut method = None::<String>;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        if let Some(definition) = line.split_once("def ").map(|(_, definition)| definition) {
            method = Some(
                definition
                    .split(['(', ' '])
                    .next()
                    .unwrap_or("")
                    .rsplit('.')
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('_')
                    .to_string(),
            );
        }
        if let Some(at) = line.find("@") {
            let memo_is_last = index + 1 < lines.len()
                && (lines[index + 1].1.trim() == "end"
                    || (line[at..].contains("begin")
                        && lines[index + 1..]
                            .iter()
                            .rev()
                            .take(2)
                            .all(|(_, line)| line.trim() == "end")));
            if line[at..].contains("||=") && memo_is_last {
                let name = line[at + 1..]
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                    .next()
                    .unwrap_or("");
                let normalized = name.trim_start_matches('_');
                if method.as_deref().is_some_and(|method| {
                    !method.starts_with("initialize")
                        && method.trim_end_matches(['?', '!', '=']) != normalized
                }) {
                    let method = method.as_deref().unwrap_or("");
                    let actual = format!("@{name}");
                    let expected = format!("@{}", method.trim_end_matches(['?', '!', '=']));
                    context.replace(
                        format!(
                            "Memoized variable `{actual}` does not match method name `{method}`. Use `{expected}` instead."
                        ),
                        offset + at..offset + at + name.len() + 1,
                        offset + at..offset + at + name.len() + 1,
                        expected,
                    );
                }
            }
        }
        if line.trim() == "end" {
            method = None;
        }
    }
}

fn file_name(context: &mut CopContext<'_, '_>) {
    if context.source().starts_with("#!") {
        return;
    }
    let Some(file) = std::path::Path::new(context.path())
        .file_name()
        .and_then(|name| name.to_str())
    else {
        return;
    };
    if file
        .bytes()
        .any(|byte| byte.is_ascii_uppercase() || byte == b'-')
    {
        context.report(
            format!("The name of this source file (`{file}`) should use snake_case."),
            0..0,
        );
    }
}

fn variable_number(context: &mut CopContext<'_, '_>) {
    if !context.config_values("AllowedPatterns").is_empty() {
        return;
    }
    let snake_case = context.policy().enforced_style("normalcase") == "snake_case";
    for (offset, line) in context.source_file().lines() {
        let Some((left, _)) = line.split_once('=') else {
            continue;
        };
        for word in
            left.split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        {
            if context
                .config_values("AllowedIdentifiers")
                .iter()
                .any(|allowed| allowed == word)
            {
                continue;
            }
            if snake_case
                && word
                    .chars()
                    .last()
                    .is_some_and(|character| character.is_ascii_digit())
                && !word
                    .trim_end_matches(|character: char| character.is_ascii_digit())
                    .ends_with('_')
                && word.chars().any(|character| character.is_ascii_lowercase())
            {
                let start = offset + line.find(word).unwrap_or(0);
                context.report(
                    "Use normalcase for numbered variables.",
                    start..start + word.len(),
                );
            }
        }
    }
}

fn variable_name(context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("snake_case").to_string();
    if !context.config_values("AllowedPatterns").is_empty() {
        return;
    }
    let allowed = context.config_values("AllowedIdentifiers").to_vec();
    for (offset, line) in context.source_file().lines() {
        let mut candidates = Vec::<&str>::new();
        if let Some((_, tail)) = line
            .trim_start()
            .strip_prefix("def ")
            .and_then(|line| line.split_once('('))
        {
            if let Some((parameters, _)) = tail.rsplit_once(')') {
                candidates.extend(parameters.split(','));
            }
        }
        if let Some(first) = line.find('|') {
            if let Some(last) = line[first + 1..].find('|').map(|at| first + 1 + at) {
                candidates.extend(line[first + 1..last].split([',', ';']));
            }
        }
        if !line.trim_start().starts_with("def ") {
            if let Some((left, _)) = line.split_once('=') {
                if !left.ends_with(['=', '!', '<', '>']) {
                    candidates.extend(left.split(','));
                }
            }
        }
        let mut search_from = 0;
        for candidate in candidates {
            let token = candidate
                .trim()
                .trim_start_matches(['*', '&'])
                .split(['=', ':'])
                .next()
                .unwrap_or("")
                .trim();
            let bare = token.trim_start_matches(['@', '$']);
            if bare.is_empty()
                || allowed.iter().any(|allowed| allowed == bare)
                || !invalid_variable_name(bare, &style)
            {
                continue;
            }
            let start = offset
                + line[search_from..]
                    .find(token)
                    .map_or(0, |relative| search_from + relative);
            search_from = start - offset + token.len();
            context.report(
                format!("Use {style} for variable names."),
                start..start + token.len(),
            );
        }
    }
}

fn invalid_variable_name(name: &str, style: &str) -> bool {
    let name = name.trim_start_matches('_');
    if style == "camelCase" {
        name.contains('_')
            || name
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_uppercase())
    } else {
        name.bytes().any(|byte| byte.is_ascii_uppercase())
    }
}

fn useless_assignment(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        if let Some((name, _)) = line.split_once(" = ") {
            let name = name.trim();
            if !name.starts_with(['@', '$'])
                && !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
                && !lines[index + 1..]
                    .iter()
                    .take_while(|(_, later)| later.trim() != "end")
                    .any(|(_, later)| {
                        later
                            .split(|character: char| {
                                !character.is_ascii_alphanumeric() && character != '_'
                            })
                            .any(|word| word == name)
                    })
            {
                let start = offset + line.find(name).unwrap_or(0);
                context.replace(
                    format!("Useless assignment to variable - `{name}`."),
                    start..start + name.len(),
                    start..start + name.len(),
                    name,
                );
            }
        }
    }
}

fn method_name(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("snake_case") != "snake_case" {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let Some(definition) = line.trim_start().strip_prefix("def ") else {
            continue;
        };
        let name = definition.split(['(', ' ']).next().unwrap_or("");
        let bare = name.rsplit('.').next().unwrap_or(name);
        if name.contains('.')
            && bare
                .bytes()
                .next()
                .is_some_and(|byte| byte.is_ascii_uppercase())
        {
            continue;
        }
        if context
            .config_values("AllowedPatterns")
            .iter()
            .any(|pattern| pattern.contains(bare))
        {
            continue;
        }
        if name.bytes().any(|byte| byte.is_ascii_uppercase()) {
            let start = offset + line.find(name).unwrap_or(0);
            context.report(
                "Use snake_case for method names.",
                start..start + name.len(),
            );
        }
    }
}
