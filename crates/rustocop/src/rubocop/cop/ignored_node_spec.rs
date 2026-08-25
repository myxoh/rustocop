use super::ignored_node::{IgnoredNode, NodeIdentity, NodeLocation, NodeRef};

fn node(identity: usize, start: usize, end: usize) -> NodeRef {
    NodeRef {
        identity: NodeIdentity(identity),
        location: NodeLocation {
            expression: start..end,
            heredoc_end: None,
        },
    }
}

#[test]
fn identity_and_containment_follow_the_ruby_module() {
    let parent = node(1, 5, 20);
    let same_range_different_object = node(2, 5, 20);
    let child = node(3, 8, 12);
    let starts_before = node(4, 4, 12);
    let extends_after = node(5, 8, 21);
    let mut ignored = IgnoredNode::default();
    ignored.ignore_node(parent.clone());

    assert!(ignored.ignored_node(&parent));
    assert!(!ignored.ignored_node(&same_range_different_object));
    assert!(ignored.part_of_ignored_node(&child));
    assert!(!ignored.part_of_ignored_node(&starts_before));
    assert!(!ignored.part_of_ignored_node(&extends_after));
    assert_eq!(ignored.ignored_nodes(), &[parent]);
}

#[test]
fn heredoc_uses_heredoc_end_instead_of_expression_end() {
    let mut parent = node(1, 5, 10);
    parent.location.heredoc_end = Some(30);
    let mut ignored = IgnoredNode::default();
    ignored.ignore_node(parent);
    assert!(ignored.part_of_ignored_node(&node(2, 20, 29)));
    assert!(!ignored.part_of_ignored_node(&node(3, 20, 31)));
}
