use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use super::{InspectionPlan, InspectionResult};
use crate::config::{Parallelism, RunOptions};

pub(crate) fn inspect_files(
    files: &[String],
    options: &RunOptions,
    plan: &InspectionPlan,
) -> io::Result<Vec<InspectionResult>> {
    let parallelism = if options.inspection.autocorrect_enabled() && contains_duplicate_files(files)
    {
        Parallelism::Sequential
    } else {
        options.parallelism
    };
    let worker_count = worker_count(parallelism, files.len());
    if worker_count <= 1 {
        return files
            .iter()
            .map(|path| plan.inspect_file(path, &options.inspection))
            .collect();
    }

    let next_file = AtomicUsize::new(0);
    thread::scope(|scope| {
        let handles = (0..worker_count)
            .map(|_worker| {
                let next_file = &next_file;
                scope.spawn(move || -> io::Result<Vec<(usize, InspectionResult)>> {
                    let mut results = Vec::new();
                    loop {
                        let index = next_file.fetch_add(1, Ordering::Relaxed);
                        let Some(path) = files.get(index) else {
                            return Ok(results);
                        };
                        results.push((index, plan.inspect_file(path, &options.inspection)?));
                    }
                })
            })
            .collect::<Vec<_>>();

        let mut indexed_results = Vec::with_capacity(files.len());
        for handle in handles {
            indexed_results.extend(
                handle
                    .join()
                    .map_err(|_| io::Error::other("rustocop file worker panicked"))??,
            );
        }
        indexed_results.sort_by_key(|(index, _result)| *index);
        Ok(indexed_results
            .into_iter()
            .map(|(_index, result)| result)
            .collect())
    })
}

fn worker_count(parallelism: Parallelism, file_count: usize) -> usize {
    let requested = match parallelism {
        Parallelism::Sequential => 1,
        Parallelism::Automatic => thread::available_parallelism().map_or(1, usize::from),
        Parallelism::Fixed(jobs) => jobs,
    };
    requested.min(file_count.max(1))
}

fn contains_duplicate_files(files: &[String]) -> bool {
    let mut seen = HashSet::with_capacity(files.len());
    files.iter().any(|path| {
        let identity = fs::canonicalize(path).unwrap_or_else(|_| PathBuf::from(path));
        !seen.insert(identity)
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AutocorrectMode, CopConfig, CopSelection, InspectionConfig, RubyVersion};
    use std::sync::Arc;

    fn options(parallelism: Parallelism) -> RunOptions {
        RunOptions {
            files: Vec::new(),
            format: "json".to_string(),
            stdin_path: None,
            parallelism,
            rubocop_loaders: Vec::new(),
            config_path: None,
            inspection: InspectionConfig {
                autocorrect: AutocorrectMode::None,
                cops: CopSelection::only("Lint/EmptyExpression"),
                target_ruby_version: RubyVersion::default(),
                cop_config: Arc::new(CopConfig::default()),
            },
        }
    }

    #[test]
    fn bounds_workers_by_file_count() {
        assert_eq!(worker_count(Parallelism::Sequential, 100), 1);
        assert_eq!(worker_count(Parallelism::Fixed(8), 3), 3);
        assert_eq!(worker_count(Parallelism::Fixed(8), 0), 1);
    }

    #[test]
    fn parallel_inspection_preserves_sequential_output_order() {
        let fixture_root = format!(
            "{}/../../spec/fixtures/shared/engine/parallel_order",
            env!("CARGO_MANIFEST_DIR")
        );
        let files = [
            "empty_expression_03.rb",
            "empty_expression_01.rb",
            "empty_expression_02.rb",
        ]
        .map(|name| format!("{fixture_root}/{name}"));

        let sequential_options = options(Parallelism::Sequential);
        let parallel_options = options(Parallelism::Fixed(3));
        let sequential_plan = InspectionPlan::new(&sequential_options.inspection);
        let parallel_plan = InspectionPlan::new(&parallel_options.inspection);
        let sequential = inspect_files(&files, &sequential_options, &sequential_plan).unwrap();
        let parallel = inspect_files(&files, &parallel_options, &parallel_plan).unwrap();
        let snapshot = |results: &[InspectionResult]| {
            results
                .iter()
                .map(|result| {
                    (
                        result.path.clone(),
                        result
                            .offenses
                            .iter()
                            .map(|offense| (offense.cop_name.clone(), offense.line, offense.column))
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(snapshot(&sequential), snapshot(&parallel));
    }

    #[test]
    fn recognizes_duplicate_correction_targets() {
        let path = format!(
            "{}/../../spec/fixtures/shared/engine/parallel_order/empty_expression_01.rb",
            env!("CARGO_MANIFEST_DIR")
        );

        assert!(contains_duplicate_files(&[path.clone(), path]));
    }
}
