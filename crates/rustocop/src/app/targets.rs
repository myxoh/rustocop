use std::collections::HashSet;
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
        discover_ruby_files(&options.inspection.cop_config)?
    } else {
        expand_targets(options)?
    };
    let mut results = engine::inspect_files(&files, options, &plan)?;
    for result in &mut results {
        collapse_cli_duplicate_locations(result);
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
    let mut result = InspectionResult { path, offenses };
    collapse_cli_duplicate_locations(&mut result);
    Ok(vec![result])
}

fn collapse_cli_duplicate_locations(result: &mut InspectionResult) {
    let mut seen = HashSet::new();
    result.offenses.reverse();
    result.offenses.retain(|offense| {
        !matches!(
            offense.cop_name.as_str(),
            "Lint/AmbiguousOperatorPrecedence" | "Lint/EmptyBlock"
        ) || seen.insert((offense.cop_name.clone(), offense.line, offense.column))
    });
    result.offenses.reverse();
}

fn expand_targets(options: &RunOptions) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    for target in &options.files {
        let path = Path::new(target);
        if path.is_dir() {
            discover_ruby_files_under(path, &options.inspection.cop_config, &mut files)?;
        } else if !options.force_exclusion
            || !options.inspection.cop_config.all_cops_excluded(target)
        {
            files.push(target.to_string());
        }
    }
    Ok(files)
}

fn discover_ruby_files(config: &crate::config::CopConfig) -> io::Result<Vec<String>> {
    let mut files = Vec::new();
    discover_ruby_files_under(Path::new("."), config, &mut files)?;
    Ok(files)
}

fn discover_ruby_files_under(
    path: &Path,
    config: &crate::config::CopConfig,
    files: &mut Vec<String>,
) -> io::Result<()> {
    for entry in fs::read_dir(path)? {
        let entry = entry?;
        let entry_path = entry.path();
        let file_name = entry.file_name();
        let file_name = file_name.to_string_lossy();

        if should_skip_entry(&entry_path, &file_name) {
            continue;
        }
        if entry_path.is_dir() {
            discover_ruby_files_under(&entry_path, config, files)?;
        } else if config.target_included(&entry_path)
            && !config.all_cops_excluded(&entry_path.to_string_lossy())
        {
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn discovery_uses_all_cops_include_and_exclude_patterns() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory =
            std::env::temp_dir().join(format!("rustocop-targets-{}-{nonce}", std::process::id()));
        fs::create_dir_all(directory.join("app/views")).unwrap();
        fs::write(directory.join("app/model.rb"), "class Model; end\n").unwrap();
        fs::write(
            directory.join("app/views/index.json.jbuilder"),
            "json.id 1\n",
        )
        .unwrap();
        fs::write(directory.join("Dangerfile"), "warn 'example'\n").unwrap();
        let config_path = directory.join(".rubocop.yml");
        fs::write(&config_path, "AllCops:\n  Exclude:\n    - Dangerfile\n").unwrap();
        let config = crate::config::CopConfig::from_path(config_path.to_str().unwrap()).unwrap();
        let mut files = Vec::new();

        discover_ruby_files_under(&directory, &config, &mut files).unwrap();
        let names = files
            .iter()
            .filter_map(|path| Path::new(path).file_name()?.to_str())
            .collect::<Vec<_>>();
        assert!(names.contains(&"model.rb"));
        assert!(names.contains(&"index.json.jbuilder"));
        assert!(!names.contains(&"Dangerfile"));
        fs::remove_dir_all(directory).unwrap();
    }
}
