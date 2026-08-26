use ruby_prism::{AssocNode, HashNode, Node};

use super::*;

define_cops! {
    HashSyntax => "Style/HashSyntax" => rubocop_callbacks(HashSyntaxRule, [on_hash, on_keyword_hash]),
}

impl HashSyntaxRule<'_, '_, '_> {
    fn on_hash(&mut self, node: &HashNode<'_>) {
        let pairs = node.elements().iter().filter_map(|element| element.as_assoc_node()).collect::<Vec<_>>();
        self.check_pairs(pairs);
    }

    fn on_keyword_hash(&mut self, node: &ruby_prism::KeywordHashNode<'_>) {
        let pairs = node.elements().iter().filter_map(|element| element.as_assoc_node()).collect::<Vec<_>>();
        self.check_pairs(pairs);
    }

    fn check_pairs(&mut self, pairs: Vec<AssocNode<'_>>) {
        return_if!(pairs.is_empty());
        self.check_shorthand(&pairs);

        let style = self.policy().enforced_style("ruby19").to_string();
        let force_rockets = self.config_bool("UseHashRocketsWithSymbolValues", false)
            && pairs.iter().any(|pair| pair.value().as_symbol_node().is_some());
        if !force_rockets && matches!(style.as_str(), "ruby19_no_mixed_keys" | "no_mixed_keys")
            && pairs.iter().all(|pair| !is_rocket(pair))
        {
            return;
        }
        let all_symbols = pairs.iter().all(|pair| convertible_symbol(pair, self.source_file(), self.config_bool("PreferHashRocketsForNonAlnumEndingSymbols", false), self.target_ruby_version()));
        let first_rocket = pairs.first().is_some_and(is_rocket);
        for pair in &pairs {
            let rocket = is_rocket(pair);
            let (bad, message, to_rockets) = if style == "hash_rockets" || force_rockets {
                (!rocket, "Use hash rockets syntax.", true)
            } else if style == "ruby19" {
                (rocket && all_symbols, "Use the new Ruby 1.9 hash syntax.", false)
            } else if style == "ruby19_no_mixed_keys" {
                if all_symbols { (rocket, "Use the new Ruby 1.9 hash syntax.", false) }
                else { (!rocket, "Don't mix styles in the same hash.", true) }
            } else if all_symbols {
                (rocket != first_rocket, "Don't mix styles in the same hash.", first_rocket)
            } else {
                (!rocket, "Don't mix styles in the same hash.", true)
            };
            if bad { self.register_pair(pair, message, to_rockets); }
        }
    }

    fn register_pair(&mut self, pair: &AssocNode<'_>, message: &str, to_rockets: bool) {
        let key = pair.key();
        let source = self.source_file().node(&key);
        let operator = pair.operator_loc();
        let offense_end = operator.as_ref().map_or(key.location().end_offset(), |operator| operator.end_offset());
        let offense = key.location().start_offset()..offense_end;
        if to_rockets {
            let name = source.trim_start_matches(':').trim_end_matches(':');
            let replacement = if is_shorthand(pair) {
                format!(":{name} => {name}")
            } else {
                format!(":{name} => ")
            };
            let edit_end = consume_spaces(self.source(), offense_end);
            let edit = key.location().start_offset()..edit_end;
            add_offense!(self, offense, message: message, |corrector| { corrector.replace(edit, replacement); });
        } else {
            let name = source.trim_matches(':');
            let name = if (name.starts_with('"') && name.ends_with('"')) || (name.starts_with('\'') && name.ends_with('\'')) { name } else { name.trim_matches(['"', '\'']) };
            let prefix = if key.location().start_offset() > 0
                && self.source().as_bytes()[key.location().start_offset() - 1].is_ascii_alphanumeric()
            { " " } else { "" };
            let replacement = format!("{prefix}{name}: ");
            let edit_end = consume_spaces(self.source(), offense_end);
            let edit = key.location().start_offset()..edit_end;
            let return_wrap = self.parent().is_some_and(|parent| parent.as_return_node().is_some())
                && self.source().as_bytes().get(key.location().start_offset().saturating_sub(1)) != Some(&b'{');
            let return_end = self.parent().map(|parent| parent.location().end_offset()).unwrap_or_default();
            let hash_start = key.location().start_offset();
            add_offense!(self, offense, message: message, |corrector| {
                corrector.replace(edit, replacement);
                if return_wrap {
                    corrector.replace(hash_start..hash_start, "{");
                    corrector.replace(return_end..return_end, "}");
                }
            });
        }
    }

    fn check_shorthand(&mut self, pairs: &[AssocNode<'_>]) {
        return_if!(!self.target_ruby_version().at_least(3, 1));
        let in_modifier = self.ancestors().iter().any(modifier_conditional);
        let parenthesized_container = self.ancestors().iter().any(|ancestor| {
            ancestor.as_call_node().is_some_and(|call| call.opening_loc().is_some())
                || ancestor.as_super_node().is_some_and(|call| call.lparen_loc().is_some())
                || ancestor.as_yield_node().is_some_and(|call| call.lparen_loc().is_some())
        });
        return_if!(in_modifier && !parenthesized_container);
        let style = self.config_value("EnforcedShorthandSyntax").unwrap_or("either").to_string();
        return_if!(style == "either");
        let states = pairs.iter().map(|pair| (is_shorthand(pair), can_omit(pair, self.source_file()))).collect::<Vec<_>>();
        let all_omittable = states.iter().all(|(short, can)| *short || *can);
        let any_shorthand = states.iter().any(|(short, _)| *short);
        let any_explicit = states.iter().any(|(short, _)| !*short);
        let first = pairs.first().map(|pair| pair.key().location().start_offset()).unwrap_or_default();
        let last = pairs.last().map(|pair| pair.value().location().end_offset()).unwrap_or_default();
        let command = self.ancestors().iter().rev().find_map(|ancestor| {
            let call = ancestor.as_call_node()?;
            if call.opening_loc().is_some() || matches!(call.name().as_slice(), b"%" | b"[]" | b"[]=") || call.name().as_slice().ends_with(b"=") { return None; }
            let arguments = call.arguments()?.location();
            if arguments.start_offset() > first || last > arguments.end_offset() { return None; }
            Some((call.message_loc()?.end_offset()..call.arguments()?.location().start_offset(), call.location().end_offset()))
        }).or_else(|| self.ancestors().iter().rev().find_map(Node::as_super_node).and_then(|call| {
            if call.lparen_loc().is_some() { return None; }
            let arguments = call.arguments()?.location();
            if arguments.start_offset() > first || last > arguments.end_offset() { return None; }
            Some((call.keyword_loc().end_offset()..arguments.start_offset(), arguments.end_offset()))
        })).or_else(|| self.ancestors().iter().rev().find_map(Node::as_yield_node).and_then(|call| {
            if call.lparen_loc().is_some() { return None; }
            let arguments = call.arguments()?.location();
            if arguments.start_offset() > first || last > arguments.end_offset() { return None; }
            Some((call.keyword_loc().end_offset()..arguments.start_offset(), arguments.end_offset()))
        }));
        let command = command.filter(|(_, close)| {
            if self.ancestors().iter().any(|ancestor| ancestor.as_parentheses_node().is_some()) { return false; }
            self.source()[*close..].lines().skip(1).find(|line| !line.trim().is_empty())
                .is_none_or(|line| !line.trim_start().starts_with("def "))
        });
        let mut command_available = command;
        for (pair, (short, can)) in pairs.iter().zip(states) {
            let (bad, message, omit) = match style.as_str() {
                "always" => (can && !short, "Omit the hash value.", true),
                "never" => (short, "Include the hash value.", false),
                "consistent" if all_omittable => (can && !short, if any_shorthand { "Do not mix explicit and implicit hash values. Omit the hash value." } else { "Omit the hash value." }, true),
                "consistent" if any_shorthand => (short, "Do not mix explicit and implicit hash values. Include the hash value.", false),
                "either_consistent" if any_shorthand && any_explicit && all_omittable => {
                    if pairs.first().is_some_and(is_shorthand) { (can && !short, "Do not mix explicit and implicit hash values. Omit the hash value.", true) }
                    else { (short, "Do not mix explicit and implicit hash values. Include the hash value.", false) }
                }
                "either_consistent" if any_shorthand && any_explicit => (short, "Do not mix explicit and implicit hash values. Include the hash value.", false),
                _ => (false, "", false),
            };
            if !bad { continue; }
            if omit {
                let value = pair.value();
                let offense = value.location();
                let start = pair.operator_loc().map_or(pair.key().location().end_offset(), |operator| operator.end_offset());
                let command = command_available.take();
                add_offense!(self, offense, message: message, |corrector| {
                    corrector.replace(start..value.location().end_offset(), "");
                    if let Some((open, close)) = command {
                        corrector.replace(open, "(");
                        corrector.replace(close..close, ")");
                    }
                });
            } else {
                let key = pair.key();
                let source = self.source_file().node(&key);
                let name = source.trim_end_matches(':');
                let offense = key.location().start_offset()..key.location().end_offset().saturating_sub(1);
                add_offense!(self, offense, message: message, |corrector| { corrector.replace(key.location(), format!("{source} {name}")); });
            }
        }
    }
}

fn is_rocket(pair: &AssocNode<'_>) -> bool { pair.operator_loc().is_some_and(|operator| operator.as_slice() == b"=>") }

fn convertible_symbol(pair: &AssocNode<'_>, file: SourceFile<'_>, prefer_rockets: bool, version: crate::config::RubyVersion) -> bool {
    let key = pair.key();
    if key.as_interpolated_symbol_node().is_some() {
        return version.at_least(2, 2);
    }
    let Some(symbol) = key.as_symbol_node() else { return false };
    let source = file.node(&symbol.as_node()).trim_start_matches(':').trim_end_matches(':');
    if prefer_rockets && source.ends_with(['?', '!']) { return false; }
    let quoted = (source.starts_with('"') && source.ends_with('"')) || (source.starts_with('\'') && source.ends_with('\''));
    if quoted { return version.at_least(2, 2); }
    let mut chars = source.chars();
    chars.next().is_some_and(|character| character == '_' || character.is_ascii_alphabetic())
        && chars.all(|character| character == '_' || character.is_ascii_alphanumeric() || matches!(character, '?' | '!'))
}

fn is_shorthand(pair: &AssocNode<'_>) -> bool { pair.value().as_implicit_node().is_some() }

fn can_omit(pair: &AssocNode<'_>, file: SourceFile<'_>) -> bool {
    if is_shorthand(pair) || is_rocket(pair) { return false; }
    let key = file.node(&pair.key()).trim_end_matches(':').trim_start_matches(':');
    let value = file.node(&pair.value());
    key == value && !key.ends_with(['?', '!'])
        && (pair.value().as_local_variable_read_node().is_some()
            || pair.value().as_call_node().is_some_and(|call| call.receiver().is_none() && argument_count(&call) == 0))
}

fn consume_spaces(source: &str, mut offset: usize) -> usize {
    while source.as_bytes().get(offset).is_some_and(u8::is_ascii_whitespace) && source.as_bytes().get(offset) != Some(&b'\n') { offset += 1; }
    offset
}

fn modifier_conditional(node: &Node<'_>) -> bool {
    node.as_if_node().is_some_and(|conditional| conditional.end_keyword_loc().is_none() && conditional.then_keyword_loc().is_none())
        || node.as_unless_node().is_some_and(|conditional| conditional.end_keyword_loc().is_none())
        || node.as_while_node().is_some_and(|conditional| conditional.closing_loc().is_none())
        || node.as_until_node().is_some_and(|conditional| conditional.closing_loc().is_none())
}
