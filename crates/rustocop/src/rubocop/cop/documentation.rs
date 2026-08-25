// RuboCop 1.87.0
// Source: lib/rubocop/cop/documentation.rb
// Source SHA-256: 75c1ded0be5736d5c121ecbad24a0261185476fde0f34acdcf4b96879c8f279a

use std::collections::HashMap;
use std::path::Path;

pub(crate) type DepartmentConfig = HashMap<String, HashMap<String, String>>;

pub(crate) fn department_to_basename(department: &str) -> String {
    format!("cops_{}", department.to_lowercase().replace('/', "_"))
}

pub(crate) fn url_for(
    department: &str,
    cop_name: &str,
    builtin: bool,
    config: Option<&DepartmentConfig>,
) -> Option<String> {
    let base_url = base_url_for(department, builtin, config)?;
    let extension = extension_for(department, config);
    let fragment: String = cop_name
        .chars()
        .filter(char::is_ascii_alphabetic)
        .flat_map(char::to_lowercase)
        .collect();
    Some(format!(
        "{base_url}/{}{extension}#{fragment}",
        department_to_basename(department)
    ))
}

pub(crate) fn base_url_for(
    department: &str,
    builtin: bool,
    config: Option<&DepartmentConfig>,
) -> Option<String> {
    if let Some(url) = config
        .and_then(|config| config.get(department))
        .and_then(|department| department.get("DocumentationBaseURL"))
    {
        return Some(url.clone());
    }
    builtin.then(|| default_base_url().to_owned())
}

pub(crate) fn extension_for(department: &str, config: Option<&DepartmentConfig>) -> String {
    config
        .and_then(|config| config.get(department))
        .and_then(|department| department.get("DocumentationExtension"))
        .cloned()
        .unwrap_or_else(|| default_extension().to_owned())
}

pub(crate) const fn default_base_url() -> &'static str {
    "https://docs.rubocop.org/rubocop"
}

pub(crate) const fn default_extension() -> &'static str {
    ".html"
}

pub(crate) fn builtin(source_path: Option<&Path>, builtin_directory: &Path) -> bool {
    source_path.is_some_and(|path| path.starts_with(builtin_directory))
}
