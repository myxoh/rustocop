use super::FrozenStringLiteral;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn leading_magic_comment_controls_literal_frozen_state() {
    for (value, enabled) in [("true", true), ("false", false)] {
        let source = format!("# frozen_string_literal: {value}\n\"text\"");
        let parsed = ProcessedSource::new(&source, 3.4, None, ParserEngine::Prism).unwrap();
        let check = FrozenStringLiteral::new(&parsed, 3.4, None);
        assert_eq!(check.frozen_string_literals_enabled(), enabled);
        assert_eq!(check.frozen_string_literals_disabled(), !enabled);
        assert!(check.frozen_string_literal_specified());
        assert!(check.frozen_string_literal_comment_exists());
        assert_eq!(check.frozen_string_literal(parsed.ast().unwrap()), enabled);
    }
}

#[test]
fn configured_default_applies_only_without_a_magic_comment() {
    let parsed = ProcessedSource::new("\"text\"", 3.4, None, ParserEngine::Prism).unwrap();
    let check = FrozenStringLiteral::new(&parsed, 3.4, Some(true));
    assert!(check.frozen_string_literals_enabled());
    assert!(check.uninterpolated_string(parsed.ast().unwrap()));
}
