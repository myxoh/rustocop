use std::collections::HashMap;
use std::env;
use std::io::{BufWriter, Write};
use std::path::Path;

use crate::config::RunOptions;
use crate::engine::InspectionResult;
use crate::model::Offense;
use crate::rubocop::cop::message_annotator::{
    CopMessageConfig, MessageAnnotator, MessageConfig, MessageOptions, Urls,
};

pub(super) fn write(options: &RunOptions, results: &[InspectionResult]) -> Result<(), String> {
    let mut formatters = options
        .formats
        .iter()
        .map(|format| FormatterState::new(FormatterKind::from_name(format)))
        .collect::<Vec<_>>();
    let stdout = std::io::stdout();
    let mut output = BufWriter::new(stdout.lock());

    for formatter in &formatters {
        formatter_started(formatter.kind, results.len(), &mut output);
    }
    let mut inspected_count = 0;
    let mut file_error = None;
    'files: for (result_index, result) in results.iter().enumerate() {
        inspected_count += 1;
        for formatter in &mut formatters {
            if let Err(error) =
                formatter_file_finished(formatter, result_index, result, options, &mut output)
            {
                file_error = Some(error);
                break 'files;
            }
        }
    }
    for formatter in &formatters {
        if let Err(error) =
            formatter_finished(formatter, results, inspected_count, options, &mut output)
        {
            output.flush().expect("flush partial formatter output");
            return Err(error);
        }
    }
    output.flush().expect("flush formatter output");
    file_error.map_or(Ok(()), Err)
}

#[derive(Clone, Copy)]
enum FormatterKind {
    Json,
    Simple,
    Clang,
    Progress,
}

struct FormatterState {
    kind: FormatterKind,
    counts: ReportCounts,
    result_indexes: Vec<usize>,
}

impl FormatterState {
    fn new(kind: FormatterKind) -> Self {
        Self {
            kind,
            counts: ReportCounts::default(),
            result_indexes: Vec::new(),
        }
    }
}

impl FormatterKind {
    fn from_name(name: &str) -> Self {
        match name {
            "json" => Self::Json,
            "simple" => Self::Simple,
            "clang" => Self::Clang,
            "progress" => Self::Progress,
            _ => unreachable!("formatter names are validated by the CLI"),
        }
    }
}

#[derive(Clone, Copy, Default)]
struct ReportCounts {
    offenses: usize,
    corrected: usize,
    correctable: usize,
}

impl ReportCounts {
    fn add(&mut self, result: &InspectionResult) {
        self.offenses += result.offenses.len();
        self.corrected += result
            .offenses
            .iter()
            .filter(|offense| offense.corrected)
            .count();
        self.correctable += result
            .offenses
            .iter()
            .filter(|offense| offense.correctable && !offense.corrected)
            .count();
    }
}

fn formatter_started(format: FormatterKind, file_count: usize, output: &mut impl Write) {
    if matches!(format, FormatterKind::Progress) {
        writeln!(
            output,
            "Inspecting {} {}",
            file_count,
            pluralize("file", file_count)
        )
        .expect("write progress header");
    }
}

fn formatter_file_finished(
    formatter: &mut FormatterState,
    result_index: usize,
    result: &InspectionResult,
    options: &RunOptions,
    output: &mut impl Write,
) -> Result<(), String> {
    match formatter.kind {
        FormatterKind::Json => {
            formatter.counts.add(result);
            formatter.result_indexes.push(result_index);
            Ok(())
        }
        FormatterKind::Simple => {
            formatter.counts.add(result);
            write_simple_file(result, options, output)
        }
        FormatterKind::Clang => {
            formatter.counts.add(result);
            write_clang_file(result, options, false, output)
        }
        FormatterKind::Progress => {
            if !result.offenses.is_empty() {
                formatter.counts.add(result);
                formatter.result_indexes.push(result_index);
            }
            let mark = result
                .offenses
                .iter()
                .max_by_key(|offense| severity_rank(&offense.severity))
                .map_or('.', |offense| severity_code(&offense.severity));
            write!(output, "{mark}").expect("write progress mark");
            Ok(())
        }
    }
}

fn formatter_finished(
    formatter: &FormatterState,
    results: &[InspectionResult],
    inspected_count: usize,
    options: &RunOptions,
    output: &mut impl Write,
) -> Result<(), String> {
    match formatter.kind {
        FormatterKind::Json => {
            write!(
                output,
                "{}",
                json_report(
                    results,
                    &formatter.result_indexes,
                    formatter.counts.offenses,
                    inspected_count,
                    options,
                )?
            )
            .expect("write JSON report");
            Ok(())
        }
        FormatterKind::Simple | FormatterKind::Clang => {
            write_summary(output, inspected_count, formatter.counts);
            Ok(())
        }
        FormatterKind::Progress => {
            writeln!(output).expect("finish progress marks");
            if formatter.counts.offenses > 0 {
                writeln!(output).expect("write progress spacing");
                writeln!(output, "Offenses:").expect("write progress offenses header");
                writeln!(output).expect("write progress spacing");
                for result_index in &formatter.result_indexes {
                    write_clang_file(&results[*result_index], options, true, output)?;
                }
            }
            write_summary(output, inspected_count, formatter.counts);
            Ok(())
        }
    }
}

pub(super) fn exit_status(options: &RunOptions, results: &[InspectionResult]) -> i32 {
    let offenses = results.iter().flat_map(|result| &result.offenses);
    let (count, uncorrected) = offenses.fold((0, 0), |(count, uncorrected), offense| {
        (count + 1, uncorrected + usize::from(!offense.corrected))
    });
    if count == 0 || (options.inspection.autocorrect_enabled() && uncorrected == 0) {
        0
    } else {
        1
    }
}

pub(super) fn rustocop_version() -> String {
    env_value("RUSTOCOP_VERSION", env!("CARGO_PKG_VERSION"))
}

fn json_report(
    results: &[InspectionResult],
    result_indexes: &[usize],
    offense_count: usize,
    inspected_count: usize,
    options: &RunOptions,
) -> Result<String, String> {
    let files = result_indexes
        .iter()
        .map(|result_index| json_file(&results[*result_index], options))
        .collect::<Result<Vec<_>, _>>()?
        .join(",");
    Ok(format!(
        "{{\"metadata\":{{\"rubocop_version\":\"{}\",\"ruby_engine\":\"{}\",\"ruby_version\":\"{}\",\"ruby_patchlevel\":\"{}\",\"ruby_platform\":\"{}\"}},\"files\":[{}],\"summary\":{{\"offense_count\":{},\"target_file_count\":{},\"inspected_file_count\":{}}}}}",
        json_escape(&rustocop_version()),
        json_escape(&env_value("RUSTOCOP_RUBY_ENGINE", "ruby")),
        json_escape(&env_value("RUSTOCOP_RUBY_VERSION", "")),
        json_escape(&env_value("RUSTOCOP_RUBY_PATCHLEVEL", "")),
        json_escape(&env_value("RUSTOCOP_RUBY_PLATFORM", "")),
        files,
        offense_count,
        results.len(),
        inspected_count
    ))
}

fn json_file(result: &InspectionResult, options: &RunOptions) -> Result<String, String> {
    let offenses = result
        .offenses
        .iter()
        .map(|offense| json_offense(offense, options))
        .collect::<Result<Vec<_>, _>>()?
        .join(",");
    Ok(format!(
        "{{\"path\":\"{}\",\"offenses\":[{}]}}",
        json_escape(&result.path),
        offenses
    ))
}

fn json_offense(offense: &Offense, options: &RunOptions) -> Result<String, String> {
    let message = annotated_exact_message(offense, options)
        .map_err(|_| "source sequence is illegal/malformed utf-8".to_string())?;
    Ok(format!(
        "{{\"severity\":\"{}\",\"message\":\"{}\",\"cop_name\":\"{}\",\"corrected\":{},\"correctable\":{},\"location\":{{\"start_line\":{},\"start_column\":{},\"last_line\":{},\"last_column\":{},\"length\":{},\"line\":{},\"column\":{}}}}}",
        json_escape(&offense.severity),
        json_escape(&message),
        json_escape(&offense.cop_name),
        offense.corrected,
        offense.correctable,
        offense.line,
        offense.column,
        offense.last_line,
        offense.last_column,
        offense.length,
        offense.line,
        offense.column
    ))
}

fn write_simple_file(
    result: &InspectionResult,
    options: &RunOptions,
    output: &mut impl Write,
) -> Result<(), String> {
    if result.offenses.is_empty() {
        return Ok(());
    }
    writeln!(output, "== {} ==", smart_path(&result.path)).expect("write simple file header");
    for offense in &result.offenses {
        writeln!(
            output,
            "{}:{:>3}:{:>3}: {}",
            severity_code(&offense.severity),
            offense.line,
            offense.column,
            rendered_message(offense, options, false)?
        )
        .expect("write simple offense");
    }
    Ok(())
}

fn write_clang_file(
    result: &InspectionResult,
    options: &RunOptions,
    replace_invalid_message_bytes: bool,
    output: &mut impl Write,
) -> Result<(), String> {
    let preserve_binary_message_bytes = replace_invalid_message_bytes
        && crate::engine::source::declares_binary_encoding(&result.source);
    for offense in &result.offenses {
        let message = rendered_message(offense, options, preserve_binary_message_bytes)?;
        write!(
            output,
            "{}:{}:{}: {}: ",
            smart_path(&result.path),
            offense.line,
            offense.column,
            severity_code(&offense.severity)
        )
        .expect("write clang offense prefix");
        if preserve_binary_message_bytes {
            output
                .write_all(&crate::engine::source::restore_binary_text(&message))
                .expect("write binary clang offense message");
        } else {
            output
                .write_all(message.as_bytes())
                .expect("write clang offense message");
        }
        output.write_all(b"\n").expect("finish clang offense");
        write_source_highlight(result, offense, output);
    }
    Ok(())
}

fn write_summary(output: &mut impl Write, file_count: usize, counts: ReportCounts) {
    writeln!(output).expect("write report summary spacing");
    if counts.offenses == 0 {
        write!(
            output,
            "{} {} inspected, no offenses detected",
            file_count,
            pluralize("file", file_count)
        )
        .expect("write report summary");
    } else {
        write!(
            output,
            "{} {} inspected, {} {} detected",
            file_count,
            pluralize("file", file_count),
            counts.offenses,
            pluralize("offense", counts.offenses)
        )
        .expect("write report summary");
    }
    if counts.corrected > 0 {
        write!(
            output,
            ", {} {} corrected",
            counts.corrected,
            pluralize("offense", counts.corrected)
        )
        .expect("write report summary");
    }
    if counts.correctable > 0 {
        write!(
            output,
            ", {} {} autocorrectable",
            counts.correctable,
            pluralize("offense", counts.correctable)
        )
        .expect("write report summary");
    }
    writeln!(output).expect("write report summary");
}

fn rendered_message(
    offense: &Offense,
    options: &RunOptions,
    replace_invalid_message_bytes: bool,
) -> Result<String, String> {
    let annotated = if replace_invalid_message_bytes {
        annotated_binary_message(offense, options)
    } else {
        annotated_exact_message(offense, options)?
    };
    let mut message = annotate_backticks(&annotated);
    if let Some(label) = correction_label(offense) {
        message = format!("[{label}] {message}");
    }
    Ok(message)
}

fn annotated_binary_message(offense: &Offense, options: &RunOptions) -> String {
    let message = offense.message_bytes.as_ref().map_or_else(
        || offense.message.clone(),
        |bytes| crate::engine::source::binary_inspection_text(bytes),
    );
    let config = message_config(options, &offense.cop_name);
    let cop_config = cop_message_config(options, &offense.cop_name);
    let message_options = MessageOptions {
        display_style_guide: options.display_style_guide,
        extra_details: options.extra_details,
        debug: options.debug,
        display_cop_names: options.display_cop_names,
        format: options.explicit_message_format.clone(),
    };
    MessageAnnotator::new(&config, &offense.cop_name, &cop_config, &message_options)
        .annotate(&message)
}

fn annotated_exact_message(offense: &Offense, options: &RunOptions) -> Result<String, String> {
    let message = exact_message(offense)?;
    let config = message_config(options, &offense.cop_name);
    let cop_config = cop_message_config(options, &offense.cop_name);
    let message_options = MessageOptions {
        display_style_guide: options.display_style_guide,
        extra_details: options.extra_details,
        debug: options.debug,
        display_cop_names: options.display_cop_names,
        format: options.explicit_message_format.clone(),
    };
    Ok(
        MessageAnnotator::new(&config, &offense.cop_name, &cop_config, &message_options)
            .annotate(message),
    )
}

fn message_config(options: &RunOptions, cop_name: &str) -> MessageConfig {
    let all_cops = [
        "DisplayCopNames",
        "DisplayStyleGuide",
        "ExtraDetails",
        "StyleGuideBaseURL",
    ]
    .into_iter()
    .filter_map(|key| {
        options
            .inspection
            .cop_config
            .value("AllCops", key)
            .map(|value| (key.to_string(), value.to_string()))
    })
    .collect();
    let departments = cop_name
        .rsplit_once('/')
        .and_then(|(department, _)| {
            options
                .inspection
                .cop_config
                .value(department, "StyleGuideBaseURL")
                .map(|value| HashMap::from([("StyleGuideBaseURL".to_string(), value.to_string())]))
                .map(|config| HashMap::from([(department.to_string(), config)]))
        })
        .unwrap_or_default();
    MessageConfig {
        all_cops,
        departments,
    }
}

fn cop_message_config(options: &RunOptions, cop_name: &str) -> CopMessageConfig {
    let config = &options.inspection.cop_config;
    CopMessageConfig {
        details: config.value(cop_name, "Details").map(str::to_string),
        style_guide: config.value(cop_name, "StyleGuide").map(str::to_string),
        references: message_urls(config, cop_name, "References"),
        reference: message_urls(config, cop_name, "Reference"),
    }
}

fn message_urls(config: &crate::config::CopConfig, cop_name: &str, key: &str) -> Option<Urls> {
    let values = config.values(cop_name, key);
    if !values.is_empty() {
        return Some(Urls::Many(values.to_vec()));
    }
    config
        .value(cop_name, key)
        .map(|value| Urls::One(value.to_string()))
}

fn annotate_backticks(message: &str) -> String {
    let mut rendered = String::with_capacity(message.len());
    let mut remainder = message;
    while let Some(open) = remainder.find('`') {
        rendered.push_str(&remainder[..open]);
        let after_open = &remainder[open + 1..];
        let Some(close) = after_open.find('`') else {
            rendered.push_str(&remainder[open..]);
            return rendered;
        };
        rendered.push_str(&after_open[..close]);
        remainder = &after_open[close + 1..];
    }
    rendered.push_str(remainder);
    rendered
}

fn smart_path(path: &str) -> String {
    let path = Path::new(path);
    std::env::current_dir()
        .ok()
        .and_then(|directory| path.strip_prefix(directory).ok())
        .filter(|relative| !relative.as_os_str().is_empty())
        .unwrap_or(path)
        .to_string_lossy()
        .to_string()
}

fn write_source_highlight(result: &InspectionResult, offense: &Offense, output: &mut impl Write) {
    let Some(source_line) = result
        .source
        .split('\n')
        .nth(offense.line.saturating_sub(1))
    else {
        return;
    };
    let source_line = source_line.trim_end_matches('\r');
    if source_line.is_empty() {
        return;
    }
    let source_bytes = crate::engine::source::restore_binary_text(source_line);
    output
        .write_all(&source_bytes)
        .expect("write offense source line");
    output.write_all(b"\n").expect("finish offense source line");

    let prefix = source_line
        .chars()
        .take(offense.column.saturating_sub(1))
        .map(|character| if character == '\t' { '\t' } else { ' ' })
        .collect::<String>();
    let highlight_length = if offense.length == 0 {
        0
    } else if offense.last_line == offense.line {
        offense
            .last_column
            .saturating_sub(offense.column)
            .saturating_add(1)
    } else {
        source_line
            .chars()
            .count()
            .saturating_sub(offense.column.saturating_sub(1))
    };
    writeln!(output, "{prefix}{}", "^".repeat(highlight_length))
        .expect("write offense highlighted area");
}

fn exact_message(offense: &Offense) -> Result<&str, String> {
    match &offense.message_bytes {
        Some(bytes) => {
            std::str::from_utf8(bytes).map_err(|_| "invalid byte sequence in UTF-8".to_string())
        }
        None => Ok(&offense.message),
    }
}

fn severity_code(severity: &str) -> char {
    match severity {
        "info" => 'I',
        "refactor" => 'R',
        "convention" => 'C',
        "warning" => 'W',
        "error" => 'E',
        "fatal" => 'F',
        _ => 'C',
    }
}

fn severity_rank(severity: &str) -> usize {
    match severity {
        "info" => 0,
        "refactor" => 1,
        "convention" => 2,
        "warning" => 3,
        "error" => 4,
        "fatal" => 5,
        _ => 2,
    }
}

fn correction_label(offense: &Offense) -> Option<&'static str> {
    if offense.corrected {
        Some("Corrected")
    } else if offense.correctable {
        Some("Correctable")
    } else {
        None
    }
}

fn pluralize(word: &str, count: usize) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{word}s")
    }
}

fn env_value(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn json_escape(value: &str) -> String {
    let mut escaped = String::new();
    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }
    escaped
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn escapes_json_control_characters() {
        assert_eq!(json_escape("\"\\\n"), "\\\"\\\\\\n");
    }
}
