use super::*;

define_cops! {
    SymbolArray => "Style/SymbolArray" => source(symbol_array),
    QuotedSymbols => "Style/QuotedSymbols" => source(quoted_symbols),
    FetchEnvVar => "Style/FetchEnvVar" => source(fetch_env_var),
    SpecialGlobalVars => "Style/SpecialGlobalVars" => source(special_global_vars),
    StringConcatenation => "Style/StringConcatenation" => source(string_concatenation),
    RedundantLineContinuation => "Style/RedundantLineContinuation" => source(redundant_line_continuation),
    Lambda => "Style/Lambda" => source(lambda_literal),
    FormatString => "Style/FormatString" => source(format_string),
    WordArray => "Style/WordArray" => source(word_array),
    PercentLiteralDelimiters => "Style/PercentLiteralDelimiters" => source(percent_delimiters),
    RedundantStringEscape => "Style/RedundantStringEscape" => source(redundant_string_escape),
    SymbolProc => "Style/SymbolProc" => source(symbol_proc),
}

fn replace_code(context: &mut CopContext<'_, '_>, old: &str, new: &str, message: &str) {
    for start in context.source_file().code_offsets(old) {
        context.replace(
            message,
            start..start + old.len(),
            start..start + old.len(),
            new,
        );
    }
}

fn symbol_array(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("percent") != "percent" {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            continue;
        }
        let body = &trimmed[1..trimmed.len() - 1];
        let values = body.split(',').map(str::trim).collect::<Vec<_>>();
        if values.len() < 2 || values.iter().any(|value| !value.starts_with(':')) {
            continue;
        }
        if values
            .iter()
            .any(|value| value.contains(char::is_whitespace))
        {
            continue;
        }
        let replacement = format!(
            "%i[{}]",
            values
                .iter()
                .map(|value| value.trim_start_matches(':').trim_matches(['\'', '"']))
                .collect::<Vec<_>>()
                .join(" ")
        );
        let start = offset + line.find(trimmed).unwrap_or(0);
        context.replace(
            "Use `%i` or `%I` for an array of symbols.",
            start..start + trimmed.len(),
            start..start + trimmed.len(),
            replacement
                .replace("%i[", "%i(")
                .trim_end_matches(']')
                .to_string()
                + ")",
        );
    }
}

fn quoted_symbols(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    let mut search = 0;
    while let Some(relative) = source[search..].find(":\"") {
        let start = search + relative;
        let body_start = start + 2;
        let Some(close) = source[body_start..].find('"').map(|at| body_start + at) else {
            break;
        };
        let body = &source[body_start..close];
        if !body.contains('#')
            && !body.contains('\'')
            && !body.contains('\n')
            && !body.contains("\\n")
            && !body.contains("\\u")
            && !body.contains("\\x")
            && !body.contains("\\e")
        {
            context.replace("Prefer single-quoted symbols when you don't need string interpolation or special symbols.", start..close + 1, start..close + 1, format!(":'{}'", body.replace("\\\"", "\"").replace('\\', "\\\\")));
        }
        search = close + 1;
    }
}

fn fetch_env_var(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    for quote in ['\'', '"'] {
        let needle = format!("ENV[{quote}");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let start = search + relative;
            let value_start = start + needle.len();
            let Some(close) = source[value_start..].find(quote).map(|at| value_start + at) else {
                break;
            };
            if source.as_bytes().get(close + 1) != Some(&b']') {
                search = close + 1;
                continue;
            }
            let key = &source[value_start..close];
            let before = source[..start].trim_end().as_bytes().last().copied();
            let after = source[close + 2..].trim_start();
            if before == Some(b'!')
                || after.starts_with(['.', '&'])
                || after.starts_with("==")
                || after.starts_with("!=")
                || after.starts_with("||=")
                || after.starts_with("&&=")
            {
                search = close + 2;
                continue;
            }
            context.replace(
                format!("Use `ENV.fetch({quote}{key}{quote}, nil)` instead of `ENV[{quote}{key}{quote}]`."),
                start..close + 2,
                start..close + 2,
                format!("ENV.fetch({quote}{key}{quote}, nil)"),
            );
            search = close + 2;
        }
    }
}

fn special_global_vars(context: &mut CopContext<'_, '_>) {
    for (old, new) in [
        ("$:", "$LOAD_PATH"),
        ("$\"", "$LOADED_FEATURES"),
        ("$0", "$PROGRAM_NAME"),
        ("$!", "$ERROR_INFO"),
        ("$@", "$ERROR_POSITION"),
    ] {
        replace_code(
            context,
            old,
            new,
            "Prefer the English global variable name.",
        );
    }
}

fn string_concatenation(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        for quote in ['\'', '"'] {
            let needle = format!("{quote} + {quote}");
            if let Some(join) = line.find(&needle) {
                context.remove(
                    "Prefer string concatenation without `+`.",
                    offset + join..offset + join + needle.len(),
                    offset + join + 1..offset + join + needle.len() - 1,
                );
            }
        }
    }
}

fn redundant_line_continuation(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if line.trim_end().ends_with('\\')
            && ["(", "[", "{", ",", ".", "&&", "||"].iter().any(|token| {
                line[..line.trim_end().len() - 1]
                    .trim_end()
                    .ends_with(token)
            })
        {
            let slash = offset + line.rfind('\\').unwrap_or(0);
            context.remove(
                "Redundant line continuation.",
                slash..slash + 1,
                slash..slash + 1,
            );
        }
    }
}

fn lambda_literal(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("literal") != "literal" {
        return;
    }
    replace_code(
        context,
        "lambda {",
        "-> {",
        "Use the lambda literal syntax for all lambdas.",
    );
    replace_code(
        context,
        "lambda do",
        "-> do",
        "Use the lambda literal syntax for all lambdas.",
    );
}

fn format_string(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(percent) = line.find(" % ") else {
            continue;
        };
        let left = line[..percent].trim();
        if !left.starts_with(['\'', '"']) {
            continue;
        }
        let right = line[percent + 3..].trim();
        let start = offset + line.find(left).unwrap_or(0);
        context.replace(
            "Use `format` instead of the `%` operator.",
            start..offset + line.len(),
            start..offset + line.len(),
            format!("format({left}, {right})"),
        );
    }
}

fn word_array(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("percent") != "percent" {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('[') || !trimmed.ends_with(']') {
            continue;
        }
        let values = trimmed[1..trimmed.len() - 1]
            .split(',')
            .map(str::trim)
            .collect::<Vec<_>>();
        if values.len() < 2
            || values.iter().any(|value| {
                !(value.starts_with('\'') && value.ends_with('\'')
                    || value.starts_with('"') && value.ends_with('"'))
            })
        {
            continue;
        }
        let words = values
            .iter()
            .map(|value| value.trim_matches(['\'', '"']))
            .collect::<Vec<_>>();
        if words.iter().any(|word| word.contains(char::is_whitespace)) {
            continue;
        }
        let start = offset + line.find(trimmed).unwrap_or(0);
        context.replace(
            "Use `%w` or `%W` for an array of words.",
            start..start + trimmed.len(),
            start..start + trimmed.len(),
            format!("%w({})", words.join(" ")),
        );
    }
}

fn percent_delimiters(context: &mut CopContext<'_, '_>) {
    if context.config_value("PreferredDelimiters").is_none() {
        return;
    }
    for (old, new) in [
        ("%w(", "%w["),
        ("%i(", "%i["),
        ("%W(", "%W["),
        ("%I(", "%I["),
    ] {
        for start in context.source_file().code_offsets(old) {
            if let Some(close) = context.source()[start + old.len()..]
                .find(')')
                .map(|at| start + old.len() + at)
            {
                context.replace(
                    "Use `[]` delimiters for this percent literal.",
                    start..close + 1,
                    start..close + 1,
                    format!("{}{}]", new, &context.source()[start + old.len()..close]),
                );
            }
        }
    }
}

fn redundant_string_escape(context: &mut CopContext<'_, '_>) {
    replace_code(context, "\\/", "/", "Remove the redundant escape.");
}

fn symbol_proc(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(block) = line.find(" { |") else {
            continue;
        };
        let rest = &line[block + 4..];
        let Some(pipe) = rest.find('|') else { continue };
        let parameter = rest[..pipe].trim();
        let body = rest[pipe + 1..].trim().trim_end_matches('}').trim();
        if body != format!("{parameter}.to_s") {
            continue;
        }
        context.replace(
            "Pass `&:to_s` as an argument instead of a block.",
            offset + block..offset + line.len(),
            offset + block..offset + line.len(),
            "(&:to_s)",
        );
    }
}
