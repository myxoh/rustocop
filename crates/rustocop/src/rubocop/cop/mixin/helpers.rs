// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/allowed_receivers.rb
// Source SHA-256: 86cc5646afc84609a3db64f830de7d5d7379e3b4aaba0435b8fd138744067dbd
// Source: lib/rubocop/cop/mixin/array_syntax.rb
// Source SHA-256: 1d105a1416c8136eed15528078e916ebbad39f5ad3abe52e39dc15a637568d0a
// Source: lib/rubocop/cop/mixin/auto_corrector.rb
// Source SHA-256: b3d64252a2b94d0d70c6498267640a40f4fbaccf361dd84596979ec727e8a862
// Source: lib/rubocop/cop/mixin/duplication.rb
// Source SHA-256: 1705b64f8ce47ad5eaf4a53f6de9ec752f958d0f40c1966c09209c1f587a74f8
// Source: lib/rubocop/cop/mixin/gem_declaration.rb
// Source SHA-256: 77cf3950ec2c8cdbbdb49126fc4c8d58b1f3aa8099456de9d86b6840492b07a9
// Source: lib/rubocop/cop/mixin/integer_node.rb
// Source SHA-256: 4f45d9c73ad6499efa8fc9434734c5f6c80f0b2b62a6fd4ed908a634672c3101
// Source: lib/rubocop/cop/mixin/method_preference.rb
// Source SHA-256: 21f8587e443387597f66081f3e0ec9e5f5f25f941bc6a916161a78081340d588
// Source: lib/rubocop/cop/mixin/parentheses.rb
// Source SHA-256: 5f9262feb944c2a4a7bfa0b6ac3f3b3e7437e2545f9c9a285422f19469a94fd3
// Source: lib/rubocop/cop/mixin/percent_literal.rb
// Source SHA-256: 31b3857808f2407ba446f890a48da0625b4334916929330f9e31c5b3bf8b3a70
// Source: lib/rubocop/cop/mixin/rational_literal.rb
// Source SHA-256: 54b0510a50707b0207a58b1b3ff016398dfdbf614f31892ce7f480d721f6cf8f
// Source: lib/rubocop/cop/mixin/safe_assignment.rb
// Source SHA-256: c9ff0cef41f5ed0cf5bee5fe6a793104d37cbdc8e81b73cb0f45ea95a50d5017
// Source: lib/rubocop/cop/mixin/string_literals_help.rb
// Source SHA-256: 0edf409bfef2ccff4f226ddb555c2e15aa5cab6184081524513657786b2c9b74
// Source: lib/rubocop/cop/mixin/trailing_body.rb
// Source SHA-256: 5c21e85e952edac8783365e5c2c55ad93959e4d731f9e9af544add2cedfbaab9
// Source: lib/rubocop/cop/mixin/multiline_element_line_breaks.rb
// Source SHA-256: 5bab6685d4f918d654d70a34e0db47637ff242b41f00419e51b4a7e6bc538643
// Source: lib/rubocop/cop/mixin/match_range.rb
// Source SHA-256: ccce795d7029dcf3c8a25b9008dda3ba4de5f089d8ca220b3e49742917ee9fe6
// Source: lib/rubocop/cop/mixin/nil_methods.rb
// Source SHA-256: 5f8dea22b982abeaa15c88026d6ec4915ebce0c9fd91db0d07230143600064f2
// Source: lib/rubocop/cop/mixin/empty_parameter.rb
// Source SHA-256: ce217120299a986d5f1357e8e5116ee1c2f6715d3c0c1bca1e17a0a1aedc77a5
// Source: lib/rubocop/cop/mixin/def_node.rb
// Source SHA-256: 60e8d419f8952c654b0851c1785b8a10c803e0f2da790f35fc25f16592da6517
// Source: lib/rubocop/cop/mixin/dig_help.rb
// Source SHA-256: a059d2f01cee858f0d7f858f1c59af10ebcc90868ea90fa214eca7490c433fef
// Source: lib/rubocop/cop/mixin/negative_conditional.rb
// Source SHA-256: dcc2e06ffab504e7fdb541441062a13e7889cecc705901771c58682b3000ca7f
// Source: lib/rubocop/cop/mixin/on_normal_if_unless.rb
// Source SHA-256: 7bb964f8f459ee892be1f68665012356baebcb4fdbc00d91b6639fcdcd4b67c7

use std::collections::{HashMap, HashSet};
use std::hash::Hash;
use std::ops::Range;

use regex::Regex;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Receiver {
    pub(crate) receiver: Option<Box<Receiver>>,
    pub(crate) constant: bool,
    pub(crate) send: bool,
    pub(crate) method_name: String,
    pub(crate) source: String,
}

pub(crate) fn receiver_name(receiver: &Receiver) -> String {
    if let Some(parent) = receiver.receiver.as_deref() {
        if !parent.constant {
            return receiver_name(parent);
        }
    }
    if receiver.send {
        receiver.receiver.as_deref().map_or_else(
            || receiver.method_name.clone(),
            |parent| format!("{}.{}", receiver_name(parent), receiver.method_name),
        )
    } else {
        receiver.source.clone()
    }
}

pub(crate) fn allowed_receiver(receiver: &Receiver, allowed_receivers: &[String]) -> bool {
    allowed_receivers.contains(&receiver_name(receiver))
}

pub(crate) const fn dig_chain_enabled(cop_enabled: bool) -> bool {
    cop_enabled
}

pub(crate) fn bracketed_array_of(
    square_brackets: bool,
    value_types: &[&str],
    element_type: &str,
) -> bool {
    square_brackets
        && !value_types.is_empty()
        && value_types
            .iter()
            .all(|value_type| *value_type == element_type)
}

pub(crate) const fn support_autocorrect() -> bool {
    true
}

pub(crate) fn duplicates<T: Eq + Hash + Clone>(collection: &[T]) -> Vec<T> {
    grouped_duplicates(collection)
        .into_iter()
        .flatten()
        .collect()
}

pub(crate) fn consecutive_duplicates<T: Eq + Hash + Clone>(collection: &[T]) -> Vec<T> {
    grouped_duplicates(collection)
        .into_iter()
        .flat_map(|items| items.into_iter().skip(1))
        .collect()
}

pub(crate) fn duplicates_exist<T: Eq + Hash>(collection: &[T]) -> bool {
    collection.len() > 1 && collection.iter().collect::<HashSet<_>>().len() < collection.len()
}

pub(crate) fn grouped_duplicates<T: Eq + Hash + Clone>(collection: &[T]) -> Vec<Vec<T>> {
    let mut indices = HashMap::<&T, usize>::new();
    let mut groups = Vec::<Vec<T>>::new();
    for item in collection {
        let index = *indices.entry(item).or_insert_with(|| {
            groups.push(Vec::new());
            groups.len() - 1
        });
        groups[index].push(item.clone());
    }
    groups.retain(|items| items.len() > 1);
    groups
}

pub(crate) fn gem_declaration(
    receiver_is_none: bool,
    method_name: &str,
    first_argument_is_string: bool,
) -> bool {
    receiver_is_none && method_name == "gem" && first_argument_is_string
}

pub(crate) fn integer_part(source: &str) -> &str {
    source
        .trim_start_matches(['+', '-'])
        .split(['e', 'E', '.'])
        .next()
        .unwrap_or("")
}

pub(crate) fn preferred_methods(
    default: &[(String, String)],
    merged: &[(String, String)],
) -> HashMap<String, String> {
    let default_values: HashSet<_> = default.iter().map(|(_, value)| value).collect();
    let overrides: HashSet<_> = merged
        .iter()
        .map(|(_, value)| value)
        .filter(|value| !default_values.contains(value))
        .collect();
    merged
        .iter()
        .filter(|(key, _)| !overrides.contains(key))
        .cloned()
        .collect()
}

pub(crate) fn preferred_method<'a>(
    methods: &'a HashMap<String, String>,
    method: &str,
) -> Option<&'a str> {
    methods.get(method).map(String::as_str)
}

pub(crate) fn default_cop_config(config: &[(String, String)]) -> &[(String, String)] {
    config
}

pub(crate) fn parens_required(source: &str, range: Range<usize>) -> bool {
    range
        .start
        .checked_sub(1)
        .and_then(|position| source.as_bytes().get(position))
        .is_some_and(u8::is_ascii_lowercase)
        || source
            .as_bytes()
            .get(range.end)
            .is_some_and(u8::is_ascii_lowercase)
}

pub(crate) fn percent_literal(begin_source: Option<&str>) -> bool {
    begin_source.is_some_and(|source| source.starts_with('%'))
}

pub(crate) fn percent_literal_type(begin_source: &str) -> &str {
    &begin_source[..begin_source.len().saturating_sub(1)]
}

pub(crate) fn process_percent_literal(begin_source: Option<&str>, accepted_types: &[&str]) -> bool {
    begin_source.is_some_and(|source| {
        percent_literal(Some(source)) && accepted_types.contains(&percent_literal_type(source))
    })
}

pub(crate) fn rational_literal(
    call_receiver_is_integer: bool,
    method_name: &str,
    first_argument_is_rational: bool,
) -> bool {
    call_receiver_is_integer && method_name == "/" && first_argument_is_rational
}

pub(crate) fn empty_condition(kind: &str, child_count: usize) -> bool {
    kind == "begin" && child_count == 0
}

pub(crate) fn safe_assignment(
    kind: &str,
    child_count: usize,
    child_equals_assignment: bool,
    child_setter_method: bool,
) -> bool {
    kind == "begin" && child_count == 1 && (child_equals_assignment || child_setter_method)
}

pub(crate) const fn safe_assignment_allowed(configured: bool) -> bool {
    configured
}

pub(crate) fn setter_method(node: crate::rubocop::ast::node::core::NodeRef<'_>) -> bool {
    node.loc("operator").is_some()
}

pub(crate) fn wrong_quotes(source: &str, single_quotes_style: bool) -> bool {
    if source.starts_with(['%', '?']) {
        return false;
    }
    if single_quotes_style {
        !double_quotes_required(source)
    } else {
        !(source.contains('"')
            || source
                .as_bytes()
                .windows(2)
                .any(|pair| pair[0] == b'\\' && !matches!(pair[1], b'\'' | b'\\'))
            || source.contains("#{")
            || source.contains("#@")
            || source.contains("#$"))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StringHelpAction {
    Skip,
    Ignore,
    Offense,
    CorrectStyle,
}

pub(crate) const fn string_help_on_str(
    has_begin_location: bool,
    part_of_ignored_node: bool,
    offense: bool,
) -> StringHelpAction {
    if !has_begin_location {
        StringHelpAction::Skip
    } else if part_of_ignored_node {
        StringHelpAction::Ignore
    } else if offense {
        StringHelpAction::Offense
    } else {
        StringHelpAction::CorrectStyle
    }
}

pub(crate) const fn string_help_on_regexp() -> StringHelpAction {
    StringHelpAction::Ignore
}

pub(crate) fn enforce_double_quotes(style: &str) -> bool {
    style == "double_quotes"
}

pub(crate) const fn string_literals_config(style: &str) -> &str {
    style
}

fn double_quotes_required(source: &str) -> bool {
    if source.contains('\'') {
        return true;
    }
    let bytes = source.as_bytes();
    for (index, byte) in bytes.iter().enumerate() {
        if *byte != b'\\' {
            continue;
        }
        let before_slashes = bytes[..index]
            .iter()
            .rev()
            .take_while(|byte| **byte == b'\\')
            .count();
        if before_slashes % 2 == 0 && !matches!(bytes.get(index + 1), Some(b'\\' | b'"')) {
            return true;
        }
    }
    false
}

pub(crate) const fn preferred_string_literal(enforce_double_quotes: bool) -> &'static str {
    if enforce_double_quotes {
        "\"\""
    } else {
        "''"
    }
}

pub(crate) fn trailing_body(
    has_body: bool,
    multiline: bool,
    node_line: usize,
    body_line: usize,
) -> bool {
    has_body && multiline && node_line == body_line
}

pub(crate) const fn body_on_first_line(node_line: usize, body_line: usize) -> bool {
    node_line == body_line
}

pub(crate) fn first_part_of(
    node: crate::rubocop::ast::node::core::NodeRef<'_>,
) -> Option<std::ops::Range<usize>> {
    let first = if node.kind() == "begin" {
        node.first_node()?
    } else {
        node
    };
    first.source_range()
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct LineSpan {
    pub(crate) first_line: usize,
    pub(crate) last_line: usize,
}

pub(crate) fn all_on_same_line(nodes: &[LineSpan], ignore_last: bool) -> bool {
    let Some(first) = nodes.first() else {
        return true;
    };
    let last = nodes.last().unwrap();
    if ignore_last {
        first.first_line == last.first_line
    } else {
        first.first_line == last.last_line
    }
}

pub(crate) fn missing_element_line_breaks(nodes: &[LineSpan], ignore_last: bool) -> Vec<usize> {
    if all_on_same_line(nodes, ignore_last) {
        return Vec::new();
    }
    let mut last_seen_line = None;
    let mut offenses = Vec::new();
    for (index, node) in nodes.iter().enumerate() {
        if last_seen_line.is_some_and(|line| line >= node.first_line) {
            offenses.push(index);
        } else {
            last_seen_line = Some(node.last_line);
        }
    }
    offenses
}

pub(crate) fn each_match_range(
    source: &str,
    base_begin: usize,
    regex: &Regex,
) -> Vec<Range<usize>> {
    regex
        .captures_iter(source)
        .filter_map(|capture| capture.get(1))
        .map(|capture| {
            let begin = source[..capture.start()].chars().count();
            let end = source[..capture.end()].chars().count();
            base_begin + begin..base_begin + end
        })
        .collect()
}

const NIL_METHODS: &str = "! != !~ & <=> == === =~ ^ __id__ __send__ class clone define_singleton_method display dup enum_for eql? equal? extend freeze frozen? hash inspect instance_eval instance_exec instance_of? instance_variable_defined? instance_variable_get instance_variable_set instance_variables is_a? itself kind_of? method methods nil? object_id private_methods protected_methods public_method public_methods public_send rationalize remove_instance_variable respond_to? send singleton_class singleton_method singleton_methods tap then to_a to_c to_enum to_f to_h to_i to_r to_s yield_self |";

pub(crate) fn nil_methods(allowed_methods: &[String]) -> Vec<String> {
    NIL_METHODS
        .split_whitespace()
        .chain(["to_d"])
        .map(str::to_owned)
        .chain(allowed_methods.iter().cloned())
        .collect()
}

pub(crate) fn other_stdlib_methods() -> Vec<String> {
    vec!["to_d".to_owned()]
}

pub(crate) fn single_negative(node: crate::rubocop::ast::node::core::NodeRef<'_>) -> bool {
    node.kind() == "send"
        && node.method_name() == Some("!")
        && node.receiver().is_some_and(|receiver| {
            !(receiver.kind() == "send" && receiver.method_name() == Some("!"))
        })
}

pub(crate) const fn empty_arguments(
    block_kind: bool,
    arguments_empty: bool,
    arguments_without_delimiters: bool,
) -> bool {
    block_kind && arguments_empty && !arguments_without_delimiters
}

pub(crate) fn non_public_modifier(
    send_receiver_is_none: bool,
    method_name: &str,
    argument_is_definition: bool,
) -> bool {
    send_receiver_is_none
        && matches!(
            method_name,
            "private" | "protected" | "private_class_method"
        )
        && argument_is_definition
}

pub(crate) fn non_public(parent_modifier: bool, preceding_visibility: &str) -> bool {
    parent_modifier || preceding_visibility != "public"
}

pub(crate) fn dig(method_name: &str, argument_kinds: &[&str]) -> bool {
    method_name == "dig"
        && !argument_kinds.is_empty()
        && argument_kinds
            .iter()
            .all(|kind| !matches!(*kind, "hash" | "block_pass"))
}

pub(crate) fn single_argument_dig(method_name: &str, argument_kinds: &[&str]) -> bool {
    method_name == "dig" && argument_kinds.len() == 1 && argument_kinds[0] != "splat"
}

pub(crate) const fn check_negative_conditional(
    empty_condition: bool,
    single_negative: bool,
    is_if: bool,
    has_else: bool,
) -> bool {
    !empty_condition && single_negative && !(is_if && has_else)
}

pub(crate) const fn on_normal_if_unless(modifier_form: bool, ternary: bool) -> bool {
    !modifier_form && !ternary
}
// RuboCop API ownership: lib/rubocop/cop/mixin/negative_conditional.rb => empty_condition
// RuboCop API ownership: lib/rubocop/cop/mixin/safe_assignment.rb => empty_condition
