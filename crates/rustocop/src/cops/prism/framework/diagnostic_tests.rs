use super::*;

fn context(autocorrect: bool) -> Context {
    Context::new(
        if autocorrect {
            AutocorrectMode::All
        } else {
            AutocorrectMode::None
        },
        false,
        "example.rb",
        RubyVersion::default(),
        SourceEncoding::Utf8,
        Arc::new(CopConfig::default()),
    )
}

#[test]
fn reports_uncorrectable_findings_without_changing_source() {
    let mut context = context(true);
    context.report("Lint/Example", "Example offense.", 1..3);

    let inspection = context.finish("abcd");

    assert_eq!(inspection.corrected_source, "abcd");
    assert!(!inspection.findings[0].correctable);
    assert!(!inspection.findings[0].corrected);
}

#[test]
fn records_correctability_without_applying_disabled_corrections() {
    let mut context = context(false);
    context.replace("Style/Example", "Example offense.", (1, 3), 1..3, "X");

    let inspection = context.finish("abcd");

    assert_eq!(inspection.corrected_source, "abcd");
    assert!(inspection.findings[0].correctable);
    assert!(!inspection.findings[0].corrected);
}

#[test]
fn applies_each_correction_intent() {
    let mut context = context(true);
    context.insert("Layout/Example", "Insert.", (0, 1), 1, " ");
    context.replace("Style/Example", "Replace.", (1, 2), (1, 2), "B");
    context.remove("Style/Example", "Remove.", 3..4, 3..4);

    let inspection = context.finish("abcd");

    assert_eq!(inspection.corrected_source, "a Bc");
    assert!(inspection.findings.iter().all(|finding| finding.corrected));
}

#[test]
fn reporter_scopes_every_intent_to_one_cop() {
    let mut context = context(true);
    {
        let mut reporter = context.reporter("Style/Example");
        reporter.report("Report.", 0..1);
        reporter.replace("Replace.", 1..2, 1..2, "B");
        reporter.insert("Insert.", 2..3, 2, "!");
    }

    let inspection = context.finish("abc");

    assert_eq!(inspection.corrected_source, "aB!c");
    assert!(inspection
        .findings
        .iter()
        .all(|finding| finding.cop_name == "Style/Example"));
}

#[test]
fn rejected_conflicts_are_not_reported_as_corrected() {
    let mut context = context(true);
    context.replace("Style/First", "First.", 1..3, 1..3, "X");
    context.replace("Style/Second", "Second.", 2..4, 2..4, "Y");

    let inspection = context.finish("abcde");

    assert_eq!(inspection.corrected_source, "aXde");
    assert!(inspection.findings[0].corrected);
    assert!(!inspection.findings[1].corrected);
    assert!(inspection.findings[1].correctable);
}

#[test]
fn containing_correction_wins_over_nested_correction() {
    let mut context = context(true);
    context.replace("Style/Inner", "Inner.", 0..3, 0..3, "I");
    context.replace("Style/Outer", "Outer.", 0..5, 0..5, "O");

    let inspection = context.finish("abcde");

    assert_eq!(inspection.corrected_source, "O");
    assert!(inspection
        .findings
        .iter()
        .find(|finding| finding.cop_name == "Style/Outer")
        .is_some_and(|finding| finding.corrected));
    assert!(inspection
        .findings
        .iter()
        .find(|finding| finding.cop_name == "Style/Inner")
        .is_some_and(|finding| finding.corrected));
}

#[test]
fn identical_multi_edit_corrections_mark_each_finding_corrected() {
    let mut context = context(true);
    for offense in [0..1, 2..3] {
        let mut reporter = context.reporter("Style/Example");
        reporter.replace_many(
            "Group values.",
            offense,
            vec![(0..1, "A".to_string()), (2..3, String::new())],
        );
    }

    let inspection = context.finish("abc");

    assert_eq!(inspection.corrected_source, "Ab");
    assert!(inspection.findings.iter().all(|finding| finding.corrected));
}

#[test]
fn correction_transactions_are_atomic() {
    let mut context = context(true);
    {
        let mut reporter = context.reporter("Layout/Example");
        reporter.replace_many(
            "Move delimiters.",
            0..6,
            vec![(0..1, "[".to_string()), (5..6, "]".to_string())],
        );
    }
    context.replace("Style/Conflict", "Conflict.", 5..6, 5..6, "!");

    let inspection = context.finish("(abcd)");

    assert_eq!(inspection.corrected_source, "[abcd]");
    assert!(inspection.findings[0].corrected);
    assert!(!inspection.findings[1].corrected);
}

#[test]
fn invalid_transactions_leave_findings_uncorrected() {
    let mut context = context(true);
    context.replace("Style/Example", "Invalid.", 0..1, 10..11, "X");

    let inspection = context.finish("abc");

    assert_eq!(inspection.corrected_source, "abc");
    assert!(inspection.findings[0].correctable);
    assert!(!inspection.findings[0].corrected);
}

#[test]
fn source_directives_suppress_findings_and_corrections() {
    let source = concat!(
        "# rubocop:disable Style/Semicolon\n",
        "first; value\n",
        "# rubocop:enable Style/Semicolon\n",
        "second; value\n",
        "third; value # rubocop:disable Style/Semicolon -- generated\n",
    );
    let first = source.find("first;").unwrap() + "first".len();
    let second = source.find("second;").unwrap() + "second".len();
    let third = source.find("third;").unwrap() + "third".len();
    let mut context = context(true);
    for offset in [first, second, third] {
        context.remove(
            "Style/Semicolon",
            "Remove semicolon.",
            offset..offset + 1,
            offset..offset + 1,
        );
    }

    let inspection = context.finish(source);

    assert_eq!(inspection.findings.len(), 1);
    assert_eq!(inspection.findings[0].start_offset, second);
    assert!(inspection.corrected_source.contains("first; value"));
    assert!(inspection.corrected_source.contains("second value"));
    assert!(inspection.corrected_source.contains("third; value"));
}

#[test]
fn unrecognized_disable_next_does_not_suppress_findings() {
    let source = concat!(
        "# rubocop:disable-next Style/Semicolon\n",
        "first; value\n",
        "second; value\n",
    );
    let first = source.find("first;").unwrap() + "first".len();
    let second = source.find("second;").unwrap() + "second".len();
    let mut context = context(true);
    for offset in [first, second] {
        context.remove(
            "Style/Semicolon",
            "Remove semicolon.",
            offset..offset + 1,
            offset..offset + 1,
        );
    }

    let inspection = context.finish(source);

    assert_eq!(inspection.findings.len(), 2);
    assert_eq!(inspection.findings[0].start_offset, first);
    assert_eq!(inspection.findings[1].start_offset, second);
    assert!(inspection.corrected_source.contains("first value"));
    assert!(inspection.corrected_source.contains("second value"));
}

#[test]
fn disable_next_does_not_suppress_a_multiline_offense_starting_on_the_next_line() {
    let source = concat!(
        "# rubocop:disable-next Style/HashLikeCase\n",
        "case value\n",
        "when :one then 1\n",
        "when :two then 2\n",
        "when :three then 3\n",
        "end\n",
    );
    let start = source.find("case value").unwrap();
    let end = source.rfind("end").unwrap() + "end".len();
    let mut context = context(false);
    context.report("Style/HashLikeCase", "Use a hash lookup.", start..end);

    let inspection = context.finish(source);

    assert_eq!(inspection.findings.len(), 1);
}

#[test]
fn inline_disable_suppresses_a_multiline_offense_and_its_correction() {
    let source =
        "class Example < Struct.new( # rubocop:disable Style/StructInheritance\n  :value)\nend\n";
    let start = source.find("Struct.new").unwrap();
    let end = source.find(":value)").unwrap() + ":value)".len();
    let mut context = context(true);
    context.replace(
        "Style/StructInheritance",
        "Avoid inheritance.",
        start..end,
        start..end,
        "Struct.new(:value) do",
    );

    let inspection = context.finish(source);

    assert!(inspection.findings.is_empty());
    assert_eq!(inspection.corrected_source, source);
}

#[test]
fn ignored_disable_comments_expose_raw_investigation_offenses() {
    let source = "value # rubocop:disable Style/Example\n";
    let mut context = Context::new(
        AutocorrectMode::None,
        true,
        "example.rb",
        RubyVersion::default(),
        SourceEncoding::Utf8,
        Arc::new(CopConfig::default()),
    );
    context.report("Style/Example", "Visible.", 0..5);

    let inspection = context.finish(source);

    assert_eq!(inspection.findings.len(), 1);
}

#[test]
fn department_directives_suppress_member_cops() {
    let source = "value # rubocop:disable Style\n";
    let mut context = context(false);
    context.report("Style/Example", "Hidden.", 0..5);
    context.report("Lint/Example", "Visible.", 0..5);

    let inspection = context.finish(source);

    assert_eq!(inspection.findings.len(), 1);
    assert_eq!(inspection.findings[0].cop_name, "Lint/Example");
}

#[test]
fn slash_terminated_department_directives_suppress_member_cops() {
    let source = "value # rubocop:disable Metrics/\n";
    let mut context = context(false);
    context.report("Metrics/MethodLength", "Hidden.", 0..5);
    context.report("Style/Example", "Visible.", 0..5);

    let inspection = context.finish(source);

    assert_eq!(inspection.findings.len(), 1);
    assert_eq!(inspection.findings[0].cop_name, "Style/Example");
}
