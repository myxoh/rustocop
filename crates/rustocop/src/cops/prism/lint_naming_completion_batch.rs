use super::*;
use std::collections::{HashMap, HashSet};

define_cops! {
    HeredocDelimiterNaming => "Naming/HeredocDelimiterNaming" => compatibility_source(heredoc_naming),
    MethodParameterName => "Naming/MethodParameterName" => compatibility_prism_node(as_def_node, method_parameter_name),
    UnderscorePrefixedVariableName => "Lint/UnderscorePrefixedVariableName" => compatibility_prism_any_node(underscore_variable),
    DeprecatedConstants => "Lint/DeprecatedConstants" => compatibility_prism_any_node(deprecated_constants),
    RedundantCopEnableDirective => "Lint/RedundantCopEnableDirective" => compatibility_source(redundant_enable),
    UnreachablePatternBranch => "Lint/UnreachablePatternBranch" => compatibility_source(unreachable_pattern),
}

fn underscore_variable(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_program_node().is_none() || context.config_bool("AllowKeywordBlockArguments", false)
    {
        return;
    }
    let mut visitor = UnderscoreVariableVisitor::default();
    ruby_prism::Visit::visit(&mut visitor, node);
    let mut offenses = visitor
        .variables
        .into_values()
        .filter_map(|variable| variable.used.then_some(variable.declaration).flatten())
        .collect::<Vec<_>>();
    offenses.sort_by_key(|range| range.start);
    for range in offenses {
        let range = if context.source().starts_with('/') && context.source().contains("(?<_") {
            context.source()[1..]
                .find('/')
                .map_or(range.clone(), |end| 0..end + 2)
        } else {
            context.source()[..range.start]
                .rfind("/(?<")
                .filter(|start| {
                    let prefix = &context.source()[start + 4..range.start];
                    !prefix.contains(['=', '!'])
                        && !prefix.contains('\n')
                        && prefix.bytes().all(|byte| byte == b'_' || byte.is_ascii_alphanumeric())
                })
                .and_then(|start| {
                    context.source()[range.end..]
                        .find('/')
                        .filter(|end| !context.source()[range.end..range.end + end].contains('\n'))
                        .map(|end| start..range.end + end + 1)
                })
                .unwrap_or(range)
        };
        context.report("Do not use prefix `_` for a variable that is used.", range);
    }
}

#[derive(Default)]
struct UnderscoreVariableVisitor {
    variables: HashMap<(usize, Vec<u8>), UnderscoreVariable>,
    scopes: Vec<usize>,
    branch_scopes: Vec<bool>,
    next_scope: usize,
}

#[derive(Default)]
struct UnderscoreVariable {
    declaration: Option<std::ops::Range<usize>>,
    used: bool,
}

impl UnderscoreVariableVisitor {
    fn scope_for_depth(&self, depth: u32) -> Option<usize> {
        self.scopes
            .len()
            .checked_sub(depth as usize + 1)
            .and_then(|index| self.scopes.get(index))
            .copied()
    }

    fn declare(&mut self, name: &[u8], depth: u32, location: ruby_prism::Location<'_>) {
        if !underscore_prefixed_name(name) {
            return;
        }
        let Some(scope) = self.scope_for_depth(depth) else {
            return;
        };
        self.variables
            .entry((scope, name.to_vec()))
            .or_default()
            .declaration
            .get_or_insert(location.start_offset()..location.start_offset() + name.len());
    }

    fn use_variable(&mut self, name: &[u8], depth: u32) {
        if !underscore_prefixed_name(name) {
            return;
        }
        let Some(scope) = self.scope_for_depth(depth) else {
            return;
        };
        self.variables
            .entry((scope, name.to_vec()))
            .or_default()
            .used = true;
    }

    fn observe(&mut self, node: &Node<'_>) {
        if let Some(read) = node.as_local_variable_read_node() {
            self.use_variable(read.name().as_slice(), read.depth());
        } else if let Some(write) = node.as_local_variable_write_node() {
            self.declare(write.name().as_slice(), write.depth(), write.name_loc());
        } else if let Some(target) = node.as_local_variable_target_node() {
            self.declare(target.name().as_slice(), target.depth(), target.location());
        } else if let Some(write) = node.as_local_variable_and_write_node() {
            self.declare(write.name().as_slice(), write.depth(), write.name_loc());
            self.use_variable(write.name().as_slice(), write.depth());
        } else if let Some(write) = node.as_local_variable_or_write_node() {
            self.declare(write.name().as_slice(), write.depth(), write.name_loc());
            self.use_variable(write.name().as_slice(), write.depth());
        } else if let Some(write) = node.as_local_variable_operator_write_node() {
            self.declare(write.name().as_slice(), write.depth(), write.name_loc());
            self.use_variable(write.name().as_slice(), write.depth());
        } else if let Some(parameter) = node.as_required_parameter_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.location());
        } else if let Some(parameter) = node.as_optional_parameter_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.name_loc());
        } else if let Some(parameter) = node.as_rest_parameter_node() {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                self.declare(name.as_slice(), 0, location);
            }
        } else if let Some(parameter) = node.as_required_keyword_parameter_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.name_loc());
        } else if let Some(parameter) = node.as_optional_keyword_parameter_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.name_loc());
        } else if let Some(parameter) = node.as_keyword_rest_parameter_node() {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                self.declare(name.as_slice(), 0, location);
            }
        } else if let Some(parameter) = node.as_block_parameter_node() {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                self.declare(name.as_slice(), 0, location);
            }
        } else if let Some(parameter) = node.as_block_local_variable_node() {
            self.declare(parameter.name().as_slice(), 0, parameter.location());
        }
    }
}

impl<'pr> ruby_prism::Visit<'pr> for UnderscoreVariableVisitor {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        let opens_scope = prism_local_scope(&node);
        self.branch_scopes.push(opens_scope);
        if opens_scope {
            let scope = self.next_scope;
            self.next_scope += 1;
            self.scopes.push(scope);
        }
        self.observe(&node);
    }

    fn visit_branch_node_leave(&mut self) {
        if self.branch_scopes.pop() == Some(true) {
            self.scopes.pop();
        }
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        self.observe(&node);
    }
}

fn prism_local_scope(node: &Node<'_>) -> bool {
    node.as_program_node().is_some()
        || node.as_def_node().is_some()
        || node.as_block_node().is_some()
        || node.as_lambda_node().is_some()
        || node.as_class_node().is_some()
        || node.as_module_node().is_some()
        || node.as_singleton_class_node().is_some()
}

fn underscore_prefixed_name(name: &[u8]) -> bool {
    name.starts_with(b"_")
}

fn heredoc_naming(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    static DEFAULT_FORBIDDEN: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"(?i)(^|\s)(EO[A-Z]|END)(\s|$)").expect("default regex")
    });
    static WORD: std::sync::LazyLock<regex::Regex> = std::sync::LazyLock::new(|| {
        regex::Regex::new(r"\w").expect("word regex")
    });

    let source = context.source();
    if !context.processed_source().valid_syntax() {
        // Parser can represent a blank quoted delimiter even though Prism
        // rejects it. Only use the lexical fallback for invalid input: on a
        // valid file, the same bytes may occur harmlessly in comments or in a
        // heredoc body and must not be interpreted as Ruby syntax.
        for range in blank_heredoc_opening_ranges(source) {
            context.report("Use meaningful heredoc delimiters.", range);
        }
        return;
    }
    let Some(root) = context.processed_source().ast() else {
        return;
    };
    let configured = context.config_values("ForbiddenDelimiters").to_vec();
    let use_default_forbidden = !context.related_config_explicit(
        "Naming/HeredocDelimiterNaming",
        "ForbiddenDelimiters",
    );
    let forbidden = if use_default_forbidden {
        Vec::new()
    } else {
        configured
            .iter()
            .filter_map(|pattern| heredoc_forbidden_regex(pattern))
            .collect()
    };

    for node in root.each_node(&["any_str"]) {
        if !node.heredoc() {
            continue;
        }
        let delimiter = heredoc_delimiter(node.source().unwrap_or(""));
        let meaningful = WORD.is_match(delimiter)
            && if use_default_forbidden {
                !DEFAULT_FORBIDDEN.is_match(delimiter)
            } else {
                forbidden.iter().all(|pattern| !pattern.is_match(delimiter))
            };
        if meaningful {
            continue;
        }
        let range = if delimiter.is_empty() {
            node.source_range()
        } else {
            node.loc("heredoc_end").map(|(range, _)| range.clone())
        };
        if let Some(range) = range {
            context.report(
                "Use meaningful heredoc delimiters.",
                heredoc_character_range_to_byte(source, range),
            );
        }
    }
}

fn heredoc_delimiter(opening: &str) -> &str {
    let Some(at) = opening.find("<<") else {
        return "";
    };
    let mut tail = &opening[at + 2..];
    if matches!(tail.as_bytes().first(), Some(b'-' | b'~')) {
        tail = &tail[1..];
    }
    match tail.as_bytes().first().copied() {
        Some(quote @ (b'\'' | b'"' | b'`')) => tail[1..]
            .bytes()
            .position(|byte| byte == quote)
            .map_or("", |end| &tail[1..end + 1]),
        _ => tail
            .find(|character: char| character.is_whitespace())
            .map_or(tail, |end| &tail[..end]),
    }
}

fn heredoc_forbidden_regex(pattern: &str) -> Option<regex::Regex> {
    let pattern = pattern
        .strip_prefix("!ruby/regexp")
        .unwrap_or(pattern)
        .trim()
        .trim_matches(['\'', '"'])
        .replace("\\A", "^")
        .replace("\\z", "$");
    let pattern = if let Some(body) = pattern.strip_prefix('/') {
        let end = body.rfind('/')?;
        let flags = &body[end + 1..];
        format!("{}{}", if flags.contains('i') { "(?i)" } else { "" }, &body[..end])
    } else {
        pattern
    };
    regex::Regex::new(&pattern).ok()
}

fn heredoc_character_range_to_byte(
    source: &str,
    range: std::ops::Range<usize>,
) -> std::ops::Range<usize> {
    let start = source
        .char_indices()
        .nth(range.start)
        .map_or(source.len(), |(byte, _)| byte);
    let end = source
        .char_indices()
        .nth(range.end)
        .map_or(source.len(), |(byte, _)| byte);
    start..end
}

fn blank_heredoc_opening_ranges(source: &str) -> Vec<std::ops::Range<usize>> {
    let mut ranges = Vec::new();
    let mut line_start = 0;
    for line in source.split_inclusive('\n') {
        let bytes = line.as_bytes();
        for (at, _) in line.match_indices("<<") {
            let mut cursor = at + 2;
            if matches!(bytes.get(cursor), Some(b'-' | b'~')) {
                cursor += 1;
            }
            if let Some(quote @ (b'\'' | b'"' | b'`')) = bytes.get(cursor).copied() {
                if bytes.get(cursor + 1) == Some(&quote) {
                    ranges.push(line_start + at..line_start + cursor + 2);
                }
            }
        }
        line_start += line.len();
    }
    ranges
}

fn deprecated_constants(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let location = if let Some(read) = node.as_constant_read_node() {
        read.location()
    } else if let Some(path) = node.as_constant_path_node() {
        path.location()
    } else {
        return;
    };
    let used = context.source_file().at(&location);
    let lookup = used.strip_prefix("::").unwrap_or(used);
    let offense = location.start_offset()..location.end_offset();
    let configured = context
        .config_map("DeprecatedConstants")
        .cloned()
        .unwrap_or_default();
    let Some(details) = configured.get(lookup) else {
        return;
    };
        let mut alternative = None;
        let mut deprecated_version = None;
        for field in details.lines() {
            let Some((key, value)) = field.split_once('=') else {
                continue;
            };
            match key {
                "Alternative" => alternative = Some(value),
                "DeprecatedVersion" => deprecated_version = Some(value),
                _ => {}
            }
        }
        if deprecated_version.is_some_and(|version| {
            let mut parts = version
                .split('.')
                .filter_map(|part| part.parse::<u16>().ok());
            let (Some(major), Some(minor)) = (parts.next(), parts.next()) else {
                return false;
            };
            !context.target_ruby_version().at_least(major, minor)
        }) {
            return;
        }
    let suffix = deprecated_version
        .map(|version| format!(", deprecated since Ruby {version}"))
        .unwrap_or_default();
    if let Some(alternative) = alternative {
        context.replace(
            format!("Use `{alternative}` instead of `{used}`{suffix}."),
            offense.clone(),
            offense,
            alternative,
        );
    } else {
        context.report(format!("Do not use `{used}`{suffix}."), offense);
    }
}

fn redundant_enable(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    if context.processed_source().blank() || !context.source().contains("enable") {
        return;
    }
    let known_cops = crate::cops::cop_names();
    let known_departments = known_cops
        .iter()
        .filter_map(|cop| cop.split_once('/').map(|(department, _)| department))
        .collect::<HashSet<_>>();
    let mut disabled = HashSet::new();
    let mut configured_enable_edits = HashMap::new();
    for comment_range in context.comment_ranges() {
        let comment = &context.source()[comment_range.clone()];
        let line_start = context.source()[..comment_range.start]
            .rfind('\n')
            .map_or(0, |newline| newline + 1);
        let line_end = context.source()[comment_range.end..]
            .find('\n')
            .map_or(context.source().len(), |newline| comment_range.end + newline);
        let line = &context.source()[line_start..line_end];
        let directives = redundant_directive_bodies(comment);
        for directive in &directives {
            if let Some(list) = redundant_directive_list(directive, "disable")
                .or_else(|| redundant_directive_list(directive, "todo"))
            {
                disabled.extend(
                    redundant_listed_cops(list)
                        .into_iter()
                        .filter(|cop| {
                            redundant_enable_known(cop, &known_cops, &known_departments)
                        })
                        .map(str::to_string),
                );
            }
        }
        let Some(list) = directives
            .iter()
            .find_map(|directive| redundant_directive_list(directive, "enable"))
        else {
            continue;
        };
        let list = list.split("--").next().unwrap_or_default().trim_end();
        let list_start = line.find(list).unwrap_or(line.len());
        let listed_cops = redundant_listed_cops(list);
        let mut redundant = Vec::new();
        let mut necessary = Vec::new();
        let mut preserve_department_line = false;
        for cop in listed_cops {
            if !redundant_enable_known(cop, &known_cops, &known_departments) {
                continue;
            }
            if cop == "all" && !disabled.is_empty() {
                disabled.clear();
                necessary.push(cop);
                continue;
            }
            if disabled.remove(cop) {
                necessary.push(cop);
                continue;
            }
            if cop
                .split_once('/')
                .is_some_and(|(department, _)| disabled.contains(department))
            {
                necessary.push(cop);
                continue;
            }
            if context.related_config_value(cop, "Enabled") == Some("false")
                && !configured_enable_edits.contains_key(cop)
            {
                let start = line_start + line.find('#').unwrap_or_default();
                let mut end = line_start + line.len();
                if context.source().as_bytes().get(end) == Some(&b'\n') {
                    end += 1;
                }
                configured_enable_edits.insert(cop.to_string(), start..end);
                necessary.push(cop);
                continue;
            }
            if !cop.contains('/')
                && disabled
                    .iter()
                    .any(|disabled| disabled.starts_with(&format!("{cop}/")))
            {
                preserve_department_line = true;
            }
            redundant.push(cop);
        }
        if redundant.is_empty() {
            continue;
        }
        let separator = list
            .find(',')
            .map(|comma| {
                let mut end = comma + 1;
                while list.as_bytes().get(end) == Some(&b' ') {
                    end += 1;
                }
                &list[comma..end]
            })
            .unwrap_or(", ");
        let replacement = necessary.join(separator);
        for (index, cop) in redundant.iter().enumerate() {
            let start = line_start + line.find(cop).unwrap_or(list_start);
            let label = if *cop == "all" { "all cops" } else { cop };
            let message = format!("Unnecessary enabling of {label}.");
            if index == 0 {
                if necessary.is_empty() {
                    let mut edit_end = line_start + line.len();
                    if context.source().as_bytes().get(edit_end) == Some(&b'\n') {
                        edit_end += 1;
                        if context.source().as_bytes().get(edit_end) == Some(&b'\n') {
                            edit_end += 1;
                        }
                    }
                    let replacement = if preserve_department_line { "\n" } else { "" };
                    let edit_start = line_start + line.find('#').unwrap_or_default();
                    if let Some(first_edit) = configured_enable_edits.get(*cop) {
                        context.replace_many(
                            message,
                            start..start + cop.len(),
                            vec![
                                (first_edit.clone(), String::new()),
                                (edit_start..edit_end, replacement.to_string()),
                            ],
                        );
                    } else {
                        context.replace(
                            message,
                            start..start + cop.len(),
                            edit_start..edit_end,
                            replacement,
                        );
                    }
                } else {
                    context.replace(
                        message,
                        start..start + cop.len(),
                        line_start + list_start..line_start + list_start + list.len(),
                        replacement.clone(),
                    );
                }
            } else {
                context.replace(message, start..start + cop.len(), start..start, "");
            }
        }
    }
}

fn redundant_directive_list<'a>(comment: &'a str, action: &str) -> Option<&'a str> {
    let directive = comment.strip_prefix("rubocop:")?.trim_start();
    let list = directive.strip_prefix(action)?;
    list.as_bytes()
        .first()
        .is_some_and(u8::is_ascii_whitespace)
        .then(|| list.trim_start())
}

fn redundant_directive_bodies(comment: &str) -> Vec<&str> {
    let Some(marker) = comment.find("# rubocop:") else {
        return Vec::new();
    };
    let start = marker + 2;
    let prefix = &comment[..start];
    let nested_comment_only = prefix.bytes().filter(|byte| *byte == b'#').count() > 1
        && prefix
            .trim_matches(|character: char| character == '#' || character.is_whitespace())
            .is_empty();
    if nested_comment_only {
        Vec::new()
    } else {
        vec![comment[start..].trim_end()]
    }
}

fn redundant_listed_cops(list: &str) -> Vec<&str> {
    list.split("--")
        .next()
        .unwrap_or_default()
        .split(',')
        .filter_map(|raw| {
            let raw = raw.trim();
            let end = raw
                .bytes()
                .take_while(|byte| {
                    byte.is_ascii_alphanumeric() || matches!(byte, b'/' | b'_')
                })
                .count();
            (end > 0).then_some(&raw[..end])
        })
        .collect()
}

fn redundant_enable_known(
    name: &str,
    known_cops: &[&str],
    known_departments: &HashSet<&str>,
) -> bool {
    if name == "all"
        || known_cops.contains(&name)
        || known_departments.contains(name)
        || name
            .split_once('/')
            .is_some_and(|(department, cop)| !department.is_empty() && !cop.is_empty())
    {
        return true;
    }

    // These are the official RuboCop extension departments represented in the
    // cached project corpus. Their registries are loaded by RuboCop itself but
    // are intentionally not part of RustOcop's built-in cop inventory.
    matches!(
        name.split('/').next(),
        Some("Capybara" | "FactoryBot" | "Performance" | "Rails" | "Require" | "RSpec")
    )
}

fn unreachable_pattern(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let literal_ranges = context.literal_ranges();
    let mut cases = Vec::<(usize, Option<usize>)>::new();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let trimmed = line.trim_start();
        let indentation = line.len() - trimmed.len();
        let code_start = offset + indentation;
        if literal_ranges
            .iter()
            .any(|range| range.start <= code_start && code_start < range.end)
            || trimmed.starts_with('#')
        {
            continue;
        }
        if trimmed == "end" {
            if cases.last().is_some_and(|(case_indent, _)| *case_indent == indentation) {
                cases.pop();
            }
            continue;
        }
        if trimmed == "case" || trimmed.starts_with("case ") {
            cases.push((indentation, None));
            continue;
        }
        let Some((case_indent, catch_all_indent)) = cases.last_mut() else {
            continue;
        };
        if *case_indent > indentation {
            continue;
        }
        if *catch_all_indent == Some(indentation) && trimmed == "else" {
            context.report(
                "Unreachable `else` branch detected.",
                offset..offset + line.len(),
            );
            continue;
        }
        let Some(pattern) = trimmed.strip_prefix("in ") else {
            continue;
        };
        if catch_all_indent.is_some() {
            let end = lines[index + 1..]
                .iter()
                .find(|(_, next)| {
                    next.trim_start().starts_with("in ") || matches!(next.trim(), "else" | "end")
                })
                .map_or(offset + line.len(), |(at, _)| *at);
            context.report(
                "Unreachable `in` pattern branch detected.",
                offset..end.saturating_sub(1),
            );
            continue;
        }
        let pattern = pattern.trim();
        let guarded = pattern.contains(" if ") || pattern.contains(" unless ");
        let has_wildcard = pattern
            .split(|character: char| {
                character.is_ascii_whitespace() || "()|=>,".contains(character)
            })
            .any(|part| part == "_");
        let capture_pattern = pattern.split_once("=>").map_or(pattern, |(left, _)| left.trim());
        let bare_capture = capture_pattern
            .trim_start_matches('(')
            .trim_end_matches(')')
            .as_bytes();
        let bare_capture = bare_capture
            .first()
            .is_some_and(u8::is_ascii_lowercase)
            && bare_capture
                .iter()
                .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
            && !matches!(bare_capture, b"nil" | b"true" | b"false" | b"self");
        if !guarded && (has_wildcard || bare_capture) {
            *catch_all_indent = Some(indentation);
        }
    }
}

fn method_parameter_name(definition: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let minimum = context.config_usize("MinNameLength", 3);
    let allow_numbers = context.config_bool("AllowNamesEndingInNumbers", false);
    let allowed = context.config_values("AllowedNames").to_vec();
    let forbidden = context.config_values("ForbiddenNames").to_vec();
    let Some(parameters) = definition.parameters() else {
        return;
    };
    for (name, range) in named_method_parameters(&parameters) {
        let normalized = name.trim_start_matches('_');
        if normalized.is_empty() || allowed.iter().any(|allowed| allowed == normalized) {
            continue;
        }
        let message = if forbidden.iter().any(|forbidden| forbidden == normalized) {
            Some(format!(
                "Do not use {normalized} as a name for a method parameter."
            ))
        } else if normalized.len() < minimum {
            Some(format!(
                "Method parameter must be at least {minimum} characters long."
            ))
        } else if normalized.bytes().any(|byte| byte.is_ascii_uppercase()) {
            Some("Only use lowercase characters for method parameter.".to_string())
        } else if !allow_numbers
            && normalized
                .bytes()
                .last()
                .is_some_and(|byte| byte.is_ascii_digit())
        {
            Some("Do not end method parameter with a number.".to_string())
        } else {
            None
        };
        if let Some(message) = message {
            context.report(message, range);
        }
    }
}

fn named_method_parameters(
    parameters: &ruby_prism::ParametersNode<'_>,
) -> Vec<(String, std::ops::Range<usize>)> {
    let mut result = Vec::new();
    for parameter in parameters
        .requireds()
        .iter()
        .chain(parameters.posts().iter())
    {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                parameter.location().start_offset()..parameter.location().end_offset(),
            ));
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            let location = parameter.name_loc();
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset(),
            ));
        }
    }
    for parameter in parameters.keywords().iter() {
        if let Some(parameter) = parameter.as_required_keyword_parameter_node() {
            let location = parameter.name_loc();
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset().saturating_sub(1),
            ));
        } else if let Some(parameter) = parameter.as_optional_keyword_parameter_node() {
            let location = parameter.name_loc();
            result.push((
                String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                location.start_offset()..location.end_offset().saturating_sub(1),
            ));
        }
    }
    if let Some(parameter) = parameters
        .rest()
        .and_then(|node| node.as_rest_parameter_node())
    {
        if let (Some(name), Some(_)) = (parameter.name(), parameter.name_loc()) {
            result.push((
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                parameter.location().start_offset()..parameter.location().end_offset(),
            ));
        }
    }
    if let Some(parameter) = parameters
        .keyword_rest()
        .and_then(|node| node.as_keyword_rest_parameter_node())
    {
        if let (Some(name), Some(_)) = (parameter.name(), parameter.name_loc()) {
            result.push((
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                parameter.location().start_offset()..parameter.location().end_offset(),
            ));
        }
    }
    if let Some(parameter) = parameters.block() {
        if let (Some(name), Some(_)) = (parameter.name(), parameter.name_loc()) {
            let start = parameter.location().start_offset();
            result.push((
                String::from_utf8_lossy(name.as_slice()).into_owned(),
                start..start + name.as_slice().len(),
            ));
        }
    }
    result
}

fn accessor_method_name(definition: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let name = String::from_utf8_lossy(definition.name().as_slice());
    if name.ends_with(['!', '?', '=']) {
        return;
    }
    let parameter_count = definition.parameters().map_or(0, |parameters| {
        parameters.requireds().len()
            + parameters.optionals().len()
            + usize::from(parameters.rest().is_some())
            + parameters.posts().len()
            + parameters.keywords().len()
            + usize::from(parameters.keyword_rest().is_some())
            + usize::from(parameters.block().is_some())
    });
    let single_required = definition.parameters().is_some_and(|parameters| {
        parameters.requireds().len() == 1
            && parameters
                .requireds()
                .first()
                .is_some_and(|node| node.as_required_parameter_node().is_some())
            && parameter_count == 1
    });
    let message = if name.starts_with("get_") && parameter_count == 0 {
        Some("Do not prefix reader method names with `get_`.")
    } else if name.starts_with("set_") && single_required {
        Some("Do not prefix writer method names with `set_`.")
    } else {
        None
    };
    if let Some(message) = message {
        let location = definition.name_loc();
        context.report(message, location.start_offset()..location.end_offset());
    }
}
