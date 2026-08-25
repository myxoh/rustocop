use super::hash_transform_method::autocorrection::{Autocorrection, BlockGeometry};
use super::hash_transform_method::*;

fn expression(source: &str, local: Option<&str>, descendants: &[&str]) -> TransformExpression {
    TransformExpression {
        source: source.into(),
        local_name: local.map(str::to_owned),
        descendant_sources: descendants.iter().map(|value| (*value).into()).collect(),
        hash_type: false,
        braces: true,
    }
}

fn geometry(source: &str) -> BlockGeometry {
    BlockGeometry {
        source: source.into(),
        expression: 0..source.len(),
        selector: 5..8,
        send_end: None,
        arguments: 10..16,
        body: 18..23,
    }
}

fn found(kind: BadTransformKind, source: &str) -> MatchData {
    let correction = match kind {
        BadTransformKind::EachWithObject => Autocorrection::from_each_with_object(geometry(source)),
        BadTransformKind::HashBracketsMap => {
            Autocorrection::from_hash_brackets_map(geometry(source))
        }
        BadTransformKind::MapToH => Autocorrection::from_map_to_h(geometry(source), 0),
        BadTransformKind::ToH => Autocorrection::from_to_h(geometry(source)),
    };
    correction.match_data(Captures {
        transformed_argname: "value".into(),
        transforming_body_expr: expression("value.upcase", None, &["value"]),
        unchanged_body_expr: expression("key", Some("key"), &[]),
    })
}

#[test]
fn capture_guards_reject_noops_cross_argument_use_and_unused_arguments() {
    let mut captures = found(BadTransformKind::ToH, "hash.to_h { |k,v| body }").captures;
    captures.transforming_body_expr = expression("value", Some("value"), &[]);
    assert!(captures.noop_transformation());
    captures.transforming_body_expr = expression("key + value", None, &["key", "value"]);
    assert!(captures.transformation_uses_both_args());
    captures.transforming_body_expr = expression("constant", None, &[]);
    assert!(!captures.use_transformed_argname());
}

#[test]
fn receiver_classification_matches_every_upstream_family() {
    let transform = HashTransformMethod {
        target_ruby_version: 3.4,
        replacement_method: "transform_values".into(),
    };
    assert!(transform.hash_receiver(&HashReceiver::Hash));
    assert!(transform.hash_receiver(&HashReceiver::Send("merge".into())));
    assert!(transform.hash_receiver(&HashReceiver::Block("group_by".into())));
    assert!(transform.hash_receiver(&HashReceiver::EachWithObjectHash));
    assert!(!transform.hash_receiver(&HashReceiver::Send("map".into())));
}

#[test]
fn callbacks_keep_ruby_version_and_safe_navigation_boundaries() {
    let source = "hash.to_h { |k,v| body }";
    let node = TransformNode {
        receiver: HashReceiver::Hash,
        each_with_object: None,
        hash_brackets_map: None,
        map_to_h: Some(found(BadTransformKind::MapToH, source)),
        to_h: Some(found(BadTransformKind::ToH, source)),
    };
    let old = HashTransformMethod {
        target_ruby_version: 2.5,
        replacement_method: "transform_values".into(),
    };
    assert!(old.on_block(&node).is_empty());
    assert_eq!(old.on_send(&node).len(), 1);
    assert_eq!(old.on_csend(&node).len(), 1);
    let modern = HashTransformMethod {
        target_ruby_version: 2.6,
        ..old
    };
    assert_eq!(modern.on_block(&node).len(), 1);
    assert!(modern.prepare_correction(&node).is_some());
}

#[test]
fn autocorrection_exposes_each_ordered_edit_and_wraps_unbraced_hashes() {
    let block = BlockGeometry {
        source: "obj.map { |k, v| k => v }.to_h".into(),
        expression: 0..32,
        selector: 4..7,
        send_end: None,
        arguments: 10..16,
        body: 18..24,
    };
    let correction = Autocorrection::from_map_to_h(block, 5);
    assert_eq!(correction.strip_prefix_and_suffix().1, 27..32);
    assert_eq!(
        correction.set_new_method_name("transform_keys"),
        (4..7, "transform_keys".into())
    );
    assert_eq!(correction.set_new_arg_name("key"), (10..16, "|key|".into()));
    let mut body = expression("key: value", None, &["key"]);
    body.hash_type = true;
    body.braces = false;
    assert_eq!(
        correction.set_new_body_expression(&body),
        (18..24, "{ key: value }".into())
    );
}
