// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/space_after_punctuation.rb
// Source SHA-256: 0831cab3af375e72e96846413e6c1383fba4e8cbdda5b12aeb4974e3bbc6bf49

use crate::rubocop::ast::processed_source::SourceToken;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MissingSpace {
    pub(crate) token_range: std::ops::Range<usize>,
    pub(crate) kind: String,
    pub(crate) message: String,
}

pub(crate) struct SpaceAfterPunctuation {
    pub(crate) space_style_before_rcurly: String,
}

impl SpaceAfterPunctuation {
    pub(crate) fn on_new_investigation(
        &self,
        tokens: &[SourceToken],
        kind: impl FnMut(&SourceToken, &SourceToken) -> Option<String>,
    ) -> Vec<MissingSpace> {
        self.each_missing_space(tokens, kind)
            .into_iter()
            .map(|(token, kind)| MissingSpace {
                token_range: token.range.clone(),
                message: format!("Space missing after {kind}."),
                kind,
            })
            .collect()
    }

    pub(crate) fn each_missing_space<'tokens>(
        &self,
        tokens: &'tokens [SourceToken],
        mut kind: impl FnMut(&SourceToken, &SourceToken) -> Option<String>,
    ) -> Vec<(&'tokens SourceToken, String)> {
        tokens
            .windows(2)
            .filter_map(|pair| {
                let token_kind = kind(&pair[0], &pair[1])?;
                (self.space_missing(&pair[0], &pair[1]) && self.space_required_before(&pair[1]))
                    .then_some((&pair[0], token_kind))
            })
            .collect()
    }

    pub(crate) fn space_missing(&self, token1: &SourceToken, token2: &SourceToken) -> bool {
        token1.line == token2.line && token2.column == token1.column + self.offset()
    }

    pub(crate) fn space_required_before(&self, token: &SourceToken) -> bool {
        !(self.allowed_type(token)
            || (token.right_curly_brace() && self.space_forbidden_before_rcurly()))
    }

    pub(crate) fn allowed_type(&self, token: &SourceToken) -> bool {
        matches!(token.kind, "tRPAREN" | "tRBRACK" | "tPIPE" | "tSTRING_DEND")
    }

    pub(crate) fn space_forbidden_before_rcurly(&self) -> bool {
        self.space_style_before_rcurly == "no_space"
    }

    pub(crate) fn offset(&self) -> usize {
        1
    }
}
