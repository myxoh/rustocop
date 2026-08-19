use ruby_prism::{parse, CallNode, Diagnostic, Node, Visit};
use std::sync::Arc;

#[path = "framework/catalog_cop.rs"]
mod catalog_cop;
#[path = "framework/cop_context.rs"]
mod cop_context;
#[path = "framework/cop_policy.rs"]
mod cop_policy;
#[path = "framework/correction_engine.rs"]
mod correction_engine;
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
    accessor_rules,
    accessor_grouping_completion,
    additional_rules,
    additional_rules_literals,
    additional_rules_more,
    alias_rules,
    argument_and_inheritance_rules,
    assignment_completion_rules,
    block_association_rules,
    block_parameter_rules,
    block_chain_rules,
    block_arity_rules,
    branch_layout_rules,
    bundler_completion,
    call_conversion_rules,
    class_comparison_rules,
    class_definition_rules,
    class_methods_completion,
    compact_syntax_completion,
    compatibility_lexical_rules,
    comparable_clamp_rules,
    control_flow_completion_batch,
    control_semantics_completion_batch,
    collection_completion_rules,
    collection_query_rules,
    collection_transform_batch,
    coercion_rules,
    conditional_semantics_rules,
    declaration_semantics,
    declaration_completion_rules,
    deprecated_api_rules,
    directive_completion,
    dig_rules,
    double_splat_rules,
    empty_method_rules,
    enum_argument_rules,
    exception_argument_rules,
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
    file_structure_rules,
    file_predicate_rules,
    hash_array_rules,
    heredoc_call_rules,
    iteration_redundancy_rules,
    interpolation_condition_rules,
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
    layout_finalization_completion,
    layout_body_completion,
    layout_geometry_completion,
    lexical_completion,
    line_concatenation_rules,
    literal_and_pattern_rules,
    literal_integrity_completion,
    literal_string_completion_batch,
    logical_condition_rules,
    lookup_completion_rules,
    map_join_rules,
    method_layout_rules,
    method_signature_rules,
    metrics_naming_completion,
    metrics_completion,
    mixin_grouping_rules,
    mixin_rules,
    modern_collection_completion,
    nested_call_rules,
    nested_modifier_rules,
    nil_callable_rules,
    non_deterministic_require_rules,
    number_conversion_rules,
    numeric_operation_rules,
    numeric_predicate_rules,
    operator_ambiguity_rules,
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
    require_order_rules,
    rescue_rules,
    ruby2_keywords_rules,
    lexical_rules,
    security,
    self_rules,
    send_literal_rules,
    setter_rules,
    semantic_gap_completion,
    signal_exception_rules,
    single_line_block_rules,
    style,
    style_call_simplifications,
    style_calls,
    style_collections,
    style_compat,
    style_global_vars,
    style_metadata_completion,
    style_rewrites,
    style_source,
    structural_completion_rules,
    structural_next_completion,
    structural_forwarding_completion,
    trailing_comma_completion,
    string_conversion_rules,
    source_rules,
    source_rules_layout,
    source_rules_misc,
    source_semantics,
    ternary_rules,
    trivial_accessor_rules,
    gemspec_completion,
);

use crate::config::{CopConfig, Parallelism, RubyVersion};
use cop_context::{CopContext, CorrectionPlan};
use cop_policy::CopPolicy;
use diagnostic::{Context, Reporter};
pub(crate) use diagnostic::{Finding, Inspection};
use dsl::*;
use matchers::*;
use node_helpers::*;
use numeric_helpers::*;
pub use prism_engine::Engine;
use registry::Registry;
use runner::Runner;
use source_file::{SourceEdit, SourceFile};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(super) enum CopPhase {
    Source,
    Node,
    ParseError,
    SourceAndNode,
}

impl CopPhase {
    const fn visits_source(self) -> bool {
        matches!(self, Self::Source | Self::SourceAndNode)
    }

    const fn visits_nodes(self) -> bool {
        matches!(self, Self::Node | Self::SourceAndNode)
    }

    const fn visits_parse_errors(self) -> bool {
        matches!(self, Self::ParseError)
    }
}

pub(super) trait Cop: Sync {
    fn name(&self) -> &'static str;
    fn phase(&self) -> CopPhase {
        CopPhase::Node
    }
    fn on_source(&self, _source: &str, _context: &mut Context) {}
    fn on_parse_error(&self, _error: &Diagnostic<'_>, _source: &str, _context: &mut Context) {}
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
            .filter(|cop| enabled(cop.name()))
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
        crate::config::Parallelism::Sequential,
        target_ruby_version,
        Arc::new(CopConfig::default()),
    )
}

#[cfg(test)]
#[path = "tests/integration.rs"]
mod tests;
