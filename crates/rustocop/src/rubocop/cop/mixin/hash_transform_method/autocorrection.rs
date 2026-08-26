// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/hash_transform_method/autocorrection.rb
// Source SHA-256: 043ec952bfceb375fa1fee240b92c7e9b6c69ba331fb5819db25d1a8af6fc450

use std::ops::Range;

use super::{BadTransformKind, MatchData, TransformExpression};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct BlockGeometry {
    pub(crate) source: String,
    pub(crate) expression: Range<usize>,
    pub(crate) selector: Range<usize>,
    pub(crate) send_end: Option<usize>,
    pub(crate) arguments: Range<usize>,
    pub(crate) body: Range<usize>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Autocorrection {
    pub(crate) kind: BadTransformKind,
    pub(crate) block: BlockGeometry,
    pub(crate) leading: usize,
    pub(crate) trailing: usize,
}

impl Autocorrection {
    pub(crate) fn kind(&self) -> BadTransformKind {
        self.kind
    }
    pub(crate) fn block_node(&self) -> &BlockGeometry {
        &self.block
    }
    pub(crate) fn leading(&self) -> usize {
        self.leading
    }
    pub(crate) fn trailing(&self) -> usize {
        self.trailing
    }

    pub(crate) fn from_each_with_object(block: BlockGeometry) -> Self {
        Self {
            kind: BadTransformKind::EachWithObject,
            block,
            leading: 0,
            trailing: 0,
        }
    }

    pub(crate) fn from_hash_brackets_map(block: BlockGeometry) -> Self {
        Self {
            kind: BadTransformKind::HashBracketsMap,
            block,
            leading: "Hash[".len(),
            trailing: "]".len(),
        }
    }

    pub(crate) fn from_map_to_h(block: BlockGeometry, trailing: usize) -> Self {
        Self {
            kind: BadTransformKind::MapToH,
            block,
            leading: 0,
            trailing,
        }
    }

    pub(crate) fn from_to_h(block: BlockGeometry) -> Self {
        Self {
            kind: BadTransformKind::ToH,
            block,
            leading: 0,
            trailing: 0,
        }
    }

    pub(crate) fn strip_prefix_and_suffix(&self) -> (Range<usize>, Range<usize>) {
        let expression = &self.block.expression;
        (
            expression.start..expression.start + self.leading,
            expression.end.saturating_sub(self.trailing)..expression.end,
        )
    }

    pub(crate) fn set_new_method_name(&self, new_method_name: &str) -> (Range<usize>, String) {
        let end = self.block.send_end.unwrap_or(self.block.selector.end);
        (self.block.selector.start..end, new_method_name.to_owned())
    }

    pub(crate) fn set_new_arg_name(&self, transformed_argname: &str) -> (Range<usize>, String) {
        (
            self.block.arguments.clone(),
            format!("|{transformed_argname}|"),
        )
    }

    pub(crate) fn set_new_body_expression(
        &self,
        transforming_body_expr: &TransformExpression,
    ) -> (Range<usize>, String) {
        let body = if transforming_body_expr.hash_type && !transforming_body_expr.braces {
            format!("{{ {} }}", transforming_body_expr.source)
        } else {
            transforming_body_expr.source.clone()
        };
        (self.block.body.clone(), body)
    }

    pub(crate) fn apply(
        &self,
        new_method_name: &str,
        transformed_argname: &str,
        transforming_body_expr: &TransformExpression,
    ) -> String {
        let (leading, trailing) = self.strip_prefix_and_suffix();
        let mut edits = vec![
            (leading, String::new()),
            (trailing, String::new()),
            self.set_new_method_name(new_method_name),
            self.set_new_arg_name(transformed_argname),
            self.set_new_body_expression(transforming_body_expr),
        ];
        edits.retain(|(range, _)| !range.is_empty());
        edits.sort_by_key(|(range, _)| (range.start, range.end));
        let mut source = self.block.source.clone();
        for (range, replacement) in edits.into_iter().rev() {
            source.replace_range(range, &replacement);
        }
        source
    }

    pub(crate) fn match_data(self, captures: super::Captures) -> MatchData {
        MatchData {
            kind: self.kind,
            captures,
            correction: self,
        }
    }
}
