// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/ext/range.rb
// Source SHA-256: 4da173055bd1ccd62df33d2e001eb2be4232ed998eb56801ed9d6518945c1f85

use crate::rubocop::ast::source::SourceRange;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LineSpan {
    pub(crate) first: usize,
    pub(crate) last: usize,
    pub(crate) exclude_end: bool,
}

pub(crate) trait RangeExt {
    fn line_span(self, exclude_end: bool) -> LineSpan;
}

impl RangeExt for SourceRange<'_, '_> {
    fn line_span(self, exclude_end: bool) -> LineSpan {
        LineSpan {
            first: self.line(),
            last: self.last_line(),
            exclude_end,
        }
    }
}
