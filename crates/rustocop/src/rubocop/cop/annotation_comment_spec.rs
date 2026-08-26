// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/annotation_comment_spec.rb
// Spec SHA-256: f3a274ea61e8fe18463a6d50784cf33c1bac201a3dd04d881d29e39e7e685ad0

use super::annotation_comment::AnnotationComment;

fn annotation(text: &str) -> AnnotationComment {
    AnnotationComment::new(
        &format!("# {text}"),
        0,
        &["TODO".into(), "FOR LATER".into(), "FIXME".into()],
    )
}

#[test]
fn annotation_with_colon() {
    assert_eq!(annotation("TODO: note").annotation(), Some(true));
}
#[test]
fn annotation_with_space() {
    assert_eq!(annotation("TODO note").annotation(), Some(true));
}
#[test]
fn annotation_with_lowercase_keyword() {
    assert_eq!(annotation("todo: note").annotation(), Some(true));
}
#[test]
fn annotation_with_multiple_words() {
    assert_eq!(annotation("FOR LATER: note").annotation(), Some(true));
}
#[test]
fn non_keyword_is_nil() {
    assert_eq!(annotation("SOMETHING: note").annotation(), None);
}
#[test]
fn first_word_of_sentence_is_not_annotation() {
    assert_eq!(annotation("Todo in the future").annotation(), Some(false));
}
#[test]
fn keyword_prefix_is_nil() {
    assert_eq!(annotation("TODO2").annotation(), None);
}

#[test]
fn correct_with_and_without_colons_covers_every_shared_example() {
    for text in ["TODO: text", "FIXME: text", "FOR LATER: text"] {
        assert!(annotation(text).correct(true), "{text}");
    }
    for text in [
        "TODO: ",
        "TODO ",
        "TODO",
        "TODOtext",
        "TODO:text",
        "TODO2: text",
        "TODO text",
        "todo text",
        "UPDATE: text",
        "UPDATE text",
        "FOR LATER text",
    ] {
        assert!(!annotation(text).correct(true), "{text}");
    }
    for text in ["TODO text", "FIXME text", "FOR LATER  text"] {
        assert!(annotation(text).correct(false), "{text}");
    }
    for text in [
        "TODO: ",
        "TODO ",
        "TODO",
        "TODOtext",
        "TODO:text",
        "TODO2 text",
        "TODO: text",
        "todo text",
        "UPDATE: text",
        "UPDATE text",
        "FOR LATER: text",
    ] {
        assert!(!annotation(text).correct(false), "{text}");
    }
}

#[test]
fn longer_duplicate_keywords_match_regardless_of_configuration_order_and_bounds_are_exact() {
    for keywords in [
        vec!["TODO LATER".into(), "TODO".into()],
        vec!["TODO".into(), "TODO LATER".into()],
    ] {
        let short = AnnotationComment::new("# TODO: text", 10, &keywords);
        let long = AnnotationComment::new("# TODO LATER: text", 10, &keywords);
        assert!(short.correct(true));
        assert!(long.correct(true));
        assert_eq!(long.bounds(), Some((12, 24)));
        assert_eq!(long.margin(), Some("# "));
        assert_eq!(long.keyword(), Some("TODO LATER"));
        assert_eq!(long.colon(), Some(":"));
        assert_eq!(long.space(), Some(" "));
        assert_eq!(long.note(), Some("text"));
    }
}

#[test]
fn keyword_appearance_requires_a_keyword_and_separator() {
    assert!(annotation("TODO: note").keyword_appearance());
    assert!(annotation("TODO note").keyword_appearance());
    assert!(!annotation("ordinary note").keyword_appearance());
}
