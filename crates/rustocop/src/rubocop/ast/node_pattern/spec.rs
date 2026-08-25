// Consolidated behavioral ports from rubocop-ast 1.49.1:
// spec/rubocop/ast/node_pattern_spec.rb
// Spec SHA-256: fcfa8e8f97a7fec1e8c673a6a668ceac1a2efa557c919c5946d38b8895a81612
// spec/rubocop/ast/node_pattern/lexer_spec.rb
// Spec SHA-256: 882dd1a17c6ffd37c3eb39b37fc59fefb30d74de9d19f1b0caad6ac8e45b80dd
// spec/rubocop/ast/node_pattern/parser_spec.rb
// Spec SHA-256: 9a974e72b76aee85777f5bf92ba4a61c64b5ede8c54dbdf9b37bf05d2647999c
// spec/rubocop/ast/node_pattern/sets_spec.rb
// Spec SHA-256: 05a937265402f243c4fff50d1dca707aae4e9136174661bebb2089c70d5a8e34

use super::*;
use crate::rubocop::ast::node::core::{Ast, NodeValue};
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};

#[test]
fn initialize_and_defined_matcher_share_the_compiled_pattern() {
    let parsed = ProcessedSource::new("1", 3.4, None, ParserEngine::Prism).unwrap();
    let pattern = NodePattern::initialize("int").unwrap();
    assert!(pattern
        .def_node_matcher()
        .call(parsed.ast().unwrap())
        .is_some());
}

fn send_ast() -> (Ast, crate::rubocop::ast::node::core::NodeId) {
    let mut ast = Ast::new("obj.foo(42, :bar)");
    let receiver = ast.add_node("lvar", vec![NodeValue::Symbol("obj".into())], None);
    let integer = ast.add_node("int", vec![NodeValue::Integer(42)], None);
    let symbol = ast.add_node("sym", vec![NodeValue::Symbol("bar".into())], None);
    let send = ast.add_node(
        "send",
        vec![
            NodeValue::Node(receiver),
            NodeValue::Symbol("foo".into()),
            NodeValue::Node(integer),
            NodeValue::Node(symbol),
        ],
        None,
    );
    (ast, send)
}

#[test]
fn lexer_distinguishes_function_argument_lists_sequences_and_comments() {
    let tokens = lex("(send nil? #func(:foo) #func (bar)) # note").unwrap();
    assert!(matches!(tokens[0].kind, TokenKind::Punctuation('(')));
    assert!(matches!(tokens[3].kind,TokenKind::Function(ref name) if name=="func"));
    assert!(matches!(tokens[4].kind, TokenKind::ArgumentList));
    let second = tokens
        .iter()
        .rposition(|token| matches!(token.kind, TokenKind::Function(_)))
        .unwrap();
    assert!(matches!(
        tokens[second + 1].kind,
        TokenKind::Punctuation('(')
    ));
    assert!(!tokens
        .iter()
        .any(|token| matches!(token.kind, TokenKind::Comment(_))));
}

#[test]
fn lexer_preserves_regexp_symbols_numbers_parameters_and_unification() {
    let tokens = lex(r#"/:foo/i :& -2.5 %2 %name %CONST _same _"#).unwrap();
    assert!(
        matches!(tokens[0].kind,TokenKind::Regexp(ref body,ref opts) if body==":foo"&&opts=="i")
    );
    assert!(matches!(tokens[1].kind,TokenKind::Symbol(ref value) if value=="&"));
    assert!(matches!(tokens[2].kind,TokenKind::Number(ref value) if value=="-2.5"));
    assert!(matches!(tokens[3].kind, TokenKind::PositionalParameter(2)));
    assert!(matches!(tokens[4].kind,TokenKind::NamedParameter(ref value) if value=="name"));
    assert!(matches!(tokens[6].kind,TokenKind::Unify(ref value) if value=="same"));
}

#[test]
fn parser_builds_sequence_capture_repetition_union_and_intersection_nodes() {
    let pattern = NodePattern::new("(_ $int* ($str)+ [{:a | :b} !nil?])").unwrap();
    let Expr::Sequence(items) = pattern.expression() else {
        panic!("sequence")
    };
    assert!(matches!(items[0], Expr::Wildcard));
    assert!(matches!(items[1], Expr::Repetition(_, Repeat::ZeroOrMore)));
    assert!(matches!(items[2], Expr::Repetition(_, Repeat::OneOrMore)));
    assert!(matches!(items[3], Expr::Intersection(_)));
}

#[test]
fn bare_node_types_and_hyphenated_types_match_only_nodes() {
    let (ast, send) = send_ast();
    assert!(NodePattern::new("send")
        .unwrap()
        .matches(ast.node(send))
        .is_some());
    assert!(NodePattern::new("ivar")
        .unwrap()
        .matches(ast.node(send))
        .is_none());
    let mut other = Ast::new("");
    let op = other.add_node("op_asgn", Vec::new(), None);
    assert!(NodePattern::new("op-asgn")
        .unwrap()
        .matches(other.node(op))
        .is_some());
}

#[test]
fn sequence_matches_node_head_literals_and_rest() {
    let (ast, send) = send_ast();
    assert!(NodePattern::new("(send _ :foo ...)")
        .unwrap()
        .matches(ast.node(send))
        .is_some());
    assert!(NodePattern::new("(send nil? :foo ...)")
        .unwrap()
        .matches(ast.node(send))
        .is_none());
    assert!(NodePattern::new("(send _ {:foo | :bar} ...)")
        .unwrap()
        .matches(ast.node(send))
        .is_some());
}

#[test]
fn captures_return_values_in_pattern_order() {
    let (ast, send) = send_ast();
    let captures = NodePattern::new("(send $_ :foo $(int $_) ...)")
        .unwrap()
        .matches(ast.node(send))
        .unwrap();
    assert_eq!(captures.len(), 3);
    assert!(matches!(captures[0],MatchValue::Node(node) if node.kind()=="lvar"));
    assert!(matches!(captures[1], MatchValue::Integer(42)));
    assert!(matches!(captures[2],MatchValue::Node(node) if node.kind()=="int"));
}

#[test]
fn repetition_backtracks_to_allow_trailing_terms() {
    let (ast, send) = send_ast();
    assert!(NodePattern::new("(send _ :foo _* (sym :bar))")
        .unwrap()
        .matches(ast.node(send))
        .is_some());
    assert!(NodePattern::new("(send _ :foo _+ (int 99))")
        .unwrap()
        .matches(ast.node(send))
        .is_none());
}

#[test]
fn named_positional_and_constant_parameters_are_contextual() {
    let (ast, send) = send_ast();
    let mut context = MatchContext::default();
    context.positional.push(MatchValue::Symbol("foo"));
    context
        .named
        .insert("method".into(), MatchValue::Symbol("foo"));
    context.constants.insert(
        "METHODS".into(),
        vec![MatchValue::Symbol("foo"), MatchValue::Symbol("bar")],
    );
    for source in [
        "(send _ %1 ...)",
        "(send _ %method ...)",
        "(send _ %METHODS ...)",
    ] {
        assert!(
            NodePattern::new(source)
                .unwrap()
                .matches_with(ast.node(send), &context)
                .is_some(),
            "{source}"
        );
    }
}

#[test]
fn unification_requires_the_same_later_value() {
    let mut ast = Ast::new("");
    let pair = ast.add_node(
        "pair",
        vec![NodeValue::Symbol("x".into()), NodeValue::Symbol("x".into())],
        None,
    );
    assert!(NodePattern::new("(pair _same _same)")
        .unwrap()
        .matches(ast.node(pair))
        .is_some());
    let mismatch = ast.add_node(
        "pair",
        vec![NodeValue::Symbol("x".into()), NodeValue::Symbol("y".into())],
        None,
    );
    assert!(NodePattern::new("(pair _same _same)")
        .unwrap()
        .matches(ast.node(mismatch))
        .is_none());
}

#[test]
fn ascend_descend_and_search_use_arena_relationships() {
    let (mut ast, send) = send_ast();
    ast.complete(send);
    let integer = ast
        .node(send)
        .child_nodes()
        .into_iter()
        .find(|node| node.kind() == "int")
        .unwrap();
    assert!(NodePattern::new("^(send ...)")
        .unwrap()
        .matches(integer)
        .is_some());
    assert!(NodePattern::new("`(sym :bar)")
        .unwrap()
        .matches(ast.node(send))
        .is_some());
    assert_eq!(
        NodePattern::new("int")
            .unwrap()
            .search(ast.node(send))
            .len(),
        1
    );
}

#[test]
fn invalid_patterns_are_rejected_before_matching() {
    for source in ["[]", "(send", "{send const", "(send ... ...)", "(send !)"] {
        assert!(NodePattern::new(source).is_err(), "{source}");
    }
}

#[test]
fn lexer_round_trips_regexps_and_distinguishes_qualified_constants() {
    for (source, body, options) in [
        ("/test/", "test", ""),
        (r"/[abc]+\/()?/x", r"[abc]+\/()?", "x"),
        (r"/back\\slash/", r"back\\slash", ""),
    ] {
        let tokens = lex(source).unwrap();
        assert!(matches!(
            &tokens[0].kind,
            TokenKind::Regexp(actual_body, actual_options)
                if actual_body == body && actual_options == options
        ));
    }
    assert!(lex(r"/tricky\/").is_err());

    let tokens = lex("(aa bb Cc DD ::Ee Ff::GG %::Hh Zz %Zz)").unwrap();
    let kinds = tokens
        .iter()
        .filter_map(|token| match &token.kind {
            TokenKind::NodeType(_) => Some("node"),
            TokenKind::Constant(_) => Some("constant"),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        kinds,
        vec![
            "node", "node", "constant", "constant", "constant", "constant", "constant", "constant",
            "constant"
        ]
    );
    let tokens = lex("(array sym $int+ x)").unwrap();
    assert!(matches!(tokens[3].kind, TokenKind::Punctuation('$')));
    assert!(matches!(tokens[4].kind, TokenKind::NodeType(ref name) if name == "int"));
    assert!(matches!(tokens[5].kind, TokenKind::Punctuation('+')));
}

#[test]
fn parser_handles_function_arguments_deep_rest_unions_and_literal_sets() {
    assert!(matches!(
        NodePattern::new("#func(1, 2, 3)").unwrap().expression(),
        Expr::Function(name, arguments) if name == "func" && arguments.len() == 3
    ));
    let expression = parse_expression("({a | b ... | ... c | $...})").unwrap();
    let Expr::Sequence(items) = &expression else {
        panic!("outer sequence")
    };
    assert!(matches!(&items[0], Expr::Union(branches) if branches.len() == 4));

    let pattern = NodePattern::new("({:a 42 \"hello\"})").unwrap();
    let Expr::Sequence(items) = pattern.expression() else {
        panic!("set sequence")
    };
    assert!(matches!(&items[0], Expr::Union(branches) if branches.len() == 3));
}

#[test]
fn set_names_are_stable_order_independent_and_bounded() {
    let forward = (1..=6).map(|value| value.to_string()).collect::<Vec<_>>();
    let reverse = (1..=6)
        .rev()
        .map(|value| value.to_string())
        .collect::<Vec<_>>();
    let mut registry = SetRegistry::default();
    assert_eq!(registry.name_for(&forward), registry.name_for(&reverse));
    assert_eq!(registry.name_for(&forward), "SET_1_2_3_ETC");
    assert_ne!(
        registry.name_for(&forward),
        registry.name_for(&(1..=7).map(|value| value.to_string()).collect::<Vec<_>>())
    );
}

#[test]
fn repetition_and_rest_captures_are_grouped_like_ruby_arrays() {
    let mut ast = Ast::new("foo(1, 2)");
    let one = ast.add_node("int", vec![NodeValue::Integer(1)], None);
    let two = ast.add_node("int", vec![NodeValue::Integer(2)], None);
    let send = ast.add_node(
        "send",
        vec![
            NodeValue::Nil,
            NodeValue::Symbol("foo".into()),
            NodeValue::Node(one),
            NodeValue::Node(two),
        ],
        None,
    );
    let captures = NodePattern::new("(send _ $_ (int $_)*)")
        .unwrap()
        .matches(ast.node(send))
        .unwrap();
    assert_eq!(captures[0], MatchValue::Symbol("foo"));
    assert_eq!(
        captures[1],
        MatchValue::Array(vec![MatchValue::Integer(1), MatchValue::Integer(2)])
    );

    let captures = NodePattern::new("(send _ $_ $...)")
        .unwrap()
        .matches(ast.node(send))
        .unwrap();
    assert_eq!(captures[0], MatchValue::Symbol("foo"));
    assert!(matches!(&captures[1], MatchValue::Array(values) if values.len() == 2));

    let empty = Ast::new("");
    let mut empty = empty;
    let call = empty.add_node(
        "send",
        vec![NodeValue::Nil, NodeValue::Symbol("foo".into())],
        None,
    );
    let captures = NodePattern::new("(send _ $_ (int $_)*)")
        .unwrap()
        .matches(empty.node(call))
        .unwrap();
    assert_eq!(captures[1], MatchValue::Array(Vec::new()));
}

#[test]
fn numeric_string_symbol_nil_union_negation_and_intersection_semantics_match() {
    for (source, pattern) in [
        ("-100", "(int -100)"),
        ("1.0", "(float 1.0)"),
        ("-2.5", "(float -2.5)"),
        ("\"foo\"", "(str \"foo\")"),
        ("'foo'", "(str \"foo\")"),
        ("\"\"", "(str \"\")"),
        (":foo", "(sym :foo)"),
        ("foo", "(send nil? :foo)"),
    ] {
        let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        assert!(
            NodePattern::new(pattern)
                .unwrap()
                .matches(parsed.ast().unwrap())
                .is_some(),
            "{source} vs {pattern}"
        );
    }

    let parsed = ProcessedSource::new("foo(42)", 3.4, None, ParserEngine::Prism).unwrap();
    let node = parsed.ast().unwrap();
    assert!(NodePattern::new("{(send ...) (int ...)}")
        .unwrap()
        .matches(node)
        .is_some());
    assert!(NodePattern::new("[send !nil?]")
        .unwrap()
        .matches(node)
        .is_some());
    assert!(NodePattern::new("!int").unwrap().matches(node).is_some());
}

#[test]
fn rest_backtracks_and_parameter_zero_refers_to_the_original_target() {
    let parsed = ProcessedSource::new("1 + 10", 3.4, None, ParserEngine::Prism).unwrap();
    let root = parsed.ast().unwrap();
    let ten = root.arguments().last().copied().unwrap();
    let captures = NodePattern::new("(send $... %1)").unwrap().matches_with(
        ten,
        &MatchContext {
            positional: vec![MatchValue::Node(ten)],
            ..MatchContext::default()
        },
    );
    assert!(captures.is_none());

    let one = root.receiver().unwrap();
    assert!(NodePattern::new("^(send %0 :+ (int 10))")
        .unwrap()
        .matches(one)
        .is_some());

    let mut context = MatchContext::default();
    context.positional.push(MatchValue::Node(ten));
    let captures = NodePattern::new("(send $... %1)")
        .unwrap()
        .matches_with(root, &context)
        .unwrap();
    assert!(matches!(&captures[0], MatchValue::Array(values) if values.len() == 2));
}

#[test]
fn any_order_matches_backtracks_captures_rest_and_unifies_structurally() {
    let parsed = ProcessedSource::new(
        "[:hello, \"world\", 1, 2, 3]",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let root = parsed.ast().unwrap();
    let captures = NodePattern::new("(array <(str $_) (int 1) (int 3) (int $_) $_>)")
        .unwrap()
        .matches(root)
        .unwrap();
    assert_eq!(captures[0], MatchValue::String("world"));
    assert_eq!(captures[1], MatchValue::Integer(2));
    assert!(matches!(captures[2], MatchValue::Node(node) if node.kind() == "sym"));
    assert!(
        NodePattern::new("(array <(str $_) (int 1) (int 3) (int $_)>)")
            .unwrap()
            .matches(root)
            .is_none()
    );

    let rest = NodePattern::new("(array <(str \"world\") (int 2) $...>)")
        .unwrap()
        .matches(root)
        .unwrap();
    assert!(matches!(&rest[0], MatchValue::Array(values) if values.len() == 3));

    let whole = NodePattern::new("(array sym $<int int _ _>)")
        .unwrap()
        .matches(root)
        .unwrap();
    assert!(matches!(&whole[0], MatchValue::Array(values) if values.len() == 4));

    for source in ["foo.bar || foo.baz", "foo.baz || foo.bar"] {
        let parsed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        assert!(
            NodePattern::new("(or <(send _receiver :bar) (send _receiver :baz)>)")
                .unwrap()
                .matches(parsed.ast().unwrap())
                .is_some(),
            "{source}"
        );
    }
    let mismatch =
        ProcessedSource::new("foo.bar || bar.baz", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(
        NodePattern::new("(or <(send _receiver :bar) (send _receiver :baz)>)")
            .unwrap()
            .matches(mismatch.ast().unwrap())
            .is_none()
    );
}

#[test]
fn predicate_arguments_and_repeated_nested_captures_match_upstream_shapes() {
    let mut context = MatchContext {
        positional: vec![MatchValue::Integer(1)],
        ..MatchContext::default()
    };
    let parsed = ProcessedSource::new("1 + 2", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(NodePattern::new("(send (int equal?(%1)) ...)")
        .unwrap()
        .matches_with(parsed.ast().unwrap(), &context)
        .is_some());

    let parsed = ProcessedSource::new("\"c\"", 3.4, None, ParserEngine::Prism).unwrap();
    context.positional = vec![MatchValue::String("a"), MatchValue::String("d")];
    assert!(NodePattern::new("(str between?(%1, %2))")
        .unwrap()
        .matches_with(parsed.ast().unwrap(), &context)
        .is_some());

    let parsed = ProcessedSource::new(
        "[[:hello, 1, 2, 3], [:world, 3, 4]]",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let captures = NodePattern::new("(array (array (sym $_) (int $_)*)*)")
        .unwrap()
        .matches(parsed.ast().unwrap())
        .unwrap();
    assert!(matches!(&captures[0], MatchValue::Array(values) if values.len() == 2));
    assert!(matches!(&captures[1], MatchValue::Array(values)
        if matches!(&values[0], MatchValue::Array(inner) if inner.len() == 3)
            && matches!(&values[1], MatchValue::Array(inner) if inner.len() == 2)));
}

#[test]
fn invalid_any_order_and_comma_grammar_is_rejected() {
    for pattern in [
        "(<(str $_) (sym $_)> ...)",
        "(array <(str $_) ... (sym $_)>)",
        "(array <(str $_) <int sym>> ...)",
        ",,(,send,, ,int,:+, int ), ",
    ] {
        assert!(NodePattern::new(pattern).is_err(), "{pattern}");
    }
}

#[test]
fn pattern_metadata_results_equality_search_and_descend_match_public_contracts() {
    let left = NodePattern::new("  (send  42 \n :to_s ) ").unwrap();
    let right = NodePattern::new("(send 42 :to_s)").unwrap();
    assert_eq!(left, right);
    assert!(left.equivalent(&right));
    assert_eq!(right.match_code(), right.ast());
    assert_ne!(left, NodePattern::new("(send)").unwrap());
    assert_eq!(right.serialized_pattern(), "(send 42 :to_s)");
    assert_eq!(
        NodePattern::from_serialized_pattern(right.serialized_pattern()).unwrap(),
        right
    );
    assert_eq!(
        right.description(),
        "#<RuboCop::AST::NodePattern (send 42 :to_s)>"
    );
    assert_eq!(right.to_string(), right.description());
    assert!(std::ptr::eq(right.freeze(), &right));

    let pattern = NodePattern::new("(send $%receiver %method $...)").unwrap();
    assert_eq!(pattern.captures(), 2);
    assert_eq!(pattern.named_parameters(), ["method", "receiver"]);
    assert!(pattern.positional_parameters().is_empty());

    let parsed = ProcessedSource::new("[[1, 2], 3]", 3.4, None, ParserEngine::Prism).unwrap();
    let descended = NodePattern::descend(parsed.ast().unwrap());
    assert_eq!(descended.len(), 8);
    assert!(matches!(descended[0], MatchValue::Node(node) if node.kind() == "array"));
    assert_eq!(descended.last(), Some(&MatchValue::Integer(3)));

    assert_eq!(
        NodePattern::new("array")
            .unwrap()
            .match_result(parsed.ast().unwrap()),
        Some(MatchResult::Matched)
    );
    assert_eq!(
        NodePattern::new("(array $_ ...)")
            .unwrap()
            .match_result(parsed.ast().unwrap()),
        Some(MatchResult::Capture(MatchValue::Node(
            parsed.ast().unwrap().node_child(0).unwrap()
        )))
    );

    let mut context = MatchContext::default();
    context.constants.insert(
        "TYPES".into(),
        vec![MatchValue::Node(parsed.ast().unwrap())],
    );
    assert_eq!(
        NodePattern::new("%TYPES")
            .unwrap()
            .search_with(parsed.ast().unwrap(), &context)
            .len(),
        1
    );
}

#[test]
fn custom_function_hooks_receive_literal_parameter_and_pattern_arguments() {
    struct Functions;
    impl<'ast> PatternFunctions<'ast> for Functions {
        fn call(
            &self,
            name: &str,
            value: MatchValue<'ast>,
            arguments: &[MatchValue<'ast>],
        ) -> bool {
            match name {
                "goodmatch?" => true,
                "witharg?" => arguments.first() == Some(&value),
                "some_function?" => arguments == [MatchValue::Boolean(true)],
                "Namespace.helper?" => arguments == [MatchValue::OwnedSymbol("ok".into())],
                _ => false,
            }
        }
    }

    let parsed = ProcessedSource::new("a = 1", 3.4, None, ParserEngine::Prism).unwrap();
    let functions = Functions;
    let mut context = MatchContext {
        functions: Some(&functions),
        ..MatchContext::default()
    };
    assert!(NodePattern::new("(lvasgn #goodmatch? ...)")
        .unwrap()
        .matches_with(parsed.ast().unwrap(), &context)
        .is_some());

    let parsed = ProcessedSource::new("\"foo\"", 3.4, None, ParserEngine::Prism).unwrap();
    context.positional.push(MatchValue::String("foo"));
    assert!(NodePattern::new("(str #witharg?(%1))")
        .unwrap()
        .matches_with(parsed.ast().unwrap(), &context)
        .is_some());

    let parsed = ProcessedSource::new("2 + 2.0", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(NodePattern::new(
        "(send (int _value) :+ #some_function?({(int _value) (float _value)}))",
    )
    .unwrap()
    .matches_with(parsed.ast().unwrap(), &context)
    .is_some());

    assert!(NodePattern::new("#Namespace.helper?(:ok)")
        .unwrap()
        .matches_with(parsed.ast().unwrap(), &context)
        .is_some());
}
