use ruby_prism::{CallNode, Location, Node};

/// An allocation-free DSL for the structural part of call-based cops.
/// Semantic conditions stay in the cop beside the resulting diagnostic.
pub(super) struct CallMatcher<'call, 'pr> {
    call: &'call CallNode<'pr>,
    matches: bool,
}

/// Starts a structural call match without exposing the matcher type at every
/// call site.
pub(super) fn match_call<'call, 'pr>(call: &'call CallNode<'pr>) -> CallMatcher<'call, 'pr> {
    CallMatcher::new(call)
}

impl<'call, 'pr> CallMatcher<'call, 'pr> {
    pub(super) fn new(call: &'call CallNode<'pr>) -> Self {
        Self {
            call,
            matches: true,
        }
    }

    pub(super) fn named(mut self, name: &[u8]) -> Self {
        self.matches &= call_name(self.call) == name;
        self
    }

    pub(super) fn named_any(mut self, names: &[&[u8]]) -> Self {
        self.matches &= names.contains(&call_name(self.call));
        self
    }

    pub(super) fn on_root_constant(mut self, name: &[u8]) -> Self {
        self.matches &= root_constant(self.call.receiver(), name);
        self
    }

    pub(super) fn on_constant_read(mut self, name: &[u8]) -> Self {
        self.matches &= constant_read(self.call.receiver(), name);
        self
    }

    pub(super) fn on_implicit_or_root_constant(mut self, name: &[u8]) -> Self {
        self.matches &= self.call.receiver().is_none() || root_constant(self.call.receiver(), name);
        self
    }

    pub(super) fn without_receiver(mut self) -> Self {
        self.matches &= self.call.receiver().is_none();
        self
    }

    pub(super) fn with_receiver(mut self) -> Self {
        self.matches &= self.call.receiver().is_some();
        self
    }

    pub(super) fn with_receiver_matching(
        mut self,
        predicate: impl FnOnce(Option<Node<'pr>>) -> bool,
    ) -> Self {
        self.matches &= predicate(self.call.receiver());
        self
    }

    pub(super) fn without_arguments(mut self) -> Self {
        self.matches &= argument_count(self.call) == 0;
        self
    }

    pub(super) fn with_arguments(mut self) -> Self {
        self.matches &= argument_count(self.call) > 0;
        self
    }

    pub(super) fn with_argument_count(mut self, expected: usize) -> Self {
        self.matches &= argument_count(self.call) == expected;
        self
    }

    pub(super) fn with_block(mut self) -> Self {
        self.matches &= self.call.block().is_some();
        self
    }

    #[allow(dead_code)]
    pub(super) fn without_block(mut self) -> Self {
        self.matches &= self.call.block().is_none();
        self
    }

    #[allow(dead_code)]
    pub(super) fn with_keyword(mut self, keyword: &[u8]) -> Self {
        self.matches &= has_keyword(self.call, keyword);
        self
    }

    #[allow(dead_code)]
    pub(super) fn with_only_argument_matching(
        mut self,
        predicate: impl FnOnce(&Node<'pr>) -> bool,
    ) -> Self {
        self.matches &= only_argument(self.call).as_ref().is_some_and(predicate);
        self
    }

    #[allow(dead_code)]
    pub(super) fn with_first_argument_matching(
        mut self,
        predicate: impl FnOnce(&Node<'pr>) -> bool,
    ) -> Self {
        self.matches &= first_argument(self.call).as_ref().is_some_and(predicate);
        self
    }

    #[allow(dead_code)]
    pub(super) fn on_receiver_call_named(mut self, name: &[u8]) -> Self {
        self.matches &= receiver_call(self.call)
            .as_ref()
            .is_some_and(|receiver| call_name(receiver) == name);
        self
    }

    pub(super) fn with_operator(mut self, operator: &[u8]) -> Self {
        self.matches &= call_operator_is(self.call, operator);
        self
    }

    pub(super) fn matches(self) -> bool {
        self.matches
    }

    pub(super) fn capture_first_argument(self) -> Option<Node<'pr>> {
        self.matches.then(|| first_argument(self.call)).flatten()
    }

    #[allow(dead_code)]
    pub(super) fn capture_only_argument(self) -> Option<Node<'pr>> {
        self.matches.then(|| only_argument(self.call)).flatten()
    }
}

pub(super) fn call_name<'pr>(node: &CallNode<'pr>) -> &'pr [u8] {
    node.name().as_slice()
}

pub(super) fn first_argument<'pr>(node: &CallNode<'pr>) -> Option<Node<'pr>> {
    node.arguments()?.arguments().first()
}

pub(super) fn only_argument<'pr>(node: &CallNode<'pr>) -> Option<Node<'pr>> {
    let arguments = node.arguments()?.arguments();
    (arguments.len() == 1).then(|| arguments.first()).flatten()
}

pub(super) fn argument_count(node: &CallNode<'_>) -> usize {
    node.arguments()
        .map_or(0, |arguments| arguments.arguments().len())
}

pub(super) fn receiver_call<'pr>(node: &CallNode<'pr>) -> Option<CallNode<'pr>> {
    node.receiver()?.as_call_node()
}

#[allow(dead_code)]
pub(super) fn static_string(node: &Node<'_>) -> Option<Vec<u8>> {
    node.as_string_node()
        .map(|string| string.unescaped().to_vec())
}

#[allow(dead_code)]
pub(super) fn static_symbol(node: &Node<'_>) -> Option<Vec<u8>> {
    node.as_symbol_node()
        .map(|symbol| symbol.unescaped().to_vec())
}

#[allow(dead_code)]
pub(super) fn constant_path<'pr>(node: &Node<'pr>) -> Option<Vec<&'pr [u8]>> {
    if let Some(constant) = node.as_constant_read_node() {
        return Some(vec![constant.name().as_slice()]);
    }
    let path = node.as_constant_path_node()?;
    let mut parts = path
        .parent()
        .as_ref()
        .and_then(constant_path)
        .unwrap_or_default();
    parts.push(path.name()?.as_slice());
    Some(parts)
}

pub(super) fn call_operator_is(node: &CallNode<'_>, expected: &[u8]) -> bool {
    node.call_operator_loc()
        .is_some_and(|location| location.as_slice() == expected)
}

pub(super) fn same_location(left: &Node<'_>, right: &Node<'_>) -> bool {
    left.location().start_offset() == right.location().start_offset()
        && left.location().end_offset() == right.location().end_offset()
}

pub(super) fn eval_receiver(receiver: Option<Node<'_>>) -> bool {
    let Some(receiver) = receiver else {
        return true;
    };
    if node_is_root_constant(&receiver, b"Kernel") {
        return true;
    }
    receiver.as_call_node().is_some_and(|call| {
        call.receiver().is_none() && call.arguments().is_none() && call_name(&call) == b"binding"
    })
}

pub(super) fn root_constant(receiver: Option<Node<'_>>, expected: &[u8]) -> bool {
    receiver
        .as_ref()
        .is_some_and(|receiver| node_is_root_constant(receiver, expected))
}

pub(super) fn node_is_root_constant(receiver: &Node<'_>, expected: &[u8]) -> bool {
    if let Some(constant) = receiver.as_constant_read_node() {
        return constant.name().as_slice() == expected;
    }
    receiver.as_constant_path_node().is_some_and(|constant| {
        constant.parent().is_none()
            && constant
                .name()
                .is_some_and(|name| name.as_slice() == expected)
    })
}

pub(super) fn constant_read(receiver: Option<Node<'_>>, expected: &[u8]) -> bool {
    receiver
        .and_then(|node| node.as_constant_read_node())
        .is_some_and(|constant| constant.name().as_slice() == expected)
}

pub(super) fn marshal_dump(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"dump" && root_constant(call.receiver(), b"Marshal")
    })
}

pub(super) fn has_keyword(node: &CallNode<'_>, expected: &[u8]) -> bool {
    node.arguments().is_some_and(|arguments| {
        arguments.arguments().iter().any(|argument| {
            argument
                .as_keyword_hash_node()
                .is_some_and(|hash| keyword_hash_contains(&hash, expected))
                || argument.as_hash_node().is_some_and(|hash| {
                    hash.elements().iter().any(|element| {
                        element
                            .as_assoc_node()
                            .and_then(|association| association.key().as_symbol_node())
                            .is_some_and(|symbol| symbol.unescaped() == expected)
                    })
                })
        })
    })
}

fn keyword_hash_contains(hash: &ruby_prism::KeywordHashNode<'_>, expected: &[u8]) -> bool {
    hash.elements().iter().any(|element| {
        element
            .as_assoc_node()
            .and_then(|association| association.key().as_symbol_node())
            .is_some_and(|symbol| symbol.unescaped() == expected)
    })
}

pub(super) fn recursive_literal_string(node: &Node<'_>) -> bool {
    node.as_interpolated_string_node().is_some_and(|string| {
        string.parts().iter().all(|part| {
            part.as_string_node().is_some()
                || part.as_embedded_statements_node().is_some_and(|embedded| {
                    embedded
                        .statements()
                        .is_some_and(|statements| statements.body().iter().all(literal_node))
                })
        })
    })
}

fn literal_node(node: Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
}

pub(super) fn safe_open_argument(node: &Node<'_>) -> bool {
    if let Some(string) = node.as_string_node() {
        return safe_open_text(string.unescaped());
    }
    if let Some(string) = node.as_interpolated_string_node() {
        return string
            .parts()
            .first()
            .is_some_and(|part| safe_open_argument(&part));
    }
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"+"
            && call
                .receiver()
                .is_some_and(|receiver| safe_open_argument(&receiver))
    })
}

fn safe_open_text(text: &[u8]) -> bool {
    !text.is_empty() && !text.starts_with(b"|")
}

pub(super) fn string_starts_with_pipe(node: &Node<'_>) -> bool {
    node.as_string_node()
        .is_some_and(|string| trim_ascii_whitespace(string.unescaped()).starts_with(b"|"))
}

fn trim_ascii_whitespace(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}

pub(super) fn source_at<'source>(source: &'source str, location: &Location<'_>) -> &'source str {
    &source[location.start_offset()..location.end_offset()]
}

pub(super) fn node_source<'source>(source: &'source str, node: &Node<'_>) -> &'source str {
    source_at(source, &node.location())
}

pub(super) fn same_source(source: &str, left: &Node<'_>, right: &Node<'_>) -> bool {
    node_source(source, left) == node_source(source, right)
}

#[cfg(test)]
#[path = "matchers_tests.rs"]
mod tests;
