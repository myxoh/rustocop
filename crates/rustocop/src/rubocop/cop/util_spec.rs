// Port of RuboCop 1.87.0 spec/rubocop/cop/util_spec.rb.
// Spec SHA-256: 8d1945f3637b5080695968de28d02efd1ec31f871718930b85cda12946444b15

use super::framework::{
    line_range, parse_regexp, same_line, to_string_literal, to_supported_styles,
};
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::ast::source::SourceBuffer;

fn parsed(source: &str) -> ProcessedSource<'_> {
    ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap()
}

#[test]
fn line_range_returns_the_expression_line_range() {
    let source =
        "foo = 1\nbar = 2\nclass Test\n  def some_method\n    do_something\n  end\nend\nbaz = 8\n";
    let processed = parsed(source);
    let class = processed
        .ast()
        .unwrap()
        .each_node(&["class"])
        .into_iter()
        .next()
        .unwrap();
    assert_eq!(line_range(class), 3..=7);
}

#[test]
fn enforced_style_maps_to_supported_styles() {
    assert_eq!(to_supported_styles("EnforcedStyle"), "SupportedStyles");
}

#[test]
fn enforced_style_suffix_is_preserved() {
    assert_eq!(
        to_supported_styles("EnforcedStyleInsidePipes"),
        "SupportedStylesInsidePipes"
    );
}

fn ivars() -> (ProcessedSource<'static>, SourceBuffer<'static>) {
    let source = "@foo + @bar\n@baz\n";
    (parsed(source), SourceBuffer::new(source))
}

#[test]
fn same_line_returns_true_for_nodes_on_the_same_line() {
    let (processed, buffer) = ivars();
    let nodes = processed.ast().unwrap().each_node(&["ivar"]);
    assert!(same_line(&nodes[0], &nodes[1], &buffer));
}

#[test]
fn same_line_returns_false_for_nodes_on_different_lines() {
    let (processed, buffer) = ivars();
    let nodes = processed.ast().unwrap().each_node(&["ivar"]);
    assert!(!same_line(&nodes[0], &nodes[2], &buffer));
}

#[test]
fn same_line_accepts_source_ranges() {
    let (processed, buffer) = ivars();
    let nodes = processed.ast().unwrap().each_node(&["ivar"]);
    assert!(same_line(
        &nodes[0].source_range().unwrap(),
        &nodes[1],
        &buffer
    ));
}

#[test]
fn same_line_returns_false_for_unsupported_values() {
    let (processed, buffer) = ivars();
    let nodes = processed.ast().unwrap().each_node(&["ivar"]);
    assert!(!same_line(&nodes[0], &5_usize, &buffer));
    assert!(!same_line(&5_usize, &nodes[1], &buffer));
}

#[test]
fn parse_regexp_returns_a_structure_for_valid_regexp() {
    assert!(parse_regexp("a+").is_ok());
}

#[test]
fn parse_regexp_returns_an_error_for_invalid_regexp() {
    assert!(parse_regexp("+").is_err());
}

#[test]
fn to_string_literal_uses_single_quotes_for_normal_strings() {
    assert_eq!(to_string_literal("foo"), "'foo'");
}

#[test]
fn to_string_literal_uses_double_quotes_when_escaping_is_required() {
    assert_eq!(to_string_literal("foo'"), "\"foo'\"");
}
