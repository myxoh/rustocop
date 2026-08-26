use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use super::unit_contract_runner::{check_cop, inspection_options};
use super::InspectionPlan;
use crate::config::{AutocorrectMode, CopConfig};
use serde::Deserialize;

#[derive(Deserialize)]
struct UnitManifest {
    rubocop_version: String,
    cops: HashMap<String, UnitManifestEntry>,
}

#[derive(Deserialize)]
pub(super) struct UnitManifestEntry {
    pub(super) cases: String,
    pub(super) configs: String,
}

#[test]
fn extension_cops_keep_their_inline_smoke_contracts() {
    let examples = [
        (
            "Rails/ApplicationJob",
            "/project/app/jobs/sync_job.rb",
            "class SyncJob < ActiveJob::Base\nend\n",
            (1, 17, 1, 31, "Jobs should subclass `ApplicationJob`."),
        ),
        (
            "RSpec/Focus",
            "/project/spec/models/user_spec.rb",
            "RSpec.describe User do\n  fit \"works\" do\n    expect(user).to be_valid\n  end\nend\n",
            (2, 3, 2, 16, "Focused spec found."),
        ),
    ];
    for (cop, path, source, (line, column, last_line, last_column, message)) in examples {
        let config = Arc::new(CopConfig::from_source(&format!(
            "---\n{cop}:\n  Enabled: true\n"
        )));
        let options = inspection_options(cop, "2.7", "UTF-8", config, AutocorrectMode::None);
        let plan = InspectionPlan::new(&options);
        let (offenses, _) = plan.inspect_content(path, source, &options);
        assert_eq!(offenses.len(), 1, "{cop}");
        let offense = &offenses[0];
        assert_eq!(
            (
                offense.cop_name.as_str(),
                offense.line,
                offense.column,
                offense.last_line,
                offense.last_column,
                offense.message.as_str(),
            ),
            (cop, line, column, last_line, last_column, message)
        );
    }
}

#[test]
#[ignore = "run explicitly as the strict cached RuboCop parity audit"]
fn cached_unit_contracts_match() {
    let started = Instant::now();
    let manifest_path = fixture_manifest();
    let root = manifest_path.parent().unwrap().to_path_buf();
    let manifest: UnitManifest =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();
    let selected = selected_cops(&manifest);
    let mut failures = Vec::new();
    let mut case_count = 0;
    let mut results = BTreeMap::<String, (usize, usize, Duration)>::new();
    let worker_count = if std::env::var_os("RUSTOCOP_UNIT_BENCHMARK").is_some() {
        1
    } else {
        thread::available_parallelism()
            .map_or(1, usize::from)
            .min(selected.len().max(1))
    };
    let chunk_size = selected.len().div_ceil(worker_count);
    let batches = thread::scope(|scope| {
        selected
            .chunks(chunk_size)
            .map(|chunk| {
                scope.spawn(|| {
                    chunk
                        .iter()
                        .map(|cop| check_cop(&root, cop, manifest.cops.get(cop).unwrap()))
                        .collect::<Vec<_>>()
                })
            })
            .collect::<Vec<_>>()
            .into_iter()
            .flat_map(|handle| handle.join().unwrap())
            .collect::<Vec<_>>()
    });
    for (cop, mut cop_failures, passed, total, duration) in batches {
        failures.append(&mut cop_failures);
        case_count += total;
        results.insert(cop, (passed, total, duration));
    }

    write_report(&manifest.rubocop_version, &results);
    print_timings(&results);

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
    eprintln!(
        "checked {case_count} cached unit cases in {:.3}s",
        started.elapsed().as_secs_f64()
    );
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
}

fn write_report(rubocop_version: &str, results: &BTreeMap<String, (usize, usize, Duration)>) {
    let Ok(path) = std::env::var("RUSTOCOP_UNIT_REPORT") else {
        return;
    };
    let results = results
        .iter()
        .map(|(cop, (passed, total, duration))| {
            (
                cop.clone(),
                serde_json::json!({
                    "passed": passed,
                    "total": total,
                    "duration_ms": duration.as_secs_f64() * 1_000.0,
                }),
            )
        })
        .collect::<serde_json::Map<_, _>>();
    let report = serde_json::json!({
        "rubocop_version": rubocop_version,
        "results": results,
    });
    fs::write(path, serde_json::to_vec_pretty(&report).unwrap()).unwrap();
}

fn print_timings(results: &BTreeMap<String, (usize, usize, Duration)>) {
    if std::env::var_os("RUSTOCOP_UNIT_BENCHMARK").is_none() {
        return;
    }
    let mut timings = results
        .iter()
        .map(|(cop, (_, cases, duration))| (duration.as_secs_f64() * 1_000.0, *cases, cop))
        .collect::<Vec<_>>();
    timings.sort_by(|left, right| left.0.total_cmp(&right.0));
    let percentile = |fraction: f64| {
        let index = ((timings.len() - 1) as f64 * fraction).round() as usize;
        timings[index].0
    };
    let total_ms = timings.iter().map(|(ms, _, _)| ms).sum::<f64>();
    eprintln!(
        "per-cop sequential timings: median {:.3}ms, p95 {:.3}ms, p99 {:.3}ms, mean {:.3}ms",
        percentile(0.5),
        percentile(0.95),
        percentile(0.99),
        total_ms / timings.len() as f64,
    );
    for (milliseconds, cases, cop) in timings.iter().rev().take(10) {
        eprintln!("  {cop}: {milliseconds:.3}ms ({cases} cases)");
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

fn fixture_manifest() -> PathBuf {
    std::env::var_os("RUSTOCOP_UNIT_MANIFEST").map_or_else(
        || Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/fixtures/unit_manifest.json"),
        PathBuf::from,
    )
}
