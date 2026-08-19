use std::env;

use crate::config::RunOptions;
use crate::engine::InspectionResult;
use crate::model::Offense;

pub(super) fn write(options: &RunOptions, results: &[InspectionResult]) {
    let offense_count = results
        .iter()
        .map(|result| result.offenses.len())
        .sum::<usize>();
    if options.format == "json" {
        print!("{}", json_report(results, offense_count));
    } else {
        write_simple_report(results, offense_count);
    }
}

pub(super) fn exit_status(options: &RunOptions, results: &[InspectionResult]) -> i32 {
    let offenses = results.iter().flat_map(|result| &result.offenses);
    let (count, uncorrected) = offenses.fold((0, 0), |(count, uncorrected), offense| {
        (count + 1, uncorrected + usize::from(!offense.corrected))
    });
    if count == 0 || (options.inspection.autocorrect && uncorrected == 0) {
        0
    } else {
        1
    }
}

pub(super) fn rustocop_version() -> String {
    env_value("RUSTOCOP_VERSION", env!("CARGO_PKG_VERSION"))
}

fn json_report(results: &[InspectionResult], offense_count: usize) -> String {
    let files = results.iter().map(json_file).collect::<Vec<_>>().join(",");
    format!(
        "{{\"metadata\":{{\"rubocop_version\":\"{}\",\"ruby_engine\":\"{}\",\"ruby_version\":\"{}\",\"ruby_patchlevel\":\"{}\",\"ruby_platform\":\"{}\"}},\"files\":[{}],\"summary\":{{\"offense_count\":{},\"target_file_count\":{},\"inspected_file_count\":{}}}}}",
        json_escape(&rustocop_version()),
        json_escape(&env_value("RUSTOCOP_RUBY_ENGINE", "ruby")),
        json_escape(&env_value("RUSTOCOP_RUBY_VERSION", "")),
        json_escape(&env_value("RUSTOCOP_RUBY_PATCHLEVEL", "")),
        json_escape(&env_value("RUSTOCOP_RUBY_PLATFORM", "")),
        files,
        offense_count,
        results.len(),
        results.len()
    )
}

fn json_file(result: &InspectionResult) -> String {
    let offenses = result
        .offenses
        .iter()
        .map(json_offense)
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"path\":\"{}\",\"offenses\":[{}]}}",
        json_escape(&result.path),
        offenses
    )
}

fn json_offense(offense: &Offense) -> String {
    let severity = if warning_cop(offense.cop_name) {
        "warning"
    } else {
        "convention"
    };
    format!(
        "{{\"severity\":\"{}\",\"message\":\"{}\",\"cop_name\":\"{}\",\"corrected\":{},\"correctable\":{},\"location\":{{\"start_line\":{},\"start_column\":{},\"last_line\":{},\"last_column\":{},\"length\":{},\"line\":{},\"column\":{}}}}}",
        severity,
        json_escape(&offense.message),
        json_escape(offense.cop_name),
        offense.corrected,
        offense.correctable,
        offense.line,
        offense.column,
        offense.last_line,
        offense.last_column,
        offense.length,
        offense.line,
        offense.column
    )
}

fn warning_cop(cop_name: &str) -> bool {
    cop_name.starts_with("Lint/")
        || matches!(
            cop_name,
            "Bundler/InsecureProtocolSource"
                | "Gemspec/RubyVersionGlobalsUsage"
                | "Gemspec/DuplicatedAssignment"
                | "Layout/BeginEndAlignment"
        )
}

fn write_simple_report(results: &[InspectionResult], offense_count: usize) {
    let offenses = results.iter().flat_map(|result| &result.offenses);
    let corrected_count = offenses.clone().filter(|offense| offense.corrected).count();
    let correctable_count = offenses
        .filter(|offense| offense.correctable && !offense.corrected)
        .count();

    for result in results.iter().filter(|result| !result.offenses.is_empty()) {
        println!("== {} ==", result.path);
        for offense in &result.offenses {
            let label = correction_label(offense)
                .map(|label| format!(" [{label}]"))
                .unwrap_or_default();
            println!(
                "C:{:>3}:{:>3}:{} {}: {}",
                offense.line, offense.column, label, offense.cop_name, offense.message
            );
        }
    }

    println!();
    print!(
        "{} {} inspected, {} {} detected",
        results.len(),
        pluralize("file", results.len()),
        offense_count,
        pluralize("offense", offense_count)
    );
    if corrected_count > 0 {
        print!(
            ", {corrected_count} {} corrected",
            pluralize("offense", corrected_count)
        );
    }
    if correctable_count > 0 {
        print!(
            ", {correctable_count} {} autocorrectable",
            pluralize("offense", correctable_count)
        );
    }
    println!();
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
