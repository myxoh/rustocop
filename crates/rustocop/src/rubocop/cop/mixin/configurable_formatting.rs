// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/configurable_formatting.rb
// Source SHA-256: 6e14ea072f9fe28dc63c60b0b65d33e38e5125aacf91c5d1b70769936f1f2191

use regex::Regex;
use std::collections::BTreeMap;

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FormattingNode {
    pub(crate) has_parent: bool,
    pub(crate) singleton_definition: bool,
    pub(crate) enclosing_class_names: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StyleDetection {
    Correct,
    Unexpected(String),
    Unrecognized,
}

#[derive(Clone, Debug)]
pub(crate) struct ConfigurableFormatting {
    pub(crate) style: String,
    pub(crate) formats: BTreeMap<String, Regex>,
}

impl ConfigurableFormatting {
    pub(crate) fn check_name(
        &self,
        node: &FormattingNode,
        name: &str,
    ) -> (Option<String>, StyleDetection) {
        if self.valid_name(node, name, &self.style) {
            (None, StyleDetection::Correct)
        } else {
            (
                Some(format!("Use {} for names.", self.style)),
                self.report_opposing_styles(node, name),
            )
        }
    }

    pub(crate) fn report_opposing_styles(
        &self,
        node: &FormattingNode,
        name: &str,
    ) -> StyleDetection {
        for alternative in self
            .formats
            .keys()
            .filter(|candidate| *candidate != &self.style)
        {
            if self.valid_name(node, name, alternative) {
                return StyleDetection::Unexpected(alternative.clone());
            }
        }
        StyleDetection::Unrecognized
    }

    pub(crate) fn valid_name(&self, node: &FormattingNode, name: &str, given_style: &str) -> bool {
        self.formats
            .get(given_style)
            .expect("configured style has a format")
            .is_match(name)
            || self.class_emitter_method(node, name)
    }

    pub(crate) fn class_emitter_method(&self, node: &FormattingNode, name: &str) -> bool {
        node.has_parent
            && node.singleton_definition
            && node
                .enclosing_class_names
                .iter()
                .any(|class_name| class_name == name)
    }
}
