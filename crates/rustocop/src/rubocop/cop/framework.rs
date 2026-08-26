#![allow(clippy::too_many_arguments, clippy::too_many_lines)]
// RuboCop 1.87.0 cop runtime compatibility.
// Source: lib/rubocop/cop/autocorrect_logic.rb
// Source SHA-256: 1308316998d7c517889c306e50f2ec225e8b76a78ed89d14cae4e894f22606ac
// Source: lib/rubocop/cop/base.rb
// Source SHA-256: aaa6f74be5b160ac7132c63e63293d59f8d0516ef00eb1266c54940671e14bd3
// Source: lib/rubocop/cop/commissioner.rb
// Source SHA-256: 152961ea12a39721a0ea78297f7bd6507111d3a02a3718dbcbcfe2afbd0b31dd
// Source: lib/rubocop/cop/cop.rb
// Source SHA-256: a075bd43cc6b5c90e0063cb755a2cc39f3e37ce977d85af1aa4962c0b3798b65
// Source: lib/rubocop/cop/generator.rb
// Source SHA-256: cfe84001c8a5c023786f90662376528a912ed7fb5716d40f556ade9173b951ea
// Source: lib/rubocop/cop/team.rb
// Source SHA-256: e7d2a5c11c922d13bc693fde2d9ae225d41317ecbb46f4a3f5356c63ef2fc840
// Source: lib/rubocop/cop/util.rb
// Source SHA-256: fd321331ae74b3d529fa1780814f5991b0e7eade584785c1ddf096c8ee2f8271
// Source: lib/rubocop/cop/variable_force.rb
// Source SHA-256: f6b843c42bc19bf9bd8a130951a183a08d8b3370981f499698bf04d2ff5e0328
// Source: lib/rubocop/cop/variable_force/assignment.rb
// Source SHA-256: b09940abf139100361e94cd21ed9bd72c3fafb85acf8e6eedcf1f88e731a6948
// Source: lib/rubocop/cop/variable_force/branch.rb
// Source SHA-256: 9e1146dca1c84c350032aea5e8f4338d1e3fe1d972e1d942e81839f0802bf661
// Source: lib/rubocop/cop/variable_force/branchable.rb
// Source SHA-256: 9a2712ae06f01d9dc810a6a4c8819cf45bc4069913430a82f122f8387eb0a6be
// Source: lib/rubocop/cop/variable_force/reference.rb
// Source SHA-256: e81d9b0b9df8eaebf569ca09e10eed9303e8ade0dd42a6eb58f7d06f7b89a0b0
// Source: lib/rubocop/cop/variable_force/scope.rb
// Source SHA-256: 5bcf514b3c4377f3bd69e9c794587b234f2cf2fa7de1b1c72b6109af7a552fbe
// Source: lib/rubocop/cop/variable_force/variable.rb
// Source SHA-256: 7ae86a03a6232f583ac6a7d15feb6b3846524570b29169b0aa8217240d1068f4
// Source: lib/rubocop/cop/variable_force/variable_table.rb
// Source SHA-256: e654e1b6a8d46416e11d03550a164560b839d0ab39b2ac74ad2c05c4cdd50b84
// RuboCop API ownership: lib/rubocop/cop/autocorrect_logic.rb => disable_uncorrectable
// RuboCop API ownership: lib/rubocop/cop/base.rb => add_offense, apply_correction, begin_investigation, cop, corrector, disable_uncorrectable, inherited, initialize, name, offenses, on_investigation_end, on_new_investigation, processed_source, range_for_original, support_autocorrect
// RuboCop API ownership: lib/rubocop/cop/cop.rb => add_offense, apply_correction, begin_investigation, cop, inherited, lambda, node, offenses, on_investigation_end, on_new_investigation, range_for_original, support_autocorrect
// RuboCop API ownership: lib/rubocop/cop/variable_force/assignment.rb => initialize, name, node, reference, referenced, references, used
// RuboCop API ownership: lib/rubocop/cop/variable_force/reference.rb => initialize, node, scope
// RuboCop API ownership: lib/rubocop/cop/variable_force/variable.rb => assignments, declaration_node, initialize, name, reference, referenced, references, scope, used

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::fmt;
use std::ops::Range;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::PathBuf;
use std::str::FromStr;

use regex::Regex;
use serde_json::Value;

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::{ParserEngine, ParserEngineError, ProcessedSource};
use crate::rubocop::ast::source::{SourceBuffer, SourceRange};

use super::corrector::{CorrectionError, Corrector};
use super::documentation::{self, DepartmentConfig};
use super::message_annotator::{CopMessageConfig, MessageAnnotator, MessageConfig, MessageOptions};

use super::severity::Severity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum AutocorrectMode {
    None,
    Safe,
    All,
    DisableUncorrectable,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AutocorrectLogic {
    pub(crate) mode: AutocorrectMode,
    pub(crate) supports_autocorrect: bool,
    pub(crate) safe_autocorrect: bool,
    pub(crate) enabled: bool,
    pub(crate) always_autocorrect: bool,
    pub(crate) contextual_autocorrect: bool,
    pub(crate) lsp_enabled: bool,
}
impl AutocorrectLogic {
    pub(crate) fn autocorrect_requested(&self) -> bool {
        self.mode != AutocorrectMode::None
    }
    pub(crate) fn autocorrect_enabled(&self) -> bool {
        if !self.enabled || (self.contextual_autocorrect && self.lsp_enabled) {
            return false;
        }
        if self.mode == AutocorrectMode::Safe {
            return self.safe_autocorrect;
        }
        true
    }
    pub(crate) fn autocorrect(&self) -> bool {
        self.autocorrect_requested() && self.correctable() && self.autocorrect_enabled()
    }
    pub(crate) fn correctable(&self) -> bool {
        self.supports_autocorrect || self.disable_uncorrectable()
    }
    pub(crate) fn disable_uncorrectable(&self) -> bool {
        self.mode == AutocorrectMode::DisableUncorrectable
    }
    pub(crate) fn autocorrect_with_disable_uncorrectable(&self) -> bool {
        self.autocorrect_requested() && self.disable_uncorrectable() && self.autocorrect_enabled()
    }
    pub(crate) fn safe_autocorrect(&self) -> bool {
        self.safe_autocorrect
    }
    pub(crate) fn contextual_autocorrect(&self) -> bool {
        self.contextual_autocorrect
    }
}
pub(crate) fn eol_comment(source_line: &str) -> Option<&str> {
    let mut quote = None;
    let mut escaped = false;
    for (index, ch) in source_line.char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if matches!(ch, '\'' | '\"') {
            if quote == Some(ch) {
                quote = None
            } else if quote.is_none() {
                quote = Some(ch)
            }
            continue;
        }
        if ch == '#' && quote.is_none() {
            return Some(&source_line[index..]);
        }
    }
    None
}
pub(crate) fn multiline_string(source: &str) -> bool {
    source.contains('\n')
        && ((source.starts_with("<<") || source.starts_with('%'))
            || (source.starts_with('"') && source.ends_with('"')))
}
pub(crate) fn line_with_comment_too_long(line: &str, comment: &str, max: usize) -> bool {
    line.chars().count() + comment.chars().count() + 1 > max
}
pub(crate) fn disable_offense_comment(cop_name: &str) -> String {
    format!("# rubocop:disable {cop_name}")
}
pub(crate) fn enable_offense_comment(cop_name: &str) -> String {
    format!("# rubocop:enable {cop_name}")
}
pub(crate) fn todo_offense_comment(cop_name: &str) -> String {
    format!(" # rubocop:todo {cop_name}")
}
pub(crate) fn range_of_first_line<'buffer, 'source>(
    range: SourceRange<'buffer, 'source>,
) -> SourceRange<'buffer, 'source> {
    let buffer = range.buffer();
    SourceRange::new(
        buffer,
        buffer.line_start(range.line()),
        buffer.line_range(range.line()).end,
    )
}
pub(crate) fn range_by_lines<'buffer, 'source>(
    range: SourceRange<'buffer, 'source>,
) -> SourceRange<'buffer, 'source> {
    let buffer = range.buffer();
    SourceRange::new(
        buffer,
        buffer.line_start(range.line()),
        buffer.line_range(range.last_line()).end,
    )
}
pub(crate) fn disable_offense_at_end_of_line(
    buffer: &SourceBuffer<'_>,
    range: SourceRange<'_, '_>,
    cop_name: &str,
) -> Result<String, CorrectionError> {
    let mut corrector = Corrector::new(buffer);
    corrector.insert_after(range, todo_offense_comment(cop_name));
    corrector.rewrite()
}
pub(crate) fn disable_offense_before_and_after(
    buffer: &SourceBuffer<'_>,
    range: SourceRange<'_, '_>,
    cop_name: &str,
) -> Result<String, CorrectionError> {
    let line_range = range_by_lines(range);
    let whitespace = line_range
        .source()
        .chars()
        .take_while(|character| character.is_whitespace() && *character != '\n')
        .collect::<String>();
    let mut corrector = Corrector::new(buffer);
    corrector.wrap(
        line_range,
        format!("{whitespace}# rubocop:todo {cop_name}\n"),
        format!("\n{whitespace}# rubocop:enable {cop_name}"),
    );
    corrector.rewrite()
}
pub(crate) fn max_line_length(enabled: bool, configured: Option<usize>) -> Option<usize> {
    enabled.then_some(configured.unwrap_or(120))
}
pub(crate) fn surrounding_heredoc(node: NodeRef<'_>) -> bool {
    node.type_is(&["any_str"]) && node.heredoc()
}
pub(crate) fn heredoc_range(node: NodeRef<'_>) -> Option<Range<usize>> {
    let expression = node.source_range()?;
    let end = node.loc("heredoc_end")?.0.clone();
    Some(expression.start.min(end.start)..expression.end.max(end.end))
}
pub(crate) fn surrounding_percent_array(node: NodeRef<'_>) -> bool {
    node.kind() == "array" && node.percent_literal(None)
}
pub(crate) fn string_continuation(node: NodeRef<'_>) -> bool {
    node.type_is(&["any_str"])
        && node
            .source()
            .is_some_and(|source| source.trim_end().ends_with('\\'))
}
pub(crate) fn multiline_string_node(node: NodeRef<'_>) -> bool {
    node.kind() == "dstr" && node.multiline()
}
pub(crate) fn multiline_ranges(root: NodeRef<'_>, offense: Range<usize>) -> Vec<Range<usize>> {
    if offense.is_empty() {
        return Vec::new();
    }
    root.each_node(&[])
        .into_iter()
        .filter_map(|node| {
            if surrounding_heredoc(node) {
                heredoc_range(node)
            } else if string_continuation(node) {
                node.source_range()
                    .map(|range| line_range_for_node(node, range))
            } else if surrounding_percent_array(node) || multiline_string_node(node) {
                node.source_range()
            } else {
                None
            }
        })
        .collect()
}

pub(crate) fn disable_offense(
    buffer: &SourceBuffer<'_>,
    root: Option<NodeRef<'_>>,
    offense: SourceRange<'_, '_>,
    cop_name: &str,
    max: Option<usize>,
) -> Result<String, CorrectionError> {
    let unbreakable = root.and_then(|root| {
        multiline_ranges(root, offense.begin_pos()..offense.end_pos())
            .into_iter()
            .find(|literal| {
                let literal = SourceRange::new(buffer, literal.start, literal.end);
                offense.overlaps(literal)
                    && eol_comment_would_be_inside_literal(buffer, offense, literal, cop_name, max)
            })
    });

    if let Some(unbreakable) = unbreakable {
        disable_offense_before_and_after(
            buffer,
            range_by_lines(SourceRange::new(buffer, unbreakable.start, unbreakable.end)),
            cop_name,
        )
    } else {
        disable_offense_with_eol_or_surround_comment(buffer, offense, cop_name, max)
    }
}

pub(crate) fn disable_offense_with_eol_or_surround_comment(
    buffer: &SourceBuffer<'_>,
    range: SourceRange<'_, '_>,
    cop_name: &str,
    max: Option<usize>,
) -> Result<String, CorrectionError> {
    if line_with_eol_comment_too_long_for_range(buffer, range, cop_name, max) {
        disable_offense_before_and_after(buffer, range_by_lines(range), cop_name)
    } else {
        disable_offense_at_end_of_line(buffer, range_of_first_line(range), cop_name)
    }
}
fn line_range_for_node(node: NodeRef<'_>, range: Range<usize>) -> Range<usize> {
    let source = node.source().unwrap_or("");
    let leading = source
        .chars()
        .take_while(|character| *character != '\n')
        .count();
    let trailing = source
        .chars()
        .rev()
        .take_while(|character| *character != '\n')
        .count();
    range.start.saturating_sub(node.column())
        ..range.end + trailing.saturating_sub(leading.min(trailing))
}
pub(crate) fn line_with_eol_comment_too_long_for_range(
    buffer: &SourceBuffer<'_>,
    range: SourceRange<'_, '_>,
    cop_name: &str,
    max: Option<usize>,
) -> bool {
    max.is_some_and(|max| {
        (range.source().chars().count() + todo_offense_comment(cop_name).chars().count()) > max
    }) || buffer.source_line(range.line()).chars().count()
        + todo_offense_comment(cop_name).chars().count()
        > max.unwrap_or(usize::MAX)
}
pub(crate) fn eol_comment_would_be_inside_literal(
    buffer: &SourceBuffer<'_>,
    offense: SourceRange<'_, '_>,
    literal: SourceRange<'_, '_>,
    cop_name: &str,
    max: Option<usize>,
) -> bool {
    line_with_eol_comment_too_long_for_range(buffer, offense, cop_name, max)
        || (offense.line() >= literal.line() && offense.line() < literal.last_line())
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Finding {
    pub(crate) location: Option<Range<usize>>,
    pub(crate) message: String,
    pub(crate) cop_name: String,
    pub(crate) severity: Severity,
    pub(crate) correctable: bool,
    pub(crate) corrected: bool,
}
impl Finding {
    pub(crate) fn new(
        cop_name: &str,
        range: Range<usize>,
        message: &str,
        severity: Severity,
        correctable: bool,
    ) -> Self {
        Self {
            location: Some(range),
            message: message.into(),
            cop_name: cop_name.into(),
            severity,
            correctable,
            corrected: false,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub(crate) struct BaseConfig {
    values: BTreeMap<String, Value>,
}
impl BaseConfig {
    pub(crate) fn new(values: BTreeMap<String, Value>) -> Self {
        Self { values }
    }
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        self.values.get(key)
    }
    pub(crate) fn bool(&self, key: &str, default: bool) -> bool {
        self.get(key).and_then(Value::as_bool).unwrap_or(default)
    }
    pub(crate) fn string(&self, key: &str) -> Option<&str> {
        self.get(key).and_then(Value::as_str)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct BaseCop {
    badge: String,
    config: BaseConfig,
    findings: Vec<Finding>,
    warnings: Vec<String>,
    disabled_lines: BTreeSet<usize>,
    excluded_patterns: Vec<String>,
    offense_locations: HashSet<(usize, usize)>,
    config_to_allow_offenses: BTreeMap<String, Value>,
    ready: bool,
    gem_requirements: BTreeMap<String, Vec<String>>,
    processed_source: Option<String>,
    project_index: Option<usize>,
}

pub(crate) struct BaseInvestigationReport<'buffer, 'source> {
    cop: String,
    processed_source: String,
    offenses: Vec<Finding>,
    corrector: Option<Corrector<'buffer, 'source>>,
}

impl<'buffer, 'source> BaseInvestigationReport<'buffer, 'source> {
    pub(crate) fn cop(&self) -> &str {
        &self.cop
    }
    pub(crate) fn processed_source(&self) -> &str {
        &self.processed_source
    }
    pub(crate) fn offenses(&self) -> &[Finding] {
        &self.offenses
    }
    pub(crate) fn corrector(&self) -> Option<&Corrector<'buffer, 'source>> {
        self.corrector.as_ref()
    }
}

pub(crate) type CorrectionFn = for<'buffer, 'source> fn(&mut Corrector<'buffer, 'source>);

pub(crate) struct CopCorrection<'ast> {
    correction: CorrectionFn,
    node: NodeRef<'ast>,
    cop: String,
}

impl<'ast> CopCorrection<'ast> {
    pub(crate) fn lambda(&self) -> CorrectionFn {
        self.correction
    }
    pub(crate) fn node(&self) -> NodeRef<'ast> {
        self.node
    }
    pub(crate) fn cop(&self) -> &str {
        &self.cop
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) enum NodeOrRange<'ast> {
    Node(NodeRef<'ast>),
    Range(&'ast Range<usize>),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CorrectionStatus {
    Unsupported,
    Uncorrected,
    Corrected,
    CorrectedWithTodo,
}

pub(crate) fn documentation_url(
    cop_name: &str,
    builtin: bool,
    config: Option<&DepartmentConfig>,
) -> Option<String> {
    let (department, name) = cop_name.rsplit_once('/')?;
    documentation::url_for(department, name, builtin, config)
}

pub(crate) fn qualified_cop_name(cop_name: &str, origin: &str) -> (String, String) {
    (
        cop_name.to_owned(),
        format!(
            "`Cop.qualified_cop_name` is deprecated. Use `Registry.qualified_cop_name` instead ({origin})."
        ),
    )
}

pub(crate) fn message(default_message: &str) -> &str {
    default_message
}

pub(crate) fn annotate(
    message: &str,
    config: &MessageConfig,
    cop_name: &str,
    cop_config: &CopMessageConfig,
    options: &MessageOptions,
) -> String {
    MessageAnnotator::new(config, cop_name, cop_config, options).annotate(message)
}

pub(crate) fn find_message(
    supplied: Option<&str>,
    default_message: &str,
    config: &MessageConfig,
    cop_name: &str,
    cop_config: &CopMessageConfig,
    options: &MessageOptions,
) -> String {
    annotate(
        supplied.unwrap_or_else(|| message(default_message)),
        config,
        cop_name,
        cop_config,
        options,
    )
}

pub(crate) fn range_from_node_or_range(input: NodeOrRange<'_>) -> Result<Range<usize>, String> {
    match input {
        NodeOrRange::Node(node) => node
            .source_range()
            .ok_or_else(|| format!("Expected a source-backed node, got {}", node.kind())),
        NodeOrRange::Range(range) => Ok(range.clone()),
    }
}

pub(crate) fn range_for_original(range: Range<usize>, offset: usize) -> Range<usize> {
    range.start.saturating_add(offset)..range.end.saturating_add(offset)
}

pub(crate) fn find_location(node: NodeRef<'_>, location: Option<&str>) -> Option<Range<usize>> {
    match location {
        Some(name) => node.loc(name).map(|location| location.0.clone()),
        None => node.source_range(),
    }
}

pub(crate) const fn support_autocorrect(has_autocorrect_method: bool) -> bool {
    has_autocorrect_method
}

pub(crate) const fn correction_lambda(
    supports_autocorrect: bool,
    already_corrected_node: bool,
) -> bool {
    supports_autocorrect && !already_corrected_node
}

pub(crate) fn current_corrector<'buffer, 'source>(
    source: &'buffer SourceBuffer<'source>,
    valid_syntax: bool,
) -> Option<Corrector<'buffer, 'source>> {
    valid_syntax.then(|| Corrector::new(source))
}

pub(crate) fn apply_correction<'buffer, 'source>(
    current: &mut Corrector<'buffer, 'source>,
    correction: Option<&Corrector<'buffer, 'source>>,
) {
    if let Some(correction) = correction {
        current.transaction(|current| current.merge(correction));
    }
}

pub(crate) fn attempt_correction<'buffer, 'source>(
    current: &mut Corrector<'buffer, 'source>,
    correction: Option<&Corrector<'buffer, 'source>>,
    disable_uncorrectable: bool,
) -> CorrectionStatus {
    if let Some(correction) = correction {
        apply_correction(current, Some(correction));
        CorrectionStatus::Corrected
    } else if disable_uncorrectable {
        CorrectionStatus::CorrectedWithTodo
    } else {
        CorrectionStatus::Unsupported
    }
}

pub(crate) fn use_corrector<'buffer, 'source>(
    current: &mut Corrector<'buffer, 'source>,
    correction: Option<&Corrector<'buffer, 'source>>,
    autocorrect: bool,
    disable_uncorrectable: bool,
    always_autocorrect: bool,
    contextual_autocorrect: bool,
    lsp_enabled: bool,
) -> CorrectionStatus {
    if autocorrect {
        attempt_correction(current, correction, disable_uncorrectable)
    } else if correction.is_some()
        && (always_autocorrect || (contextual_autocorrect && !lsp_enabled))
    {
        CorrectionStatus::Uncorrected
    } else {
        CorrectionStatus::Unsupported
    }
}

pub(crate) fn correct<'buffer, 'source>(
    current: &mut Corrector<'buffer, 'source>,
    correction: Option<&Corrector<'buffer, 'source>>,
    supports_autocorrect: bool,
    logic: &AutocorrectLogic,
) -> Result<CorrectionStatus, String> {
    if correction.is_some() && !supports_autocorrect {
        return Err("The Cop must extend AutoCorrector to be able to autocorrect".into());
    }
    Ok(use_corrector(
        current,
        correction,
        logic.autocorrect(),
        logic.disable_uncorrectable(),
        logic.always_autocorrect,
        logic.contextual_autocorrect,
        logic.lsp_enabled,
    ))
}

pub(crate) fn suppress_clobbering<T>(result: Result<T, CorrectionError>) -> Option<T> {
    result.ok()
}
pub(crate) fn registry<T>(global: &[T]) -> &[T] {
    global
}
pub(crate) fn all<T>(global: &[T]) -> &[T] {
    global
}
pub(crate) fn call<T>(correction: impl FnOnce() -> Result<T, String>) -> Result<T, String> {
    correction()
}
pub(crate) fn emulate_v0_callsequence(
    corrector_empty_after_block: bool,
    correction_lambda: Option<impl FnOnce() -> Result<(), CorrectionError>>,
) -> Result<(), String> {
    if !corrector_empty_after_block {
        return Err("Your cop must inherit from Cop::Base and extend AutoCorrector".into());
    }
    if let Some(correction) = correction_lambda {
        let _ = suppress_clobbering(correction());
    }
    Ok(())
}
impl BaseCop {
    pub(crate) fn initialize(config: Option<BaseConfig>) -> Self {
        Self::new("", config.unwrap_or_default())
    }

    pub(crate) fn inherited(subclass: &str) -> String {
        subclass.to_owned()
    }

    pub(crate) fn new(badge: &str, config: BaseConfig) -> Self {
        Self {
            badge: badge.into(),
            config,
            findings: Vec::new(),
            warnings: Vec::new(),
            disabled_lines: BTreeSet::new(),
            excluded_patterns: Vec::new(),
            offense_locations: HashSet::new(),
            config_to_allow_offenses: BTreeMap::new(),
            ready: false,
            gem_requirements: BTreeMap::new(),
            processed_source: None,
            project_index: None,
        }
    }
    pub(crate) fn config(&self) -> &BaseConfig {
        &self.config
    }
    pub(crate) fn gem_requirements(&self) -> &BTreeMap<String, Vec<String>> {
        &self.gem_requirements
    }
    pub(crate) fn processed_source(&self) -> Option<&str> {
        self.processed_source.as_deref()
    }
    pub(crate) fn set_processed_source(&mut self, processed_source: &ProcessedSource<'_>) {
        self.processed_source = Some(processed_source.raw_source().to_owned());
    }
    pub(crate) fn project_index(&self) -> Option<usize> {
        self.project_index
    }
    pub(crate) fn set_project_index(&mut self, project_index: Option<usize>) {
        self.project_index = project_index;
    }
    pub(crate) fn badge(&self) -> &str {
        &self.badge
    }
    pub(crate) fn inspect(&self) -> String {
        format!("#<{} @config={:?}>", self.badge, self.config)
    }
    pub(crate) fn requires_gem(&mut self, gem_name: &str, requirements: &[&str]) {
        self.gem_requirements.insert(
            gem_name.to_owned(),
            requirements
                .iter()
                .map(|requirement| (*requirement).to_owned())
                .collect(),
        );
    }
    pub(crate) fn target_satisfies_all_gem_version_requirements(&self) -> bool {
        self.gem_requirements.iter().all(|(name, requirements)| {
            let Some(version) = self
                .config
                .get("GemVersions")
                .and_then(Value::as_object)
                .and_then(|versions| versions.get(name))
                .and_then(Value::as_str)
            else {
                return false;
            };
            requirements
                .iter()
                .all(|requirement| gem_requirement_satisfied(version, requirement))
        })
    }
    pub(crate) fn cop_name(&self) -> &str {
        &self.badge
    }
    pub(crate) fn department(&self) -> Option<&str> {
        self.badge.split_once('/').map(|(department, _)| department)
    }
    pub(crate) fn cop_config(&self) -> &BaseConfig {
        &self.config
    }
    pub(crate) fn begin_investigation(&mut self) {
        self.findings.clear();
        self.warnings.clear();
        self.offense_locations.clear();
        self.ready = true
    }
    pub(crate) fn complete_investigation(&mut self) -> Vec<Finding> {
        self.ready = false;
        std::mem::take(&mut self.findings)
    }
    pub(crate) fn ready(&self) -> bool {
        self.ready
    }
    pub(crate) fn add_offense(
        &mut self,
        range: Range<usize>,
        message: &str,
        severity: Option<Severity>,
        correctable: bool,
    ) -> bool {
        if !self.offense_locations.insert((range.start, range.end)) {
            return false;
        }
        let line = range.start;
        if self.enabled_line(line) {
            if let Some(warning) = self.custom_severity_warning() {
                self.warnings.push(warning);
            }
            let severity = self.find_severity(severity);
            self.findings.push(Finding::new(
                &self.badge,
                range,
                message,
                severity,
                correctable,
            ));
            true
        } else {
            false
        }
    }
    pub(crate) fn add_global_offense(&mut self, message: &str, severity: Option<Severity>) {
        if let Some(warning) = self.custom_severity_warning() {
            self.warnings.push(warning);
        }
        self.findings.push(Finding {
            location: None,
            message: message.into(),
            cop_name: self.badge.clone(),
            severity: self.find_severity(severity),
            correctable: false,
            corrected: false,
        })
    }
    pub(crate) fn offenses(&self) -> &[Finding] {
        &self.findings
    }
    pub(crate) fn warnings(&self) -> &[String] {
        &self.warnings
    }
    pub(crate) fn disable_line(&mut self, line: usize) {
        self.disabled_lines.insert(line);
    }
    pub(crate) fn enabled_line(&self, line: usize) -> bool {
        !self.disabled_lines.contains(&line)
    }
    pub(crate) fn exclude(&mut self, pattern: &str) {
        self.excluded_patterns.push(pattern.into())
    }
    pub(crate) fn excluded_file(&self, path: &str) -> bool {
        self.excluded_patterns
            .iter()
            .any(|pattern| glob_match(pattern, path))
    }
    pub(crate) fn relevant_file(&self, path: &str) -> bool {
        !self.excluded_file(path)
    }
    pub(crate) fn lint(&self) -> bool {
        self.badge.starts_with("Lint/")
    }
    pub(crate) fn default_severity(&self) -> Severity {
        if self.lint() {
            Severity::Warning
        } else {
            Severity::Convention
        }
    }
    pub(crate) fn target_ruby_version(&self) -> f64 {
        self.config
            .get("TargetRubyVersion")
            .and_then(Value::as_f64)
            .unwrap_or(3.3)
    }
    pub(crate) fn parser_engine(&self) -> &str {
        self.config
            .string("ParserEngine")
            .unwrap_or("parser_whitequark")
    }
    pub(crate) fn always_autocorrect(&self) -> bool {
        matches!(
            self.config.get("AutoCorrect"),
            None | Some(Value::Bool(true))
        ) || self.config.string("AutoCorrect") == Some("always")
    }
    pub(crate) fn contextual_autocorrect(&self) -> bool {
        self.config.string("AutoCorrect") == Some("contextual")
    }
    pub(crate) fn target_rails_version(&self) -> Option<f64> {
        self.config
            .get("TargetRailsVersion")
            .and_then(Value::as_f64)
    }
    pub(crate) fn active_support_extensions_enabled(&self) -> bool {
        self.config.bool("ActiveSupportExtensionsEnabled", false)
    }
    pub(crate) fn string_literals_frozen_by_default(&self) -> bool {
        self.config.bool("StringLiteralsFrozenByDefault", false)
    }
    pub(crate) fn target_gem_version(&self, gem: &str) -> Option<&str> {
        self.config
            .get("GemVersions")
            .and_then(Value::as_object)
            .and_then(|versions| versions.get(gem))
            .and_then(Value::as_str)
    }
    pub(crate) fn custom_severity(&self) -> Option<Severity> {
        self.config
            .string("Severity")
            .and_then(|severity| Severity::from_str(severity).ok())
    }
    pub(crate) fn custom_severity_warning(&self) -> Option<String> {
        let severity = self.config.string("Severity")?;
        Severity::from_str(severity)
            .is_err()
            .then(|| format!("Warning: Invalid severity '{severity}'."))
    }
    pub(crate) fn find_severity(&self, severity: Option<Severity>) -> Severity {
        self.custom_severity()
            .or(severity)
            .unwrap_or_else(|| self.default_severity())
    }
    pub(crate) fn config_to_allow_offenses(&self) -> &BTreeMap<String, Value> {
        &self.config_to_allow_offenses
    }
    pub(crate) fn set_config_to_allow_offenses(&mut self, config: BTreeMap<String, Value>) {
        self.config_to_allow_offenses = config
    }
    pub(crate) fn current_offenses(&self) -> &[Finding] {
        &self.findings
    }
    pub(crate) fn current_offense_locations(&self) -> &HashSet<(usize, usize)> {
        &self.offense_locations
    }
    pub(crate) fn currently_disabled_lines(&self) -> &BTreeSet<usize> {
        &self.disabled_lines
    }
    pub(crate) fn file_name_matches_any(
        &self,
        file: &str,
        parameter: &str,
        default_result: bool,
    ) -> bool {
        let Some(patterns) = self.config.get(parameter) else {
            return default_result;
        };
        match patterns {
            Value::String(pattern) => glob_match(pattern, file),
            Value::Array(patterns) => patterns
                .iter()
                .filter_map(Value::as_str)
                .any(|pattern| glob_match(pattern, file)),
            _ => default_result,
        }
    }
    pub(crate) fn parse<'source>(
        &self,
        source: &'source str,
        path: Option<PathBuf>,
    ) -> Result<ProcessedSource<'source>, ParserEngineError> {
        let engine = if self.parser_engine().contains("prism") {
            ParserEngine::Prism
        } else {
            ParserEngine::Whitequark
        };
        ProcessedSource::new(source, self.target_ruby_version(), path, engine)
    }
    pub(crate) fn reset_investigation(&mut self) {
        self.findings.clear();
        self.offense_locations.clear();
        self.ready = false
    }
}

fn gem_requirement_satisfied(version: &str, requirement: &str) -> bool {
    let requirement = requirement.trim();
    let (operator, target) = [">=", "<=", "!=", "~>", ">", "<", "="]
        .into_iter()
        .find_map(|operator| {
            requirement
                .strip_prefix(operator)
                .map(|target| (operator, target.trim()))
        })
        .unwrap_or(("=", requirement));
    let parse = |value: &str| {
        value
            .split('.')
            .map(|part| part.parse::<u64>().unwrap_or(0))
            .collect::<Vec<_>>()
    };
    let current = parse(version);
    let target = parse(target);
    let compare = |left: &[u64], right: &[u64]| {
        let width = left.len().max(right.len());
        (0..width)
            .map(|index| left.get(index).copied().unwrap_or(0))
            .cmp((0..width).map(|index| right.get(index).copied().unwrap_or(0)))
    };
    match operator {
        ">=" => compare(&current, &target).is_ge(),
        "<=" => compare(&current, &target).is_le(),
        ">" => compare(&current, &target).is_gt(),
        "<" => compare(&current, &target).is_lt(),
        "!=" => compare(&current, &target).is_ne(),
        "=" => compare(&current, &target).is_eq(),
        "~>" => {
            let mut upper = target.clone();
            let bump = upper.len().saturating_sub(2);
            upper[bump] += 1;
            upper.truncate(bump + 1);
            compare(&current, &target).is_ge() && compare(&current, &upper).is_lt()
        }
        _ => false,
    }
}

pub(crate) trait CopRuntime {
    fn name(&self) -> &str;
    fn begin_investigation(&mut self, _source: &SourceBuffer<'_>) {}
    fn on_node(&mut self, _node: NodeRef<'_>) {}
    fn after_node(&mut self, _node: NodeRef<'_>) {}
    fn on_new_investigation(&mut self) {}
    fn on_other_file(&mut self) {}
    fn on_investigation_end(&mut self, _source: &SourceBuffer<'_>) {}
    fn take_findings(&mut self) -> Vec<Finding>;
    fn relevant_file(&self, _path: &str) -> bool {
        true
    }
    fn external_dependency_checksum(&self) -> Option<String> {
        None
    }
    fn callbacks_needed(&self) -> Option<&[&str]> {
        None
    }
    fn restrict_on_send(&self) -> &[&str] {
        &[]
    }
    fn take_correction(&mut self) -> Option<CorrectionPlan> {
        None
    }
    fn autocorrect_incompatible_with(&self) -> &[&str] {
        &[]
    }
    fn supports_autocorrect(&self) -> bool {
        false
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CorrectionPlan {
    edits: Vec<(Range<usize>, String)>,
}

impl CorrectionPlan {
    pub(crate) fn new() -> Self {
        Self::default()
    }
    pub(crate) fn replace(&mut self, range: Range<usize>, replacement: impl Into<String>) {
        self.edits.push((range, replacement.into()));
    }
    pub(crate) fn is_empty(&self) -> bool {
        self.edits.is_empty()
    }
    fn apply_to(&self, corrector: &mut Corrector<'_, '_>) -> Result<(), CorrectionError> {
        corrector.apply_edits(self.edits.iter().cloned())
    }

    pub(crate) fn apply_to_with_offset(
        &self,
        corrector: &mut Corrector<'_, '_>,
        offset: isize,
    ) -> Result<(), CorrectionError> {
        let edits = self
            .edits
            .iter()
            .map(|(range, replacement)| {
                let start = range
                    .start
                    .checked_add_signed(offset)
                    .ok_or(CorrectionError::InvalidRange)?;
                let end = range
                    .end
                    .checked_add_signed(offset)
                    .ok_or(CorrectionError::InvalidRange)?;
                Ok((start..end, replacement.clone()))
            })
            .collect::<Result<Vec<_>, _>>()?;
        corrector.apply_edits(edits)
    }
}

pub(crate) trait ForceRuntime {
    fn investigate(&mut self, processed_source: &ProcessedSource<'_>);
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CopError {
    pub(crate) cop_name: String,
    pub(crate) message: String,
    pub(crate) line: Option<usize>,
    pub(crate) column: Option<usize>,
}
impl fmt::Display for CopError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "An error occurred while {} cop was inspecting: {}",
            self.cop_name, self.message
        )
    }
}

#[derive(Default)]
pub(crate) struct Commissioner {
    cops: Vec<Box<dyn CopRuntime>>,
    forces: Vec<Box<dyn ForceRuntime>>,
    errors: Vec<CopError>,
    warnings: Vec<String>,
    raise_error: bool,
    raise_cop_error: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct InvestigationReport {
    pub(crate) cops: Vec<String>,
    pub(crate) offenses_per_cop: Vec<Vec<Finding>>,
    pub(crate) correctors: Vec<Option<CorrectionPlan>>,
    pub(crate) incompatible_cops: Vec<Vec<String>>,
    pub(crate) errors: Vec<CopError>,
}
impl InvestigationReport {
    pub(crate) fn offenses(&self) -> Vec<Finding> {
        self.offenses_per_cop.iter().flatten().cloned().collect()
    }
    pub(crate) fn merge(mut self, other: Self) -> Self {
        self.cops.extend(other.cops);
        self.offenses_per_cop.extend(other.offenses_per_cop);
        self.correctors.extend(other.correctors);
        self.incompatible_cops.extend(other.incompatible_cops);
        self.errors.extend(other.errors);
        self
    }
}
impl Commissioner {
    pub(crate) fn new(cops: Vec<Box<dyn CopRuntime>>) -> Self {
        Self {
            cops,
            forces: Vec::new(),
            errors: Vec::new(),
            warnings: Vec::new(),
            raise_error: false,
            raise_cop_error: false,
        }
    }
    pub(crate) fn with_runtime(
        cops: Vec<Box<dyn CopRuntime>>,
        forces: Vec<Box<dyn ForceRuntime>>,
        raise_error: bool,
        raise_cop_error: bool,
    ) -> Self {
        Self {
            cops,
            forces,
            errors: Vec::new(),
            warnings: Vec::new(),
            raise_error,
            raise_cop_error,
        }
    }
    pub(crate) fn cops(&self) -> &[Box<dyn CopRuntime>] {
        &self.cops
    }
    pub(crate) fn errors(&self) -> &[CopError] {
        &self.errors
    }
    pub(crate) fn warnings(&self) -> &[String] {
        &self.warnings
    }
    pub(crate) fn investigate(
        &mut self,
        source: &SourceBuffer<'_>,
        root: Option<NodeRef<'_>>,
    ) -> Vec<Finding> {
        self.investigate_selected(source, root, None)
    }
    fn investigate_selected(
        &mut self,
        source: &SourceBuffer<'_>,
        root: Option<NodeRef<'_>>,
        selected: Option<&BTreeSet<String>>,
    ) -> Vec<Finding> {
        let active = |cop: &dyn CopRuntime| selected.is_none_or(|names| names.contains(cop.name()));
        for cop in self.cops.iter_mut().filter(|cop| active(cop.as_ref())) {
            cop.begin_investigation(source)
        }
        for cop in self.cops.iter_mut().filter(|cop| active(cop.as_ref())) {
            cop.on_new_investigation();
        }
        if let Some(root) = root {
            self.walk_selected(root, selected);
        }
        for cop in self.cops.iter_mut().filter(|cop| active(cop.as_ref())) {
            cop.on_investigation_end(source)
        }
        let mut findings = self
            .cops
            .iter_mut()
            .filter(|cop| active(cop.as_ref()))
            .flat_map(|cop| cop.take_findings())
            .collect::<Vec<_>>();
        findings.sort_by_key(|finding| {
            (
                finding
                    .location
                    .as_ref()
                    .map_or(usize::MAX, |range| range.start),
                finding.cop_name.clone(),
                finding.message.clone(),
            )
        });
        findings
    }
    pub(crate) fn investigate_processed(
        &mut self,
        processed_source: &ProcessedSource<'_>,
    ) -> Vec<Finding> {
        self.errors.clear();
        let buffer = processed_source.buffer();
        for cop in &mut self.cops {
            cop.begin_investigation(&buffer);
        }
        if processed_source.valid_syntax() {
            for cop in &mut self.cops {
                cop.on_new_investigation();
            }
            for force in &mut self.forces {
                force.investigate(processed_source);
            }
            if let Some(root) = processed_source.ast() {
                self.walk(root);
            }
            for cop in &mut self.cops {
                cop.on_investigation_end(&buffer);
            }
        } else {
            for cop in &mut self.cops {
                cop.on_other_file();
            }
        }
        let mut findings = self
            .cops
            .iter_mut()
            .flat_map(|cop| cop.take_findings())
            .collect::<Vec<_>>();
        findings.sort_by_key(|finding| {
            (
                finding
                    .location
                    .as_ref()
                    .map_or(usize::MAX, |range| range.start),
                finding.cop_name.clone(),
                finding.message.clone(),
            )
        });
        findings
    }
    pub(crate) fn investigate_report(
        &mut self,
        processed_source: &ProcessedSource<'_>,
    ) -> InvestigationReport {
        let findings = self.investigate_processed(processed_source);
        self.finish_report(findings, None)
    }
    fn investigate_report_parts(
        &mut self,
        source: &SourceBuffer<'_>,
        root: Option<NodeRef<'_>>,
        selected: &BTreeSet<String>,
    ) -> InvestigationReport {
        let findings = self.investigate_selected(source, root, Some(selected));
        self.finish_report(findings, Some(selected))
    }
    fn finish_report(
        &mut self,
        findings: Vec<Finding>,
        selected: Option<&BTreeSet<String>>,
    ) -> InvestigationReport {
        let active_indices = self
            .cops
            .iter()
            .enumerate()
            .filter(|(_, cop)| selected.is_none_or(|names| names.contains(cop.name())))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let cops = active_indices
            .iter()
            .map(|index| self.cops[*index].name().to_owned())
            .collect::<Vec<_>>();
        let grouped = Self::offenses_per_cop(&findings);
        let incompatible_cops = active_indices
            .iter()
            .map(|index| {
                self.cops[*index]
                    .autocorrect_incompatible_with()
                    .iter()
                    .map(|name| (*name).to_owned())
                    .collect()
            })
            .collect();
        let correctors = active_indices
            .into_iter()
            .map(|index| self.cops[index].take_correction())
            .collect();
        InvestigationReport {
            offenses_per_cop: cops
                .iter()
                .map(|cop| grouped.get(cop).cloned().unwrap_or_default())
                .collect(),
            correctors,
            incompatible_cops,
            cops,
            errors: self.errors.clone(),
        }
    }
    fn walk(&mut self, node: NodeRef<'_>) {
        self.walk_selected(node, None);
    }
    fn walk_selected(&mut self, node: NodeRef<'_>, selected: Option<&BTreeSet<String>>) {
        self.trigger_selected(node, false, selected);
        let children = node.child_nodes();
        for child in &children {
            self.walk_selected(*child, selected);
        }
        if crate::rubocop::ast::traversal::has_compiled_child_traversal(node.kind()) {
            self.trigger_selected(node, true, selected);
        }
    }
    fn trigger(&mut self, node: NodeRef<'_>, after: bool) {
        self.trigger_selected(node, after, None);
    }
    fn trigger_selected(
        &mut self,
        node: NodeRef<'_>,
        after: bool,
        selected: Option<&BTreeSet<String>>,
    ) {
        for cop in self
            .cops
            .iter_mut()
            .filter(|cop| selected.is_none_or(|names| names.contains(cop.name())))
        {
            let event = if after {
                format!("after_{}", node.kind())
            } else {
                format!("on_{}", node.kind())
            };
            let responds = cop
                .callbacks_needed()
                .is_none_or(|callbacks| callbacks.contains(&event.as_str()));
            if !responds {
                continue;
            }
            let restricted =
                matches!(node.kind(), "send" | "csend") && !cop.restrict_on_send().is_empty();
            if restricted
                && !node
                    .method_name()
                    .is_some_and(|name| cop.restrict_on_send().contains(&name))
            {
                continue;
            }
            let result = catch_unwind(AssertUnwindSafe(|| {
                if after {
                    cop.after_node(node)
                } else {
                    cop.on_node(node)
                }
            }));
            if let Err(payload) = result {
                if self.raise_error || self.raise_cop_error {
                    std::panic::resume_unwind(payload);
                }
                let message = payload.downcast_ref::<&str>().map_or_else(
                    || "cop callback panicked".to_owned(),
                    |value| (*value).to_owned(),
                );
                self.errors.push(CopError {
                    cop_name: cop.name().into(),
                    message,
                    line: Some(node.first_line()),
                    column: Some(node.column()),
                });
            }
        }
    }
    pub(crate) fn offenses_per_cop(findings: &[Finding]) -> BTreeMap<String, Vec<Finding>> {
        let mut grouped = BTreeMap::new();
        for finding in findings {
            grouped
                .entry(finding.cop_name.clone())
                .or_insert_with(Vec::new)
                .push(finding.clone());
        }
        grouped
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamResult {
    pub(crate) findings: Vec<Finding>,
    pub(crate) errors: Vec<CopError>,
    pub(crate) updated_source: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct TeamInvestigation {
    pub(crate) result: TeamResult,
    pub(crate) correction: Option<CorrectionPlan>,
}

pub(crate) struct Team {
    commissioner: Commissioner,
    autocorrect: AutocorrectMode,
    max_iterations: usize,
}
impl Team {
    pub(crate) fn new(cops: Vec<Box<dyn CopRuntime>>, autocorrect: AutocorrectMode) -> Self {
        Self {
            commissioner: Commissioner::new(cops),
            autocorrect,
            max_iterations: 200,
        }
    }
    pub(crate) fn roundup_relevant_cops(&self, path: &str) -> Vec<&str> {
        self.commissioner
            .cops
            .iter()
            .filter(|cop| cop.relevant_file(path))
            .map(|cop| cop.name())
            .collect()
    }
    pub(crate) fn investigate(
        &mut self,
        path: &str,
        source: &SourceBuffer<'_>,
        root: Option<NodeRef<'_>>,
    ) -> TeamResult {
        self.investigate_with_correction(path, source, root).result
    }

    pub(crate) fn investigate_with_correction(
        &mut self,
        path: &str,
        source: &SourceBuffer<'_>,
        root: Option<NodeRef<'_>>,
    ) -> TeamInvestigation {
        self.investigate_selected(path, source, root, None)
    }

    pub(crate) fn investigate_with_selected_cops(
        &mut self,
        path: &str,
        source: &SourceBuffer<'_>,
        root: Option<NodeRef<'_>>,
        selected: &BTreeSet<String>,
    ) -> TeamInvestigation {
        self.investigate_selected(path, source, root, Some(selected))
    }

    fn investigate_selected(
        &mut self,
        path: &str,
        source: &SourceBuffer<'_>,
        root: Option<NodeRef<'_>>,
        selected: Option<&BTreeSet<String>>,
    ) -> TeamInvestigation {
        let relevant = self
            .commissioner
            .cops
            .iter()
            .filter(|cop| {
                cop.relevant_file(path) && selected.is_none_or(|names| names.contains(cop.name()))
            })
            .map(|cop| cop.name().to_owned())
            .collect::<BTreeSet<_>>();
        if relevant.is_empty() {
            return TeamInvestigation {
                result: TeamResult {
                    findings: Vec::new(),
                    errors: Vec::new(),
                    updated_source: None,
                },
                correction: None,
            };
        }
        let autocorrecting = self
            .commissioner
            .cops
            .iter()
            .filter(|cop| relevant.contains(cop.name()) && cop.supports_autocorrect())
            .map(|cop| cop.name().to_owned())
            .collect::<BTreeSet<_>>();
        let others = relevant
            .difference(&autocorrecting)
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut report = self
            .commissioner
            .investigate_report_parts(source, root, &autocorrecting);
        let (updated_source, correction) = if self.autocorrect() {
            let mut corrector = Corrector::new(source);
            let mut accepted = CorrectionPlan::new();
            let mut skips = BTreeSet::new();
            for ((cop, plan), incompatible) in report
                .cops
                .iter()
                .zip(&report.correctors)
                .zip(&report.incompatible_cops)
            {
                let Some(plan) = plan else { continue };
                if skips.contains(cop) || plan.is_empty() {
                    continue;
                }
                if plan.apply_to(&mut corrector).is_ok() {
                    accepted.edits.extend(plan.edits.iter().cloned());
                    skips.extend(incompatible.iter().cloned());
                }
            }
            let updated = (!corrector.is_empty())
                .then(|| corrector.rewrite().ok())
                .flatten();
            let correction = (updated.is_some() && !accepted.is_empty()).then_some(accepted);
            (updated, correction)
        } else {
            (None, None)
        };
        if updated_source.is_none() && !others.is_empty() {
            report = report.merge(
                self.commissioner
                    .investigate_report_parts(source, root, &others),
            );
        }
        let findings = report.offenses();
        TeamInvestigation {
            result: TeamResult {
                findings,
                errors: report.errors,
                updated_source,
            },
            correction,
        }
    }
    pub(crate) fn autocorrect(&self) -> bool {
        self.autocorrect != AutocorrectMode::None
    }
    pub(crate) fn max_iterations(&self) -> usize {
        self.max_iterations
    }
    pub(crate) fn external_dependency_checksum(&self) -> Vec<(String, String)> {
        self.commissioner
            .cops
            .iter()
            .filter_map(|cop| {
                cop.external_dependency_checksum()
                    .map(|sum| (cop.name().into(), sum))
            })
            .collect()
    }
}

pub(crate) fn source_line<'source>(buffer: &SourceBuffer<'source>, line: usize) -> &'source str {
    buffer.source_line(line)
}
pub(crate) fn buffer_line_range(buffer: &SourceBuffer<'_>, line: usize) -> Range<usize> {
    buffer.line_range(line)
}
pub(crate) fn line_range(node: NodeRef<'_>) -> std::ops::RangeInclusive<usize> {
    node.first_line()..=node.last_line()
}
pub(crate) trait SourceLinePosition {
    fn source_line_position(&self, buffer: &SourceBuffer<'_>) -> Option<usize>;
}
impl SourceLinePosition for Range<usize> {
    fn source_line_position(&self, buffer: &SourceBuffer<'_>) -> Option<usize> {
        Some(line_number(buffer, self.start))
    }
}
impl SourceLinePosition for NodeRef<'_> {
    fn source_line_position(&self, _buffer: &SourceBuffer<'_>) -> Option<usize> {
        Some(self.first_line())
    }
}
impl SourceLinePosition for usize {
    fn source_line_position(&self, _buffer: &SourceBuffer<'_>) -> Option<usize> {
        None
    }
}
pub(crate) fn line<T: SourceLinePosition>(value: &T, buffer: &SourceBuffer<'_>) -> Option<usize> {
    value.source_line_position(buffer)
}
pub(crate) fn same_line<L: SourceLinePosition, R: SourceLinePosition>(
    left: &L,
    right: &R,
    buffer: &SourceBuffer<'_>,
) -> bool {
    line(left, buffer)
        .zip(line(right, buffer))
        .is_some_and(|(left, right)| left == right)
}
pub(crate) fn begins_its_line(range: Range<usize>, buffer: &SourceBuffer<'_>) -> bool {
    buffer
        .slice(buffer.line_start(line_number(buffer, range.start))..range.start)
        .trim()
        .is_empty()
}
pub(crate) fn comment_line(line: &str) -> bool {
    line.trim_start().starts_with('#')
}
pub(crate) fn comment_lines(node: NodeRef<'_>, processed_source: &ProcessedSource<'_>) -> bool {
    processed_source
        .lines_slice(
            node.first_line().saturating_sub(1),
            node.last_line() - node.first_line() + 1,
        )
        .iter()
        .any(|line| comment_line(line))
}
pub(crate) fn parentheses(source: &str) -> bool {
    source.starts_with('(') && source.ends_with(')')
}
pub(crate) fn node_parentheses(node: NodeRef<'_>) -> bool {
    node.loc_is("end", ")")
}
pub(crate) fn any_descendant(
    node: NodeRef<'_>,
    types: &[&str],
    predicate: impl Fn(NodeRef<'_>) -> bool,
) -> bool {
    node.each_descendant(types).into_iter().any(predicate)
}
pub(crate) fn on_node<'ast>(
    types: &[&str],
    node: NodeRef<'ast>,
    excludes: &[&str],
) -> Vec<NodeRef<'ast>> {
    let mut result = Vec::new();
    fn visit<'ast>(
        types: &[&str],
        node: NodeRef<'ast>,
        excludes: &[&str],
        result: &mut Vec<NodeRef<'ast>>,
    ) {
        if types.is_empty() || node.type_is(types) {
            result.push(node);
        }
        if excludes.is_empty() || !node.type_is(excludes) {
            for child in node.child_nodes() {
                visit(types, child, excludes, result);
            }
        }
    }
    visit(types, node, excludes, &mut result);
    result
}
pub(crate) fn first_part_of_call_chain(mut node: NodeRef<'_>) -> NodeRef<'_> {
    loop {
        if node.type_is(&["call"]) {
            let Some(receiver) = node.receiver() else {
                return node;
            };
            node = receiver;
        } else if node.type_is(&["any_block"]) {
            let Some(send) = node.send_node() else {
                return node;
            };
            node = send;
        } else {
            return node;
        }
    }
}
pub(crate) fn args_begin(node: NodeRef<'_>) -> Option<Range<usize>> {
    let location = if node.type_is(&["super", "yield"]) {
        node.loc("keyword")
    } else if node.type_is(&["any_def"]) {
        node.loc("name")
    } else {
        node.loc("selector")
    }?;
    Some(location.0.end..location.0.end.saturating_add(1))
}
pub(crate) fn args_end(node: NodeRef<'_>) -> Option<usize> {
    node.source_range().map(|range| range.end)
}
pub(crate) fn include_or_equal<T: Ord>(range: Range<T>, value: &T) -> bool {
    range.start <= *value && *value <= range.end
}
pub(crate) fn indent(source: &str, width: usize) -> String {
    let prefix = " ".repeat(width);
    source
        .lines()
        .map(|line| format!("{prefix}{line}"))
        .collect::<Vec<_>>()
        .join("\n")
}
pub(crate) fn escape_string(source: &str) -> String {
    let escaped = format!("{source:?}");
    escaped[1..escaped.len() - 1].replace("\\\"", "\"")
}
pub(crate) fn interpret_string_escapes(source: &str) -> String {
    let mut out = String::new();
    let characters = source.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] != '\\' || index + 1 == characters.len() {
            out.push(characters[index]);
            index += 1;
            continue;
        }
        index += 1;
        let escaped = characters[index];
        index += 1;
        match escaped {
            'a' => out.push('\u{7}'),
            'b' => out.push('\u{8}'),
            'e' => out.push('\u{1b}'),
            'f' => out.push('\u{c}'),
            'n' => out.push('\n'),
            'r' => out.push('\r'),
            's' => out.push(' '),
            't' => out.push('\t'),
            'v' => out.push('\u{b}'),
            '\n' => {}
            '0'..='9' => {
                let mut digits = String::from(escaped);
                while digits.len() < 3
                    && index < characters.len()
                    && characters[index].is_ascii_digit()
                {
                    digits.push(characters[index]);
                    index += 1;
                }
                if let Ok(value) = u8::from_str_radix(&digits, 8) {
                    out.push(char::from(value));
                }
            }
            'x' => {
                let mut digits = String::new();
                while digits.len() < 2
                    && index < characters.len()
                    && characters[index].is_ascii_hexdigit()
                {
                    digits.push(characters[index]);
                    index += 1;
                }
                if let Ok(value) = u8::from_str_radix(&digits, 16) {
                    out.push(char::from(value));
                }
            }
            'u' if characters.get(index) == Some(&'{') => {
                index += 1;
                let start = index;
                while index < characters.len() && characters[index] != '}' {
                    index += 1;
                }
                let values = characters[start..index].iter().collect::<String>();
                index += usize::from(index < characters.len());
                for value in values.split_whitespace() {
                    if let Ok(codepoint) = u32::from_str_radix(value, 16) {
                        if let Some(character) = char::from_u32(codepoint) {
                            out.push(character);
                        }
                    }
                }
            }
            'u' => {
                let end = (index + 4).min(characters.len());
                let digits = characters[index..end].iter().collect::<String>();
                index = end;
                if let Ok(codepoint) = u32::from_str_radix(&digits, 16) {
                    if let Some(character) = char::from_u32(codepoint) {
                        out.push(character);
                    }
                }
            }
            other => out.push(other),
        }
    }
    out
}
pub(crate) fn needs_escaping(source: &str) -> bool {
    double_quotes_required(&escape_string(source))
}
pub(crate) fn to_string_literal(source: &str) -> String {
    if needs_escaping(source) {
        format!("{source:?}")
    } else {
        format!("'{}'", source.replace('\\', "\\\\"))
    }
}
pub(crate) fn double_quotes_required(source: &str) -> bool {
    if source.contains('\'') {
        return true;
    }
    let characters = source.chars().collect::<Vec<_>>();
    let mut index = 0;
    while index < characters.len() {
        if characters[index] != '\\' {
            index += 1;
            continue;
        }
        let start = index;
        while index < characters.len() && characters[index] == '\\' {
            index += 1;
        }
        let run = index - start;
        if run % 2 == 1
            && characters
                .get(index)
                .is_none_or(|character| *character != '"')
        {
            return true;
        }
    }
    false
}
pub(crate) fn add_parentheses(
    buffer: &SourceBuffer<'_>,
    node: NodeRef<'_>,
) -> Result<String, CorrectionError> {
    let Some(expression) = node.source_range() else {
        return Err(CorrectionError::InvalidRange);
    };
    let expression = SourceRange::new(buffer, expression.start, expression.end);
    let mut corrector = Corrector::new(buffer);
    if node.kind() == "args" {
        let begin = expression.begin_pos();
        if begin > 0 && buffer.character(begin - 1).is_some_and(char::is_whitespace) {
            corrector.replace(SourceRange::new(buffer, begin - 1, begin), "(");
        } else {
            corrector.insert_before(expression, "(");
        }
        corrector.insert_after(expression, ")");
    } else if !matches!(
        node.kind(),
        "send" | "csend" | "super" | "yield" | "def" | "defs"
    ) {
        corrector.wrap(expression, "(", ")");
    } else if node.arguments().is_empty() {
        corrector.insert_after(expression, "()");
    } else {
        let Some(begin) = args_begin(node) else {
            return Err(CorrectionError::InvalidRange);
        };
        let begin = SourceRange::new(buffer, begin.start, begin.end.min(buffer.len()));
        corrector.remove(begin);
        corrector.insert_before(begin, "(");
        corrector.insert_after(expression, ")");
    }
    corrector.rewrite()
}
pub(crate) fn trim_string_interpolation_escape(source: &str) -> String {
    source.replace("\\#{", "#{")
}
pub(crate) const fn compatible_external_encoding_for(_source: &str) -> bool {
    // Rust strings are valid UTF-8 by construction.
    true
}
pub(crate) fn to_supported_styles(enforced_style: &str) -> String {
    let supported = enforced_style.strip_prefix("Enforced").map_or_else(
        || enforced_style.to_owned(),
        |remainder| format!("Supported{remainder}"),
    );
    supported.replacen("Style", "Styles", 1)
}
pub(crate) fn parse_regexp(source: &str) -> Result<Regex, regex::Error> {
    let body = source
        .strip_prefix('/')
        .and_then(|s| s.rsplit_once('/').map(|(body, _)| body))
        .unwrap_or(source);
    Regex::new(body)
}
fn line_number(buffer: &SourceBuffer<'_>, position: usize) -> usize {
    (1..)
        .find(|line| buffer.line_range(*line).end >= position)
        .unwrap_or(1)
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Variable {
    pub(crate) name: String,
    pub(crate) assignments: Vec<Range<usize>>,
    pub(crate) references: Vec<Range<usize>>,
    pub(crate) scope: usize,
    declaration_kind: String,
    assignment_referenced: Vec<bool>,
    assignment_reassigned: Vec<bool>,
    captured_by_block: bool,
}
impl Variable {
    pub(crate) fn name(&self) -> &str {
        &self.name
    }
    pub(crate) fn assignments(&self) -> &[Range<usize>] {
        &self.assignments
    }
    pub(crate) fn references(&self) -> &[Range<usize>] {
        &self.references
    }
    pub(crate) fn scope(&self) -> usize {
        self.scope
    }
    pub(crate) fn declaration_node(&self) -> Option<&Range<usize>> {
        self.assignments.first()
    }

    pub(crate) fn mark_last_as_reassigned(&mut self, same_branch: bool) {
        if !self.captured_by_block && same_branch {
            if let Some(last) = self.assignment_reassigned.last_mut() {
                *last = true;
            }
        }
    }

    pub(crate) fn in_modifier_conditional(assignment: NodeRef<'_>) -> bool {
        let mut parent = assignment.parent();
        if parent.is_some_and(|node| node.kind() == "begin") {
            parent = parent.and_then(NodeRef::parent);
        }
        parent.is_some_and(|node| node.basic_conditional() && node.modifier_form())
    }

    pub(crate) fn referenced(&self) -> bool {
        !self.references.is_empty()
    }
    pub(crate) fn captured_by_block(&self) -> bool {
        self.captured_by_block
    }
    pub(crate) fn used(&self) -> bool {
        self.captured_by_block || self.referenced()
    }
    pub(crate) fn should_be_unused(&self) -> bool {
        self.name.starts_with('_')
    }
    pub(crate) fn argument(&self) -> bool {
        matches!(
            self.declaration_kind.as_str(),
            "arg"
                | "optarg"
                | "restarg"
                | "kwarg"
                | "kwoptarg"
                | "kwrestarg"
                | "blockarg"
                | "shadowarg"
        )
    }
    pub(crate) fn keyword_argument(&self) -> bool {
        matches!(self.declaration_kind.as_str(), "kwarg" | "kwoptarg")
    }
    pub(crate) fn explicit_block_local_variable(&self) -> bool {
        self.declaration_kind == "shadowarg"
    }
    pub(crate) fn assignment_used(&self, index: usize) -> bool {
        self.assignment_referenced
            .get(index)
            .copied()
            .unwrap_or(false)
            || self.captured_by_block
                && !self
                    .assignment_reassigned
                    .get(index)
                    .copied()
                    .unwrap_or(false)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum VariableScopeKind {
    TopLevel,
    Block,
    Method,
    ClassOrModule,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct VariableScope {
    kind: VariableScopeKind,
    variables: HashMap<String, Variable>,
}
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct VariableTable {
    scopes: Vec<VariableScope>,
    completed: Vec<Variable>,
    current_scope: usize,
}
impl VariableTable {
    pub(crate) fn new() -> Self {
        Self {
            scopes: vec![VariableScope {
                kind: VariableScopeKind::TopLevel,
                variables: HashMap::new(),
            }],
            completed: Vec::new(),
            current_scope: 0,
        }
    }
    pub(crate) fn enter_scope(&mut self) {
        self.enter_scope_kind(VariableScopeKind::Block);
    }
    pub(crate) fn enter_scope_kind(&mut self, kind: VariableScopeKind) {
        self.scopes.push(VariableScope {
            kind,
            variables: HashMap::new(),
        });
        self.current_scope = self.scopes.len() - 1
    }
    pub(crate) fn leave_scope(&mut self) -> Vec<Variable> {
        let vars: Vec<_> = self
            .scopes
            .pop()
            .unwrap_or_default()
            .variables
            .into_values()
            .collect();
        self.completed.extend(vars.iter().cloned());
        self.current_scope = self.scopes.len().saturating_sub(1);
        vars
    }
    pub(crate) fn assign(&mut self, name: &str, range: Range<usize>) {
        self.assign_kind(name, range, "lvasgn")
    }
    pub(crate) fn assign_kind(&mut self, name: &str, range: Range<usize>, kind: &str) {
        let variable_scope = self.find_variable_scope(name).unwrap_or(self.current_scope);
        let captured = self.scopes[self.current_scope].kind == VariableScopeKind::Block
            && variable_scope != self.current_scope;
        let variable = self.scopes[variable_scope]
            .variables
            .entry(name.into())
            .or_insert_with(|| Variable {
                name: name.into(),
                assignments: Vec::new(),
                references: Vec::new(),
                scope: variable_scope,
                declaration_kind: kind.into(),
                assignment_referenced: Vec::new(),
                assignment_reassigned: Vec::new(),
                captured_by_block: false,
            });
        if captured {
            variable.captured_by_block = true;
        }
        if !variable.captured_by_block {
            if let Some(last) = variable.assignment_reassigned.last_mut() {
                let last_referenced = variable
                    .assignment_referenced
                    .last()
                    .copied()
                    .unwrap_or(false);
                if !last_referenced {
                    *last = true;
                }
            }
        }
        variable.assignments.push(range);
        variable.assignment_referenced.push(false);
        variable.assignment_reassigned.push(false);
    }
    pub(crate) fn declare(&mut self, name: &str, _range: Range<usize>, kind: &str) {
        let scope = self.current_scope;
        self.scopes[scope]
            .variables
            .entry(name.into())
            .or_insert_with(|| Variable {
                name: name.into(),
                assignments: Vec::new(),
                references: Vec::new(),
                scope,
                declaration_kind: kind.into(),
                assignment_referenced: Vec::new(),
                assignment_reassigned: Vec::new(),
                captured_by_block: false,
            });
    }
    pub(crate) fn reference(&mut self, name: &str, range: Range<usize>) -> bool {
        let Some(scope) = self.find_variable_scope(name) else {
            return false;
        };
        let captured = self.scopes[self.current_scope].kind == VariableScopeKind::Block
            && scope != self.current_scope;
        let variable = self.scopes[scope].variables.get_mut(name).unwrap();
        variable.references.push(range);
        if let Some(referenced) = variable.assignment_referenced.last_mut() {
            *referenced = true;
        }
        variable.captured_by_block |= captured;
        true
    }
    pub(crate) fn variables(&self) -> Vec<&Variable> {
        self.scopes
            .iter()
            .flat_map(|scope| scope.variables.values())
            .chain(self.completed.iter())
            .collect()
    }
    pub(crate) fn accessible_variables(&self) -> Vec<&Variable> {
        let mut variables = Vec::new();
        for scope in self.scopes.iter().rev() {
            variables.extend(scope.variables.values());
            if scope.kind != VariableScopeKind::Block {
                break;
            }
        }
        variables
    }
    fn find_variable_scope(&self, name: &str) -> Option<usize> {
        for (index, scope) in self.scopes.iter().enumerate().rev() {
            if scope.variables.contains_key(name) {
                return Some(index);
            }
            if scope.kind != VariableScopeKind::Block {
                break;
            }
        }
        None
    }
    pub(crate) fn variable_exists(&self, name: &str) -> bool {
        self.find_variable_scope(name).is_some()
    }
}

impl Default for VariableScope {
    fn default() -> Self {
        Self {
            kind: VariableScopeKind::TopLevel,
            variables: HashMap::new(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VariableBranch<'ast> {
    pub(crate) control: NodeRef<'ast>,
    pub(crate) child: NodeRef<'ast>,
    scope_root: Option<NodeRef<'ast>>,
}
impl<'ast> VariableBranch<'ast> {
    pub(crate) fn of(target: NodeRef<'ast>, scope_root: Option<NodeRef<'ast>>) -> Option<Self> {
        for candidate in std::iter::once(target).chain(target.ancestors()) {
            if scope_root.is_some_and(|scope| candidate == scope) {
                return None;
            }
            let control = candidate.parent()?;
            if matches!(
                control.kind(),
                "if" | "while"
                    | "until"
                    | "while_post"
                    | "until_post"
                    | "case"
                    | "case_match"
                    | "for"
                    | "and"
                    | "and_asgn"
                    | "or"
                    | "or_asgn"
                    | "op_asgn"
                    | "rescue"
                    | "ensure"
            ) {
                let branch = Self {
                    control,
                    child: candidate,
                    scope_root,
                };
                if branch.branched() {
                    return Some(branch);
                }
            }
        }
        None
    }
    pub(crate) fn parent(self) -> Option<Self> {
        Self::of(self.control, self.scope_root)
    }
    pub(crate) fn always_run(self) -> bool {
        let index = self.child.sibling_index();
        match self.control.kind() {
            "if" | "while" | "until" | "while_post" | "until_post" => index == Some(0),
            "case" | "case_match" => index == Some(0),
            "for" => matches!(index, Some(0 | 1)),
            "and" | "and_asgn" | "or" | "or_asgn" | "op_asgn" => index == Some(0),
            "ensure" => index == Some(self.control.children().len().saturating_sub(1)),
            "rescue" => false,
            _ => true,
        }
    }
    pub(crate) fn branched(self) -> bool {
        !self.always_run()
    }
    pub(crate) fn may_jump_to_other_branch(self) -> bool {
        matches!(self.control.kind(), "rescue" | "ensure") && self.child.sibling_index() == Some(0)
    }
    pub(crate) fn may_run_incompletely(self) -> bool {
        self.may_jump_to_other_branch()
    }
    pub(crate) fn ancestors(self, include_self: bool) -> Vec<Self> {
        let mut branches = Vec::new();
        let mut current = include_self.then_some(self).or_else(|| self.parent());
        while let Some(branch) = current {
            branches.push(branch);
            current = branch.parent();
        }
        branches
    }
    pub(crate) fn exclusive_with(self, other: Option<Self>) -> bool {
        let Some(other) = other else {
            return false;
        };
        if self.may_jump_to_other_branch() {
            return false;
        }
        for ancestor in other.ancestors(true) {
            if self.control == ancestor.control {
                return self.child != ancestor.child;
            }
        }
        self.parent()
            .is_some_and(|parent| parent.exclusive_with(Some(other)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VariableReference<'ast> {
    pub(crate) node: NodeRef<'ast>,
    pub(crate) scope_root: Option<NodeRef<'ast>>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct VariableScopeView<'ast> {
    pub(crate) node: NodeRef<'ast>,
    naked_top_level: bool,
}
impl<'ast> VariableScopeView<'ast> {
    pub(crate) fn new(node: NodeRef<'ast>) -> Option<Self> {
        let scope = matches!(
            node.kind(),
            "block" | "numblock" | "itblock" | "class" | "sclass" | "defs" | "module" | "def"
        );
        (scope || node.root()).then_some(Self {
            node,
            naked_top_level: !scope,
        })
    }
    pub(crate) fn naked_top_level(self) -> bool {
        self.naked_top_level
    }
    pub(crate) fn name(self) -> Option<&'ast str> {
        self.node.method_name()
    }
    pub(crate) fn body(self) -> Option<NodeRef<'ast>> {
        if self.naked_top_level {
            Some(self.node)
        } else {
            self.node.body()
        }
    }
    pub(crate) fn includes(self, target: NodeRef<'ast>) -> bool {
        if target == self.node {
            return self.naked_top_level;
        }
        let mut child = target;
        loop {
            let Some(parent) = child.parent() else {
                return false;
            };
            if parent == self.node {
                return self.naked_top_level || !outer_scope_child(parent, child);
            }
            if is_variable_scope(parent) && !outer_scope_child(parent, child) {
                return false;
            }
            child = parent;
        }
    }
    pub(crate) fn nodes(self) -> Vec<NodeRef<'ast>> {
        let mut nodes = self
            .node
            .descendants()
            .into_iter()
            .filter(|node| self.includes(*node))
            .collect::<Vec<_>>();
        if self.naked_top_level {
            nodes.insert(0, self.node);
        }
        nodes
    }
}

fn is_variable_scope(node: NodeRef<'_>) -> bool {
    matches!(
        node.kind(),
        "block" | "numblock" | "itblock" | "class" | "sclass" | "defs" | "module" | "def"
    )
}
fn outer_scope_child(parent: NodeRef<'_>, child: NodeRef<'_>) -> bool {
    let index = child.sibling_index();
    match parent.kind() {
        "defs" | "module" | "sclass" | "block" | "numblock" | "itblock" => index == Some(0),
        "class" => matches!(index, Some(0 | 1)),
        _ => false,
    }
}

#[derive(Clone, Debug)]
pub(crate) struct VariableAssignment<'ast> {
    pub(crate) node: NodeRef<'ast>,
    pub(crate) references: Vec<NodeRef<'ast>>,
    referenced: bool,
    reassigned: bool,
    captured_by_block: bool,
    scope_root: Option<NodeRef<'ast>>,
}
impl<'ast> VariableAssignment<'ast> {
    pub(crate) fn node(&self) -> NodeRef<'ast> {
        self.node
    }
    pub(crate) fn variable(&self) -> Option<&'ast str> {
        self.node.name()
    }
    pub(crate) fn references(&self) -> &[NodeRef<'ast>] {
        &self.references
    }

    pub(crate) fn new(node: NodeRef<'ast>, scope_root: Option<NodeRef<'ast>>) -> Option<Self> {
        matches!(node.kind(), "lvasgn" | "match_with_lvasgn" | "match_var").then_some(Self {
            node,
            references: Vec::new(),
            referenced: false,
            reassigned: false,
            captured_by_block: false,
            scope_root,
        })
    }
    pub(crate) fn name(&self) -> Option<&'ast str> {
        self.node.name()
    }
    pub(crate) fn reference(&mut self, node: NodeRef<'ast>) {
        self.references.push(node);
        self.referenced = true;
    }
    pub(crate) fn reassign(&mut self) {
        if !self.referenced {
            self.reassigned = true;
        }
    }
    pub(crate) fn referenced(&self) -> bool {
        self.referenced
    }
    pub(crate) fn reassigned(&self) -> bool {
        self.reassigned
    }
    pub(crate) fn capture_with_block(&mut self) {
        self.captured_by_block = true;
    }
    pub(crate) fn used(&self) -> bool {
        (!self.reassigned && self.captured_by_block) || self.referenced
    }
    pub(crate) fn branch(&self) -> Option<VariableBranch<'ast>> {
        VariableBranch::of(self.node, self.scope_root)
    }
    pub(crate) fn runs_exclusively_with(&self, reference: VariableReference<'ast>) -> bool {
        self.branch()
            .is_some_and(|branch| branch.exclusive_with(reference.branch()))
    }
    pub(crate) fn regexp_named_capture(&self) -> bool {
        self.node.kind() == "match_with_lvasgn"
    }
    pub(crate) fn exception_assignment(&self) -> bool {
        self.node.parent().is_some_and(|parent| {
            parent.kind() == "resbody" && parent.exception_variable() == Some(self.node)
        })
    }
    pub(crate) fn meta_assignment_node(&self) -> Option<NodeRef<'ast>> {
        let parent = self.node.parent()?;
        if matches!(parent.kind(), "op_asgn" | "or_asgn" | "and_asgn")
            && self.node.sibling_index() == Some(0)
        {
            return Some(parent);
        }
        if parent.kind() == "splat" && matches!(parent.parent()?.kind(), "masgn" | "mlhs" | "for") {
            return parent.parent();
        }
        let mut candidate = parent;
        while candidate.kind() == "mlhs" {
            candidate = candidate.parent()?;
        }
        matches!(candidate.kind(), "masgn" | "for" | "splat").then_some(candidate)
    }
    pub(crate) fn operator_assignment(&self) -> bool {
        self.meta_assignment_node()
            .is_some_and(|node| matches!(node.kind(), "op_asgn" | "or_asgn" | "and_asgn"))
    }
    pub(crate) fn multiple_assignment(&self) -> bool {
        self.meta_assignment_node()
            .is_some_and(|node| node.kind() == "masgn")
    }
    pub(crate) fn rest_assignment(&self) -> bool {
        self.meta_assignment_node()
            .is_some_and(|node| node.kind() == "splat")
    }
    pub(crate) fn for_assignment(&self) -> bool {
        self.meta_assignment_node()
            .is_some_and(|node| node.kind() == "for")
    }
    pub(crate) fn operator(&self) -> Option<String> {
        let node = self.meta_assignment_node().unwrap_or(self.node);
        node.loc("operator")
            .map(|(_, source)| source.clone())
            .or_else(|| node.operator().map(|operator| format!("{operator}=")))
    }
}
impl<'ast> VariableReference<'ast> {
    pub(crate) fn node(&self) -> NodeRef<'ast> {
        self.node
    }
    pub(crate) fn scope(&self) -> Option<NodeRef<'ast>> {
        self.scope_root
    }

    pub(crate) fn new(node: NodeRef<'ast>, scope_root: Option<NodeRef<'ast>>) -> Option<Self> {
        matches!(
            node.kind(),
            "lvar" | "op_asgn" | "or_asgn" | "and_asgn" | "zsuper" | "send"
        )
        .then_some(Self { node, scope_root })
    }
    pub(crate) fn explicit(self) -> bool {
        !matches!(self.node.kind(), "zsuper" | "send")
    }
    pub(crate) fn branch(self) -> Option<VariableBranch<'ast>> {
        VariableBranch::of(self.node, self.scope_root)
    }
    pub(crate) fn runs_exclusively_with(self, other: Self) -> bool {
        self.branch()
            .is_some_and(|branch| branch.exclusive_with(other.branch()))
    }
}
pub(crate) fn assignment_node(node: NodeRef<'_>) -> bool {
    node.assignment()
        || matches!(
            node.kind(),
            "arg" | "optarg" | "kwarg" | "kwoptarg" | "restarg" | "kwrestarg" | "blockarg"
        )
}
pub(crate) fn variable_name(node: NodeRef<'_>) -> Option<&str> {
    node.name()
}
pub(crate) fn scan_variables(root: NodeRef<'_>) -> VariableTable {
    let mut table = VariableTable::new();
    #[allow(clippy::cognitive_complexity)] // Branch order mirrors VariableForce traversal semantics.
    fn process(node: NodeRef<'_>, table: &mut VariableTable) {
        let range = node.source_range().unwrap_or(0..0);
        match node.kind() {
            "lvasgn" | "match_with_lvasgn" | "match_var" => {
                for child in node.child_nodes() {
                    process(child, table);
                }
                if let Some(name) = variable_name(node) {
                    if !table.variable_exists(name) {
                        table.declare(name, range.clone(), node.kind());
                    }
                    table.assign_kind(name, range, node.kind());
                }
            }
            "arg" | "optarg" | "restarg" | "kwarg" | "kwoptarg" | "kwrestarg" | "blockarg"
            | "shadowarg" => {
                if let Some(name) = variable_name(node) {
                    table.declare(name, range, node.kind());
                }
                for child in node.child_nodes() {
                    process(child, table);
                }
            }
            "lvar" => {
                if let Some(name) = variable_name(node) {
                    table.reference(name, range);
                }
            }
            "op_asgn" | "or_asgn" | "and_asgn" => {
                if let Some(lhs) = node.node_child(0) {
                    if let Some(name) = variable_name(lhs) {
                        if !table.variable_exists(name) {
                            table.declare(name, lhs.source_range().unwrap_or(0..0), lhs.kind());
                        }
                        table.reference(name, range.clone());
                        if let Some(rhs) = node.rhs() {
                            process(rhs, table);
                        }
                        table.assign_kind(name, lhs.source_range().unwrap_or(range), lhs.kind());
                    }
                }
            }
            "masgn" => {
                if let Some(rhs) = node.node_child(1) {
                    process(rhs, table);
                }
                if let Some(lhs) = node.node_child(0) {
                    process(lhs, table);
                }
            }
            "while_post" | "until_post" => {
                if let Some(body) = node.node_child(1) {
                    process(body, table);
                }
                if let Some(condition) = node.node_child(0) {
                    process(condition, table);
                }
            }
            "block" | "numblock" | "itblock" => {
                if let Some(call) = node.node_child(0) {
                    process(call, table);
                }
                table.enter_scope_kind(VariableScopeKind::Block);
                if let Some(arguments) = node.node_child(1) {
                    process(arguments, table);
                }
                if let Some(body) = node.node_child(2) {
                    process(body, table);
                }
                table.leave_scope();
            }
            "def" | "defs" => {
                if node.kind() == "defs" {
                    if let Some(receiver) = node.node_child(0) {
                        process(receiver, table);
                    }
                }
                table.enter_scope_kind(VariableScopeKind::Method);
                if let Some(arguments) = node.arguments_node() {
                    process(arguments, table);
                }
                if let Some(body) = node.body() {
                    process(body, table);
                }
                table.leave_scope();
            }
            "class" | "module" | "sclass" => {
                if let Some(identifier) = node.node_child(0) {
                    process(identifier, table);
                }
                if node.kind() == "class" {
                    if let Some(superclass) = node.node_child(1) {
                        process(superclass, table);
                    }
                }
                table.enter_scope_kind(VariableScopeKind::ClassOrModule);
                if let Some(body) = node.body() {
                    process(body, table);
                }
                table.leave_scope();
            }
            _ => {
                for child in node.child_nodes() {
                    process(child, table);
                }
            }
        }
    }
    process(root, &mut table);
    table
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GeneratedCop {
    pub(crate) source_path: String,
    pub(crate) spec_path: String,
    pub(crate) source: String,
    pub(crate) spec: String,
    pub(crate) config: String,
}
pub(crate) struct Generator;
impl Generator {
    pub(crate) fn snake_case(name: &str) -> String {
        let name = name.replace("RSpec", "Rspec");
        let first = Regex::new(r"([^A-Z/])([A-Z]+)")
            .expect("static regex")
            .replace_all(&name, "${1}_${2}");
        Regex::new(r"([A-Z])([A-Z][^A-Z\d/]+)")
            .expect("static regex")
            .replace_all(&first, "${1}_${2}")
            .to_ascii_lowercase()
    }
    pub(crate) fn generate(
        qualified_name: &str,
        description: &str,
    ) -> Result<GeneratedCop, String> {
        let (department, name) = qualified_name
            .rsplit_once('/')
            .ok_or_else(|| "Cop name must be Department/CopName".to_string())?;
        if !name
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_uppercase())
        {
            return Err("Cop name must be CamelCase".into());
        }
        let snake = Self::snake_case(name);
        let department_snake = Self::snake_case(department);
        let department_namespace = department.replace('/', "::");
        let source_path = format!("lib/rubocop/cop/{department_snake}/{snake}.rb");
        let spec_path = format!("spec/rubocop/cop/{department_snake}/{snake}_spec.rb");
        let source=format!("# frozen_string_literal: true\n\nmodule RuboCop\n  module Cop\n    module {department_namespace}\n      # {description}\n      class {name} < Base\n      end\n    end\n  end\nend\n");
        let spec=format!("# frozen_string_literal: true\n\nRSpec.describe RuboCop::Cop::{department_namespace}::{name} do\n  pending 'add some examples'\nend\n");
        let config =
            format!("{department}/{name}:\n  Description: '{description}'\n  Enabled: pending\n");
        Ok(GeneratedCop {
            source_path,
            spec_path,
            source,
            spec,
            config,
        })
    }
    pub(crate) fn todo(qualified_name: &str) -> Result<String, String> {
        let _ = qualified_name
            .rsplit_once('/')
            .ok_or_else(|| "Specify a cop name with Department/Name style".to_owned())?;
        Ok(format!(
            "Do 4 steps:\n  1. Modify the description of {qualified_name} in config/default.yml\n  2. Implement your new cop in the generated file!\n  3. Commit your new cop with a message such as\n     e.g. \"Add new `{qualified_name}` cop\"\n  4. Run `bundle exec rake changelog:new` to generate a changelog entry\n     for your new cop.\n"
        ))
    }
    pub(crate) fn inject_config(
        source: &str,
        qualified_name: &str,
        version_added: &str,
    ) -> Result<String, String> {
        if !qualified_name.contains('/') {
            return Err("Specify a cop name with Department/Name style".into());
        }
        let entry = format!(
            "{qualified_name}:\n  Description: 'TODO: Write a description of the cop.'\n  Enabled: pending\n  VersionAdded: '{version_added}'"
        );
        let mut sections = source
            .trim_end()
            .split("\n\n")
            .map(str::to_owned)
            .collect::<Vec<_>>();
        sections.push(entry);
        sections.sort_by(|left, right| {
            left.lines()
                .next()
                .unwrap_or("")
                .cmp(right.lines().next().unwrap_or(""))
        });
        Ok(format!("{}\n", sections.join("\n\n")))
    }
}

fn glob_match(pattern: &str, path: &str) -> bool {
    let regex = format!(
        "^{}$",
        regex::escape(pattern)
            .replace(r"\*\*", ".*")
            .replace(r"\*", "[^/]*")
            .replace(r"\?", ".")
    );
    Regex::new(&regex).is_ok_and(|matcher| matcher.is_match(path))
}

pub(crate) fn dedupe_findings(findings: &mut Vec<Finding>) {
    let mut seen = HashSet::new();
    findings.retain(|finding| {
        seen.insert((
            finding.cop_name.clone(),
            finding.location.clone(),
            finding.message.clone(),
        ))
    })
}
