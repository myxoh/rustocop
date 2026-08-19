use ruby_prism::{parse, CallNode, Node, Visit};
use std::sync::Arc;

mod accessor_rules;
mod additional_rules;
mod additional_rules_literals;
mod additional_rules_more;
mod alias_rules;
mod argument_and_inheritance_rules;
mod assignment_completion_rules;
mod block_arity_rules;
mod block_association_rules;
mod block_chain_rules;
mod block_parameter_rules;
mod branch_layout_rules;
mod call_conversion_rules;
mod class_comparison_rules;
mod coercion_rules;
mod collection_completion_rules;
mod comparable_clamp_rules;
mod compatibility_lexical_rules;
mod conditional_semantics_rules;
mod cop_context;
mod correction_engine;
mod declaration_completion_rules;
mod declaration_semantics;
mod deprecated_api_rules;
mod diagnostic;
mod dig_rules;
mod double_splat_rules;
mod dsl;
mod empty_method_rules;
mod enum_argument_rules;
mod exception_argument_rules;
mod fetch_completion_rules;
mod file_predicate_rules;
mod file_structure_rules;
mod hash_array_rules;
mod heredoc_call_rules;
mod interpolation_condition_rules;
mod io_scheduler_rules;
mod it_parameter_rules;
mod iteration_redundancy_rules;
mod layout;
mod lexical_completion;
mod lexical_rules;
mod line_concatenation_rules;
mod lint;
mod lint_builtin_overrides;
mod lint_control_flow;
mod lint_suspicious_calls;
mod literal_and_pattern_rules;
mod logical_condition_rules;
mod lookup_completion_rules;
mod map_join_rules;
mod matchers;
mod method_layout_rules;
mod method_signature_rules;
mod mixin_grouping_rules;
mod mixin_rules;
mod nested_call_rules;
mod nil_callable_rules;
mod node_helpers;
mod number_conversion_rules;
mod numeric_operation_rules;
mod numeric_predicate_rules;
mod operator_ambiguity_rules;
mod path_and_literal_rules;
mod percent_string_rules;
mod predicate_conversion_rules;
mod prism_engine;
mod random_rules;
mod registry;
mod require_order_rules;
mod require_rules;
mod rescue_rules;
mod resource_and_precedence_rules;
mod runner;
mod security;
mod self_rules;
mod send_literal_rules;
mod setter_rules;
mod signal_exception_rules;
mod single_line_block_rules;
mod source_file;
mod source_helpers;
mod source_rules;
mod source_rules_layout;
mod source_rules_misc;
mod source_semantics;
mod source_syntax;
mod string_conversion_rules;
mod structural_completion_rules;
mod style;
mod style_call_simplifications;
mod style_calls;
mod style_collections;
mod style_compat;
mod style_global_vars;
mod style_rewrites;
mod style_source;
mod ternary_rules;

use crate::config::{CopConfig, RubyVersion};
use cop_context::{CopContext, CopPolicy};
use diagnostic::{Context, Reporter};
pub(crate) use diagnostic::{Finding, Inspection};
use dsl::*;
use matchers::*;
use node_helpers::*;
pub use prism_engine::Engine;
use registry::Registry;
use runner::Runner;
use source_file::{SourceEdit, SourceFile};

pub(super) trait Cop: Sync {
    fn name(&self) -> &'static str;
    fn on_source(&self, _source: &str, _context: &mut Context) {}
    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        if let Some(call) = node.as_call_node() {
            self.on_call(&call, context);
        }
    }
    fn on_call(&self, _node: &CallNode<'_>, _context: &mut Context) {}
}

pub(crate) fn cop_names() -> Vec<&'static str> {
    registry::cop_names()
}

impl Registry {
    fn enabled(enabled: &dyn Fn(&str) -> bool) -> Self {
        type Provider = fn() -> Vec<Box<dyn Cop>>;
        let providers: &[Provider] = &[
            lint::cops,
            accessor_rules::cops,
            additional_rules::cops,
            additional_rules_literals::cops,
            additional_rules_more::cops,
            alias_rules::cops,
            argument_and_inheritance_rules::cops,
            assignment_completion_rules::cops,
            block_association_rules::cops,
            block_parameter_rules::cops,
            block_chain_rules::cops,
            block_arity_rules::cops,
            branch_layout_rules::cops,
            call_conversion_rules::cops,
            class_comparison_rules::cops,
            compatibility_lexical_rules::cops,
            comparable_clamp_rules::cops,
            collection_completion_rules::cops,
            coercion_rules::cops,
            conditional_semantics_rules::cops,
            declaration_semantics::cops,
            declaration_completion_rules::cops,
            deprecated_api_rules::cops,
            dig_rules::cops,
            double_splat_rules::cops,
            empty_method_rules::cops,
            enum_argument_rules::cops,
            exception_argument_rules::cops,
            fetch_completion_rules::cops,
            file_structure_rules::cops,
            file_predicate_rules::cops,
            hash_array_rules::cops,
            heredoc_call_rules::cops,
            iteration_redundancy_rules::cops,
            interpolation_condition_rules::cops,
            io_scheduler_rules::cops,
            it_parameter_rules::cops,
            lint_builtin_overrides::cops,
            lint_control_flow::cops,
            lint_suspicious_calls::cops,
            layout::cops,
            lexical_completion::cops,
            line_concatenation_rules::cops,
            literal_and_pattern_rules::cops,
            logical_condition_rules::cops,
            lookup_completion_rules::cops,
            map_join_rules::cops,
            method_layout_rules::cops,
            method_signature_rules::cops,
            mixin_grouping_rules::cops,
            mixin_rules::cops,
            nested_call_rules::cops,
            nil_callable_rules::cops,
            number_conversion_rules::cops,
            numeric_operation_rules::cops,
            numeric_predicate_rules::cops,
            operator_ambiguity_rules::cops,
            path_and_literal_rules::cops,
            percent_string_rules::cops,
            predicate_conversion_rules::cops,
            random_rules::cops,
            resource_and_precedence_rules::cops,
            require_rules::cops,
            require_order_rules::cops,
            rescue_rules::cops,
            lexical_rules::cops,
            security::cops,
            self_rules::cops,
            send_literal_rules::cops,
            setter_rules::cops,
            signal_exception_rules::cops,
            single_line_block_rules::cops,
            style::cops,
            style_call_simplifications::cops,
            style_calls::cops,
            style_collections::cops,
            style_compat::cops,
            style_global_vars::cops,
            style_rewrites::cops,
            style_source::cops,
            structural_completion_rules::cops,
            string_conversion_rules::cops,
            source_rules::cops,
            source_rules_layout::cops,
            source_rules_misc::cops,
            source_semantics::cops,
            ternary_rules::cops,
        ];
        let cops = providers
            .iter()
            .flat_map(|provide| provide())
            .filter(|cop| enabled(cop.name()))
            .collect();

        Self { cops }
    }
}

#[cfg(test)]
fn inspect(
    source: &str,
    autocorrect: bool,
    target_ruby_version: RubyVersion,
    enabled: &dyn Fn(&str) -> bool,
) -> Inspection {
    Engine::new(enabled).inspect(
        "example.rb",
        source,
        autocorrect,
        target_ruby_version,
        Arc::new(CopConfig::default()),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_once_and_dispatches_to_enabled_cops() {
        let inspection = inspect(
            "eval(code)\nJSON.load(payload)\n",
            false,
            RubyVersion::default(),
            &|cop| matches!(cop, "Security/Eval" | "Security/JSONLoad"),
        );

        assert_eq!(inspection.findings.len(), 2);
        assert_eq!(inspection.findings[0].cop_name, "Security/Eval");
        assert_eq!(inspection.findings[1].cop_name, "Security/JSONLoad");
    }

    #[test]
    fn applies_non_overlapping_autocorrections_from_the_shared_tree() {
        let inspection = inspect(
            "JSON.load(payload)\nIO.read(path)\n",
            true,
            RubyVersion::default(),
            &|cop| matches!(cop, "Security/JSONLoad" | "Security/IoMethods"),
        );

        assert_eq!(
            inspection.corrected_source,
            "JSON.parse(payload)\nFile.read(path)\n"
        );
        assert!(inspection.findings.iter().all(|finding| finding.corrected));
    }

    #[test]
    fn applies_compatibility_batch_corrections_from_one_tree() {
        let source = concat!(
            "{a:3}\n",
            "STDOUT.puts('hello')\n",
            "require 'foo.rb'\n",
            "super name, age\n",
            "test { |a, b,| a + b }\n",
            "while cond do\nend\n",
        );
        let inspection = inspect(source, true, RubyVersion::default(), &|cop| {
            matches!(
                cop,
                "Layout/SpaceAfterColon"
                    | "Style/GlobalStdStream"
                    | "Style/RedundantFileExtensionInRequire"
                    | "Style/SuperWithArgsParentheses"
                    | "Style/TrailingCommaInBlockArgs"
                    | "Style/WhileUntilDo"
            )
        });

        assert_eq!(inspection.findings.len(), 6);
        assert_eq!(
            inspection.corrected_source,
            concat!(
                "{a: 3}\n",
                "$stdout.puts('hello')\n",
                "require 'foo'\n",
                "super(name, age)\n",
                "test { |a, b| a + b }\n",
                "while cond\nend\n",
            )
        );
    }

    #[test]
    fn replaces_empty_append_file_open_block() {
        let inspection = inspect(
            "File.open(filename, 'a') {}\n",
            true,
            RubyVersion::default(),
            &|cop| cop == "Style/FileTouch",
        );

        assert_eq!(inspection.findings.len(), 1);
        assert_eq!(inspection.corrected_source, "FileUtils.touch(filename)\n");
    }

    #[test]
    fn public_prism_registry_is_sorted_and_unique() {
        let names = cop_names();
        assert!(names.windows(2).all(|pair| pair[0] < pair[1]));
        assert!(names.contains(&"Security/Eval"));
    }

    #[test]
    fn target_ruby_version_is_available_to_cops() {
        let ruby_30 = inspect(
            "YAML.load(payload)\n",
            false,
            RubyVersion::new(3, 0),
            &|cop| cop == "Security/YAMLLoad",
        );
        let ruby_31 = inspect(
            "YAML.load(payload)\n",
            false,
            RubyVersion::new(3, 1),
            &|cop| cop == "Security/YAMLLoad",
        );

        assert_eq!(ruby_30.findings.len(), 1);
        assert!(ruby_31.findings.is_empty());
    }

    #[test]
    fn corrects_verified_collection_call_and_condition_cops() {
        let source = concat!(
            "arr[0]\n",
            "items.flatten.join\n",
            "service::call\n",
            "if /foo/\nend\n",
            "items.sort_by { |item| item }\n",
        );
        let inspection = inspect(source, true, RubyVersion::new(3, 4), &|cop| {
            matches!(
                cop,
                "Style/ArrayFirstLast"
                    | "Style/RedundantArrayFlatten"
                    | "Style/ColonMethodCall"
                    | "Lint/RegexpAsCondition"
                    | "Style/RedundantSortBy"
            )
        });

        assert_eq!(inspection.findings.len(), 5);
        assert_eq!(
            inspection.corrected_source,
            concat!(
                "arr.first\n",
                "items.join\n",
                "service.call\n",
                "if /foo/ =~ $_\nend\n",
                "items.sort\n",
            )
        );
    }

    #[test]
    fn leaves_chained_bracket_access_unchanged() {
        let inspection = inspect("arr[0][-1]\n", true, RubyVersion::default(), &|cop| {
            cop == "Style/ArrayFirstLast"
        });

        assert!(inspection.findings.is_empty());
        assert_eq!(inspection.corrected_source, "arr[0][-1]\n");
    }

    #[test]
    fn runs_verified_suspicious_call_and_control_flow_cops() {
        let source = concat!(
            "a.x == a.x\n",
            "hash.key?(value.object_id)\n",
            "rand(1)\n",
            "return unless value&.empty?\n",
            "begin\n  work\nend while active\n",
        );
        let inspection = inspect(source, true, RubyVersion::default(), &|cop| {
            matches!(
                cop,
                "Lint/BinaryOperatorWithIdenticalOperands"
                    | "Lint/HashCompareByIdentity"
                    | "Lint/RandOne"
                    | "Lint/SafeNavigationWithEmpty"
                    | "Lint/Loop"
            )
        });

        assert_eq!(inspection.findings.len(), 5);
        assert_eq!(
            inspection.corrected_source,
            concat!(
                "a.x == a.x\n",
                "hash.key?(value.object_id)\n",
                "rand(1)\n",
                "return unless value && value.empty?\n",
                "loop do\n  work\nbreak unless active\nend\n",
            )
        );
    }

    #[test]
    fn registers_the_twenty_cop_parity_batch() {
        let names = cop_names();
        for cop in [
            "Bundler/GemVersion",
            "Layout/InitialIndentation",
            "Layout/MultilineArrayLineBreaks",
            "Lint/DuplicateMagicComment",
            "Lint/EmptyInterpolation",
            "Lint/ErbNewArguments",
            "Lint/HashNewWithKeywordArgumentsAsDefault",
            "Lint/InterpolationCheck",
            "Lint/LambdaWithoutLiteralBlock",
            "Lint/RequireRangeParentheses",
            "Lint/RequireRelativeSelfPath",
            "Lint/SharedMutableDefault",
            "Lint/TopLevelReturnWithArgument",
            "Naming/AsciiIdentifiers",
            "Style/MultilineIfThen",
            "Style/OptionalArguments",
            "Style/OptionalBooleanParameter",
            "Style/ReturnNil",
            "Style/Send",
            "Style/VariableInterpolation",
        ] {
            assert!(names.contains(&cop), "missing {cop}");
        }
    }

    #[test]
    fn corrects_representative_parity_batch_offenses_together() {
        let source = concat!(
            "return nil\n",
            "send(:work)\n",
            "Hash.new(key: :value)\n",
            "lambda(&callback)\n",
        );
        let inspection = inspect(source, true, RubyVersion::default(), &|cop| {
            matches!(
                cop,
                "Style/ReturnNil"
                    | "Style/Send"
                    | "Lint/HashNewWithKeywordArgumentsAsDefault"
                    | "Lint/LambdaWithoutLiteralBlock"
            )
        });

        assert_eq!(inspection.findings.len(), 4);
        assert_eq!(
            inspection.corrected_source,
            concat!(
                "return\n",
                "send(:work)\n",
                "Hash.new({key: :value})\n",
                "callback\n",
            )
        );
    }
}
