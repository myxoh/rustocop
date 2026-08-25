// Ported from RuboCop 1.87.0:
// spec/rubocop/cop/corrector_spec.rb
// Spec SHA-256: 3dc154916600b672335d6f7dc37764d7675e6f1aa31623c032a47cb60246c583

use super::corrector::{CorrectionError, Corrector};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

macro_rules! rewrite {
    (|$buffer:ident, $corrector:ident| $body:block) => {{
        let source_buffer = SourceBuffer::new("true and false");
        let $buffer = &source_buffer;
        let mut $corrector = Corrector::new($buffer);
        $body
        $corrector.rewrite().unwrap()
    }};
}

#[test]
fn rewrites_with_the_upstream_corrector_operations() {
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.remove(SourceRange::new(buffer, 5, 8));
        }),
        "true  false"
    );
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.insert_before(SourceRange::new(buffer, 5, 8), ";nil ");
        }),
        "true ;nil and false"
    );
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.insert_after(SourceRange::new(buffer, 5, 8), " nil;");
        }),
        "true and nil; false"
    );
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.wrap(SourceRange::new(buffer, 5, 8), "(", ")");
        }),
        "true (and) false"
    );
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.replace(SourceRange::new(buffer, 5, 8), "or");
        }),
        "true or false"
    );
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.remove_preceding(SourceRange::new(buffer, 5, 8), 2);
        }),
        "truand false"
    );
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.remove_leading(SourceRange::new(buffer, 5, 8), 2);
        }),
        "true d false"
    );
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.remove_trailing(SourceRange::new(buffer, 5, 8), 2);
        }),
        "true a false"
    );
}

#[test]
fn swaps_disjoint_and_adjacent_ranges_like_rubocop() {
    assert_eq!(
        rewrite!(|buffer, corrector| {
            corrector.swap(
                SourceRange::new(buffer, 0, 4),
                SourceRange::new(buffer, 9, 14),
            );
        }),
        "false and true"
    );

    let buffer = SourceBuffer::new("leftRIGHT");
    let mut corrector = Corrector::new(&buffer);
    corrector.swap(
        SourceRange::new(&buffer, 0, 4),
        SourceRange::new(&buffer, 4, 9),
    );
    assert_eq!(corrector.rewrite().unwrap(), "RIGHTleft");
}

#[test]
fn character_ranges_are_safe_for_unicode_source() {
    let buffer = SourceBuffer::new("é and 🍒");
    let mut corrector = Corrector::new(&buffer);
    corrector.swap(
        SourceRange::new(&buffer, 0, 1),
        SourceRange::new(&buffer, 6, 7),
    );
    assert_eq!(corrector.rewrite().unwrap(), "🍒 and é");
}

#[test]
fn converts_and_validates_source_ranges() {
    let buffer = SourceBuffer::new("true");
    let corrector = Corrector::new(&buffer);
    let range = SourceRange::new(&buffer, 0, 4);
    assert_eq!(corrector.to_range(range), range);
    corrector.check_range_validity(range);
}

#[test]
fn imports_fragment_edits_at_character_offsets_and_rolls_back_invalid_imports() {
    let original = SourceBuffer::new("α = one\nβ = two\n");
    let fragment = SourceBuffer::new("two");
    let mut fragment_corrector = Corrector::new(&fragment);
    fragment_corrector.replace(SourceRange::new(&fragment, 0, 3), "three");

    let mut combined = Corrector::new(&original);
    combined.import(&fragment_corrector, 12).unwrap();
    assert_eq!(combined.rewrite().unwrap(), "α = one\nβ = three\n");

    let mut invalid = Corrector::new(&original);
    assert_eq!(
        invalid.import(&fragment_corrector, -1),
        Err(CorrectionError::InvalidRange)
    );
    assert!(invalid.is_empty());
}

#[test]
fn transaction_detects_clobbering_even_when_edits_arrive_out_of_order() {
    let buffer = SourceBuffer::new("abcdef");
    let mut corrector = Corrector::new(&buffer);
    corrector.transaction(|corrector| {
        corrector.replace(SourceRange::new(&buffer, 3, 6), "right");
        corrector.replace(SourceRange::new(&buffer, 1, 4), "left");
    });
    assert!(corrector.is_empty());
}
