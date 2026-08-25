// Ported from rubocop-ast 1.49.1:
// spec/rubocop/ast/ext/range_spec.rb
// Spec SHA-256: 75bd01af24daf73ef60e5d32e61831de44739b1e4877ebfeab69dfe722470e8b

use super::range::{LineSpan, RangeExt};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

#[test]
fn returns_inclusive_and_exclusive_line_spans() {
    let buffer = SourceBuffer::new("[\n  1,\n  2\n]\n");
    assert_eq!(
        SourceRange::new(&buffer, 0, 1).line_span(false),
        LineSpan {
            first: 1,
            last: 1,
            exclude_end: false,
        }
    );
    assert_eq!(
        SourceRange::new(&buffer, 0, 12).line_span(true),
        LineSpan {
            first: 1,
            last: 4,
            exclude_end: true,
        }
    );
}
