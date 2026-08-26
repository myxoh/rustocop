use std::collections::HashMap;

use std::path::Path;

use super::documentation::{builtin, department_to_basename, url_for, DepartmentConfig};

#[test]
fn builds_builtin_and_nested_department_urls() {
    assert_eq!(department_to_basename("RSpec/Rails"), "cops_rspec_rails");
    assert_eq!(
        url_for("Layout", "Layout/BlockEndNewline", true, None).as_deref(),
        Some("https://docs.rubocop.org/rubocop/cops_layout.html#layoutblockendnewline")
    );
    assert_eq!(url_for("Some", "Some/Cop", false, None), None);
    assert!(builtin(
        Some(Path::new("/gem/lib/rubocop/cop/layout/example.rb")),
        Path::new("/gem/lib/rubocop/cop")
    ));
    assert!(!builtin(
        Some(Path::new("/plugin/lib/example.rb")),
        Path::new("/gem/lib/rubocop/cop")
    ));
}

#[test]
fn department_configuration_overrides_base_and_extension() {
    let mut config = DepartmentConfig::new();
    config.insert(
        "Sorbet".into(),
        HashMap::from([
            (
                "DocumentationBaseURL".into(),
                "https://github.com/Shopify/rubocop-sorbet/blob/main/manual".into(),
            ),
            ("DocumentationExtension".into(), ".md".into()),
        ]),
    );
    assert_eq!(
        url_for("Sorbet", "Sorbet/FalseSigil", false, Some(&config)).as_deref(),
        Some("https://github.com/Shopify/rubocop-sorbet/blob/main/manual/cops_sorbet.md#sorbetfalsesigil")
    );
}
