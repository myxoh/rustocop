use super::*;

define_cops! {
    SpecialGlobalVars => "Style/SpecialGlobalVars" => node(as_global_variable_read_node, special_global_vars),
}

const VARIABLES: &[(&str, &[&str])] = &[
    ("$:", &["$LOAD_PATH"]),
    ("$\"", &["$LOADED_FEATURES"]),
    ("$0", &["$PROGRAM_NAME"]),
    ("$!", &["$ERROR_INFO"]),
    ("$@", &["$ERROR_POSITION"]),
    ("$;", &["$FIELD_SEPARATOR", "$FS"]),
    ("$,", &["$OUTPUT_FIELD_SEPARATOR", "$OFS"]),
    ("$/", &["$INPUT_RECORD_SEPARATOR", "$RS"]),
    ("$\\", &["$OUTPUT_RECORD_SEPARATOR", "$ORS"]),
    ("$.", &["$INPUT_LINE_NUMBER", "$NR"]),
    ("$_", &["$LAST_READ_LINE"]),
    ("$>", &["$DEFAULT_OUTPUT"]),
    ("$<", &["$DEFAULT_INPUT"]),
    ("$$", &["$PROCESS_ID", "$PID"]),
    ("$?", &["$CHILD_STATUS"]),
    ("$~", &["$LAST_MATCH_INFO"]),
    ("$=", &["$IGNORECASE"]),
    ("$*", &["$ARGV", "ARGV"]),
];

fn special_global_vars(
    node: &ruby_prism::GlobalVariableReadNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let current = String::from_utf8_lossy(node.name().as_slice());
    let style = context.policy().enforced_style("use_english_names");
    let Some((perl, english)) = variable_entry(&current) else {
        return;
    };
    let preferred = match style {
        "use_english_names" => english.first().copied(),
        "use_perl_names" => (current.as_ref() != perl).then_some(perl),
        "use_builtin_english_names" => builtin_preferred(&current, perl, english),
        _ => None,
    };
    let Some(preferred) = preferred else {
        return;
    };
    if preferred == current {
        return;
    }

    let message = if style == "use_english_names" {
        english_message(perl, english)
    } else {
        format!("Prefer `{preferred}` over `{current}`.")
    };
    let (edit, replacement) = interpolation_replacement(node.location(), preferred, style, context);
    let needs_english = style == "use_english_names"
        && context.config_bool("RequireEnglish", false)
        && !matches!(
            preferred,
            "$LOAD_PATH" | "$LOADED_FEATURES" | "$PROGRAM_NAME" | "ARGV"
        );
    if needs_english {
        let mut edits = english_require_edits(context.source(), node.location().start_offset());
        edits.push((edit, replacement));
        context.replace_many(message, node.location(), edits);
    } else {
        context.replace(message, node.location(), edit, replacement);
    }
}

fn variable_entry(current: &str) -> Option<(&'static str, &'static [&'static str])> {
    VARIABLES.iter().find_map(|(perl, english)| {
        (*perl == current || english.contains(&current)).then_some((*perl, *english))
    })
}

fn builtin_preferred<'a>(current: &str, perl: &'a str, english: &'a [&str]) -> Option<&'a str> {
    let builtin = match perl {
        "$:" => Some("$LOAD_PATH"),
        "$\"" => Some("$LOADED_FEATURES"),
        "$0" => Some("$PROGRAM_NAME"),
        _ => None,
    };
    if let Some(builtin) = builtin {
        return (current != builtin).then_some(builtin);
    }
    (current != perl && english.contains(&current)).then_some(perl)
}

fn english_message(perl: &str, preferred: &[&str]) -> String {
    let (regular, english): (Vec<_>, Vec<_>) = preferred.iter().copied().partition(|name| {
        matches!(
            *name,
            "$LOAD_PATH" | "$LOADED_FEATURES" | "$PROGRAM_NAME" | "ARGV"
        )
    });
    let english = english.join("` or `");
    let regular = regular.join("` or `");
    if english.is_empty() {
        format!("Prefer `{regular}` over `{perl}`.")
    } else if regular.is_empty() {
        format!("Prefer `{english}` from the stdlib 'English' module (don't forget to require it) over `{perl}`.")
    } else {
        format!("Prefer `{english}` from the stdlib 'English' module (don't forget to require it) or `{regular}` over `{perl}`.")
    }
}

fn interpolation_replacement(
    location: ruby_prism::Location<'_>,
    preferred: &str,
    style: &str,
    context: &CopContext<'_, '_>,
) -> (std::ops::Range<usize>, String) {
    let range = location.start_offset()..location.end_offset();
    let Some(parent) = context.parent() else {
        return (range, preferred.to_string());
    };
    if parent.as_embedded_variable_node().is_some() {
        if style == "use_english_names" {
            return (range, format!("{{{preferred}}}"));
        }
        return (
            parent.location().start_offset()..parent.location().end_offset(),
            format!("#{preferred}"),
        );
    }
    if parent.as_embedded_statements_node().is_some() && style != "use_english_names" {
        return (
            parent.location().start_offset()..parent.location().end_offset(),
            format!("#{preferred}"),
        );
    }
    (range, preferred.to_string())
}

fn english_require_edits(
    source: &str,
    offense_start: usize,
) -> Vec<(std::ops::Range<usize>, String)> {
    let existing = source
        .lines()
        .scan(0usize, |offset, line| {
            let start = *offset;
            *offset += line.len() + 1;
            Some((start, line))
        })
        .find(|(_, line)| matches!(line.trim(), "require 'English'" | "require \"English\""));
    if existing.is_some_and(|(start, _)| start < offense_start) {
        return Vec::new();
    }
    let insertion = english_require_insertion_offset(source);
    let mut edits = vec![(insertion..insertion, "require 'English'\n".to_string())];
    if let Some((start, line)) = existing {
        let end = (start
            + line.len()
            + usize::from(source.as_bytes().get(start + line.len()) == Some(&b'\n')))
        .min(source.len());
        edits.push((start..end, String::new()));
    }
    edits
}

fn english_require_insertion_offset(source: &str) -> usize {
    let mut offset = 0;
    let mut lines = source.split_inclusive('\n').peekable();
    if lines.peek().is_some_and(|line| line.starts_with("#!")) {
        offset += lines.next().unwrap_or_default().len();
    }
    while let Some(line) = lines.peek() {
        let lower = line.trim().to_ascii_lowercase();
        if !(lower.starts_with("# frozen_string_literal:")
            || lower.starts_with("# coding:")
            || lower.starts_with("# encoding:")
            || lower.starts_with("# warn_indent:")
            || lower.starts_with("# shareable_constant_value:"))
        {
            break;
        }
        offset += lines.next().unwrap_or_default().len();
    }
    while lines.peek().is_some_and(|line| line.trim().is_empty()) {
        offset += lines.next().unwrap_or_default().len();
    }
    offset
}
