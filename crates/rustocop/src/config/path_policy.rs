use std::path::Path;

use regex::Regex;

use std::collections::HashMap;

use super::{ConfigValue, CopConfig};

impl CopConfig {
    pub(crate) fn cop_applies_to_path(&self, cop: &str, path: &str) -> bool {
        if self.path_excluded(cop, path) {
            return false;
        }
        self.values(cop, "Include").is_empty() || self.section_path_matches(cop, "Include", path)
    }

    pub(crate) fn all_cops_excluded(&self, path: &str) -> bool {
        self.section_path_matches("AllCops", "Exclude", path)
    }

    pub(crate) fn target_included(&self, path: &Path) -> bool {
        let path = path.to_string_lossy();
        self.section_path_matches("AllCops", "Include", &path)
    }

    fn path_excluded(&self, cop: &str, path: &str) -> bool {
        let department = cop.split_once('/').map(|(department, _)| department);
        self.section_path_matches("AllCops", "Exclude", path)
            || department
                .is_some_and(|department| self.section_path_matches(department, "Exclude", path))
            || self.section_path_matches(cop, "Exclude", path)
    }

    fn section_path_matches(&self, section: &str, key: &str, path: &str) -> bool {
        let Some(patterns) = self
            .path_globs
            .get(section)
            .and_then(|entries| entries.get(key))
        else {
            return false;
        };
        let supplied = Path::new(path);
        let rooted = if supplied.is_absolute() {
            supplied.to_path_buf()
        } else if let Some(root) = &self.root {
            root.join(supplied)
        } else {
            supplied.to_path_buf()
        };
        let absolute = normalized_path(&rooted);
        let relative = self.root.as_deref().and_then(|root| {
            let root = normalized_path(root);
            if absolute == root {
                Some(String::new())
            } else {
                absolute
                    .strip_prefix(&format!("{root}/"))
                    .map(str::to_string)
            }
        });
        patterns.iter().any(|pattern| {
            pattern.is_match(&absolute)
                || relative
                    .as_deref()
                    .is_some_and(|path| pattern.is_match(path))
        })
    }
}

pub(super) fn compile_path_globs(
    values: &HashMap<String, HashMap<String, ConfigValue>>,
) -> HashMap<String, HashMap<String, Vec<Regex>>> {
    values
        .iter()
        .filter_map(|(section, entries)| {
            let compiled = entries
                .iter()
                .filter(|(key, _)| matches!(key.as_str(), "Include" | "Exclude"))
                .filter_map(|(key, value)| {
                    let ConfigValue::List(patterns) = value else {
                        return None;
                    };
                    Some((
                        key.clone(),
                        patterns
                            .iter()
                            .filter_map(|pattern| {
                                Regex::new(&glob_regex(&normalized(
                                    pattern.trim_start_matches("./"),
                                )))
                                .ok()
                            })
                            .collect(),
                    ))
                })
                .collect::<HashMap<_, _>>();
            (!compiled.is_empty()).then(|| (section.clone(), compiled))
        })
        .collect()
}

fn normalized(path: &str) -> String {
    path.replace('\\', "/")
}

fn normalized_path(path: &Path) -> String {
    use std::path::Component;

    let mut components = Vec::new();
    let mut rooted = false;
    for component in path.components() {
        match component {
            Component::Prefix(prefix) => components.push(prefix.as_os_str().to_string_lossy()),
            Component::RootDir => rooted = true,
            Component::CurDir => {}
            Component::ParentDir => {
                components.pop();
            }
            Component::Normal(value) => components.push(value.to_string_lossy()),
        }
    }
    let body = components.join("/");
    if rooted {
        format!("/{body}")
    } else {
        body
    }
}

#[cfg(test)]
pub(super) fn glob_matches(pattern: &str, path: &str) -> bool {
    let Ok(pattern) = Regex::new(&glob_regex(pattern)) else {
        return false;
    };
    pattern.is_match(path)
}

fn glob_regex(pattern: &str) -> String {
    let bytes = pattern.as_bytes();
    let mut regex = String::from("^");
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'*' if bytes.get(index + 1) == Some(&b'*') => {
                index += 2;
                if bytes.get(index) == Some(&b'/') {
                    regex.push_str("(?:.*/)?");
                    index += 1;
                } else {
                    regex.push_str(".*");
                }
                continue;
            }
            b'*' => regex.push_str("[^/]*"),
            b'?' => regex.push_str("[^/]"),
            b'{' => {
                if let Some(end) = pattern[index + 1..].find('}') {
                    let alternatives = &pattern[index + 1..index + 1 + end];
                    regex.push_str("(?:");
                    regex.push_str(
                        &alternatives
                            .split(',')
                            .map(regex::escape)
                            .collect::<Vec<_>>()
                            .join("|"),
                    );
                    regex.push(')');
                    index += end + 2;
                    continue;
                }
                regex.push_str("\\{");
            }
            byte => regex.push_str(&regex::escape(&(byte as char).to_string())),
        }
        index += 1;
    }
    regex.push('$');
    regex
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rubocop_globs_distinguish_single_and_double_stars() {
        assert!(glob_matches("app/**/*.rb", "app/models/user.rb"));
        assert!(glob_matches("app/**/*.rb", "app/user.rb"));
        assert!(!glob_matches("app/*.rb", "app/models/user.rb"));
        assert!(glob_matches("**/*.{rb,rake}", "lib/tasks/example.rake"));
    }

    #[test]
    fn applies_global_and_cop_path_policy_relative_to_the_config_root() {
        let mut config = CopConfig::from_source(
            "AllCops:\n  Exclude:\n    - vendor/**/*\nStyle/StringLiterals:\n  Include:\n    - app/**/*\n",
        );
        config.root = Some(std::path::PathBuf::from("/project"));

        assert!(config.cop_applies_to_path("Style/StringLiterals", "/project/app/model.rb"));
        assert!(!config.cop_applies_to_path("Style/StringLiterals", "/project/db/schema.rb"));
        assert!(!config.cop_applies_to_path("Style/StringLiterals", "/project/vendor/example.rb"));
    }

    #[test]
    fn absolute_compiled_excludes_match_dot_prefixed_discovery_paths() {
        let mut config =
            CopConfig::from_source("AllCops:\n  Exclude:\n    - /project/Dangerfile\n");
        config.root = Some(std::path::PathBuf::from("/project"));

        assert!(config.all_cops_excluded("./Dangerfile"));
        assert!(config.all_cops_excluded("/project/./Dangerfile"));
    }
}
