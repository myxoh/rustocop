// RuboCop 1.87.0
// Source: lib/rubocop/cop/badge.rb
// Source SHA-256: cc863d9f9887d32277a389be61c5bb32cbb95a83bc02eddbb819b5e99d01fee2

use std::fmt;
use std::hash::{DefaultHasher, Hash, Hasher};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub(crate) struct Badge {
    department: Option<String>,
    cop_name: String,
}

impl Badge {
    pub(crate) fn for_class(class_name: &str) -> Self {
        let parts: Vec<_> = class_name.split("::").collect();
        if parts.len() >= 4 {
            Self::new(&parts[2..])
        } else {
            Self::new(&parts[parts.len().saturating_sub(2)..])
        }
    }

    pub(crate) fn parse(identifier: &str) -> Self {
        let parts: Vec<_> = identifier.split('/').map(Self::camel_case).collect();
        let borrowed: Vec<_> = parts.iter().map(String::as_str).collect();
        Self::new(&borrowed)
    }

    pub(crate) fn camel_case(name_part: &str) -> String {
        if name_part == "rspec" {
            return "RSpec".to_owned();
        }
        let mut uppercase_next = true;
        let mut result = String::with_capacity(name_part.len());
        for character in name_part.chars() {
            if character == '_' {
                uppercase_next = true;
            } else if uppercase_next && character.is_ascii_lowercase() {
                result.push(character.to_ascii_uppercase());
                uppercase_next = false;
            } else {
                result.push(character);
                uppercase_next = false;
            }
        }
        result
    }

    pub(crate) fn new(class_name_parts: &[&str]) -> Self {
        let (cop_name, department_parts) = class_name_parts
            .split_last()
            .map_or(("", &[][..]), |(last, rest)| (*last, rest));
        Self {
            department: (!department_parts.is_empty()).then(|| department_parts.join("/")),
            cop_name: cop_name.to_owned(),
        }
    }

    pub(crate) fn department(&self) -> Option<&str> {
        self.department.as_deref()
    }

    pub(crate) fn department_name(&self) -> Option<&str> {
        self.department()
    }

    pub(crate) fn cop_name(&self) -> &str {
        &self.cop_name
    }

    pub(crate) fn matches(&self, other: &Self) -> bool {
        self.cop_name == other.cop_name
            && (!self.qualified() || self.department == other.department)
    }

    /// RuboCop defines Badge equality in terms of its manually combined hash.
    pub(crate) fn equivalent(&self, other: &Self) -> bool {
        self.hash_value() == other.hash_value()
    }

    pub(crate) fn hash_value(&self) -> u64 {
        let mut department = DefaultHasher::new();
        self.department.hash(&mut department);
        let mut cop_name = DefaultHasher::new();
        self.cop_name.hash(&mut cop_name);
        department.finish() ^ cop_name.finish()
    }

    pub(crate) fn display(&self) -> String {
        self.to_string()
    }

    pub(crate) fn qualified(&self) -> bool {
        self.department.is_some()
    }

    pub(crate) fn with_department(&self, department: &str) -> Self {
        Self {
            department: Some(department.to_owned()),
            cop_name: self.cop_name.clone(),
        }
    }
}

impl fmt::Display for Badge {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(department) = &self.department {
            write!(formatter, "{department}/{}", self.cop_name)
        } else {
            formatter.write_str(&self.cop_name)
        }
    }
}
