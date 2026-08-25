use super::helpers::*;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn receivers_and_array_syntax_follow_nested_dispatch_rules() {
    let constant = Receiver {
        receiver: None,
        constant: true,
        send: false,
        method_name: String::new(),
        source: "File".into(),
    };
    let join = Receiver {
        receiver: Some(Box::new(constant)),
        constant: false,
        send: true,
        method_name: "join".into(),
        source: "File.join".into(),
    };
    let expand = Receiver {
        receiver: Some(Box::new(join)),
        constant: false,
        send: true,
        method_name: "expand_path".into(),
        source: "File.join.expand_path".into(),
    };
    assert_eq!(receiver_name(&expand), "File.join");
    assert!(allowed_receiver(&expand, &["File.join".into()]));
    assert!(bracketed_array_of(true, &["str", "str"], "str"));
    assert!(!bracketed_array_of(true, &[], "str"));
    assert!(support_autocorrect());
}

#[test]
fn duplication_preserves_first_group_and_member_order() {
    let items = ["a", "b", "a", "c", "b"];
    assert!(duplicates_exist(&items));
    assert_eq!(grouped_duplicates(&items), [vec!["a", "a"], vec!["b", "b"]]);
    assert_eq!(duplicates(&items), ["a", "a", "b", "b"]);
    assert_eq!(consecutive_duplicates(&items), ["a", "b"]);
}

#[test]
fn literal_and_node_pattern_predicates_match_source_shapes() {
    assert!(gem_declaration(true, "gem", true));
    assert!(!gem_declaration(false, "gem", true));
    assert_eq!(integer_part("-12.3e4"), "12");
    assert!(rational_literal(true, "/", true));
    assert!(empty_condition("begin", 0));
    assert!(safe_assignment("begin", 1, true, false));
    assert!(!safe_assignment("begin", 2, true, false));
}

#[test]
fn preferred_methods_remove_reversed_defaults() {
    let default = [("collect".into(), "map".into())];
    let merged = [
        ("collect".into(), "map".into()),
        ("map".into(), "collect".into()),
    ];
    let preferences = preferred_methods(&default, &merged);
    assert_eq!(preferences.get("map").map(String::as_str), Some("collect"));
    assert_eq!(preferred_method(&preferences, "map"), Some("collect"));
    assert!(!preferences.contains_key("collect"));
    assert_eq!(default_cop_config(&default), &default);
    assert!(safe_assignment_allowed(true));
}

#[test]
fn body_line_and_first_part_helpers_use_source_backed_node_ranges() {
    assert!(body_on_first_line(2, 2));
    let processed =
        ProcessedSource::new("(first; second)", 3.4, None, ParserEngine::Prism).unwrap();
    let node = processed.ast().unwrap();
    assert_eq!(
        first_part_of(node),
        node.first_node().unwrap().source_range()
    );
}

#[test]
fn parentheses_percent_and_quote_branches_match_rubocop() {
    assert!(parens_required("foo!bar", 3..4));
    assert!(!parens_required(" ! ", 1..2));
    assert!(percent_literal(Some("%w[")));
    assert_eq!(percent_literal_type("%w["), "%w");
    assert!(process_percent_literal(Some("%w["), &["%w"]));
    assert!(wrong_quotes("\"plain\"", true));
    assert!(!wrong_quotes("\"it's\"", true));
    assert_eq!(preferred_string_literal(true), "\"\"");
    assert!(enforce_double_quotes("double_quotes"));
    assert!(!enforce_double_quotes("single_quotes"));
    assert_eq!(string_literals_config("single_quotes"), "single_quotes");
    assert_eq!(
        string_help_on_str(false, false, true),
        StringHelpAction::Skip
    );
    assert_eq!(
        string_help_on_str(true, true, true),
        StringHelpAction::Ignore
    );
    assert_eq!(
        string_help_on_str(true, false, true),
        StringHelpAction::Offense
    );
    assert_eq!(string_help_on_regexp(), StringHelpAction::Ignore);
}

#[test]
fn trailing_body_requires_a_multiline_node_with_same_first_line() {
    assert!(trailing_body(true, true, 2, 2));
    assert!(!trailing_body(true, false, 2, 2));
    assert!(!trailing_body(false, true, 2, 2));
}

#[test]
fn multiline_and_match_range_helpers_preserve_line_and_character_offsets() {
    let lines = [
        LineSpan {
            first_line: 1,
            last_line: 1,
        },
        LineSpan {
            first_line: 1,
            last_line: 2,
        },
        LineSpan {
            first_line: 3,
            last_line: 3,
        },
    ];
    assert_eq!(missing_element_line_breaks(&lines, false), [1]);
    let regex = regex::Regex::new(r"x=(\p{L}+)").unwrap();
    assert_eq!(
        each_match_range("é x=β", 10, &regex),
        std::iter::once(14..15).collect::<Vec<_>>()
    );
}

#[test]
fn remaining_small_mixins_match_their_node_pattern_guards() {
    let methods = nil_methods(&["custom".into()]);
    assert!(methods.contains(&"nil?".into()));
    assert!(methods.ends_with(&["to_d".into(), "custom".into()]));
    assert!(empty_arguments(true, true, false));
    assert!(!empty_arguments(true, true, true));
    assert!(non_public_modifier(true, "private", true));
    assert!(non_public(false, "protected"));
    assert!(dig("dig", &["sym", "int"]));
    assert!(dig_chain_enabled(true));
    assert!(!dig("dig", &["hash"]));
    assert!(single_argument_dig("dig", &["sym"]));
    assert_eq!(other_stdlib_methods(), ["to_d"]);
    assert!(check_negative_conditional(false, true, false, false));
    assert!(!check_negative_conditional(false, true, true, true));
    assert!(on_normal_if_unless(false, false));
}
