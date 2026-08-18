use super::source_helpers::*;
use super::*;

define_cops! {
    EmptyFile => "Lint/EmptyFile" => source(empty_file),
    NumericLiteralPrefix => "Style/NumericLiteralPrefix" => source(numeric_literal_prefix),
    NestedPercentLiteral => "Lint/NestedPercentLiteral" => source(nested_percent_literal),
    RescueException => "Lint/RescueException" => source(rescue_exception),
    OpenStructUse => "Style/OpenStructUse" => source(open_struct_use),
}

fn empty_file(reporter: &mut CopContext<'_, '_>) {
    let source = reporter.source();
    let empty =
        source.is_empty() || (!source.ends_with('\n') && source.trim_start().starts_with('#'));
    if empty {
        reporter.report("Empty file detected.", 0..0);
    }
}

fn numeric_literal_prefix(reporter: &mut CopContext<'_, '_>) {
    let source = reporter.source();
    let zero_only = reporter.config_value("EnforcedOctalStyle") == Some("zero_only");
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        if bytes[index] != b'0' || index > 0 && identifier_byte(bytes[index - 1]) {
            index += 1;
            continue;
        }
        let end = index
            + bytes[index..]
                .iter()
                .take_while(|byte| {
                    byte.is_ascii_hexdigit()
                        || matches!(byte, b'x' | b'X' | b'b' | b'B' | b'o' | b'O' | b'd' | b'D')
                })
                .count();
        let literal = &source[index..end];
        let (message, replacement) =
            if zero_only && (literal.starts_with("0o") || literal.starts_with("0O")) {
                ("Use 0 for octal literals.", format!("0{}", &literal[2..]))
            } else if let Some(digits) = literal.strip_prefix("0X") {
                ("Use 0x for hexadecimal literals.", format!("0x{digits}"))
            } else if let Some(digits) = literal.strip_prefix("0B") {
                ("Use 0b for binary literals.", format!("0b{digits}"))
            } else if let Some(digits) = literal.strip_prefix("0O") {
                ("Use 0o for octal literals.", format!("0o{digits}"))
            } else if literal.starts_with("0d") || literal.starts_with("0D") {
                (
                    "Do not use prefixes for decimal literals.",
                    literal[2..].to_string(),
                )
            } else if !zero_only
                && literal.len() > 1
                && literal[1..].bytes().all(|byte| matches!(byte, b'0'..=b'7'))
            {
                ("Use 0o for octal literals.", format!("0o{}", &literal[1..]))
            } else {
                index = end.max(index + 1);
                continue;
            };
        reporter.replace(message, index..end, index..end, replacement);
        index = end;
    }
}

fn nested_percent_literal(reporter: &mut CopContext<'_, '_>) {
    let source = reporter.source();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim();
        let nested = trimmed.starts_with("%i[") && trimmed[3..].contains("%i[")
            || trimmed.starts_with("%W[") && trimmed[3..].contains("%W[");
        if nested {
            let start = offset + line.len() - line.trim_start().len();
            reporter.report("Within percent literals, nested percent literals do not function and may be unwanted in the result.", start..offset + line.len());
        }
    }
}

fn rescue_exception(reporter: &mut CopContext<'_, '_>) {
    let source = reporter.source();
    for (offset, line) in source_lines(source) {
        let trimmed = line.trim_start();
        let exceptions = trimmed
            .strip_prefix("rescue ")
            .map(|list| list.split("=>").next().unwrap_or(list));
        if exceptions.is_some_and(|list| {
            list.split(',')
                .any(|name| matches!(name.trim(), "Exception" | "::Exception"))
        }) {
            let start = offset + line.len() - trimmed.len();
            reporter.report("Avoid rescuing the `Exception` class. Perhaps you meant to rescue `StandardError`?", start..offset + line.len());
        }
    }
}

fn open_struct_use(reporter: &mut CopContext<'_, '_>) {
    let source = reporter.source();
    for name in ["::OpenStruct", "OpenStruct"] {
        for start in all_offsets(source, name) {
            if name == "OpenStruct" && start >= 2 && &source[start - 2..start] == "::" {
                continue;
            }
            let prefixed = start > 0
                && matches!(source.as_bytes()[start - 1], b':' | b'_' | b'A'..=b'Z' | b'a'..=b'z');
            let line_start = source[..start].rfind('\n').map_or(0, |offset| offset + 1);
            let declaration = matches!(source[line_start..start].trim(), "class" | "module");
            if !prefixed && !declaration {
                reporter.report("Avoid using `OpenStruct`; use `Struct`, `Hash`, a class or test doubles instead.", start..start + name.len());
            }
        }
    }
}
