// Consolidated behavioral port from rubocop-ast 1.49.1:
// spec/rubocop/ast/processed_source_spec.rb
// Spec SHA-256: 2518178fb4a5fe278e79cb998f0bfa0cfb2a4a355a583a4d43dbb154e8dcf374

use super::processed_source::{
    default_parser_engine, OwnedProcessedSource, ParserEngine, ProcessedSource,
};

#[test]
fn parser_and_builder_selection_preserve_each_version_boundary() {
    assert_eq!(
        ProcessedSource::builder_class(ParserEngine::Whitequark),
        "RuboCop::AST::Builder"
    );
    assert_eq!(
        ProcessedSource::parser_class(3.4, ParserEngine::Whitequark).unwrap(),
        "Parser::Ruby34"
    );
    assert_eq!(
        ProcessedSource::parser_class(4.1, ParserEngine::Prism).unwrap(),
        "Prism::Translation::Parser41"
    );
    assert!(ProcessedSource::parser_class(3.2, ParserEngine::Prism).is_err());
    let descriptor = ProcessedSource::create_parser(3.4, ParserEngine::Prism, true).unwrap();
    assert!(descriptor.reuses_prism_result);
    assert!(ProcessedSource::parse("1", 3.4, ParserEngine::Prism)
        .unwrap()
        .valid_syntax());
    assert!(ProcessedSource::parse_lex("1", 3.4).unwrap().valid_syntax());
    assert!(
        ProcessedSource::initialize("1", 3.4, None, ParserEngine::Prism)
            .unwrap()
            .valid_syntax()
    );
}

const SOURCE: &str = "# an awesome method\ndef some_method\n  puts 'foo'\nend\nsome_method\n";

fn source() -> ProcessedSource<'static> {
    ProcessedSource::new(
        SOURCE,
        3.4,
        Some("ast/and_node_spec.rb".into()),
        ParserEngine::Default,
    )
    .unwrap()
}

#[test]
fn normalizes_default_and_explicit_parser_engines() {
    assert_eq!(default_parser_engine(3.3), ParserEngine::Whitequark);
    assert_eq!(default_parser_engine(3.4), ParserEngine::Prism);
    assert_eq!(
        ProcessedSource::new("true", 3.3, None, ParserEngine::Default)
            .unwrap()
            .parser_engine(),
        ParserEngine::Whitequark
    );
    assert_eq!(
        ProcessedSource::new("true", 3.4, None, ParserEngine::Default)
            .unwrap()
            .parser_engine(),
        ParserEngine::Prism
    );
    assert_eq!(
        ProcessedSource::new("true", 3.4, None, ParserEngine::Whitequark)
            .unwrap()
            .parser_engine(),
        ParserEngine::Whitequark
    );
    assert_eq!(
        ProcessedSource::new("true", 3.3, None, ParserEngine::Prism)
            .unwrap()
            .parser_engine(),
        ParserEngine::Prism
    );
}

#[test]
fn exposes_path_buffer_lines_and_indexing() {
    let processed = source();
    assert_eq!(
        processed.path().unwrap().to_str(),
        Some("ast/and_node_spec.rb")
    );
    assert_eq!(processed.file_path(), "ast/and_node_spec.rb");
    assert_eq!(processed.buffer().source(), SOURCE);
    assert_eq!(processed.lines().len(), 6);
    assert_eq!(&processed[0], "# an awesome method");
    assert_eq!(processed.line(0), Some("# an awesome method"));
    assert_eq!(&processed[3], "end");
    assert_eq!(processed.ruby_version(), 3.4);
    assert_eq!(processed.raw_source(), SOURCE);
}

#[test]
fn syntax_diagnostics_distinguish_valid_and_invalid_source() {
    let valid =
        ProcessedSource::new("def valid_code; end", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(valid.valid_syntax());
    assert!(valid.diagnostics().is_empty());
    assert!(valid.parser_error().is_none());
    let invalid =
        ProcessedSource::new("def invalid_code; en", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(!invalid.valid_syntax());
    assert!(!invalid.diagnostics().is_empty());
}

#[test]
fn comments_are_indexed_and_enumerated_by_line() {
    let source = "# comment one\n[ 1,\n  { a: 2,\n    b: 3 # comment two\n  }\n]\n";
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    assert_eq!(processed.comments().len(), 2);
    assert_eq!(processed.each_comment().count(), 2);
    assert_eq!(processed.comment_at_line(1).unwrap().text, "# comment one");
    assert_eq!(processed.comment_at_line(4).unwrap().text, "# comment two");
    assert!(processed.line_with_comment(1));
    assert!(!processed.line_with_comment(3));
    assert_eq!(processed.each_comment_in_lines(1..5).len(), 2);
    assert!(processed.contains_comment(2, 4));
    assert_eq!(processed.comments_before_line(3).len(), 1);
}

#[test]
fn owned_processed_source_reads_and_parses_a_file_with_its_path() {
    let path = std::env::temp_dir().join(format!(
        "rustocop-processed-source-{}.rb",
        std::process::id()
    ));
    std::fs::write(&path, "value = 1\n").unwrap();
    let owned = OwnedProcessedSource::from_file(&path, 3.4, ParserEngine::Prism).unwrap();
    let processed = owned.processed().unwrap();
    assert_eq!(processed.raw_source(), "value = 1\n");
    assert_eq!(processed.path(), Some(path.as_path()));
    std::fs::remove_file(path).unwrap();
}

#[test]
fn ast_comment_association_keeps_leading_groups_and_inline_comments_with_one_node() {
    let source = "# first\n# second\ngem 'alpha'\ngem 'beta' # inline\n";
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let root = processed.ast().unwrap();
    let calls = root.each_descendant(&["send"]);
    assert_eq!(
        processed
            .comments_for(calls[0])
            .iter()
            .map(|comment| comment.text.as_str())
            .collect::<Vec<_>>(),
        ["# first", "# second"]
    );
    assert_eq!(
        processed
            .comments_for(calls[1])
            .iter()
            .map(|comment| comment.text.as_str())
            .collect::<Vec<_>>(),
        ["# inline"]
    );
    let associated = processed.ast_with_comments();
    assert_eq!(associated.len(), 2);
    assert_eq!(
        associated
            .iter()
            .map(|(_, comments)| comments.len())
            .sum::<usize>(),
        3
    );
}

#[test]
fn lexical_tokens_cover_enumeration_and_line_navigation() {
    let processed = ProcessedSource::new("foo(1, 2)\n", 3.4, None, ParserEngine::Prism).unwrap();
    assert_eq!(processed.tokens().len(), 7);
    assert_eq!(processed.tokens()[0].text, "foo");
    assert_eq!(
        processed
            .tokens()
            .iter()
            .find(|token| token.comma())
            .unwrap()
            .text,
        ","
    );
    assert!(processed.tokens().last().unwrap().kind == "tNL");

    let processed = ProcessedSource::new(
        "[ line, 1 ]\n{ line: 2 }\n# line 3\n",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let brace = processed
        .tokens()
        .iter()
        .find(|token| token.left_brace())
        .unwrap();
    assert_eq!(processed.preceding_line(brace), Some("[ line, 1 ]"));
    assert_eq!(processed.following_line(brace), Some("# line 3"));
    let comment = processed
        .tokens()
        .iter()
        .find(|token| token.comment())
        .unwrap();
    assert_eq!(processed.current_line(comment), Some("# line 3"));
}

#[test]
fn tokens_within_use_sorted_character_ranges() {
    let processed =
        ProcessedSource::new("foo(1, 2)\nbar(3)\n", 3.4, None, ParserEngine::Prism).unwrap();
    let start = "foo(1, 2)\n".chars().count();
    let end = start + "bar(3)".chars().count();
    let tokens = processed.tokens_within(start..end);
    assert_eq!(
        tokens
            .iter()
            .map(|token| token.text.as_str())
            .collect::<Vec<_>>(),
        ["bar", "(", "3", ")"]
    );
    assert_eq!(processed.first_token_of(start..end).unwrap().text, "bar");
    assert_eq!(processed.last_token_of(start..end).unwrap().text, ")");
    assert_eq!(processed.first_token_index(start..end), Some(7));
    assert_eq!(processed.last_token_index(start..end), Some(10));
    assert_eq!(ProcessedSource::source_range(start..end), start..end);
    assert_eq!(processed.each_token().count(), processed.tokens().len());
}

#[test]
fn blank_start_indentation_checksum_and_end_marker_match_contracts() {
    let blank = ProcessedSource::new("", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(blank.blank());
    assert!(!blank.start_with(""));
    let present =
        ProcessedSource::new("  foo\n__END__\ndata\n", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(!present.blank());
    assert!(present.start_with("  f"));
    assert_eq!(present.line_indentation(1), 2);
    assert!(!present.lines().iter().any(|line| line == "data"));
    assert_eq!(
        ProcessedSource::new("abc", 3.4, None, ParserEngine::Prism)
            .unwrap()
            .checksum(),
        "a9993e364706816aba3e25717850c26c9cd0d89d"
    );
}

#[test]
fn unicode_positions_are_character_based() {
    let processed = ProcessedSource::new("é = 1 # β\n", 3.4, None, ParserEngine::Prism).unwrap();
    let comment = processed.comments().first().unwrap();
    assert_eq!(comment.range, 6..9);
    assert_eq!(comment.line, 1);
}

#[test]
fn exposes_parser_shaped_ast_with_parent_links() {
    let processed = ProcessedSource::new(
        "items.each { |item| puts(item) }",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let root = processed.ast().unwrap();
    assert_eq!(root.kind(), "block");
    assert_eq!(root.node_child(0).unwrap().kind(), "send");
    assert_eq!(root.node_child(0).unwrap().symbol_child(1), Some("each"));
    assert_eq!(root.node_child(1).unwrap().kind(), "args");
    assert_eq!(root.node_child(2).unwrap().kind(), "send");
    assert_eq!(root.node_child(0).unwrap().parent(), Some(root));
    assert!(root.complete());
}

#[test]
fn exposes_scalar_values_and_wraps_multiple_top_level_expressions() {
    let processed =
        ProcessedSource::new("answer = 42\nanswer\n", 3.4, None, ParserEngine::Prism).unwrap();
    let root = processed.ast().unwrap();
    assert_eq!(root.kind(), "begin");
    let assignment = root.node_child(0).unwrap();
    assert_eq!(assignment.kind(), "lvasgn");
    assert_eq!(assignment.symbol_child(0), Some("answer"));
    assert_eq!(assignment.node_child(1).unwrap().integer_child(0), Some(42));
    assert_eq!(root.node_child(1).unwrap().kind(), "lvar");
}

#[test]
fn ast_preserves_definition_and_conditional_parser_layouts() {
    let processed = ProcessedSource::new(
        "def self.call(value = 1)\n  value if ready\nend\n",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let definition = processed.ast().unwrap();
    assert_eq!(definition.kind(), "defs");
    assert_eq!(definition.node_child(0).unwrap().kind(), "self");
    assert_eq!(definition.symbol_child(1), Some("call"));
    let args = definition.node_child(2).unwrap();
    assert_eq!(args.kind(), "args");
    assert_eq!(args.node_child(0).unwrap().kind(), "optarg");
    let conditional = definition.node_child(3).unwrap();
    assert_eq!(conditional.kind(), "if");
    assert_eq!(conditional.children().len(), 3);
    assert_eq!(conditional.node_child(0).unwrap().kind(), "send");
    assert_eq!(conditional.node_child(1).unwrap().kind(), "lvar");
}

#[test]
fn rejects_parser_and_ruby_version_combinations_outside_rubocop_contract() {
    assert!(ProcessedSource::new("1", 3.2, None, ParserEngine::Prism).is_err());
    assert!(ProcessedSource::new("1", 3.5, None, ParserEngine::Whitequark).is_err());
    assert!(ProcessedSource::new("1", 4.1, None, ParserEngine::Prism).is_ok());
}

#[test]
fn invalid_syntax_has_no_ast_comments_or_tokens() {
    let processed =
        ProcessedSource::new("def broken( # comment", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(!processed.valid_syntax());
    assert!(processed.ast().is_none());
    assert!(processed.blank());
    assert!(processed.comments().is_empty());
    assert!(processed.tokens().is_empty());
}

#[test]
fn line_slices_and_deprecated_find_contracts_match_collection_operations() {
    let processed = ProcessedSource::new(
        "one # first\ntwo\nthree # last\n",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    assert_eq!(processed.lines_range(0..2), ["one # first", "two"]);
    assert_eq!(processed.lines_slice(1, 8), ["two", "three # last", ""]);
    assert_eq!(
        processed
            .find_comment(|comment| comment.line == 3)
            .unwrap()
            .text,
        "# last"
    );
    assert_eq!(
        processed
            .find_token(|token| token.text == "two")
            .unwrap()
            .line,
        2
    );
}

#[test]
fn lexer_preserves_context_sensitive_token_predicates() {
    let source =
        "# comment\ndef some_method\n  [ 1, 2 ];\n  foo[0] = 3.to_i\n  1..42\n  1...42\nend\n";
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let token = |text: &str, line: usize| {
        processed
            .tokens()
            .iter()
            .find(|token| token.text == text && token.line == line)
            .unwrap_or_else(|| panic!("missing {text:?} on line {line}: {:?}", processed.tokens()))
    };
    assert!(token("# comment", 1).comment());
    assert!(token("[", 3).left_array_bracket());
    assert!(token("[", 4).left_ref_bracket());
    assert!(token(";", 3).semicolon());
    assert!(token("=", 4).equal_sign());
    assert!(token(".", 4).dot());
    assert!(token("..", 5).regexp_dots());
    assert!(token("...", 6).regexp_dots());
    assert!(token("end", 7).end_keyword());

    let braces = ProcessedSource::new(
        "{ a: 1 }\nfoo { |f| bar(f) }\n-> { f }\n",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let opening: Vec<_> = braces
        .tokens()
        .iter()
        .filter(|token| token.text == "{")
        .collect();
    assert!(opening[0].left_brace());
    assert!(opening[1].left_curly_brace());
    assert!(opening[2].left_curly_brace());
    assert!(braces
        .tokens()
        .iter()
        .find(|token| token.text == "(")
        .unwrap()
        .left_parens());
    assert!(braces
        .tokens()
        .iter()
        .find(|token| token.text == ")")
        .unwrap()
        .right_parens());

    let rescue = ProcessedSource::new("bar rescue qux\n", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(rescue
        .tokens()
        .iter()
        .find(|token| token.text == "rescue")
        .unwrap()
        .rescue_modifier());
}
