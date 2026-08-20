use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![Box::new(Semicolon), Box::new(UnlessElse)]
}

struct Semicolon;

impl Cop for Semicolon {
    fn name(&self) -> &'static str {
        "Style/Semicolon"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        if node.as_program_node().is_none() {
            return;
        }
        let allow_separators = {
            let cop_context = context.cop_context(self.name(), source, _ancestors);
            cop_context.config_bool("AllowAsExpressionSeparator", false)
        };
        for offset in semicolon_offsets(node, source, allow_separators) {
            let line_start = source[..offset].rfind('\n').map_or(0, |at| at + 1);
            let line_end = source[offset..]
                .find('\n')
                .map_or(source.len(), |at| offset + at);
            let prefix = &source[line_start..offset];
            if allow_separators && !source[offset + 1..line_end].trim().is_empty() {
                continue;
            }
            let line_source = &source[line_start..line_end];
            let replacement = if prefix.trim().is_empty()
                || prefix.trim_end().ends_with('{')
                || source[offset + 1..line_end].trim().is_empty()
                || source[offset + 1..line_end].trim_start().starts_with('}')
            {
                ""
            } else {
                "\n"
            };
            if let Some(range) = endless_range_ending_at(node, offset) {
                let expression = &source[range.clone()];
                context.replace(
                    self.name(),
                    "Do not use semicolons to terminate expressions.",
                    (offset, offset + 1),
                    (range.start, offset + 1),
                    format!("({expression})"),
                );
                continue;
            }
            let trimmed_prefix = prefix.trim();
            if !trimmed_prefix.contains('(')
                && trimmed_prefix
                    .split_whitespace()
                    .skip(1)
                    .flat_map(|arguments| arguments.split(','))
                    .map(str::trim)
                    .filter(|argument| !argument.is_empty())
                    .all(|argument| argument.ends_with(':'))
                && trimmed_prefix.split_whitespace().count() > 1
            {
                let (method, arguments) = trimmed_prefix.split_once(' ').expect("count checked");
                let indent = &source
                    [line_start..line_start + line_source.len() - line_source.trim_start().len()];
                context.replace(
                    self.name(),
                    "Do not use semicolons to terminate expressions.",
                    (offset, offset + 1),
                    (line_start, offset + 1),
                    format!("{indent}{method}({arguments})"),
                );
                continue;
            }
            context.replace(
                self.name(),
                "Do not use semicolons to terminate expressions.",
                (offset, offset + 1),
                (offset, offset + 1),
                replacement,
            );
        }
    }
}

fn line_start(source: &str, offset: usize) -> usize {
    SourceFile::new(source).line_start(offset)
}

fn endless_range_ending_at(root: &Node<'_>, offset: usize) -> Option<std::ops::Range<usize>> {
    struct Finder {
        offset: usize,
        found: Option<std::ops::Range<usize>>,
    }

    impl<'pr> Visit<'pr> for Finder {
        fn visit_range_node(&mut self, node: &ruby_prism::RangeNode<'pr>) {
            let location = node.location();
            if node.right().is_none() && location.end_offset() == self.offset {
                self.found = Some(location.start_offset()..location.end_offset());
            }
            ruby_prism::visit_range_node(self, node);
        }
    }

    let mut finder = Finder {
        offset,
        found: None,
    };
    finder.visit(root);
    finder.found
}

fn semicolon_offsets(root: &Node<'_>, source: &str, allow_separators: bool) -> Vec<usize> {
    #[derive(Default)]
    struct LiteralContent {
        ranges: Vec<std::ops::Range<usize>>,
    }

    impl LiteralContent {
        fn push(&mut self, location: ruby_prism::Location<'_>) {
            self.ranges
                .push(location.start_offset()..location.end_offset());
        }
    }

    impl<'pr> Visit<'pr> for LiteralContent {
        fn visit_string_node(&mut self, node: &ruby_prism::StringNode<'pr>) {
            self.push(node.content_loc());
        }

        fn visit_symbol_node(&mut self, node: &ruby_prism::SymbolNode<'pr>) {
            if let Some(value) = node.value_loc() {
                self.push(value);
            }
        }

        fn visit_regular_expression_node(
            &mut self,
            node: &ruby_prism::RegularExpressionNode<'pr>,
        ) {
            self.push(node.content_loc());
        }

        fn visit_match_last_line_node(&mut self, node: &ruby_prism::MatchLastLineNode<'pr>) {
            self.push(node.content_loc());
        }

        fn visit_x_string_node(&mut self, node: &ruby_prism::XStringNode<'pr>) {
            self.push(node.content_loc());
        }
    }

    let mut literals = LiteralContent::default();
    literals.visit(root);
    literals.ranges.sort_by_key(|range| range.start);

    struct MultiExpressionLines<'src> {
        source: &'src str,
        starts: std::collections::HashSet<usize>,
    }

    impl<'pr> Visit<'pr> for MultiExpressionLines<'_> {
        fn visit_statements_node(&mut self, node: &ruby_prism::StatementsNode<'pr>) {
            let mut seen = std::collections::HashSet::new();
            for statement in node.body().iter() {
                let location = statement.location();
                let final_offset = location.end_offset().saturating_sub(1);
                let start = line_start(self.source, final_offset);
                if !seen.insert(start) {
                    self.starts.insert(start);
                }
            }
            ruby_prism::visit_statements_node(self, node);
        }
    }

    let mut multi_expression_lines = MultiExpressionLines {
        source,
        starts: std::collections::HashSet::new(),
    };
    if !allow_separators {
        multi_expression_lines.visit(root);
    }

    let bytes = source.as_bytes();
    let mut offsets = Vec::new();
    let mut index = 0;
    let mut comment = false;
    let mut literal_index = 0;

    while index < bytes.len() {
        while literals
            .ranges
            .get(literal_index)
            .is_some_and(|range| range.end <= index)
        {
            literal_index += 1;
        }
        if let Some(range) = literals.ranges.get(literal_index) {
            if range.start <= index && index < range.end {
                index = range.end;
                continue;
            }
        }
        let byte = bytes[index];
        if comment {
            if byte == b'\n' {
                comment = false;
            }
            index += 1;
            continue;
        }
        match byte {
            b'#' if bytes.get(index + 1) != Some(&b'{') => comment = true,
            b';' => {
                let start = line_start(source, index);
                let end = source[index..]
                    .find('\n')
                    .map_or(source.len(), |at| index + at);
                let prefix = &source[start..index];
                let suffix = &source[index + 1..end];
                let trailing = suffix.trim().is_empty() || suffix.trim_start().starts_with('#');
                let leading = prefix.trim().is_empty();
                let before_closing_brace = suffix.trim_start().starts_with('}')
                    && suffix
                        .trim_start()
                        .strip_prefix('}')
                        .is_some_and(|rest| {
                            rest.trim().is_empty()
                                || rest.trim_start().starts_with('#')
                                || prefix.contains("#{")
                        });
                let opener = prefix.trim_end().strip_suffix('{').map(str::trim_end);
                let after_opening_brace = opener.is_some_and(|before| {
                    (!before.is_empty()
                        && before
                            .bytes()
                            .all(|byte| byte == b'_' || byte.is_ascii_alphanumeric()))
                        || before.ends_with("->")
                        || before.ends_with('#')
                });
                if trailing || leading || before_closing_brace || after_opening_brace {
                    offsets.push(index);
                }
            }
            _ => {}
        }
        index += 1;
    }
    for start in multi_expression_lines.starts {
        let end = source[start..]
            .find('\n')
            .map_or(source.len(), |at| start + at);
        offsets.extend(
            source[start..end]
                .match_indices(';')
                .map(|(relative, _)| start + relative),
        );
    }
    offsets.sort_unstable();
    offsets.dedup();
    offsets
}

define_node_cop!(UnlessElse => "Style/UnlessElse" => as_unless_node => unless_else);

fn unless_else(node: &ruby_prism::UnlessNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(else_clause) = node.else_clause() else {
        return;
    };
    let location = node.location();
    let message = "Do not use `unless` with `else`. Rewrite these with the positive case first.";
    if context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_unless_node().is_some())
    {
        if !context.autocorrect_enabled() {
            context.report(message, &location);
        }
        return;
    }
    let Some(end_keyword) = node.end_keyword_loc() else {
        context.report(message, &location);
        return;
    };
    let body_start = node.then_keyword_loc().map_or_else(
        || node.predicate().location().end_offset(),
        |keyword| keyword.end_offset(),
    );
    let body = body_start..else_clause.else_keyword_loc().start_offset();
    let alternative = else_clause.else_keyword_loc().end_offset()..end_keyword.start_offset();
    let mut correction = CorrectionPlan::default();
    correction.replace(
        node.keyword_loc().start_offset()..node.keyword_loc().end_offset(),
        "if",
    );
    if correction.swap(context.source(), body, alternative) {
        context.apply_correction(message, &location, correction);
    } else {
        context.report(message, &location);
    }
}
