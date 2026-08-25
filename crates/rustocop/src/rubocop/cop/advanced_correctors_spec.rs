use super::advanced_correctors::*;
use super::corrector::Corrector;
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};
use crate::rubocop::ast::token::Token;

fn rewrite<'s>(source: &'s str, apply: impl for<'b> FnOnce(&mut Corrector<'b, 's>)) -> String {
    let buffer = SourceBuffer::new(source);
    let mut corrector = Corrector::new(&buffer);
    apply(&mut corrector);
    corrector.rewrite().unwrap()
}

#[test]
fn each_and_for_corrections_preserve_collection_and_arguments() {
    assert_eq!(
        EachToForCorrector::correction("items", Some("a, b")),
        "for a, b in items do"
    );
    assert_eq!(
        EachToForCorrector::correction("items", None),
        "for _ in items do"
    );
    assert_eq!(
        ForToEachCorrector::correction("a..b", "item", false, true),
        "(a..b).each do |item|"
    );
    assert_eq!(
        ForToEachCorrector::correction("items", "item", true, false),
        "items&.each do |item|"
    );
}

#[test]
fn each_and_for_ast_adapters_rewrite_parser_shaped_nodes() {
    let each_source = "items.each do |item|\n  use(item)\nend\n";
    let each = ProcessedSource::new(each_source, 3.4, None, ParserEngine::Prism).unwrap();
    let block = each.ast().unwrap();
    assert_eq!(block.kind(), "block");
    assert_eq!(
        rewrite(each_source, |corrector| EachToForCorrector::call(
            corrector, block
        )),
        "for item in items do\n  use(item)\nend\n"
    );

    let for_source = "for item in items do\n  use(item)\nend\n";
    let parsed_for = ProcessedSource::new(for_source, 3.4, None, ParserEngine::Prism).unwrap();
    let for_node = parsed_for.ast().unwrap();
    assert_eq!(for_node.kind(), "for");
    assert_eq!(
        rewrite(for_source, |corrector| ForToEachCorrector::call(
            corrector, for_node
        )),
        "items.each do |item|\n  use(item)\nend\n"
    );
}

#[test]
fn if_then_recursively_rewrites_elsif_and_else() {
    let elsif = IfThenBranch {
        keyword: "elsif",
        condition: "b",
        body: Some("two"),
        elsif: true,
        else_branch: None,
        else_source: Some("three"),
    };
    let branch = IfThenBranch {
        keyword: "if",
        condition: "a",
        body: Some("one"),
        elsif: false,
        else_branch: Some(Box::new(elsif)),
        else_source: None,
    };
    assert_eq!(
        IfThenCorrector::replacement(&branch, 0, 2),
        "if a\n  one\nelsif b\n  two\nelse\n  three\nend"
    );
}

#[test]
fn if_then_ast_adapter_rewrites_parsed_nested_branches() {
    let source = "if a then one elsif b then two else three end";
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let node = processed.ast().unwrap();
    assert_eq!(node.kind(), "if");
    assert_eq!(
        rewrite(source, |corrector| IfThenCorrector::call(
            corrector, node, None
        )),
        "if a\n  one\nelsif b\n  two\nelse\n  three\nend"
    );
}

#[test]
fn percent_literals_balance_delimiters_and_upgrade_for_escaping() {
    assert_eq!(
        PercentLiteralCorrector::correction(&["one", "two"], 'w', ('[', ']'), None),
        "%w[one two]"
    );
    assert_eq!(
        PercentLiteralCorrector::correction(&["one two"], 'w', ('(', ')'), None),
        "%w(one two)"
    );
    assert_eq!(
        PercentLiteralCorrector::correction(&["a)b"], 'w', ('(', ')'), None),
        r"%w(a\)b)"
    );
}

#[test]
fn percent_literal_corrector_uses_ast_words_and_original_line_layout() {
    for (source, kind, expected) in [
        ("[\"one\", \"two\"]", 'w', "%w[one two]"),
        ("[\n  \"one\",\n  \"two\"\n]", 'w', "%w[\n  one\n  two\n]"),
        ("[\"one two\"]", 'w', "%w[one two]"),
        ("[:one, :two]", 'i', "%i[one two]"),
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let node = processed.ast().unwrap();
        assert_eq!(
            rewrite(source, |corrector| {
                PercentLiteralCorrector::call(corrector, node, kind, ('[', ']'))
            }),
            expected
        );
    }
}

#[test]
fn space_corrector_adds_both_missing_sides() {
    let output = rewrite("[]", |corrector| {
        let buffer = corrector.source_buffer();
        SpaceCorrector::add_space(
            corrector,
            SourceRange::new(buffer, 0, 1),
            SourceRange::new(buffer, 1, 2),
            false,
            false,
        )
    });
    assert_eq!(output, "[  ]");
}

#[test]
fn space_corrector_exposes_the_upstream_token_contract() {
    let output = rewrite("[  ]", |corrector| {
        let buffer = corrector.source_buffer();
        let left = Token::new(SourceRange::new(buffer, 0, 1), "tLBRACK", "[");
        let right = Token::new(SourceRange::new(buffer, 3, 4), "tRBRACK", "]");
        SpaceCorrector::empty_corrections(corrector, "space", &left, &right);
    });
    assert_eq!(output, "[ ]");

    let output = rewrite("[ ]", |corrector| {
        let buffer = corrector.source_buffer();
        let left = Token::new(SourceRange::new(buffer, 0, 1), "tLBRACK", "[");
        let right = Token::new(SourceRange::new(buffer, 2, 3), "tRBRACK", "]");
        SpaceCorrector::empty_corrections(corrector, "no_space", &left, &right);
    });
    assert_eq!(output, "[]");

    let output = rewrite("a  b", |corrector| {
        let buffer = corrector.source_buffer();
        let left = Token::new(SourceRange::new(buffer, 0, 1), "tIDENTIFIER", "a");
        let right = Token::new(SourceRange::new(buffer, 3, 4), "tIDENTIFIER", "b");
        SpaceCorrector::remove_token_space(corrector, &left, &right);
    });
    assert_eq!(output, "ab");

    let output = rewrite("ab", |corrector| {
        let buffer = corrector.source_buffer();
        let left = Token::new(SourceRange::new(buffer, 0, 1), "tIDENTIFIER", "a");
        let right = Token::new(SourceRange::new(buffer, 1, 2), "tIDENTIFIER", "b");
        SpaceCorrector::add_token_space(corrector, &left, &right);
    });
    assert_eq!(output, "a  b");
}

#[test]
fn ordered_gem_correction_swaps_whole_declarations() {
    let output = rewrite("gem 'z'\ngem 'a'\n", |corrector| {
        let buffer = corrector.source_buffer();
        OrderedGemCorrector::correct(
            corrector,
            SourceRange::new(buffer, 8, 16),
            SourceRange::new(buffer, 0, 8),
        )
    });
    assert_eq!(output, "gem 'a'\ngem 'z'\n");
}

#[test]
fn ordered_gem_corrector_derives_declaration_and_comment_ranges() {
    let source = "# z\ngem \"z\"\n# a\ngem \"a\"\n";
    for (comments_as_separators, expected) in [
        (false, "# a\ngem \"a\"\n# z\ngem \"z\"\n"),
        (true, "# z\ngem \"a\"\n# a\ngem \"z\"\n"),
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let nodes = processed.ast().unwrap().each_node(&["send"]);
        let output = rewrite(source, |corrector| {
            OrderedGemCorrector::call(
                corrector,
                &processed,
                nodes[1],
                nodes[0],
                comments_as_separators,
            )
        });
        assert_eq!(output, expected);
    }
}

#[test]
fn alignment_moves_every_non_taboo_line_by_the_same_delta() {
    let output = rewrite("a\nb\n", |corrector| {
        let buffer = corrector.source_buffer();
        AlignmentCorrector::correct(
            corrector,
            SourceRange::new(buffer, 0, 4),
            2,
            false,
            false,
            &[],
        )
    });
    assert_eq!(output, "  a\n  b\n");
    let unchanged = rewrite("a\n", |corrector| {
        let buffer = corrector.source_buffer();
        AlignmentCorrector::correct(
            corrector,
            SourceRange::new(buffer, 0, 2),
            2,
            true,
            false,
            &[],
        )
    });
    assert_eq!(unchanged, "a\n");
}

#[test]
fn lambda_correction_replaces_selector_delimiters_and_moves_arguments() {
    let output = rewrite("->(x) do x end", |corrector| {
        let buffer = corrector.source_buffer();
        LambdaLiteralToMethodCorrector::correct(
            corrector,
            LambdaCorrection {
                method: SourceRange::new(buffer, 0, 2),
                arguments: Some(SourceRange::new(buffer, 2, 5)),
                block_begin: SourceRange::new(buffer, 6, 8),
                block_end: SourceRange::new(buffer, 11, 14),
                argument_sources: &["x"],
                braces: false,
                convert_do_to_braces: true,
                needs_space: false,
            },
        )
    });
    assert_eq!(output, "lambda { |x| x }");
}

#[test]
fn lambda_corrector_operates_on_the_parser_shaped_prism_adapter() {
    for (source, expected) in [
        ("->(x) do x end", "lambda do |x| x end"),
        ("-> { 1 }", "lambda { 1 }"),
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let block = processed.ast().unwrap();
        assert_eq!(block.kind(), "block");
        assert_eq!(
            rewrite(source, |corrector| LambdaLiteralToMethodCorrector::call(
                corrector, block
            )),
            expected
        );
    }
}

#[test]
fn line_break_corrector_uses_keyword_column_and_width() {
    let output = rewrite("if x then y end", |corrector| {
        let buffer = corrector.source_buffer();
        LineBreakCorrector::break_line_before(corrector, SourceRange::new(buffer, 10, 11), 0, 2, 1)
    });
    assert_eq!(output, "if x then \n  y end");
}

#[test]
fn line_break_corrector_uses_processed_source_tokens_and_comments() {
    for (source, expected) in [
        (
            "class Foo; def x; end; end",
            "class Foo \n  def x; end; end",
        ),
        (
            "class Foo; def x; end; end # explanation",
            "# explanation\nclass Foo \n  def x; end; end ",
        ),
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let node = processed.ast().unwrap();
        let body = node.body().unwrap();
        let tokens = processed.sorted_tokens();
        let semicolon = LineBreakCorrector::semicolon(
            node.source_range().unwrap().start,
            body.first_line(),
            body.column(),
            &tokens,
        )
        .unwrap();
        assert!(LineBreakCorrector::trailing_class_definition(
            semicolon,
            body.column()
        ));
        let output = rewrite(source, |corrector| {
            LineBreakCorrector::correct_trailing_body(corrector, node, &processed, 2)
        });
        assert_eq!(output, expected);
    }
}

#[test]
fn brace_corrector_moves_a_same_line_closer_down() {
    let output = rewrite("[1]", |corrector| {
        let buffer = corrector.source_buffer();
        MultilineLiteralBraceCorrector::move_to_next_line(corrector, SourceRange::new(buffer, 2, 3))
    });
    assert_eq!(output, "[1\n]");
}

#[test]
fn brace_corrector_uses_parsed_literal_children_and_comments() {
    for (source, expected) in [
        ("[1]", "[1\n]"),
        ("[\n  1\n]", "[\n  1]"),
        ("[\n  1, # hi\n]", "[\n  1,] # hi"),
        ("foo([\n  1 # hi\n])", "foo([\n  1 # hi\n])"),
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let node = processed
            .ast()
            .unwrap()
            .each_node(&["array"])
            .into_iter()
            .next()
            .unwrap();
        let output = rewrite(source, |corrector| {
            MultilineLiteralBraceCorrector::call(corrector, node, &processed)
        });
        assert_eq!(output, expected);
    }
}

#[test]
fn parentheses_corrector_removes_delimiters_and_surrounding_spaces() {
    let output = rewrite("( value )", |corrector| {
        let buffer = corrector.source_buffer();
        ParenthesesCorrector::correct(
            corrector,
            SourceRange::new(buffer, 0, 2),
            SourceRange::new(buffer, 7, 9),
            false,
            None,
        )
    });
    assert_eq!(output, "value");
}

#[test]
fn parentheses_corrector_accepts_a_parsed_parenthesized_node() {
    for (source, expected) in [
        ("( value )", "value"),
        ("(\n  foo # hi\n).bar", "foo # hi\n.bar"),
        (
            "condition = (foo) ? one : two",
            "condition = foo ? one : two",
        ),
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let node = processed
            .ast()
            .unwrap()
            .each_node(&["begin"])
            .into_iter()
            .next()
            .unwrap();
        assert_eq!(
            rewrite(source, |corrector| ParenthesesCorrector::call(
                corrector, node
            )),
            expected
        );
    }
}

#[test]
fn public_string_indentation_and_same_line_corrector_contracts_are_executable() {
    let string = ProcessedSource::new("\"value\"", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(AlignmentCorrector::delimited_string_literal(
        string.ast().unwrap()
    ));
    assert_eq!(AlignmentCorrector::indentation_string(3, false), "   ");
    assert_eq!(AlignmentCorrector::indentation_string(2, true), "\t\t");

    assert_eq!(
        rewrite("  end", |corrector| {
            let buffer = corrector.source_buffer();
            AlignmentCorrector::align_end(corrector, SourceRange::new(buffer, 0, 2), 1, false);
        }),
        " end"
    );

    assert_eq!(
        rewrite("item\n  ]", |corrector| {
            let buffer = corrector.source_buffer();
            MultilineLiteralBraceCorrector::move_to_same_line(
                corrector,
                SourceRange::new(buffer, 4, 8),
                SourceRange::new(buffer, 0, 4),
                "]",
                None,
            );
        }),
        "item]"
    );
}
