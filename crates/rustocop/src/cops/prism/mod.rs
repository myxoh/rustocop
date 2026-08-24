use ruby_prism::{parse, CallNode, Diagnostic, Node, Visit};
use std::any::Any;
use std::sync::Arc;

#[path = "framework/catalog_cop.rs"]
mod catalog_cop;
#[path = "framework/context_node_facade.rs"]
mod context_node_facade;
#[path = "framework/cop_context.rs"]
mod cop_context;
#[path = "framework/cop_policy.rs"]
mod cop_policy;
#[path = "framework/correction_engine.rs"]
mod correction_engine;
#[path = "framework/corrector.rs"]
mod corrector;
#[path = "framework/diagnostic.rs"]
mod diagnostic;
#[path = "framework/dsl.rs"]
mod dsl;
#[path = "framework/matchers.rs"]
mod matchers;
#[path = "framework/node_helpers.rs"]
mod node_helpers;
#[path = "framework/numeric_helpers.rs"]
mod numeric_helpers;
#[path = "runtime/prism_engine.rs"]
mod prism_engine;
#[path = "runtime/registry.rs"]
mod registry;
#[path = "framework/rule_dsl.rs"]
mod rule_dsl;
#[path = "runtime/runner.rs"]
mod runner;
#[path = "framework/source_file.rs"]
mod source_file;
#[path = "framework/source_helpers.rs"]
mod source_helpers;
#[path = "framework/source_syntax.rs"]
mod source_syntax;
#[path = "framework/ternary_conversion.rs"]
mod ternary_conversion;
use ternary_conversion::*;

macro_rules! cop_modules {
    ($($module:ident),+ $(,)?) => {
        $(mod $module;)+
        const COP_PROVIDERS: &[Provider] = &[$($module::cops),+];
    };
}

cop_modules!(
    lint,
    magic_comment_format_rules,
    map_compact_conditional_rules,
    map_into_array_rules,
    accessor_rules,
    accessor_grouping_completion,
    additional_rules,
    additional_rules_literals,
    additional_rules_more,
    alias_rules,
    argument_and_inheritance_rules,
    argument_default_rules,
    assignment_completion_rules,
    assignment_rewrite_rules,
    block_association_rules,
    block_comments_rules,
    block_parameter_rules,
    block_chain_rules,
    block_arity_rules,
    begin_rewrite_rules,
    branch_layout_rules,
    bundler_completion,
    call_conversion_rules,
    class_comparison_rules,
    class_check_rules,
    class_definition_rules,
    class_methods_completion,
    class_vars_rules,
    compact_syntax_completion,
    compatibility_lexical_rules,
    comparable_clamp_rules,
    control_flow_completion_batch,
    control_semantics_completion_batch,
    collection_completion_rules,
    collection_query_rules,
    collection_rewrite_rules,
    collection_transform_batch,
    conditional_rewrite_rules,
    coercion_rules,
    conditional_semantics_rules,
    declaration_semantics,
    declaration_completion_rules,
    deprecated_api_rules,
    directive_completion,
    dig_rules,
    dir_rules,
    double_splat_rules,
    empty_method_rules,
    empty_lambda_parameter_rules,
    empty_else_rules,
    empty_class_rules,
    endless_method_rules,
    enum_argument_rules,
    exception_argument_rules,
    exception_rewrite_rules,
    exception_location_completion,
    fetch_completion_rules,
    final_ast_structural_batch,
    final_control_flow_batch,
    final_file_metadata_batch,
    final_layout_batch_a,
    final_layout_batch_b,
    final_metrics_batch,
    final_project_context_batch,
    final_regexp_batch,
    final_scope_batch_a,
    final_scope_batch_b,
    format_string_rules,
    format_string_token_rules,
    file_structure_rules,
    frozen_string_literal_comment_rules,
    guard_clause_rules,
    file_predicate_rules,
    hash_array_rules,
    hash_conversion_rules,
    hash_each_methods_rules,
    hash_fetch_chain_rules,
    hash_subset_rules,
    hash_syntax_rules,
    hash_transform_rules,
    heredoc_argument_closing_parenthesis_rules,
    heredoc_call_rules,
    iteration_redundancy_rules,
    interpolation_condition_rules,
    identical_conditional_branches_rules,
    if_with_semicolon_rules,
    if_unless_modifier_rules,
    if_with_boolean_literal_branches_rules,
    infinite_loop_rules,
    invertible_unless_condition_rules,
    inverse_methods_rules,
    io_scheduler_rules,
    it_parameter_rules,
    lint_builtin_overrides,
    lint_control_flow,
    lint_suspicious_calls,
    lint_scope_completion,
    lint_signature_completion_batch,
    lint_naming_completion_batch,
    layout,
    layout_line_break_completion,
    layout_spacing_completion,
    lambda_rules,
    layout_finalization_completion,
    layout_qualification,
    layout_body_qualification,
    layout_core_qualification,
    layout_geometry_completion,
    lexical_completion,
    line_concatenation_rules,
    literal_and_pattern_rules,
    literal_integrity_completion,
    literal_rewrite_rules,
    literal_string_completion_batch,
    logical_condition_rules,
    lookup_completion_rules,
    map_conversion_rules,
    map_join_rules,
    method_layout_rules,
    method_def_parentheses_rules,
    method_call_parentheses_rules,
    method_signature_rules,
    missing_else_rules,
    metrics_naming_completion,
    metrics_completion,
    mixin_grouping_rules,
    mixin_rules,
    modern_collection_completion,
    module_member_existence_rules,
    mutable_constant_rules,
    nested_call_rules,
    nested_modifier_rules,
    negative_array_index_rules,
    negated_if_else_rules,
    next_rules,
    nil_callable_rules,
    non_deterministic_require_rules,
    number_conversion_rules,
    numeric_operation_rules,
    numeric_predicate_rules,
    preferred_hash_methods_rules,
    one_line_conditional_rules,
    operator_ambiguity_rules,
    operator_method_call_rules,
    path_and_literal_rules,
    parameter_order_completion,
    percent_string_rules,
    predicate_conversion_rules,
    project_scope_completion,
    project_structural_completion_batch,
    random_rules,
    resource_and_precedence_rules,
    require_rules,
    redundant_freeze_completion,
    redundant_filter_chain_rules,
    redundant_format_rules,
    redundant_line_continuation_rules,
    redundant_min_max_by_rules,
    redundant_parentheses_rules,
    redundant_regexp_rules,
    redundant_self_assignment_branch_rules,
    redundant_sort_rules,
    redundant_string_escape_rules,
    redundant_return_rules,
    regexp_literal_rules,
    require_order_rules,
    rescue_rules,
    rescue_modifier_rules,
    rescue_standard_error_rules,
    restored_structural_cops,
    restored_layout_indentation,
    restored_layout_line_breaks,
    restored_multiline_delimiters,
    return_nil_predicate_rules,
    ruby2_keywords_rules,
    lexical_rules,
    security,
    self_rules,
    send_literal_rules,
    setter_rules,
    semantic_gap_completion,
    signal_exception_rules,
    single_line_block_rules,
    sole_nested_conditional_rules,
    special_global_vars_rules,
    stabby_lambda_parentheses_rules,
    style,
    style_call_simplifications,
    style_calls,
    style_collections,
    style_compat,
    style_global_vars,
    style_metadata_completion,
    symbol_proc_rules,
    symbol_literal_rules,
    super_arguments_rules,
    style_rewrites,
    style_source,
    structural_completion_rules,
    structural_next_completion,
    structural_forwarding_completion,
    trailing_comma_completion,
    trailing_argument_comma_rules,
    trailing_underscore_rules,
    uri_regexp_rules,
    while_until_do_rules,
    yoda_condition_rules,
    string_conversion_rules,
    source_rules,
    source_rules_layout,
    source_rules_misc,
    source_semantics,
    ternary_rules,
    ternary_parentheses_rules,
    trivial_accessor_rules,
    gemspec_completion,
);

use crate::config::{CopConfig, RubyVersion};
use context_node_facade::*;
use cop_context::CopContext;
use cop_policy::CopPolicy;
use corrector::CorrectionPlan;
use diagnostic::{Context, Reporter};
pub(crate) use diagnostic::{Finding, Inspection};
use dsl::*;
use matchers::*;
use node_helpers::*;
use numeric_helpers::*;
pub use prism_engine::Engine;
use registry::Registry;
use rule_dsl::*;
use runner::Runner;
use source_file::{SourceEdit, SourceFile};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CopPhase {
    Source,
    Node,
    ParseErrorAndSource,
    SourceAndNode,
}

impl CopPhase {
    const fn visits_source(self) -> bool {
        matches!(
            self,
            Self::Source | Self::ParseErrorAndSource | Self::SourceAndNode
        )
    }

    const fn visits_nodes(self) -> bool {
        matches!(self, Self::Node | Self::SourceAndNode)
    }

    const fn visits_parse_errors(self) -> bool {
        matches!(self, Self::ParseErrorAndSource)
    }
}

pub(super) trait Cop: Sync {
    fn name(&self) -> &'static str;
    fn phase(&self) -> CopPhase {
        CopPhase::Node
    }
    fn on_source(&self, _source: &str, _context: &mut Context) {}
    fn on_parse_error(&self, _error: &Diagnostic<'_>, _source: &str, _context: &mut Context) {}
    fn visits_recovered_nodes(&self) -> bool {
        false
    }
    fn investigation_state(&self) -> Box<dyn Any> {
        Box::new(())
    }
    fn on_new_investigation(&self, _state: &mut dyn Any) {}
    fn on_node_with_state<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
        _state: &mut dyn Any,
    ) {
        self.on_node(node, ancestors, source, context);
    }
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

type Provider = fn() -> Vec<Box<dyn Cop>>;

pub(crate) fn cop_names() -> Vec<&'static str> {
    registry::cop_names()
}

impl Registry {
    fn enabled(enabled: &dyn Fn(&str) -> bool) -> Self {
        let cops = COP_PROVIDERS
            .iter()
            .flat_map(|provide| provide())
            .filter(|cop| !crate::cops::intentionally_pending(cop.name()) && enabled(cop.name()))
            .collect();

        Self::new(cops)
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
#[path = "tests/integration.rs"]
mod tests;
