use crate::config::CopConfig;

/// Common RuboCop configuration conventions shared by many cops.
pub(super) struct CopPolicy<'config> {
    config: &'config CopConfig,
    cop_name: &'static str,
}

impl<'config> CopPolicy<'config> {
    pub(super) fn new(config: &'config CopConfig, cop_name: &'static str) -> Self {
        Self { config, cop_name }
    }

    pub(super) fn enforced_style(&self, default: &'config str) -> &'config str {
        self.config
            .value(self.cop_name, "EnforcedStyle")
            .unwrap_or(default)
    }

    #[allow(dead_code)]
    pub(super) fn allows_method(&self, method: &[u8]) -> bool {
        self.allows_name("AllowedMethods", "AllowedPatterns", method)
    }

    #[allow(dead_code)]
    pub(super) fn allows_receiver(&self, receiver: &[u8]) -> bool {
        self.allows_name("AllowedReceivers", "AllowedReceiverPatterns", receiver)
    }

    #[allow(dead_code)]
    pub(super) fn excluded_path(&self, path: &str) -> bool {
        self.config
            .values("AllCops", "Exclude")
            .iter()
            .chain(self.config.values(self.cop_name, "Exclude"))
            .any(|pattern| glob_matches(pattern, path))
    }

    #[allow(dead_code)]
    pub(super) fn included_path(&self, path: &str) -> bool {
        let includes = self.config.values(self.cop_name, "Include");
        includes.is_empty() || includes.iter().any(|pattern| glob_matches(pattern, path))
    }

    fn allows_name(&self, names_key: &str, patterns_key: &str, name: &[u8]) -> bool {
        let Ok(name) = std::str::from_utf8(name) else {
            return false;
        };
        self.config
            .values(self.cop_name, names_key)
            .iter()
            .any(|allowed| allowed == name)
            || self
                .config
                .patterns(self.cop_name, patterns_key)
                .iter()
                .any(|pattern| pattern.is_match(name))
    }
}

fn glob_matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.as_bytes();
    let path = path.as_bytes();
    let mut previous = vec![false; path.len() + 1];
    previous[0] = true;
    for token in pattern {
        let mut current = vec![false; path.len() + 1];
        if *token == b'*' {
            current[0] = previous[0];
            for index in 1..=path.len() {
                current[index] = previous[index] || current[index - 1];
            }
        } else {
            for index in 1..=path.len() {
                current[index] =
                    previous[index - 1] && (*token == b'?' || *token == path[index - 1]);
            }
        }
        previous = current;
    }
    previous[path.len()]
}
