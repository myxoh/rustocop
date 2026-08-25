// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/registry_spec.rb
// Spec SHA-256: f4bf5ab3323cfd211489ca4200831412ce13f69acc89e01747a89a287817a427
// Registry examples also ported from:
// spec/rubocop/cop/cop_spec.rb
// Spec SHA-256: 7d9e9850ef3594e218419afa5bc1c83d49b7f91732d7fc868452bc9c0603c9ab

use std::collections::{BTreeSet, HashMap};

use super::registry::{
    CopDescriptor, CopRegistryConfig, EnabledValue, Registry, RegistryConfig, RegistryOptions,
};

fn cops() -> Vec<CopDescriptor> {
    [
        "Lint/BooleanSymbol",
        "Lint/DuplicateMethods",
        "Layout/FirstArrayElementIndentation",
        "Metrics/MethodLength",
        "RSpec/Foo",
        "Test/FirstArrayElementIndentation",
    ]
    .map(CopDescriptor::new)
    .to_vec()
}

#[test]
fn enrollment_filtering_lookup_lazy_loading_and_sorting_match_registry() {
    let mut registry = Registry::new(cops(), RegistryOptions::default());
    assert_eq!(registry.length(), 6);
    assert_eq!(
        registry.departments(),
        ["Lint", "Layout", "Metrics", "RSpec", "Test"]
    );
    assert!(registry.department("Lint"));
    assert!(registry.qualified_cop("Lint/BooleanSymbol"));
    assert!(registry.contains_cop_matching(&["Lint/BooleanSymbol".into()]));
    assert_eq!(
        registry.names_for_department("Lint"),
        ["Lint/BooleanSymbol", "Lint/DuplicateMethods"]
    );
    assert!(registry.find_by_cop_name("Lint/BooleanSymbol").is_some());
    assert!(registry.find_by_cop_name("Foo/Bar").is_none());
    assert_eq!(registry.with_department("Lint").length(), 2);
    assert_eq!(registry.without_department("Lint").length(), 4);

    registry.lazy_load("LazyLoad/Foo");
    assert_eq!(registry.length(), 7);
    assert!(registry.departments().contains(&"LazyLoad".to_owned()));
    assert!(registry.find_by_cop_name("LazyLoad/Foo").is_some());
    registry.sort();
    assert_eq!(registry.names()[0], "Lint/BooleanSymbol");
}

#[test]
fn temporary_global_and_initialize_copy_restore_independent_registry_state() {
    let mut registry = Registry::new(cops(), RegistryOptions::default());
    let mut copied = Registry::initialize_copy(&mut registry);
    assert_eq!(copied.names(), registry.names());

    let temporary = Registry::with(vec![CopDescriptor::new("Style/Temporary")]);
    let observed = registry.with_temporary_global(temporary, |active| active.names());
    assert_eq!(observed, ["Style/Temporary"]);
    assert_eq!(registry.length(), 6);
}

#[test]
fn qualification_warns_repairs_and_rejects_ambiguity() {
    let mut registry = Registry::new(cops(), RegistryOptions::default());
    assert_eq!(
        registry
            .qualified_cop_name("MethodLength", "/app/.rubocop.yml", true)
            .unwrap(),
        "Metrics/MethodLength"
    );
    assert!(registry.warnings("/app/.rubocop.yml").is_some());
    assert_eq!(
        registry
            .qualified_cop_name("Style/MethodLength", "config.yml", true)
            .unwrap(),
        "Metrics/MethodLength"
    );
    assert_eq!(
        registry
            .qualified_cop_name("NotReal", "config.yml", true)
            .unwrap(),
        "NotReal"
    );

    let mut ambiguous = Registry::new(
        ["Test/Same", "Test/Foo/Same", "Test/Bar/Same"]
            .map(CopDescriptor::new)
            .to_vec(),
        RegistryOptions::default(),
    );
    let error = ambiguous
        .qualified_cop_name("Same", "config.yml", true)
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("Did you mean Test/Same or Test/Foo/Same or Test/Bar/Same?"));
}

#[test]
fn enabled_safe_pending_and_only_precedence_match_rubocop() {
    let config = RegistryConfig::new(
        HashMap::from([
            (
                "Lint/BooleanSymbol".into(),
                CopRegistryConfig {
                    enabled: EnabledValue::Pending,
                    safe: true,
                },
            ),
            (
                "RSpec/Foo".into(),
                CopRegistryConfig {
                    enabled: EnabledValue::Enabled,
                    safe: false,
                },
            ),
            (
                "Test/FirstArrayElementIndentation".into(),
                CopRegistryConfig {
                    enabled: EnabledValue::Disabled,
                    safe: true,
                },
            ),
        ]),
        false,
    );
    let mut registry = Registry::new(
        cops(),
        RegistryOptions {
            safe: true,
            ..RegistryOptions::default()
        },
    );
    let enabled: Vec<_> = registry
        .enabled(&config)
        .iter()
        .map(CopDescriptor::cop_name)
        .collect();
    assert!(!enabled.contains(&"Lint/BooleanSymbol".to_owned()));
    assert!(!enabled.contains(&"RSpec/Foo".to_owned()));

    let mut registry = Registry::new(
        cops(),
        RegistryOptions {
            enable_pending_cops: true,
            only: BTreeSet::from(["Test/FirstArrayElementIndentation".into()]),
            ..RegistryOptions::default()
        },
    );
    let enabled: Vec<_> = registry
        .enabled(&config)
        .iter()
        .map(CopDescriptor::cop_name)
        .collect();
    assert!(enabled.contains(&"Lint/BooleanSymbol".to_owned()));
    assert!(enabled.contains(&"Test/FirstArrayElementIndentation".to_owned()));
    assert_eq!(registry.disabled(&config).len() + enabled.len(), 6);
}

#[test]
fn rapid_dismissal_and_directive_lookup_match_queue_semantics() {
    let mut registry = Registry::new(cops(), RegistryOptions::default());
    let added = CopDescriptor::new("Metrics/AbcSize");
    registry.enlist(added.clone());
    registry.dismiss(&added).unwrap();
    assert!(!registry.names().contains(&"Metrics/AbcSize".to_owned()));
    registry.enlist(added.clone());
    let _ = registry.cops();
    assert!(registry.dismiss(&added).is_err());
    assert_eq!(registry.find_cops_by_directive("Lint").len(), 2);
    assert_eq!(
        registry.find_cops_by_directive("Metrics/AbcSize"),
        vec![added]
    );
}
