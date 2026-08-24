use std::collections::HashMap;
use std::env;
use std::process::{Command, Output, Stdio};

use serde::Deserialize;

use super::{fail, report, targets};
use crate::config::RunOptions;
use crate::engine::{expanded_path, InspectionResult};
use crate::model::Offense;

pub(super) fn custom_cops(options: &RunOptions) -> Option<Vec<String>> {
    if options.rubocop_loaders.is_empty() {
        return None;
    }
    let requested = options.inspection.cops.requested()?;
    let native = crate::cops::cop_names();
    let custom = requested
        .iter()
        .filter(|selection| !native_selection(selection, &native))
        .cloned()
        .collect::<Vec<_>>();
    (!custom.is_empty()).then_some(custom)
}

pub(super) fn run(options: &RunOptions, custom_cops: &[String]) -> i32 {
    if options.inspection.autocorrect_enabled() {
        return fail("mixed custom-cop runs do not yet support autocorrection");
    }
    if options.stdin_path.is_some() {
        return fail("mixed custom-cop runs do not yet support --stdin");
    }

    match inspect(options, custom_cops) {
        Ok(results) => {
            report::write(options, &results);
            report::exit_status(options, &results)
        }
        Err(error) => fail(error),
    }
}

fn inspect(options: &RunOptions, custom_cops: &[String]) -> Result<Vec<InspectionResult>, String> {
    let child = rubocop_command(options, custom_cops)
        .spawn()
        .map_err(|error| format!("failed to launch RuboCop for custom cops: {error}"))?;
    let native = targets::inspect(options);
    let output = child
        .wait_with_output()
        .map_err(|error| format!("failed waiting for RuboCop custom cops: {error}"))?;
    let mut results = native.map_err(|error| error.to_string())?;
    merge_custom_report(&mut results, parse_output(output)?);
    Ok(results)
}

fn rubocop_command(options: &RunOptions, custom_cops: &[String]) -> Command {
    let executable = env::var("RUSTOCOP_RUBOCOP_PATH").unwrap_or_else(|_| "rubocop".to_string());
    let mut command = Command::new(executable);
    command.args(["--cache", "false", "--no-server", "--format", "json"]);
    for (name, value) in &options.rubocop_loaders {
        command.args([name, value]);
    }
    if let Some(path) = &options.config_path {
        command.args(["--config", path]);
    }
    command.args(["--only", &custom_cops.join(",")]);
    command.args(&options.files);
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    command
}

fn parse_output(output: Output) -> Result<ExternalReport, String> {
    if !output.stderr.is_empty() {
        eprint!("{}", String::from_utf8_lossy(&output.stderr));
    }
    if !matches!(output.status.code(), Some(0 | 1)) {
        return Err(format!(
            "RuboCop custom-cop run failed with status {}",
            output.status
        ));
    }
    serde_json::from_slice(&output.stdout)
        .map_err(|error| format!("invalid RuboCop custom-cop JSON: {error}"))
}

fn merge_custom_report(results: &mut Vec<InspectionResult>, report: ExternalReport) {
    let mut indices = results
        .iter()
        .enumerate()
        .map(|(index, result)| (result.path.clone(), index))
        .collect::<HashMap<_, _>>();
    for file in report.files {
        let path = expanded_path(&file.path);
        let index = *indices.entry(path.clone()).or_insert_with(|| {
            results.push(InspectionResult {
                path: path.clone(),
                offenses: Vec::new(),
            });
            results.len() - 1
        });
        for offense in file.offenses {
            results[index].offenses.push(offense.into_offense());
        }
        results[index].offenses.sort_by(|left, right| {
            left.line
                .cmp(&right.line)
                .then(left.column.cmp(&right.column))
                .then(left.cop_name.cmp(&right.cop_name))
                .then(left.message.cmp(&right.message))
        });
    }
}

fn native_selection(selection: &str, native: &[&str]) -> bool {
    let prefix = format!("{selection}/");
    native
        .iter()
        .any(|name| *name == selection || name.starts_with(&prefix))
}

#[derive(Deserialize)]
struct ExternalReport {
    files: Vec<ExternalFile>,
}

#[derive(Deserialize)]
struct ExternalFile {
    path: String,
    offenses: Vec<ExternalOffense>,
}

#[derive(Deserialize)]
struct ExternalOffense {
    message: String,
    cop_name: String,
    corrected: bool,
    correctable: bool,
    location: ExternalLocation,
}

impl ExternalOffense {
    fn into_offense(self) -> Offense {
        Offense {
            cop_name: self.cop_name,
            message: self.message,
            corrected: self.corrected,
            correctable: self.correctable,
            line: self.location.start_line,
            column: self.location.start_column,
            last_line: self.location.last_line,
            last_column: self.location.last_column,
            length: self.location.length.max(1),
        }
    }
}

#[derive(Deserialize)]
struct ExternalLocation {
    start_line: usize,
    start_column: usize,
    last_line: usize,
    last_column: usize,
    length: usize,
}
