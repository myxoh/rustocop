use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom("Lint/ShadowedArgument", shadowed_argument),
        custom("Naming/InclusiveLanguage", inclusive_language),
    ]
}

fn shadowed_argument(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let ignore_implicit = context.config_bool("IgnoreImplicitReferences", false);
    for (opener_index, (opener_offset, opener)) in lines.iter().copied().enumerate() {
        let method = opener.trim_start().starts_with("def ");
        let argument_section = if method {
            opener
                .split_once('(')
                .and_then(|(_, rest)| rest.rsplit_once(')').map(|(arguments, _)| arguments))
        } else {
            opener
                .split_once('|')
                .and_then(|(_, rest)| rest.split_once('|').map(|(arguments, _)| arguments))
        };
        let Some(argument_section) = argument_section else {
            continue;
        };
        let section_start = opener.find(argument_section).unwrap_or(0);
        for raw_argument in argument_section.split(',') {
            let raw_argument = raw_argument.trim();
            if raw_argument.is_empty() || raw_argument.contains(';') {
                continue;
            }
            let argument = raw_argument
                .trim_start_matches('*')
                .split([':', '='])
                .next()
                .unwrap_or("")
                .trim();
            if argument.is_empty() {
                continue;
            }
            if ignore_implicit
                && (context.source().contains("super") || context.source().contains("binding"))
            {
                continue;
            }
            let declaration_at = section_start + argument_section.find(argument).unwrap_or(0);
            let declaration_range =
                opener_offset + declaration_at..opener_offset + declaration_at + argument.len();
            let mut depth = 0usize;
            let mut nested_assignment_before_direct = false;
            let mut direct_assignment = None::<std::ops::Range<usize>>;
            let mut used_before_assignment = false;
            let mut used_after_direct = false;

            for (offset, line) in lines.iter().copied().skip(opener_index + 1) {
                let trimmed = line.trim();
                if trimmed == "end" {
                    if depth == 0 {
                        break;
                    }
                    depth -= 1;
                    continue;
                }

                let occurrences = identifier_occurrences(line, argument);
                let assignment = argument_assignment(line, argument);
                if let Some((at, rhs_start, splat)) = assignment {
                    let nested = depth > 0
                        || line[..at].contains('{')
                        || line[rhs_start..].contains(" if ")
                        || line[rhs_start..].contains(" unless ");
                    let rhs_uses_argument =
                        occurrences.iter().any(|position| *position >= rhs_start);
                    if rhs_uses_argument {
                        if direct_assignment.is_some() {
                            used_after_direct = true;
                        } else {
                            used_before_assignment = true;
                        }
                    }
                    if nested {
                        if direct_assignment.is_none() {
                            nested_assignment_before_direct = true;
                        }
                    } else if direct_assignment.is_none() {
                        let range = if splat {
                            offset + at..offset + at + argument.len()
                        } else {
                            let end = line.trim_end().len();
                            offset + at..offset + end
                        };
                        direct_assignment = Some(range);
                    }
                } else if !occurrences.is_empty() {
                    if direct_assignment.is_some() {
                        used_after_direct = true;
                    } else {
                        used_before_assignment = true;
                    }
                } else if !ignore_implicit
                    && (trimmed == "binding" || method && trimmed == "super")
                    && direct_assignment.is_some()
                {
                    used_after_direct = true;
                }

                if starts_nested_scope(trimmed) {
                    depth += 1;
                }
            }

            if used_before_assignment || !used_after_direct {
                continue;
            }
            let Some(assignment_range) = direct_assignment else {
                continue;
            };
            let range = if nested_assignment_before_direct {
                declaration_range
            } else {
                assignment_range
            };
            context.report(
                format!(
                    "Argument `{argument}` was shadowed by a local variable before it was used."
                ),
                range,
            );
        }
    }
}

fn identifier_occurrences(line: &str, identifier: &str) -> Vec<usize> {
    line.match_indices(identifier)
        .filter_map(|(start, _)| {
            let before = line[..start].chars().next_back();
            let after = line[start + identifier.len()..].chars().next();
            (!before.is_some_and(|character| character.is_ascii_alphanumeric() || character == '_')
                && !after
                    .is_some_and(|character| character.is_ascii_alphanumeric() || character == '_'))
            .then_some(start)
        })
        .collect()
}

fn argument_assignment(line: &str, argument: &str) -> Option<(usize, usize, bool)> {
    for at in identifier_occurrences(line, argument) {
        let rest = line[at + argument.len()..].trim_start();
        if rest.starts_with("||=") || rest.starts_with("&&=") {
            return Some((at, at, false));
        }
        if rest.starts_with('=') && !rest.starts_with("==") && !rest.starts_with("=>") {
            let equal = line[at + argument.len()..].find('=')? + at + argument.len();
            return Some((at, equal + 1, line[..at].trim_end().ends_with('*')));
        }
        if line[..at].trim_end().ends_with('*') && line[at + argument.len()..].contains('=') {
            let equal = line[at + argument.len()..].find('=')? + at + argument.len();
            return Some((at, equal + 1, true));
        }
    }
    None
}

fn starts_nested_scope(trimmed: &str) -> bool {
    [
        "if ", "unless ", "case ", "begin", "for ", "while ", "until ",
    ]
    .iter()
    .any(|keyword| trimmed.starts_with(keyword))
        || trimmed.ends_with(" do")
        || trimmed.contains(" do |")
}

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
            let in_heredoc = heredocs
                .iter()
                .any(|range| range.start <= start && end <= range.end)
                && !file.line(start).contains("<<");
            let in_literal = literals
                .iter()
                .any(|range| range.start <= start && end <= range.end);
            let previous = context.source()[..start].chars().next_back();
            let symbol = previous == Some(':')
                && context.source().as_bytes().get(start.saturating_sub(2)) != Some(&b':');
            let variable = matches!(previous, Some('@' | '$'));
            let token_start = context.source()[..start]
                .rfind(|character: char| !character.is_alphanumeric() && character != '_')
                .map_or(0, |offset| offset + 1);
            let token = &context.source()[token_start..end];
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
    value.trim().trim_matches('/').to_string()
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
        if regex::RegexBuilder::new(matcher)
            .case_insensitive(true)
            .build()
            .is_ok_and(|regex| regex.is_match(context.path()))
        {
            found.push((term.as_str(), config));
        }
    }
    if found.is_empty() {
        return;
    }
    let names = found
        .iter()
        .map(|(term, _)| format!("'{term}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let replacement = if found.len() == 1 {
        match found[0].1.suggestions.as_slice() {
            [] => "another term".to_string(),
            [only] => format!("'{only}'"),
            _ => "other terms".to_string(),
        }
    } else {
        "other terms".to_string()
    };
    context.report(
        format!("Consider replacing {names} in file path with {replacement}."),
        0..0,
    );
}
