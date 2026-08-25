// RuboCop 1.87.0
// Source: lib/rubocop/cop/exclude_limit.rb
// Source SHA-256: 6eee8b278fc9b68a3487c8180fc5fd39e0914ab589325c491aacf639601f76cd

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ExcludeLimit {
    tmp_dir: Option<PathBuf>,
}

impl ExcludeLimit {
    pub(crate) fn new(tmp_dir: Option<PathBuf>) -> Self {
        Self { tmp_dir }
    }

    pub(crate) fn cop_dir_for(&self, cop_name: &str) -> Option<PathBuf> {
        self.tmp_dir
            .as_ref()
            .map(|directory| directory.join(cop_name.replace('/', "-")))
    }

    pub(crate) fn read_limits(&self, cop_name: &str) -> io::Result<BTreeMap<String, i64>> {
        let Some(cop_dir) = self.cop_dir_for(cop_name) else {
            return Ok(BTreeMap::new());
        };
        if !cop_dir.is_dir() {
            return Ok(BTreeMap::new());
        }
        let mut limits = BTreeMap::new();
        for entry in fs::read_dir(cop_dir)? {
            let path = entry?.path();
            if !path.is_file() {
                continue;
            }
            let maximum = fs::read_to_string(&path)?
                .lines()
                .filter_map(|line| line.parse::<i64>().ok())
                .max();
            if let (Some(name), Some(maximum)) = (path.file_name(), maximum) {
                limits.insert(name.to_string_lossy().into_owned(), maximum);
            }
        }
        Ok(limits)
    }

    pub(crate) fn record(
        &self,
        cop_name: &str,
        parameter_name: &str,
        value: i64,
    ) -> io::Result<()> {
        let Some(cop_dir) = self.cop_dir_for(cop_name) else {
            return Ok(());
        };
        fs::create_dir_all(&cop_dir)?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(cop_dir.join(parameter_name))?;
        writeln!(file, "{value}")
    }

    pub(crate) fn transform(parameter_name: &str) -> String {
        let mut output = String::with_capacity(parameter_name.len());
        for (index, character) in parameter_name.chars().enumerate() {
            if index > 0 && character.is_ascii_uppercase() {
                output.push('_');
            }
            output.extend(character.to_lowercase());
        }
        output
    }

    pub(crate) fn tmp_dir(&self) -> Option<&Path> {
        self.tmp_dir.as_deref()
    }
}
