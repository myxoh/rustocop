use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Lint/ShadowedArgument", shadowed_argument),
        custom("Naming/InclusiveLanguage", inclusive_language),
    ]
}

fn shadowed_argument(context: &mut CopContext<'_, '_>) {
    let parsed = ruby_prism::parse(context.source().as_bytes());
    let ignore_implicit = context.config_bool("IgnoreImplicitReferences", false);
    let mut collector = ShadowedArgumentScopes {
        offenses: Vec::new(),
        ignore_implicit,
    };
    ruby_prism::Visit::visit(&mut collector, &parsed.node());
    for (name, range) in collector.offenses {
        context.report(
            format!("Argument `{name}` was shadowed by a local variable before it was used."),
            range,
        );
    }
}

struct ShadowedArgumentScopes {
    offenses: Vec<(String, std::ops::Range<usize>)>,
    ignore_implicit: bool,
}

impl<'pr> ruby_prism::Visit<'pr> for ShadowedArgumentScopes {
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        if let (Some(parameters), Some(body)) = (node.parameters(), node.body()) {
            self.inspect_scope(parameter_infos(&parameters), &body, true);
        }
        ruby_prism::visit_def_node(self, node);
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        if let (Some(parameters), Some(body)) = (
            node.parameters()
                .and_then(|parameters| parameters.as_block_parameters_node())
                .and_then(|parameters| parameters.parameters()),
            node.body(),
        ) {
            self.inspect_scope(parameter_infos(&parameters), &body, false);
        }
        ruby_prism::visit_block_node(self, node);
    }

    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
        if let (Some(parameters), Some(body)) = (
            node.parameters()
                .and_then(|parameters| parameters.as_parameters_node()),
            node.body(),
        ) {
            self.inspect_scope(parameter_infos(&parameters), &body, false);
        }
        ruby_prism::visit_lambda_node(self, node);
    }
}

impl ShadowedArgumentScopes {
    fn inspect_scope<'pr>(
        &mut self,
        parameters: Vec<ArgumentInfo>,
        body: &Node<'pr>,
        implicit_references: bool,
    ) {
        if parameters.is_empty() {
            return;
        }
        let mut events = ArgumentEvents::default();
        ruby_prism::Visit::visit(&mut events, body);
        for parameter in parameters {
            let assignments = events
                .assignments
                .iter()
                .filter(|assignment| assignment.name == parameter.name)
                .collect::<Vec<_>>();
            let references = events
                .reads
                .iter()
                .filter(|read| {
                    read.name == parameter.name
                        || (read.implicit
                            && (implicit_references || read.implicit_for_blocks))
                })
                .collect::<Vec<_>>();
            if references.is_empty() {
                continue;
            }
            let mut location_known = true;
            for assignment in assignments {
                if assignment.shorthand {
                    location_known = false;
                    continue;
                }
                if assignment.uses_argument {
                    continue;
                }
                if assignment.conditional {
                    location_known = false;
                    continue;
                }
                let assignment_reference_range = events
                    .reference_exclusions
                    .iter()
                    .find(|(name, range)| {
                        *name == parameter.name
                            && range.start <= assignment.range.start
                            && assignment.range.end <= range.end
                    })
                    .map(|(_, range)| range);
                if references
                    .iter()
                    .any(|reference| {
                        if reference.implicit && self.ignore_implicit {
                            return true;
                        }
                        if assignment_reference_range.is_some_and(|range| {
                            range.start <= reference.position && reference.position < range.end
                        }) {
                            return reference.implicit
                                && assignment_reference_range
                                    .is_some_and(|range| range.start < assignment.range.start);
                        }
                        reference.position <= assignment.range.start
                    })
                {
                    break;
                }
                let range = if location_known {
                    assignment.range.clone()
                } else {
                    parameter.range.clone()
                };
                self.offenses.push((
                    String::from_utf8_lossy(&parameter.name).into_owned(),
                    range,
                ));
                break;
            }
        }
    }
}

struct ArgumentInfo {
    name: Vec<u8>,
    range: std::ops::Range<usize>,
}

fn parameter_infos(parameters: &ruby_prism::ParametersNode<'_>) -> Vec<ArgumentInfo> {
    let mut result = Vec::new();
    for parameter in parameters
        .requireds()
        .iter()
        .chain(parameters.posts().iter())
    {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            push_argument(
                &mut result,
                parameter.name().as_slice(),
                parameter.location(),
            );
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            push_argument(&mut result, parameter.name().as_slice(), parameter.name_loc());
        }
    }
    for parameter in parameters.keywords().iter() {
        if let Some(parameter) = parameter.as_required_keyword_parameter_node() {
            push_argument(&mut result, parameter.name().as_slice(), parameter.name_loc());
        } else if let Some(parameter) = parameter.as_optional_keyword_parameter_node() {
            push_argument(&mut result, parameter.name().as_slice(), parameter.name_loc());
        }
    }
    if let Some(parameter) = parameters.rest().and_then(|node| node.as_rest_parameter_node()) {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            push_argument(&mut result, name.as_slice(), location);
        }
    }
    if let Some(parameter) = parameters
        .keyword_rest()
        .and_then(|node| node.as_keyword_rest_parameter_node())
    {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            push_argument(&mut result, name.as_slice(), location);
        }
    }
    if let Some(parameter) = parameters.block() {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            push_argument(&mut result, name.as_slice(), location);
        }
    }
    result
}

fn push_argument(
    arguments: &mut Vec<ArgumentInfo>,
    name: &[u8],
    location: ruby_prism::Location<'_>,
) {
    arguments.push(ArgumentInfo {
        name: name.to_vec(),
        range: location.start_offset()..location.end_offset().min(
            location.start_offset() + name.len(),
        ),
    });
}

#[derive(Default)]
struct ArgumentEvents {
    assignments: Vec<ArgumentAssignment>,
    reads: Vec<ArgumentRead>,
    reference_exclusions: Vec<(Vec<u8>, std::ops::Range<usize>)>,
    nested_scopes: u32,
    conditional_depth: u32,
}

struct ArgumentAssignment {
    name: Vec<u8>,
    range: std::ops::Range<usize>,
    conditional: bool,
    shorthand: bool,
    uses_argument: bool,
}

struct ArgumentRead {
    name: Vec<u8>,
    position: usize,
    implicit: bool,
    implicit_for_blocks: bool,
}

impl<'pr> ruby_prism::Visit<'pr> for ArgumentEvents {
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        if node.depth() == self.nested_scopes {
            self.reads.push(ArgumentRead {
                name: node.name().as_slice().to_vec(),
                position: node.location().start_offset(),
                implicit: false,
                implicit_for_blocks: false,
            });
        }
    }

    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        if node.depth() == self.nested_scopes {
            self.reference_exclusions.push((
                node.name().as_slice().to_vec(),
                node.location().start_offset()..node.location().end_offset(),
            ));
            let mut reads = LocalReadNames::default();
            ruby_prism::Visit::visit(&mut reads, &node.value());
            self.assignments.push(ArgumentAssignment {
                name: node.name().as_slice().to_vec(),
                range: node.location().start_offset()..node.location().end_offset(),
                conditional: self.conditional_depth > 0,
                shorthand: false,
                uses_argument: reads.names.contains(node.name().as_slice()),
            });
        }
        ruby_prism::visit_local_variable_write_node(self, node);
    }

    fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode<'pr>) {
        let mut reads = LocalReadNames::default();
        ruby_prism::Visit::visit(&mut reads, &node.value());
        let assignment_start = self.assignments.len();
        ruby_prism::visit_multi_write_node(self, node);
        let operator = node.operator_loc().start_offset();
        let full_range = node.location().start_offset()..node.location().end_offset();
        for assignment in &mut self.assignments[assignment_start..] {
            if assignment.range.start < operator {
                self.reference_exclusions
                    .push((assignment.name.clone(), full_range.clone()));
                if reads.names.contains(&assignment.name) {
                    assignment.uses_argument = true;
                }
            }
        }
    }

    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        if node.depth() == self.nested_scopes {
            self.assignments.push(ArgumentAssignment {
                name: node.name().as_slice().to_vec(),
                range: node.location().start_offset()..node.location().end_offset(),
                conditional: self.conditional_depth > 0,
                shorthand: false,
                uses_argument: false,
            });
        }
    }

    fn visit_local_variable_operator_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        self.record_shorthand(
            node.name().as_slice(),
            node.depth(),
            node.location(),
        );
        ruby_prism::visit_local_variable_operator_write_node(self, node);
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        self.record_shorthand(
            node.name().as_slice(),
            node.depth(),
            node.location(),
        );
        ruby_prism::visit_local_variable_or_write_node(self, node);
    }

    fn visit_local_variable_and_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        self.record_shorthand(
            node.name().as_slice(),
            node.depth(),
            node.location(),
        );
        ruby_prism::visit_local_variable_and_write_node(self, node);
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if node.receiver().is_none() && node.arguments().is_none() && node.name().as_slice() == b"binding" {
            self.reads.push(ArgumentRead {
                name: Vec::new(),
                position: node.location().start_offset(),
                implicit: true,
                implicit_for_blocks: true,
            });
        }
        ruby_prism::visit_call_node(self, node);
    }

    fn visit_forwarding_super_node(&mut self, node: &ruby_prism::ForwardingSuperNode<'pr>) {
        self.reads.push(ArgumentRead {
            name: Vec::new(),
            position: node.location().start_offset(),
            implicit: true,
            implicit_for_blocks: false,
        });
        ruby_prism::visit_forwarding_super_node(self, node);
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        self.nested_scopes += 1;
        self.conditional_depth += 1;
        ruby_prism::visit_block_node(self, node);
        self.conditional_depth -= 1;
        self.nested_scopes -= 1;
    }

    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
        self.nested_scopes += 1;
        self.conditional_depth += 1;
        ruby_prism::visit_lambda_node(self, node);
        self.conditional_depth -= 1;
        self.nested_scopes -= 1;
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
    fn visit_class_node(&mut self, _node: &ruby_prism::ClassNode<'pr>) {}
    fn visit_module_node(&mut self, _node: &ruby_prism::ModuleNode<'pr>) {}

    fn visit_if_node(&mut self, node: &ruby_prism::IfNode<'pr>) {
        self.with_conditional(|visitor| ruby_prism::visit_if_node(visitor, node));
    }
    fn visit_unless_node(&mut self, node: &ruby_prism::UnlessNode<'pr>) {
        self.with_conditional(|visitor| ruby_prism::visit_unless_node(visitor, node));
    }
    fn visit_case_node(&mut self, node: &ruby_prism::CaseNode<'pr>) {
        self.with_conditional(|visitor| ruby_prism::visit_case_node(visitor, node));
    }
    fn visit_case_match_node(&mut self, node: &ruby_prism::CaseMatchNode<'pr>) {
        self.with_conditional(|visitor| ruby_prism::visit_case_match_node(visitor, node));
    }
    fn visit_while_node(&mut self, node: &ruby_prism::WhileNode<'pr>) {
        self.with_conditional(|visitor| ruby_prism::visit_while_node(visitor, node));
    }
    fn visit_until_node(&mut self, node: &ruby_prism::UntilNode<'pr>) {
        self.with_conditional(|visitor| ruby_prism::visit_until_node(visitor, node));
    }
    fn visit_for_node(&mut self, node: &ruby_prism::ForNode<'pr>) {
        self.with_conditional(|visitor| ruby_prism::visit_for_node(visitor, node));
    }
    fn visit_rescue_node(&mut self, node: &ruby_prism::RescueNode<'pr>) {
        self.with_conditional(|visitor| ruby_prism::visit_rescue_node(visitor, node));
    }
}

impl ArgumentEvents {
    fn record_shorthand(
        &mut self,
        name: &[u8],
        depth: u32,
        location: ruby_prism::Location<'_>,
    ) {
        if depth == self.nested_scopes {
            self.assignments.push(ArgumentAssignment {
                name: name.to_vec(),
                range: location.start_offset()..location.end_offset(),
                conditional: self.conditional_depth > 0,
                shorthand: true,
                uses_argument: true,
            });
        }
    }

    fn with_conditional(&mut self, visit: impl FnOnce(&mut Self)) {
        self.conditional_depth += 1;
        visit(self);
        self.conditional_depth -= 1;
    }
}

#[derive(Default)]
struct LocalReadNames {
    names: std::collections::HashSet<Vec<u8>>,
}

impl<'pr> ruby_prism::Visit<'pr> for LocalReadNames {
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        self.names.insert(node.name().as_slice().to_vec());
    }
}

#[allow(clippy::too_many_lines)]
fn inclusive_language(context: &mut CopContext<'_, '_>) {
    let terms = context
        .config_map("FlaggedTerms")
        .cloned()
        .unwrap_or_default();
    let mut configured = terms
        .into_iter()
        .filter_map(|(term, encoded)| {
            (!matches!(encoded.as_str(), "" | "nil" | "null" | "~"))
                .then(|| (term, inclusive_term_config(&encoded)))
        })
        .collect::<Vec<_>>();
    configured.sort_by(|left, right| left.0.cmp(&right.0));
    inclusive_filepath(&configured, context);

    let file = context.source_file();
    let comments = file.comment_ranges();
    let literals = file.literal_ranges();
    let heredocs = file.heredoc_ranges();
    for (term, config) in configured {
        let pattern = config.regex.clone().unwrap_or_else(|| regex::escape(&term));
        let pattern = if config.whole_word {
            format!(r"(?i)(?<![[:alnum:]])(?:{pattern})(?![[:alnum:]])")
        } else {
            format!(r"(?i:{pattern})")
        };
        // `regex` does not support look-around; whole-word boundaries are
        // checked below while the regex supplies the configurable spelling.
        let pattern = pattern
            .replace("(?i)(?<![[:alnum:]])(?:", "(?i:")
            .replace(")(?![[:alnum:]])", ")");
        let Ok(matcher) = regex::Regex::new(&pattern) else {
            continue;
        };
        for matched in matcher.find_iter(context.source()) {
            let start = matched.start();
            let end = matched.end();
            if config.whole_word {
                let before = context.source()[..start].chars().next_back();
                let after = context.source()[end..].chars().next();
                if before.is_some_and(char::is_alphanumeric)
                    || after.is_some_and(char::is_alphanumeric)
                {
                    continue;
                }
            }
            if config.allowed_regex.as_ref().is_some_and(|allowed| {
                regex::RegexBuilder::new(allowed)
                    .case_insensitive(true)
                    .build()
                    .is_ok_and(|allowed| {
                        let line = file.line(start);
                        allowed.is_match(line)
                    })
            }) {
                continue;
            }
            let in_comment = comments
                .iter()
                .any(|range| range.start <= start && end <= range.end);
            let containing_heredoc = heredocs
                .iter()
                .find(|range| range.start <= start && end <= range.end);
            let containing_literal = literals
                .iter()
                .find(|range| range.start <= start && end <= range.end);
            let interpolation_start = containing_literal
                .map(|range| range.start)
                .or_else(|| containing_heredoc.map(|range| range.start));
            let in_interpolation = interpolation_start.is_some_and(|range_start| {
                let prefix = &context.source()[range_start..start];
                prefix.rfind("#{") > prefix.rfind('}')
            });
            let in_heredoc = containing_heredoc.is_some()
                && !file.line(start).contains("<<")
                && !in_interpolation;
            let in_literal = containing_literal.is_some() && !in_interpolation;
            let token_start = context.source()[..start]
                .rfind(|character: char| !character.is_alphanumeric() && character != '_')
                .map_or(0, |offset| {
                    offset
                        + context.source()[offset..]
                            .chars()
                            .next()
                            .map_or(1, char::len_utf8)
                });
            let token_end = context.source()[end..]
                .find(|character: char| !character.is_alphanumeric() && character != '_')
                .map_or(context.source().len(), |offset| end + offset);
            let previous = context.source()[..token_start].chars().next_back();
            let symbol = previous == Some(':')
                && context.source().as_bytes().get(token_start.saturating_sub(2)) != Some(&b':');
            let variable = matches!(previous, Some('@' | '$'));
            let token = &context.source()[token_start..token_end];
            let predicate_suffix = context.source()[token_end..].chars().next();
            if !in_comment
                && !in_heredoc
                && !in_literal
                && !symbol
                && !variable
                && matches!(predicate_suffix, Some('?' | '!'))
                && !inclusive_method_definition(context.source(), token_start)
            {
                continue;
            }
            let constant =
                !variable && !symbol && token.chars().next().is_some_and(char::is_uppercase);
            let enabled = if in_comment {
                context.config_bool("CheckComments", true)
            } else if in_heredoc || in_literal && !symbol {
                context.config_bool("CheckStrings", false)
            } else if symbol {
                context.config_bool("CheckSymbols", true)
            } else if variable {
                context.config_bool("CheckVariables", true)
            } else if constant {
                context.config_bool("CheckConstants", true)
            } else {
                context.config_bool("CheckIdentifiers", true)
            };
            if !enabled {
                continue;
            }
            let found = &context.source()[start..end];
            let message = inclusive_message(found, &config.suggestions);
            if config.suggestions.len() == 1 {
                context.replace(message, start..end, start..end, &config.suggestions[0]);
            } else {
                context.report(message, start..end);
            }
        }
    }
}

fn inclusive_method_definition(source: &str, token_start: usize) -> bool {
    let line_start = source[..token_start]
        .rfind('\n')
        .map_or(0, |offset| offset + 1);
    let before = source[line_start..token_start].trim_start();
    let Some(after_def) = before
        .strip_prefix("def ")
        .or_else(|| before.split_once(" def ").map(|(_, suffix)| suffix))
    else {
        return false;
    };
    after_def.trim_start().trim_end_matches("self.").is_empty()
}

#[derive(Default)]
struct InclusiveTermConfig {
    suggestions: Vec<String>,
    regex: Option<String>,
    allowed_regex: Option<String>,
    whole_word: bool,
}

fn inclusive_term_config(encoded: &str) -> InclusiveTermConfig {
    let mut result = InclusiveTermConfig::default();
    let mut current = "";
    for line in encoded.lines() {
        if let Some((key, value)) = line.split_once('=') {
            current = key;
            match key {
                "Suggestions" if !value.is_empty() => {
                    result.suggestions.extend(inclusive_list(value))
                }
                "Regex" if !value.is_empty() => result.regex = Some(inclusive_regex(value)),
                "$regexp" => result.regex = Some(inclusive_regex(value)),
                "AllowedRegex" => result.allowed_regex = Some(value.to_string()),
                "WholeWord" => result.whole_word = value == "true",
                _ => {}
            }
        } else if current == "Suggestions" && !line.is_empty() {
            result.suggestions.extend(inclusive_list(line));
        }
    }
    result
}

fn inclusive_list(value: &str) -> Vec<String> {
    value
        .trim_matches(['[', ']'])
        .split([',', '\n'])
        .map(|value| value.trim().trim_matches(['\'', '"']).to_string())
        .filter(|value| !value.is_empty())
        .collect()
}

fn inclusive_regex(value: &str) -> String {
    let value = value.trim();
    let value = value.strip_prefix("!ruby/regexp").unwrap_or(value).trim();
    value
        .trim_matches(['\'', '"'])
        .trim_matches('/')
        .to_string()
}

fn inclusive_message(found: &str, suggestions: &[String]) -> String {
    let replacement = match suggestions {
        [] => "another term".to_string(),
        [only] => format!("'{only}'"),
        [first, second] => format!("'{first}' or '{second}'"),
        many => {
            let (last, rest) = many.split_last().unwrap();
            format!(
                "{}, or '{last}'",
                rest.iter()
                    .map(|item| format!("'{item}'"))
                    .collect::<Vec<_>>()
                    .join(", ")
            )
        }
    };
    format!("Consider replacing '{found}' with {replacement}.")
}

fn inclusive_filepath(terms: &[(String, InclusiveTermConfig)], context: &mut CopContext<'_, '_>) {
    if !context.config_bool("CheckFilepaths", true) {
        return;
    }
    let mut found = Vec::new();
    for (term, config) in terms {
        let matcher = config.regex.as_deref().unwrap_or(term);
        if let Ok(regex) = regex::RegexBuilder::new(matcher)
            .case_insensitive(true)
            .build()
        {
            found.extend(
                regex
                    .find_iter(context.path())
                    .map(|matched| (matched.as_str().to_string(), config)),
            );
        }
    }
    if found.is_empty() {
        return;
    }
    let message = if found.len() == 1 {
        inclusive_message(&found[0].0, &found[0].1.suggestions).replacen(
            " with ",
            " in file path with ",
            1,
        )
    } else {
        let names = found
            .iter()
            .map(|(term, _)| format!("'{term}'"))
            .collect::<Vec<_>>()
            .join(", ");
        format!("Consider replacing {names} in file path with other terms.")
    };
    context.report(message, 0..0);
}
