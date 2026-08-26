// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/token.rb
// Source SHA-256: 015f6dd257c7bdfa55d1a590002fa78451b38353a056e57b09d618d3538397c2

use std::fmt;

use super::source::SourceRange;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Token<'buffer, 'source> {
    pos: SourceRange<'buffer, 'source>,
    kind: &'static str,
    text: String,
}

impl<'buffer, 'source> Token<'buffer, 'source> {
    pub(crate) fn initialize(
        pos: SourceRange<'buffer, 'source>,
        kind: &'static str,
        text: impl ToString,
    ) -> Self {
        Self::new(pos, kind, text)
    }

    pub(crate) fn from_parser_token(
        parser_token: (&'static str, (impl ToString, SourceRange<'buffer, 'source>)),
    ) -> Self {
        let (kind, (text, range)) = parser_token;
        Self::new(range, kind, text)
    }

    pub(crate) fn new(
        pos: SourceRange<'buffer, 'source>,
        kind: &'static str,
        text: impl ToString,
    ) -> Self {
        Self {
            pos,
            kind,
            text: text.to_string(),
        }
    }

    pub(crate) fn pos(&self) -> SourceRange<'buffer, 'source> {
        self.pos
    }

    pub(crate) fn kind(&self) -> &'static str {
        self.kind
    }

    pub(crate) fn token_type(&self) -> &'static str {
        self.kind
    }

    pub(crate) fn text(&self) -> &str {
        &self.text
    }

    pub(crate) fn display(&self) -> String {
        self.to_string()
    }

    pub(crate) fn line(&self) -> usize {
        self.pos.line()
    }

    pub(crate) fn column(&self) -> usize {
        self.pos.column()
    }

    pub(crate) fn begin_pos(&self) -> usize {
        self.pos.begin_pos()
    }

    pub(crate) fn end_pos(&self) -> usize {
        self.pos.end_pos()
    }

    pub(crate) fn space_after(&self) -> bool {
        self.pos
            .buffer()
            .character(self.end_pos())
            .is_some_and(ruby_space)
    }

    pub(crate) fn space_before(&self) -> bool {
        self.begin_pos()
            .checked_sub(1)
            .and_then(|position| self.pos.buffer().character(position))
            .is_some_and(ruby_space)
    }

    pub(crate) fn comment(&self) -> bool {
        self.kind == "tCOMMENT"
    }

    pub(crate) fn semicolon(&self) -> bool {
        self.kind == "tSEMI"
    }

    pub(crate) fn left_array_bracket(&self) -> bool {
        self.kind == "tLBRACK"
    }

    pub(crate) fn left_ref_bracket(&self) -> bool {
        self.kind == "tLBRACK2"
    }

    pub(crate) fn left_bracket(&self) -> bool {
        matches!(self.kind, "tLBRACK" | "tLBRACK2")
    }

    pub(crate) fn right_bracket(&self) -> bool {
        self.kind == "tRBRACK"
    }

    pub(crate) fn left_brace(&self) -> bool {
        self.kind == "tLBRACE"
    }

    pub(crate) fn left_curly_brace(&self) -> bool {
        matches!(self.kind, "tLCURLY" | "tLAMBEG")
    }

    pub(crate) fn right_curly_brace(&self) -> bool {
        self.kind == "tRCURLY"
    }

    pub(crate) fn left_parens(&self) -> bool {
        matches!(self.kind, "tLPAREN" | "tLPAREN2")
    }

    pub(crate) fn right_parens(&self) -> bool {
        self.kind == "tRPAREN"
    }

    pub(crate) fn comma(&self) -> bool {
        self.kind == "tCOMMA"
    }

    pub(crate) fn dot(&self) -> bool {
        self.kind == "tDOT"
    }

    pub(crate) fn regexp_dots(&self) -> bool {
        matches!(self.kind, "tDOT2" | "tDOT3")
    }

    pub(crate) fn rescue_modifier(&self) -> bool {
        self.kind == "kRESCUE_MOD"
    }

    pub(crate) fn end(&self) -> bool {
        self.kind == "kEND"
    }

    pub(crate) fn equal_sign(&self) -> bool {
        matches!(self.kind, "tEQL" | "tOP_ASGN")
    }

    pub(crate) fn new_line(&self) -> bool {
        self.kind == "tNL"
    }
}

impl fmt::Display for Token<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "[[{}, {}], {}, {:?}]",
            self.line(),
            self.column(),
            self.kind,
            self.text
        )
    }
}

fn ruby_space(character: char) -> bool {
    matches!(
        character,
        ' ' | '\t' | '\r' | '\n' | '\u{000b}' | '\u{000c}'
    )
}
