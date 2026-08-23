use std::ffi::OsStr;
use std::fs;
use std::io::{self, Read};
use std::path::Path;

use crate::config::RunOptions;
use crate::engine::{self, InspectionPlan, InspectionResult};

pub(super) fn inspect(options: &RunOptions) -> io::Result<Vec<InspectionResult>> {
    let plan = InspectionPlan::new(&options.inspection);
    if let Some(path) = &options.stdin_path {
        return inspect_stdin(path, options, &plan);
    }

    let files = if options.files.is_empty() {
        discover_ruby_files()?
    } else {
        expand_targets(&options.files)?
    };
    let mut results = engine::inspect_files(&files, options, &plan)?;
    for result in &mut results {
        if fs::metadata(&result.path).is_ok_and(|metadata| metadata.len() == 0) {
            for offense in &mut result.offenses {
                if offense.cop_name == "Lint/EmptyFile" && offense.length == 0 {
                    offense.last_column = 1;
                }
            }
        }
    }
    Ok(results)
}

fn inspect_stdin(
    path: &str,
    options: &RunOptions,
    plan: &InspectionPlan,
) -> io::Result<Vec<InspectionResult>> {
    let mut bytes = Vec::new();
    io::stdin().read_to_end(&mut bytes)?;
    let content = crate::engine::source::DecodedSource::from_bytes(&bytes)?;
    let path = engine::expanded_path(path);
    let (offenses, _) = plan.inspect_content(&path, content.as_str(), &options.inspection);
    Ok(vec![InspectionResult { path, offenses }])
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
