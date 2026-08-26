// Source: spec/rubocop/cop/mixin/enforce_superclass_spec.rb
// Spec SHA-256: 2aaa136f382342544af143cbf4f62070e93a9315c11e997b60d032b67d4f9317
// Source: spec/rubocop/cop/visibility_help_spec.rb
// Spec SHA-256: 6c3a1803d663cd528251ed2911e101d65ddddee4fd3823ad2199a89745096556

use super::advanced::*;
use crate::rubocop::ast::node::core::{Ast, NodeValue};
use crate::rubocop::ast::processed_source::{ParserEngine, ProcessedSource};
use crate::rubocop::ast::source::SourceBuffer;
use crate::rubocop::cop::corrector::Corrector;
use std::collections::{BTreeSet, HashSet};

#[test]
fn alignment_and_width_helpers_preserve_unicode_and_tabs() {
    assert_eq!(display_column("é\tx", 2), 4);
    assert_eq!(indentation("  body"), "  ");
    assert!(within(2..4, 1..5));
    assert_eq!(alignment_offset(4, 2), -2);
}

#[test]
fn ordered_gem_source_range_respects_comment_separator_configuration() {
    assert!(!treat_comments_as_separators(None));
    assert!(!treat_comments_as_separators(Some(false)));
    assert!(treat_comments_as_separators(Some(true)));
    assert_eq!(get_source_range(10..20, Some(3..8), false), 3..8);
    assert_eq!(get_source_range(10..20, Some(3..8), true), 10..20);
    assert_eq!(get_source_range(10..20, None, false), 10..20);
}

#[test]
fn code_length_ignores_blanks_comments_and_folded_sections() {
    let options = CodeLengthOptions {
        count_comments: false,
        count_as_one: Vec::new(),
    };
    assert_eq!(
        code_length(
            "# docs\n\nwork\nattribute :a\nattribute :b\nmore\n",
            &options
        ),
        4
    );
    assert_eq!(
        code_length_message("Block", 4, 3),
        "Block has too many lines. [4/3]"
    );
}

#[test]
fn code_length_folds_supported_ast_types_and_rejects_unknown_types() {
    let source = "def call\n  values = [\n    1,\n    2\n  ]\n  finish\nend";
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let node = processed.ast().unwrap();
    assert_eq!(
        code_length_for_node(
            node,
            &processed,
            &CodeLengthOptions {
                count_comments: false,
                count_as_one: Vec::new(),
            }
        )
        .unwrap(),
        5
    );
    assert_eq!(
        code_length_for_node(
            node,
            &processed,
            &CodeLengthOptions {
                count_comments: false,
                count_as_one: vec!["array".into()],
            }
        )
        .unwrap(),
        2
    );
    assert!(code_length_for_node(
        node,
        &processed,
        &CodeLengthOptions {
            count_comments: false,
            count_as_one: vec!["unknown".into()],
        }
    )
    .is_err());
}

#[test]
fn comments_are_filtered_by_range_and_disable_directives() {
    let comments = vec![
        Comment {
            range: 1..4,
            line: 1,
            text: "rubocop:disable X".into(),
        },
        Comment {
            range: 8..10,
            line: 3,
            text: "note".into(),
        },
    ];
    assert_eq!(comments_in_range(&comments, &(0..5)).len(), 1);
    assert!(contains_comments(&comments, &(0..2)));
    assert!(comments_contain_disables(&comments));
    assert_eq!(preceding_comment(&comments, 2), Some(&comments[0]));
}

#[test]
fn formatting_and_documentation_classification_match_configuration() {
    assert_eq!(formatting_style(2, 0), DetectedStyle::Style);
    assert_eq!(formatting_style(1, 1), DetectedStyle::Mixed);
    assert!(valid_formatting_name("snake_2"));
    assert!(documentation_comment("# Describes widget", &["TODO"]));
    assert!(!documentation_comment("# rubocop:disable X", &[]));
}

#[test]
fn frozen_string_literal_reads_leading_magic_comments() {
    assert_eq!(
        frozen_string_literal("#!/usr/bin/ruby\n# frozen_string_literal: true\nputs 1"),
        FrozenStringLiteral::Enabled
    );
    assert_eq!(
        frozen_string_literal("puts 1"),
        FrozenStringLiteral::Unspecified
    );
    assert_eq!(
        frozen_string_literal("# frozen_string_literal: yes\nputs 1"),
        FrozenStringLiteral::Unspecified
    );
}

#[test]
fn frozen_heredoc_alias_uses_the_same_uninterpolated_dstr_contract() {
    let processed = ProcessedSource::new(
        "# frozen_string_literal: true\nvalue",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let runtime = super::frozen_string_literal::FrozenStringLiteral::new(&processed, 3.4, None);
    let mut ast = Ast::new("body\nEND");
    let string = ast.add_node("str", vec![NodeValue::String("body".into())], Some(0..4));
    let heredoc = ast.add_node("dstr", vec![NodeValue::Node(string)], Some(0..8));
    ast.set_location(heredoc, "heredoc_body", 0..4, "body");
    ast.set_location(heredoc, "heredoc_end", 5..8, "END");
    ast.complete(heredoc);
    assert!(runtime.frozen_heredoc(ast.node(heredoc)));
}

#[test]
fn hash_helpers_cover_alignment_shorthand_subset_and_transform() {
    assert_eq!(
        hash_alignment_delta((2, 6, 9), (4, 8, 12)),
        HashAlignmentDelta {
            key: -2,
            separator: -2,
            value: -3
        }
    );
    assert!(hash_value_omittable("foo", "foo"));
    assert!(mixed_hash_shorthand(&[
        ("a".into(), None),
        ("b".into(), Some("b".into()))
    ]));
    assert_eq!(preferred_hash_subset("reject", false), Some("except"));
    assert_eq!(transformed_hash_method("map"), Some("transform_values"));
}

#[test]
fn gemspec_helpers_find_the_exact_specification_block_and_its_declarations() {
    let processed = ProcessedSource::new(
        "Gem::Specification.new do |spec|\n  spec.name = 'example'\n  spec.add_dependency 'rack'\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let block = processed.ast().unwrap();
    assert_eq!(gemspec_block_variable(block), Some("spec"));
    let declarations = gemspec_assignment_declarations(block, "spec");
    assert_eq!(declarations.len(), 2);
    assert_eq!(assignment_method_declarations(block).len(), 2);
    assert!(gem_specification_call(Some("Gem::Specification"), "new"));
    assert!(!gem_specification_call(None, "new"));
    assert!(gem_assignment_method("add_dependency"));
    assert!(gem_assignment_method("name="));
}

#[test]
fn heredoc_and_line_length_helpers_keep_rubocop_semantics() {
    assert_eq!(heredoc_type("<<~SQL"), Some("squiggly"));
    assert_eq!(heredoc_delimiter("<<~'SQL'"), "SQL");
    assert_eq!(heredoc_indent("   query"), 3);
    assert_eq!(heredoc_delimiter_string("<<~'SQL'"), "<<~");
    assert_eq!(heredoc_type_string("<<~'SQL'"), "SQL");
    assert_eq!(heredoc_indent_level("  one\n    two\n\n"), 2);
    assert_eq!(line_length("éé", 2), 2);
    assert_eq!(
        line_length_without_directive("long # rubocop:disable Layout/LineLength", 2),
        4
    );
    assert!(valid_uri("https://example.com/a"));
    assert!(qualified_name("Foo::Bar"));
}

#[test]
fn single_line_suitability_preserves_rubocops_join_rules_and_safety_checks() {
    assert_eq!(to_single_line("foo\n  .bar"), "foo.bar");
    assert_eq!(to_single_line("\"a\" \\\n  \"b\""), "\"ab\"");
    assert_eq!(to_single_line("\"a\" \\\n  'b'"), "\"a\" + 'b'");

    let safe = ProcessedSource::new("foo(\n  1\n)", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(suitable_as_single_line_node(
        safe.ast().unwrap(),
        &safe,
        Some(20)
    ));
    let commented =
        ProcessedSource::new("foo(\n  1 # explanation\n)", 3.4, None, ParserEngine::Prism).unwrap();
    assert!(!suitable_as_single_line_node(
        commented.ast().unwrap(),
        &commented,
        Some(80)
    ));
}

#[test]
fn small_structural_mixins_operate_on_parser_shaped_nodes() {
    let assignment =
        ProcessedSource::new("value = call(1)", 3.4, None, ParserEngine::Prism).unwrap();
    let assignment_node = assignment.ast().unwrap();
    let (_, rhs) = check_assignment_target(assignment_node).unwrap();
    assert_eq!(rhs.method_name(), Some("call"));

    let interpolation =
        ProcessedSource::new("\"before #{value} after\"", 3.4, None, ParserEngine::Prism).unwrap();
    let string = interpolation.ast().unwrap();
    let nodes = interpolation_nodes(string);
    assert_eq!(nodes.len(), 1);
    assert!(inside_interpolation(nodes[0].first_node().unwrap()));

    let endless =
        ProcessedSource::new("def answer(x) = x", 3.4, None, ParserEngine::Prism).unwrap();
    let node = endless.ast().unwrap();
    assert_eq!(
        endless_method_replacement(node, "").as_deref(),
        Some("def answer(x)\n  x\nend")
    );
    let buffer = endless.buffer();
    let mut corrector = Corrector::new(&buffer);
    correct_endless_to_multiline(&mut corrector, node);
    assert_eq!(corrector.rewrite().unwrap(), "def answer(x)\n  x\nend");
}

#[test]
fn rescue_node_uses_lexer_modifier_locations_and_resbody_exceptions() {
    let modifier =
        ProcessedSource::new("work rescue fallback", 3.4, None, ParserEngine::Prism).unwrap();
    let root = modifier.ast().unwrap();
    assert_eq!(root.kind(), "rescue");
    let resbody = root.each_node(&["resbody"])[0];
    let locations = rescue_modifier_locations(&modifier);
    assert_eq!(locations.len(), 1);
    assert!(rescue_modifier_at(resbody, &locations));

    let standard = ProcessedSource::new(
        "begin\n  work\nrescue Error\n  fallback\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let resbody = standard.ast().unwrap().each_node(&["resbody"])[0];
    let exceptions = rescued_exceptions(resbody);
    assert_eq!(exceptions.len(), 1);
    assert_eq!(exceptions[0].const_name().as_deref(), Some("Error"));
}

#[test]
fn indentation_and_brace_layout_styles_are_explicit() {
    assert_eq!(
        expected_element_column(2, 2, IndentationStyle::Special, 10),
        4
    );
    assert_eq!(
        expected_element_column(2, 2, IndentationStyle::Consistent, 10),
        10
    );
    assert!(incorrect_indentation(3, 4));
    assert!(closing_brace_on_same_line(2, 2));
    assert!(symmetrical_braces(1, 1, 3, 3));
}

#[test]
fn first_element_and_visibility_helpers_follow_ast_relationships() {
    let call = ProcessedSource::new("call(one,\n  two)", 3.4, None, ParserEngine::Prism).unwrap();
    let node = call.ast().unwrap();
    let children = node.arguments();
    assert!(method_uses_parentheses(node, children[0]));
    assert_eq!(
        method_first_element_line_break_offense(node, &children, false),
        Some(children[0])
    );

    let source = "class Example\n  private\n  def hidden; end\n  public def shown; end\nend";
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let definitions = processed.ast().unwrap().each_node(&["def"]);
    assert_eq!(definitions.len(), 2);
    assert_eq!(exact_node_visibility(definitions[0]), Visibility::Private);
    assert_eq!(exact_node_visibility(definitions[1]), Visibility::Public);
}

#[test]
fn enforce_superclass_matches_class_and_class_new_contracts() {
    let base_pattern = |node: crate::rubocop::ast::node::core::NodeRef<'_>| {
        node.const_name().as_deref() == Some("ActiveRecord::Base")
    };
    for source in [
        "class MyModel < ActiveRecord::Base; end",
        "class ::MyModel < ::ActiveRecord::Base; end",
        "MyModel = Class.new(ActiveRecord::Base)",
        "::MyModel = ::Class.new(::ActiveRecord::Base) {}",
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let runtime = EnforceSuperclass {
            superclass: "ApplicationRecord",
        };
        assert!(runtime.included().contains("deprecated"));
        let candidate = processed
            .ast()
            .unwrap()
            .each_node(&["class", "send"])
            .into_iter()
            .find_map(|node| match node.kind() {
                "class" => runtime.on_class(node, base_pattern),
                "send" => runtime.on_send(node, base_pattern),
                _ => None,
            });
        assert_eq!(
            candidate.and_then(|node| node.const_name()).as_deref(),
            Some("ActiveRecord::Base"),
            "{source}"
        );
    }

    for source in [
        "class ApplicationRecord < ActiveRecord::Base; end",
        "class MyModel < ApplicationRecord; end",
        "ApplicationRecord = Class.new(ActiveRecord::Base)",
        "MyModel = ::Class.new(::ApplicationRecord) {}",
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        assert!(
            processed
                .ast()
                .unwrap()
                .each_node(&["class", "send"])
                .into_iter()
                .all(
                    |node| enforced_superclass_offense(node, "ApplicationRecord", base_pattern)
                        .is_none()
                ),
            "{source}"
        );
    }
}

#[test]
fn gem_order_and_percent_array_helpers_are_deterministic() {
    assert!(gem_out_of_order("Zulu_gem", "alpha-gem", false));
    assert_eq!(gem_canonical_name("Rack_Test", false), "racktest");
    assert_eq!(gem_canonical_name("Rack-Test", true), "rack-test");
    assert!(consecutive_lines(2, 3));
    assert_eq!(
        percent_array_message("w"),
        "Use `%w` for an array of words."
    );
    assert_eq!(bracket_array(&["a", "b"], '\''), "['a', 'b']");
    assert!(percent_array_context_valid(false, false));
}

#[test]
fn project_signature_and_require_tracking_are_stable() {
    assert_eq!(
        project_index_signature(["b", "a"]),
        project_index_signature(["a", "b"])
    );
    assert_eq!(
        project_index_signature(["rubydex:built-in", "file:///missing-rustocop-path"]),
        vec!["/missing-rustocop-path:0:0"]
    );
    assert_eq!(
        project_index_checksum(["b", "a"]),
        project_index_checksum(["a", "b"])
    );
    let mut required = BTreeSet::new();
    assert!(ensure_required(&mut required, "set"));
    assert!(!ensure_required(&mut required, "set"));
    assert!(require_any_library(
        &required.iter().cloned().collect(),
        &["json", "set"]
    ));
}

#[test]
fn require_library_tracks_top_level_requires_and_deduplicates_when_inserting() {
    let source = "work\nrequire 'set'\n";
    let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
    let root = processed.ast().unwrap();
    let sends = root.each_node(&["send"]);
    assert_eq!(require_library_name(sends[1]), Some("set"));
    let mut runtime = RequireLibrary::default();
    runtime.on_send(sends[1]);
    assert!(runtime.required_libs().contains("set"));
    runtime.on_new_investigation();
    assert!(runtime.required_libs().is_empty());
    let mut tracked = BTreeSet::new();
    assert_eq!(
        track_top_level_required_library(&mut tracked, sends[1]).as_deref(),
        Some("set")
    );

    let buffer = processed.buffer();
    let mut corrector = Corrector::new(&buffer);
    ensure_required_library(&mut corrector, sends[0], "set", &BTreeSet::new());
    assert_eq!(corrector.rewrite().unwrap(), "require 'set'\nwork\n");
}

#[test]
fn punctuation_and_surrounding_space_helpers_use_character_offsets() {
    let buffer = SourceBuffer::new("a  ,b");
    assert!(!missing_space_before(&buffer, 3));
    assert!(missing_space_after(&buffer, 4));
    assert_eq!(side_space_range(&buffer, 3, true).source(), "  ");
    assert!(space_between(&buffer, 1, 3));
    assert!(extra_space("  "));
}

#[test]
fn punctuation_mixins_scan_adjacent_lexer_tokens_with_style_exceptions() {
    let after = ProcessedSource::new("call(one,two)", 3.4, None, ParserEngine::Prism).unwrap();
    let missing = missing_space_after_punctuation(after.tokens(), 1, "space", |left, _| {
        (left.text == ",").then(|| "comma".to_owned())
    });
    assert_eq!(missing.len(), 1);
    assert_eq!(missing[0].1, "comma");

    let before = ProcessedSource::new("call(one ,two)", 3.4, None, ParserEngine::Prism).unwrap();
    let spaces = spaces_before_punctuation(before.tokens(), "space", |token| {
        (token.text == ",").then(|| "comma".to_owned())
    });
    assert_eq!(spaces.len(), 1);
    assert_eq!(spaces[0].1, 8..9);
}

#[test]
fn statement_modifier_and_trailing_comma_helpers_cover_style_branches() {
    assert_eq!(
        modifier_form("if", "ready?", "work", true),
        "work if (ready?)"
    );
    assert!(modifier_fits("work\nif ready", 20));
    assert!(should_have_trailing_comma(true, "comma", true));
    assert!(!should_have_trailing_comma(true, "no_comma", true));
    assert!(should_have_trailing_comma_for(
        "consistent_comma",
        true,
        false,
        false,
        false
    ));
    assert!(should_have_trailing_comma_for(
        "diff_comma",
        true,
        false,
        false,
        true
    ));
    assert!(!should_have_trailing_comma_for(
        "comma", true, false, false, true
    ));
    assert_eq!(trailing_comma_range("a,  "), Some(1));
}

#[test]
fn name_policy_reports_each_independent_problem() {
    let policy = NamePolicy {
        min_length: 3,
        allowed: HashSet::new(),
        forbidden: HashSet::from(["X1".into()]),
        allow_numbers: false,
        allow_uppercase: false,
    };
    assert_eq!(
        name_issues("X1", &policy),
        vec![
            NameIssue::TooShort,
            NameIssue::Forbidden,
            NameIssue::EndsWithNumber,
            NameIssue::Uppercase
        ]
    );
    assert!(argument_unused("arg", &HashSet::new()));
    assert!(!argument_unused("_arg", &HashSet::new()));
}

#[test]
fn visibility_helpers_respect_inline_then_current_state() {
    assert_eq!(visibility_from_method("private"), Some(Visibility::Private));
    assert_eq!(
        node_visibility("call", Some(Visibility::Protected), Visibility::Public),
        Visibility::Protected
    );
    assert_eq!(
        node_visibility("call", None, Visibility::Private),
        Visibility::Private
    );
    for (source, expected) in [
        ("class A\n def x; end\nend", Visibility::Public),
        ("class A\n public\n def x; end\nend", Visibility::Public),
        ("class A\n private\n def x; end\nend", Visibility::Private),
        (
            "class A\n public\n private\n def x; end\nend",
            Visibility::Private,
        ),
        ("class A\n public def x; end\nend", Visibility::Public),
        ("class A\n private def x; end\nend", Visibility::Private),
        (
            "class A\n def x; end\n private :x\nend",
            Visibility::Private,
        ),
    ] {
        let processed = ProcessedSource::new(source, 3.4, None, ParserEngine::Prism).unwrap();
        let definition = processed.ast().unwrap().each_node(&["def"])[0];
        assert_eq!(exact_node_visibility(definition), expected, "{source}");
    }
}

#[test]
fn structural_node_helpers_use_translated_node_contracts() {
    let mut ast = Ast::new("");
    let rhs = ast.add_node("int", vec![NodeValue::Integer(1)], None);
    let asgn = ast.add_node(
        "lvasgn",
        vec![NodeValue::Symbol("x".into()), NodeValue::Node(rhs)],
        None,
    );
    assert_eq!(assignment_rhs(ast.node(asgn)), Some(ast.node(rhs)));
    let cond = ast.add_node(
        "if",
        vec![NodeValue::Node(rhs), NodeValue::Nil, NodeValue::Nil],
        None,
    );
    assert!(!non_eligible_modifier(ast.node(asgn), ast.node(cond)));
    assert_eq!(method_complexity(ast.node(cond), &[]), 1);
}

#[test]
fn public_scalar_collection_and_layout_aliases_preserve_rubocop_semantics() {
    assert!(end_keyword_aligned(2, 2));
    assert_eq!(variable_alignment(2, 7, "variable"), 7);
    assert_eq!(variable_alignment(2, 7, "keyword"), 2);
    assert!(superclass_allowed(
        Some("ApplicationRecord"),
        &["ApplicationRecord"]
    ));
    assert!(first_element_needs_line_break(1, 1));
    assert!(checkable_hash_layout(&[(2, 2), (4, 4)]));
    assert_eq!(
        hash_transform_correction("values", "to_h", "item", "[item, item]"),
        "values.to_h { |item| [item, item] }"
    );
    assert_eq!(excessive_range("abcdef", 4, 2), Some(4..6));
    assert!(case_insensitive_out_of_order("alpha", "Zulu", false));
    assert!(aligned_with_any(4, &[2, 4, 8]));
    assert!(punctuation_allowed("comma"));
    assert!(!punctuation_allowed("period"));
    assert!(empty_brackets("[]"));
    assert_eq!(
        visibility_span(
            &[
                (1, Some(Visibility::Private)),
                (2, None),
                (3, Some(Visibility::Public)),
            ],
            1,
            Visibility::Public,
        ),
        (1, 2, Visibility::Private)
    );

    let left = crate::rubocop::ast::processed_source::SourceToken {
        kind: "tLCURLY",
        text: "{".into(),
        range: 0..1,
        line: 1,
        column: 0,
    };
    assert!(space_required_after(&left, "space"));
}

#[test]
fn public_node_collection_aliases_operate_on_real_parser_shaped_trees() {
    let call = ProcessedSource::new("call(one,\n  two)", 3.4, None, ParserEngine::Prism).unwrap();
    let node = call.ast().unwrap();
    let children = node.arguments();
    assert_eq!(first_by_line(&children), Some(children[0]));
    assert_eq!(
        check_children_line_break(node, &children, None, false),
        Some(children[0])
    );
    assert_eq!(grouped_by_line(&children).len(), 2);

    let define = ProcessedSource::new(
        "define_method(:call) { _1 }",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    assert!(define_method_block(define.ast().unwrap()));

    let gemspec = ProcessedSource::new(
        "Gem::Specification.new do |spec|\n  spec.metadata['key'] = 'value'\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let root = gemspec.ast().unwrap();
    assert!(match_block_variable_name(root, "spec"));
    assert_eq!(indexed_assignment_method_declarations(root).len(), 1);

    let visibility = ProcessedSource::new(
        "class Example\n  private\n  def hidden; end\n  public\n  def shown; end\nend",
        3.4,
        None,
        ParserEngine::Prism,
    )
    .unwrap();
    let definitions = visibility.ast().unwrap().each_node(&["def"]);
    assert_eq!(find_visibility_end(definitions[0]), Some(definitions[1]));
}
