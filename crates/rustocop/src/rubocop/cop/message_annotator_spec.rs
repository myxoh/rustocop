// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/message_annotator_spec.rb
// Spec SHA-256: 74047ed0fd2950bdef9dbc2f18c5d359d588dbcb3668ca81ac10758777ccc500

use std::collections::HashMap;

use super::message_annotator::{
    CopMessageConfig, MessageAnnotator, MessageConfig, MessageOptions, Urls,
};

#[test]
fn annotates_only_the_enabled_parts() {
    let config = MessageConfig::default();
    let cop = CopMessageConfig::default();
    let options = MessageOptions::default();
    assert_eq!(
        MessageAnnotator::new(&config, "Cop/Cop", &cop, &options).annotate("message"),
        "message"
    );

    let cop = CopMessageConfig {
        details: Some("my cop details".into()),
        style_guide: Some("http://example.org/styleguide".into()),
        ..CopMessageConfig::default()
    };
    let options = MessageOptions {
        extra_details: true,
        display_cop_names: Some(true),
        display_style_guide: true,
        ..MessageOptions::default()
    };
    assert_eq!(
        MessageAnnotator::new(&config, "Cop/Cop", &cop, &options).annotate("message"),
        "Cop/Cop: message my cop details (http://example.org/styleguide)"
    );
}

#[test]
fn resolves_department_nested_relative_absolute_and_fragment_urls() {
    let config = MessageConfig {
        all_cops: HashMap::from([(
            "StyleGuideBaseURL".into(),
            "https://github.com/rubocop/ruby-style-guide/".into(),
        )]),
        departments: HashMap::from([(
            "Foo/Bar".into(),
            HashMap::from([("StyleGuideBaseURL".into(), "http://foo.example.org".into())]),
        )]),
    };
    let options = MessageOptions {
        display_style_guide: true,
        ..MessageOptions::default()
    };
    for (name, style, expected) in [
        ("Foo/Bar/Cop", "#target", "http://foo.example.org#target"),
        (
            "Cop/Cop",
            "../rails-style-guide#target",
            "https://github.com/rubocop/rails-style-guide#target",
        ),
        (
            "Cop/Cop",
            "http://other.org#absolute",
            "http://other.org#absolute",
        ),
    ] {
        let cop = CopMessageConfig {
            style_guide: Some(style.into()),
            ..CopMessageConfig::default()
        };
        assert_eq!(
            MessageAnnotator::new(&config, name, &cop, &options).urls(),
            vec![expected]
        );
    }
}

#[test]
fn preserves_plural_and_legacy_reference_order_and_blank_filtering() {
    let config = MessageConfig::default();
    let cop = CopMessageConfig {
        references: Some(Urls::Many(vec!["one".into(), "".into(), "two".into()])),
        reference: Some(Urls::One("one".into())),
        ..CopMessageConfig::default()
    };
    assert_eq!(
        MessageAnnotator::new(&config, "Cop/Cop", &cop, &MessageOptions::default()).urls(),
        ["one", "two", "one"]
    );
}

#[test]
fn debug_forces_names_and_json_suppresses_configured_names() {
    let config = MessageConfig {
        all_cops: HashMap::from([("DisplayCopNames".into(), "true".into())]),
        ..MessageConfig::default()
    };
    let cop = CopMessageConfig::default();
    let json = MessageOptions {
        format: Some("json".into()),
        ..MessageOptions::default()
    };
    assert_eq!(
        MessageAnnotator::new(&config, "Cop/Cop", &cop, &json).annotate("message"),
        "message"
    );
    let debug = MessageOptions {
        debug: true,
        display_cop_names: Some(false),
        ..MessageOptions::default()
    };
    assert_eq!(
        MessageAnnotator::new(&config, "Cop/Cop", &cop, &debug).annotate("message"),
        "Cop/Cop: message"
    );
    assert!(MessageAnnotator::new(&config, "Cop/Cop", &cop, &debug).debug());
}
