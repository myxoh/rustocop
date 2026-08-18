use std::fs;
use std::path::PathBuf;

use super::InspectionPlan;
use crate::config::{CopSelection, InspectionConfig, RubyVersion};
use crate::model::Offense;

fn run_fixture(
    fixture: &str,
    inspected_path: &str,
    cops: &str,
    autocorrect: bool,
    ruby_version: RubyVersion,
) {
    let directory = fixture_directory(fixture);
    let source = fs::read_to_string(directory.join("input.rb")).unwrap();
    let expected_offenses = fs::read_to_string(directory.join("offenses.tsv")).unwrap();
    let options = InspectionConfig {
        autocorrect,
        cops: CopSelection::only(cops),
        target_ruby_version: ruby_version,
    };
    let plan = InspectionPlan::new(&options);

    let (offenses, corrected_source) = plan.inspect_content(inspected_path, &source, &options);

    assert_eq!(offense_snapshot(&offenses), expected_offenses);
    let corrected_fixture = directory.join("corrected.rb");
    let expected_source = if corrected_fixture.exists() {
        fs::read_to_string(corrected_fixture).unwrap()
    } else {
        source
    };
    assert_eq!(corrected_source, expected_source);
}

fn fixture_directory(fixture: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/inspection")
        .join(fixture)
}

fn offense_snapshot(offenses: &[Offense]) -> String {
    let mut snapshot = String::from(
        "cop\tline\tcolumn\tlast_line\tlast_column\tcorrectable\tcorrected\tmessage\n",
    );
    for offense in offenses {
        snapshot.push_str(&format!(
            "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            offense.cop_name,
            offense.line,
            offense.column,
            offense.last_line,
            offense.last_column,
            offense.correctable,
            offense.corrected,
            offense.message
        ));
    }
    snapshot
}

macro_rules! fixture_test {
    ($name:ident, $fixture:literal, $path:literal, $cops:literal, $autocorrect:literal, $version:expr) => {
        #[test]
        fn $name() {
            run_fixture($fixture, $path, $cops, $autocorrect, $version);
        }
    };
}

fixture_test!(
    reports_security_calls,
    "security_offenses",
    "/project/security.rb",
    "Security/Eval,Security/JSONLoad,Security/MarshalLoad",
    false,
    RubyVersion::default()
);
fixture_test!(
    ignores_safe_security_calls,
    "security_clean",
    "/project/security.rb",
    "Security/Eval,Security/JSONLoad,Security/MarshalLoad,Security/Open,Security/IoMethods",
    false,
    RubyVersion::default()
);
fixture_test!(
    orders_text_and_prism_offenses_by_location,
    "mixed_ordering",
    "/project/mixed.rb",
    "Layout/ExtraSpacing,Security/Eval,Security/JSONLoad",
    false,
    RubyVersion::default()
);
fixture_test!(
    applies_prism_corrections_from_one_parse,
    "prism_autocorrect",
    "/project/correctable.rb",
    "Security/JSONLoad,Security/IoMethods,Style/ArrayFirstLast,Style/RedundantArrayFlatten",
    true,
    RubyVersion::default()
);
fixture_test!(
    applies_path_sensitive_gemfile_correction,
    "bundler_autocorrect",
    "/project/Gemfile",
    "Bundler/OrderedGems",
    true,
    RubyVersion::default()
);
fixture_test!(
    reports_yaml_load_for_ruby_30,
    "yaml_load",
    "/project/config.rb",
    "Security/YAMLLoad",
    false,
    RubyVersion::new(3, 0)
);
fixture_test!(
    accepts_yaml_load_for_ruby_31,
    "yaml_load_ruby_31",
    "/project/config.rb",
    "Security/YAMLLoad",
    false,
    RubyVersion::new(3, 1)
);
fixture_test!(
    reports_utf8_columns_in_characters,
    "utf8_position",
    "/project/utf8.rb",
    "Security/Eval",
    false,
    RubyVersion::default()
);
fixture_test!(
    applies_rails_path_context,
    "rails_job",
    "/project/app/jobs/sync_job.rb",
    "Rails/ApplicationJob",
    false,
    RubyVersion::default()
);
fixture_test!(
    applies_rspec_path_context,
    "rspec_focus",
    "/project/spec/models/user_spec.rb",
    "RSpec/Focus",
    false,
    RubyVersion::default()
);
