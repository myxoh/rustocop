use super::catalog_cop::{custom, report};
use super::*;
use std::collections::HashSet;

mod naming;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        custom("Lint/ShadowedException", shadowed_exception),
        custom("Lint/ConstantDefinitionInBlock", constant_in_block),
        custom("Lint/ShadowingOuterLocalVariable", shadowing_outer_local),
        report(
            "Lint/LiteralAssignmentInCondition",
            "if value = 1",
            "Do not use a literal assignment in a condition.",
        ),
        Box::new(HeredocDelimiterCase) as Box<dyn Cop>,
        Box::new(BlockForwarding) as Box<dyn Cop>,
        custom("Lint/AmbiguousAssignment", ambiguous_assignment),
        Box::new(RescuedExceptionsVariableName) as Box<dyn Cop>,
        custom("Lint/ConstantReassignment", constant_reassignment),
    ];
    cops.extend(naming::cops());
    cops
}

define_any_node_cop!(HeredocDelimiterCase => "Naming/HeredocDelimiterCase" => heredoc_case);
define_node_cop!(BlockForwarding => "Naming/BlockForwarding" => as_def_node => block_forwarding);
define_node_cop!(RescuedExceptionsVariableName => "Naming/RescuedExceptionsVariableName" => as_rescue_node => rescued_exception_name);

fn ambiguous_assignment(context: &mut CopContext<'_, '_>) {
    for (needle, operator) in [("=-", "-"), ("=+", "+"), ("=*", "*"), ("=!", "!")] {
        for start in context.source_file().code_offsets(needle) {
            context.report(
                format!("Suspicious assignment detected. Did you mean `{operator}=`?"),
                start..start + needle.len(),
            );
        }
    }
}

fn shadowed_exception(context: &mut CopContext<'_, '_>) {
    let mut rescued = None::<String>;
    for (offset, line) in context.source_file().lines() {
        if let Some((_, name)) = line.split_once("rescue => ") {
            rescued = Some(name.trim().to_string());
        } else if let Some(name) = &rescued {
            if let Some(at) = line.find(&format!("{name} =")) {
                context.report(
                    "Rescued exception variable is overwritten.",
                    offset + at..offset + at + name.len(),
                );
            }
        }
        if line.trim() == "end" {
            rescued = None;
        }
    }
}

fn constant_in_block(context: &mut CopContext<'_, '_>) {
    let mut block_depth = 0;
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if trimmed.contains(" do") || trimmed.ends_with('{') {
            block_depth += 1;
        }
        if block_depth > 0 {
            let name = trimmed.split('=').next().unwrap_or("").trim();
            if !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            {
                let start = offset + line.find(name).unwrap_or(0);
                context.report(
                    "Do not define constants inside a block.",
                    start..start + name.len(),
                );
            }
        }
        if trimmed == "end" && block_depth > 0 {
            block_depth -= 1;
        }
    }
}

fn shadowing_outer_local(context: &mut CopContext<'_, '_>) {
    let mut locals = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim_start();
        if ["def ", "class ", "module "]
            .iter()
            .any(|prefix| trimmed.starts_with(prefix))
            || trimmed == "end"
        {
            locals.clear();
        }
        if let Some((name, _)) = line.split_once(" = ") {
            let name = name.trim();
            if !name.is_empty()
                && name
                    .bytes()
                    .all(|byte| byte.is_ascii_lowercase() || byte == b'_')
                && !line.contains(&format!("|{name}|"))
                && !line.contains(&format!("|{name},"))
                && !line.contains(&format!(", {name}|"))
            {
                locals.clear();
                locals.insert(name.to_string());
            }
        }
        if let Some(first) = line.find('|') {
            if let Some(close) = line[first + 1..].find('|').map(|at| first + 1 + at) {
                for argument in line[first + 1..close].split(',').map(str::trim) {
                    if locals.contains(argument) {
                        let start =
                            offset + first + 1 + line[first + 1..close].find(argument).unwrap_or(0);
                        context.report(
                            "Shadowing outer local variable.",
                            start..start + argument.len(),
                        );
                    }
                }
            }
        }
        if !trimmed.is_empty()
            && !line.contains(" = ")
            && !line.contains('|')
            && !trimmed.starts_with('#')
        {
            locals.clear();
        }
    }
}

fn heredoc_case(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let uppercase = context.policy().enforced_style("uppercase") == "uppercase";
    let locations = if let Some(string) = node.as_string_node() {
        string.opening_loc().zip(string.closing_loc())
    } else if let Some(string) = node.as_interpolated_string_node() {
        string.opening_loc().zip(string.closing_loc())
    } else if let Some(string) = node.as_x_string_node() {
        Some((string.opening_loc(), string.closing_loc()))
    } else {
        node.as_interpolated_x_string_node()
            .map(|string| (string.opening_loc(), string.closing_loc()))
    };
    let Some((opening, closing)) = locations else {
        return;
    };
    if !opening.as_slice().starts_with(b"<<") {
        return;
    }
    let closing_source = String::from_utf8_lossy(closing.as_slice());
    let delimiter = closing_source.trim();
    let wrong_case = if uppercase {
        delimiter.bytes().any(|byte| byte.is_ascii_lowercase())
    } else {
        delimiter.bytes().any(|byte| byte.is_ascii_uppercase())
    };
    if delimiter.is_empty() || !wrong_case {
        return;
    }
    let replacement = if uppercase {
        delimiter.to_ascii_uppercase()
    } else {
        delimiter.to_ascii_lowercase()
    };
    let opening_source = String::from_utf8_lossy(opening.as_slice());
    let Some(relative) = opening_source.rfind(delimiter) else {
        return;
    };
    let opening_range = opening.start_offset() + relative..opening.start_offset() + relative + delimiter.len();
    let closing_end = closing.end_offset()
        - closing
            .as_slice()
            .iter()
            .rev()
            .take_while(|byte| matches!(byte, b'\n' | b'\r'))
            .count();
    let closing_range = closing.start_offset()..closing_end;
    let closing_edit = closing_end.saturating_sub(delimiter.len())..closing_end;
    context.replace_many(
        if uppercase {
            "Use uppercase heredoc delimiters."
        } else {
            "Use lowercase heredoc delimiters."
        },
        closing_range.clone(),
        vec![
            (opening_range, replacement.clone()),
            (closing_edit, replacement),
        ],
    );
}

fn rescued_exception_name(
    node: &ruby_prism::RescueNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let preferred = context
        .config_value("PreferredName")
        .unwrap_or("e")
        .to_string();
    let Some(reference) = node.reference() else {
        return;
    };
    if context.ancestors().iter().any(|ancestor| {
        ancestor.as_rescue_node().is_some_and(|rescue| {
            rescue.statements().is_some_and(|statements| {
                statements.location().start_offset() <= node.keyword_loc().start_offset()
                    && node.keyword_loc().start_offset() < statements.location().end_offset()
            })
        })
    }) {
        return;
    }
    let range = reference.location().start_offset()..reference.location().end_offset();
    let actual = context.source_file().slice(range.clone()).unwrap_or_default();
    if actual.is_empty() || actual.contains('.') {
        return;
    }
    let expected = if actual.starts_with('_') && !preferred.starts_with('_') {
        format!("_{preferred}")
    } else {
        preferred
    };
    let rescue_start = node.keyword_loc().start_offset();
    let assignment_scope_end = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_begin_node)
        .and_then(|begin| begin.begin_keyword_loc())
        .map(|keyword| keyword.start_offset())
        .or_else(|| {
            context
                .ancestors()
                .iter()
                .filter_map(Node::as_rescue_node)
                .map(|rescue| rescue.keyword_loc().start_offset())
                .min()
        })
        .unwrap_or(rescue_start);
    if identifier_assigned(&context.source()[..assignment_scope_end], &expected) {
        return;
    }
    if node.statements().is_some_and(|statements| {
        identifier_assigned(context.source_file().node(&statements.as_node()), &expected)
    }) {
        return;
    }
    if actual != expected {
        context.replace(
            format!("Use `{expected}` instead of `{actual}`."),
            range.clone(),
            range,
            expected,
        );
    }
}

fn identifier_assigned(source: &str, name: &str) -> bool {
    source.match_indices(name).any(|(start, _)| {
        let before = source[..start].bytes().next_back();
        let after = source[start + name.len()..].trim_start();
        !before.is_some_and(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
            && after.starts_with('=')
            && !after.starts_with("==")
            && !after.starts_with("=>")
    })
}

fn block_forwarding(
    definition: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let style = context.policy().enforced_style("anonymous");
    if style == "explicit" {
        explicit_block_forwarding(definition, context);
        return;
    }
    if style != "anonymous" {
        return;
    }
    if !context.target_ruby_version().at_least(3, 1) {
        return;
    }
    let Some(parameter) = definition.parameters().and_then(|parameters| parameters.block()) else {
        return;
    };
    let Some(name) = parameter.name() else { return };
    if definition
        .parameters()
        .is_some_and(|parameters| !parameters.keywords().is_empty())
    {
        return;
    }
    let mut usage = BlockForwardingUsage {
        name: name.as_slice(),
        forwarded: Vec::new(),
        other_use: false,
        nested_block_depth: 0,
        allow_nested: context.target_ruby_version().at_least(3, 4),
    };
    if let Some(body) = definition.body() {
        ruby_prism::Visit::visit(&mut usage, &body);
    }
    if usage.other_use {
        return;
    }
    let range = parameter.location().start_offset()..parameter.location().end_offset();
    for forwarded in usage.forwarded {
        context.replace(
            "Use anonymous block forwarding.",
            forwarded.clone(),
            forwarded,
            "&",
        );
    }
    context.replace(
        "Use anonymous block forwarding.",
        range.clone(),
        range,
        "&",
    );
}

struct BlockForwardingUsage<'a> {
    name: &'a [u8],
    forwarded: Vec<std::ops::Range<usize>>,
    other_use: bool,
    nested_block_depth: usize,
    allow_nested: bool,
}

impl<'pr> ruby_prism::Visit<'pr> for BlockForwardingUsage<'_> {
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode<'pr>) {
        if node.expression().and_then(|value| value.as_local_variable_read_node())
            .is_some_and(|read| read.name().as_slice() == self.name)
        {
            if self.nested_block_depth > 0 && !self.allow_nested {
                self.other_use = true;
            }
            self.forwarded
                .push(node.location().start_offset()..node.location().end_offset());
            return;
        }
        ruby_prism::visit_block_argument_node(self, node);
    }

    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
    }

    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
        ruby_prism::visit_local_variable_write_node(self, node);
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
        ruby_prism::visit_local_variable_or_write_node(self, node);
    }

    fn visit_local_variable_and_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
        ruby_prism::visit_local_variable_and_write_node(self, node);
    }

    fn visit_local_variable_operator_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        if node.name().as_slice() == self.name {
            self.other_use = true;
        }
        ruby_prism::visit_local_variable_operator_write_node(self, node);
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        self.nested_block_depth += 1;
        ruby_prism::visit_block_node(self, node);
        self.nested_block_depth -= 1;
    }
}

fn explicit_block_forwarding(
    definition: &ruby_prism::DefNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    if !context.target_ruby_version().at_least(3, 1) {
        return;
    }
    let Some(parameter) = definition.parameters().and_then(|parameters| parameters.block()) else {
        return;
    };
    if parameter.name().is_some() {
        return;
    }
    let name = context
        .config_value("BlockForwardingName")
        .unwrap_or("block")
        .to_string();
    let in_use = definition.body().is_some_and(|body| {
        context.source_file().node(&body).split(|character: char| {
            !character.is_ascii_alphanumeric() && character != '_'
        }).any(|word| word == name)
    });
    let mut ranges = Vec::new();
    if let Some(body) = definition.body() {
        let mut finder = AnonymousBlockForwarding { ranges: Vec::new() };
        ruby_prism::Visit::visit(&mut finder, &body);
        ranges = finder.ranges;
    }
    ranges.push(parameter.location().start_offset()..parameter.location().end_offset());
    for range in ranges {
        if in_use {
            context.report("Use explicit block forwarding.", range);
        } else {
            context.replace(
                "Use explicit block forwarding.",
                range.clone(),
                range,
                format!("&{name}"),
            );
        }
    }
}

struct AnonymousBlockForwarding {
    ranges: Vec<std::ops::Range<usize>>,
}

impl<'pr> ruby_prism::Visit<'pr> for AnonymousBlockForwarding {
    fn visit_block_argument_node(&mut self, node: &ruby_prism::BlockArgumentNode<'pr>) {
        if node.expression().is_none() {
            self.ranges
                .push(node.location().start_offset()..node.location().end_offset());
            return;
        }
        ruby_prism::visit_block_argument_node(self, node);
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
}

fn constant_reassignment(context: &mut CopContext<'_, '_>) {
    if context.source().contains("remove_const")
        || context.source().contains(" do\n")
        || context.source().contains(" unless ")
        || context
            .source()
            .lines()
            .any(|line| line.trim_start().starts_with("if "))
    {
        return;
    }
    if context
        .source()
        .lines()
        .filter(|line| {
            ["class ", "module "]
                .iter()
                .any(|keyword| line.trim_start().starts_with(keyword))
        })
        .count()
        > 1
    {
        return;
    }
    let mut constants = HashSet::new();
    for (offset, line) in context.source_file().lines() {
        if [
            "class ", "module ", "def ", "if ", "unless ", "case ", "begin",
        ]
        .iter()
        .any(|keyword| line.trim_start().starts_with(keyword))
            || matches!(line.trim(), "end" | "else" | "elsif" | "rescue" | "ensure")
        {
            constants.clear();
        }
        if line.contains("||=") {
            continue;
        }
        let Some((name, _)) = line.split_once('=') else {
            continue;
        };
        let name = name.trim();
        if !name.is_empty()
            && name
                .bytes()
                .all(|byte| byte.is_ascii_uppercase() || byte == b'_')
            && !constants.insert(name.to_string())
        {
            let start = offset + line.find(name).unwrap_or(0);
            context.report("Constant is already assigned.", start..start + name.len());
        }
    }
}
