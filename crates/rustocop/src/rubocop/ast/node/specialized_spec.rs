// Focused contracts derived from rubocop-ast 1.49.1 specialized node specs.

use super::core::{Ast, NodeValue};
use super::specialized::HashElementDelta;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn assignment_and_argument_nodes_preserve_child_layout() {
    let mut ast = Ast::new("x = 1");
    let value = ast.add_node("int", vec![NodeValue::Integer(1)], Some(4..5));
    let assignment = ast.add_node(
        "lvasgn",
        vec![NodeValue::Symbol("x".into()), NodeValue::Node(value)],
        Some(0..5),
    );
    assert_eq!(ast.node(assignment).name(), Some("x"));
    assert_eq!(ast.node(assignment).expression(), Some(ast.node(value)));

    let default = ast.add_node("str", vec![NodeValue::String("v".into())], None);
    let argument = ast.add_node(
        "kwoptarg",
        vec![NodeValue::Symbol("key".into()), NodeValue::Node(default)],
        None,
    );
    assert!(ast.node(argument).default_argument());
    assert!(ast.node(argument).has_default());
    assert_eq!(ast.node(argument).default_value(), Some(ast.node(default)));
}

#[test]
fn public_collection_keyword_and_branch_aliases_preserve_node_contracts() {
    let conditional = ProcessedSource::new(
        "if ready then one elsif other; two else three end",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let conditional = conditional.ast().unwrap();
    assert_eq!(
        conditional.second_node().unwrap().method_name(),
        Some("one")
    );
    assert!(conditional.third_node().is_some());
    assert_eq!(conditional.each_branch().len(), 3);
    assert!(conditional.has_else_keyword());
    assert!(conditional.has_then_keyword());

    let loop_source =
        ProcessedSource::new("while ready do work end", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(loop_source.ast().unwrap().has_do_keyword());

    let case_source = ProcessedSource::new(
        "case value; when 1, 2 then work; end",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let case_node = case_source.ast().unwrap();
    let when_node = case_node.each_when()[0];
    assert_eq!(when_node.each_condition().len(), 2);

    let assignment = ProcessedSource::new("a, b = 1, 2", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(assignment.ast().unwrap().multiple_rhs());

    let hash = ProcessedSource::new("{a: 1, **options}", 3.4, None, ParserEngine::Prism).unwrap();
    let elements = hash.ast().unwrap().child_nodes();
    assert!(elements[1].keyword_splat());
    assert!(elements[0].valid_hash_element_types(elements[1]));
    assert_eq!(elements[0].hash_element_column_delta(elements[0], false), 0);

    let numbered =
        ProcessedSource::new("items.map { _2 }", 3.4, None, ParserEngine::Prism).unwrap();
    assert_eq!(numbered.ast().unwrap().numbered_arguments(), ["_1", "_2"]);
}

#[test]
fn array_and_string_delimiter_predicates_follow_source_maps() {
    let mut ast = Ast::new("%i[a b]");
    let array = ast.add_node("array", Vec::new(), Some(0..7));
    ast.set_location(array, "begin", 0..3, "%i[");
    assert!(ast.node(array).percent_literal(Some("symbol")));
    assert!(ast.node(array).bracketed());

    let string = ast.add_node("str", vec![NodeValue::String("x".into())], None);
    ast.set_location(string, "begin", 0..1, "\"");
    assert!(ast.node(string).double_quoted());
    assert!(!ast.node(string).single_quoted());
}

#[test]
fn parsed_regexp_nodes_preserve_content_delimiters_and_options() {
    let processed = ProcessedSource::new("/foo/im", 3.4, None, ParserEngine::Prism).unwrap();
    let regexp = processed.ast().unwrap();
    assert_eq!(regexp.kind(), "regexp");
    assert_eq!(regexp.content(), "foo");
    assert_eq!(regexp.regopt().unwrap().kind(), "regopt");
    assert!(regexp.ignore_case());
    assert!(regexp.multiline_mode());
    assert_eq!(regexp.options(), 5);
    assert_eq!(regexp.loc("begin").unwrap().1, "/");
    assert_eq!(regexp.loc("end").unwrap().1, "/im");
    assert!(regexp.to_regexp().unwrap().is_match("FOO\n"));
}

#[test]
fn parsed_string_symbol_and_xstring_nodes_use_unescaped_values_and_locations() {
    let processed = ProcessedSource::new(
        "\"a\\n\"; :'two words'; `echo hi`",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let root = processed.ast().unwrap();
    let string = root.each_node(&["str"])[0];
    assert_eq!(string.string_content().as_deref(), Some("a\n"));
    assert!(string.double_quoted());
    let symbol = root.each_node(&["sym"])[0];
    assert_eq!(symbol.name(), Some("two words"));
    assert_eq!(symbol.loc("begin").unwrap().1, ":'");
    let xstring = root.each_node(&["xstr"])[0];
    assert_eq!(xstring.string_content().as_deref(), Some("echo hi"));
    assert_eq!(xstring.loc("begin").unwrap().1, "`");
}

#[test]
fn send_block_and_definition_accessors_match_parser_positions() {
    let mut ast = Ast::new("items.each { |x| x }");
    let recv = ast.add_node("lvar", vec![NodeValue::Symbol("items".into())], None);
    let send = ast.add_node(
        "send",
        vec![NodeValue::Node(recv), NodeValue::Symbol("each".into())],
        None,
    );
    let arg = ast.add_node("arg", vec![NodeValue::Symbol("x".into())], None);
    let args = ast.add_node("args", vec![NodeValue::Node(arg)], None);
    let body = ast.add_node("lvar", vec![NodeValue::Symbol("x".into())], None);
    let block = ast.add_node(
        "block",
        vec![
            NodeValue::Node(send),
            NodeValue::Node(args),
            NodeValue::Node(body),
        ],
        None,
    );
    assert_eq!(ast.node(block).method_name(), Some("each"));
    assert_eq!(ast.node(block).arguments(), vec![ast.node(arg)]);
    assert_eq!(ast.node(block).body(), Some(ast.node(body)));
    assert!(ast.node(block).void_context());

    let defn = ast.add_node(
        "def",
        vec![
            NodeValue::Symbol("call".into()),
            NodeValue::Node(args),
            NodeValue::Node(body),
        ],
        None,
    );
    assert_eq!(ast.node(defn).method_name(), Some("call"));
    assert_eq!(ast.node(defn).body(), Some(ast.node(body)));
}

#[test]
fn conditional_and_case_branches_preserve_nil_and_else_slots() {
    let mut ast = Ast::new("if ok then yes else no end");
    let cond = ast.add_node("lvar", vec![NodeValue::Symbol("ok".into())], None);
    let yes = ast.add_node("sym", vec![NodeValue::Symbol("yes".into())], None);
    let no = ast.add_node("sym", vec![NodeValue::Symbol("no".into())], None);
    let if_node = ast.add_node(
        "if",
        vec![
            NodeValue::Node(cond),
            NodeValue::Node(yes),
            NodeValue::Node(no),
        ],
        None,
    );
    ast.set_location(if_node, "else", 15..19, "else");
    assert_eq!(
        ast.node(if_node).branches(),
        vec![Some(ast.node(yes)), Some(ast.node(no))]
    );
    assert_eq!(ast.node(if_node).condition(), Some(ast.node(cond)));
    assert!(ast.node(if_node).has_else());
}

#[test]
fn hash_pair_delimiters_keys_values_and_omission_are_exact() {
    let mut ast = Ast::new("{a: 1, b => 2}");
    let a = ast.add_node("sym", vec![NodeValue::Symbol("a".into())], Some(1..2));
    let one = ast.add_node("int", vec![NodeValue::Integer(1)], Some(4..5));
    let first = ast.add_node(
        "pair",
        vec![NodeValue::Node(a), NodeValue::Node(one)],
        Some(1..5),
    );
    ast.set_location(first, "operator", 2..3, ":");
    let b = ast.add_node("sym", vec![NodeValue::Symbol("b".into())], Some(7..8));
    let two = ast.add_node("int", vec![NodeValue::Integer(2)], Some(12..13));
    let second = ast.add_node(
        "pair",
        vec![NodeValue::Node(b), NodeValue::Node(two)],
        Some(7..13),
    );
    ast.set_location(second, "operator", 9..11, "=>");
    let hash = ast.add_node(
        "hash",
        vec![NodeValue::Node(first), NodeValue::Node(second)],
        Some(0..14),
    );
    assert_eq!(ast.node(hash).keys(), vec![ast.node(a), ast.node(b)]);
    assert_eq!(ast.node(hash).values(), vec![ast.node(one), ast.node(two)]);
    assert!(ast.node(hash).mixed_delimiters());
    assert_eq!(ast.node(first).delimiter(true), Some(": "));
    assert_eq!(ast.node(second).inverse_delimiter(false), Some(":"));
}

#[test]
fn multiple_assignment_flattens_nested_lhs_and_distinguishes_implicit_arrays() {
    let mut ast = Ast::new("");
    let a = ast.add_node("lvasgn", vec![NodeValue::Symbol("a".into())], None);
    let b = ast.add_node("lvasgn", vec![NodeValue::Symbol("b".into())], None);
    let nested = ast.add_node("mlhs", vec![NodeValue::Node(b)], None);
    let lhs = ast.add_node(
        "mlhs",
        vec![NodeValue::Node(a), NodeValue::Node(nested)],
        None,
    );
    let one = ast.add_node("int", vec![NodeValue::Integer(1)], None);
    let two = ast.add_node("int", vec![NodeValue::Integer(2)], None);
    let rhs = ast.add_node(
        "array",
        vec![NodeValue::Node(one), NodeValue::Node(two)],
        None,
    );
    let masgn = ast.add_node(
        "masgn",
        vec![NodeValue::Node(lhs), NodeValue::Node(rhs)],
        None,
    );
    assert_eq!(ast.node(masgn).assignment_names(), vec!["a", "b"]);
    assert_eq!(ast.node(masgn).values(), vec![ast.node(one), ast.node(two)]);
}

#[test]
fn rescue_and_ensure_nodes_expose_structural_branches() {
    let mut ast = Ast::new("");
    let error = ast.add_node(
        "const",
        vec![NodeValue::Nil, NodeValue::Symbol("Error".into())],
        None,
    );
    let errors = ast.add_node("array", vec![NodeValue::Node(error)], None);
    let body = ast.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("recover".into())],
        None,
    );
    let resbody = ast.add_node(
        "resbody",
        vec![
            NodeValue::Node(errors),
            NodeValue::Nil,
            NodeValue::Node(body),
        ],
        None,
    );
    let main = ast.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("work".into())],
        None,
    );
    let rescue = ast.add_node(
        "rescue",
        vec![
            NodeValue::Node(main),
            NodeValue::Node(resbody),
            NodeValue::Nil,
        ],
        None,
    );
    let cleanup = ast.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("cleanup".into())],
        None,
    );
    let ensure = ast.add_node(
        "ensure",
        vec![NodeValue::Node(rescue), NodeValue::Node(cleanup)],
        None,
    );
    assert_eq!(ast.node(resbody).exceptions(), vec![ast.node(error)]);
    assert_eq!(ast.node(rescue).branch_nodes(), vec![ast.node(resbody)]);
    assert_eq!(ast.node(ensure).ensure_branch(), Some(ast.node(cleanup)));
}

#[test]
fn regex_and_pair_source_map_predicates_are_independent() {
    let mut ast = Ast::new("/abc/im");
    let regexp = ast.add_node("regexp", Vec::new(), Some(0..7));
    ast.set_location(regexp, "begin", 0..1, "/");
    ast.set_location(regexp, "end", 4..7, "/im");
    assert!(ast.node(regexp).slash_literal());
    assert!(ast.node(regexp).regexp_option('i'));
    assert!(ast.node(regexp).regexp_option('m'));
    assert!(!ast.node(regexp).regexp_option('x'));
}

#[test]
fn loops_ranges_and_pattern_branches_use_the_same_indices_as_rubocop_ast() {
    let mut ast = Ast::new("");
    let var = ast.add_node("lvasgn", vec![NodeValue::Symbol("x".into())], None);
    let collection = ast.add_node("array", Vec::new(), None);
    let body = ast.add_node("nil", Vec::new(), None);
    let for_node = ast.add_node(
        "for",
        vec![
            NodeValue::Node(var),
            NodeValue::Node(collection),
            NodeValue::Node(body),
        ],
        None,
    );
    assert_eq!(ast.node(for_node).loop_variable(), Some(ast.node(var)));
    assert_eq!(ast.node(for_node).collection(), Some(ast.node(collection)));

    let range = ast.add_node(
        "irange",
        vec![NodeValue::Node(var), NodeValue::Node(body)],
        None,
    );
    assert_eq!(ast.node(range).range_begin(), Some(ast.node(var)));
    assert_eq!(ast.node(range).range_end(), Some(ast.node(body)));
}

#[test]
fn operator_and_call_classification_matches_specialized_subclasses() {
    let mut ast = Ast::new("");
    let and = ast.add_node("and", Vec::new(), None);
    ast.set_location(and, "operator", 0..2, "&&");
    assert_eq!(ast.node(and).alternate_operator(), Some("and"));
    assert_eq!(ast.node(and).inverse_operator(), Some("||"));
    let csend = ast.add_node(
        "csend",
        vec![NodeValue::Nil, NodeValue::Symbol("foo".into())],
        None,
    );
    assert!(!ast.node(csend).send_type());
    assert_eq!(ast.node(csend).method_name(), Some("foo"));

    let receiver = ast.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("items".into())],
        None,
    );
    let key = ast.add_node("sym", vec![NodeValue::Symbol("key".into())], None);
    let value = ast.add_node("int", vec![NodeValue::Integer(1)], None);
    let index = ast.add_node(
        "index",
        vec![NodeValue::Node(receiver), NodeValue::Node(key)],
        None,
    );
    assert_eq!(ast.node(index).receiver(), Some(ast.node(receiver)));
    assert_eq!(ast.node(index).method_name(), Some("[]"));
    assert_eq!(ast.node(index).arguments(), vec![ast.node(key)]);
    assert!(!ast.node(index).attribute_accessor());
    assert!(!ast.node(index).assignment_method());
    let indexasgn = ast.add_node(
        "indexasgn",
        vec![
            NodeValue::Node(receiver),
            NodeValue::Node(key),
            NodeValue::Node(value),
        ],
        None,
    );
    assert_eq!(ast.node(indexasgn).method_name(), Some("[]="));
    assert_eq!(
        ast.node(indexasgn).arguments(),
        vec![ast.node(key), ast.node(value)]
    );
    assert!(ast.node(indexasgn).assignment_method());
}

#[test]
fn unless_node_parts_and_keyword_predicates_are_normalized() {
    let mut ast = Ast::new("work unless ready");
    let condition = ast.add_node("lvar", vec![NodeValue::Symbol("ready".into())], None);
    let false_branch = ast.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("work".into())],
        None,
    );
    let node = ast.add_node(
        "if",
        vec![
            NodeValue::Node(condition),
            NodeValue::Nil,
            NodeValue::Node(false_branch),
        ],
        Some(0..17),
    );
    ast.set_location(node, "keyword", 5..11, "unless");
    assert!(ast.node(node).unless_keyword());
    assert_eq!(ast.node(node).inverse_keyword(), Some("if"));
    assert_eq!(
        ast.node(node).node_parts(),
        vec![
            NodeValue::Node(condition),
            NodeValue::Node(false_branch),
            NodeValue::Nil
        ]
    );
}

#[test]
fn elsif_branches_flatten_but_nested_regular_conditionals_are_detected() {
    let mut ast = Ast::new("");
    let condition = ast.add_node("true", Vec::new(), None);
    let first = ast.add_node("int", vec![NodeValue::Integer(1)], None);
    let second = ast.add_node("int", vec![NodeValue::Integer(2)], None);
    let elsif = ast.add_node(
        "if",
        vec![
            NodeValue::Node(condition),
            NodeValue::Node(second),
            NodeValue::Nil,
        ],
        None,
    );
    ast.set_location(elsif, "keyword", 0..5, "elsif");
    let outer = ast.add_node(
        "if",
        vec![
            NodeValue::Node(condition),
            NodeValue::Node(first),
            NodeValue::Node(elsif),
        ],
        None,
    );
    ast.set_location(outer, "keyword", 0..2, "if");
    ast.set_location(outer, "else", 0..5, "elsif");
    assert!(ast.node(outer).elsif_conditional());
    assert_eq!(
        ast.node(outer).branches(),
        vec![Some(ast.node(first)), Some(ast.node(second))]
    );
}

#[test]
fn ensure_body_is_the_ensure_branch_and_is_always_void_context() {
    let mut ast = Ast::new("");
    let protected = ast.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("work".into())],
        None,
    );
    let cleanup = ast.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("cleanup".into())],
        None,
    );
    let ensure = ast.add_node(
        "ensure",
        vec![NodeValue::Node(protected), NodeValue::Node(cleanup)],
        None,
    );
    assert_eq!(ast.node(ensure).body(), Some(ast.node(cleanup)));
    assert_eq!(ast.node(ensure).ensure_branch(), Some(ast.node(cleanup)));
    assert!(ast.node(ensure).void_context());
}

#[test]
fn dynamic_string_and_regexp_content_join_only_literal_children() {
    let mut ast = Ast::new("");
    let a = ast.add_node("str", vec![NodeValue::String("a".into())], None);
    let b = ast.add_node("str", vec![NodeValue::String("b".into())], None);
    let embedded = ast.add_node("begin", Vec::new(), None);
    let dstr = ast.add_node(
        "dstr",
        vec![
            NodeValue::Node(a),
            NodeValue::Node(embedded),
            NodeValue::Node(b),
        ],
        None,
    );
    assert_eq!(ast.node(dstr).string_content(), Some("ab".into()));
    let regopt = ast.add_node("regopt", vec![NodeValue::Symbol("i".into())], None);
    let regexp = ast.add_node(
        "regexp",
        vec![
            NodeValue::Node(a),
            NodeValue::Node(embedded),
            NodeValue::Node(regopt),
        ],
        None,
    );
    ast.set_location(regexp, "end", 0..2, "/i");
    assert_eq!(ast.node(regexp).regexp_content(), "a");
    assert!(ast.node(regexp).regexp_interpolation());
    assert!(ast.node(regexp).regexp_ignore_case());
    assert_eq!(ast.node(regexp).regopt(), Some(ast.node(regopt)));
}

#[test]
fn keyword_splat_duck_types_as_both_hash_key_and_value() {
    let mut ast = Ast::new("**opts");
    let opts = ast.add_node("lvar", vec![NodeValue::Symbol("opts".into())], None);
    let splat = ast.add_node("kwsplat", vec![NodeValue::Node(opts)], Some(0..6));
    assert!(ast.node(splat).kwsplat_type());
    assert!(!ast.node(splat).colon());
    assert!(!ast.node(splat).hash_rocket());
    assert_eq!(ast.node(splat).assignment_operator(), Some("**"));
    assert_eq!(ast.node(splat).hash_key(), Some(ast.node(splat)));
    assert_eq!(ast.node(splat).value_node(), Some(ast.node(splat)));
}

#[test]
fn hash_element_deltas_honor_same_line_delimiter_and_keyword_splat_rules() {
    let mut ast = Ast::new("a: 1\nlong: 2");
    let a = ast.add_node("sym", Vec::new(), Some(0..1));
    let one = ast.add_node("int", Vec::new(), Some(3..4));
    let first = ast.add_node(
        "pair",
        vec![NodeValue::Node(a), NodeValue::Node(one)],
        Some(0..4),
    );
    ast.set_location(first, "operator", 1..2, ":");
    let long = ast.add_node("sym", Vec::new(), Some(5..9));
    let two = ast.add_node("int", Vec::new(), Some(11..12));
    let second = ast.add_node(
        "pair",
        vec![NodeValue::Node(long), NodeValue::Node(two)],
        Some(5..12),
    );
    ast.set_location(second, "operator", 9..10, ":");
    let delta = HashElementDelta::initialize(ast.node(first), ast.node(second)).unwrap();
    assert!(delta.valid_argument_types());
    assert_eq!(ast.node(first).key_delta(ast.node(second), false), 0);
    assert_eq!(ast.node(first).value_delta(ast.node(second)), -3);
    assert_eq!(ast.node(first).delimiter_delta(ast.node(second)), -3);
}

#[test]
fn argument_and_forwarding_collection_edge_cases_match() {
    let mut ast = Ast::new("");
    let args = ast.add_node("args", Vec::new(), None);
    assert!(ast.node(args).empty_and_without_delimiters());
    let forward = ast.add_node("forward_args", Vec::new(), None);
    assert_eq!(
        ast.node(forward).forwarded_arguments(),
        vec![ast.node(forward)]
    );
}

#[test]
fn normalized_super_yield_and_keyword_splat_node_parts_include_dispatch_shape() {
    let mut ast = Ast::new("");
    let value = ast.add_node("int", vec![NodeValue::Integer(1)], None);
    let super_node = ast.add_node("super", vec![NodeValue::Node(value)], None);
    let yield_node = ast.add_node("yield", vec![NodeValue::Node(value)], None);
    assert_eq!(
        ast.node(super_node).node_parts(),
        vec![
            NodeValue::Nil,
            NodeValue::Symbol("super".into()),
            NodeValue::Node(value)
        ]
    );
    assert_eq!(
        ast.node(yield_node).node_parts(),
        vec![
            NodeValue::Nil,
            NodeValue::Symbol("yield".into()),
            NodeValue::Node(value)
        ]
    );
}
