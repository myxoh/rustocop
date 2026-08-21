use super::*;

pub(super) fn numeric_literals(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_integer_node().is_none() && node.as_float_node().is_none() {
        return;
    }
    let location = node.location();
    let line_start = context.source_file().line_start(location.start_offset());
    let line_end = context.source()[line_start..]
        .find('\n')
        .map_or(context.source().len(), |at| line_start + at);
    if context.source()[line_start..line_end].contains("rubocop:disable Style/NumericLiterals") {
        return;
    }
    let source = context.source_file().at(&location);
    let unsigned = source.strip_prefix('-').unwrap_or(source);
    let integer = unsigned.split(['.', 'e', 'E']).next().unwrap_or(unsigned);
    if integer.starts_with('0') {
        return;
    }
    if context
        .config_values("AllowedPatterns")
        .iter()
        .any(|pattern| {
            let pattern = pattern.replace("\\\\", "\\");
            regex::Regex::new(&format!("^(?:{pattern})$"))
                .is_ok_and(|pattern| pattern.is_match(integer))
        })
    {
        return;
    }
    let digits = integer.replace('_', "");
    let minimum = context.config_usize("MinDigits", 5);
    if digits.len() < minimum || context.config_values("AllowedNumbers").contains(&digits) {
        return;
    }
    let groups = integer.split('_').collect::<Vec<_>>();
    let strict = context.config_bool("Strict", false);
    let valid = groups.len() > 1
        && groups
            .first()
            .is_some_and(|group| (1..=3).contains(&group.len()))
        && groups.iter().skip(1).all(|group| group.len() == 3);
    let tolerated = !strict
        && groups.len() > 2
        && groups[1..groups.len() - 1]
            .iter()
            .all(|group| group.len() == 3);
    if valid || tolerated {
        return;
    }
    let mut formatted = String::new();
    for (index, byte) in digits.bytes().rev().enumerate() {
        if index > 0 && index % 3 == 0 {
            formatted.push('_');
        }
        formatted.push(byte as char);
    }
    let formatted = formatted.chars().rev().collect::<String>();
    let replacement = source.replacen(integer, &formatted, 1);
    context.replace(
        "Use underscores(_) as thousands separator and separate every 3 digits with them.",
        &location,
        &location,
        replacement,
    );
}

pub(super) fn command_literal(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if node.as_x_string_node().is_none() && node.as_interpolated_x_string_node().is_none() {
        return;
    }
    let location = node.location();
    let source = context.source_file().at(&location);
    if source.starts_with("<<") {
        return;
    }
    let backticks = source.starts_with('`');
    let contains_backtick =
        source[usize::from(backticks)..source.len().saturating_sub(1)].contains('`');
    let allow_inner_backticks = context.config_bool("AllowInnerBackticks", false);
    let style = context.policy().enforced_style("backticks");
    let allowed = match style {
        "backticks" => {
            backticks && (!contains_backtick || allow_inner_backticks)
                || !backticks && contains_backtick && !allow_inner_backticks
        }
        "percent_x" => !backticks,
        "mixed" => {
            backticks && !source.contains('\n') && (!contains_backtick || allow_inner_backticks)
                || !backticks
                    && (source.contains('\n') || contains_backtick && !allow_inner_backticks)
        }
        _ => true,
    };
    if allowed {
        return;
    }
    let (message, replacement) = if backticks {
        let body = source.trim_matches('`');
        let replacement = if contains_backtick {
            None
        } else {
            let delimiters = context
                .related_config_map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
                .and_then(|values| values.get("%x").or_else(|| values.get("default")))
                .map(String::as_str)
                .unwrap_or("()");
            let (open, close) = delimiters.split_at(1);
            Some(format!("%x{open}{body}{close}"))
        };
        ("Use `%x` around command string.", replacement)
    } else {
        let body = &source[3..source.len() - 1];
        let replacement = (!contains_backtick).then(|| format!("`{body}`"));
        ("Use backticks around command string.", replacement)
    };
    if let Some(replacement) = replacement {
        context.replace(message, &location, &location, replacement);
    } else {
        context.report(message, &location);
    }
}
