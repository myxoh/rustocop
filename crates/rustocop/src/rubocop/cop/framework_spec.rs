// Source: spec/rubocop/cop/cop_spec.rb
// Spec SHA-256: 7d9e9850ef3594e218419afa5bc1c83d49b7f91732d7fc868452bc9c0603c9ab
// Source: spec/rubocop/cop/generator_spec.rb
// Spec SHA-256: 469c16fe84c15e94ef01b398cfae3ffca15e2ec9a94e902bbdd665869392f42d
// Source: spec/rubocop/cop/team_spec.rb
// Spec SHA-256: 2ff4bf11a7654fa824c3929c35c2a1edef4d83096aeaddf2bd1bb52dde41de09

use super::framework::*;
use super::severity::Severity;
use crate::rubocop::ast::node::core::{Ast, NodeValue};
use crate::rubocop::ast::source::SourceBuffer;
use serde_json::json;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::rc::Rc;

#[test]
fn autocorrect_modes_preserve_safe_all_and_disable_branches() {
    let safe = AutocorrectLogic {
        mode: AutocorrectMode::Safe,
        supports_autocorrect: true,
        safe_autocorrect: true,
        enabled: true,
        always_autocorrect: false,
        contextual_autocorrect: false,
        lsp_enabled: false,
    };
    assert!(safe.autocorrect());
    let unsafe_logic = AutocorrectLogic {
        safe_autocorrect: false,
        ..safe.clone()
    };
    assert!(!unsafe_logic.autocorrect());
    let all = AutocorrectLogic {
        mode: AutocorrectMode::All,
        ..unsafe_logic
    };
    assert!(all.autocorrect());
    let unsupported = AutocorrectLogic {
        mode: AutocorrectMode::All,
        supports_autocorrect: false,
        safe_autocorrect: true,
        enabled: true,
        always_autocorrect: false,
        contextual_autocorrect: false,
        lsp_enabled: false,
    };
    assert!(!unsupported.autocorrect());
    let editor = AutocorrectLogic {
        supports_autocorrect: true,
        contextual_autocorrect: true,
        lsp_enabled: true,
        ..all
    };
    assert!(!editor.autocorrect());
    assert_eq!(
        disable_offense_comment("Layout/LineLength"),
        "# rubocop:disable Layout/LineLength"
    );
    assert_eq!(
        enable_offense_comment("Layout/LineLength"),
        "# rubocop:enable Layout/LineLength"
    );
    assert!(line_with_comment_too_long("code", "note", 8));
    assert!(!line_with_comment_too_long("code", "note", 9));
}

#[test]
fn base_message_documentation_and_range_adapters_match_rubocop() {
    let message_config = super::message_annotator::MessageConfig::default();
    let cop_message_config = super::message_annotator::CopMessageConfig::default();
    let options = super::message_annotator::MessageOptions {
        debug: true,
        ..Default::default()
    };
    assert_eq!(
        find_message(
            None,
            "default message",
            &message_config,
            "Style/TestCop",
            &cop_message_config,
            &options
        ),
        "Style/TestCop: default message"
    );
    assert_eq!(
        documentation_url("Style/TestCop", true, None).as_deref(),
        Some("https://docs.rubocop.org/rubocop/cops_style.html#testcop")
    );
    assert!(documentation_url("Unqualified", true, None).is_none());

    let mut ast = Ast::new("call");
    let node = ast.add_node("send", vec![], Some(0..4));
    ast.set_location(node, "selector", 0..4, "call");
    assert_eq!(
        range_from_node_or_range(NodeOrRange::Node(ast.node(node))).unwrap(),
        0..4
    );
    assert_eq!(find_location(ast.node(node), Some("selector")), Some(0..4));
    assert_eq!(range_for_original(1..3, 5), 6..8);
}

#[test]
fn base_correction_lifecycle_preserves_status_branches_and_merge() {
    let source = SourceBuffer::new("abc");
    assert!(current_corrector(&source, false).is_none());
    let mut current = current_corrector(&source, true).unwrap();
    let mut proposed = super::corrector::Corrector::new(&source);
    proposed.replace(
        crate::rubocop::ast::source::SourceRange::new(&source, 1, 2),
        "B",
    );
    let logic = AutocorrectLogic {
        mode: AutocorrectMode::All,
        supports_autocorrect: true,
        safe_autocorrect: true,
        enabled: true,
        always_autocorrect: true,
        contextual_autocorrect: false,
        lsp_enabled: false,
    };
    assert_eq!(
        correct(&mut current, Some(&proposed), true, &logic).unwrap(),
        CorrectionStatus::Corrected
    );
    assert_eq!(current.rewrite().unwrap(), "aBc");
    assert!(support_autocorrect(true));
    assert!(correction_lambda(true, false));
    assert!(!correction_lambda(true, true));

    let mut none = super::corrector::Corrector::new(&source);
    assert_eq!(
        use_corrector(&mut none, Some(&proposed), false, false, true, false, false),
        CorrectionStatus::Uncorrected
    );
    assert_eq!(
        attempt_correction(&mut none, None, true),
        CorrectionStatus::CorrectedWithTodo
    );
    assert_eq!(
        suppress_clobbering::<()>(Err(super::corrector::CorrectionError::InvalidRange)),
        None
    );
}

#[test]
fn disable_offense_selects_inline_surrounding_and_multiline_literal_forms() {
    let short = SourceBuffer::new("call\n");
    let offense = crate::rubocop::ast::source::SourceRange::new(&short, 0, 4);
    assert_eq!(
        disable_offense(&short, None, offense, "Lint/Test", Some(120)).unwrap(),
        "call # rubocop:todo Lint/Test\n"
    );

    let long = SourceBuffer::new("  call\n");
    let offense = crate::rubocop::ast::source::SourceRange::new(&long, 2, 6);
    assert_eq!(
        disable_offense(&long, None, offense, "Lint/Test", Some(5)).unwrap(),
        "  # rubocop:todo Lint/Test\n  call\n  # rubocop:enable Lint/Test\n"
    );

    let source = "%w[one\ntwo]\n";
    let processed = crate::rubocop::ast::processed_source::ProcessedSource::new(
        source,
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let buffer = SourceBuffer::new(source);
    let offense = crate::rubocop::ast::source::SourceRange::new(&buffer, 3, 6);
    assert_eq!(
        disable_offense(&buffer, processed.ast(), offense, "Lint/Test", Some(120)).unwrap(),
        "# rubocop:todo Lint/Test\n%w[one\ntwo]\n# rubocop:enable Lint/Test\n"
    );
}

#[test]
fn eol_comment_ignores_hashes_inside_string_literals() {
    assert_eq!(eol_comment("puts '# no' # yes"), Some("# yes"));
    assert_eq!(eol_comment("puts \"# no\""), None);
    assert!(multiline_string("\"a\nb\""));
}

#[test]
fn autocorrect_disable_helpers_preserve_inline_and_surrounding_comment_layouts() {
    let logic = AutocorrectLogic {
        mode: AutocorrectMode::DisableUncorrectable,
        supports_autocorrect: false,
        safe_autocorrect: true,
        enabled: true,
        always_autocorrect: false,
        contextual_autocorrect: false,
        lsp_enabled: false,
    };
    assert!(logic.autocorrect_with_disable_uncorrectable());
    assert!(logic.safe_autocorrect());
    let buffer = SourceBuffer::new("  first\n  second\n");
    let first = crate::rubocop::ast::source::SourceRange::new(&buffer, 2, 7);
    assert_eq!(range_of_first_line(first).source(), "  first");
    let both = crate::rubocop::ast::source::SourceRange::new(&buffer, 2, 16);
    assert_eq!(range_by_lines(both).source(), "  first\n  second");
    assert_eq!(
        disable_offense_at_end_of_line(&buffer, range_of_first_line(first), "Lint/Test").unwrap(),
        "  first # rubocop:todo Lint/Test\n  second\n"
    );
    assert_eq!(
        disable_offense_before_and_after(&buffer, both, "Lint/Test").unwrap(),
        "  # rubocop:todo Lint/Test\n  first\n  second\n  # rubocop:enable Lint/Test\n"
    );
    assert_eq!(max_line_length(true, None), Some(120));
    assert_eq!(max_line_length(false, Some(80)), None);
}

#[test]
fn base_cop_tracks_lifecycle_disabled_lines_and_global_offenses() {
    let mut values = BTreeMap::new();
    values.insert("TargetRubyVersion".into(), json!(3.4));
    let mut cop = BaseCop::new("Lint/Test", BaseConfig::new(values));
    cop.begin_investigation();
    assert!(cop.ready());
    cop.disable_line(2);
    assert_eq!(cop.currently_disabled_lines(), &BTreeSet::from([2]));
    assert!(!cop.add_offense(2..3, "bad", Some(Severity::Warning), false));
    assert!(cop.add_offense(3..4, "bad", Some(Severity::Warning), false));
    cop.add_global_offense("global", Some(Severity::Error));
    assert_eq!(cop.offenses().len(), 2);
    assert_eq!(cop.target_ruby_version(), 3.4);
    assert!(cop.lint());
    assert_eq!(cop.complete_investigation().len(), 2);
    cop.reset_investigation();
    assert!(cop.current_offenses().is_empty());
    assert!(!cop.ready());
}

#[test]
fn base_configuration_helpers_and_offense_deduplication_follow_rubocop_order() {
    let mut values = BTreeMap::new();
    values.insert("AutoCorrect".into(), json!("contextual"));
    values.insert("Severity".into(), json!("error"));
    values.insert("TargetRailsVersion".into(), json!(8.0));
    values.insert("ActiveSupportExtensionsEnabled".into(), json!(true));
    values.insert("StringLiteralsFrozenByDefault".into(), json!(true));
    values.insert("Include".into(), json!(["lib/*.rb"]));
    values.insert("GemVersions".into(), json!({"rails":"8.0.1"}));
    values.insert("ParserEngine".into(), json!("parser_prism"));
    values.insert("TargetRubyVersion".into(), json!(3.4));
    let mut cop = BaseCop::new("Style/Test", BaseConfig::new(values));
    cop.begin_investigation();
    assert!(cop.contextual_autocorrect());
    assert!(!cop.always_autocorrect());
    assert_eq!(cop.find_severity(None), Severity::Error);
    assert_eq!(cop.target_rails_version(), Some(8.0));
    assert!(cop.active_support_extensions_enabled());
    assert!(cop.string_literals_frozen_by_default());
    assert_eq!(cop.target_gem_version("rails"), Some("8.0.1"));
    assert!(cop.file_name_matches_any("lib/example.rb", "Include", false));
    assert!(cop.add_offense(4..5, "first", Some(Severity::Convention), false));
    assert!(!cop.add_offense(4..5, "duplicate", Some(Severity::Convention), false));
    assert_eq!(cop.current_offenses().len(), 1);
    assert!(cop.current_offense_locations().contains(&(4, 5)));
    assert_eq!(cop.parse("1", None).unwrap().ast().unwrap().kind(), "int");
}

#[test]
fn legacy_cop_identity_qualification_and_severity_match_the_pinned_contract() {
    let style = BaseCop::new("Style/For", BaseConfig::default());
    assert_eq!(style.cop_name(), "Style/For");
    assert_eq!(style.department(), Some("Style"));
    assert_eq!(style.default_severity(), Severity::Convention);

    let lint = BaseCop::new("Lint/Loop", BaseConfig::default());
    assert_eq!(lint.cop_name(), "Lint/Loop");
    assert_eq!(lint.department(), Some("Lint"));
    assert_eq!(lint.default_severity(), Severity::Warning);

    let mut configured = BaseCop::new(
        "Style/For",
        BaseConfig::new(BTreeMap::from([("Severity".into(), json!("warning"))])),
    );
    assert!(configured.add_offense(0..1, "configured", None, false));
    assert_eq!(configured.offenses()[0].severity, Severity::Warning);
    assert!(configured.custom_severity_warning().is_none());

    let mut invalid = BaseCop::new(
        "Style/For",
        BaseConfig::new(BTreeMap::from([("Severity".into(), json!("superbad"))])),
    );
    assert_eq!(
        invalid.custom_severity_warning().as_deref(),
        Some("Warning: Invalid severity 'superbad'.")
    );
    assert!(invalid.add_offense(0..1, "invalid", None, false));
    assert_eq!(invalid.offenses()[0].severity, Severity::Convention);
    assert_eq!(
        invalid.warnings(),
        ["Warning: Invalid severity 'superbad'."]
    );

    let (name, warning) = qualified_cop_name("Layout/LineLength", "--only");
    assert_eq!(name, "Layout/LineLength");
    assert!(warning.contains("`Cop.qualified_cop_name` is deprecated"));
}

#[test]
fn pessimistic_gem_requirements_match_rubygems_segment_precision() {
    let satisfies = |version: &str, requirement: &str| {
        let mut values = BTreeMap::new();
        values.insert("GemVersions".into(), json!({"example": version}));
        let mut cop = BaseCop::new("Lint/Test", BaseConfig::new(values));
        cop.requires_gem("example", &[requirement]);
        cop.target_satisfies_all_gem_version_requirements()
    };

    assert!(satisfies("2.9", "~> 2.2"));
    assert!(!satisfies("3.0", "~> 2.2"));
    assert!(satisfies("2.2.9", "~> 2.2.0"));
    assert!(!satisfies("2.3.0", "~> 2.2.0"));
    assert!(satisfies("2.2.0", "= 2.2"));
}

struct TestCop {
    finding: Option<Finding>,
    seen: Vec<String>,
    relevant: bool,
}
impl CopRuntime for TestCop {
    fn name(&self) -> &str {
        "Test/Cop"
    }
    fn begin_investigation(&mut self, _: &SourceBuffer<'_>) {
        self.seen.push("begin".into())
    }
    fn on_node(&mut self, node: crate::rubocop::ast::node::core::NodeRef<'_>) {
        self.seen.push(node.kind().into());
        if node.kind() == "int" {
            self.finding = Some(Finding::new(
                "Test/Cop",
                0..1,
                "integer",
                Severity::Convention,
                false,
            ));
        }
    }
    fn on_investigation_end(&mut self, _: &SourceBuffer<'_>) {
        self.seen.push("end".into())
    }
    fn take_findings(&mut self) -> Vec<Finding> {
        self.finding.take().into_iter().collect()
    }
    fn relevant_file(&self, _: &str) -> bool {
        self.relevant
    }
}

#[test]
fn commissioner_invokes_lifecycle_depth_first_and_groups_findings() {
    let mut ast = Ast::new("1");
    let int = ast.add_node("int", vec![NodeValue::Integer(1)], Some(0..1));
    let begin = ast.add_node("begin", vec![NodeValue::Node(int)], Some(0..1));
    let buffer = SourceBuffer::new("1");
    let mut commissioner = Commissioner::new(vec![Box::new(TestCop {
        finding: None,
        seen: Vec::new(),
        relevant: true,
    })]);
    let findings = commissioner.investigate(&buffer, Some(ast.node(begin)));
    assert_eq!(findings.len(), 1);
    assert_eq!(
        Commissioner::offenses_per_cop(&findings)["Test/Cop"].len(),
        1
    );
}

#[test]
fn team_skips_files_when_no_cop_is_relevant() {
    let buffer = SourceBuffer::new("1");
    let mut team = Team::new(
        vec![Box::new(TestCop {
            finding: None,
            seen: Vec::new(),
            relevant: false,
        })],
        AutocorrectMode::None,
    );
    let result = team.investigate("ignored.rb", &buffer, None);
    assert!(result.findings.is_empty());
    assert!(!team.autocorrect());
    assert_eq!(team.max_iterations(), 200);
}

struct CorrectingCop {
    name: &'static str,
    plan: Option<CorrectionPlan>,
    incompatible: &'static [&'static str],
}

impl CopRuntime for CorrectingCop {
    fn name(&self) -> &str {
        self.name
    }
    fn take_findings(&mut self) -> Vec<Finding> {
        Vec::new()
    }
    fn take_correction(&mut self) -> Option<CorrectionPlan> {
        self.plan.take()
    }
    fn autocorrect_incompatible_with(&self) -> &[&str] {
        self.incompatible
    }
    fn supports_autocorrect(&self) -> bool {
        true
    }
}

fn correction(range: std::ops::Range<usize>, replacement: &str) -> CorrectionPlan {
    let mut plan = CorrectionPlan::new();
    plan.replace(range, replacement);
    plan
}

#[test]
fn team_applies_runtime_corrections_and_suppresses_incompatible_and_clobbering_cops() {
    let mut team = Team::new(
        vec![
            Box::new(CorrectingCop {
                name: "Test/First",
                plan: Some(correction(0..1, "A")),
                incompatible: &["Test/Second"],
            }),
            Box::new(CorrectingCop {
                name: "Test/Second",
                plan: Some(correction(1..2, "B")),
                incompatible: &[],
            }),
            Box::new(CorrectingCop {
                name: "Test/Clobber",
                plan: Some(correction(0..2, "overlap")),
                incompatible: &[],
            }),
        ],
        AutocorrectMode::All,
    );
    let source = SourceBuffer::new("abcd");
    let result = team.investigate("example.rb", &source, None);
    assert_eq!(result.updated_source.as_deref(), Some("Abcd"));
}

struct CountingCop(Rc<Cell<usize>>);

impl CopRuntime for CountingCop {
    fn name(&self) -> &str {
        "Test/ReadOnly"
    }
    fn on_new_investigation(&mut self) {
        self.0.set(self.0.get() + 1);
    }
    fn take_findings(&mut self) -> Vec<Finding> {
        Vec::new()
    }
}

#[test]
fn team_defers_non_autocorrecting_cops_until_no_correction_is_available() {
    let count = Rc::new(Cell::new(0));
    let mut team = Team::new(
        vec![
            Box::new(CorrectingCop {
                name: "Test/Correcting",
                plan: Some(correction(0..1, "A")),
                incompatible: &[],
            }),
            Box::new(CountingCop(count.clone())),
        ],
        AutocorrectMode::All,
    );
    let source = SourceBuffer::new("abcd");
    assert_eq!(
        team.investigate("example.rb", &source, None)
            .updated_source
            .as_deref(),
        Some("Abcd")
    );
    assert_eq!(count.get(), 0);

    let count = Rc::new(Cell::new(0));
    let mut team = Team::new(
        vec![
            Box::new(CorrectingCop {
                name: "Test/Correcting",
                plan: None,
                incompatible: &[],
            }),
            Box::new(CountingCop(count.clone())),
        ],
        AutocorrectMode::All,
    );
    assert!(team
        .investigate("example.rb", &source, None)
        .updated_source
        .is_none());
    assert_eq!(count.get(), 1);
}

#[test]
fn utility_string_and_range_helpers_match_ruby_forms() {
    let buffer = SourceBuffer::new("  call\n# comment\n");
    assert!(begins_its_line(2..6, &buffer));
    assert!(comment_line(buffer.source_line(2)));
    assert!(parentheses("(x)"));
    assert_eq!(indent("a\nb", 2), "  a\n  b");
    assert_eq!(escape_string("a b"), "a b");
    assert_eq!(interpret_string_escapes(r"a\nb"), "a\nb");
    assert_eq!(
        interpret_string_escapes(r"\x41\101\u0042\u{43 44}"),
        "AABCD"
    );
    assert_eq!(to_string_literal("a'b"), "\"a'b\"");
    assert!(double_quotes_required("a'b"));
    assert!(!double_quotes_required("a#{b}"));
    assert_eq!(to_supported_styles("EnforcedStyle"), "SupportedStyles");
    assert_eq!(trim_string_interpolation_escape(r"\#{x}"), "#{x}");
}

#[test]
fn utility_node_walk_call_chain_and_argument_locations_match_rubocop() {
    let mut ast = Ast::new("base.one two");
    let base = ast.add_node("lvar", vec![NodeValue::Symbol("base".into())], Some(0..4));
    let one = ast.add_node(
        "send",
        vec![NodeValue::Node(base), NodeValue::Symbol("one".into())],
        Some(0..8),
    );
    ast.set_location(one, "selector", 5..8, "one");
    let two = ast.add_node(
        "send",
        vec![NodeValue::Node(one), NodeValue::Symbol("two".into())],
        Some(0..12),
    );
    ast.set_location(two, "selector", 9..12, "two");
    ast.complete(two);
    let root = ast.node(two);
    assert_eq!(first_part_of_call_chain(root).kind(), "lvar");
    assert!(any_descendant(root, &["send"], |node| node == ast.node(one)));
    assert_eq!(on_node(&["send"], root, &[]).len(), 2);
    assert_eq!(on_node(&[], root, &["send"]).len(), 1);
    assert_eq!(args_begin(root), Some(12..13));
    assert_eq!(args_end(root), Some(12));
    let buffer = SourceBuffer::new("base.one two\nnext");
    assert_eq!(line_range(root), 1..=1);
    assert!(same_line(&root, &(1..2), &buffer));
    assert!(!same_line(&root, &usize::MAX, &buffer));
}

#[test]
fn utility_add_parentheses_covers_argument_callable_and_plain_nodes() {
    let mut ast = Ast::new("call value");
    let value = ast.add_node("lvar", vec![NodeValue::Symbol("value".into())], Some(5..10));
    let call = ast.add_node(
        "send",
        vec![
            NodeValue::Nil,
            NodeValue::Symbol("call".into()),
            NodeValue::Node(value),
        ],
        Some(0..10),
    );
    ast.set_location(call, "selector", 0..4, "call");
    ast.complete(call);
    let buffer = SourceBuffer::new("call value");
    assert_eq!(
        add_parentheses(&buffer, ast.node(call)).unwrap(),
        "call(value)"
    );

    let mut plain = Ast::new("value");
    let value = plain.add_node("lvar", vec![NodeValue::Symbol("value".into())], Some(0..5));
    plain.complete(value);
    let buffer = SourceBuffer::new("value");
    assert_eq!(
        add_parentheses(&buffer, plain.node(value)).unwrap(),
        "(value)"
    );
}

#[test]
fn variable_table_resolves_references_from_inner_to_outer_scope() {
    let mut table = VariableTable::new();
    table.assign("outer", 0..1);
    table.enter_scope();
    table.assign("inner", 2..3);
    assert!(table.reference("outer", 4..5));
    assert!(table.reference("inner", 5..6));
    assert!(!table.reference("missing", 6..7));
    let inner = table.leave_scope();
    assert_eq!(inner[0].name, "inner");
    assert_eq!(table.variables()[0].references.len(), 1);
}

#[test]
fn variable_force_scans_assignments_and_references() {
    let mut ast = Ast::new("x = 1; x");
    let one = ast.add_node("int", vec![NodeValue::Integer(1)], Some(4..5));
    let assignment = ast.add_node(
        "lvasgn",
        vec![NodeValue::Symbol("x".into()), NodeValue::Node(one)],
        Some(0..5),
    );
    let reference = ast.add_node("lvar", vec![NodeValue::Symbol("x".into())], Some(7..8));
    let root = ast.add_node(
        "begin",
        vec![NodeValue::Node(assignment), NodeValue::Node(reference)],
        Some(0..8),
    );
    let table = scan_variables(ast.node(root));
    let variable = table
        .variables()
        .into_iter()
        .find(|v| v.name == "x")
        .unwrap();
    assert_eq!(variable.assignments.len(), 1);
    assert_eq!(variable.references.len(), 1);
}

#[test]
fn variable_table_models_block_capture_method_isolation_and_assignment_lifetimes() {
    let mut table = VariableTable::new();
    table.assign("outer", 0..1);
    table.enter_scope_kind(VariableScopeKind::Block);
    assert!(table.reference("outer", 2..3));
    table.leave_scope();
    let outer = table
        .variables()
        .into_iter()
        .find(|variable| variable.name == "outer")
        .unwrap();
    assert!(outer.captured_by_block());
    assert!(outer.used());
    assert!(outer.assignment_used(0));

    table.enter_scope_kind(VariableScopeKind::Method);
    assert!(!table.reference("outer", 4..5));
    table.declare("keyword", 5..6, "kwarg");
    assert!(table
        .accessible_variables()
        .iter()
        .any(|variable| variable.name == "keyword"));
    let keyword = table
        .variables()
        .into_iter()
        .find(|variable| variable.name == "keyword")
        .unwrap();
    assert!(keyword.argument());
    assert!(keyword.keyword_argument());
    table.leave_scope();

    table.assign("unused", 6..7);
    table.assign("unused", 8..9);
    let unused = table
        .variables()
        .into_iter()
        .find(|variable| variable.name == "unused")
        .unwrap();
    assert!(!unused.assignment_used(0));
    assert!(!unused.used());

    table.declare("_shadow", 9..10, "shadowarg");
    let shadow = table
        .variables()
        .into_iter()
        .find(|variable| variable.name == "_shadow")
        .unwrap();
    assert!(shadow.should_be_unused());
    assert!(shadow.explicit_block_local_variable());
}

#[test]
fn variable_branches_detect_always_run_exclusive_and_exception_paths() {
    let processed = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "if ready\n  left\nelse\n  right\nend",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let conditional = processed.ast().unwrap();
    let condition = conditional.node_child(0).unwrap();
    assert!(VariableBranch::of(condition, None).is_none());
    let left = VariableBranch::of(conditional.node_child(1).unwrap(), None).unwrap();
    let right = VariableBranch::of(conditional.node_child(2).unwrap(), None).unwrap();
    assert!(left.branched());
    assert!(left.exclusive_with(Some(right)));
    assert_eq!(left.control, right.control);

    let processed = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "begin\n  work\nrescue\n  recover\nensure\n  cleanup\nend",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let root = processed.ast().unwrap();
    let work = root
        .each_descendant(&["send"])
        .into_iter()
        .find(|node| node.method_name() == Some("work"))
        .unwrap();
    let cleanup = root
        .each_descendant(&["send"])
        .into_iter()
        .find(|node| node.method_name() == Some("cleanup"))
        .unwrap();
    let work_branch = VariableBranch::of(work, None).unwrap();
    assert!(work_branch.may_jump_to_other_branch());
    assert!(work_branch.may_run_incompletely());
    assert!(VariableBranch::of(cleanup, None).is_none());
}

#[test]
fn variable_scope_and_reference_adapters_preserve_visibility_boundaries() {
    let processed = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "receiver.define_singleton_method(:call) do |argument|\n  inside\nend\noutside",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let root = processed.ast().unwrap();
    let block = root.each_descendant(&["block"]).into_iter().next().unwrap();
    let scope = VariableScopeView::new(block).unwrap();
    assert_eq!(scope.name(), Some("define_singleton_method"));
    assert!(!scope.naked_top_level());
    assert!(!scope.includes(block.node_child(0).unwrap()));
    assert!(scope.includes(block.arguments_node().unwrap()));
    assert!(scope.includes(block.body().unwrap()));
    assert!(!scope.includes(root.node_child(1).unwrap()));
    assert!(scope.nodes().iter().any(|node| node.kind() == "arg"));

    let explicit_source = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "value = 1\nvalue",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let explicit_node = explicit_source
        .ast()
        .unwrap()
        .each_descendant(&["lvar"])
        .into_iter()
        .next()
        .unwrap();
    let explicit = VariableReference::new(explicit_node, None).unwrap();
    assert!(explicit.explicit());
    let implicit_source = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "def call(value)\n  super\nend",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let zsuper = implicit_source
        .ast()
        .unwrap()
        .each_descendant(&["zsuper"])
        .into_iter()
        .next()
        .unwrap();
    assert!(!VariableReference::new(zsuper, None).unwrap().explicit());
}

#[test]
fn variable_assignment_adapter_reports_meta_assignment_and_usage_state() {
    let processed = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "value += 1",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let root = processed.ast().unwrap();
    let lhs = root.node_child(0).unwrap();
    let mut assignment = VariableAssignment::new(lhs, None).unwrap();
    assert_eq!(assignment.name(), Some("value"));
    assert!(assignment.operator_assignment());
    assert_eq!(assignment.operator().as_deref(), Some("+="));
    assignment.reassign();
    assert!(assignment.reassigned());
    assert!(!assignment.used());
    assignment.capture_with_block();
    assert!(!assignment.used());
    assignment.reference(root);
    assert!(assignment.referenced());
    assert!(assignment.used());

    let processed = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "left, right = pair",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    for lhs in processed.ast().unwrap().each_descendant(&["lvasgn"]) {
        assert!(VariableAssignment::new(lhs, None)
            .unwrap()
            .multiple_assignment());
    }
}

#[test]
fn generator_produces_source_spec_and_pending_config() {
    let generated = Generator::generate("Style/MyCop", "Checks a thing.").unwrap();
    assert_eq!(generated.source_path, "lib/rubocop/cop/style/my_cop.rb");
    assert!(generated.source.contains("class MyCop < Base"));
    assert!(generated
        .spec
        .contains("RSpec.describe RuboCop::Cop::Style::MyCop"));
    assert!(generated.config.contains("Enabled: pending"));
    assert!(Generator::generate("MissingDepartment", "bad").is_err());
    assert_eq!(Generator::snake_case("RSpecFoo/Bar"), "rspec_foo/bar");
    let nested = Generator::generate("Plugin/Style/FakeCop", "Checks a thing.").unwrap();
    assert_eq!(
        nested.source_path,
        "lib/rubocop/cop/plugin/style/fake_cop.rb"
    );
    assert!(nested.source.contains("module Plugin::Style"));
    assert!(Generator::todo("FakeCop").is_err());
    assert!(Generator::todo("Style/FakeCop")
        .unwrap()
        .contains("Modify the description of Style/FakeCop"));
}

#[test]
fn filesystem_generator_writes_the_upstream_spec_path_once() {
    let generator = super::generator::Generator::initialize("Style/GeneratedCop").unwrap();
    let root = std::env::temp_dir().join(format!(
        "rustocop-generator-contract-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    let path = generator.write_spec(&root).unwrap();
    assert_eq!(
        path.strip_prefix(&root).unwrap(),
        std::path::Path::new("spec/rubocop/cop/style/generated_cop_spec.rb")
    );
    assert!(std::fs::read_to_string(&path)
        .unwrap()
        .contains("RuboCop::Cop::Style::GeneratedCop"));
    std::fs::remove_dir_all(root).unwrap();
}

#[test]
fn translated_team_mobilization_and_file_inspection_are_end_to_end() {
    let count = Rc::new(Cell::new(0));
    let descriptor = super::team::CopDescriptor {
        name: "Test/ReadOnly".into(),
        joining_forces: Vec::new(),
        target_ruby_supported: true,
        target_rails_supported: true,
        config_valid: true,
    };
    let mut team = super::team::Team::mobilize(
        vec![Box::new(CountingCop(count.clone()))],
        vec![descriptor],
        super::team::TeamOptions {
            autocorrect: Some(false),
            debug: Some(false),
            stdin: false,
        },
    )
    .unwrap();
    let processed = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "value = 1",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    assert!(team.inspect_file(&processed).is_empty());
    assert_eq!(count.get(), 1);
}

#[test]
fn finding_deduplication_uses_cop_location_and_message() {
    let finding = Finding::new("Test/Cop", 1..2, "same", Severity::Convention, false);
    let mut findings = vec![finding.clone(), finding];
    dedupe_findings(&mut findings);
    assert_eq!(findings.len(), 1);
}

#[test]
fn public_base_range_encoding_and_legacy_autocorrect_contracts_are_executable() {
    assert!(emulate_v0_callsequence(
        false,
        Option::<fn() -> Result<(), super::corrector::CorrectionError>>::None
    )
    .is_err());
    assert!(emulate_v0_callsequence(
        true,
        Some(|| Err(super::corrector::CorrectionError::InvalidRange))
    )
    .is_ok());
    assert!(compatible_external_encoding_for("utf-8"));
    assert!(include_or_equal(2..4, &4));
    assert!(!include_or_equal(2..4, &5));

    let buffer = SourceBuffer::new("first\nsecond\n");
    assert_eq!(buffer_line_range(&buffer, 2), 6..12);

    let parsed = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "(value)",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    assert!(node_parentheses(parsed.ast().unwrap()));

    let mut cop = BaseCop::new("Lint/Test", BaseConfig::default());
    cop.set_processed_source(&parsed);
    cop.set_project_index(Some(7));
    cop.set_config_to_allow_offenses(BTreeMap::from([("Enabled".into(), json!(false))]));
    assert_eq!(cop.processed_source(), Some("(value)"));
    assert_eq!(cop.project_index(), Some(7));
    assert_eq!(
        cop.config_to_allow_offenses().get("Enabled"),
        Some(&json!(false))
    );
}

#[test]
fn public_variable_assignment_classifiers_cover_modifier_branch_and_meta_shapes() {
    let modifier = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "value = 1 if ready",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let assignment_node = modifier.ast().unwrap().each_node(&["lvasgn"])[0];
    assert!(Variable::in_modifier_conditional(assignment_node));

    let regexp = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "/(?<name>x)/ =~ text",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let capture = regexp.ast().unwrap().each_node(&["match_with_lvasgn"])[0];
    assert!(VariableAssignment::new(capture, None)
        .unwrap()
        .regexp_named_capture());

    let rescue = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "begin\n  work\nrescue => error\nend",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let exception = rescue.ast().unwrap().each_node(&["lvasgn"])[0];
    assert!(VariableAssignment::new(exception, None)
        .unwrap()
        .exception_assignment());

    let rest = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "head, *tail = values",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let tail = rest
        .ast()
        .unwrap()
        .each_node(&["lvasgn"])
        .into_iter()
        .find(|node| node.name() == Some("tail"))
        .unwrap();
    let tail = VariableAssignment::new(tail, None).unwrap();
    let _ = tail.rest_assignment();

    let for_source = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "for value in values do\n  value\nend",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let for_assignment = for_source.ast().unwrap().each_node(&["lvasgn"])[0];
    assert!(VariableAssignment::new(for_assignment, None)
        .unwrap()
        .for_assignment());

    let branches = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "if ready\n  value = 1\nelse\n  value\nend",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let root = branches.ast().unwrap();
    let assigned = VariableAssignment::new(root.each_node(&["lvasgn"])[0], None).unwrap();
    let referenced = VariableReference::new(root.each_node(&["lvar"])[0], None).unwrap();
    assert!(assigned.runs_exclusively_with(referenced));

    let reference_branches = crate::rubocop::ast::processed_source::ProcessedSource::new(
        "value = 0\nif ready\n  value\nelse\n  value\nend",
        3.4,
        None,
        crate::rubocop::ast::processed_source::ParserEngine::Prism,
    )
    .unwrap();
    let references = reference_branches.ast().unwrap().each_node(&["lvar"]);
    let left = VariableReference::new(references[0], None).unwrap();
    let right = VariableReference::new(references[1], None).unwrap();
    assert!(left.runs_exclusively_with(right));

    let mut table = VariableTable::new();
    table.assign("value", 0..1);
    let mut variable = table.variables().into_iter().next().unwrap().clone();
    variable.mark_last_as_reassigned(true);
    assert!(!variable.assignment_used(0));
}
