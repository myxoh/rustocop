use super::*;

fn context(autocorrect: bool) -> Context {
    Context::new(
        autocorrect,
        "example.rb",
        RubyVersion::default(),
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
