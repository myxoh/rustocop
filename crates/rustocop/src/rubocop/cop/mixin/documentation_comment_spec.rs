use super::documentation_comment::*;

fn line(number: usize, text: &str, comment: bool) -> DocumentationLine {
    DocumentationLine {
        line: number,
        source: text.into(),
        text: text.into(),
        comment,
    }
}

#[test]
fn documentation_requires_an_adjacent_comment_but_any_non_directive_line_can_document() {
    let checker = DocumentationComment::new(vec!["TODO".into(), "FIXME".into()]);
    let node = line(4, "class Widget", false);
    assert!(checker.documentation_comment(
        &node,
        &[
            line(2, "# TODO: later", true),
            line(3, "# Widget docs", true)
        ]
    ));
    assert!(!checker.documentation_comment(&node, &[line(2, "# Widget docs", true)]));
    assert!(!checker.documentation_comment(&node, &[line(3, "# TODO: later", true)]));
}

#[test]
fn magic_rubocop_and_annotation_comments_are_not_documentation() {
    let checker = DocumentationComment::new(vec!["TODO".into()]);
    assert!(checker.interpreter_directive_comment(&line(1, "# frozen_string_literal: true", true)));
    assert!(checker.rubocop_directive_comment(&line(1, "# rubocop:disable Metrics", true)));
    assert_eq!(checker.annotation_keywords(), ["TODO"]);
    assert!(checker.precede(&line(1, "# a", true), &line(2, "class A", false)));
}
