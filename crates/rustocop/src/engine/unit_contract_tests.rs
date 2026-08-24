use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Instant;

use serde::Deserialize;
use serde_json::Value;

use super::source::DecodedSource;
use super::InspectionPlan;
use crate::config::{AutocorrectMode, CopConfig, CopSelection, InspectionConfig, RubyVersion};
use crate::model::Offense;

#[derive(Deserialize)]
struct UnitManifest {
    rubocop_version: String,
    cops: HashMap<String, UnitManifestEntry>,
}

#[derive(Deserialize)]
struct UnitManifestEntry {
    cases: String,
    configs: String,
}

#[derive(Deserialize)]
struct UnitCase {
    id: String,
    cop: String,
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

struct ContractFailure {
    cop: String,
    kind: &'static str,
    detail: String,
}

#[test]
#[ignore = "run explicitly as the strict cached RuboCop parity audit"]
fn cached_unit_contracts_match() {
    let started = Instant::now();
    let root = fixture_root();
    let manifest: UnitManifest =
        serde_json::from_str(&fs::read_to_string(root.join("unit_manifest.json")).unwrap())
            .unwrap();
    let selected = selected_cops(&manifest);
    let mut failures = Vec::new();
    let mut case_count = 0;
    let mut results = BTreeMap::<String, (usize, usize)>::new();

    for cop in selected {
        let entry = manifest.cops.get(&cop).unwrap();
        let configs: HashMap<String, String> =
            serde_json::from_str(&fs::read_to_string(root.join(&entry.configs)).unwrap()).unwrap();
        let cases = fs::read_to_string(root.join(&entry.cases)).unwrap();
        let mut plans = HashMap::<String, (Arc<CopConfig>, InspectionPlan)>::new();

        for line in cases.lines() {
            let unit: UnitCase = serde_json::from_str(line).unwrap();
            case_count += 1;
            let failure_count_before = failures.len();
            let (config, plan) = plans.entry(unit.config.clone()).or_insert_with(|| {
                let config = Arc::new(CopConfig::from_source(configs.get(&unit.config).unwrap()));
                let options = inspection_options(
                    &unit.cop,
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
                &unit.cop,
                &unit.ruby_version,
                config.clone(),
                AutocorrectMode::None,
            );
            let (offenses, _) =
                plan.inspect_content(&unit.path, decoded.as_str(), &diagnostic_options);
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
            let result = results.entry(unit.cop.clone()).or_default();
            result.1 += 1;
            if failures.len() == failure_count_before {
                result.0 += 1;
            }
        }
    }

    write_report(&manifest.rubocop_version, &results);

    let mut failure_summary = BTreeMap::new();
    for failure in &failures {
        *failure_summary
            .entry((failure.cop.as_str(), failure.kind))
            .or_insert(0usize) += 1;
    }
    let summary = failure_summary
        .into_iter()
        .map(|((cop, kind), count)| format!("{cop} {kind}: {count}"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(
        failures.is_empty(),
        "{} cached unit failures across {} cases:\n{}\n\nshowing up to 20:\n{}",
        failures.len(),
        case_count,
        summary,
        failures
            .into_iter()
            .take(20)
            .map(|failure| failure.detail)
            .collect::<Vec<_>>()
            .join("\n\n")
    );
    eprintln!(
        "checked {case_count} cached unit cases in {:.3}s",
        started.elapsed().as_secs_f64()
    );
}

fn write_report(rubocop_version: &str, results: &BTreeMap<String, (usize, usize)>) {
    let Ok(path) = std::env::var("RUSTOCOP_UNIT_REPORT") else {
        return;
    };
    let results = results
        .iter()
        .map(|(cop, (passed, total))| {
            (
                cop.clone(),
                serde_json::json!({ "passed": passed, "total": total }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let report = serde_json::json!({
        "rubocop_version": rubocop_version,
        "results": results,
    });
    fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
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
    let options = inspection_options(&unit.cop, &unit.ruby_version, config.clone(), mode);
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

fn inspection_options(
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

fn selected_cops(manifest: &UnitManifest) -> Vec<String> {
    let requested = std::env::var("RUSTOCOP_UNIT_COP").ok();
    let mut cops: Vec<String> = requested.map_or_else(
        || manifest.cops.keys().cloned().collect(),
        |value| {
            value
                .split(',')
                .map(str::trim)
                .map(str::to_string)
                .collect()
        },
    );
    for cop in &cops {
        assert!(
            manifest.cops.contains_key(cop),
            "unknown cached unit cop {cop}"
        );
    }
    cops.sort();
    cops
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

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/fixtures")
}
