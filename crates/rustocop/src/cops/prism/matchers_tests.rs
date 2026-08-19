use super::*;
use ruby_prism::parse;

fn first_call_matches(source: &[u8], predicate: impl FnOnce(&CallNode<'_>) -> bool) -> bool {
    let parsed = parse(source);
    let program = parsed.node().as_program_node().unwrap();
    let call = program
        .statements()
        .body()
        .first()
        .unwrap()
        .as_call_node()
        .unwrap();
    predicate(&call)
}

#[test]
fn matches_call_name_root_receiver_and_argument_count() {
    assert!(first_call_matches(b"JSON.load(document)", |call| {
        match_call(call)
            .named_any(&[b"load", b"restore"])
            .on_root_constant(b"JSON")
            .with_argument_count(1)
            .matches()
    }));
}

#[test]
fn rejects_nested_constants_when_root_constant_is_required() {
    assert!(!first_call_matches(b"Other::JSON.load(document)", |call| {
        match_call(call)
            .named(b"load")
            .on_root_constant(b"JSON")
            .matches()
    }));
}

#[test]
fn distinguishes_implicit_calls_from_calls_with_receivers() {
    assert!(first_call_matches(b"require('example')", |call| {
        match_call(call)
            .named(b"require")
            .without_receiver()
            .with_argument_count(1)
            .matches()
    }));
}

#[test]
fn matches_receiver_block_and_operator_shapes() {
    assert!(first_call_matches(b"items&.map { _1 }", |call| {
        match_call(call)
            .named(b"map")
            .with_receiver()
            .without_arguments()
            .with_block()
            .with_operator(b"&.")
            .matches()
    }));
}

#[test]
fn matches_argument_presence_content_and_receiver_calls() {
    assert!(first_call_matches(b"items.sort.first(2)", |call| {
        match_call(call)
            .named(b"first")
            .with_arguments()
            .with_first_argument_matching(|argument| argument.as_integer_node().is_some())
            .on_receiver_call_named(b"sort")
            .matches()
    }));
    assert!(!first_call_matches(b"items.sort.first", |call| {
        match_call(call).with_arguments().matches()
    }));
    assert!(first_call_matches(b"example()", |call| {
        match_call(call).without_arguments().matches()
    }));
}

#[test]
fn exact_argument_helpers_do_not_treat_the_first_of_many_as_the_only_one() {
    assert!(first_call_matches(b"example(first, second)", |call| {
        argument_count(call) == 2 && first_argument(call).is_some() && only_argument(call).is_none()
    }));
}

#[test]
fn captures_arguments_only_after_the_complete_call_shape_matches() {
    assert!(first_call_matches(b"Kernel.eval(source)", |call| {
        match_call(call)
            .named(b"eval")
            .with_receiver_matching(|receiver| root_constant(receiver, b"Kernel"))
            .capture_first_argument()
            .is_some_and(|argument| node_source("Kernel.eval(source)", &argument) == "source")
    }));
    assert!(first_call_matches(b"Kernel.load(source)", |call| {
        match_call(call)
            .named(b"eval")
            .capture_first_argument()
            .is_none()
    }));
}

#[test]
fn matches_implicit_or_named_root_receivers() {
    assert!(first_call_matches(b"rand(1)", |call| {
        match_call(call)
            .named(b"rand")
            .on_implicit_or_root_constant(b"Kernel")
            .matches()
    }));
    assert!(first_call_matches(b"Kernel.rand(1)", |call| {
        match_call(call)
            .named(b"rand")
            .on_implicit_or_root_constant(b"Kernel")
            .matches()
    }));
}

#[test]
fn reads_and_compares_node_source_without_offset_boilerplate() {
    let parsed = parse(b"value == value");
    let program = parsed.node().as_program_node().unwrap();
    let call = program
        .statements()
        .body()
        .first()
        .unwrap()
        .as_call_node()
        .unwrap();
    let left = call.receiver().unwrap();
    let right = only_argument(&call).unwrap();

    assert_eq!(node_source("value == value", &left), "value");
    assert!(same_source("value == value", &left, &right));
}

#[test]
fn matches_keywords_predicates_literals_and_constant_paths() {
    assert!(first_call_matches(
        b"Example::Client.load('value', safe: true)",
        |call| {
            let argument = first_argument(call).unwrap();
            let receiver = call.receiver().unwrap();
            match_call(call)
                .with_keyword(b"safe")
                .without_block()
                .matches()
                && static_string(&argument) == Some(b"value".to_vec())
                && constant_path(&receiver)
                    == Some(vec![b"Example".as_slice(), b"Client".as_slice()])
        }
    ));
}
