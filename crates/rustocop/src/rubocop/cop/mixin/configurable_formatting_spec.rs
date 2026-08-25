use super::configurable_formatting::*;
use regex::Regex;
use std::collections::BTreeMap;

fn formatting() -> ConfigurableFormatting {
    ConfigurableFormatting {
        style: "snake".into(),
        formats: BTreeMap::from([
            ("camel".into(), Regex::new(r"^[A-Z]").unwrap()),
            ("snake".into(), Regex::new(r"^[a-z]+(_[a-z]+)*$").unwrap()),
        ]),
    }
}

#[test]
fn correct_opposing_and_unrecognized_names_have_distinct_detections() {
    let node = FormattingNode::default();
    assert_eq!(
        formatting().check_name(&node, "valid_name").1,
        StyleDetection::Correct
    );
    assert_eq!(
        formatting().check_name(&node, "Camel").1,
        StyleDetection::Unexpected("camel".into())
    );
    assert_eq!(
        formatting().check_name(&node, "bad-name").1,
        StyleDetection::Unrecognized
    );
}

#[test]
fn singleton_class_emitters_are_valid_for_every_configured_style() {
    let node = FormattingNode {
        has_parent: true,
        singleton_definition: true,
        enclosing_class_names: vec!["Widget".into()],
    };
    assert!(formatting().class_emitter_method(&node, "Widget"));
    assert!(formatting().valid_name(&node, "Widget", "snake"));
    assert!(!formatting().class_emitter_method(&FormattingNode::default(), "Widget"));
}
