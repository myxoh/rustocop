use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use serde_json::Value;

use super::source::DecodedSource;
use super::unit_contract_tests::UnitManifestEntry;
use super::InspectionPlan;
use crate::config::{
    AutocorrectMode, CopConfig, CopSelection, InspectionConfig, RubyVersion, SourceEncoding,
};
use crate::model::Offense;

#[derive(Deserialize)]
struct UnitCase {
    id: String,
    cop: String,
    selection: Option<String>,
    source: Value,
    path: String,
    ruby_version: String,
    external_encoding: String,
    file_mode: Option<u32>,
    config: String,
    diagnostics: Vec<ExpectedOffense>,
    autocorrect_checked: bool,
    autocorrect_all: Option<Value>,
    autocorrect_all_error: Option<String>,
    autocorrect_safe: Option<Value>,
    autocorrect_safe_error: Option<String>,
}

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct ExpectedOffense {
    message: String,
    severity: String,
    correctable: bool,
    line: usize,
    column: usize,
    last_line: usize,
    last_column: usize,
}

pub(super) struct ContractFailure {
    pub(super) cop: String,
    pub(super) kind: &'static str,
    pub(super) detail: String,
}

pub(super) fn check_cop(
    root: &Path,
    cop: &str,
    entry: &UnitManifestEntry,
) -> (String, Vec<ContractFailure>, usize, usize, Duration) {
    let started = Instant::now();
    if std::env::var_os("RUSTOCOP_UNIT_TRACE").is_some() {
        eprintln!("checking {cop}");
    }
    let configs: HashMap<String, String> =
        serde_json::from_str(&fs::read_to_string(root.join(&entry.configs)).unwrap()).unwrap();
    let cases = fs::read_to_string(root.join(&entry.cases)).unwrap();
    let requested_case = std::env::var("RUSTOCOP_UNIT_CASE").ok();
    let mut plans = HashMap::<(String, String), (Arc<CopConfig>, InspectionPlan)>::new();
    let mut failures = Vec::new();
    let mut passed = 0;
    let mut total = 0;

    for line in cases.lines() {
        let unit: UnitCase = serde_json::from_str(line).unwrap();
        if requested_case
            .as_deref()
            .is_some_and(|requested| requested != unit.id)
        {
            continue;
        }
        if std::env::var_os("RUSTOCOP_UNIT_TRACE").is_some() {
            eprintln!("checking {} {}", unit.cop, unit.id);
        }
        total += 1;
        let failure_count_before = failures.len();
        let selection = unit.selection.as_deref().unwrap_or(&unit.cop);
        let plan_key = (unit.config.clone(), selection.to_string());
        let (config, plan) = plans.entry(plan_key).or_insert_with(|| {
            let config = Arc::new(
                CopConfig::from_source(configs.get(&unit.config).unwrap()).without_path_policy(),
            );
            let options = inspection_options(
                selection,
                &unit.ruby_version,
                &unit.external_encoding,
                config.clone(),
                AutocorrectMode::None,
            );
            let plan = InspectionPlan::new(&options);
            (config, plan)
        });
        let original = source_bytes(&unit.source);
        let decoded = DecodedSource::from_bytes(&original).unwrap();
        let materialized_path = materialize_file_metadata(&unit, &original);
        let inspection_path = materialized_path
            .as_deref()
            .and_then(Path::to_str)
            .unwrap_or(&unit.path);
        let diagnostic_options = inspection_options(
            unit.selection.as_deref().unwrap_or(&unit.cop),
            &unit.ruby_version,
            &unit.external_encoding,
            config.clone(),
            AutocorrectMode::None,
        );
        let (offenses, _) =
            plan.inspect_content(inspection_path, decoded.as_str(), &diagnostic_options);
        let actual = offenses.iter().map(expected_offense).collect::<Vec<_>>();
        if actual != unit.diagnostics {
            failures.push(ContractFailure {
                cop: unit.cop.clone(),
                kind: "diagnostics",
                detail: format!(
                    "{} {} diagnostics\n{}",
                    unit.cop,
                    unit.id,
                    diagnostic_diff(&unit.diagnostics, &actual),
                ),
            });
        }
        if unit.autocorrect_checked {
            check_correction(
                &unit,
                config,
                plan,
                &decoded,
                &original,
                inspection_path,
                &offenses,
                AutocorrectMode::All,
                &unit.autocorrect_all,
                unit.autocorrect_all_error.as_deref(),
                &mut failures,
            );
            check_correction(
                &unit,
                config,
                plan,
                &decoded,
                &original,
                inspection_path,
                &offenses,
                AutocorrectMode::Safe,
                &unit.autocorrect_safe,
                unit.autocorrect_safe_error.as_deref(),
                &mut failures,
            );
        }
        if let Some(path) = materialized_path {
            let _ = fs::remove_dir_all(path.parent().unwrap_or(&path));
        }
        if failures.len() == failure_count_before {
            passed += 1;
        }
    }
    (cop.to_string(), failures, passed, total, started.elapsed())
}

fn diagnostic_diff(expected: &[ExpectedOffense], actual: &[ExpectedOffense]) -> String {
    let first = expected
        .iter()
        .zip(actual)
        .position(|(expected, actual)| expected != actual)
        .unwrap_or_else(|| expected.len().min(actual.len()));
    format!(
        "first differing offense: {}\nexpected ({}): {:?}\nactual ({}): {:?}",
        first + 1,
        expected.len(),
        expected.get(first),
        actual.len(),
        actual.get(first),
    )
}

#[allow(clippy::too_many_arguments)]
fn check_correction(
    unit: &UnitCase,
    config: &Arc<CopConfig>,
    plan: &InspectionPlan,
    decoded: &DecodedSource,
    original: &[u8],
    inspection_path: &str,
    diagnostic_offenses: &[Offense],
    mode: AutocorrectMode,
    cached: &Option<Value>,
    cached_error: Option<&str>,
    failures: &mut Vec<ContractFailure>,
) {
    // RuboCop itself could not converge for this cached input. Preserve the
    // exact terminal output/error as evidence, but do not feed an ever-growing
    // correction back through Rustocop merely to reproduce an upstream abort.
    if matches!(cached_error, Some("infinite_loop" | "maximum_iterations")) {
        return;
    }
    let options = inspection_options(
        correction_selection(unit, config, diagnostic_offenses, mode),
        &unit.ruby_version,
        &unit.external_encoding,
        config.clone(),
        mode,
    );
    let correction_plan;
    let plan = if unit
        .selection
        .as_deref()
        .is_some_and(|selection| selection.contains(','))
    {
        correction_plan = InspectionPlan::new(&options);
        &correction_plan
    } else {
        plan
    };
    let (_, corrected, actual_error) =
        plan.inspect_content_with_corrections(inspection_path, decoded.as_str(), &options);
    let actual_error = actual_error.map(|error| error.as_str());
    let actual = decoded.restore(&corrected);
    let expected = expected_correction(cached, &unit.autocorrect_all, original);
    let mode_name = match mode {
        AutocorrectMode::Safe => "-a",
        AutocorrectMode::All => "-A",
        AutocorrectMode::None => unreachable!(),
    };
    if cached_error != actual_error {
        failures.push(ContractFailure {
            cop: unit.cop.clone(),
            kind: mode_name,
            detail: format!(
                "{} {} {mode_name} error\nexpected: {:?}\nactual:   {:?}\nterminal source: {:?}",
                unit.cop,
                unit.id,
                cached_error,
                actual_error,
                String::from_utf8_lossy(&actual)
            ),
        });
    } else if actual != expected {
        failures.push(ContractFailure {
            cop: unit.cop.clone(),
            kind: mode_name,
            detail: format!(
                "{} {} {mode_name} correction\n{}",
                unit.cop,
                unit.id,
                correction_diff(&expected, &actual),
            ),
        });
    }
}

fn correction_diff(expected: &[u8], actual: &[u8]) -> String {
    let expected = String::from_utf8_lossy(expected);
    let actual = String::from_utf8_lossy(actual);
    let expected_lines = expected.split_inclusive('\n').collect::<Vec<_>>();
    let actual_lines = actual.split_inclusive('\n').collect::<Vec<_>>();
    let first = expected_lines
        .iter()
        .zip(&actual_lines)
        .position(|(expected, actual)| expected != actual)
        .unwrap_or_else(|| expected_lines.len().min(actual_lines.len()));
    let start = first.saturating_sub(2);
    let end = (first + 3).max(start + 1);
    let context = |lines: &[&str]| {
        lines
            .iter()
            .enumerate()
            .skip(start)
            .take(end.saturating_sub(start))
            .map(|(index, line)| format!("{:>5}: {line:?}", index + 1))
            .collect::<Vec<_>>()
            .join("\n")
    };
    format!(
        "first differing line: {}\nexpected ({} bytes):\n{}\nactual ({} bytes):\n{}",
        first + 1,
        expected.len(),
        context(&expected_lines),
        actual.len(),
        context(&actual_lines),
    )
}

fn correction_selection<'a>(
    unit: &'a UnitCase,
    config: &CopConfig,
    diagnostic_offenses: &'a [Offense],
    mode: AutocorrectMode,
) -> &'a str {
    let selected = unit
        .selection
        .as_deref()
        .unwrap_or(&unit.cop)
        .split(',')
        .map(str::trim)
        .find(|cop| mode.enabled_for(config, cop))
        .unwrap_or(&unit.cop);
    if diagnostic_offenses
        .iter()
        .any(|offense| offense.cop_name == selected && offense.correctable)
    {
        return selected;
    }
    diagnostic_offenses
        .iter()
        .find(|offense| {
            offense.correctable
                && unit
                    .selection
                    .as_deref()
                    .unwrap_or(&unit.cop)
                    .split(',')
                    .map(str::trim)
                    .any(|cop| cop == offense.cop_name && mode.enabled_for(config, cop))
        })
        .map_or(selected, |offense| offense.cop_name.as_str())
}

fn materialize_file_metadata(unit: &UnitCase, source: &[u8]) -> Option<std::path::PathBuf> {
    let mode = unit.file_mode?;
    let file_name = Path::new(&unit.path).file_name()?;
    let directory = std::env::temp_dir()
        .join(format!("rustocop-unit-contracts-{}", std::process::id()))
        .join(&unit.id);
    fs::create_dir_all(&directory).ok()?;
    let path = directory.join(file_name);
    fs::write(&path, source).ok()?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(mode)).ok()?;
    }
    Some(path)
}

pub(super) fn inspection_options(
    cop: &str,
    ruby_version: &str,
    external_encoding: &str,
    config: Arc<CopConfig>,
    autocorrect: AutocorrectMode,
) -> InspectionConfig {
    InspectionConfig {
        autocorrect,
        // RuboCop::Cop::Team exposes raw offenses during investigation, while
        // its correction pass still honors inline disable comments. The unit
        // cache records those two upstream contracts independently.
        ignore_disable_comments: autocorrect == AutocorrectMode::None,
        cops: CopSelection::only(cop),
        target_ruby_version: RubyVersion::parse(ruby_version).unwrap(),
        source_encoding: SourceEncoding::parse(external_encoding),
        cop_config: config,
        inspected_path: None,
        registry_context: None,
    }
}

fn expected_offense(offense: &Offense) -> ExpectedOffense {
    ExpectedOffense {
        message: offense.message.clone(),
        severity: offense_severity(&offense.cop_name).to_string(),
        correctable: offense.correctable,
        line: offense.line,
        column: offense.column,
        last_line: offense.last_line,
        last_column: offense.last_column,
    }
}

fn offense_severity(cop: &str) -> &'static str {
    if cop == "Lint/Syntax" {
        "fatal"
    } else if cop.starts_with("Lint/")
        || matches!(
            cop,
            "Bundler/DuplicatedGem"
                | "Bundler/DuplicatedGroup"
                | "Gemspec/RequireMFA"
                | "Bundler/InsecureProtocolSource"
                | "Gemspec/RubyVersionGlobalsUsage"
                | "Gemspec/DuplicatedAssignment"
                | "Gemspec/DeprecatedAttributeAssignment"
                | "Gemspec/RequiredRubyVersion"
                | "Layout/BeginEndAlignment"
                | "Layout/DefEndAlignment"
                | "Layout/EndAlignment"
        )
    {
        "warning"
    } else {
        "convention"
    }
}

fn expected_correction(
    cached: &Option<Value>,
    all_cached: &Option<Value>,
    original: &[u8],
) -> Vec<u8> {
    match cached {
        None => original.to_vec(),
        Some(Value::String(value)) if value == "$all" => {
            expected_correction(all_cached, &None, original)
        }
        Some(value) => source_bytes(value),
    }
}

fn source_bytes(value: &Value) -> Vec<u8> {
    match value {
        Value::String(source) => source.as_bytes().to_vec(),
        Value::Object(encoded) => decode_hex(encoded.get("hex").unwrap().as_str().unwrap()),
        _ => panic!("invalid cached source value"),
    }
}

fn decode_hex(value: &str) -> Vec<u8> {
    value
        .as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let text = std::str::from_utf8(pair).unwrap();
            u8::from_str_radix(text, 16).unwrap()
        })
        .collect()
}
