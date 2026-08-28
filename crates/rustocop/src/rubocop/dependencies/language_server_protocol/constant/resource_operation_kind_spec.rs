use super::ResourceOperationKind;

#[test]
fn exposes_the_complete_resource_operation_mapping() {
    assert_eq!(ResourceOperationKind::CREATE, "create");
    assert_eq!(ResourceOperationKind::RENAME, "rename");
    assert_eq!(ResourceOperationKind::DELETE, "delete");
}

#[test]
fn values_can_be_selected_as_protocol_operations() {
    let operations = [ResourceOperationKind::CREATE, ResourceOperationKind::DELETE];
    assert!(operations.contains(&"create"));
    assert!(!operations.contains(&"rename"));
}
