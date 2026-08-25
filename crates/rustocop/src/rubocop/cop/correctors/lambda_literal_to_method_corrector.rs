// RuboCop 1.87.0
// Source: lib/rubocop/cop/correctors/lambda_literal_to_method_corrector.rb
// Source SHA-256: e8194ba4595304674a38b17cf678182bd90fc1d1365c794c46d44f6d82c31b3a

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::cop::corrector::Corrector;

pub(crate) struct LambdaLiteralToMethodCorrector<'ast> {
    block_node: NodeRef<'ast>,
    method: NodeRef<'ast>,
    arguments: NodeRef<'ast>,
}

impl<'ast> LambdaLiteralToMethodCorrector<'ast> {
    pub(crate) fn block_node(&self) -> NodeRef<'ast> {
        self.block_node
    }

    pub(crate) fn method(&self) -> NodeRef<'ast> {
        self.method
    }

    pub(crate) fn arguments(&self) -> NodeRef<'ast> {
        self.arguments
    }

    pub(crate) fn initialize(block_node: NodeRef<'ast>) -> Option<Self> {
        Some(Self {
            block_node,
            method: block_node.send_node()?,
            arguments: block_node.arguments_node()?,
        })
    }

    pub(crate) fn call<'buffer, 'source>(&self, corrector: &mut Corrector<'buffer, 'source>) {
        self.remove_unparenthesized_whitespace(corrector);

        if self.block_node.kind() == "block" {
            self.insert_separating_space(corrector);
            self.remove_arguments(corrector);
        }

        self.replace_selector(corrector);
        self.replace_delimiters(corrector);
        self.insert_arguments(corrector);
    }

    pub(crate) fn remove_unparenthesized_whitespace<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
    ) {
        if self.arguments.empty() || self.arguments.parenthesized_call() {
            return;
        }
        self.remove_leading_whitespace(corrector);
        self.remove_trailing_whitespace(corrector);
    }

    pub(crate) fn insert_separating_space<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
    ) {
        if self.needs_separating_space() {
            if let Some(block_begin) = self.block_begin(corrector.source_buffer()) {
                corrector.insert_before(block_begin, " ");
            }
        }
    }

    pub(crate) fn replace_selector<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
    ) {
        if let Some(range) = self.method.source_range() {
            corrector.replace(
                SourceRange::new(corrector.source_buffer(), range.start, range.end),
                "lambda",
            );
        }
    }

    pub(crate) fn remove_arguments<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
    ) {
        if self.arguments.empty_and_without_delimiters() {
            return;
        }
        if let Some(range) = self.arguments.source_range() {
            corrector.remove(SourceRange::new(
                corrector.source_buffer(),
                range.start,
                range.end,
            ));
        }
    }

    pub(crate) fn insert_arguments<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
    ) {
        if self.arguments.empty() {
            return;
        }
        let Some(block_begin) = self.block_begin(corrector.source_buffer()) else {
            return;
        };
        corrector.insert_after(block_begin, format!(" |{}|", self.lambda_arg_string()));
    }

    pub(crate) fn remove_leading_whitespace<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
    ) {
        let Some(arguments) = self.arguments.source_range() else {
            return;
        };
        let Some(method) = self.method.source_range() else {
            return;
        };
        let arguments = SourceRange::new(corrector.source_buffer(), arguments.start, arguments.end);
        corrector.remove_preceding(arguments, arguments.begin_pos().saturating_sub(method.end));
    }

    pub(crate) fn remove_trailing_whitespace<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
    ) {
        let Some(arguments) = self.arguments.source_range() else {
            return;
        };
        let Some(block_begin) = self.block_begin(corrector.source_buffer()) else {
            return;
        };
        let size = block_begin
            .begin_pos()
            .saturating_sub(arguments.end)
            .saturating_sub(1);
        if size > 0 {
            corrector.remove_preceding(block_begin, size);
        }
    }

    pub(crate) fn replace_delimiters<'buffer, 'source>(
        &self,
        corrector: &mut Corrector<'buffer, 'source>,
    ) {
        if self.block_node.braces() || !self.arg_to_unparenthesized_call() {
            return;
        }
        let Some(block_begin) = self.block_begin(corrector.source_buffer()) else {
            return;
        };
        let Some(block_end) = self.block_end(corrector.source_buffer()) else {
            return;
        };
        if !self.separating_space(corrector.source_buffer()) {
            corrector.insert_after(block_begin, " ");
        }
        corrector.replace(block_begin, "{");
        corrector.replace(block_end, "}");
    }

    pub(crate) fn lambda_arg_string(&self) -> String {
        self.arguments
            .child_nodes()
            .into_iter()
            .filter_map(NodeRef::source)
            .collect::<Vec<_>>()
            .join(", ")
    }

    pub(crate) fn needs_separating_space(&self) -> bool {
        let Some(block_begin) = self
            .block_node
            .loc("begin")
            .map(|location| location.0.start)
        else {
            return false;
        };
        (self
            .arguments_end_pos()
            .is_some_and(|end| block_begin == end)
            && self.arguments_begin_pos() == self.selector_end())
            || self.selector_end() == Some(block_begin)
    }

    pub(crate) fn arguments_end_pos(&self) -> Option<usize> {
        self.arguments.loc("end").map(|location| location.0.end)
    }

    pub(crate) fn arguments_begin_pos(&self) -> Option<usize> {
        self.arguments.loc("begin").map(|location| location.0.start)
    }

    pub(crate) fn block_end<'buffer, 'source>(
        &self,
        buffer: &'buffer SourceBuffer<'source>,
    ) -> Option<SourceRange<'buffer, 'source>> {
        self.block_node
            .loc("end")
            .map(|location| SourceRange::new(buffer, location.0.start, location.0.end))
    }

    pub(crate) fn block_begin<'buffer, 'source>(
        &self,
        buffer: &'buffer SourceBuffer<'source>,
    ) -> Option<SourceRange<'buffer, 'source>> {
        self.block_node
            .loc("begin")
            .map(|location| SourceRange::new(buffer, location.0.start, location.0.end))
    }

    pub(crate) fn selector_end(&self) -> Option<usize> {
        self.method.loc("selector").map(|location| location.0.end)
    }

    pub(crate) fn arg_to_unparenthesized_call(&self) -> bool {
        let mut current_node = self.block_node;
        let mut parent = current_node.parent();
        if parent.is_some_and(|node| node.kind() == "pair") {
            current_node = parent.and_then(NodeRef::parent).unwrap_or(current_node);
            parent = current_node.parent();
        }
        parent.is_some_and(|node| {
            node.kind() == "send"
                && !node.parenthesized_call()
                && current_node.sibling_index().is_some_and(|index| index > 1)
        })
    }

    pub(crate) fn separating_space(&self, buffer: &SourceBuffer<'_>) -> bool {
        self.block_node
            .loc("begin")
            .and_then(|location| buffer.character(location.0.start.saturating_add(2)))
            .is_some_and(char::is_whitespace)
    }
}

#[cfg(test)]
mod spec;
