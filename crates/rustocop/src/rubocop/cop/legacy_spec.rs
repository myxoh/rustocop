use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

use super::corrector::Corrector;
use super::legacy::{CorrectionsProxy, LegacyCorrector};

#[test]
fn proxy_applies_and_concatenates_legacy_callable_corrections() {
    let source = SourceBuffer::new("abc");
    let whole = SourceRange::new(&source, 0, 3);
    let mut legacy = LegacyCorrector::new(&source);
    {
        let mut proxy = legacy.corrections();
        assert!(proxy.is_empty());
        proxy.push(|corrector| corrector.wrap(whole, "(", ")"));
        proxy.concat([|corrector: &mut Corrector<'_, '_>| {
            let buffer = corrector.source_buffer();
            corrector.insert_after(SourceRange::new(buffer, 0, 3), "!");
        }]);
        assert!(!proxy.is_empty());
    }
    assert_eq!(legacy.rewrite().unwrap(), "(abc)!");
}

#[test]
fn legacy_corrector_can_start_from_an_existing_corrector() {
    let source = SourceBuffer::new("abc");
    let mut current = Corrector::new(&source);
    current.replace(SourceRange::new(&source, 1, 2), "B");
    let mut legacy = LegacyCorrector::with_corrector(&source, &current);
    {
        let mut proxy = legacy.corrections();
        let mut second = Corrector::new(&source);
        second.insert_after(SourceRange::new(&source, 2, 3), "!");
        proxy.concat_corrector(&second);
    }
    assert_eq!(legacy.rewrite().unwrap(), "aBc!");
}

#[test]
fn legacy_proxy_suppresses_clobbering_results() {
    assert_eq!(
        CorrectionsProxy::suppress_clobbering(Ok::<_, super::corrector::CorrectionError>(42)),
        Some(42)
    );
    assert_eq!(
        CorrectionsProxy::<'static, 'static, 'static>::suppress_clobbering::<()>(Err(
            super::corrector::CorrectionError::OverlappingReplacements(0..2, 1..3)
        )),
        None
    );
}
