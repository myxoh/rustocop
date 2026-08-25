// RuboCop 1.87.0
// Source: lib/rubocop/cop/legacy/corrections_proxy.rb
// Source SHA-256: 297967a243e5472efd1515efd0c5d794eecffecc728be9305eb05ca003dab97c
// Source: lib/rubocop/cop/legacy/corrector.rb
// Source SHA-256: b5ede8a950e40a80b28e01c98541b7e62a0a52fa2cb49fee4fec5eb6138f949f

use crate::rubocop::ast::source::SourceBuffer;

use super::corrector::{CorrectionError, Corrector};

pub(crate) struct CorrectionsProxy<'corrector, 'buffer, 'source> {
    corrector: &'corrector mut Corrector<'buffer, 'source>,
}

impl<'corrector, 'buffer, 'source> CorrectionsProxy<'corrector, 'buffer, 'source> {
    pub(crate) fn new(corrector: &'corrector mut Corrector<'buffer, 'source>) -> Self {
        Self { corrector }
    }

    pub(crate) fn corrector(&mut self) -> &mut Corrector<'buffer, 'source> {
        self.corrector
    }

    pub(crate) fn push<F>(&mut self, callable: F)
    where
        F: FnOnce(&mut Corrector<'buffer, 'source>),
    {
        self.corrector.transaction(callable);
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.corrector.is_empty()
    }

    pub(crate) fn concat_corrector(&mut self, corrections: &Corrector<'buffer, 'source>) {
        self.corrector.merge(corrections);
    }

    pub(crate) fn concat<F, I>(&mut self, corrections: I)
    where
        F: FnOnce(&mut Corrector<'buffer, 'source>),
        I: IntoIterator<Item = F>,
    {
        for correction in corrections {
            self.push(correction);
        }
    }

    pub(crate) fn suppress_clobbering<T>(result: Result<T, CorrectionError>) -> Option<T> {
        match result {
            Ok(value) => Some(value),
            Err(
                CorrectionError::DifferentReplacements(_)
                | CorrectionError::OverlappingReplacements(_, _),
            ) => None,
            Err(CorrectionError::InvalidRange) => None,
        }
    }
}

pub(crate) struct LegacyCorrector<'buffer, 'source> {
    corrector: Corrector<'buffer, 'source>,
}

impl<'buffer, 'source> LegacyCorrector<'buffer, 'source> {
    pub(crate) fn new(source: &'buffer SourceBuffer<'source>) -> Self {
        Self {
            corrector: Corrector::new(source),
        }
    }

    pub(crate) fn with_corrector(
        source: &'buffer SourceBuffer<'source>,
        corrections: &Corrector<'buffer, 'source>,
    ) -> Self {
        let mut corrector = Corrector::new(source);
        corrector.merge(corrections);
        Self { corrector }
    }

    pub(crate) fn corrections(&mut self) -> CorrectionsProxy<'_, 'buffer, 'source> {
        CorrectionsProxy::new(&mut self.corrector)
    }

    pub(crate) fn rewrite(self) -> Result<String, CorrectionError> {
        self.corrector.rewrite()
    }
}
