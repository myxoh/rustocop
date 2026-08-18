use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;

use crate::{inspect_file, InspectionResult, Options, Parallelism};

pub(super) fn inspect_files(
    files: &[String],
    options: &Options,
) -> io::Result<Vec<InspectionResult>> {
    let parallelism = if options.autocorrect && contains_duplicate_files(files) {
        Parallelism::Sequential
    } else {
        options.parallelism
    };
    let worker_count = worker_count(parallelism, files.len());
    if worker_count <= 1 {
        return files
            .iter()
            .map(|path| inspect_file(path, options))
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
                        results.push((index, inspect_file(path, options)?));
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
    use crate::json_report;

    fn options(parallelism: Parallelism) -> Options {
        Options {
            autocorrect: false,
            files: Vec::new(),
            format: "json".to_string(),
            only: Some("Lint/EmptyExpression".to_string()),
            stdin_path: None,
            target_ruby_version: crate::prism_engine::RubyVersion::default(),
            parallelism,
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
            "{}/../../spec/fixtures/rubocop_builtin_examples/lint_empty_expression",
            env!("CARGO_MANIFEST_DIR")
        );
        let files = ["03_offense.rb", "01_offense.rb", "02_offense.rb"]
            .map(|name| format!("{fixture_root}/{name}"));

        let sequential = inspect_files(&files, &options(Parallelism::Sequential)).unwrap();
        let parallel = inspect_files(&files, &options(Parallelism::Fixed(3))).unwrap();
        let sequential_offenses = sequential.iter().map(|result| result.offenses.len()).sum();
        let parallel_offenses = parallel.iter().map(|result| result.offenses.len()).sum();

        assert_eq!(
            json_report(&sequential, sequential_offenses),
            json_report(&parallel, parallel_offenses)
        );
    }

    #[test]
    fn recognizes_duplicate_correction_targets() {
        let path = format!(
            "{}/../../spec/fixtures/rubocop_builtin_examples/lint_empty_expression/01_offense.rb",
            env!("CARGO_MANIFEST_DIR")
        );

        assert!(contains_duplicate_files(&[path.clone(), path]));
    }
}
