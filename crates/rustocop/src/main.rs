use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

mod cli;
mod cop_registry;
mod cop_selection;
mod diagnostic;
mod file_runner;
mod inspection;
mod line_cops;
mod prism_engine;
mod source_lines;

use cop_registry::SUPPORTED_COPS;
use diagnostic::Offense;
use inspection::{InspectionPlan, InspectionResult};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Parallelism {
    Sequential,
    Automatic,
    Fixed(usize),
}

#[derive(Clone, Debug)]
struct Options {
    autocorrect: bool,
    files: Vec<String>,
    format: String,
    cops: cop_selection::CopSelection,
    stdin_path: Option<String>,
    target_ruby_version: prism_engine::RubyVersion,
    parallelism: Parallelism,
}

enum Command {
    Run(Options),
    Version,
    VerboseVersion,
    ShowCops,
}

fn main() {
    match cli::parse_args(env::args().skip(1).collect()) {
        Ok(Command::Version) => {
            println!("{}", rustocop_version());
        }
        Ok(Command::VerboseVersion) => {
            println!(
                "{} (rustocop native, RuboCop-compatible JSON formatter)",
                rustocop_version()
            );
        }
        Ok(Command::ShowCops) => {
            for cop in SUPPORTED_COPS {
                println!("{}", cop);
            }
        }
        Ok(Command::Run(options)) => match inspect_targets(&options) {
            Ok(results) => {
                let offense_count = results
                    .iter()
                    .map(|result| result.offenses.len())
                    .sum::<usize>();

                if options.format == "json" {
                    print!("{}", json_report(&results, offense_count));
                } else {
                    write_simple_report(&results, offense_count);
                }

                process::exit(exit_status(&options, &results));
            }
            Err(error) => {
                eprintln!("rustocop: {}", error);
                process::exit(2);
            }
        },
        Err(error) => {
            eprintln!("rustocop: {}", error);
            process::exit(2);
        }
    }
}

fn inspect_targets(options: &Options) -> io::Result<Vec<InspectionResult>> {
    let plan = InspectionPlan::new(options);
    if let Some(path) = &options.stdin_path {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        let (offenses, _) = plan.inspect_content(&expanded_path(path), &content, options);
        return Ok(vec![InspectionResult {
            path: expanded_path(path),
            offenses,
        }]);
    }

    let files = if options.files.is_empty() {
        discover_ruby_files()?
    } else {
        expand_targets(&options.files)?
    };

    file_runner::inspect_files(&files, options, &plan)
}

fn expand_targets(targets: &[String]) -> io::Result<Vec<String>> {
    let mut files = Vec::new();

    for target in targets {
        let path = Path::new(target);

        if path.is_dir() {
            discover_ruby_files_under(path, &mut files)?;
        } else {
            files.push(target.to_string());
        }
    }

    Ok(files)
}

fn discover_ruby_files() -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    discover_ruby_files_under(Path::new("."), &mut files)?;
    Ok(files)
}

fn discover_ruby_files_under(path: &Path, files: &mut Vec<String>) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if should_skip_entry(&entry_path, &file_name) {
            continue;
        }

        if entry_path.is_dir() {
            discover_ruby_files_under(&entry_path, files)?;
        } else if is_ruby_target(&entry_path) {
            files.push(entry_path.to_string_lossy().to_string());
        }
    }

    Ok(())
}

fn should_skip_entry(path: &Path, file_name: &str) -> bool {
    if file_name.starts_with('.') || matches!(file_name, "node_modules" | "target" | "tmp") {
        return true;
    }

    let text = path.to_string_lossy();
    text.contains("vendor/gems") || text.contains("vendor/bundle")
}

fn is_ruby_target(path: &Path) -> bool {
    if path
        .extension()
        .and_then(OsStr::to_str)
        .is_some_and(|extension| matches!(extension, "rb" | "rake" | "gemspec"))
    {
        return true;
    }

    path.file_name()
        .and_then(OsStr::to_str)
        .is_some_and(|name| {
            matches!(
                name,
                "Gemfile" | "Rakefile" | "Guardfile" | "Dangerfile" | "config.ru"
            )
        })
}

fn cop_enabled(options: &Options, cop: &str) -> bool {
    options.cops.enabled(cop)
}

fn exit_status(options: &Options, results: &[InspectionResult]) -> i32 {
    let mut offense_count = 0;
    let mut uncorrected_count = 0;

    for offense in results.iter().flat_map(|result| &result.offenses) {
        offense_count += 1;
        if !offense.corrected {
            uncorrected_count += 1;
        }
    }

    if offense_count == 0 || (options.autocorrect && uncorrected_count == 0) {
        0
    } else {
        1
    }
}

fn json_report(results: &[InspectionResult], offense_count: usize) -> String {
    let files = results
        .iter()
        .map(json_file)
        .collect::<Vec<String>>()
        .join(",");

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
        .collect::<Vec<String>>()
        .join(",");

    format!(
        "{{\"path\":\"{}\",\"offenses\":[{}]}}",
        json_escape(&result.path),
        offenses
    )
}

fn json_offense(offense: &Offense) -> String {
    let severity = if offense.cop_name.starts_with("Lint/") {
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

fn write_simple_report(results: &[InspectionResult], offense_count: usize) {
    let corrected_count = results
        .iter()
        .flat_map(|result| &result.offenses)
        .filter(|offense| offense.corrected)
        .count();
    let correctable_count = results
        .iter()
        .flat_map(|result| &result.offenses)
        .filter(|offense| offense.correctable && !offense.corrected)
        .count();

    for result in results {
        if result.offenses.is_empty() {
            continue;
        }

        println!("== {} ==", result.path);

        for offense in &result.offenses {
            let correction_label = if offense.corrected {
                Some("Corrected")
            } else if offense.correctable {
                Some("Correctable")
            } else {
                None
            };
            let label = correction_label
                .map(|label| format!(" [{}]", label))
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
            ", {} {} corrected",
            corrected_count,
            pluralize("offense", corrected_count)
        );
    }

    if correctable_count > 0 {
        print!(
            ", {} {} autocorrectable",
            correctable_count,
            pluralize("offense", correctable_count)
        );
    }

    println!();
}

fn pluralize(word: &str, count: usize) -> String {
    if count == 1 {
        word.to_string()
    } else {
        format!("{}s", word)
    }
}

fn rustocop_version() -> String {
    env_value("RUSTOCOP_VERSION", env!("CARGO_PKG_VERSION"))
}

fn env_value(key: &str, default: &str) -> String {
    env::var(key).unwrap_or_else(|_| default.to_string())
}

fn expanded_path(path: &str) -> String {
    let path = PathBuf::from(path);
    let absolute = if path.is_absolute() {
        path
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };

    absolute.to_string_lossy().to_string()
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
