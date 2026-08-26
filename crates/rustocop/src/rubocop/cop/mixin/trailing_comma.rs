// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/trailing_comma.rb
// Source SHA-256: 6da90c46665b3108a509cbc604777f6c4abec47978852cb82fab05e345a3a6d8

use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum TrailingCommaStyle {
    NoComma,
    Comma,
    ConsistentComma,
    DiffComma,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Location {
    pub(crate) bytes: Range<usize>,
    pub(crate) line: usize,
    pub(crate) last_line: usize,
    pub(crate) source: String,
    pub(crate) begins_its_line: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Item {
    pub(crate) kind: String,
    pub(crate) source_range: Location,
    pub(crate) children: Vec<Item>,
    pub(crate) arguments: Vec<Item>,
    pub(crate) call_type: bool,
    pub(crate) multiline: bool,
    pub(crate) braces: bool,
    pub(crate) block_pass: bool,
    pub(crate) heredoc_body: bool,
    pub(crate) end_location: Option<Location>,
    pub(crate) selector_line: Option<usize>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CommaAction {
    Add,
    Remove,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommaOffense {
    pub(crate) range: Range<usize>,
    pub(crate) message: String,
    pub(crate) action: CommaAction,
}

pub(crate) struct TrailingComma {
    pub(crate) style: TrailingCommaStyle,
}

impl TrailingComma {
    pub(crate) fn style_parameter_name(&self) -> &'static str {
        "EnforcedStyleForMultiline"
    }

    pub(crate) fn check(
        &self,
        node: &Item,
        items: &[Item],
        kind: &str,
        after_last_item: &Location,
        comment_begin: Option<usize>,
    ) -> Option<CommaOffense> {
        let comma_offset = self.comma_offset(items, after_last_item);
        if let Some(offset) = comma_offset
            .filter(|offset| !self.inside_comment(after_last_item, *offset, comment_begin))
        {
            self.check_comma(node, kind, after_last_item.bytes.start + offset)
        } else if self.should_have_comma(self.style, node) {
            self.put_comma(items, kind)
        } else {
            None
        }
    }

    pub(crate) fn comma_offset(&self, items: &[Item], range: &Location) -> Option<usize> {
        let allow_newline = !self.any_heredoc(items);
        let prefix = range
            .source
            .char_indices()
            .take_while(|(_, character)| {
                character.is_whitespace() && (allow_newline || *character != '\n')
            })
            .last()
            .map_or(0, |(index, character)| index + character.len_utf8());
        (range.source[prefix..].starts_with(',')).then_some(prefix)
    }

    pub(crate) fn check_comma(
        &self,
        node: &Item,
        kind: &str,
        comma_pos: usize,
    ) -> Option<CommaOffense> {
        (!self.should_have_comma(self.style, node))
            .then(|| self.avoid_comma(kind, comma_pos, self.extra_avoid_comma_info()))
    }

    pub(crate) fn check_literal(
        &self,
        node: &Item,
        kind: &str,
        comment_begin: Option<usize>,
    ) -> Option<CommaOffense> {
        if node.children.is_empty() || !self.brackets(node) {
            return None;
        }
        let last = node.children.last().expect("checked nonempty");
        let end = node
            .end_location
            .as_ref()
            .expect("bracketed node has end location");
        let between = Location {
            bytes: last.source_range.bytes.end..end.bytes.start,
            line: last.source_range.last_line,
            last_line: end.line,
            source: end.source.clone(),
            begins_its_line: false,
        };
        self.check(node, &node.children, kind, &between, comment_begin)
    }

    pub(crate) fn extra_avoid_comma_info(&self) -> &'static str {
        match self.style {
            TrailingCommaStyle::Comma => ", unless each item is on its own line",
            TrailingCommaStyle::ConsistentComma => ", unless items are split onto multiple lines",
            TrailingCommaStyle::DiffComma => ", unless that item immediately precedes a newline",
            TrailingCommaStyle::NoComma => "",
        }
    }

    pub(crate) fn should_have_comma(&self, style: TrailingCommaStyle, node: &Item) -> bool {
        match style {
            TrailingCommaStyle::Comma => {
                self.multiline(node) && self.no_elements_on_same_line(node)
            }
            TrailingCommaStyle::ConsistentComma => {
                self.multiline(node) && !self.method_name_and_arguments_on_same_line(node)
            }
            TrailingCommaStyle::DiffComma => {
                self.multiline(node) && self.last_item_precedes_newline(node)
            }
            TrailingCommaStyle::NoComma => false,
        }
    }

    pub(crate) fn inside_comment(
        &self,
        range: &Location,
        comma_offset: usize,
        comment_begin: Option<usize>,
    ) -> bool {
        comment_begin.is_some_and(|begin| begin < range.bytes.start + comma_offset)
    }

    pub(crate) fn brackets(&self, node: &Item) -> bool {
        node.end_location.is_some()
    }

    pub(crate) fn multiline(&self, node: &Item) -> bool {
        node.multiline && !self.allowed_multiline_argument(node)
    }

    pub(crate) fn method_name_and_arguments_on_same_line(&self, node: &Item) -> bool {
        let Some(last) = node.arguments.last().filter(|_| node.call_type) else {
            return false;
        };
        if node.source_range.last_line != last.source_range.last_line {
            return false;
        }
        if last.kind == "hash" && last.braces {
            return true;
        }
        node.selector_line.unwrap_or(node.source_range.line) == last.source_range.last_line
    }

    pub(crate) fn allowed_multiline_argument(&self, node: &Item) -> bool {
        let elements = self.elements(node);
        elements.len() == 1 && !self.node_end_location(node).begins_its_line
    }

    pub(crate) fn elements<'node>(&self, node: &'node Item) -> Vec<&'node Item> {
        if !node.call_type {
            return node.children.iter().collect();
        }
        node.arguments
            .iter()
            .flat_map(|argument| {
                if argument.kind == "hash" && argument.multiline && !argument.braces {
                    argument.children.iter().collect::<Vec<_>>()
                } else {
                    vec![argument]
                }
            })
            .collect()
    }

    pub(crate) fn no_elements_on_same_line(&self, node: &Item) -> bool {
        let mut locations: Vec<_> = self
            .elements(node)
            .into_iter()
            .map(|item| &item.source_range)
            .collect();
        locations.push(self.node_end_location(node));
        locations
            .windows(2)
            .all(|pair| !self.on_same_line(pair[0], pair[1]))
    }

    pub(crate) fn node_end_location<'node>(&self, node: &'node Item) -> &'node Location {
        node.end_location.as_ref().unwrap_or(&node.source_range)
    }

    pub(crate) fn on_same_line(&self, first: &Location, second: &Location) -> bool {
        first.last_line == second.line
    }

    pub(crate) fn last_item_precedes_newline(&self, node: &Item) -> bool {
        let Some(last) = node.children.last() else {
            return false;
        };
        let offset = last
            .source_range
            .bytes
            .end
            .saturating_sub(node.source_range.bytes.start);
        let source = node.source_range.source.get(offset..).unwrap_or_default();
        regex::Regex::new(r"^,?\s*(#.*)?\n")
            .expect("static regex")
            .is_match(source)
    }

    pub(crate) fn avoid_comma(
        &self,
        kind: &str,
        comma_begin_pos: usize,
        extra_info: &str,
    ) -> CommaOffense {
        let article = if kind.contains("array") { "an" } else { "a" };
        let unit = kind.replace("%<article>s", article);
        CommaOffense {
            range: comma_begin_pos..comma_begin_pos + 1,
            message: format!("Avoid comma after the last {unit}{extra_info}."),
            action: CommaAction::Remove,
        }
    }

    pub(crate) fn put_comma(&self, items: &[Item], kind: &str) -> Option<CommaOffense> {
        let last_item = items.last()?;
        if last_item.block_pass {
            return None;
        }
        Some(CommaOffense {
            range: self.autocorrect_range(last_item),
            message: format!(
                "Put a comma after the last {}.",
                kind.replace("%<article>s", "a multiline")
            ),
            action: CommaAction::Add,
        })
    }

    pub(crate) fn autocorrect_range(&self, item: &Item) -> Range<usize> {
        let source = &item.source_range.source;
        let newline = source.rfind('\n').unwrap_or(0);
        let nonspace = source[newline..]
            .find(|character: char| !character.is_whitespace())
            .unwrap_or(0);
        item.source_range.bytes.start + newline + nonspace..item.source_range.bytes.end
    }

    pub(crate) fn any_heredoc(&self, items: &[Item]) -> bool {
        items.iter().any(|item| self.heredoc(item))
    }

    pub(crate) fn heredoc(&self, node: &Item) -> bool {
        if node.heredoc_body {
            return true;
        }
        if node.call_type {
            return self.heredoc_send(node);
        }
        matches!(node.kind.as_str(), "pair" | "hash")
            && node
                .children
                .last()
                .is_some_and(|child| self.heredoc(child))
    }

    pub(crate) fn heredoc_send(&self, node: &Item) -> bool {
        if node.children.len() == 2 {
            node.children
                .first()
                .is_some_and(|child| self.heredoc(child))
        } else if node.children.len() > 2 {
            node.children
                .last()
                .is_some_and(|child| self.heredoc(child))
        } else {
            false
        }
    }
}
