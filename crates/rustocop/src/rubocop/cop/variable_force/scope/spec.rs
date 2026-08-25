use super::Scope;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn method_scope_enumeration_excludes_nested_scope_bodies() {
    let source = "def outer\n  before\n  items.each do\n    inside\n  end\n  after\nend";
    let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let method = parsed.ast().unwrap();
    let scope = Scope::initialize(method).unwrap();
    let sources = scope
        .each_node()
        .into_iter()
        .filter_map(|node| node.source())
        .collect::<Vec<_>>();
    assert!(sources.contains(&"before"));
    assert!(sources
        .iter()
        .any(|source| source.starts_with("items.each")));
    assert!(!sources.contains(&"inside"));
    assert_eq!(scope.name(), Some("outer"));
}

#[test]
fn arbitrary_root_is_the_naked_top_level_scope() {
    let parsed = ProcessedSource::new("one\ntwo", 3.4, None, ParserEngine::Prism).unwrap();
    let scope = Scope::initialize(parsed.ast().unwrap()).unwrap();
    assert!(scope.naked_top_level());
    assert_eq!(scope.body_node(), Some(parsed.ast().unwrap()));
    assert!(scope.equivalent(&Scope::initialize(parsed.ast().unwrap()).unwrap()));
}
