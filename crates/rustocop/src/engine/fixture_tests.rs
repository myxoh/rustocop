use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use super::InspectionPlan;
use crate::config::{CopConfig, CopSelection, InspectionConfig, RubyVersion};
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
        cop_config: Arc::new(CopConfig::default()),
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

// New-cop generator registrations are inserted directly below this line.

fixture_test!(
    scans_utf8_comment_annotations_without_splitting_characters,
    "comment_annotation_utf8_real_project_regression",
    "/project/test/models/user_test.rb",
    "Style/CommentAnnotation",
    true,
    RubyVersion::default()
);

fixture_test!(
    checks_space_before_first_argument_on_call_nodes,
    "space_before_first_arg_real_project_regression",
    "/project/spec/rspec/core/drb_spec.rb",
    "Layout/SpaceBeforeFirstArg",
    true,
    RubyVersion::default()
);

fixture_test!(
    tracks_underscore_variables_by_declaration_and_scope,
    "underscore_prefixed_variable_real_project_regression",
    "/project/actionpack/lib/action_controller/metal/strong_parameters.rb",
    "Lint/UnderscorePrefixedVariableName",
    false,
    RubyVersion::default()
);

fixture_test!(
    accepts_multibyte_offsets_at_node_ends,
    "unicode_node_end_real_project_regression",
    "/project/activerecord/test/cases/base_test.rb",
    "Style/TrailingMethodEndStatement,Style/TrailingCommaInArrayLiteral,Style/Semicolon",
    false,
    RubyVersion::default()
);

fixture_test!(
    skips_ordinary_cops_on_real_project_parse_errors,
    "recovered_syntax_real_project_regression",
    "/project/lib/generators/active_record/templates/migration.rb",
    "Style/TrailingBodyOnClass,Style/TrailingCommaInArrayLiteral,Style/Semicolon",
    false,
    RubyVersion::default()
);

fixture_test!(
    handles_empty_single_line_do_end_block,
    "block_delimiters_real_project_regression",
    "/project/railties/lib/rails/application/bootstrap.rb",
    "Style/BlockDelimiters",
    true,
    RubyVersion::default()
);

fixture_test!(
    accepts_percent_text_in_utf8_interpolation,
    "percent_literal_real_project_regression",
    "/project/lib/seeders/reports/report_data_seeder.rb",
    "Style/PercentLiteralDelimiters",
    false,
    RubyVersion::default()
);

fixture_test!(
    accepts_utf8_interpolation_inside_class,
    "trailing_body_class_real_project_regression",
    "/project/app/services/notification/push_test_service.rb",
    "Style/TrailingBodyOnClass",
    false,
    RubyVersion::default()
);

fixture_test!(
    ignores_heredoc_braces_as_inline_blocks,
    "explicit_block_argument_real_project_regression",
    "/project/lib/linear/mutations.rb",
    "Style/ExplicitBlockArgument",
    false,
    RubyVersion::default()
);

fixture_test!(
    accepts_used_argument_in_endless_method,
    "unused_method_argument_real_project_regression",
    "/project/app/builders/v2/reports/drilldown_builder.rb",
    "Lint/UnusedMethodArgument",
    false,
    RubyVersion::default()
);

fixture_test!(
    ignores_dependency_calls_outside_gemspecs,
    "ordered_dependencies_real_project_regression",
    "/project/rubocop/cop/rspec/before_all.rb",
    "Gemspec/OrderedDependencies",
    false,
    RubyVersion::default()
);

fixture_test!(
    ignores_gem_declarations_outside_bundler_files,
    "ordered_gems_path_real_project_regression",
    "/project/railties/test/generators/actions_test.rb",
    "Bundler/OrderedGems",
    false,
    RubyVersion::default()
);

fixture_test!(
    ignores_semicolons_in_embedded_documents,
    "semicolon_embedded_document_real_project_regression",
    "/project/script/import_scripts/nabble.rb",
    "Style/Semicolon",
    false,
    RubyVersion::default()
);

fixture_test!(
    honors_real_project_cop_directives,
    "cop_directive_real_project_regression",
    "/project/spec/lib/web_ide/settings/main_spec.rb",
    "Style/Semicolon,Style/TrailingCommaInArrayLiteral",
    false,
    RubyVersion::default()
);

fixture_test!(
    checks_static_segments_of_interpolated_word_arrays,
    "percent_literal_interpolated_array_real_project_regression",
    "/project/app/models/optimized_image.rb",
    "Style/PercentLiteralDelimiters",
    true,
    RubyVersion::default()
);

fixture_test!(
    distinguishes_regexp_receivers_from_command_arguments,
    "regexp_literal_parent_real_project_regression",
    "/project/app/validators/devise_email_validator.rb",
    "Style/RegexpLiteral",
    true,
    RubyVersion::default()
);

fixture_test!(
    preserves_word_array_unicode_matrix_and_whitespace_semantics,
    "word_array_real_project_regression",
    "/project/app/helpers/languages_helper.rb",
    "Style/WordArray",
    true,
    RubyVersion::default()
);

fixture_test!(
    rejects_unscoped_guard_clause_exits,
    "style_guard_clause_regression",
    "/project/example.rb",
    "Style/GuardClause",
    false,
    RubyVersion::default()
);

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
    "Lint/BigDecimalNew,Security/Eval,Security/JSONLoad",
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
fixture_test!(
    applies_source_rule_corrections,
    "source_rule_autocorrect",
    "/project/source_rules.rb",
    "Style/SymbolLiteral,Style/ArrayIntersectWithSingleElement,Style/EnvHome,Style/WhenThen",
    true,
    RubyVersion::default()
);
fixture_test!(
    reports_source_rule_diagnostics,
    "source_rule_diagnostics",
    "/project/source_rules.rb",
    "Lint/DuplicateElsifCondition,Lint/EnsureReturn,Naming/ClassAndModuleCamelCase",
    false,
    RubyVersion::default()
);
fixture_test!(
    applies_additional_rule_batch,
    "additional_rules",
    "/project/additional_rules.rb",
    "Style/PreferredHashMethods,Style/EmptyBlockParameter,Lint/UriEscapeUnescape,Style/OpenStructUse",
    true,
    RubyVersion::default()
);
