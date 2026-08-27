use super::catalog_cop::compatibility_custom;
use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(MemoizedVariable),
        compatibility_custom("Naming/FileName", file_name),
        Box::new(AssignmentInCondition),
        Box::new(VariableNumber),
        Box::new(VariableName),
        Box::new(UselessAssignment),
        Box::new(MethodName),
        Box::new(PredicateMethod),
    ]
}

struct PredicateMethod;

#[derive(Clone, Copy, PartialEq, Eq)]
enum PredicateReturn {
    Boolean,
    NonBoolean,
    Unknown,
    UnknownCall,
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
                collect_implicit_predicate_returns(value, wayward, &mut values);
            } else {
                values.push(PredicateReturn::NonBoolean);
            }
        }
        collect_implicit_predicate_returns(&body, wayward, &mut values);

        let conservative =
            cop_context.config_value("Mode").unwrap_or("conservative") == "conservative";
        if conservative
            && values
                .iter()
                .any(|value| matches!(value, PredicateReturn::UnknownCall | PredicateReturn::Super))
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
        collect_if_predicate_returns(&conditional, wayward, values, true);
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
        collect_logical_predicate_return(&logical.left(), wayward, values);
        collect_logical_predicate_return(&logical.right(), wayward, values);
        return;
    }
    if let Some(logical) = node.as_or_node() {
        collect_logical_predicate_return(&logical.left(), wayward, values);
        collect_logical_predicate_return(&logical.right(), wayward, values);
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

fn collect_if_predicate_returns<'pr>(
    conditional: &ruby_prism::IfNode<'pr>,
    wayward: &[String],
    values: &mut Vec<PredicateReturn>,
    add_missing_else: bool,
) {
    if let Some(statements) = conditional.statements() {
        collect_implicit_predicate_returns(&statements.as_node(), wayward, values);
    } else {
        values.push(PredicateReturn::NonBoolean);
    }
    if let Some(subsequent) = conditional.subsequent() {
        if let Some(elsif) = subsequent.as_if_node() {
            collect_if_predicate_returns(&elsif, wayward, values, false);
        } else {
            collect_implicit_predicate_returns(&subsequent, wayward, values);
        }
    } else if add_missing_else {
        values.push(PredicateReturn::NonBoolean);
    }
}

fn collect_logical_predicate_return<'pr>(
    node: &Node<'pr>,
    wayward: &[String],
    values: &mut Vec<PredicateReturn>,
) {
    if node.as_parentheses_node().is_some() {
        values.push(PredicateReturn::Unknown);
    } else {
        collect_implicit_predicate_returns(node, wayward, values);
    }
}

fn predicate_return_kind(node: &Node<'_>, wayward: &[String]) -> PredicateReturn {
    if node.as_true_node().is_some() || node.as_false_node().is_some() {
        return PredicateReturn::Boolean;
    }
    if node.as_super_node().is_some() || node.as_forwarding_super_node().is_some() {
        return PredicateReturn::Super;
    }
    if let Some(call) = node.as_call_node() {
        if call
            .block()
            .is_some_and(|block| block.as_block_node().is_some())
        {
            return PredicateReturn::Unknown;
        }
        let name = call.name().as_slice();
        let boolean = matches!(
            name,
            b"==" | b"===" | b"!=" | b"<" | b"<=" | b">" | b">=" | b"!"
        ) || name.ends_with(b"?")
            && !wayward
                .iter()
                .any(|configured| configured.as_bytes() == name);
        return if boolean {
            PredicateReturn::Boolean
        } else {
            PredicateReturn::UnknownCall
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

struct MemoizedVariable;

impl Cop for MemoizedVariable {
    fn name(&self) -> &'static str {
        "Naming/MemoizedInstanceVariableName"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if let Some(write) = node.as_instance_variable_or_write_node() {
            let Some((method, body)) = memoized_definition(ancestors, source) else {
                return;
            };
            if !memoized_final_expression(&body, node) {
                return;
            }
            let name = String::from_utf8_lossy(write.name().as_slice()).into_owned();
            report_memoized_name(
                &method,
                &name,
                write.location().start_offset()..write.location().start_offset() + name.len(),
                &mut cop_context,
            );
        } else if let Some(defined) = node.as_defined_node() {
            check_defined_memoization(&defined, ancestors, source, &mut cop_context);
        }
    }
}

fn memoized_definition<'pr>(
    ancestors: &[Node<'pr>],
    source: &'pr str,
) -> Option<(String, Node<'pr>)> {
    for (index, ancestor) in ancestors.iter().enumerate().rev() {
        if let Some(definition) = ancestor.as_def_node() {
            let body = definition.body()?;
            return Some((
                String::from_utf8_lossy(definition.name().as_slice()).into_owned(),
                body,
            ));
        }
        let Some(block) = ancestor.as_block_node() else {
            continue;
        };
        let Some(call) = ancestors[..index].iter().rev().find_map(|candidate| {
            let call = candidate.as_call_node()?;
            let owned = call.block().is_some_and(|owned| {
                owned.location().start_offset() == block.location().start_offset()
                    && owned.location().end_offset() == block.location().end_offset()
            });
            owned.then_some(call)
        }) else {
            continue;
        };
        if !matches!(
            call_name(&call),
            b"define_method" | b"define_singleton_method"
        ) {
            continue;
        }
        let argument = first_argument(&call)?;
        let name = if let Some(symbol) = argument.as_symbol_node() {
            String::from_utf8_lossy(symbol.unescaped()).into_owned()
        } else if let Some(string) = argument.as_string_node() {
            String::from_utf8_lossy(string.unescaped()).into_owned()
        } else {
            continue;
        };
        return Some((name, block.body()?));
    }
    let _ = source;
    None
}

fn memoized_final_expression(body: &Node<'_>, candidate: &Node<'_>) -> bool {
    if body.location().start_offset() == candidate.location().start_offset()
        && body.location().end_offset() == candidate.location().end_offset()
    {
        return true;
    }
    if let Some(last) = body
        .as_statements_node()
        .and_then(|statements| statements.body().iter().last())
    {
        if body
            .as_statements_node()
            .is_some_and(|statements| statements.body().len() > 1)
            && (last.location().start_offset() != candidate.location().start_offset()
                || last.location().end_offset() != candidate.location().end_offset())
        {
            return false;
        }
        return memoized_final_expression(&last, candidate);
    }
    if let Some(block) = body.as_block_node() {
        return block
            .body()
            .is_some_and(|body| memoized_final_expression(&body, candidate));
    }
    if let Some(call) = body.as_call_node() {
        return call
            .block()
            .and_then(|block| block.as_block_node())
            .and_then(|block| block.body())
            .is_some_and(|body| memoized_only_statement(&body, candidate));
    }
    if let Some(condition) = body.as_if_node() {
        if condition
            .if_keyword_loc()
            .is_some_and(|keyword| keyword.start_offset() != condition.location().start_offset())
        {
            return false;
        }
        if let Some(subsequent) = condition.subsequent() {
            return if let Some(otherwise) = subsequent.as_else_node() {
                otherwise.statements().is_some_and(|statements| {
                    memoized_only_statement(&statements.as_node(), candidate)
                })
            } else {
                false
            };
        }
        return false;
    }
    if let Some(condition) = body.as_unless_node() {
        if condition.keyword_loc().start_offset() != condition.location().start_offset() {
            return false;
        }
        if let Some(otherwise) = condition.else_clause() {
            return otherwise.statements().is_some_and(|statements| {
                memoized_only_statement(&statements.as_node(), candidate)
            });
        }
        return false;
    }
    if let Some(case_node) = body.as_case_node() {
        let statements = case_node
            .else_clause()
            .and_then(|branch| branch.statements());
        return statements
            .is_some_and(|statements| memoized_only_statement(&statements.as_node(), candidate));
    }
    if let Some(begin) = body.as_begin_node() {
        if begin.rescue_clause().is_some()
            || begin.else_clause().is_some()
            || begin.ensure_clause().is_some()
        {
            return false;
        }
        return begin
            .statements()
            .is_some_and(|statements| memoized_final_expression(&statements.as_node(), candidate));
    }
    false
}

fn memoized_only_statement(body: &Node<'_>, candidate: &Node<'_>) -> bool {
    if let Some(statements) = body.as_statements_node() {
        return statements.body().len() == 1
            && statements
                .body()
                .first()
                .is_some_and(|statement| memoized_final_expression(&statement, candidate));
    }
    memoized_final_expression(body, candidate)
}

fn report_memoized_name(
    method: &str,
    variable: &str,
    range: std::ops::Range<usize>,
    context: &mut CopContext<'_, '_>,
) {
    if matches!(
        method,
        "initialize" | "initialize_clone" | "initialize_copy" | "initialize_dup"
    ) {
        return;
    }
    let method_name = method.replace(['!', '?', '='], "");
    let variable_name = variable.trim_start_matches('@');
    let style = context
        .config_value("EnforcedStyleForLeadingUnderscores")
        .unwrap_or("disallowed");
    let no_underscore = method_name.strip_prefix('_').unwrap_or(&method_name);
    let with_underscore = format!("_{method_name}");
    let matches = match style {
        "required" => {
            variable_name == with_underscore
                || method_name.starts_with('_') && variable_name == method_name
        }
        "optional" => {
            variable_name == method_name
                || variable_name == with_underscore
                || variable_name == no_underscore
        }
        _ => variable_name == method_name || variable_name == no_underscore,
    };
    if matches {
        return;
    }
    let suggestion = if style == "required" {
        with_underscore
    } else {
        method_name
    };
    let expected = format!("@{suggestion}");
    let message = if style == "required" && !variable_name.starts_with('_') {
        format!("Memoized variable `{variable}` does not start with `_`. Use `{expected}` instead.")
    } else {
        format!(
            "Memoized variable `{variable}` does not match method name `{method}`. Use `{expected}` instead."
        )
    };
    context.replace(message, range.clone(), range, expected);
}

fn check_defined_memoization(
    defined: &ruby_prism::DefinedNode<'_>,
    ancestors: &[Node<'_>],
    source: &str,
    context: &mut CopContext<'_, '_>,
) {
    let Some(defined_read) = defined.value().as_instance_variable_read_node() else {
        return;
    };
    let name = defined_read.name().as_slice();
    let Some((method, body)) = memoized_definition(ancestors, source) else {
        return;
    };
    let Some(statements) = body.as_statements_node() else {
        return;
    };
    let body_nodes = statements.body().iter().collect::<Vec<_>>();
    let (Some(first), Some(last)) = (body_nodes.first(), body_nodes.last()) else {
        return;
    };
    let Some(condition) = first.as_if_node() else {
        return;
    };
    if condition.predicate().location().start_offset() != defined.location().start_offset()
        || condition.predicate().location().end_offset() != defined.location().end_offset()
        || condition.subsequent().is_some()
    {
        return;
    }
    let Some(return_read) = condition
        .statements()
        .and_then(|statements| statements.body().first())
        .and_then(|statement| statement.as_return_node())
        .and_then(|returned| returned.arguments())
        .and_then(|arguments| arguments.arguments().first())
        .and_then(|argument| argument.as_instance_variable_read_node())
    else {
        return;
    };
    let Some(assignment) = last.as_instance_variable_write_node() else {
        return;
    };
    if return_read.name().as_slice() != name || assignment.name().as_slice() != name {
        return;
    }
    let variable = String::from_utf8_lossy(name).into_owned();
    let ranges = [
        defined_read.location(),
        return_read.location(),
        assignment.location(),
    ];
    for location in ranges {
        report_memoized_name(
            &method,
            &variable,
            location.start_offset()..location.start_offset() + variable.len(),
            context,
        );
    }
}

#[allow(clippy::too_many_lines)]
fn file_name(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    if context.source().starts_with("#!") && context.config_bool("IgnoreExecutableScripts", true) {
        return;
    }
    let path = std::path::Path::new(context.path());
    let Some(file) = path.file_name().and_then(|name| name.to_str()) else {
        return;
    };
    let stem = file
        .trim_start_matches('.')
        .split_once('.')
        .map_or(file.trim_start_matches('.'), |(stem, _)| stem)
        .replacen('+', "_", 1);
    let regex_pattern = context
        .config_map("Regex")
        .and_then(|values| values.get("$regexp"));
    let filename_good = if let Some(pattern) = regex_pattern {
        let normalized = pattern.replace("\\\\", "\\");
        if normalized == "\\A[aeiou]\\z" {
            stem.len() == 1 && "aeiouAEIOU".contains(&stem)
        } else {
            regex::Regex::new(&normalized).is_ok_and(|regex| regex.is_match(&stem))
        }
    } else {
        file.ends_with(".gemspec")
            || matches!(file, "Gemfile" | "Rakefile")
            || stem.chars().all(|character| {
                character.is_lowercase()
                    || character.is_ascii_digit()
                    || matches!(character, '_' | '.' | '?' | '!')
            })
    };
    if !filename_good {
        let message = if let Some(pattern) = regex_pattern {
            let normalized = pattern.replace("\\\\", "\\");
            let rendered = if normalized == "\\A[aeiou]\\z" {
                "(?i-mx:\\A[aeiou]\\z)"
            } else {
                &normalized
            };
            format!("`{file}` should match `{rendered}`.")
        } else {
            format!("The name of this source file (`{file}`) should use snake_case.")
        };
        context.report(message, 0..0);
        return;
    }
    if !context.config_bool("ExpectMatchingDefinition", false) {
        return;
    }

    #[derive(Default)]
    struct Definitions<'source> {
        source: &'source str,
        stack: Vec<String>,
        names: Vec<String>,
    }

    impl Definitions<'_> {
        fn enter(&mut self, raw: &str) {
            let raw = raw.trim_start_matches("::");
            let name = if raw.contains("::") || self.stack.is_empty() {
                raw.to_string()
            } else {
                format!("{}::{raw}", self.stack.last().expect("nonempty stack"))
            };
            self.names.push(name.clone());
            self.stack.push(name);
        }

        fn source_name(&self, node: &Node<'_>) -> String {
            let location = node.location();
            self.source[location.start_offset()..location.end_offset()].to_string()
        }
    }

    impl<'pr> Visit<'pr> for Definitions<'_> {
        fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
            self.enter(&self.source_name(&node.constant_path()));
            ruby_prism::visit_module_node(self, node);
            self.stack.pop();
        }

        fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
            self.enter(&self.source_name(&node.constant_path()));
            ruby_prism::visit_class_node(self, node);
            self.stack.pop();
        }

        fn visit_constant_write_node(&mut self, node: &ruby_prism::ConstantWriteNode<'pr>) {
            if node.value().as_call_node().is_some_and(|call| {
                call_name(&call) == b"new" && root_constant(call.receiver(), b"Struct")
            }) {
                let raw = String::from_utf8_lossy(node.name().as_slice()).into_owned();
                let name = self
                    .stack
                    .last()
                    .map_or_else(|| raw.clone(), |scope| format!("{scope}::{raw}"));
                self.names.push(name);
            }
            ruby_prism::visit_constant_write_node(self, node);
        }

        fn visit_constant_path_write_node(
            &mut self,
            node: &ruby_prism::ConstantPathWriteNode<'pr>,
        ) {
            if node.value().as_call_node().is_some_and(|call| {
                call_name(&call) == b"new" && root_constant(call.receiver(), b"Struct")
            }) {
                let location = node.target().location();
                self.names
                    .push(self.source[location.start_offset()..location.end_offset()].to_string());
            }
            ruby_prism::visit_constant_path_write_node(self, node);
        }
    }

    let mut definitions = Definitions {
        source: context.source(),
        ..Definitions::default()
    };
    definitions.visit(&parse(context.source().as_bytes()).node());

    let module_name = |component: &str| {
        component
            .split('.')
            .next()
            .unwrap_or(component)
            .split('_')
            .map(|word| {
                let mut chars = word.chars();
                chars.next().map_or_else(String::new, |first| {
                    format!("{}{}", first.to_ascii_uppercase(), chars.as_str())
                })
            })
            .collect::<String>()
    };
    let mut components = path
        .components()
        .filter_map(|component| component.as_os_str().to_str())
        .collect::<Vec<_>>();
    let check_hierarchy = context.config_bool("CheckDefinitionPathHierarchy", true);
    let expected = if check_hierarchy {
        let roots = context.config_values("CheckDefinitionPathHierarchyRoots");
        let start = components
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, component)| {
                roots.contains(&component.to_string()).then_some(index + 1)
            });
        if let Some(start) = start {
            components.drain(..start);
        } else {
            components = vec![file];
        }
        components
            .iter()
            .map(|component| module_name(component))
            .collect::<Vec<_>>()
            .join("::")
    } else {
        module_name(file)
    };
    let acronyms = context.config_values("AllowedAcronyms");
    let normalize_acronyms = |mut name: String| {
        for acronym in acronyms {
            let mut chars = acronym.chars();
            let replacement = chars.next().map_or_else(String::new, |first| {
                format!(
                    "{}{}",
                    first.to_ascii_uppercase(),
                    chars.as_str().to_ascii_lowercase()
                )
            });
            name = name.replace(acronym, &replacement);
        }
        name
    };
    let matching_definition = definitions.names.iter().any(|name| {
        let name = normalize_acronyms(name.clone());
        if expected.contains("::") {
            name == expected
        } else {
            name.rsplit("::").next() == Some(expected.as_str())
        }
    });
    if !matching_definition {
        context.report(
            format!("`{file}` should define a class or module called `{expected}`."),
            0..0,
        );
    }
}

struct AssignmentInCondition;

impl Cop for AssignmentInCondition {
    fn name(&self) -> &'static str {
        "Lint/AssignmentInCondition"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(operator) = plain_assignment_operator(node, source) else {
            return;
        };
        if !assignment_is_in_condition(node, ancestors) {
            return;
        }
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        let allow_safe = cop_context.config_bool("AllowSafeAssignment", true);
        let already_parenthesized = directly_parenthesized_assignment(node, ancestors);
        if allow_safe && already_parenthesized {
            return;
        }
        let message = if allow_safe {
            "Use `==` if you meant to do a comparison or wrap the expression in parentheses to indicate you meant to assign in a condition."
        } else {
            "Use `==` if you meant to do a comparison or move the assignment up out of the condition."
        };
        if allow_safe {
            let location = node.location();
            let replacement = format!(
                "({})",
                &source[location.start_offset()..location.end_offset()]
            );
            cop_context.replace(message, operator, location, replacement);
        } else {
            cop_context.report(message, operator);
        }
    }
}

fn directly_parenthesized_assignment(node: &Node<'_>, ancestors: &[Node<'_>]) -> bool {
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_statements_node().is_some() {
            continue;
        }
        if ancestor.as_and_node().is_some() || ancestor.as_or_node().is_some() {
            return false;
        }
        return ancestor.as_parentheses_node().is_some_and(|parentheses| {
            parentheses
                .body()
                .and_then(|body| body.as_statements_node())
                .is_some_and(|statements| {
                    statements.body().len() == 1
                        && statements.body().first().is_some_and(|expression| {
                            expression.location().start_offset() == node.location().start_offset()
                                && expression.location().end_offset()
                                    == node.location().end_offset()
                        })
                })
        });
    }
    false
}

fn plain_assignment_operator(node: &Node<'_>, source: &str) -> Option<std::ops::Range<usize>> {
    macro_rules! assignment {
        ($($cast:ident),+ $(,)?) => {$ (
            if let Some(write) = node.$cast() {
                let operator = write.operator_loc();
                return Some(operator.start_offset()..operator.end_offset());
            }
        )+ };
    }
    assignment!(
        as_local_variable_write_node,
        as_instance_variable_write_node,
        as_class_variable_write_node,
        as_global_variable_write_node,
        as_constant_write_node,
        as_constant_path_write_node,
    );
    if let Some(write) = node.as_multi_write_node() {
        let start = write.location().start_offset();
        let end = write.value().location().start_offset();
        return source[start..end]
            .rfind('=')
            .map(|relative| start + relative..start + relative + 1);
    }
    node.as_call_node().and_then(|call| {
        call.equal_loc()
            .map(|operator| operator.start_offset()..operator.end_offset())
    })
}

fn assignment_is_in_condition(node: &Node<'_>, ancestors: &[Node<'_>]) -> bool {
    let location = node.location();
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_block_node().is_some()
            || ancestor.as_lambda_node().is_some()
            || ancestor.as_def_node().is_some()
        {
            return false;
        }
        if ancestor.as_call_node().is_some() {
            return false;
        }
        let predicate = if let Some(condition) = ancestor.as_if_node() {
            Some(condition.predicate())
        } else if let Some(condition) = ancestor.as_unless_node() {
            Some(condition.predicate())
        } else if let Some(condition) = ancestor.as_while_node() {
            Some(condition.predicate())
        } else {
            ancestor
                .as_until_node()
                .map(|condition| condition.predicate())
        };
        if let Some(predicate) = predicate {
            let predicate = predicate.location();
            return predicate.start_offset() <= location.start_offset()
                && location.end_offset() <= predicate.end_offset();
        }
    }
    false
}

struct VariableName;

impl Cop for VariableName {
    fn name(&self) -> &'static str {
        "Naming/VariableName"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let parameters = if let Some(definition) = node.as_def_node() {
            definition.parameters()
        } else if let Some(block) = node.as_block_node() {
            block
                .parameters()
                .and_then(|parameters| parameters.as_block_parameters_node())
                .and_then(|parameters| parameters.parameters())
        } else if let Some(lambda) = node.as_lambda_node() {
            lambda
                .parameters()
                .and_then(|parameters| parameters.as_block_parameters_node())
                .and_then(|parameters| parameters.parameters())
        } else {
            None
        };
        if let Some(parameter) = parameters.and_then(|parameters| parameters.block()) {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                let mut cop_context = context.cop_context(self.name(), source, ancestors);
                check_variable_name(
                    String::from_utf8_lossy(name.as_slice()).into_owned(),
                    location,
                    &mut cop_context,
                );
            }
        }
        let Some((name, location, global)) = variable_identifier(node, true) else {
            return;
        };
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if global {
            let bare = name.trim_start_matches('$');
            let forbidden = cop_context
                .config_values("ForbiddenIdentifiers")
                .iter()
                .any(|item| item == bare)
                || patterns_match(cop_context.config_values("ForbiddenPatterns"), bare);
            if forbidden {
                cop_context.report(
                    format!("`{name}` is forbidden, use another name instead."),
                    location,
                );
            }
            return;
        }
        check_variable_name(name, location, &mut cop_context);
    }
}

fn check_variable_name(
    name: String,
    location: ruby_prism::Location<'_>,
    cop_context: &mut CopContext<'_, '_>,
) {
    let bare = name.trim_start_matches(['@', '$']);
    if bare.is_empty()
        || name.starts_with('$')
            && bare
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'_' || byte.is_ascii_digit())
        || identifier_allowed(bare, cop_context)
    {
        return;
    }

    let offense = location.start_offset()..location.start_offset() + name.len();
    let forbidden = cop_context
        .config_values("ForbiddenIdentifiers")
        .iter()
        .any(|item| item == bare)
        || patterns_match(cop_context.config_values("ForbiddenPatterns"), bare);
    if forbidden {
        cop_context.report(
            format!("`{name}` is forbidden, use another name instead."),
            offense,
        );
        return;
    }

    let style = cop_context.policy().enforced_style("snake_case");
    if invalid_variable_name(bare, style) {
        cop_context.report(format!("Use {style} for variable names."), offense);
    }
}

struct VariableNumber;

impl Cop for VariableNumber {
    fn name(&self) -> &'static str {
        "Naming/VariableNumber"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let (name, location, kind) = if let Some(definition) = node.as_def_node() {
            let location = definition.name_loc();
            (
                String::from_utf8_lossy(definition.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset(),
                "method name",
            )
        } else if let Some(symbol) = node.as_symbol_node() {
            // Parser AST represents a hash label's expression as its name,
            // excluding the trailing colon. Prism's SymbolNode location
            // includes that colon, while an ordinary `:symbol` offense does
            // include its leading colon.
            let symbol_location = symbol.location();
            let location = if symbol_location.as_slice().ends_with(b":") {
                symbol_location.start_offset()..symbol_location.end_offset() - 1
            } else {
                symbol_location.start_offset()..symbol_location.end_offset()
            };
            (
                String::from_utf8_lossy(symbol.unescaped()).into_owned(),
                location,
                "symbol",
            )
        } else if let Some((name, location, _)) = variable_number_identifier(node) {
            (
                name,
                location.start_offset()..location.end_offset(),
                "variable",
            )
        } else {
            return;
        };
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if kind == "method name" && !cop_context.config_bool("CheckMethodNames", true)
            || kind == "symbol" && !cop_context.config_bool("CheckSymbols", true)
        {
            return;
        }
        let bare = name.trim_start_matches(['@', '$']);
        let empty_hash_label = kind == "symbol"
            && bare.is_empty()
            && source.as_bytes().get(location.end) == Some(&b':');
        if bare.is_empty() && !empty_hash_label
            || bare == "_1"
            || !bare.is_empty() && bare.chars().all(|character| character.is_ascii_digit())
            || identifier_allowed(bare, &cop_context)
        {
            return;
        }
        let style = cop_context.policy().enforced_style("normalcase");
        if invalid_variable_number(bare, style) {
            cop_context.report(format!("Use {style} for {kind} numbers."), location);
        }
    }
}

fn variable_number_identifier<'pr>(
    node: &Node<'pr>,
) -> Option<(String, ruby_prism::Location<'pr>, bool)> {
    // RuboCop's callback contract aliases only `on_arg` here. Optional,
    // keyword, rest, keyword-rest, and block parameters deliberately do not
    // participate in VariableNumber (unlike Naming/VariableName).
    if node.as_optional_parameter_node().is_some()
        || node.as_required_keyword_parameter_node().is_some()
        || node.as_optional_keyword_parameter_node().is_some()
        || node.as_rest_parameter_node().is_some()
        || node.as_keyword_rest_parameter_node().is_some()
        || node.as_block_parameter_node().is_some()
    {
        return None;
    }
    variable_identifier(node, false)
}

fn variable_identifier<'pr>(
    node: &Node<'pr>,
    include_local_reads: bool,
) -> Option<(String, ruby_prism::Location<'pr>, bool)> {
    macro_rules! named {
        ($cast:ident, $loc:ident, $global:expr) => {
            if let Some(value) = node.$cast() {
                return Some((
                    String::from_utf8_lossy(value.name().as_slice()).into_owned(),
                    value.$loc(),
                    $global,
                ));
            }
        };
    }
    if include_local_reads {
        named!(as_local_variable_read_node, location, false);
    }
    named!(as_local_variable_write_node, name_loc, false);
    named!(as_local_variable_target_node, location, false);
    named!(as_local_variable_and_write_node, name_loc, false);
    named!(as_local_variable_or_write_node, name_loc, false);
    named!(as_local_variable_operator_write_node, name_loc, false);
    named!(as_instance_variable_write_node, name_loc, false);
    named!(as_instance_variable_target_node, location, false);
    named!(as_instance_variable_and_write_node, name_loc, false);
    named!(as_instance_variable_or_write_node, name_loc, false);
    named!(as_instance_variable_operator_write_node, name_loc, false);
    named!(as_class_variable_write_node, name_loc, false);
    named!(as_class_variable_target_node, location, false);
    named!(as_class_variable_and_write_node, name_loc, false);
    named!(as_class_variable_or_write_node, name_loc, false);
    named!(as_class_variable_operator_write_node, name_loc, false);
    named!(as_global_variable_write_node, name_loc, true);
    named!(as_global_variable_target_node, location, true);
    named!(as_global_variable_and_write_node, name_loc, true);
    named!(as_global_variable_or_write_node, name_loc, true);
    named!(as_global_variable_operator_write_node, name_loc, true);
    named!(as_required_parameter_node, location, false);
    named!(as_optional_parameter_node, name_loc, false);
    named!(as_required_keyword_parameter_node, name_loc, false);
    named!(as_optional_keyword_parameter_node, name_loc, false);
    if let Some(value) = node.as_rest_parameter_node() {
        return value.name().zip(value.name_loc()).map(|(name, location)| {
            (
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                location,
                false,
            )
        });
    }
    if let Some(value) = node.as_keyword_rest_parameter_node() {
        return value.name().zip(value.name_loc()).map(|(name, location)| {
            (
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                location,
                false,
            )
        });
    }
    if let Some(value) = node.as_block_parameter_node() {
        return value.name().zip(value.name_loc()).map(|(name, location)| {
            (
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                location,
                false,
            )
        });
    }
    None
}

fn identifier_allowed(name: &str, context: &CopContext<'_, '_>) -> bool {
    context
        .config_values("AllowedIdentifiers")
        .iter()
        .any(|allowed| allowed == name)
        || !context.config_explicit("AllowedIdentifiers")
            && matches!(
                name,
                "TLS1_1"
                    | "TLS1_2"
                    | "capture3"
                    | "iso8601"
                    | "rfc1123_date"
                    | "rfc822"
                    | "rfc2822"
                    | "rfc3339"
                    | "x86_64"
            )
        || patterns_match(context.config_values("AllowedPatterns"), name)
}

fn patterns_match(patterns: &[String], name: &str) -> bool {
    patterns.iter().any(|pattern| {
        let normalized = pattern.replace("\\A", "^").replace("\\z", "$");
        regex::Regex::new(&normalized).is_ok_and(|matcher| matcher.is_match(name))
    })
}

fn invalid_variable_number(name: &str, style: &str) -> bool {
    if name.is_empty() {
        return true;
    }
    if style == "non_integer" {
        return name.chars().any(|character| character.is_ascii_digit());
    }
    let prefix = name.trim_end_matches(|character: char| character.is_ascii_digit());
    if prefix.len() == name.len() {
        return false;
    }
    if style == "snake_case" {
        !prefix.ends_with('_')
    } else {
        prefix.ends_with('_')
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

struct UselessAssignment;

impl Cop for UselessAssignment {
    fn name(&self) -> &'static str {
        "Lint/UselessAssignment"
    }
    fn phase(&self) -> CopPhase {
        CopPhase::Source
    }

    #[allow(clippy::cognitive_complexity)]
    fn on_source(&self, source: &str, context: &mut Context) {
        let parsed = parse(source.as_bytes());
        let mut collector = AssignmentEventCollector {
            source,
            scope: 0,
            next_scope: 1,
            events: Vec::new(),
            candidates: Vec::new(),
            branches: Vec::new(),
            next_branch: 0,
            modifier_branches: std::collections::HashSet::new(),
            incomplete_branches: std::collections::HashSet::new(),
            loops: Vec::new(),
            next_loop: 0,
            loop_conditions: Vec::new(),
            retry_loops: std::collections::HashSet::new(),
            retry_region_stack: Vec::new(),
            retry_regions: Vec::new(),
            interrupts: Vec::new(),
            next_interrupt: 0,
            block_depth: 0,
            target: AssignmentTarget::Normal,
            ignore_targets: 0,
            block_scopes: Vec::new(),
            parameter_stack: Vec::new(),
            captures: std::collections::HashMap::new(),
            scope_returns: std::collections::HashMap::new(),
        };
        let root = parsed.node();
        collector
            .scope_returns
            .insert(0, final_assignment_expression_range(&root));
        collector.visit(&root);
        let mut rubocop_referenced_assignments = std::collections::HashSet::new();
        for (read_index, read) in collector.events.iter().enumerate() {
            let compound_reference = matches!(
                read.kind,
                AssignmentEventKind::Write {
                    consumes_previous: true,
                    ..
                }
            );
            if !matches!(read.kind, AssignmentEventKind::Read) && !compound_reference {
                continue;
            }
            let mut consumed_branches = std::collections::HashSet::new();
            for write_index in (0..read_index).rev() {
                let write = &collector.events[write_index];
                if write.scope != read.scope
                    || write.name != read.name
                    || !matches!(write.kind, AssignmentEventKind::Write { .. })
                    || branch_conflict(write, read)
                    || compound_reference && write.offense.start > read.offense.start
                    || write
                        .branches
                        .last()
                        .is_some_and(|branch| consumed_branches.contains(branch))
                {
                    continue;
                }
                rubocop_referenced_assignments.insert(write_index);
                let modifier_assignment = write
                    .branches
                    .last()
                    .is_some_and(|branch| collector.modifier_branches.contains(&branch.0));
                if modifier_assignment {
                    continue;
                }
                if write.branches.last().is_none()
                    || write.branches.last() == read.branches.last()
                {
                    break;
                }
                if let Some(branch) = write
                    .branches
                    .last()
                    .filter(|branch| !collector.incomplete_branches.contains(branch))
                {
                    consumed_branches.insert(*branch);
                }
            }
        }
        collector.candidates.extend(
            collector
                .events
                .iter()
                .map(|event| (event.scope, event.name.clone())),
        );
        for (index, event) in collector.events.iter().enumerate() {
            let AssignmentEventKind::Write {
                correction,
                consumes_previous,
                suffix,
            } = &event.kind
            else {
                continue;
            };
            if event.name.starts_with('_') {
                continue;
            }
            if collector.events[index + 1..]
                .iter()
                .enumerate()
                .any(|(offset, outer)| {
                    let AssignmentEventKind::Write {
                        correction: Some(edit),
                        consumes_previous: false,
                        ..
                    } = &outer.kind
                    else {
                        return false;
                    };
                    let outer_index = index + 1 + offset;
                    !rubocop_referenced_assignments.contains(&outer_index)
                        && outer.scope == event.scope
                        && outer.offense.start < event.offense.start
                        && edit.range.end <= event.offense.start
                        && source[edit.range.end..event.offense.start]
                            .bytes()
                            .all(|byte| matches!(byte, b' ' | b'\t' | b'-' | b'+' | b'!' | b'~'))
                })
            {
                continue;
            }
            if *consumes_previous && !event.loops.is_empty() && !event.branches.is_empty() {
                continue;
            }
            let mut used = rubocop_referenced_assignments.contains(&index);
            used |= collector.retry_regions.iter().any(|(scope, range)| {
                *scope == event.scope
                    && range.start <= event.offense.start
                    && event.offense.end <= range.end
                    && collector.events.iter().any(|other| {
                        other.scope == event.scope
                            && other.name == event.name
                            && range.start <= other.offense.start
                            && other.offense.end <= range.end
                            && (matches!(other.kind, AssignmentEventKind::Read)
                                || matches!(
                                    other.kind,
                                    AssignmentEventKind::Write {
                                        consumes_previous: true,
                                        ..
                                    }
                                ))
                    })
            });
            if let Some(capture_start) = collector
                .captures
                .get(&(event.scope, event.name.clone()))
                .copied()
            {
                let reassigned_before_capture = event.offense.start < capture_start
                    && collector.events[index + 1..].iter().any(|later| {
                        later.scope == event.scope
                            && later.name == event.name
                            && later.offense.start < capture_start
                            && later.branches == event.branches
                            && matches!(later.kind, AssignmentEventKind::Write { .. })
                    });
                if event.offense.start >= capture_start || !reassigned_before_capture {
                    // VariableForce stops proving reassignments once a local
                    // is captured by a block, because the block's invocation
                    // time and frequency are unknowable.
                    used = true;
                }
            }
            if event.block_depth > 0 {
                used |= collector.events[..index].iter().any(|other| {
                    other.scope == event.scope
                        && other.name == event.name
                        && other.block_depth < event.block_depth
                        && other.offense.start < event.offense.start
                        && !branch_conflict(event, other)
                        && matches!(other.kind, AssignmentEventKind::Read)
                });
            }
            if event.block_depth > 0 {
                used |= collector.events[..index].iter().any(|other| {
                    other.scope == event.scope
                        && other.name == event.name
                        && other.block_depth < event.block_depth
                        && !branch_conflict(event, other)
                        && matches!(other.kind, AssignmentEventKind::Write { .. })
                });
            }
            if !event.interrupts.is_empty() {
                for other in collector.events[index + 1..].iter().filter(|other| {
                    other.scope == event.scope
                        && other.name == event.name
                        && event
                            .interrupts
                            .iter()
                            .any(|id| !other.interrupts.contains(id))
                }) {
                    match other.kind {
                        AssignmentEventKind::Read => {
                            used = true;
                            break;
                        }
                        AssignmentEventKind::Write { .. } if other.branches.is_empty() => break,
                        AssignmentEventKind::Write { .. } => {}
                    }
                }
            }
            if event
                .loops
                .iter()
                .any(|id| collector.retry_loops.contains(id))
            {
                used = true;
            }
            used |= event.loops.iter().any(|loop_id| {
                collector.loop_conditions.iter().any(|(id, range)| {
                    id == loop_id
                        && collector.events.iter().any(|other| {
                            other.scope == event.scope
                                && other.name == event.name
                                && range.start <= other.offense.start
                                && other.offense.end <= range.end
                                && matches!(other.kind, AssignmentEventKind::Read)
                        })
                        && !collector.events.iter().any(|other| {
                            range.start <= other.offense.start
                                && other.offense.end <= range.end
                                && matches!(other.kind, AssignmentEventKind::Write { .. })
                        })
                })
            });
            if event.loops.iter().rev().any(|loop_id| {
                let referenced_in_loop = collector.events.iter().any(|other| {
                    other.scope == event.scope
                        && other.name == event.name
                        && other.loops.contains(loop_id)
                        && (matches!(other.kind, AssignmentEventKind::Read)
                            || matches!(
                                other.kind,
                                AssignmentEventKind::Write {
                                    consumes_previous: true,
                                    ..
                                }
                            ))
                });
                let last_assignment_in_loop = !collector.events[index + 1..].iter().any(|later| {
                    later.scope == event.scope
                        && later.name == event.name
                        && later.loops.contains(loop_id)
                        && matches!(later.kind, AssignmentEventKind::Write { .. })
                });
                referenced_in_loop && (!event.branches.is_empty() || last_assignment_in_loop)
            }) {
                used = true;
            }
            if used {
                continue;
            }
            let mut shadowed_branches = Vec::<Vec<(usize, usize)>>::new();
            for later in &collector.events[index + 1..] {
                if later.scope != event.scope {
                    continue;
                }
                if branch_conflict(event, later) {
                    continue;
                }
                if shadowed_branches
                    .iter()
                    .any(|shadow| shadow.iter().all(|branch| later.branches.contains(branch)))
                {
                    continue;
                }
                if later.name != event.name {
                    continue;
                }
                match later.kind {
                    AssignmentEventKind::Read => used = true,
                    AssignmentEventKind::Write {
                        consumes_previous, ..
                    } => {
                        if consumes_previous && later.offense.start >= event.end {
                            used = true;
                        }
                        let conditional_write = later
                            .branches
                            .iter()
                            .any(|branch| !event.branches.iter().any(|own| own.0 == branch.0))
                            || later.target != AssignmentTarget::For
                                && later
                                    .loops
                                    .iter()
                                    .any(|loop_id| !event.loops.contains(loop_id))
                                && !collector.loop_conditions.iter().any(|(_, range)| {
                                    range.start <= later.offense.start
                                        && later.offense.end <= range.end
                                })
                            || later.block_depth > event.block_depth;
                        if later.block_depth > event.block_depth {
                            used = true;
                        }
                        if conditional_write {
                            let shadow = later
                                .branches
                                .iter()
                                .filter(|branch| {
                                    !collector.modifier_branches.contains(&branch.0)
                                        && !event.branches.iter().any(|own| own.0 == branch.0)
                                })
                                .copied()
                                .collect::<Vec<_>>();
                            if !shadow.is_empty() {
                                shadowed_branches.push(shadow);
                            }
                            continue;
                        }
                    }
                }
                break;
            }
            if used {
                continue;
            }
            let suggestion = if suffix.is_empty() && event.target != AssignmentTarget::Multiple {
                did_you_mean_name(
                    &event.name,
                    collector
                        .candidates
                        .iter()
                        .filter(|(scope, _)| *scope == event.scope)
                        .map(|(_, name)| name.as_str()),
                )
                .map(|name| format!(" Did you mean `{name}`?"))
            } else {
                None
            };
            let mut effective_suffix = suggestion.as_deref().unwrap_or(suffix);
            if suffix.contains(" instead of `")
                && collector.scope_returns.get(&event.scope).is_some_and(|range| {
                    range.end != event.end || range.start > event.offense.start
                })
            {
                effective_suffix = "";
            }
            if suffix.starts_with(" Use `||`") {
                let tail = &source[event.end..];
                if tail
                    .lines()
                    .take_while(|line| line.trim() != "end")
                    .any(|line| !line.trim().is_empty())
                {
                    effective_suffix = "";
                }
            }
            let message = format!(
                "Useless assignment to variable - `{}`.{}",
                event.name, effective_suffix
            );
            let line_start = source[..event.offense.start]
                .rfind('\n')
                .map_or(0, |position| position + 1);
            let line_end = source[event.offense.end..]
                .find('\n')
                .map_or(source.len(), |position| event.offense.end + position);
            let line = &source[line_start..line_end];
            let prefix = &source[line_start..event.offense.start];
            let paren_depth = prefix.bytes().fold(0isize, |depth, byte| match byte {
                b'(' => depth + 1,
                b')' => (depth - 1).max(0),
                _ => depth,
            });
            let sequential =
                paren_depth == 0 && line.contains(',') && line.matches('=').count() >= 2;
            if let Some(edit) = correction.clone().filter(|_| !sequential) {
                context.replace(
                    self.name(),
                    message,
                    event.offense.clone(),
                    edit.range,
                    edit.replacement,
                );
            } else if matches!(
                event.target,
                AssignmentTarget::Multiple | AssignmentTarget::For
            ) {
                context.replace(
                    self.name(),
                    message,
                    event.offense.clone(),
                    event.offense.clone(),
                    "_",
                );
            } else {
                context.report(self.name(), message, event.offense.clone());
            }
        }
    }
}

enum AssignmentEventKind {
    Read,
    Write {
        correction: Option<AssignmentCorrection>,
        consumes_previous: bool,
        suffix: String,
    },
}

#[derive(Clone)]
struct AssignmentCorrection {
    range: std::ops::Range<usize>,
    replacement: &'static str,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum AssignmentTarget {
    Normal,
    Multiple,
    For,
    Rescue,
}

struct AssignmentEvent {
    scope: usize,
    name: String,
    offense: std::ops::Range<usize>,
    end: usize,
    target: AssignmentTarget,
    branches: Vec<(usize, usize)>,
    loops: Vec<usize>,
    interrupts: Vec<usize>,
    block_depth: usize,
    kind: AssignmentEventKind,
}

struct AssignmentEventCollector<'s> {
    source: &'s str,
    scope: usize,
    next_scope: usize,
    events: Vec<AssignmentEvent>,
    candidates: Vec<(usize, String)>,
    branches: Vec<(usize, usize)>,
    next_branch: usize,
    modifier_branches: std::collections::HashSet<usize>,
    incomplete_branches: std::collections::HashSet<(usize, usize)>,
    loops: Vec<usize>,
    next_loop: usize,
    loop_conditions: Vec<(usize, std::ops::Range<usize>)>,
    retry_loops: std::collections::HashSet<usize>,
    retry_region_stack: Vec<std::ops::Range<usize>>,
    retry_regions: Vec<(usize, std::ops::Range<usize>)>,
    interrupts: Vec<usize>,
    next_interrupt: usize,
    block_depth: usize,
    target: AssignmentTarget,
    ignore_targets: usize,
    block_scopes: Vec<usize>,
    parameter_stack: Vec<Vec<String>>,
    captures: std::collections::HashMap<(usize, String), usize>,
    scope_returns: std::collections::HashMap<usize, std::ops::Range<usize>>,
}

fn final_assignment_expression_range(node: &Node<'_>) -> std::ops::Range<usize> {
    if let Some(program) = node.as_program_node() {
        return final_assignment_expression_range(&program.statements().as_node());
    }
    if let Some(statements) = node.as_statements_node() {
        if let Some(last) = statements.body().iter().last() {
            return final_assignment_expression_range(&last);
        }
    }
    if let Some(begin) = node.as_begin_node().filter(|begin| {
        begin.begin_keyword_loc().is_none()
            && begin.rescue_clause().is_none()
            && begin.ensure_clause().is_none()
    }) {
        if let Some(statements) = begin.statements() {
            return final_assignment_expression_range(&statements.as_node());
        }
    }
    node.location().start_offset()..node.location().end_offset()
}

fn edit_distance(left: &str, right: &str) -> usize {
    let mut row = (0..=right.len()).collect::<Vec<_>>();
    for (i, a) in left.bytes().enumerate() {
        let mut diagonal = i;
        row[0] = i + 1;
        for (j, b) in right.bytes().enumerate() {
            let above = row[j + 1];
            row[j + 1] = if a == b {
                diagonal
            } else {
                1 + diagonal.min(above).min(row[j])
            };
            diagonal = above;
        }
    }
    row[right.len()]
}

fn did_you_mean_name<'a>(target: &str, names: impl Iterator<Item = &'a str>) -> Option<String> {
    let normalized_target = target.to_ascii_lowercase().replace('@', "");
    let jaro_threshold = if normalized_target.len() > 3 {
        0.834
    } else {
        0.77
    };
    let mut unique = Vec::<String>::new();
    for name in names {
        if name != target && !unique.iter().any(|known| known == name) {
            unique.push(name.to_string());
        }
    }
    let mut words = unique
        .into_iter()
        .filter_map(|word| {
            let normalized = word.to_ascii_lowercase().replace('@', "");
            let score = jaro_winkler_distance(&normalized, &normalized_target);
            (score >= jaro_threshold).then_some((word, normalized, score))
        })
        .collect::<Vec<_>>();
    words.sort_by(|left, right| {
        jaro_winkler_distance(&left.0, &normalized_target)
            .partial_cmp(&jaro_winkler_distance(&right.0, &normalized_target))
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    words.reverse();
    let threshold = normalized_target.len().div_ceil(4);
    if let Some((word, _, _)) = words
        .iter()
        .find(|(_, normalized, _)| edit_distance(normalized, &normalized_target) <= threshold)
    {
        return Some(word.clone());
    }
    words
        .into_iter()
        .find(|(_, normalized, _)| {
            let length = normalized_target.len().min(normalized.len());
            edit_distance(normalized, &normalized_target) < length
        })
        .map(|(word, _, _)| word)
}

fn jaro_winkler_distance(left: &str, right: &str) -> f64 {
    let mut left = left.chars().collect::<Vec<_>>();
    let mut right = right.chars().collect::<Vec<_>>();
    if left.len() > right.len() {
        std::mem::swap(&mut left, &mut right);
    }
    let range = if right.len() > 3 {
        right.len() / 2 - 1
    } else {
        0
    };
    let mut left_match = vec![false; left.len()];
    let mut right_match = vec![false; right.len()];
    let mut matches = 0.0;
    for (index, character) in left.iter().enumerate() {
        let start = index.saturating_sub(range);
        let end = (index + range).min(right.len().saturating_sub(1));
        if start <= end {
            if let Some(other) =
                (start..=end).find(|&other| !right_match[other] && right[other] == *character)
            {
                left_match[index] = true;
                right_match[other] = true;
                matches += 1.0;
            }
        }
    }
    if matches == 0.0 {
        return 0.0;
    }
    let mut right_index = 0;
    let mut transpositions = 0.0;
    for (index, character) in left.iter().enumerate() {
        if !left_match[index] {
            continue;
        }
        while !right_match[right_index] {
            right_index += 1;
        }
        if *character != right[right_index] {
            transpositions += 1.0;
        }
        right_index += 1;
    }
    let transpositions = (transpositions / 2.0_f64).floor();
    let jaro = (matches / left.len() as f64
        + matches / right.len() as f64
        + (matches - transpositions) / matches)
        / 3.0;
    if jaro > 0.7 {
        let prefix = left
            .iter()
            .zip(&right)
            .take(4)
            .take_while(|(a, b)| a == b)
            .count() as f64;
        jaro + prefix * 0.1 * (1.0 - jaro)
    } else {
        jaro
    }
}

fn branch_conflict(left: &AssignmentEvent, right: &AssignmentEvent) -> bool {
    left.branches.iter().any(|branch| {
        right
            .branches
            .iter()
            .any(|other| {
                branch.0 == other.0
                    && branch.1 != other.1
                    && left.interrupts.is_empty()
                    && right.interrupts.is_empty()
            })
    })
}

impl AssignmentEventCollector<'_> {
    fn nested_scope(&mut self, visit: impl FnOnce(&mut Self)) {
        let parent = self.scope;
        let parent_branches = std::mem::take(&mut self.branches);
        let parent_loops = std::mem::take(&mut self.loops);
        let parent_block = self.block_depth;
        let parent_block_scopes = std::mem::take(&mut self.block_scopes);
        self.block_depth = 0;
        self.scope = self.next_scope;
        self.next_scope += 1;
        visit(self);
        self.scope = parent;
        self.branches = parent_branches;
        self.loops = parent_loops;
        self.block_depth = parent_block;
        self.block_scopes = parent_block_scopes;
    }

    fn scope_for_depth(&self, depth: u32) -> usize {
        let depth = depth as usize;
        if depth < self.block_scopes.len() {
            self.block_scopes[self.block_scopes.len() - 1 - depth]
        } else {
            self.scope
        }
    }

    fn read(&mut self, name: &[u8], location: ruby_prism::Location<'_>, depth: u32) {
        let name = String::from_utf8_lossy(name).into_owned();
        let scope = self.scope_for_depth(depth);
        if depth > 0 {
            self.captures
                .entry((scope, name.clone()))
                .or_insert(location.start_offset());
        }
        self.events.push(AssignmentEvent {
            scope,
            name,
            offense: location.start_offset()..location.end_offset(),
            end: location.end_offset(),
            target: AssignmentTarget::Normal,
            branches: self.branches.clone(),
            loops: self.loops.clone(),
            interrupts: self.interrupts.clone(),
            block_depth: self.block_depth,
            kind: AssignmentEventKind::Read,
        });
    }

    fn write(
        &mut self,
        name: &[u8],
        location: ruby_prism::Location<'_>,
        correction: Option<AssignmentCorrection>,
        consumes_previous: bool,
        depth: u32,
    ) {
        let suffix = if consumes_previous {
            String::new()
        } else if self.target == AssignmentTarget::Multiple {
            format!(
                " Use `_` or `_{}` as a variable name to indicate that it won't be used.",
                String::from_utf8_lossy(name)
            )
        } else {
            String::new()
        };
        let name = String::from_utf8_lossy(name).into_owned();
        let scope = self.scope_for_depth(depth);
        if depth > 0 {
            self.captures
                .entry((scope, name.clone()))
                .or_insert(location.start_offset());
        }
        self.events.push(AssignmentEvent {
            scope,
            name,
            offense: location.start_offset()..location.end_offset(),
            end: location.end_offset(),
            target: self.target,
            branches: self.branches.clone(),
            loops: self.loops.clone(),
            interrupts: self.interrupts.clone(),
            block_depth: self.block_depth,
            kind: AssignmentEventKind::Write {
                correction,
                consumes_previous,
                suffix,
            },
        });
    }

    fn with_branch(&mut self, group: usize, arm: usize, visit: impl FnOnce(&mut Self)) {
        self.branches.push((group, arm));
        visit(self);
        self.branches.pop();
    }
    fn with_loop(&mut self, visit: impl FnOnce(&mut Self)) {
        let id = self.next_loop;
        self.next_loop += 1;
        self.loops.push(id);
        visit(self);
        self.loops.pop();
    }
}

impl<'s, 'pr> Visit<'pr> for AssignmentEventCollector<'s> {
    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if node.receiver().is_none()
            && node.arguments().is_none()
            && node.name().as_slice() == b"binding"
        {
            let accessible_scopes = std::iter::once(self.scope)
                .chain(self.block_scopes.iter().copied())
                .collect::<std::collections::HashSet<_>>();
            let variables = self
                .events
                .iter()
                .filter(|event| accessible_scopes.contains(&event.scope))
                .map(|event| (event.scope, event.name.clone()))
                .collect::<std::collections::HashSet<_>>();
            for (scope, name) in variables {
                self.events.push(AssignmentEvent {
                    scope,
                    name,
                    offense: node.location().start_offset()..node.location().end_offset(),
                    end: node.location().end_offset(),
                    target: AssignmentTarget::Normal,
                    // VariableForce resolves an implicit `binding` reference
                    // from the referenced variable's scope. A binding inside
                    // a nested block therefore has no branch identity in the
                    // outer scope, even when that block sits in a rescue body.
                    branches: Vec::new(),
                    loops: self.loops.clone(),
                    interrupts: self.interrupts.clone(),
                    block_depth: self.block_depth,
                    kind: AssignmentEventKind::Read,
                });
            }
        }
        if node.receiver().is_none() && node.arguments().is_none() {
            self.candidates.push((
                self.scope_for_depth(0),
                String::from_utf8_lossy(node.name().as_slice()).into_owned(),
            ));
        }
        ruby_prism::visit_call_node(self, node);
    }
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        if let Some(receiver) = node.receiver() {
            self.visit(&receiver);
        }
        let names = node
            .parameters()
            .map(|parameters| {
                let slice = &self.source
                    [parameters.location().start_offset()..parameters.location().end_offset()];
                slice
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                    .filter(|name| !name.is_empty() && !matches!(*name, "nil" | "true" | "false"))
                    .map(str::to_string)
                    .collect()
            })
            .unwrap_or_default();
        let return_range = node
            .body()
            .map(|body| final_assignment_expression_range(&body));
        self.nested_scope(|this| {
            if let Some(range) = return_range {
                this.scope_returns.insert(this.scope, range);
            }
            this.parameter_stack.push(names);
            if let Some(parameters) = node.parameters() {
                this.visit(&parameters.as_node());
            }
            if let Some(body) = node.body() {
                this.visit(&body);
            }
            this.parameter_stack.pop();
        });
    }
    fn visit_class_node(&mut self, node: &ruby_prism::ClassNode<'pr>) {
        if let Some(superclass) = node.superclass() {
            self.visit(&superclass);
        }
        let return_range = node
            .body()
            .map(|body| final_assignment_expression_range(&body));
        self.nested_scope(|this| {
            if let Some(range) = return_range {
                this.scope_returns.insert(this.scope, range);
            }
            if let Some(body) = node.body() {
                this.visit(&body);
            }
        });
    }
    fn visit_module_node(&mut self, node: &ruby_prism::ModuleNode<'pr>) {
        self.visit(&node.constant_path());
        let return_range = node
            .body()
            .map(|body| final_assignment_expression_range(&body));
        self.nested_scope(|this| {
            if let Some(range) = return_range {
                this.scope_returns.insert(this.scope, range);
            }
            if let Some(body) = node.body() {
                this.visit(&body);
            }
        });
    }
    fn visit_singleton_class_node(&mut self, node: &ruby_prism::SingletonClassNode<'pr>) {
        self.visit(&node.expression());
        let return_range = node
            .body()
            .map(|body| final_assignment_expression_range(&body));
        self.nested_scope(|this| {
            if let Some(range) = return_range {
                this.scope_returns.insert(this.scope, range);
            }
            if let Some(body) = node.body() {
                this.visit(&body);
            }
        });
    }
    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
        let group = self.next_branch;
        self.next_branch += 1;
        let modifier = node
            .if_keyword_loc()
            .is_some_and(|keyword| keyword.start_offset() != node.location().start_offset());
        if modifier {
            self.modifier_branches.insert(group);
            self.with_branch(group, 0, |this| {
                this.visit(&node.predicate());
                if let Some(statements) = node.statements() {
                    this.visit(&statements.as_node());
                }
            });
            if let Some(subsequent) = node.subsequent() {
                self.with_branch(group, 1, |this| this.visit(&subsequent));
            }
            return;
        }
        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            self.with_branch(group, 0, |this| this.visit(&statements.as_node()));
        }
        if let Some(subsequent) = node.subsequent() {
            self.with_branch(group, 1, |this| this.visit(&subsequent));
        }
    }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
        let group = self.next_branch;
        self.next_branch += 1;
        let modifier = node.keyword_loc().start_offset() != node.location().start_offset();
        if modifier {
            self.modifier_branches.insert(group);
            self.with_branch(group, 0, |this| {
                this.visit(&node.predicate());
                if let Some(statements) = node.statements() {
                    this.visit(&statements.as_node());
                }
            });
            if let Some(clause) = node.else_clause() {
                self.with_branch(group, 1, |this| this.visit(&clause.as_node()));
            }
            return;
        }
        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            self.with_branch(group, 0, |this| this.visit(&statements.as_node()));
        }
        if let Some(clause) = node.else_clause() {
            self.with_branch(group, 1, |this| this.visit(&clause.as_node()));
        }
    }
    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode<'pr>) {
        self.loop_conditions.push((
            self.next_loop,
            node.predicate().location().start_offset()..node.predicate().location().end_offset(),
        ));
        let modifier = node.keyword_loc().start_offset() != node.location().start_offset();
        if modifier {
            let group = self.next_branch;
            self.next_branch += 1;
            self.modifier_branches.insert(group);
            self.with_branch(group, 0, |this| {
                this.with_loop(|this| ruby_prism::visit_while_node(this, node))
            });
        } else {
            self.with_loop(|this| ruby_prism::visit_while_node(this, node));
        }
    }
    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode<'pr>) {
        self.loop_conditions.push((
            self.next_loop,
            node.predicate().location().start_offset()..node.predicate().location().end_offset(),
        ));
        let modifier = node.keyword_loc().start_offset() != node.location().start_offset();
        if modifier {
            let group = self.next_branch;
            self.next_branch += 1;
            self.modifier_branches.insert(group);
            self.with_branch(group, 0, |this| {
                this.with_loop(|this| ruby_prism::visit_until_node(this, node))
            });
        } else {
            self.with_loop(|this| ruby_prism::visit_until_node(this, node));
        }
    }
    fn visit_for_node(&mut self, node: &ruby_prism::ForNode<'pr>) {
        self.visit(&node.collection());
        self.with_loop(|this| {
            let old = this.target;
            this.target = AssignmentTarget::For;
            this.visit(&node.index());
            this.target = old;
            if let Some(statements) = node.statements() {
                this.visit(&statements.as_node());
            }
        });
    }
    fn visit_and_node(&mut self, node: &ruby_prism::AndNode<'pr>) {
        self.visit(&node.left());
        let group = self.next_branch;
        self.next_branch += 1;
        self.with_branch(group, 0, |this| this.visit(&node.right()));
    }
    fn visit_or_node(&mut self, node: &ruby_prism::OrNode<'pr>) {
        self.visit(&node.left());
        let group = self.next_branch;
        self.next_branch += 1;
        self.with_branch(group, 0, |this| this.visit(&node.right()));
    }
    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode<'pr>) {
        if let Some(predicate) = node.predicate() {
            self.visit(&predicate);
        }
        let group = self.next_branch;
        self.next_branch += 1;
        for (arm, condition) in node.conditions().iter().enumerate() {
            self.with_branch(group, arm, |this| this.visit(&condition));
        }
        if let Some(otherwise) = node.else_clause() {
            self.with_branch(group, node.conditions().len(), |this| {
                this.visit(&otherwise.as_node())
            });
        }
    }
    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode<'pr>) {
        if let Some(predicate) = node.predicate() {
            self.visit(&predicate);
        }
        let group = self.next_branch;
        self.next_branch += 1;
        for (arm, condition) in node.conditions().iter().enumerate() {
            self.with_branch(group, arm, |this| this.visit(&condition));
        }
        if let Some(otherwise) = node.else_clause() {
            self.with_branch(group, node.conditions().len(), |this| {
                this.visit(&otherwise.as_node())
            });
        }
    }
    fn visit_begin_node(&mut self, node: &ruby_prism::BeginNode<'pr>) {
        if node.rescue_clause().is_none() && node.ensure_clause().is_none() {
            ruby_prism::visit_begin_node(self, node);
            return;
        }
        self.retry_region_stack
            .push(node.location().start_offset()..node.location().end_offset());
        let group = self.next_branch;
        self.next_branch += 1;
        self.incomplete_branches.insert((group, 0));
        let interrupt = self.next_interrupt;
        self.next_interrupt += 1;
        if let Some(statements) = node.statements() {
            self.with_branch(group, 0, |this| {
                this.interrupts.push(interrupt);
                this.visit(&statements.as_node());
                this.interrupts.pop();
            });
        }
        let mut rescue = node.rescue_clause();
        let mut arm = 1;
        while let Some(clause) = rescue {
            self.with_branch(group, arm, |this| {
                for exception in clause.exceptions().iter() {
                    this.visit(&exception);
                }
                if let Some(reference) = clause.reference() {
                    if let Some(target) = reference.as_local_variable_target_node() {
                        let old = this.target;
                        this.target = AssignmentTarget::Rescue;
                        let start = clause
                            .operator_loc()
                            .map_or(target.location().start_offset(), |operator| {
                                operator.start_offset().saturating_sub(1)
                            });
                        this.write(
                            target.name().as_slice(),
                            target.location(),
                            Some(AssignmentCorrection {
                                range: start..target.location().end_offset(),
                                replacement: "",
                            }),
                            false,
                            target.depth(),
                        );
                        this.target = old;
                    } else {
                        this.visit(&reference);
                    }
                }
                if let Some(statements) = clause.statements() {
                    this.visit(&statements.as_node());
                }
            });
            rescue = clause.subsequent();
            arm += 1;
        }
        if let Some(otherwise) = node.else_clause() {
            self.with_branch(group, arm, |this| this.visit(&otherwise.as_node()));
        }
        if let Some(ensure) = node.ensure_clause() {
            self.visit(&ensure.as_node());
        }
        self.retry_region_stack.pop();
    }
    fn visit_in_node(&mut self, node: &ruby_prism::InNode<'pr>) {
        self.ignore_targets += 1;
        self.visit(&node.pattern());
        self.ignore_targets -= 1;
        if let Some(statements) = node.statements() {
            self.visit(&statements.as_node());
        }
    }
    fn visit_match_predicate_node(&mut self, node: &ruby_prism::MatchPredicateNode<'pr>) {
        self.visit(&node.value());
        self.ignore_targets += 1;
        self.visit(&node.pattern());
        self.ignore_targets -= 1;
    }
    fn visit_match_required_node(&mut self, node: &ruby_prism::MatchRequiredNode<'pr>) {
        self.visit(&node.value());
        self.ignore_targets += 1;
        self.visit(&node.pattern());
        self.ignore_targets -= 1;
    }
    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode<'pr>) {
        self.visit(&node.value());
        let old = self.target;
        self.target = AssignmentTarget::Multiple;
        for target in node.lefts().iter() {
            self.visit(&target);
        }
        if let Some(target) = node.rest() {
            self.visit(&target);
        }
        for target in node.rights().iter() {
            self.visit(&target);
        }
        self.target = old;
    }
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        let scope = self.next_scope;
        self.next_scope += 1;
        if let Some(range) = node
            .body()
            .map(|body| final_assignment_expression_range(&body))
        {
            self.scope_returns.insert(scope, range);
        }
        self.block_scopes.push(scope);
        if let Some(parameters) = node.parameters() {
            self.visit(&parameters);
        }
        self.block_depth += 1;
        if let Some(body) = node.body() {
            self.visit(&body);
        }
        self.block_depth -= 1;
        self.block_scopes.pop();
    }

    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
        let scope = self.next_scope;
        self.next_scope += 1;
        if let Some(range) = node
            .body()
            .map(|body| final_assignment_expression_range(&body))
        {
            self.scope_returns.insert(scope, range);
        }
        self.block_scopes.push(scope);
        self.block_depth += 1;
        ruby_prism::visit_lambda_node(self, node);
        self.block_depth -= 1;
        self.block_scopes.pop();
    }

    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        self.read(node.name().as_slice(), node.location(), node.depth());
    }
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.visit(&node.value());
        self.write(
            node.name().as_slice(),
            node.name_loc(),
            Some(AssignmentCorrection {
                range: node.name_loc().start_offset()..node.value().location().start_offset(),
                replacement: "",
            }),
            false,
            node.depth(),
        );
    }
    fn visit_local_variable_operator_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        let group = self.next_branch;
        self.next_branch += 1;
        self.with_branch(group, 0, |this| this.visit(&node.value()));
        let operator = String::from_utf8_lossy(node.binary_operator_loc().as_slice())
            .trim_end_matches('=')
            .to_string();
        let suffix = format!(" Use `{operator}` instead of `{operator}=`.");
        let scope = self.scope_for_depth(node.depth());
        let name = String::from_utf8_lossy(node.name().as_slice()).into_owned();
        if node.depth() > 0 {
            self.captures
                .entry((scope, name.clone()))
                .or_insert(node.name_loc().start_offset());
        }
        self.events.push(AssignmentEvent {
            scope,
            name,
            offense: node.name_loc().start_offset()..node.name_loc().end_offset(),
            end: node.location().end_offset(),
            target: AssignmentTarget::Normal,
            branches: self.branches.clone(),
            loops: self.loops.clone(),
            interrupts: self.interrupts.clone(),
            block_depth: self.block_depth,
            kind: AssignmentEventKind::Write {
                correction: Some(AssignmentCorrection {
                    range: node.binary_operator_loc().start_offset()
                        ..node.binary_operator_loc().end_offset(),
                    replacement: match operator.as_str() {
                        "+" => "+",
                        "-" => "-",
                        "*" => "*",
                        "/" => "/",
                        "%" => "%",
                        "**" => "**",
                        "&" => "&",
                        "|" => "|",
                        "^" => "^",
                        "<<" => "<<",
                        ">>" => ">>",
                        _ => "",
                    },
                }),
                consumes_previous: true,
                suffix,
            },
        });
    }
    fn visit_local_variable_or_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        let group = self.next_branch;
        self.next_branch += 1;
        self.with_branch(group, 0, |this| this.visit(&node.value()));
        let scope = self.scope_for_depth(node.depth());
        let name = String::from_utf8_lossy(node.name().as_slice()).into_owned();
        if node.depth() > 0 {
            self.captures
                .entry((scope, name.clone()))
                .or_insert(node.name_loc().start_offset());
        }
        self.events.push(AssignmentEvent {
            scope,
            name,
            offense: node.name_loc().start_offset()..node.name_loc().end_offset(),
            end: node.location().end_offset(),
            target: AssignmentTarget::Normal,
            branches: self.branches.clone(),
            loops: self.loops.clone(),
            interrupts: self.interrupts.clone(),
            block_depth: self.block_depth,
            kind: AssignmentEventKind::Write {
                correction: None,
                consumes_previous: true,
                suffix: " Use `||` instead of `||=`.".into(),
            },
        });
    }
    fn visit_local_variable_and_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        let group = self.next_branch;
        self.next_branch += 1;
        self.with_branch(group, 0, |this| this.visit(&node.value()));
        let scope = self.scope_for_depth(node.depth());
        let name = String::from_utf8_lossy(node.name().as_slice()).into_owned();
        if node.depth() > 0 {
            self.captures
                .entry((scope, name.clone()))
                .or_insert(node.name_loc().start_offset());
        }
        self.events.push(AssignmentEvent {
            scope,
            name,
            offense: node.name_loc().start_offset()..node.name_loc().end_offset(),
            end: node.location().end_offset(),
            target: AssignmentTarget::Normal,
            branches: self.branches.clone(),
            loops: self.loops.clone(),
            interrupts: self.interrupts.clone(),
            block_depth: self.block_depth,
            kind: AssignmentEventKind::Write {
                correction: None,
                consumes_previous: true,
                suffix: " Use `&&` instead of `&&=`.".into(),
            },
        });
    }
    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        if self.ignore_targets == 0 {
            self.write(
                node.name().as_slice(),
                node.location(),
                None,
                false,
                node.depth(),
            );
        } else {
            self.read(node.name().as_slice(), node.location(), node.depth());
        }
    }
    fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode<'pr>) {
        if let Some(names) = self.parameter_stack.last().cloned() {
            for name in names {
                self.read(
                    name.as_bytes(),
                    node.location(),
                    self.block_scopes.len() as u32,
                );
            }
        }
        ruby_prism::visit_forwarding_super_node(self, node);
    }
    fn visit_match_write_node(&mut self, node: &ruby_prism::MatchWriteNode<'pr>) {
        let call = node.call();
        if let Some(arguments) = call.arguments() {
            self.visit(&arguments.as_node());
        }
        if let Some(receiver) = call.receiver() {
            let offense = receiver.location().start_offset()..receiver.location().end_offset();
            let receiver_source = &self.source[offense.clone()];
            for target in node.targets().iter() {
                if let Some(local) = target.as_local_variable_target_node() {
                    let name = String::from_utf8_lossy(local.name().as_slice());
                    let needle = format!("(?<{name}>");
                    if let Some(position) = receiver_source.find(&needle) {
                        let start = offense.start + position + 1;
                        let end = start + needle.len() - 1;
                        let correction = AssignmentCorrection {
                            range: start..end,
                            replacement: "?:",
                        };
                        let saved = self.target;
                        self.target = AssignmentTarget::Normal;
                        self.write(
                            local.name().as_slice(),
                            receiver.location(),
                            Some(correction),
                            false,
                            local.depth(),
                        );
                        if let Some(event) = self.events.last_mut() {
                            event.offense = offense.clone();
                        }
                        self.target = saved;
                    }
                }
            }
        }
    }
    fn visit_retry_node(&mut self, _node: &ruby_prism::RetryNode<'pr>) {
        for id in &self.loops {
            self.retry_loops.insert(*id);
        }
        if let Some(range) = self.retry_region_stack.last().cloned() {
            self.retry_regions
                .push((self.scope_for_depth(0), range));
        }
    }
}

struct MethodName;

impl Cop for MethodName {
    fn name(&self) -> &'static str {
        "Naming/MethodName"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let mut context = context.cop_context(self.name(), source, ancestors);
        if let Some(definition) = node.as_def_node() {
            let definition_name = String::from_utf8_lossy(definition.name().as_slice());
            if definition.receiver().is_some()
                && definition_name
                    .bytes()
                    .next()
                    .is_some_and(|byte| byte.is_ascii_uppercase())
                && source.lines().any(|line| {
                    line.trim_start()
                        .strip_prefix("class ")
                        .is_some_and(|class_name| {
                            class_name.split_whitespace().next() == Some(definition_name.as_ref())
                        })
                })
            {
                return;
            }
            check_method_identifier(
                definition.name().as_slice(),
                definition.name_loc(),
                &mut context,
            );
            return;
        }
        if let Some(alias) = node.as_alias_method_node() {
            let new_name = alias.new_name();
            if let Some(name) = method_name_literal(&new_name) {
                check_method_identifier(name.as_bytes(), new_name.location(), &mut context);
            }
            return;
        }
        let Some(call) = node.as_call_node() else {
            return;
        };
        let arguments = call
            .arguments()
            .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
            .unwrap_or_default();
        if matches!(
            call_name(&call),
            b"attr" | b"attr_reader" | b"attr_writer" | b"attr_accessor"
        ) && call.receiver().is_none()
        {
            let issue = arguments.iter().find_map(|argument| {
                method_name_literal(argument)
                    .and_then(|name| method_identifier_message(&name, &context))
            });
            if let (Some(message), Some(first), Some(last)) =
                (issue, arguments.first(), arguments.last())
            {
                context.report(
                    message,
                    first.location().start_offset()..last.location().end_offset(),
                );
            }
            return;
        }
        let selected: Vec<Node<'_>> = match call_name(&call) {
            b"define_method" | b"define_singleton_method" if call.receiver().is_none() => {
                arguments.into_iter().take(1).collect()
            }
            b"alias_method" if call.receiver().is_none() && arguments.len() == 2 => {
                arguments.into_iter().take(1).collect()
            }
            b"new" if method_name_constant_receiver(&call, b"Struct") => {
                let skip = arguments
                    .first()
                    .is_some_and(|argument| argument.as_string_node().is_some());
                arguments.into_iter().skip(usize::from(skip)).collect()
            }
            b"define" if method_name_constant_receiver(&call, b"Data") => arguments,
            _ => return,
        };
        for argument in selected {
            if let Some(name) = method_name_literal(&argument) {
                if let Some(message) = method_identifier_message(&name, &context) {
                    let location = argument.location();
                    let start = location.start_offset();
                    let mut end = location.end_offset();
                    let quoted_symbol = source[start..end].starts_with(":'")
                        || source[start..end].starts_with(":\"");
                    if quoted_symbol
                        && source
                            .as_bytes()
                            .get(end)
                            .is_some_and(|byte| matches!(byte, b'\'' | b'\"' | b')'))
                    {
                        end += 1;
                    }
                    context.report(message, start..end);
                }
            }
        }
    }
}

fn method_name_literal(node: &Node<'_>) -> Option<String> {
    if let Some(symbol) = node.as_symbol_node() {
        Some(String::from_utf8_lossy(symbol.unescaped()).into_owned())
    } else {
        node.as_string_node()
            .map(|string| String::from_utf8_lossy(string.unescaped()).into_owned())
    }
}

fn method_name_constant_receiver(call: &CallNode<'_>, expected: &[u8]) -> bool {
    call.receiver().is_some_and(|receiver| {
        receiver
            .as_constant_read_node()
            .is_some_and(|constant| constant.name().as_slice() == expected)
            || receiver
                .as_constant_path_node()
                .is_some_and(|path| path.name().is_some_and(|name| name.as_slice() == expected))
    })
}

fn check_method_identifier(
    name: &[u8],
    location: ruby_prism::Location<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let Ok(name) = std::str::from_utf8(name) else {
        return;
    };
    if let Some(message) = method_identifier_message(name, context) {
        context.report(message, location);
    }
}

fn method_identifier_message(name: &str, context: &CopContext<'_, '_>) -> Option<String> {
    if matches!(
        name,
        "|" | "^"
            | "&"
            | "<=>"
            | "=="
            | "==="
            | "=~"
            | ">"
            | ">="
            | "<"
            | "<="
            | "<<"
            | ">>"
            | "+"
            | "-"
            | "*"
            | "/"
            | "%"
            | "**"
            | "~"
            | "+@"
            | "-@"
            | "!@"
            | "~@"
            | "[]"
            | "[]="
            | "!"
            | "!="
            | "!~"
            | "`"
    ) {
        return None;
    }
    if context.policy().allows_method(name.as_bytes()) {
        return None;
    }
    if context
        .config_values("AllowedPatterns")
        .iter()
        .any(|pattern| {
            let pattern = pattern.replace("\\A", "^").replace("\\z", "$");
            regex::Regex::new(&pattern).is_ok_and(|pattern| pattern.is_match(name))
        })
    {
        return None;
    }
    let forbidden = context
        .config_values("ForbiddenIdentifiers")
        .iter()
        .any(|identifier| identifier == name)
        || context
            .config_values("ForbiddenPatterns")
            .iter()
            .any(|pattern| regex::Regex::new(pattern).is_ok_and(|pattern| pattern.is_match(name)));
    if forbidden {
        return Some(format!(
            "`{name}` is forbidden, use another method name instead."
        ));
    }
    let style = context.policy().enforced_style("snake_case");
    let core = name.trim_end_matches(['?', '!', '=']);
    let mut characters = core.chars();
    let first = characters.next();
    let invalid = if style == "camelCase" {
        first.is_none_or(|character| {
            if character.is_ascii() {
                !character.is_ascii_lowercase()
            } else {
                !character.is_alphabetic()
            }
        }) || characters.any(|character| {
            if character.is_ascii() {
                !character.is_ascii_alphanumeric()
            } else {
                !character.is_alphanumeric()
            }
        })
    } else {
        first.is_none_or(|character| {
            if character.is_ascii() {
                !character.is_ascii_lowercase() && character != '_'
            } else {
                !character.is_alphabetic()
            }
        }) || characters.any(|character| {
            if character.is_ascii() {
                !character.is_ascii_lowercase() && !character.is_ascii_digit() && character != '_'
            } else {
                !character.is_alphanumeric()
            }
        })
    };
    if invalid {
        Some(format!("Use {style} for method names."))
    } else {
        None
    }
}
