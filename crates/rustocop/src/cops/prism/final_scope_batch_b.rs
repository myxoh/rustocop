use super::catalog_cop::{custom, report};
use super::*;
use std::collections::{HashMap, HashSet};

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
        custom("Style/MutableConstant", mutable_constant),
        custom("Naming/PredicateMethod", predicate_method),
    ]
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
        let (name, operator, value, variable) = if let Some(write) = node.as_local_variable_write_node() {
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
                (operator.start_offset()..operator.start_offset(), shorthand.to_string()),
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
        let shorthand = OPERATORS
            .iter()
            .find_map(|(method, shorthand)| (call.name().as_slice() == *method).then_some(*shorthand))?;
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
        if let Some(definition) = line.trim_start().strip_prefix("def ") {
            method = Some(
                definition
                    .split(['(', ' '])
                    .next()
                    .unwrap_or("")
                    .rsplit('.')
                    .next()
                    .unwrap_or("")
                    .trim_start_matches('_')
                    .trim_end_matches(['?', '!'])
                    .to_string(),
            );
        }
        if let Some(at) = line.find("@") {
            if line[at..].contains("||=")
                && index + 1 < lines.len()
                && lines[index + 1].1.trim() == "end"
            {
                let name = line[at + 1..]
                    .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
                    .next()
                    .unwrap_or("");
                let normalized = name.trim_start_matches('_');
                if method
                    .as_deref()
                    .is_some_and(|method| !method.starts_with("initialize") && method != normalized)
                {
                    context.report(
                        "Memoized variable name should match the method name.",
                        offset + at..offset + at + name.len() + 1,
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
    let Some(file) = std::path::Path::new(context.path())
        .file_name()
        .and_then(|name| name.to_str())
    else {
        return;
    };
    if file.ends_with(".rb")
        && file
            .bytes()
            .any(|byte| byte.is_ascii_uppercase() || byte == b'-')
    {
        context.report("The name of this source file should use snake_case.", 0..0);
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
    if context.policy().enforced_style("snake_case") != "snake_case"
        || !context.config_values("AllowedPatterns").is_empty()
    {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let Some((name, _)) = line.split_once(" = ") else {
            continue;
        };
        let name = name.trim();
        let bare = name.trim_start_matches(['@', '$']);
        if bare
            .bytes()
            .next()
            .is_some_and(|byte| byte.is_ascii_uppercase())
            || context
                .config_values("AllowedIdentifiers")
                .iter()
                .any(|allowed| allowed == bare)
        {
            continue;
        }
        if name.bytes().any(|byte| byte.is_ascii_uppercase())
            && name.bytes().any(|byte| byte.is_ascii_lowercase())
        {
            let start = offset + line.find(name).unwrap_or(0);
            context.report(
                "Use snake_case for variable names.",
                start..start + name.len(),
            );
        }
    }
}

fn useless_assignment(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    let mut assignments = HashMap::<String, (usize, usize)>::new();
    for (offset, line) in context.source_file().lines() {
        if let Some((name, _)) = line.split_once(" = ") {
            let name = name.trim();
            if !name.is_empty() {
                assignments.insert(
                    name.to_string(),
                    (offset + line.find(name).unwrap_or(0), name.len()),
                );
            }
        }
    }
    for (name, (start, len)) in assignments {
        if name.starts_with("unused") && source.match_indices(&name).count() == 1 {
            context.report("Useless assignment to variable.", start..start + len);
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

fn mutable_constant(context: &mut CopContext<'_, '_>) {
    if context.source().contains("shareable_constant_value")
        || context.source().contains("frozen_string_literal: true")
    {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let Some((name, value)) = line.split_once(" = ") else {
            continue;
        };
        let name = name.trim();
        if name
            .bytes()
            .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            && value.trim_start().starts_with(['[', '{', '"', '\''])
            && !value.contains(".freeze")
            && !value.contains(" + ")
            && !value.contains(".count")
            && !value.contains(".length")
            && !value.contains(".size")
        {
            context.insert(
                "Freeze mutable objects assigned to constants.",
                offset..offset + line.len(),
                offset + line.len(),
                ".freeze",
            );
        }
    }
}

fn predicate_method(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let predicates = HashSet::from(["is_", "has_", "does_"]);
    for (offset, line) in lines {
        let Some(definition) = line.trim_start().strip_prefix("def ") else {
            continue;
        };
        let name = definition.split(['(', ' ']).next().unwrap_or("");
        if predicates.iter().any(|prefix| name.starts_with(prefix)) && !name.ends_with('?') {
            let start = offset + line.find(name).unwrap_or(0);
            context.insert(
                "Predicate method names should end with `?`.",
                start..start + name.len(),
                start + name.len(),
                "?",
            );
        }
    }
}
