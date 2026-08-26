use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use serde::Deserialize;

#[derive(Deserialize)]
struct Manifest {
    format_version: usize,
    updated_at: String,
    rubocop_version: String,
    rubocop_ast_version: String,
    components: Vec<Component>,
}

#[derive(Deserialize)]
struct Component {
    package: String,
    source: String,
    source_sha256: String,
    kind: String,
    api: Vec<String>,
    rust: Option<String>,
    status: String,
    api_coverage: Option<ApiCoverage>,
    evidence: Option<String>,
    deviations: Vec<String>,
    specs: Vec<TranslationSpec>,
}

#[derive(Deserialize)]
struct ApiCoverage {
    total: usize,
    direct: usize,
    equivalent: usize,
    unresolved: Vec<String>,
    unexercised_targets: BTreeMap<String, String>,
}

#[derive(Deserialize)]
struct TranslationSpec {
    package: String,
    source: String,
    source_sha256: String,
    rust: String,
    status: String,
    deviations: Vec<String>,
    upstream_examples: usize,
    covered_upstream_examples: usize,
    rust_tests: usize,
    coverage_inventory: String,
    coverage_rust_files: Vec<String>,
    contract_tests: Vec<ContractTests>,
    example_contracts: Vec<ExampleContract>,
    contract_sha256: String,
}

#[derive(Deserialize)]
struct ContractTests {
    rust: String,
    tests: Vec<String>,
}

#[derive(Deserialize)]
struct ExampleContract {
    rspec_id: String,
    description_sha256: String,
    rust: String,
    test: String,
    mapping_basis: String,
    matched_terms: Vec<String>,
}

fn verify_contract_tests(crate_root: &Path, spec: &TranslationSpec) {
    let declared_test_count: usize = spec
        .contract_tests
        .iter()
        .map(|contract| contract.tests.len())
        .sum();
    assert_eq!(declared_test_count, spec.rust_tests);
    for contract in &spec.contract_tests {
        assert!(spec.coverage_rust_files.contains(&contract.rust));
        let source = fs::read_to_string(crate_root.join(&contract.rust)).unwrap();
        for test in &contract.tests {
            assert!(
                source.contains(&format!("fn {test}")),
                "missing executable contract {}#{test}",
                contract.rust
            );
        }
    }
}

fn verify_example_contracts(crate_root: &Path, spec: &TranslationSpec) {
    for contract in &spec.example_contracts {
        assert!(contract.rspec_id.starts_with('['));
        assert_eq!(contract.description_sha256.len(), 64);
        assert!(spec.coverage_rust_files.contains(&contract.rust));
        assert!(matches!(
            contract.mapping_basis.as_str(),
            "semantic_terms" | "explicit_source_rule"
        ));
        if contract.mapping_basis == "semantic_terms" {
            assert!(!contract.matched_terms.is_empty());
        }
        let source = fs::read_to_string(crate_root.join(&contract.rust)).unwrap();
        assert!(
            source.contains(&format!("fn {}", contract.test)),
            "missing example contract {}#{} for {}",
            contract.rust,
            contract.test,
            contract.rspec_id
        );
    }
}

fn verify_translation_spec(crate_root: &Path, spec: &TranslationSpec) {
    assert!(matches!(spec.package.as_str(), "rubocop" | "rubocop-ast"));
    assert!(matches!(spec.status.as_str(), "partial" | "translated"));
    assert!(spec.rust_tests > 0);
    assert!(spec.upstream_examples > 0);
    assert_eq!(spec.covered_upstream_examples, spec.upstream_examples);
    assert_eq!(
        spec.coverage_inventory,
        "spec/upstream/rubocop-compatibility-examples.json"
    );
    assert!(!spec.coverage_rust_files.is_empty());
    assert!(!spec.contract_tests.is_empty());
    assert_eq!(spec.example_contracts.len(), spec.upstream_examples);
    assert_eq!(spec.contract_sha256.len(), 64);
    assert_eq!(spec.source_sha256.len(), 64);
    assert!(spec.source.starts_with("spec/rubocop/"));
    let rust = fs::read_to_string(crate_root.join(&spec.rust)).unwrap();
    assert!(rust.contains(&spec.source));
    assert!(rust.contains(&format!("Spec SHA-256: {}", spec.source_sha256)));
    verify_contract_tests(crate_root, spec);
    verify_example_contracts(crate_root, spec);
    for deviation in &spec.deviations {
        assert!(!deviation.trim().is_empty());
    }
}

#[test]
#[allow(clippy::cognitive_complexity)] // One audit reports all manifest invariants together.
fn translation_manifest_is_traceable_and_complete_for_registered_files() {
    let crate_root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let manifest_path = crate_root.join("rubocop-translation.json");
    let manifest: Manifest =
        serde_json::from_str(&fs::read_to_string(manifest_path).unwrap()).unwrap();

    assert_eq!(manifest.format_version, 5);
    assert!(manifest.updated_at.ends_with('Z'));
    assert_eq!(manifest.rubocop_version, "1.87.0");
    assert_eq!(manifest.rubocop_ast_version, "1.49.1");
    assert_eq!(manifest.components.len(), 228);

    for translation in manifest.components {
        assert!(matches!(
            translation.package.as_str(),
            "rubocop" | "rubocop-ast"
        ));
        assert!(matches!(
            translation.status.as_str(),
            "pending" | "partial" | "translated" | "native" | "not_applicable"
        ));
        assert_eq!(translation.source_sha256.len(), 64);
        assert!(translation.source.starts_with("lib/rubocop/"));
        assert!(matches!(
            translation.kind.as_str(),
            "ast" | "cop_mixin" | "corrector" | "legacy" | "cop_framework"
        ));
        let mut sorted_api = translation.api.clone();
        sorted_api.sort();
        sorted_api.dedup();
        assert_eq!(translation.api, sorted_api);
        if translation.status != "pending" {
            assert!(
                translation
                    .evidence
                    .as_deref()
                    .is_some_and(|value| !value.trim().is_empty()),
                "{} needs evidence",
                translation.source
            );
        }
        if matches!(
            translation.status.as_str(),
            "partial" | "translated" | "native"
        ) {
            let rust_path = translation
                .rust
                .as_deref()
                .expect("implemented component needs Rust path");
            let rust = fs::read_to_string(crate_root.join(rust_path)).unwrap();
            if matches!(translation.status.as_str(), "partial" | "translated") {
                assert!(rust.contains(&format!("Source: {}", translation.source)));
                assert!(rust.contains(&format!("Source SHA-256: {}", translation.source_sha256)));
            }
        }
        if matches!(translation.status.as_str(), "translated" | "native") {
            let coverage = translation
                .api_coverage
                .as_ref()
                .expect("implemented component needs API coverage");
            assert_eq!(coverage.direct + coverage.equivalent, coverage.total);
            assert!(coverage.unresolved.is_empty(), "{}", translation.source);
            assert!(
                coverage.unexercised_targets.is_empty(),
                "{}",
                translation.source
            );
        }
        for deviation in translation.deviations {
            assert!(!deviation.trim().is_empty());
        }
        for spec in &translation.specs {
            verify_translation_spec(crate_root, spec);
        }
    }
}
