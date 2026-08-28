use std::path::Path;

use glob::{MatchOptions, Pattern};

use std::collections::HashMap;

use super::{ConfigValue, CopConfig};

#[derive(Clone, Debug)]
pub(super) struct PathGlob {
    patterns: Vec<Pattern>,
}

impl PathGlob {
    fn compile(pattern: &str) -> Option<Self> {
        let patterns = expand_braces(pattern)
            .into_iter()
            .filter_map(|pattern| {
                let pattern = translate_ruby_escapes(&pattern);
                Pattern::new(&normalize_negated_classes(&pattern)).ok()
            })
            .collect::<Vec<_>>();
        (!patterns.is_empty()).then_some(Self { patterns })
    }

    fn is_match(&self, path: &str) -> bool {
        let ordinary = MatchOptions {
            case_sensitive: true,
            require_literal_separator: true,
            require_literal_leading_dot: true,
        };
        if self
            .patterns
            .iter()
            .any(|pattern| pattern.matches_with(path, ordinary))
        {
            return true;
        }

        // RuboCop retries with FNM_DOTMATCH only for a hidden basename whose
        // parent directories are not hidden.
        if !hidden_file_in_not_hidden_dir(path) {
            return false;
        }
        let dotmatch = MatchOptions {
            require_literal_leading_dot: false,
            ..ordinary
        };
        self.patterns
            .iter()
            .any(|pattern| pattern.matches_with(path, dotmatch))
    }
}

impl CopConfig {
    pub(crate) fn cop_applies_to_path(&self, cop: &str, path: &str) -> bool {
        if self.path_excluded(cop, path) {
            return false;
        }
        !self.contains(cop, "Include") || self.section_path_matches(cop, "Include", path)
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
) -> HashMap<String, HashMap<String, Vec<PathGlob>>> {
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
                                PathGlob::compile(pattern.trim_start_matches("./"))
                            })
                            .collect(),
                    ))
                })
                .collect::<HashMap<_, _>>();
            (!compiled.is_empty()).then(|| (section.clone(), compiled))
        })
        .collect()
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
    PathGlob::compile(pattern).is_some_and(|pattern| pattern.is_match(path))
}

fn expand_braces(pattern: &str) -> Vec<String> {
    let mut escaped = false;
    let mut in_class = false;
    let opening = pattern.char_indices().find_map(|(offset, character)| {
        if escaped {
            escaped = false;
            return None;
        }
        match character {
            '\\' => escaped = true,
            '[' => in_class = true,
            ']' => in_class = false,
            '{' if !in_class => return Some(offset),
            _ => {}
        }
        None
    });
    let Some(opening) = opening else {
        return vec![pattern.to_owned()];
    };
    let mut depth = 0usize;
    let mut closing = None;
    let mut escaped = false;
    let mut in_class = false;
    for (offset, character) in pattern[opening..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '[' => in_class = true,
            ']' => in_class = false,
            '{' if !in_class => depth += 1,
            '}' if !in_class => {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    closing = Some(opening + offset);
                    break;
                }
            }
            _ => {}
        }
    }
    let Some(closing) = closing else {
        return vec![pattern.to_owned()];
    };
    let body = &pattern[opening + 1..closing];
    let mut alternatives = Vec::new();
    let mut depth = 0usize;
    let mut start = 0usize;
    let mut escaped = false;
    let mut in_class = false;
    for (offset, character) in body.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        match character {
            '\\' => escaped = true,
            '[' => in_class = true,
            ']' => in_class = false,
            '{' if !in_class => depth += 1,
            '}' if !in_class => depth = depth.saturating_sub(1),
            ',' if depth == 0 && !in_class => {
                alternatives.push(&body[start..offset]);
                start = offset + 1;
            }
            _ => {}
        }
    }
    if alternatives.is_empty() {
        return vec![pattern.to_owned()];
    }
    alternatives.push(&body[start..]);
    alternatives
        .into_iter()
        .flat_map(|alternative| {
            expand_braces(&format!(
                "{}{}{}",
                &pattern[..opening],
                alternative,
                &pattern[closing + 1..]
            ))
        })
        .collect()
}

fn translate_ruby_escapes(pattern: &str) -> String {
    let mut translated = String::with_capacity(pattern.len());
    let mut characters = pattern.chars();
    while let Some(character) = characters.next() {
        if character != '\\' {
            translated.push(character);
            continue;
        }
        let Some(escaped) = characters.next() else {
            translated.push('\\');
            break;
        };
        translated.push_str(&Pattern::escape(&escaped.to_string()));
    }
    translated
}

fn normalize_negated_classes(pattern: &str) -> String {
    pattern.replace("[^", "[!")
}

fn hidden_file_in_not_hidden_dir(path: &str) -> bool {
    let components = path.split('/').filter(|component| !component.is_empty());
    let components = components.collect::<Vec<_>>();
    components
        .last()
        .is_some_and(|basename| basename.starts_with('.'))
        && components[..components.len().saturating_sub(1)]
            .iter()
            .all(|component| !component.starts_with('.'))
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
    fn rubocop_globs_support_positive_and_negated_character_classes() {
        assert!(glob_matches("**/[Gg]emfile", "Gemfile"));
        assert!(glob_matches("**/[Gg]emfile", "config/gemfile"));
        assert!(glob_matches("**/[!a]emfile", "Gemfile"));
        assert!(glob_matches("**/[^a]emfile", "Gemfile"));
        assert!(!glob_matches("**/[!G]emfile", "Gemfile"));
        assert!(!glob_matches("**/[Gg]emfile", "config/aemfile"));
    }

    #[test]
    fn rubocop_globs_preserve_backslash_escapes() {
        assert!(glob_matches("**/Gem\\file", "Gemfile"));
        assert!(!glob_matches("**/Gem\\file", "Gem/file"));
        assert!(glob_matches("**/Gem\\*file", "Gem*file"));
        assert!(!glob_matches("**/Gem\\*file", "Gem-any-file"));
        assert!(glob_matches(
            "**/\\{Gemfile,Gems.rb\\}",
            "{Gemfile,Gems.rb}"
        ));

        let config =
            CopConfig::from_source("Bundler/DuplicatedGem:\n  Include:\n    - '**/Gem\\file'\n");
        assert!(config.cop_applies_to_path("Bundler/DuplicatedGem", "Gemfile"));

        let resolved = CopConfig::from_resolved_source(
            "Bundler/DuplicatedGem:\n  Include:\n    - \"**/Gem\\\\file\"\n",
            None,
        );
        assert!(resolved.cop_applies_to_path("Bundler/DuplicatedGem", "Gemfile"));
    }

    #[test]
    fn rubocop_globs_match_hidden_files_but_not_hidden_directories() {
        assert!(glob_matches("**/*", "lib/.rubocop.yml"));
        assert!(!glob_matches("**/*", ".config/rubocop.yml"));
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
    fn cop_include_policy_supports_rubocop_brace_globs() {
        let config =
            CopConfig::from_source("Bundler/DuplicatedGem:\n  Include:\n    - '**/*.{rb,rake}'\n");

        assert!(config.cop_applies_to_path("Bundler/DuplicatedGem", "lib/Gemfile.rb"));
        assert!(config.cop_applies_to_path("Bundler/DuplicatedGem", "tasks/deps.rake"));
        assert!(!config.cop_applies_to_path("Bundler/DuplicatedGem", "Gemfile"));
    }

    #[test]
    fn explicitly_empty_cop_include_matches_no_paths() {
        let config = CopConfig::from_source("Bundler/DuplicatedGem:\n  Include: []\n");

        assert!(!config.cop_applies_to_path("Bundler/DuplicatedGem", "Gemfile"));
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
