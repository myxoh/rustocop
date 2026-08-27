use super::catalog_cop::compatibility_custom;
use super::*;
use std::collections::HashSet;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(DuplicateRegexpCharacterClassElement),
        Box::new(RedundantRegexpQuantifiers),
        Box::new(UnescapedBracketInRegexp),
        Box::new(AmbiguousRegexpLiteral),
        compatibility_custom("Lint/OutOfRangeRegexpRef", out_of_range_ref),
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
    if regexp_character_is_escaped(source, quantifier_start) {
        return None;
    }
    let quantifier = greedy_quantifier(source, quantifier_start)?;
    (quantifier.end == significant_end).then_some(quantifier)
}

fn regexp_character_is_escaped(source: &str, at: usize) -> bool {
    let mut slashes = 0;
    let mut cursor = at;
    while cursor > 0 && source.as_bytes().get(cursor - 1) == Some(&b'\\') {
        slashes += 1;
        cursor -= 1;
    }
    slashes % 2 == 1
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

struct AmbiguousRegexpLiteral;

impl Cop for AmbiguousRegexpLiteral {
    fn name(&self) -> &'static str { "Lint/AmbiguousRegexpLiteral" }

    fn on_node<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context) {
        let Some(call) = node.as_call_node() else { return };
        if call.opening_loc().is_some() { return; }
        let name = call.name();
        if call.equal_loc().is_some()
            || matches!(name.as_slice(), b"=~" | b"!~" | b"==" | b"===" | b"!=" | b"<=" | b">=")
            || name.as_slice().ends_with(b"=")
        {
            return;
        }
        let Some(arguments) = call.arguments() else { return };
        let Some(first) = arguments.arguments().iter().next() else { return };
        let Some(opening) = leading_regexp_opening(&first) else { return };
        if !context.parser_warning_at(opening.start_offset(), "ambiguous `/`") {
            return;
        }
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        let message = "Ambiguous regexp literal. Parenthesize the method arguments if it's surely a regexp literal, or add a whitespace to the right of the `/` if it should be a division.";
        let start = arguments.location().start_offset();
        let end = arguments.location().end_offset();
        cop_context.add_offense(opening, message, |corrector| {
            let grouped_match = first.as_call_node().is_some_and(|operator| matches!(operator.name().as_slice(), b"=~" | b"!~"));
            let opening_edit = if grouped_match {
                start..start
            } else {
                call.message_loc().map_or(start..start, |message| message.end_offset()..start)
            };
            corrector.replace(opening_edit, "(");
            corrector.replace(end..end, ")");
        });
    }
}

fn leading_regexp_opening<'pr>(node: &Node<'pr>) -> Option<ruby_prism::Location<'pr>> {
    if let Some(regexp) = node.as_regular_expression_node() { return Some(regexp.opening_loc()); }
    if let Some(regexp) = node.as_interpolated_regular_expression_node() { return Some(regexp.opening_loc()); }
    node.as_call_node().and_then(|call| call.receiver()).and_then(|receiver| leading_regexp_opening(&receiver))
}

struct DuplicateRegexpCharacterClassElement;

impl Cop for DuplicateRegexpCharacterClassElement {
    fn name(&self) -> &'static str { "Lint/DuplicateRegexpCharacterClassElement" }

    fn on_node<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context) {
        let location = if let Some(regexp) = node.as_regular_expression_node() {
            regexp.location()
        } else if let Some(regexp) = node.as_interpolated_regular_expression_node() {
            regexp.location()
        } else { return };
        let literal = &source[location.start_offset()..location.end_offset()];
        let mut cop_context = context.cop_context(self.name(), source, ancestors);
        for duplicate in duplicate_regexp_class_tokens(literal) {
            let range = location.start_offset() + duplicate.start..location.start_offset() + duplicate.end;
            cop_context.remove("Duplicate element inside regexp character class", range.clone(), range);
        }
    }
}

#[allow(clippy::cognitive_complexity)]
fn duplicate_regexp_class_tokens(literal: &str) -> Vec<std::ops::Range<usize>> {
    let bytes = literal.as_bytes();
    let mut duplicates = Vec::new();
    // In `%r[...]`, the first `[` is the regexp delimiter, not a character
    // class. Prism locations include the literal delimiters.
    let mut index = if bytes.starts_with(b"%r") { 3 } else { 1 };
    while index < bytes.len() {
        if bytes.get(index..index + 2) == Some(b"#{") {
            let mut depth = 1usize;
            index += 2;
            while index < bytes.len() && depth > 0 {
                if bytes[index] == b'{' {
                    depth += 1;
                } else if bytes[index] == b'}' {
                    depth -= 1;
                }
                index += 1;
            }
            continue;
        }
        if bytes[index] != b'[' || regexp_byte_is_escaped(bytes, index) {
            index += 1;
            continue;
        }
        let class_start = index + 1;
        let mut end = class_start;
        let mut interpolation = 0usize;
        let mut class_depth = 1usize;
        while end < bytes.len() {
            if bytes.get(end..end + 2) == Some(b"#{") { interpolation += 1; end += 2; continue; }
            if interpolation > 0 {
                if bytes[end] == b'}' { interpolation -= 1; }
                end += 1;
                continue;
            }
            if bytes.get(end..end + 2) == Some(b"[:") {
                if let Some(close) = literal[end + 2..].find(":]") { end += close + 4; continue; }
            }
            if bytes[end] == b'\\' {
                end = regexp_escape_end(bytes, end, bytes.len());
                continue;
            }
            if bytes[end] == b'[' {
                class_depth += 1;
            } else if bytes[end] == b']' {
                class_depth -= 1;
                if class_depth == 0 {
                    break;
                }
            }
            end += 1;
        }
        if end >= bytes.len() { break; }
        let body = &literal[class_start..end];
        if body.contains("&&") { index = end + 1; continue; }
        let mut seen = HashSet::<String>::new();
        let mut at = class_start;
        // A caret at the beginning negates the set; it is metadata, not an
        // element that can duplicate a later literal caret.
        if at < end && bytes[at] == b'^' {
            at += 1;
        }
        while at < end {
            if bytes.get(at..at + 2) == Some(b"#{") {
                let mut depth = 1usize; at += 2;
                while at < end && depth > 0 { if bytes[at] == b'{' { depth += 1; } else if bytes[at] == b'}' { depth -= 1; } at += 1; }
                continue;
            }
            if bytes[at] == b'[' && bytes.get(at..at + 2) != Some(b"[:") {
                let mut nested_depth = 1usize;
                at += 1;
                while at < end && nested_depth > 0 {
                    if bytes.get(at..at + 2) == Some(b"#{") {
                        let mut interpolation_depth = 1usize;
                        at += 2;
                        while at < end && interpolation_depth > 0 {
                            if bytes[at] == b'{' {
                                interpolation_depth += 1;
                            } else if bytes[at] == b'}' {
                                interpolation_depth -= 1;
                            }
                            at += 1;
                        }
                    } else if bytes[at] == b'\\' {
                        at = regexp_escape_end(bytes, at, end);
                    } else {
                        if bytes[at] == b'[' {
                            nested_depth += 1;
                        } else if bytes[at] == b']' {
                            nested_depth -= 1;
                        }
                        at += 1;
                    }
                }
                continue;
            }
            let token_start = at;
            if bytes.get(at..at + 2) == Some(b"[:") {
                if let Some(close) = literal[at + 2..end].find(":]") { at += close + 4; } else { at += 1; }
            } else if bytes[at] == b'\\' {
                at = regexp_escape_end(bytes, at, end);
            } else {
                let char_len = literal[at..].chars().next().map_or(1, char::len_utf8);
                at = (at + char_len).min(end);
            }
            if at < end && bytes[at] == b'-' && at + 1 < end && bytes[token_start] != b'-' {
                at += 1;
                if bytes[at] == b'\\' {
                    at = regexp_escape_end(bytes, at, end);
                } else {
                    at += literal[at..].chars().next().map_or(1, char::len_utf8);
                }
            }
            let token = literal[token_start..at].to_string();
            if !seen.insert(token) { duplicates.push(token_start..at); }
        }
        index = end + 1;
    }
    duplicates
}

fn regexp_byte_is_escaped(bytes: &[u8], index: usize) -> bool {
    let mut backslashes = 0usize;
    let mut cursor = index;
    while cursor > 0 && bytes[cursor - 1] == b'\\' {
        backslashes += 1;
        cursor -= 1;
    }
    backslashes % 2 == 1
}

fn regexp_escape_end(bytes: &[u8], start: usize, limit: usize) -> usize {
    let mut at = start + 1;
    if at >= limit {
        return limit;
    }
    match bytes[at] {
        b'0'..=b'7' => {
            let mut digits = 0;
            while at < limit && digits < 3 && matches!(bytes[at], b'0'..=b'7') {
                at += 1;
                digits += 1;
            }
        }
        b'x' => {
            at += 1;
            for _ in 0..2 {
                if at < limit && bytes[at].is_ascii_hexdigit() {
                    at += 1;
                }
            }
        }
        b'u' => {
            at += 1;
            if at < limit && bytes[at] == b'{' {
                at += 1;
                while at < limit && bytes[at] != b'}' {
                    at += 1;
                }
                at = (at + 1).min(limit);
            } else {
                for _ in 0..4 {
                    if at < limit && bytes[at].is_ascii_hexdigit() {
                        at += 1;
                    }
                }
            }
        }
        b'p' | b'P' if bytes.get(at + 1) == Some(&b'{') => {
            at += 2;
            while at < limit && bytes[at] != b'}' {
                at += 1;
            }
            at = (at + 1).min(limit);
        }
        _ => at = (at + 1).min(limit),
    }
    at
}

fn out_of_range_ref(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    let parsed = context.prism_result();
    let mut collector = OutOfRangeRefCollector {
        source: context.source(),
        valid_ref: Some(0),
        offenses: Vec::new(),
    };
    collector.visit(&parsed.node());
    for (range, reference, captures) in collector.offenses {
        let groups = match captures {
            0 => "no regexp capture groups detected".to_string(),
            1 => "1 regexp capture group detected".to_string(),
            count => format!("{count} regexp capture groups detected"),
        };
        context.report(
            format!("${reference} is out of range ({groups})."),
            range,
        );
    }
}

struct OutOfRangeRefCollector<'a> {
    source: &'a str,
    valid_ref: Option<usize>,
    offenses: Vec<(std::ops::Range<usize>, usize, usize)>,
}

impl<'pr> Visit<'pr> for OutOfRangeRefCollector<'_> {
    fn visit_numbered_reference_read_node(
        &mut self,
        node: &ruby_prism::NumberedReferenceReadNode<'pr>,
    ) {
        let location = node.location();
        let source = &self.source[location.start_offset() + 1..location.end_offset()];
        if let (Ok(reference), Some(captures)) = (source.parse::<usize>(), self.valid_ref) {
            if reference > captures {
                self.offenses.push((
                    location.start_offset()..location.end_offset(),
                    reference,
                    captures,
                ));
            }
        }
        ruby_prism::visit_numbered_reference_read_node(self, node);
    }

    fn visit_call_node(&mut self, node: &CallNode<'pr>) {
        let method = call_name(node);
        let preserved_ref = self.valid_ref;
        let receiver_methods = [b"=~".as_slice(), b"===", b"match"];
        let argument_methods = [
            b"=~".as_slice(),
            b"match",
            b"grep",
            b"gsub",
            b"gsub!",
            b"sub",
            b"sub!",
            b"[]",
            b"slice",
            b"slice!",
            b"index",
            b"rindex",
            b"scan",
            b"partition",
            b"rpartition",
            b"start_with?",
            b"end_with?",
        ];
        if let Some(receiver) = node.receiver() {
            self.visit(&receiver);
        }
        if let Some(arguments) = node.arguments() {
            self.visit(&arguments.as_node());
        }
        let relevant = receiver_methods.contains(&method) || argument_methods.contains(&method);
        let regexp = if argument_methods.contains(&method) {
            node.arguments()
                .and_then(|arguments| arguments.arguments().iter().next())
                .and_then(|argument| argument.as_regular_expression_node())
                .or_else(|| {
                    receiver_methods
                        .contains(&method)
                        .then(|| node.receiver())
                        .flatten()
                        .and_then(|receiver| receiver.as_regular_expression_node())
                })
        } else {
            node.receiver()
                .and_then(|receiver| receiver.as_regular_expression_node())
        };
        if relevant {
            self.valid_ref = None;
        }
        if let Some(regexp) = regexp {
            let body = &self.source
                [regexp.content_loc().start_offset()..regexp.content_loc().end_offset()];
            self.valid_ref = Some(regexp_capture_count(&format!("/{body}/")));
        }
        if let Some(block) = node.block() {
            self.visit(&block);
        }
        if method == b"match?" {
            self.valid_ref = preserved_ref;
        }
    }

    fn visit_when_node(&mut self, node: &ruby_prism::WhenNode<'pr>) {
        let mut captures = Vec::new();
        for condition in node.conditions().iter() {
            self.visit(&condition);
            if let Some(regexp) = condition.as_regular_expression_node() {
                captures.push(regexp_capture_count_from_node(self.source, &regexp));
            }
        }
        self.valid_ref = captures.into_iter().max();
        if let Some(statements) = node.statements() {
            self.visit(&statements.as_node());
        }
    }

    fn visit_in_node(&mut self, node: &ruby_prism::InNode<'pr>) {
        let pattern = node.pattern();
        self.visit(&pattern);
        let mut regexps = PatternRegexpCollector {
            source: self.source,
            captures: Vec::new(),
        };
        regexps.visit(&pattern);
        self.valid_ref = regexps.captures.into_iter().max();
        if let Some(statements) = node.statements() {
            self.visit(&statements.as_node());
        }
    }
}

struct PatternRegexpCollector<'a> {
    source: &'a str,
    captures: Vec<usize>,
}

impl<'pr> Visit<'pr> for PatternRegexpCollector<'_> {
    fn visit_regular_expression_node(
        &mut self,
        node: &ruby_prism::RegularExpressionNode<'pr>,
    ) {
        self.captures
            .push(regexp_capture_count_from_node(self.source, node));
    }
}

fn regexp_capture_count_from_node(
    source: &str,
    regexp: &ruby_prism::RegularExpressionNode<'_>,
) -> usize {
    let body = &source[regexp.content_loc().start_offset()..regexp.content_loc().end_offset()];
    regexp_capture_count(&format!("/{body}/"))
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
