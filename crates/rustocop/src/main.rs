use std::env;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

const TRAILING_WHITESPACE_COP: &str = "Layout/TrailingWhitespace";
const TRAILING_WHITESPACE_MESSAGE: &str = "Trailing whitespace detected.";

#[derive(Debug)]
struct Options {
    autocorrect: bool,
    files: Vec<String>,
    format: String,
    only: Option<String>,
    stdin_path: Option<String>,
}

#[derive(Debug)]
struct InspectionResult {
    path: String,
    offenses: Vec<Offense>,
}

#[derive(Debug)]
struct Offense {
    corrected: bool,
    line: usize,
    column: usize,
    length: usize,
}

enum Command {
    Run(Options),
    Version,
    VerboseVersion,
}

fn main() {
    match parse_args(env::args().skip(1).collect()) {
        Ok(Command::Version) => {
            println!("{}", rustocop_version());
        }
        Ok(Command::VerboseVersion) => {
            println!(
                "{} (rustocop native, RuboCop-compatible JSON formatter)",
                rustocop_version()
            );
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

                process::exit(corrected_exit_status(&options, offense_count));
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

fn parse_args(mut args: Vec<String>) -> Result<Command, String> {
    let mut options = Options {
        autocorrect: false,
        files: Vec::new(),
        format: "simple".to_string(),
        only: None,
        stdin_path: None,
    };

    while !args.is_empty() {
        let arg = args.remove(0);

        match arg.as_str() {
            "--version" => return Ok(Command::Version),
            "-V" => return Ok(Command::VerboseVersion),
            "-A" | "-a" | "--autocorrect" | "--autocorrect-all" | "--auto-correct"
            | "--auto-correct-all" => options.autocorrect = true,
            "--format" | "-f" => options.format = take_value(&mut args, &arg)?,
            "--only" => options.only = Some(take_value(&mut args, &arg)?),
            "--stdin" => options.stdin_path = Some(take_value(&mut args, &arg)?),
            "--force-exclusion" | "--no-server" => {}
            "--cache" => {
                if args.first().is_some_and(|value| !value.starts_with('-')) {
                    args.remove(0);
                }
            }
            "--" => {
                options.files.extend(args);
                break;
            }
            _ if arg.starts_with("--format=") => {
                options.format = arg
                    .strip_prefix("--format=")
                    .unwrap_or_default()
                    .to_string();
            }
            _ if arg.starts_with("--only=") => {
                options.only = Some(arg.strip_prefix("--only=").unwrap_or_default().to_string());
            }
            _ if arg.starts_with("--stdin=") => {
                options.stdin_path =
                    Some(arg.strip_prefix("--stdin=").unwrap_or_default().to_string());
            }
            _ if arg.starts_with('-') => return Err(format!("unsupported option {}", arg)),
            _ => options.files.push(arg),
        }
    }

    if options.format != "json" && options.format != "simple" {
        return Err(format!("unsupported formatter {}", options.format));
    }

    Ok(Command::Run(options))
}

fn take_value(args: &mut Vec<String>, option: &str) -> Result<String, String> {
    if args.is_empty() {
        return Err(format!("missing value for {}", option));
    }

    Ok(args.remove(0))
}

fn inspect_targets(options: &Options) -> io::Result<Vec<InspectionResult>> {
    if let Some(path) = &options.stdin_path {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        return Ok(vec![inspect_content(
            &expanded_path(path),
            &content,
            options,
        )]);
    }

    let files = if options.files.is_empty() {
        discover_ruby_files()?
    } else {
        expand_targets(&options.files)?
    };

    files
        .iter()
        .map(|path| inspect_file(path, options))
        .collect()
}

fn inspect_file(path: &str, options: &Options) -> io::Result<InspectionResult> {
    let content = fs::read_to_string(path)?;
    let result = inspect_content(&expanded_path(path), &content, options);

    if options.autocorrect && !result.offenses.is_empty() {
        fs::write(path, remove_trailing_whitespace(&content))?;
    }

    Ok(result)
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

fn remove_trailing_whitespace(content: &str) -> String {
    let mut corrected = String::with_capacity(content.len());

    for raw_line in content.split_inclusive('\n') {
        let (body, ending) = if let Some(body) = raw_line.strip_suffix("\r\n") {
            (body, "\r\n")
        } else if let Some(body) = raw_line.strip_suffix('\n') {
            (body, "\n")
        } else {
            (raw_line, "")
        };

        corrected
            .push_str(body.trim_end_matches(|character| character == ' ' || character == '\t'));
        corrected.push_str(ending);
    }

    corrected
}

fn corrected_exit_status(options: &Options, offense_count: usize) -> i32 {
    if offense_count == 0 || options.autocorrect {
        0
    } else {
        1
    }
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

        if file_name.starts_with('.') || file_name == "target" {
            continue;
        }

        if entry_path.is_dir() {
            discover_ruby_files_under(&entry_path, files)?;
        } else if entry_path
            .extension()
            .is_some_and(|extension| extension == "rb")
        {
            files.push(entry_path.to_string_lossy().to_string());
        }
    }

    Ok(())
}

fn inspect_content(path: &str, content: &str, options: &Options) -> InspectionResult {
    let offenses = if supports_trailing_whitespace(options.only.as_deref()) {
        trailing_whitespace_offenses(content, options.autocorrect)
    } else {
        Vec::new()
    };

    InspectionResult {
        path: path.to_string(),
        offenses,
    }
}

fn supports_trailing_whitespace(only: Option<&str>) -> bool {
    match only {
        None => true,
        Some(value) => value
            .split(',')
            .map(str::trim)
            .any(|cop| cop == TRAILING_WHITESPACE_COP),
    }
}

fn trailing_whitespace_offenses(content: &str, corrected: bool) -> Vec<Offense> {
    content
        .split_inclusive('\n')
        .enumerate()
        .filter_map(|(index, raw_line)| {
            let line = raw_line.trim_end_matches('\n').trim_end_matches('\r');
            let length = line
                .chars()
                .rev()
                .take_while(|character| *character == ' ' || *character == '\t')
                .count();

            if length == 0 {
                return None;
            }

            let column = line.chars().count() - length + 1;
            Some(Offense {
                corrected,
                line: index + 1,
                column,
                length,
            })
        })
        .collect()
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
    let last_column = offense.column + offense.length - 1;

    format!(
        "{{\"severity\":\"convention\",\"message\":\"{}\",\"cop_name\":\"{}\",\"corrected\":{},\"correctable\":true,\"location\":{{\"start_line\":{},\"start_column\":{},\"last_line\":{},\"last_column\":{},\"length\":{},\"line\":{},\"column\":{}}}}}",
        json_escape(TRAILING_WHITESPACE_MESSAGE),
        json_escape(TRAILING_WHITESPACE_COP),
        offense.corrected,
        offense.line,
        offense.column,
        offense.line,
        last_column,
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

    for result in results {
        if result.offenses.is_empty() {
            continue;
        }

        println!("== {} ==", result.path);

        for offense in &result.offenses {
            let correction_label = if offense.corrected {
                "Corrected"
            } else {
                "Correctable"
            };

            println!(
                "C:{:>3}:{:>3}: [{}] {}: {}",
                offense.line,
                offense.column,
                correction_label,
                TRAILING_WHITESPACE_COP,
                TRAILING_WHITESPACE_MESSAGE
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
    } else if offense_count > 0 {
        print!(
            ", {} {} autocorrectable",
            offense_count,
            pluralize("offense", offense_count)
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
