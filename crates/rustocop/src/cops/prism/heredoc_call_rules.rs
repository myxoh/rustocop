use super::*;

declare_source_cops! {
    HeredocMethodCallPosition => "Lint/HeredocMethodCallPosition" => heredoc_method_call_position,
}

fn heredoc_method_call_position(source: &str, reporter: &mut Reporter<'_>) {
    if !reporter.config_bool("Enabled", true) {
        return;
    }
    let lines = SourceFile::new(source)
        .lines()
        .map(|(start, line)| {
            let end = source[start..]
                .find('\n')
                .map_or(source.len(), |relative| start + relative + 1);
            (start, end, line)
        })
        .collect::<Vec<_>>();
    let message =
        "Put a method call with a HEREDOC receiver on the same line as the HEREDOC opening.";

    for (index, (opening_start, _, opening_line)) in lines.iter().enumerate() {
        let Some((terminator, marker_end)) = heredoc_marker(opening_line) else {
            continue;
        };
        if opening_line[marker_end..].contains('.') || opening_line[marker_end..].contains("&.") {
            continue;
        }
        let Some(terminator_index) = lines[index + 1..]
            .iter()
            .position(|(_, _, line)| line.trim() == terminator)
            .map(|relative| index + 1 + relative)
        else {
            continue;
        };
        let Some((call_start, call_end, call_line)) = lines.get(terminator_index + 1) else {
            continue;
        };
        let indentation = call_line.len() - call_line.trim_start().len();
        let call = call_line.trim_start();
        if !call.starts_with('.') && !call.starts_with("&.") {
            continue;
        }
        let offense = call_start + indentation..call_start + indentation + 1;
        let continuation = lines.get(terminator_index + 2).is_some_and(|(_, _, line)| {
            let line = line.trim_start();
            line.starts_with('.') || line.starts_with("&.")
        });
        let balanced = call.bytes().filter(|byte| *byte == b'(').count()
            == call.bytes().filter(|byte| *byte == b')').count();
        if continuation || !balanced {
            reporter.report(message, offense);
        } else {
            reporter.replace_many(
                message,
                offense,
                vec![
                    (
                        opening_start + marker_end..opening_start + marker_end,
                        call.trim_end().to_string(),
                    ),
                    (*call_start..*call_end, String::new()),
                ],
            );
        }
    }
}

fn heredoc_marker(line: &str) -> Option<(&str, usize)> {
    let marker = line.find("<<")?;
    let mut start = marker + 2;
    if line
        .as_bytes()
        .get(start)
        .is_some_and(|byte| matches!(byte, b'-' | b'~'))
    {
        start += 1;
    }
    if line
        .as_bytes()
        .get(start)
        .is_some_and(|byte| matches!(byte, b'\'' | b'"' | b'`'))
    {
        start += 1;
    }
    let length = line[start..]
        .bytes()
        .take_while(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        .count();
    (length > 0).then_some((&line[start..start + length], start + length))
}
