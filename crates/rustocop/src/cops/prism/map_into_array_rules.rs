use regex::Regex;
use ruby_prism::{BlockNode, CallNode, Node};

use super::*;

define_cops! {
    MapIntoArray => "Style/MapIntoArray" => rubocop_callbacks(MapIntoArrayRule, [on_block]),
}

impl MapIntoArrayRule<'_, '_, '_> {
    fn on_block(&mut self, block: &BlockNode<'_>) {
        let Some(each) = self.parent().and_then(Node::as_call_node) else { return };
        return_unless!(each.name().as_slice() == b"each" && argument_count(&each) == 0);
        return_if!(each.receiver().is_none_or(|receiver| receiver.as_self_node().is_some()));
        let Some(push) = block.body().and_then(single_expression).and_then(|body| body.as_call_node()) else { return };
        return_unless!(matches!(push.name().as_slice(), b"<<" | b"push" | b"append"));
        let Some(destination) = push.receiver() else { return };
        let destination = self.source_file().node(&destination).to_string();
        return_unless!(destination.bytes().all(|byte| byte.is_ascii_alphanumeric() || byte == b'_'));
        let push_arguments = push.arguments().map(|arguments| arguments.arguments().iter().collect::<Vec<_>>()).unwrap_or_default();
        let [argument] = push_arguments.as_slice() else { return };
        return_if!(argument.as_splat_node().is_some()
            || argument.as_assoc_splat_node().is_some()
            || argument.as_block_argument_node().is_some()
            || self.source_file().node(argument).trim().starts_with("**")
            || self.source_file().node(argument).trim() == "...");
        return_if!(word_count(self.source_file().node(argument), &destination) > 0);

        let assignment = closest_empty_assignment(&each, &destination, self.source());
        let tap = if assignment.is_none() {
            empty_array_tap(self.ancestors(), &each, &destination, self.source_file())
        } else {
            None
        };
        return_if!(assignment.is_none() && tap.is_none());
        return_if!(self.ancestors().iter().any(|ancestor| {
            ancestor.as_array_node().is_some()
                || ancestor.as_case_node().is_some()
        }));
        if let Some(assignment) = assignment.as_ref() {
            let line_start = self.source_file().line_start(each.location().start_offset());
            return_if!(!self.source()[line_start..each.location().start_offset()].trim().is_empty());
            return_if!(self.source_file().indentation(assignment.range.start).len()
                != self.source_file().indentation(each.location().start_offset()).len());
            return_if!(self.ancestors().iter().any(|ancestor| {
                ancestor.as_block_node().is_some_and(|ancestor_block| {
                    ancestor_block.location().start_offset() != block.location().start_offset()
                        && ancestor_block.location().start_offset() > assignment.range.end
                        && ancestor_block.location().start_offset() < each.location().start_offset()
                })
            }));
            let span = assignment.range.start..each.location().end_offset();
            return_unless!(word_count(self.source_file().slice(span).unwrap_or_default(), &destination) == 2);
            return_if!(self.source()[assignment.range.end..each.location().start_offset()].lines().any(|line| line.trim() == "end"));
        }

        let new_method = self.related_config_map("Style/CollectionMethods", "PreferredMethods")
            .and_then(|methods| methods.get("map"))
            .cloned()
            .unwrap_or_else(|| "map".to_string());
        let message = format!("Use `{new_method}` instead of `each` to map elements into an array.");
        let offense = each.location();
        let selector = each.message_loc().expect("each has selector");
        let argument_range = argument.location().start_offset()..argument.location().end_offset();
        let push_range = push.location().start_offset()..push.location().end_offset();
        let hash_without_braces = argument.as_keyword_hash_node().is_some()
            && !self.source_file().node(argument).trim_start().starts_with('{');
        let trailing_destination = trailing_destination(&each, &destination, self.source());
        let return_value_used = tap.is_none() && return_value_used(self.ancestors(), &each, self.source_file());
        let assignment_removal = assignment.as_ref().map(|assignment| range_with_following_separator(assignment.range.clone(), self.source()));
        let tap_removals = tap.as_ref().map(|tap| (
            tap.location().start_offset()..each.location().start_offset(),
            each.location().end_offset()..tap.location().end_offset(),
        ));
        if return_value_used {
            self.report(message, offense);
            return;
        }
        add_offense!(self, offense, message: message, |corrector| {
            corrector.replace(selector, new_method);
            corrector.replace(push_range.start..argument_range.start, if hash_without_braces { "{ " } else { "" });
            corrector.replace(argument_range.end..push_range.end, if hash_without_braces { " }" } else { "" });
            corrector.replace(each.location().start_offset()..each.location().start_offset(), format!("{destination} = "));
            if let Some(range) = assignment_removal {
                corrector.remove(range);
            }
            if let Some((prefix, suffix)) = tap_removals {
                corrector.remove(prefix);
                corrector.remove(suffix);
            }
            if let Some(range) = trailing_destination {
                corrector.remove(range);
            }
        });
    }
}

fn word_count(source: &str, word: &str) -> usize {
    let comments = SourceFile::new(source).comment_ranges();
    Regex::new(&format!(r"\b{}\b", regex::escape(word)))
        .expect("escaped local name")
        .find_iter(source)
        .filter(|matched| {
            !comments
                .iter()
                .any(|comment| comment.start <= matched.start() && matched.start() < comment.end)
                && !matches!(
                source.as_bytes().get(matched.start().saturating_sub(1)),
                Some(b'@' | b'$' | b':' | b'.')
            )
        })
        .count()
}

#[derive(Clone)]
struct EmptyAssignment {
    range: std::ops::Range<usize>,
}

fn closest_empty_assignment(each: &CallNode<'_>, destination: &str, source: &str) -> Option<EmptyAssignment> {
    let pattern = Regex::new(&format!(
        r"\b{}\s*=\s*(?:\[\]|Array\.new\(\[\]\)|Array\.new|Array\[\]|Array\(\[\]\))",
        regex::escape(destination)
    )).ok()?;
    pattern.find_iter(&source[..each.location().start_offset()]).last().map(|matched| EmptyAssignment {
        range: matched.start()..matched.end(),
    })
}

fn trailing_destination(each: &CallNode<'_>, destination: &str, source: &str) -> Option<std::ops::Range<usize>> {
    let tail = source.get(each.location().end_offset()..)?;
    let pattern = Regex::new(&format!(r"\A(?:\s|;)*\b{}\b", regex::escape(destination))).ok()?;
    let matched = pattern.find(tail)?;
    Some(each.location().end_offset() + matched.start()..each.location().end_offset() + matched.end())
}

fn range_with_following_separator(location: std::ops::Range<usize>, source: &str) -> std::ops::Range<usize> {
    let mut end = location.end;
    while end < source.len() && matches!(source.as_bytes()[end], b' ' | b'\t') { end += 1; }
    if end < source.len() && source.as_bytes()[end] == b';' { end += 1; }
    while end < source.len() && source.as_bytes()[end].is_ascii_whitespace() { end += 1; }
    location.start..end
}

fn empty_array_tap<'pr>(ancestors: &[Node<'pr>], each: &CallNode<'pr>, destination: &str, file: SourceFile<'_>) -> Option<CallNode<'pr>> {
    for (index, ancestor) in ancestors.iter().enumerate().rev() {
        let Some(block) = ancestor.as_block_node() else { continue };
        if block_parameter(&block, file).as_deref() != Some(destination) { continue; }
        let parent = ancestors.get(index.wrapping_sub(1)).and_then(Node::as_call_node)?;
        if parent.name().as_slice() != b"tap"
            || parent.receiver().is_none_or(|receiver| file.node(&receiver) != "[]")
        {
            continue;
        }
        let only = block.body().and_then(single_expression)?;
        if only.location().start_offset() == each.location().start_offset() {
            return Some(parent);
        }
    }
    None
}

fn block_parameter(block: &BlockNode<'_>, file: SourceFile<'_>) -> Option<String> {
    let parameters = block.parameters()?;
    let source = file.node(&parameters).trim().trim_matches('|').trim();
    (!source.is_empty() && !source.contains(',')).then(|| source.to_string())
}

fn return_value_used(ancestors: &[Node<'_>], each: &CallNode<'_>, file: SourceFile<'_>) -> bool {
    for ancestor in ancestors.iter().rev() {
        if let Some(definition) = ancestor.as_def_node() {
            let name = file.at(&definition.name_loc());
            if name == "initialize" || name.ends_with('=') {
                return false;
            }
            let end = definition.end_keyword_loc().map_or(definition.location().end_offset(), |end| end.start_offset());
            return only_closing_syntax(file.slice(each.location().end_offset()..end).unwrap_or_default());
        }
        if let Some(block) = ancestor.as_block_node() {
            let owner = ancestors.iter().position(|node| {
                node.as_block_node().is_some_and(|candidate| candidate.location().start_offset() == block.location().start_offset())
            }).and_then(|index| index.checked_sub(1)).and_then(|index| ancestors.get(index)).and_then(Node::as_call_node);
            if let Some(owner) = owner {
                return owner.name().as_slice() != b"each"
                    && only_closing_syntax(file.slice(each.location().end_offset()..block.closing_loc().start_offset()).unwrap_or_default());
            }
        }
        if let Some(write) = ancestor.as_local_variable_write_node() {
            return only_closing_syntax(file.slice(each.location().end_offset()..write.location().end_offset()).unwrap_or_default());
        }
    }
    false
}

fn only_closing_syntax(source: &str) -> bool {
    let compact = source.chars().filter(|character| !character.is_whitespace() && *character != ';').collect::<String>();
    matches!(compact.as_str(), "" | ")" | "end" | ")end")
}
