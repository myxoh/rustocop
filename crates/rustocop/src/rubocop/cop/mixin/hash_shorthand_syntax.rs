// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/hash_shorthand_syntax.rb
// Source SHA-256: 590dc7c208d1a2f760315021b4c5a04e8805517fc29218eef2592a01dac7393a

use std::collections::BTreeMap;

pub(crate) const OMIT_HASH_VALUE_MSG: &str = "Omit the hash value.";
pub(crate) const EXPLICIT_HASH_VALUE_MSG: &str = "Include the hash value.";
pub(crate) const DO_NOT_MIX_MSG_PREFIX: &str = "Do not mix explicit and implicit hash values.";
pub(crate) const DO_NOT_MIX_OMIT_VALUE_MSG: &str =
    "Do not mix explicit and implicit hash values. Omit the hash value.";
pub(crate) const DO_NOT_MIX_EXPLICIT_VALUE_MSG: &str =
    "Do not mix explicit and implicit hash values. Include the hash value.";

#[allow(clippy::enum_variant_names)] // Names mirror RuboCop's value-type vocabulary.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub(crate) enum HashValueType {
    ValueOmitted,
    ValueNeeded,
    ValueOmittable,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DispatchContext {
    pub(crate) method_name: String,
    pub(crate) send_type: bool,
    pub(crate) hash_is_receiver: bool,
    pub(crate) assignment_method: bool,
    pub(crate) parenthesized: bool,
    pub(crate) parent_parenthesized: bool,
    pub(crate) modifier_form_ancestor: bool,
    pub(crate) last_expression: bool,
    pub(crate) requires_parentheses_context: bool,
    pub(crate) selector: String,
    pub(crate) arguments: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HashPair {
    pub(crate) key_source: String,
    pub(crate) value_source: Option<String>,
    pub(crate) key_is_symbol: bool,
    pub(crate) value_is_send_or_local: bool,
    pub(crate) parent_is_hash: bool,
    pub(crate) parent_has_braces: bool,
    pub(crate) dispatch: Option<DispatchContext>,
}

impl HashPair {
    pub(crate) fn value_omission(&self) -> bool {
        self.value_source.is_none()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HashNode {
    pub(crate) pairs: Vec<HashPair>,
    pub(crate) hash_type: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DefNode {
    dispatch: DispatchContext,
}

impl DefNode {
    pub(crate) fn new(dispatch: DispatchContext) -> Self {
        Self { dispatch }
    }

    pub(crate) fn node(&self) -> &DispatchContext {
        &self.dispatch
    }

    pub(crate) fn selector(&self) -> &str {
        &self.dispatch.selector
    }

    pub(crate) fn first_argument(&self) -> Option<&str> {
        self.dispatch.arguments.first().map(String::as_str)
    }

    pub(crate) fn last_argument(&self) -> Option<&str> {
        self.dispatch.arguments.last().map(String::as_str)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct HashShorthandOffense {
    pub(crate) pair_index: usize,
    pub(crate) message: &'static str,
    pub(crate) replacement: String,
    pub(crate) add_parentheses: bool,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HashShorthandSyntax {
    pub(crate) target_ruby_version: f64,
    pub(crate) enforced_style: String,
}

impl HashShorthandSyntax {
    pub(crate) fn new(target_ruby_version: f64, enforced_style: Option<&str>) -> Self {
        Self {
            target_ruby_version,
            enforced_style: enforced_style.unwrap_or("always").to_owned(),
        }
    }

    pub(crate) fn enforced_shorthand_syntax(&self) -> &str {
        &self.enforced_style
    }

    pub(crate) fn on_hash_for_mixed_shorthand(
        &self,
        hash_node: &HashNode,
    ) -> Vec<HashShorthandOffense> {
        if self.ignore_mixed_hash_shorthand_syntax(hash_node) {
            return Vec::new();
        }
        let breakdown = self.breakdown_value_types_of_hash(hash_node);
        if self.hash_with_mixed_shorthand_syntax(&breakdown) {
            self.mixed_shorthand_syntax_check(hash_node, &breakdown)
        } else {
            self.no_mixed_shorthand_syntax_check(hash_node, &breakdown)
        }
    }

    pub(crate) fn on_pair(
        &self,
        pair_index: usize,
        pair: &HashPair,
        last_pair: Option<&HashPair>,
    ) -> Option<HashShorthandOffense> {
        if self.ignore_hash_shorthand_syntax(pair) {
            return None;
        }
        if self.enforced_shorthand_syntax() == "always" {
            if pair.value_omission() || self.require_hash_value(&pair.key_source, pair) {
                return None;
            }
            Some(self.register_offense(
                pair_index,
                pair,
                last_pair,
                OMIT_HASH_VALUE_MSG,
                format!("{}:", pair.key_source),
            ))
        } else {
            if !pair.value_omission() {
                return None;
            }
            Some(self.register_offense(
                pair_index,
                pair,
                last_pair,
                EXPLICIT_HASH_VALUE_MSG,
                format!("{}: {}", pair.key_source, pair.key_source),
            ))
        }
    }

    pub(crate) fn register_offense(
        &self,
        pair_index: usize,
        pair: &HashPair,
        last_pair: Option<&HashPair>,
        message: &'static str,
        replacement: String,
    ) -> HashShorthandOffense {
        HashShorthandOffense {
            pair_index,
            message,
            replacement,
            add_parentheses: self
                .def_node_that_require_parentheses(pair, last_pair)
                .is_some(),
        }
    }

    pub(crate) fn ignore_mixed_hash_shorthand_syntax(&self, hash_node: &HashNode) -> bool {
        self.target_ruby_version <= 3.0
            || !matches!(
                self.enforced_shorthand_syntax(),
                "consistent" | "either_consistent"
            )
            || !hash_node.hash_type
    }

    pub(crate) fn ignore_hash_shorthand_syntax(&self, pair: &HashPair) -> bool {
        self.target_ruby_version <= 3.0
            || matches!(
                self.enforced_shorthand_syntax(),
                "either" | "consistent" | "either_consistent"
            )
            || !pair.parent_is_hash
    }

    pub(crate) fn require_hash_value(&self, hash_key_source: &str, pair: &HashPair) -> bool {
        if !pair.key_is_symbol || self.require_hash_value_for_around_hash_literal(pair) {
            return true;
        }
        let Some(hash_value) = pair.value_source.as_deref() else {
            return true;
        };
        if !pair.value_is_send_or_local {
            return true;
        }
        hash_key_source != hash_value || hash_key_source.ends_with(['!', '?'])
    }

    pub(crate) fn require_hash_value_for_around_hash_literal(&self, pair: &HashPair) -> bool {
        let Some(dispatch) = self.find_ancestor_method_dispatch_node(pair) else {
            return false;
        };
        !pair.parent_has_braces
            && !self.use_element_of_hash_literal_as_receiver(dispatch)
            && self.use_modifier_form_without_parenthesized_method_call(dispatch)
    }

    pub(crate) fn def_node_that_require_parentheses(
        &self,
        pair: &HashPair,
        last_pair: Option<&HashPair>,
    ) -> Option<DefNode> {
        let last_pair = last_pair?;
        if last_pair.key_source != last_pair.value_source.as_deref()? {
            return None;
        }
        let dispatch = self.find_ancestor_method_dispatch_node(pair)?.clone();
        if dispatch.assignment_method
            || dispatch.parenthesized
            || dispatch.parent_parenthesized
            || (self.last_expression(&dispatch) && !self.requires_parentheses_context(&dispatch))
            || dispatch.arguments.is_empty()
        {
            return None;
        }
        Some(DefNode::new(dispatch))
    }

    pub(crate) fn find_ancestor_method_dispatch_node<'pair>(
        &self,
        pair: &'pair HashPair,
    ) -> Option<&'pair DispatchContext> {
        pair.dispatch
            .as_ref()
            .filter(|dispatch| !self.brackets(dispatch))
    }

    pub(crate) fn brackets(&self, dispatch: &DispatchContext) -> bool {
        matches!(dispatch.method_name.as_str(), "[]" | "[]=")
    }

    pub(crate) fn use_element_of_hash_literal_as_receiver(
        &self,
        dispatch: &DispatchContext,
    ) -> bool {
        dispatch.send_type && dispatch.hash_is_receiver
    }

    pub(crate) fn use_modifier_form_without_parenthesized_method_call(
        &self,
        dispatch: &DispatchContext,
    ) -> bool {
        !dispatch.parenthesized && dispatch.modifier_form_ancestor
    }

    pub(crate) fn last_expression(&self, dispatch: &DispatchContext) -> bool {
        dispatch.last_expression
    }

    pub(crate) fn requires_parentheses_context(&self, dispatch: &DispatchContext) -> bool {
        dispatch.requires_parentheses_context
    }

    pub(crate) fn breakdown_value_types_of_hash(
        &self,
        hash_node: &HashNode,
    ) -> BTreeMap<HashValueType, Vec<usize>> {
        let mut breakdown = BTreeMap::<HashValueType, Vec<usize>>::new();
        for (index, pair) in hash_node.pairs.iter().enumerate() {
            let value_type = if pair.value_omission() {
                HashValueType::ValueOmitted
            } else if self.require_hash_value(&pair.key_source, pair) {
                HashValueType::ValueNeeded
            } else {
                HashValueType::ValueOmittable
            };
            breakdown.entry(value_type).or_default().push(index);
        }
        breakdown
    }

    pub(crate) fn hash_with_mixed_shorthand_syntax(
        &self,
        breakdown: &BTreeMap<HashValueType, Vec<usize>>,
    ) -> bool {
        breakdown.len() > 1
    }

    pub(crate) fn hash_with_values_that_cant_be_omitted(
        &self,
        breakdown: &BTreeMap<HashValueType, Vec<usize>>,
    ) -> bool {
        breakdown
            .get(&HashValueType::ValueNeeded)
            .is_some_and(|pairs| !pairs.is_empty())
    }

    pub(crate) fn ignore_explicit_omissible_hash_shorthand_syntax(
        &self,
        breakdown: &BTreeMap<HashValueType, Vec<usize>>,
    ) -> bool {
        breakdown.len() == 1
            && breakdown.contains_key(&HashValueType::ValueOmittable)
            && self.enforced_shorthand_syntax() == "either_consistent"
    }

    pub(crate) fn each_omitted_value_pair(
        &self,
        breakdown: &BTreeMap<HashValueType, Vec<usize>>,
    ) -> Vec<usize> {
        breakdown
            .get(&HashValueType::ValueOmitted)
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn each_omittable_value_pair(
        &self,
        breakdown: &BTreeMap<HashValueType, Vec<usize>>,
    ) -> Vec<usize> {
        breakdown
            .get(&HashValueType::ValueOmittable)
            .cloned()
            .unwrap_or_default()
    }

    pub(crate) fn mixed_shorthand_syntax_check(
        &self,
        hash_node: &HashNode,
        breakdown: &BTreeMap<HashValueType, Vec<usize>>,
    ) -> Vec<HashShorthandOffense> {
        if self.hash_with_values_that_cant_be_omitted(breakdown) {
            self.each_omitted_value_pair(breakdown)
                .into_iter()
                .map(|index| {
                    let pair = &hash_node.pairs[index];
                    self.register_offense(
                        index,
                        pair,
                        hash_node.pairs.last(),
                        DO_NOT_MIX_EXPLICIT_VALUE_MSG,
                        format!("{}: {}", pair.key_source, pair.key_source),
                    )
                })
                .collect()
        } else {
            self.each_omittable_value_pair(breakdown)
                .into_iter()
                .map(|index| {
                    let pair = &hash_node.pairs[index];
                    self.register_offense(
                        index,
                        pair,
                        hash_node.pairs.last(),
                        DO_NOT_MIX_OMIT_VALUE_MSG,
                        format!("{}:", pair.key_source),
                    )
                })
                .collect()
        }
    }

    pub(crate) fn no_mixed_shorthand_syntax_check(
        &self,
        hash_node: &HashNode,
        breakdown: &BTreeMap<HashValueType, Vec<usize>>,
    ) -> Vec<HashShorthandOffense> {
        if self.hash_with_values_that_cant_be_omitted(breakdown)
            || self.ignore_explicit_omissible_hash_shorthand_syntax(breakdown)
        {
            return Vec::new();
        }
        self.each_omittable_value_pair(breakdown)
            .into_iter()
            .map(|index| {
                let pair = &hash_node.pairs[index];
                self.register_offense(
                    index,
                    pair,
                    hash_node.pairs.last(),
                    OMIT_HASH_VALUE_MSG,
                    format!("{}:", pair.key_source),
                )
            })
            .collect()
    }
}
