// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/hash_subset.rb
// Source SHA-256: 35bbdc65c4bad6fda7808b3378d8e4b69ac654c50884f8d638263e24d04cd91e

use std::ops::Range;

pub(crate) const MSG: &str = "Use `%<prefer>s` instead.";
pub(crate) const SUBSET_METHODS: &[&str] = &["==", "!=", "eql?", "include?"];
pub(crate) const ACTIVE_SUPPORT_SUBSET_METHODS: &[&str] =
    &["==", "!=", "eql?", "include?", "in?", "exclude?"];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OperandKind {
    Symbol,
    String,
    DynamicSymbol,
    DynamicString,
    Local,
    Range,
    OtherLiteral,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Operand {
    pub(crate) source: String,
    pub(crate) kind: OperandKind,
    pub(crate) name: Option<String>,
    pub(crate) values: Vec<Operand>,
    pub(crate) percent_literal: bool,
}

impl Operand {
    pub(crate) fn local(name: &str) -> Self {
        Self {
            source: name.into(),
            kind: OperandKind::Local,
            name: Some(name.into()),
            values: Vec::new(),
            percent_literal: false,
        }
    }

    pub(crate) fn literal(source: &str, kind: OperandKind) -> Self {
        Self {
            source: source.into(),
            kind,
            name: None,
            values: Vec::new(),
            percent_literal: false,
        }
    }

    pub(crate) fn array(values: Vec<Operand>, percent_literal: bool) -> Self {
        Self {
            source: String::new(),
            kind: OperandKind::Other,
            name: None,
            values,
            percent_literal,
        }
    }

    pub(crate) fn array_type(&self) -> bool {
        !self.values.is_empty()
    }

    pub(crate) fn literal_type(&self) -> bool {
        matches!(
            self.kind,
            OperandKind::Symbol
                | OperandKind::String
                | OperandKind::DynamicSymbol
                | OperandKind::DynamicString
                | OperandKind::OtherLiteral
        )
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SubsetBody {
    pub(crate) method_name: String,
    pub(crate) receiver: Operand,
    pub(crate) first_argument: Operand,
    pub(crate) argument_count: usize,
    pub(crate) negated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HashSubsetBlock {
    pub(crate) key_argument: Operand,
    pub(crate) value_argument: Operand,
    pub(crate) body: SubsetBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HashSubsetSend {
    pub(crate) method_name: String,
    pub(crate) selector_range: Range<usize>,
    pub(crate) block_end: usize,
    pub(crate) block: Option<HashSubsetBlock>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum HashSubsetPreference {
    Except,
    Slice,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HashSubsetOffense {
    pub(crate) range: Range<usize>,
    pub(crate) message: String,
    pub(crate) replacement: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct HashSubset {
    pub(crate) active_support_extensions_enabled: bool,
    pub(crate) preference: HashSubsetPreference,
}

impl HashSubset {
    pub(crate) fn on_send(&self, node: &HashSubsetSend) -> Option<HashSubsetOffense> {
        let (range, key_source) = self.extract_offense(node)?;
        if !self.semantically_subset_method(node) {
            return None;
        }
        let replacement = format!("{}({key_source})", self.preferred_method_name());
        Some(HashSubsetOffense {
            range,
            message: format!("Use `{replacement}` instead."),
            replacement,
        })
    }

    pub(crate) fn on_csend(&self, node: &HashSubsetSend) -> Option<HashSubsetOffense> {
        self.on_send(node)
    }

    pub(crate) fn semantically_subset_method(&self, node: &HashSubsetSend) -> bool {
        match self.preference {
            HashSubsetPreference::Except => self.semantically_except_method(node),
            HashSubsetPreference::Slice => self.semantically_slice_method(node),
        }
    }

    pub(crate) const fn preferred_method_name(&self) -> &'static str {
        match self.preference {
            HashSubsetPreference::Except => "except",
            HashSubsetPreference::Slice => "slice",
        }
    }

    pub(crate) fn extract_offense(&self, node: &HashSubsetSend) -> Option<(Range<usize>, String)> {
        let block = node.block.as_ref()?;
        if !self.extracts_hash_subset(block) {
            return None;
        }
        let except_key = self.except_key(block)?;
        if !self.safe_to_register_offense(block, except_key) {
            return None;
        }
        Some((self.offense_range(node), self.except_key_source(except_key)))
    }

    pub(crate) fn block_with_first_arg_check<'block>(
        &self,
        block: &'block HashSubsetBlock,
    ) -> (
        &'block Operand,
        &'block Operand,
        &'block SubsetBody,
        &'block str,
    ) {
        (
            &block.key_argument,
            &block.value_argument,
            &block.body,
            &block.body.method_name,
        )
    }

    pub(crate) fn extracts_hash_subset(&self, block: &HashSubsetBlock) -> bool {
        let (key_arg, value_arg, send, method) = self.block_with_first_arg_check(block);
        if send.argument_count != 1
            || !self.supported_subset_method(method)
            || self.range_include(send)
        {
            return false;
        }
        match method {
            "include?" | "exclude?" => self.slices_key(send, false, key_arg, value_arg),
            "in?" => self.slices_key(send, true, key_arg, value_arg),
            _ => true,
        }
    }

    pub(crate) fn slices_key(
        &self,
        send: &SubsetBody,
        use_receiver: bool,
        key_arg: &Operand,
        value_arg: &Operand,
    ) -> bool {
        if self.using_value_variable(send, value_arg) {
            return false;
        }
        let node = if use_receiver {
            &send.receiver
        } else {
            &send.first_argument
        };
        node.source == key_arg.source
    }

    pub(crate) fn range_include(&self, send: &SubsetBody) -> bool {
        send.first_argument.kind == OperandKind::Range || send.receiver.kind == OperandKind::Range
    }

    pub(crate) fn using_value_variable(&self, send: &SubsetBody, value_arg: &Operand) -> bool {
        let value_name = value_arg.name.as_deref();
        (send.receiver.kind == OperandKind::Local && send.receiver.name.as_deref() == value_name)
            || (send.first_argument.kind == OperandKind::Local
                && send.first_argument.name.as_deref() == value_name)
    }

    pub(crate) fn supported_subset_method(&self, method: &str) -> bool {
        if self.active_support_extensions_enabled {
            ACTIVE_SUPPORT_SUBSET_METHODS.contains(&method)
        } else {
            SUBSET_METHODS.contains(&method)
        }
    }

    pub(crate) fn semantically_except_method(&self, node: &HashSubsetSend) -> bool {
        let block = match node.block.as_ref() {
            Some(block) => block,
            None => return false,
        };
        let (body, negated) = self.extract_body_if_negated(&block.body);
        if node.method_name == "reject" {
            matches!(body.method_name.as_str(), "==" | "eql?") || self.included(body, negated)
        } else {
            body.method_name == "!=" || self.not_included(body, negated)
        }
    }

    pub(crate) fn semantically_slice_method(&self, node: &HashSubsetSend) -> bool {
        !self.semantically_except_method(node)
    }

    pub(crate) fn included(&self, body: &SubsetBody, negated: bool) -> bool {
        if negated {
            body.method_name == "exclude?"
        } else {
            matches!(body.method_name.as_str(), "include?" | "in?")
        }
    }

    pub(crate) fn not_included(&self, body: &SubsetBody, negated: bool) -> bool {
        self.included(body, !negated)
    }

    pub(crate) fn safe_to_register_offense(
        &self,
        block: &HashSubsetBlock,
        except_key: &Operand,
    ) -> bool {
        if matches!(block.body.method_name.as_str(), "==" | "!=") {
            matches!(except_key.kind, OperandKind::Symbol | OperandKind::String)
        } else {
            true
        }
    }

    pub(crate) fn extract_body_if_negated<'body>(
        &self,
        body: &'body SubsetBody,
    ) -> (&'body SubsetBody, bool) {
        (body, body.negated)
    }

    pub(crate) fn except_key_source(&self, key: &Operand) -> String {
        if key.array_type() {
            return key
                .values
                .iter()
                .map(|value| {
                    if key.percent_literal {
                        self.decorate_source(value)
                    } else {
                        value.source.clone()
                    }
                })
                .collect::<Vec<_>>()
                .join(", ");
        }
        if key.literal_type() {
            key.source.clone()
        } else {
            format!("*{}", key.source)
        }
    }

    pub(crate) fn decorate_source(&self, value: &Operand) -> String {
        match value.kind {
            OperandKind::DynamicSymbol => format!(":\"{}\"", value.source),
            OperandKind::DynamicString => format!("\"{}\"", value.source),
            OperandKind::Symbol => format!(":{}", value.source),
            _ => format!("'{}'", value.source),
        }
    }

    pub(crate) fn except_key<'block>(
        &self,
        block: &'block HashSubsetBlock,
    ) -> Option<&'block Operand> {
        let key_source = &block.key_argument.source;
        let (body, _) = self.extract_body_if_negated(&block.body);
        if body.receiver.source == *key_source {
            Some(&body.first_argument)
        } else {
            Some(&body.receiver)
        }
    }

    pub(crate) fn offense_range(&self, node: &HashSubsetSend) -> Range<usize> {
        node.selector_range.start..node.block_end
    }
}
