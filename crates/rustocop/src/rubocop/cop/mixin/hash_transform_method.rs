// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/hash_transform_method.rb
// Source SHA-256: 1b4984200f14e13817355f80d8d9b5cc5bdfd06358a800a76c65e581c296609a

pub(crate) mod autocorrection;

use autocorrection::Autocorrection;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransformExpression {
    pub(crate) source: String,
    pub(crate) local_name: Option<String>,
    pub(crate) descendant_sources: Vec<String>,
    pub(crate) hash_type: bool,
    pub(crate) braces: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Captures {
    pub(crate) transformed_argname: String,
    pub(crate) transforming_body_expr: TransformExpression,
    pub(crate) unchanged_body_expr: TransformExpression,
}

impl Captures {
    pub(crate) fn transformed_argname(&self) -> &str {
        &self.transformed_argname
    }
    pub(crate) fn transforming_body_expr(&self) -> &TransformExpression {
        &self.transforming_body_expr
    }
    pub(crate) fn unchanged_body_expr(&self) -> &TransformExpression {
        &self.unchanged_body_expr
    }

    pub(crate) fn noop_transformation(&self) -> bool {
        self.transforming_body_expr.local_name.as_deref() == Some(self.transformed_argname.as_str())
    }

    pub(crate) fn transformation_uses_both_args(&self) -> bool {
        self.transforming_body_expr
            .descendant_sources
            .contains(&self.unchanged_body_expr.source)
    }

    pub(crate) fn use_transformed_argname(&self) -> bool {
        self.transforming_body_expr
            .descendant_sources
            .iter()
            .any(|source| source == &self.transformed_argname)
            || self.transforming_body_expr.local_name.as_deref()
                == Some(self.transformed_argname.as_str())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum HashReceiver {
    Hash,
    Send(String),
    Block(String),
    EachWithObjectHash,
    Other,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum BadTransformKind {
    EachWithObject,
    HashBracketsMap,
    MapToH,
    ToH,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct MatchData {
    pub(crate) kind: BadTransformKind,
    pub(crate) captures: Captures,
    pub(crate) correction: Autocorrection,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransformNode {
    pub(crate) receiver: HashReceiver,
    pub(crate) each_with_object: Option<MatchData>,
    pub(crate) hash_brackets_map: Option<MatchData>,
    pub(crate) map_to_h: Option<MatchData>,
    pub(crate) to_h: Option<MatchData>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TransformOffense {
    pub(crate) message: String,
    pub(crate) corrected_source: String,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) struct HashTransformMethod {
    pub(crate) target_ruby_version: f64,
    pub(crate) replacement_method: String,
}

impl HashTransformMethod {
    pub(crate) fn hash_receiver(&self, receiver: &HashReceiver) -> bool {
        match receiver {
            HashReceiver::Hash | HashReceiver::EachWithObjectHash => true,
            HashReceiver::Send(method) => matches!(
                method.as_str(),
                "to_h" | "to_hash" | "merge" | "merge!" | "update" | "invert" | "except" | "tally"
            ),
            HashReceiver::Block(method) => matches!(
                method.as_str(),
                "group_by"
                    | "to_h"
                    | "tally"
                    | "transform_keys"
                    | "transform_keys!"
                    | "transform_values"
                    | "transform_values!"
            ),
            HashReceiver::Other => false,
        }
    }

    pub(crate) fn on_block(&self, node: &TransformNode) -> Vec<TransformOffense> {
        let mut offenses = Vec::new();
        if let Some(found) = self.on_bad_each_with_object(node) {
            if let Some(offense) = self.handle_possible_offense(found, "each_with_object") {
                offenses.push(offense);
            }
        }
        if self.target_ruby_version >= 2.6 {
            if let Some(found) = self.on_bad_to_h(node) {
                if let Some(offense) = self.handle_possible_offense(found, "to_h {...}") {
                    offenses.push(offense);
                }
            }
        }
        offenses
    }

    pub(crate) fn on_send(&self, node: &TransformNode) -> Vec<TransformOffense> {
        [
            (self.on_bad_hash_brackets_map(node), "Hash[_.map {...}]"),
            (self.on_bad_map_to_h(node), "map {...}.to_h"),
        ]
        .into_iter()
        .filter_map(|(found, description)| self.handle_possible_offense(found?, description))
        .collect()
    }

    pub(crate) fn on_csend(&self, node: &TransformNode) -> Vec<TransformOffense> {
        self.on_bad_map_to_h(node)
            .and_then(|found| self.handle_possible_offense(found, "map {...}.to_h"))
            .into_iter()
            .collect()
    }

    pub(crate) fn on_bad_each_with_object<'node>(
        &self,
        node: &'node TransformNode,
    ) -> Option<&'node MatchData> {
        node.each_with_object.as_ref()
    }

    pub(crate) fn on_bad_hash_brackets_map<'node>(
        &self,
        node: &'node TransformNode,
    ) -> Option<&'node MatchData> {
        node.hash_brackets_map.as_ref()
    }

    pub(crate) fn on_bad_map_to_h<'node>(
        &self,
        node: &'node TransformNode,
    ) -> Option<&'node MatchData> {
        node.map_to_h.as_ref()
    }

    pub(crate) fn on_bad_to_h<'node>(
        &self,
        node: &'node TransformNode,
    ) -> Option<&'node MatchData> {
        node.to_h.as_ref()
    }

    pub(crate) fn handle_possible_offense(
        &self,
        found: &MatchData,
        match_description: &str,
    ) -> Option<TransformOffense> {
        let captures = self.extract_captures(found);
        if captures.noop_transformation()
            || captures.transformation_uses_both_args()
            || !captures.use_transformed_argname()
        {
            return None;
        }
        Some(TransformOffense {
            message: format!(
                "Prefer `{}` over `{match_description}`.",
                self.new_method_name()
            ),
            corrected_source: self.execute_correction(found),
        })
    }

    pub(crate) fn extract_captures<'found>(&self, found: &'found MatchData) -> &'found Captures {
        &found.captures
    }

    pub(crate) fn new_method_name(&self) -> &str {
        &self.replacement_method
    }

    pub(crate) fn prepare_correction<'node>(
        &self,
        node: &'node TransformNode,
    ) -> Option<&'node Autocorrection> {
        self.on_bad_each_with_object(node)
            .or_else(|| self.on_bad_hash_brackets_map(node))
            .or_else(|| self.on_bad_map_to_h(node))
            .or_else(|| self.on_bad_to_h(node))
            .map(|found| &found.correction)
    }

    pub(crate) fn execute_correction(&self, found: &MatchData) -> String {
        found.correction.apply(
            self.new_method_name(),
            &found.captures.transformed_argname,
            &found.captures.transforming_body_expr,
        )
    }
}
