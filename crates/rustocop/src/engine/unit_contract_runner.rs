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
use crate::config::{AutocorrectMode, CopConfig, CopSelection, InspectionConfig, RubyVersion};
use crate::model::Offense;

#[derive(Deserialize)]
struct UnitCase {
    id: String,
    cop: String,
    selection: Option<String>,
    source: Value,
    path: String,
    ruby_version: String,
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
    let mut plans = HashMap::<String, (Arc<CopConfig>, InspectionPlan)>::new();
    let mut failures = Vec::new();
    let mut passed = 0;
    let mut total = 0;

    for line in cases.lines() {
        let unit: UnitCase = serde_json::from_str(line).unwrap();
        if std::env::var_os("RUSTOCOP_UNIT_TRACE").is_some() {
            eprintln!("checking {} {}", unit.cop, unit.id);
        }
        total += 1;
        let failure_count_before = failures.len();
        let (config, plan) = plans.entry(unit.config.clone()).or_insert_with(|| {
            let config = Arc::new(CopConfig::from_source(configs.get(&unit.config).unwrap()));
            let options = inspection_options(
                unit.selection.as_deref().unwrap_or(&unit.cop),
                &unit.ruby_version,
                config.clone(),
                AutocorrectMode::None,
            );
            let plan = InspectionPlan::new(&options);
            (config, plan)
        });
        let original = source_bytes(&unit.source);
        let decoded = DecodedSource::from_bytes(&original).unwrap();
        let diagnostic_options = inspection_options(
            unit.selection.as_deref().unwrap_or(&unit.cop),
            &unit.ruby_version,
            config.clone(),
            AutocorrectMode::None,
        );
        let (offenses, _) = plan.inspect_content(&unit.path, decoded.as_str(), &diagnostic_options);
        let actual = offenses.iter().map(expected_offense).collect::<Vec<_>>();
        if actual != unit.diagnostics {
            failures.push(ContractFailure {
                cop: unit.cop.clone(),
                kind: "diagnostics",
                detail: format!(
                    "{} {} diagnostics\nexpected: {:?}\nactual:   {:?}",
                    unit.cop, unit.id, unit.diagnostics, actual
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
                AutocorrectMode::Safe,
                &unit.autocorrect_safe,
                unit.autocorrect_safe_error.as_deref(),
                &mut failures,
            );
        }
        if failures.len() == failure_count_before {
            passed += 1;
        }
    }
    (cop.to_string(), failures, passed, total, started.elapsed())
}

#[allow(clippy::too_many_arguments)]
fn check_correction(
    unit: &UnitCase,
    config: &Arc<CopConfig>,
    plan: &InspectionPlan,
    decoded: &DecodedSource,
    original: &[u8],
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
        unit.selection.as_deref().unwrap_or(&unit.cop),
        &unit.ruby_version,
        config.clone(),
        mode,
    );
    let (_, corrected, actual_error) =
        plan.inspect_content_with_corrections(&unit.path, decoded.as_str(), &options);
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
                "{} {} {mode_name} error\nexpected: {:?}\nactual:   {:?}",
                unit.cop, unit.id, cached_error, actual_error
            ),
        });
    } else if actual != expected {
        failures.push(ContractFailure {
            cop: unit.cop.clone(),
            kind: mode_name,
            detail: format!(
                "{} {} {mode_name} correction\nexpected: {:?}\nactual:   {:?}",
                unit.cop,
                unit.id,
                String::from_utf8_lossy(&expected),
                String::from_utf8_lossy(&actual)
            ),
        });
    }
}

pub(super) fn inspection_options(
    cop: &str,
    ruby_version: &str,
    config: Arc<CopConfig>,
    autocorrect: AutocorrectMode,
) -> InspectionConfig {
    InspectionConfig {
        autocorrect,
        cops: CopSelection::only(cop),
        target_ruby_version: RubyVersion::parse(ruby_version).unwrap(),
        cop_config: config,
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
