use super::catalog_cop::{custom, report};
use super::*;
use std::collections::HashSet;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom(
            "Lint/DuplicateRegexpCharacterClassElement",
            duplicate_character_class,
        ),
        Box::new(RedundantRegexpQuantifiers),
        Box::new(UnescapedBracketInRegexp),
        report(
            "Lint/AmbiguousRegexpLiteral",
            "puts /",
            "Ambiguous regexp literal. Parenthesize the method arguments.",
        ),
        report(
            "Lint/ArrayLiteralInRegexp",
            "Regexp.new([",
            "Passing an array to `Regexp.new` is invalid.",
        ),
        custom("Lint/OutOfRangeRegexpRef", out_of_range_ref),
        Box::new(SelectByRegexp),
    ]
}

struct RedundantRegexpQuantifiers;

impl Cop for RedundantRegexpQuantifiers {
    fn name(&self) -> &'static str {
        "Lint/RedundantRegexpQuantifiers"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(regexp) = node.as_regular_expression_node() else {
            return;
        };
        let content = regexp.content_loc();
        let body = &source[content.start_offset()..content.end_offset()];
        let closing =
            &source[regexp.closing_loc().start_offset()..regexp.closing_loc().end_offset()];
        let extended = closing
            .get(1..)
            .is_some_and(|options| options.contains('x'));
        let mut offenses = redundant_quantifier_offenses(body, extended);
        let combined_correction =
            (offenses.len() > 1).then(|| reduce_redundant_quantifiers(body.to_string(), extended));
        offenses.sort_by_key(|(inner, _, _)| std::cmp::Reverse(inner.start));
        for (inner, outer, replacement) in offenses {
            let absolute_inner = content.start_offset() + inner.start;
            let offense = absolute_inner..content.start_offset() + outer.end;
            let preserved = &body[inner.end..outer.start];
            let (edit, corrected) = combined_correction.as_ref().map_or_else(
                || (offense.clone(), format!("{replacement}{preserved}")),
                |corrected| {
                    (
                        content.start_offset()..content.end_offset(),
                        corrected.clone(),
                    )
                },
            );
            context.replace(
                self.name(),
                format!(
                    "Replace redundant quantifiers `{}` and `{}` with a single `{replacement}`.",
                    inner.source, outer.source
                ),
                offense,
                edit,
                corrected,
            );
        }
    }
}

fn redundant_quantifier_offenses<'a>(
    body: &'a str,
    extended: bool,
) -> Vec<(GreedyQuantifier<'a>, GreedyQuantifier<'a>, &'static str)> {
    let mut offenses = Vec::new();
    let mut seen_groups = HashSet::new();
    let mut search = 0;
    while let Some(relative) = body[search..].find("(?:") {
        let group_start = search + relative;
        search = group_start + 3;
        if seen_groups.contains(&group_start) {
            continue;
        }
        let Some(close) = matching_regexp_group(body, group_start) else {
            break;
        };
        let outer_start = skip_regexp_whitespace(body, close + 1, extended);
        let Some(outer) = greedy_quantifier(body, outer_start) else {
            continue;
        };
        for inner in quantifier_chain(body, group_start + 3, close, extended, &mut seen_groups) {
            let replacement = if inner.normalized == outer.normalized {
                inner.normalized
            } else {
                "*"
            };
            offenses.push((inner, outer, replacement));
        }
    }
    offenses
}

fn reduce_redundant_quantifiers(mut body: String, extended: bool) -> String {
    loop {
        let mut offenses = redundant_quantifier_offenses(&body, extended);
        offenses.sort_by_key(|(inner, _, _)| std::cmp::Reverse(inner.start));
        let Some((inner, outer, replacement)) = offenses.first().copied() else {
            return body;
        };
        let preserved = body[inner.end..outer.start].to_string();
        body.replace_range(inner.start..outer.end, &format!("{replacement}{preserved}"));
    }
}

fn matching_regexp_group(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut depth = 0_usize;
    let mut in_class = false;
    let mut index = start;
    while index < bytes.len() {
        if bytes[index] == b'\\' {
            index += 2;
            continue;
        }
        if bytes[index] == b'[' {
            in_class = true;
        } else if bytes[index] == b']' && in_class {
            in_class = false;
        } else if !in_class && bytes[index] == b'(' {
            depth += 1;
        } else if !in_class && bytes[index] == b')' {
            depth -= 1;
            if depth == 0 {
                return Some(index);
            }
        }
        index += 1;
    }
    None
}

#[derive(Clone, Copy)]
struct GreedyQuantifier<'a> {
    start: usize,
    end: usize,
    source: &'a str,
    normalized: &'static str,
}

fn greedy_quantifier(source: &str, start: usize) -> Option<GreedyQuantifier<'_>> {
    let first = source.get(start..start + 1)?;
    let (end, normalized) = match first {
        "+" => (start + 1, "+"),
        "*" => (start + 1, "*"),
        "?" => (start + 1, "?"),
        "{" => {
            let relative_end = source.get(start + 1..)?.find('}')?;
            let end = start + relative_end + 2;
            let interval = source.get(start..end)?;
            let normalized = match interval {
                "{1,}" => "+",
                "{0,}" | "{,}" => "*",
                "{0,1}" | "{,1}" => "?",
                _ => return None,
            };
            (end, normalized)
        }
        _ => return None,
    };
    if source
        .get(end..end + 1)
        .is_some_and(|suffix| matches!(suffix, "+" | "?"))
    {
        return None;
    }
    Some(GreedyQuantifier {
        start,
        end,
        source: source.get(start..end)?,
        normalized,
    })
}

fn trailing_greedy_quantifier(
    source: &str,
    start: usize,
    end: usize,
    extended: bool,
) -> Option<GreedyQuantifier<'_>> {
    let significant_end = trim_regexp_whitespace_end(source, start, end, extended);
    let last = source.get(significant_end.checked_sub(1)?..significant_end)?;
    let quantifier_start = if matches!(last, "+" | "*" | "?") {
        significant_end - 1
    } else if last == "}" {
        source.get(start..significant_end)?.rfind('{')? + start
    } else {
        return None;
    };
    let quantifier = greedy_quantifier(source, quantifier_start)?;
    (quantifier.end == significant_end).then_some(quantifier)
}

fn quantifier_chain<'a>(
    source: &'a str,
    start: usize,
    end: usize,
    extended: bool,
    seen_groups: &mut HashSet<usize>,
) -> Vec<GreedyQuantifier<'a>> {
    let content_start = skip_regexp_whitespace(source, start, extended);
    let content_end = trim_regexp_whitespace_end(source, content_start, end, extended);
    let quantifier = trailing_greedy_quantifier(source, content_start, content_end, extended);
    let atom_end = trim_regexp_whitespace_end(
        source,
        content_start,
        quantifier.map_or(content_end, |item| item.start),
        extended,
    );
    let Some(atom) = source.get(content_start..atom_end) else {
        return Vec::new();
    };
    if single_regexp_atom(atom) {
        return quantifier.into_iter().collect();
    }
    let Some((nested_start, nested_end)) =
        noncapturing_group_content(source, content_start, atom_end)
    else {
        return Vec::new();
    };
    seen_groups.insert(content_start);
    let mut quantifiers: Vec<_> = quantifier.into_iter().collect();
    quantifiers.extend(quantifier_chain(
        source,
        nested_start,
        nested_end,
        extended,
        seen_groups,
    ));
    quantifiers
}

fn skip_regexp_whitespace(source: &str, mut at: usize, extended: bool) -> usize {
    if extended {
        while source
            .as_bytes()
            .get(at)
            .is_some_and(u8::is_ascii_whitespace)
        {
            at += 1;
        }
    }
    at
}

fn trim_regexp_whitespace_end(source: &str, start: usize, mut end: usize, extended: bool) -> usize {
    if extended {
        while end > start
            && source
                .as_bytes()
                .get(end - 1)
                .is_some_and(u8::is_ascii_whitespace)
        {
            end -= 1;
        }
    }
    end
}

fn noncapturing_group_content(source: &str, start: usize, end: usize) -> Option<(usize, usize)> {
    let group = source.get(start..end)?;
    if !group.starts_with("(?:") {
        return None;
    }
    let close = matching_regexp_group(group, 0)?;
    (close + 1 == group.len()).then_some((start + 3, start + close))
}

fn single_regexp_atom(source: &str) -> bool {
    let bytes = source.as_bytes();
    if source.chars().count() == 1 {
        return true;
    }
    if bytes.first() == Some(&b'\\') {
        return bytes.len() == 2;
    }
    if bytes.first() == Some(&b'[') && bytes.last() == Some(&b']') {
        let mut escaped = false;
        return bytes[1..bytes.len() - 1].iter().all(|byte| {
            if escaped {
                escaped = false;
                true
            } else if *byte == b'\\' {
                escaped = true;
                true
            } else {
                *byte != b']'
            }
        });
    }
    false
}

struct UnescapedBracketInRegexp;

impl Cop for UnescapedBracketInRegexp {
    fn name(&self) -> &'static str {
        "Lint/UnescapedBracketInRegexp"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let (start, end) = if let Some(regexp) = node.as_regular_expression_node() {
            (
                regexp.opening_loc().end_offset(),
                regexp.closing_loc().start_offset(),
            )
        } else if let Some(call) = node.as_call_node() {
            if !matches!(call_name(&call), b"new" | b"compile")
                || !root_constant(call.receiver(), b"Regexp")
            {
                return;
            }
            let Some(string) = first_argument(&call).and_then(|argument| argument.as_string_node())
            else {
                return;
            };
            let content = string.content_loc();
            (content.start_offset(), content.end_offset())
        } else {
            return;
        };
        let mut escaped = false;
        let mut character_class_depth = 0_usize;
        for (relative, byte) in source[start..end].bytes().enumerate() {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'[' {
                character_class_depth += 1;
            } else if byte == b']' {
                let string_escaped_bracket = node.as_call_node().is_some()
                    && relative >= 2
                    && source.as_bytes().get(start + relative - 2..start + relative)
                        == Some(&b"\\\\"[..]);
                if string_escaped_bracket {
                    continue;
                } else if character_class_depth > 0 {
                    character_class_depth -= 1;
                } else if node.as_call_node().is_some()
                    && relative > 0
                    && source.as_bytes().get(start + relative - 1) == Some(&b'\\')
                {
                    // `Regexp.new` receives the interpreted string. In a Ruby
                    // string, even a source-level doubled slash can escape the
                    // closing bracket in the resulting regular expression.
                    continue;
                } else if relative > 0 {
                    let at = start + relative;
                    context.replace(
                        self.name(),
                        "Regular expression has `]` without escape.",
                        at..at + 1,
                        at..at + 1,
                        "\\]",
                    );
                }
            }
        }
    }
}

struct SelectByRegexp;

enum BlockParameter {
    Named(Vec<u8>),
    Numbered,
    It,
}

struct RegexpSelection<'pr> {
    pattern: Node<'pr>,
    negated: bool,
}

impl Cop for SelectByRegexp {
    fn name(&self) -> &'static str {
        "Style/SelectByRegexp"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let original = call.name().as_slice();
        if !matches!(original, b"select" | b"filter" | b"find_all" | b"reject") {
            return;
        }
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        if original == b"filter" && !cop_context.target_ruby_version().at_least(2, 6) {
            return;
        }
        let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
            return;
        };
        if call.receiver().as_ref().is_some_and(hash_like_receiver) {
            return;
        }
        let Some(parameter) = select_block_parameter(&block) else {
            return;
        };
        let Some(body) = block.body().and_then(single_expression) else {
            return;
        };
        let Some(selection) = regexp_selection(body, &parameter) else {
            return;
        };
        let selecting = matches!(original, b"select" | b"filter" | b"find_all");
        let replacement = if selecting == selection.negated {
            "grep_v"
        } else {
            "grep"
        };
        if replacement == "grep_v" && !cop_context.target_ruby_version().at_least(2, 3) {
            return;
        }
        let original = String::from_utf8_lossy(original);
        let Some(selector) = call.message_loc() else {
            return;
        };
        cop_context.replace(
            format!("Prefer `{replacement}` to `{original}` with a regexp match."),
            call.location(),
            selector.start_offset()..block.location().end_offset(),
            format!(
                "{replacement}({})",
                source_at(source, &selection.pattern.location())
            ),
        );
    }
}

fn select_block_parameter(block: &ruby_prism::BlockNode<'_>) -> Option<BlockParameter> {
    let parameters = block.parameters()?;
    if let Some(numbered) = parameters.as_numbered_parameters_node() {
        return (numbered.maximum() == 1).then_some(BlockParameter::Numbered);
    }
    if parameters.as_it_parameters_node().is_some() {
        return Some(BlockParameter::It);
    }
    let block_parameters = parameters.as_block_parameters_node()?;
    let parameters = block_parameters.parameters()?;
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    let parameter = parameters
        .requireds()
        .first()?
        .as_required_parameter_node()?;
    Some(BlockParameter::Named(parameter.name().as_slice().to_vec()))
}

fn regexp_selection<'pr>(
    body: Node<'pr>,
    parameter: &BlockParameter,
) -> Option<RegexpSelection<'pr>> {
    let (body, explicitly_negated) = unwrap_regexp_negation(body)?;
    let call = body
        .as_match_write_node()
        .map(|write| write.call())
        .or_else(|| body.as_call_node())?;
    let method = call.name().as_slice();
    if !matches!(method, b"match?" | b"=~" | b"!~") {
        return None;
    }
    if method == b"match?" && call.receiver().is_none() {
        return None;
    }
    let receiver = call.receiver()?;
    let arguments = call.arguments()?;
    let mut arguments = arguments.arguments().iter();
    let argument = arguments.next()?;
    if arguments.next().is_some() {
        return None;
    }
    let pattern = if is_select_parameter(&receiver, parameter) {
        argument
    } else if is_select_parameter(&argument, parameter) {
        receiver
    } else {
        return None;
    };
    Some(RegexpSelection {
        pattern,
        negated: explicitly_negated || method == b"!~",
    })
}

fn unwrap_regexp_negation(mut node: Node<'_>) -> Option<(Node<'_>, bool)> {
    let mut negated = false;
    if let Some(call) = node.as_call_node() {
        if call.name().as_slice() == b"!" {
            if call
                .arguments()
                .is_some_and(|arguments| !arguments.arguments().is_empty())
            {
                return None;
            }
            node = call.receiver()?;
            negated = true;
        }
    }
    if let Some(parentheses) = node.as_parentheses_node() {
        node = parentheses.body().and_then(single_expression)?;
    }
    Some((node, negated))
}

fn is_select_parameter(node: &Node<'_>, parameter: &BlockParameter) -> bool {
    match parameter {
        BlockParameter::Named(name) => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == name),
        BlockParameter::Numbered => node
            .as_local_variable_read_node()
            .is_some_and(|read| read.name().as_slice() == b"_1"),
        BlockParameter::It => node.as_it_local_variable_read_node().is_some(),
    }
}

fn hash_like_receiver(node: &Node<'_>) -> bool {
    if node.as_hash_node().is_some() || node_is_root_constant(node, b"ENV") {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        matches!(call.name().as_slice(), b"to_h" | b"to_hash")
            || matches!(call.name().as_slice(), b"new" | b"[]")
                && call
                    .receiver()
                    .as_ref()
                    .is_some_and(|receiver| node_is_root_constant(receiver, b"Hash"))
    })
}

fn regexp_ranges(source: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    source.match_indices('/').filter_map(|(start, _)| {
        source[start + 1..]
            .find('/')
            .map(|relative| (start, start + 1 + relative))
    })
}

fn duplicate_character_class(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    for (regexp_start, regexp_end) in regexp_ranges(&source) {
        let regexp = &source[regexp_start + 1..regexp_end];
        if regexp.contains('\\') || regexp.contains("&&") || regexp.contains("#{") {
            continue;
        }
        let Some(open) = regexp.find('[') else {
            continue;
        };
        let Some(close) = regexp[open + 1..].find(']').map(|at| open + 1 + at) else {
            continue;
        };
        let mut seen = HashSet::new();
        for (relative, character) in regexp[open + 1..close].char_indices() {
            if !seen.insert(character) {
                let start = regexp_start + 1 + open + 1 + relative;
                context.remove(
                    "Duplicate element inside regexp character class.",
                    start..start + character.len_utf8(),
                    start..start + character.len_utf8(),
                );
            }
        }
    }
}

fn out_of_range_ref(context: &mut CopContext<'_, '_>) {
    let regexp_literal = regex::Regex::new(r"/(?:\\.|[^/\n])+/[a-z]*").expect("regexp literal");
    let parsed = parse(context.source().as_bytes());
    let mut references = RegexpReferenceCollector::default();
    references.visit(&parsed.node());
    let reference_offsets = references.offsets;
    let mut captures = 0usize;
    let mut captures_known = true;
    for (offset, line) in context.source_file().lines() {
        let references_first = line.trim_start().starts_with('$');
        if references_first {
            report_out_of_range_references(
                context,
                offset,
                line,
                captures,
                captures_known,
                &reference_offsets,
            );
        }
        let literals = regexp_literal
            .find_iter(line)
            .filter(|matched| !matched.as_str().contains("#{"))
            .map(|matched| regexp_capture_count(matched.as_str()))
            .collect::<Vec<_>>();
        let matching_method = [
            ".match(",
            ".grep(",
            ".gsub(",
            ".gsub!(",
            ".sub(",
            ".sub!(",
            ".scan(",
            ".slice(",
            ".slice!(",
            ".index(",
            ".rindex(",
            ".partition(",
            ".rpartition(",
            ".start_with?(",
            ".end_with?(",
            "&.match(",
            "&.slice(",
            "&.slice!(",
            "&.index(",
            "&.rindex(",
            "&.partition(",
            "&.rpartition(",
            "&.start_with?(",
            "&.end_with?(",
        ]
        .iter()
        .any(|method| line.contains(method));
        let pattern_clause =
            line.trim_start().starts_with("when ") || line.trim_start().starts_with("in ");
        let bracket_match = line.contains("[/") || line.contains("\"[") || line.contains("'[");
        let matching_construct = line.contains("=~")
            || line.contains(" === ")
            || matching_method
            || pattern_clause
            || bracket_match;
        if !line.contains(".match?(") && matching_construct {
            if literals.is_empty() {
                captures_known = false;
            } else {
                captures_known = true;
                captures = if line.contains("=~") {
                    *literals.last().unwrap_or(&0)
                } else {
                    literals.into_iter().max().unwrap_or(0)
                };
            }
        }
        if !references_first {
            report_out_of_range_references(
                context,
                offset,
                line,
                captures,
                captures_known,
                &reference_offsets,
            );
        }
    }
}

#[derive(Default)]
struct RegexpReferenceCollector {
    offsets: HashSet<usize>,
}

impl<'pr> Visit<'pr> for RegexpReferenceCollector {
    fn visit_numbered_reference_read_node(
        &mut self,
        node: &ruby_prism::NumberedReferenceReadNode<'pr>,
    ) {
        self.offsets.insert(node.location().start_offset());
        ruby_prism::visit_numbered_reference_read_node(self, node);
    }

    fn visit_global_variable_read_node(
        &mut self,
        node: &ruby_prism::GlobalVariableReadNode<'pr>,
    ) {
        self.offsets.insert(node.location().start_offset());
        ruby_prism::visit_global_variable_read_node(self, node);
    }
}

fn report_out_of_range_references(
    context: &mut CopContext<'_, '_>,
    offset: usize,
    line: &str,
    captures: usize,
    captures_known: bool,
    reference_offsets: &HashSet<usize>,
) {
    if !captures_known {
        return;
    }
    for (at, _) in line.match_indices('$') {
        if !reference_offsets.contains(&(offset + at)) {
            continue;
        }
        let digits = line[at + 1..]
            .bytes()
            .take_while(u8::is_ascii_digit)
            .count();
        if digits == 0 {
            continue;
        }
        let Ok(reference) = line[at + 1..at + 1 + digits].parse::<usize>() else {
            continue;
        };
        if reference <= captures {
            continue;
        }
        let groups = match captures {
            0 => "no regexp capture groups detected".to_string(),
            1 => "1 regexp capture group detected".to_string(),
            count => format!("{count} regexp capture groups detected"),
        };
        context.report(
            format!("${reference} is out of range ({groups})."),
            offset + at..offset + at + digits + 1,
        );
    }
}

fn regexp_capture_count(literal: &str) -> usize {
    let end = literal[1..].rfind('/').map_or(literal.len(), |at| at + 1);
    let pattern = &literal[1..end];
    let bytes = pattern.as_bytes();
    let mut named = 0usize;
    let mut numbered = 0usize;
    let mut escaped = false;
    let mut in_class = false;
    for index in 0..bytes.len() {
        let byte = bytes[index];
        if escaped {
            escaped = false;
            continue;
        }
        if byte == b'\\' {
            escaped = true;
            continue;
        }
        if byte == b'[' {
            in_class = true;
            continue;
        }
        if byte == b']' {
            in_class = false;
            continue;
        }
        if byte != b'(' || in_class {
            continue;
        }
        if bytes.get(index + 1) != Some(&b'?') {
            numbered += 1;
        } else if bytes.get(index + 2) == Some(&b'<')
            && !matches!(bytes.get(index + 3), Some(b'=' | b'!'))
        {
            named += 1;
        }
    }
    if named > 0 {
        named
    } else {
        numbered
    }
}
