use std::env;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::{Path, PathBuf};
use std::process;

mod line_cops;
mod prism_engine;

const TRAILING_WHITESPACE_COP: &str = "Layout/TrailingWhitespace";

const SUPPORTED_COPS: &[&str] = &[
    "Bundler/OrderedGems",
    "Rails/DefaultScope",
    "Rails/FilePath",
    "Rails/ApplicationJob",
    "Rails/ReversibleMigration",
    "Metrics/BlockLength",
    "Metrics/MethodLength",
    "Metrics/AbcSize",
    "Layout/LineLength",
    "Layout/ExtraSpacing",
    "Layout/EndAlignment",
    "Layout/FirstHashElementIndentation",
    "Layout/IndentationConsistency",
    "Layout/IndentationWidth",
    "Layout/TrailingWhitespace",
    "Layout/SpaceAfterColon",
    "Style/HashSyntax",
    "Style/ColonMethodDefinition",
    "Style/DoubleCopDisableDirective",
    "Style/EmptyLambdaParameter",
    "Style/EndBlock",
    "Style/InlineComment",
    "Style/KeywordParametersOrder",
    "Style/RedundantBegin",
    "Style/IfUnlessModifier",
    "Style/CaseLikeIf",
    "Style/ConditionalAssignment",
    "Style/EmptyCaseCondition",
    "Style/EmptyElse",
    "Style/GuardClause",
    "Style/HashLikeCase",
    "Style/ClassMethodsDefinitions",
    "Style/EndlessMethod",
    "Style/FrozenStringLiteralComment",
    "Style/Documentation",
    "Style/TrailingCommaInArrayLiteral",
    "Style/TrailingCommaInArguments",
    "Style/TrailingCommaInHashLiteral",
    "Style/ItAssignment",
    "Style/NumberedParameters",
    "Style/StringLiterals",
    "Style/CharacterLiteral",
    "Style/BeginBlock",
    "Style/DefWithParentheses",
    "Style/MethodCallWithoutArgsParentheses",
    "Style/NilComparison",
    "Style/Not",
    "Style/RedundantArrayConstructor",
    "Style/RedundantFreeze",
    "Style/Semicolon",
    "Style/StringChars",
    "Style/StringMethods",
    "Style/UnlessElse",
    "Style/FileTouch",
    "Style/GlobalStdStream",
    "Style/MinMax",
    "Style/RedundantFileExtensionInRequire",
    "Style/SuperWithArgsParentheses",
    "Style/TrailingCommaInBlockArgs",
    "Style/WhileUntilDo",
    "Style/ArrayJoin",
    "Style/NestedFileDirname",
    "Style/Proc",
    "Style/StderrPuts",
    "Style/Strip",
    "Naming/PredicatePrefix",
    "Naming/AccessorMethodName",
    "Lint/MissingSuper",
    "Lint/EmptyBlock",
    "Lint/UnusedMethodArgument",
    "Lint/Debugger",
    "Lint/BooleanSymbol",
    "Lint/BigDecimalNew",
    "Lint/EmptyEnsure",
    "Lint/EmptyExpression",
    "Lint/FlipFlop",
    "Lint/FloatComparison",
    "Lint/FloatOutOfRange",
    "Lint/IdentityComparison",
    "Lint/SelfAssignment",
    "Lint/ToJSON",
    "Lint/TrailingCommaInAttributeDeclaration",
    "Lint/UselessElseWithoutRescue",
    "Security/CompoundHash",
    "Security/Eval",
    "Security/JSONLoad",
    "Security/MarshalLoad",
    "Security/Open",
    "Security/IoMethods",
    "Security/YAMLLoad",
    "RSpec/NestedGroups",
    "RSpec/EmptyExampleGroup",
    "RSpec/MessageChain",
    "RSpec/MultipleExpectations",
    "RSpec/ExampleLength",
    "RSpec/VariableName",
    "RSpec/MultipleMemoizedHelpers",
    "RSpec/Focus",
    "RSpec/PendingWithoutReason",
    "RSpec/ScatteredSetup",
    "RSpec/SpecFilePathSuffix",
    "RSpec/SpecFilePathFormat",
];

const DEFAULT_DISABLED_COPS: &[&str] = &[
    "Style/Documentation",
    "Security/IoMethods",
    "RSpec/MessageChain",
    "RSpec/MultipleExpectations",
    "RSpec/MultipleMemoizedHelpers",
    "RSpec/PendingWithoutReason",
];

#[derive(Debug)]
struct Options {
    autocorrect: bool,
    files: Vec<String>,
    format: String,
    only: Option<String>,
    stdin_path: Option<String>,
    target_ruby_version: prism_engine::RubyVersion,
}

#[derive(Clone, Debug)]
struct SourceLine {
    body: String,
    ending: String,
}

#[derive(Debug)]
struct InspectionResult {
    path: String,
    offenses: Vec<Offense>,
}

#[derive(Debug)]
struct Offense {
    cop_name: &'static str,
    message: String,
    corrected: bool,
    correctable: bool,
    line: usize,
    column: usize,
    last_line: usize,
    last_column: usize,
    length: usize,
}

enum Command {
    Run(Options),
    Version,
    VerboseVersion,
    ShowCops,
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

fn parse_args(mut args: Vec<String>) -> Result<Command, String> {
    let mut options = Options {
        autocorrect: false,
        files: Vec::new(),
        format: "simple".to_string(),
        only: None,
        stdin_path: None,
        target_ruby_version: prism_engine::RubyVersion::default(),
    };

    while !args.is_empty() {
        let arg = args.remove(0);

        match arg.as_str() {
            "--version" => return Ok(Command::Version),
            "-V" => return Ok(Command::VerboseVersion),
            "--show-cops" => return Ok(Command::ShowCops),
            "-A" | "-a" | "--autocorrect" | "--autocorrect-all" | "--auto-correct"
            | "--auto-correct-all" => options.autocorrect = true,
            "--format" | "-f" => options.format = take_value(&mut args, &arg)?,
            "--only" => options.only = Some(take_value(&mut args, &arg)?),
            "--stdin" => options.stdin_path = Some(take_value(&mut args, &arg)?),
            "--config" | "-c" => {
                let path = take_value(&mut args, &arg)?;
                options.target_ruby_version = target_ruby_version_from_config(&path);
            }
            "--require" | "--plugin" => {
                let _ = take_value(&mut args, &arg)?;
            }
            "--force-exclusion"
            | "--no-server"
            | "--display-cop-names"
            | "--extra-details"
            | "--parallel" => {}
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
            _ if arg.starts_with("--config=") => {
                let path = arg.strip_prefix("--config=").unwrap_or_default();
                options.target_ruby_version = target_ruby_version_from_config(path);
            }
            _ if arg.starts_with("--require=") || arg.starts_with("--plugin=") => {}
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

fn target_ruby_version_from_config(path: &str) -> prism_engine::RubyVersion {
    fs::read_to_string(path)
        .ok()
        .and_then(|source| target_ruby_version_from_source(&source))
        .unwrap_or_default()
}

fn target_ruby_version_from_source(source: &str) -> Option<prism_engine::RubyVersion> {
    source.lines().find_map(|line| {
        let value = line.trim().strip_prefix("TargetRubyVersion:")?;
        prism_engine::RubyVersion::parse(value.split('#').next()?.trim())
    })
}

fn inspect_targets(options: &Options) -> io::Result<Vec<InspectionResult>> {
    if let Some(path) = &options.stdin_path {
        let mut content = String::new();
        io::stdin().read_to_string(&mut content)?;
        let (offenses, _) = inspect_content(&expanded_path(path), &content, options);
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

    files
        .iter()
        .map(|path| inspect_file(path, options))
        .collect()
}

fn inspect_file(path: &str, options: &Options) -> io::Result<InspectionResult> {
    let content = fs::read_to_string(path)?;
    let absolute_path = expanded_path(path);
    let (offenses, corrected_content) = inspect_content(&absolute_path, &content, options);

    if options.autocorrect && corrected_content != content {
        fs::write(path, corrected_content)?;
    }

    Ok(InspectionResult {
        path: absolute_path,
        offenses,
    })
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

fn inspect_content(path: &str, content: &str, options: &Options) -> (Vec<Offense>, String) {
    let mut lines = split_source(content);
    let original_lines = lines.clone();
    let mut offenses = Vec::new();

    line_cops::before_prism(path, &mut lines, options, &mut offenses);

    // Every file is parsed once. All AST-based cops share this Prism tree and
    // the corrections they produce are applied together after traversal.
    let prism_source = join_source(&lines);
    let prism_inspection = prism_engine::inspect(
        &prism_source,
        options.autocorrect,
        options.target_ruby_version,
        &|cop| cop_enabled(options, cop),
    );
    offenses.extend(
        prism_inspection
            .findings
            .into_iter()
            .map(|finding| prism_offense(&prism_source, finding)),
    );

    line_cops::after_prism(path, &original_lines, options, &mut offenses);

    offenses.sort_by(|left, right| {
        left.line
            .cmp(&right.line)
            .then(left.column.cmp(&right.column))
            .then(left.cop_name.cmp(right.cop_name))
    });

    (offenses, prism_inspection.corrected_source)
}

fn prism_offense(source: &str, finding: prism_engine::Finding) -> Offense {
    let (line, column) = source_position(source, finding.start_offset);
    let mut last_offset = finding
        .end_offset
        .saturating_sub(1)
        .max(finding.start_offset);
    while last_offset > finding.start_offset && !source.is_char_boundary(last_offset) {
        last_offset -= 1;
    }
    let (last_line, last_column) = source_position(source, last_offset);

    Offense {
        cop_name: finding.cop_name,
        message: finding.message,
        corrected: finding.corrected,
        correctable: finding.correctable,
        line,
        column,
        last_line,
        last_column,
        length: finding
            .end_offset
            .saturating_sub(finding.start_offset)
            .max(1),
    }
}

fn source_position(source: &str, offset: usize) -> (usize, usize) {
    let mut offset = offset.min(source.len());
    while offset > 0 && !source.is_char_boundary(offset) {
        offset -= 1;
    }
    let prefix = &source[..offset];
    let line = prefix.bytes().filter(|byte| *byte == b'\n').count() + 1;
    let line_start = prefix.rfind('\n').map_or(0, |index| index + 1);
    let column = source[line_start..offset].chars().count() + 1;
    (line, column)
}

fn split_source(content: &str) -> Vec<SourceLine> {
    if content.is_empty() {
        return Vec::new();
    }

    content
        .split_inclusive('\n')
        .map(|raw_line| {
            if let Some(body) = raw_line.strip_suffix("\r\n") {
                SourceLine {
                    body: body.to_string(),
                    ending: "\r\n".to_string(),
                }
            } else if let Some(body) = raw_line.strip_suffix('\n') {
                SourceLine {
                    body: body.to_string(),
                    ending: "\n".to_string(),
                }
            } else {
                SourceLine {
                    body: raw_line.to_string(),
                    ending: String::new(),
                }
            }
        })
        .collect()
}

fn join_source(lines: &[SourceLine]) -> String {
    let mut content = String::new();

    for line in lines {
        content.push_str(&line.body);
        content.push_str(&line.ending);
    }

    content
}

fn push_offense(
    offenses: &mut Vec<Offense>,
    cop_name: &'static str,
    message: &str,
    line: usize,
    column: usize,
    length: usize,
    correctable: bool,
    corrected: bool,
) {
    offenses.push(Offense {
        cop_name,
        message: message.to_string(),
        corrected,
        correctable,
        line,
        column: column.max(1),
        last_line: line,
        last_column: column.max(1) + length.max(1) - 1,
        length: length.max(1),
    });
}

fn cop_enabled(options: &Options, cop: &str) -> bool {
    let Some(only) = &options.only else {
        return !DEFAULT_DISABLED_COPS.contains(&cop);
    };

    only.split(',').map(str::trim).any(|requested| {
        requested == cop
            || (!DEFAULT_DISABLED_COPS.contains(&cop)
                && cop
                    .strip_prefix(requested)
                    .is_some_and(|suffix| suffix.starts_with('/')))
    })
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_target_ruby_version_from_rubocop_config() {
        let config = "AllCops:\n  TargetRubyVersion: 3.1 # compatibility target\n";

        assert_eq!(
            target_ruby_version_from_source(config),
            Some(prism_engine::RubyVersion::new(3, 1))
        );
    }

    #[test]
    fn ignores_unrelated_configuration() {
        assert_eq!(
            target_ruby_version_from_source("AllCops:\n  NewCops: enable\n"),
            None
        );
    }
}
