use std::collections::BTreeMap;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::thread;
use std::time::Instant;

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
        let options = inspection_options(cop, "2.7", config, AutocorrectMode::None);
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
    let root = fixture_root();
    let manifest: UnitManifest =
        serde_json::from_str(&fs::read_to_string(root.join("unit_manifest.json")).unwrap())
            .unwrap();
    let selected = selected_cops(&manifest);
    let mut failures = Vec::new();
    let mut case_count = 0;
    let mut results = BTreeMap::<String, (usize, usize)>::new();
    let worker_count = thread::available_parallelism()
        .map_or(1, usize::from)
        .min(selected.len().max(1));
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
    for (cop, mut cop_failures, passed, total) in batches {
        failures.append(&mut cop_failures);
        case_count += total;
        results.insert(cop, (passed, total));
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

fn fixture_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../spec/fixtures")
}
