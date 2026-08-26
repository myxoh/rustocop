use super::VariableTable;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn block_scopes_capture_visible_outer_variables() {
    let parsed = ProcessedSource::new(
        "outer = 1\nitems.each do\n  outer = 2\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let root = parsed.ast().unwrap();
    let declaration = root.child_nodes()[0];
    let block = root.each_node(&["block"]).into_iter().next().unwrap();
    let assignment = block.each_node(&["lvasgn"]).into_iter().next().unwrap();
    let mut table = VariableTable::initialize();
    table.push_scope(root);
    table.declare_variable("outer", declaration).unwrap();
    table.push_scope(block);
    table.assign_to_variable("outer", assignment).unwrap();
    assert!(table.find_variable("outer").unwrap().captured_by_block);
    assert_eq!(table.accessible_variables().len(), 1);
}

#[test]
fn method_like_scopes_stop_outer_visibility_and_missing_references_are_skipped() {
    let parsed =
        ProcessedSource::new("def work\n  missing\nend", 3.4, None, ParserEngine::Prism).unwrap();
    let method = parsed.ast().unwrap();
    let reference = method.body().unwrap();
    let mut table = VariableTable::initialize();
    table.push_scope(method);
    assert!(!table.reference_variable("missing", reference));
    assert!(!table.variable_exist("missing"));
    assert_eq!(table.current_scope_level(), 1);
    assert!(table.pop_scope().is_some());
}
