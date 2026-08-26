// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/method_complexity.rb
// Source SHA-256: 6d30cc815605b57f14bc11e488c5c8b9dc871d3a81b363169a72ad2131c8376a

use std::ops::Range;

use regex::Regex;

use crate::rubocop::ast::node::core::NodeRef;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ComplexityOffense {
    pub(crate) location: Range<usize>,
    pub(crate) message: String,
    pub(crate) complexity: usize,
}

pub(crate) struct MethodComplexity<'config> {
    max: usize,
    counted_nodes: &'config [&'config str],
    allowed_methods: &'config [&'config str],
    allowed_patterns: &'config [Regex],
    message_template: &'config str,
    lsp_enabled: bool,
    complexity_score_for: fn(NodeRef<'_>) -> usize,
}

impl<'config> MethodComplexity<'config> {
    pub(crate) fn new(
        max: usize,
        counted_nodes: &'config [&'config str],
        allowed_methods: &'config [&'config str],
        allowed_patterns: &'config [Regex],
        message_template: &'config str,
        lsp_enabled: bool,
        complexity_score_for: fn(NodeRef<'_>) -> usize,
    ) -> Self {
        Self {
            max,
            counted_nodes,
            allowed_methods,
            allowed_patterns,
            message_template,
            lsp_enabled,
            complexity_score_for,
        }
    }

    pub(crate) fn on_def(&self, node: NodeRef<'_>) -> Option<ComplexityOffense> {
        let name = node.method_name()?;
        if self.allowed(name) {
            None
        } else {
            self.check_complexity(node, name)
        }
    }

    pub(crate) fn on_defs(&self, node: NodeRef<'_>) -> Option<ComplexityOffense> {
        self.on_def(node)
    }

    pub(crate) fn on_block(&self, node: NodeRef<'_>) -> Option<ComplexityOffense> {
        let name = self.define_method(node)?;
        if self.allowed(name) {
            None
        } else {
            self.check_complexity(node, name)
        }
    }

    pub(crate) fn on_numblock(&self, node: NodeRef<'_>) -> Option<ComplexityOffense> {
        self.on_block(node)
    }

    pub(crate) fn on_itblock(&self, node: NodeRef<'_>) -> Option<ComplexityOffense> {
        self.on_block(node)
    }

    pub(crate) fn define_method<'ast>(&self, node: NodeRef<'ast>) -> Option<&'ast str> {
        let send = matches!(node.kind(), "block" | "numblock" | "itblock")
            .then(|| node.send_node())
            .flatten()?;
        if send.receiver().is_some() || send.method_name() != Some("define_method") {
            return None;
        }
        let name = send.arguments().first().copied()?;
        matches!(name.kind(), "sym" | "str")
            .then(|| name.string_child(0).or_else(|| name.symbol_child(0)))
            .flatten()
    }

    pub(crate) fn check_complexity(
        &self,
        node: NodeRef<'_>,
        method_name: &str,
    ) -> Option<ComplexityOffense> {
        let body = node.body()?;
        let complexity = self.complexity(body);
        if complexity <= self.max {
            return None;
        }
        let message = self
            .message_template
            .replace("%<method>s", method_name)
            .replace("%<complexity>d", &complexity.to_string())
            .replace("%<abc_vector>s", "")
            .replace("%<max>d", &self.max.to_string());
        Some(ComplexityOffense {
            location: self.location(node).unwrap_or_default(),
            message,
            complexity,
        })
    }

    pub(crate) fn complexity(&self, body: NodeRef<'_>) -> usize {
        1 + body
            .each_node(&[])
            .into_iter()
            .filter(|node| self.counted_nodes.contains(&node.kind()))
            .map(|node| (self.complexity_score_for)(node))
            .sum::<usize>()
    }

    pub(crate) fn location(&self, node: NodeRef<'_>) -> Option<Range<usize>> {
        let source = node.source_range()?;
        if self.lsp_enabled {
            let end = node
                .loc("name")
                .or_else(|| node.loc("begin"))
                .map_or(source.end, |location| location.0.end);
            Some(source.start..end)
        } else {
            Some(source)
        }
    }

    fn allowed(&self, name: &str) -> bool {
        self.allowed_methods.contains(&name)
            || self
                .allowed_patterns
                .iter()
                .any(|pattern| pattern.is_match(name))
    }
}

#[cfg(test)]
mod spec;
