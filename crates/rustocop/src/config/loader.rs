use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::CopConfig;

pub(super) fn load(path: &str) -> Result<CopConfig, String> {
    let requested = PathBuf::from(path);
    let requested = if requested.is_absolute() {
        requested
    } else {
        std::env::current_dir()
            .map_err(|error| format!("could not resolve config {path}: {error}"))?
            .join(requested)
    };
    let root = requested
        .parent()
        .unwrap_or_else(|| Path::new("."))
        .to_path_buf();
    let mut stack = Vec::new();
    let mut layers = Vec::new();
    load_file(&requested, &mut stack, &mut layers, &resolve_gem)?;
    Ok(CopConfig::from_sources(
        layers
            .iter()
            .map(|layer| (layer.source.as_str(), layer.merge_keys.clone())),
        Some(root),
    ))
}

struct Layer {
    source: String,
    merge_keys: HashSet<String>,
}

fn load_file(
    path: &Path,
    stack: &mut Vec<PathBuf>,
    layers: &mut Vec<Layer>,
    gem_resolver: &dyn Fn(&str) -> Result<PathBuf, String>,
) -> Result<(), String> {
    let canonical = fs::canonicalize(path)
        .map_err(|error| format!("could not read config {}: {error}", path.display()))?;
    if let Some(cycle_start) = stack.iter().position(|entry| entry == &canonical) {
        let mut cycle = stack[cycle_start..]
            .iter()
            .map(|entry| entry.display().to_string())
            .collect::<Vec<_>>();
        cycle.push(canonical.display().to_string());
        return Err(format!(
            "configuration inheritance cycle: {}",
            cycle.join(" -> ")
        ));
    }
    let source = fs::read_to_string(&canonical)
        .map_err(|error| format!("could not read config {}: {error}", canonical.display()))?;
    let directives = Directives::parse(&source);
    stack.push(canonical.clone());
    let directory = canonical.parent().unwrap_or_else(|| Path::new("."));
    for inherited in directives.inherit_from {
        if inherited.contains(['*', '?', '{']) {
            return Err(format!(
                "unsupported glob in inherit_from {inherited} from {}",
                canonical.display()
            ));
        }
        load_file(&directory.join(inherited), stack, layers, gem_resolver)?;
    }
    for (gem, inherited) in directives.inherit_gem {
        let gem_root = gem_resolver(&gem)?;
        load_file(&gem_root.join(inherited), stack, layers, gem_resolver)?;
    }
    stack.pop();
    layers.push(Layer {
        source,
        merge_keys: directives.merge_keys,
    });
    Ok(())
}

fn resolve_gem(name: &str) -> Result<PathBuf, String> {
    let ruby = std::env::var("RUSTOCOP_RUBY_PATH").unwrap_or_else(|_| "ruby".to_string());
    let output = Command::new(ruby)
        .args([
            "-e",
            "require 'rubygems'; print Gem::Specification.find_by_name(ARGV.fetch(0)).full_gem_path",
            name,
        ])
        .output()
        .map_err(|error| format!("could not resolve inherited gem {name}: {error}"))?;
    if !output.status.success() {
        let message = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "could not resolve inherited gem {name}: {}",
            message.trim()
        ));
    }
    Ok(PathBuf::from(
        String::from_utf8_lossy(&output.stdout).trim(),
    ))
}

#[derive(Default)]
struct Directives {
    inherit_from: Vec<String>,
    inherit_gem: Vec<(String, String)>,
    merge_keys: HashSet<String>,
}

impl Directives {
    fn parse(source: &str) -> Self {
        let mut directives = Self::default();
        let lines = source.lines().collect::<Vec<_>>();
        let mut index = 0;
        while index < lines.len() {
            let line = lines[index];
            index += 1;
            if line.starts_with(char::is_whitespace) {
                continue;
            }
            if let Some(value) = line.trim().strip_prefix("inherit_from:") {
                let value = clean(value);
                if value.is_empty() {
                    while index < lines.len() && lines[index].starts_with(char::is_whitespace) {
                        if let Some(item) = lines[index].trim().strip_prefix("- ") {
                            directives.inherit_from.push(clean(item));
                        }
                        index += 1;
                    }
                } else if value.starts_with('[') && value.ends_with(']') {
                    directives.inherit_from.extend(
                        value[1..value.len() - 1]
                            .split(',')
                            .map(clean)
                            .filter(|value| !value.is_empty()),
                    );
                } else {
                    directives.inherit_from.push(value);
                }
            } else if line.trim() == "inherit_gem:" {
                while index < lines.len() && lines[index].starts_with(char::is_whitespace) {
                    let nested = lines[index].trim();
                    if let Some((gem, file)) = nested.split_once(':') {
                        directives.inherit_gem.push((clean(gem), clean(file)));
                    }
                    index += 1;
                }
            } else if line.trim() == "inherit_mode:" {
                let mut in_merge = false;
                while index < lines.len() && lines[index].starts_with(char::is_whitespace) {
                    let nested = lines[index].trim();
                    if nested == "merge:" {
                        in_merge = true;
                    } else if in_merge {
                        if let Some(key) = nested.strip_prefix("- ") {
                            directives.merge_keys.insert(clean(key));
                        }
                    }
                    index += 1;
                }
            }
        }
        directives
    }
}

fn clean(value: &str) -> String {
    value
        .split('#')
        .next()
        .unwrap_or_default()
        .trim()
        .trim_matches(['\'', '"'])
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(0);

    fn temporary_directory() -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let directory = std::env::temp_dir().join(format!(
            "rustocop-config-loader-{}-{nonce}-{}",
            std::process::id(),
            NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&directory).unwrap();
        directory
    }

    #[test]
    fn inherited_files_are_merged_before_the_requesting_config() {
        let directory = temporary_directory();
        let parent = directory.join("parent.yml");
        let child = directory.join("child.yml");
        fs::write(
            &parent,
            "Layout:\n  Enabled: false\nStyle/StringLiterals:\n  EnforcedStyle: single_quotes\n",
        )
        .unwrap();
        fs::write(
            &child,
            "inherit_from: parent.yml\nStyle/StringLiterals:\n  EnforcedStyle: double_quotes\n",
        )
        .unwrap();

        let config = load(child.to_str().unwrap()).unwrap();
        assert_eq!(config.bool("Layout", "Enabled"), Some(false));
        assert_eq!(
            config.value("Style/StringLiterals", "EnforcedStyle"),
            Some("double_quotes")
        );
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn inherited_gem_paths_use_the_resolved_gem_root() {
        let directory = temporary_directory();
        let gem_root = directory.join("fake-gem");
        fs::create_dir_all(&gem_root).unwrap();
        fs::write(
            gem_root.join("rubocop.yml"),
            "Layout/LineLength:\n  Max: 99\n",
        )
        .unwrap();
        let child = directory.join("child.yml");
        fs::write(
            &child,
            "inherit_gem:\n  fake-gem: rubocop.yml\nLayout/LineLength:\n  Max: 120\n",
        )
        .unwrap();
        let mut stack = Vec::new();
        let mut layers = Vec::new();
        load_file(&child, &mut stack, &mut layers, &|name| {
            assert_eq!(name, "fake-gem");
            Ok(gem_root.clone())
        })
        .unwrap();
        let config = CopConfig::from_sources(
            layers
                .iter()
                .map(|layer| (layer.source.as_str(), layer.merge_keys.clone())),
            Some(directory.clone()),
        );
        assert_eq!(config.usize("Layout/LineLength", "Max"), Some(120));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn inherit_mode_merges_excludes_across_layers() {
        let directory = temporary_directory();
        let parent = directory.join("parent.yml");
        let child = directory.join("child.yml");
        fs::write(
            &parent,
            "inherit_mode:\n  merge:\n    - Exclude\nAllCops:\n  Exclude:\n    - data/**/*\n",
        )
        .unwrap();
        fs::write(
            &child,
            "inherit_from: parent.yml\nAllCops:\n  Exclude:\n    - services/**/*\n",
        )
        .unwrap();

        let config = load(child.to_str().unwrap()).unwrap();
        assert!(config
            .values("AllCops", "Exclude")
            .contains(&"data/**/*".to_string()));
        assert!(config
            .values("AllCops", "Exclude")
            .contains(&"services/**/*".to_string()));
        fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn inheritance_cycles_fail_visibly() {
        let directory = temporary_directory();
        let first = directory.join("first.yml");
        let second = directory.join("second.yml");
        fs::write(&first, "inherit_from: second.yml\n").unwrap();
        fs::write(&second, "inherit_from: first.yml\n").unwrap();

        let error = load(first.to_str().unwrap()).unwrap_err();
        assert!(error.starts_with("configuration inheritance cycle:"));
        fs::remove_dir_all(directory).unwrap();
    }
}
