use ruby_prism::{BlockNode, DefNode, IfNode, Node, StatementsNode, UnlessNode};

use super::*;

define_cops! {
    GuardClause => "Style/GuardClause" => recovery_rubocop_callbacks(
        GuardClauseRule,
        [on_def, on_block, on_if, on_unless]
    ),
}

const MESSAGE: &str = "Use a guard clause (`{example}`) instead of wrapping the code inside a conditional expression.";

impl GuardClauseRule<'_, '_, '_> {
    fn on_def(&mut self, node: &DefNode<'_>) {
        let Some(body) = node.body().and_then(|body| body.as_statements_node()) else { return };
        self.check_ending_body(body);
    }

    fn on_block(&mut self, node: &BlockNode<'_>) {
        let define_method = self.ancestors().iter().rev().find_map(Node::as_call_node)
            .is_some_and(|call| matches!(call.name().as_slice(), b"define_method" | b"define_singleton_method"));
        return_unless!(define_method);
        let Some(body) = node.body().and_then(|body| body.as_statements_node()) else { return };
        self.check_ending_body(body);
    }

    fn on_if(&mut self, node: &IfNode<'_>) {
        let Some(keyword) = node.if_keyword_loc() else { return };
        return_if!(keyword.as_slice() == b"elsif" || node.end_keyword_loc().is_none());
        let Some(branch) = node.subsequent().and_then(|branch| branch.as_else_node()) else { return };
        self.check_guard(
            node.location(), keyword, node.end_keyword_loc().expect("normal conditional"), node.predicate(), "if", "unless",
            node.statements(), branch.statements(), branch.else_keyword_loc(),
        );
    }

    fn on_unless(&mut self, node: &UnlessNode<'_>) {
        let Some(branch) = node.else_clause() else { return };
        let Some(end_keyword) = node.end_keyword_loc() else { return };
        self.check_guard(
            node.location(), node.keyword_loc(), end_keyword, node.predicate(), "unless", "if",
            node.statements(), branch.statements(), branch.else_keyword_loc(),
        );
    }

    fn check_ending_body(&mut self, body: StatementsNode<'_>) {
        let Some(final_expression) = body.body().iter().last() else { return };
        if let Some(conditional) = final_expression.as_if_node() {
            self.check_ending_if(&conditional);
        } else if let Some(conditional) = final_expression.as_unless_node() {
            self.check_ending_unless(&conditional);
        }
    }

    fn check_ending_if(&mut self, node: &IfNode<'_>) {
        let Some(keyword) = node.if_keyword_loc() else { return };
        return_if!(keyword.as_slice() == b"elsif" || node.end_keyword_loc().is_none() || node.subsequent().is_some());
        let end_keyword = node.end_keyword_loc().expect("normal conditional");
        let condition = node.predicate();
        let statements = node.statements();
        return_if!(self.ending_accepted(&keyword, &end_keyword, &condition, statements.as_ref(), "unless"));
        self.register_ending(keyword, end_keyword, condition, statements, "unless");
        if let Some(body) = node.statements() { self.check_ending_body(body); }
    }

    fn check_ending_unless(&mut self, node: &UnlessNode<'_>) {
        return_if!(node.end_keyword_loc().is_none() || node.else_clause().is_some());
        let keyword = node.keyword_loc();
        let end_keyword = node.end_keyword_loc().expect("normal conditional");
        let condition = node.predicate();
        let statements = node.statements();
        return_if!(self.ending_accepted(&keyword, &end_keyword, &condition, statements.as_ref(), "if"));
        self.register_ending(keyword, end_keyword, condition, statements, "if");
        if let Some(body) = node.statements() { self.check_ending_body(body); }
    }

    fn ending_accepted(
        &self,
        keyword: &ruby_prism::Location<'_>,
        end_keyword: &ruby_prism::Location<'_>,
        condition: &Node<'_>,
        statements: Option<&StatementsNode<'_>>,
        inverse_keyword: &str,
    ) -> bool {
        let condition_source = self.source_file().node(condition);
        if condition_source.contains('\n')
            || assigned_local_used(condition, statements)
        {
            return true;
        }
        let body_lines = branch_line_count(
            statements,
            keyword.end_offset(),
            end_keyword.start_offset(),
            self.source(),
        );
        let minimum = self
            .config_value("MinBodyLength")
            .and_then(|value| value.parse::<isize>().ok())
            .unwrap_or(1);
        if minimum < 0
            || body_lines < minimum as usize
            || self.config_bool("AllowConsecutiveConditionals", false)
                && preceding_conditional(self.source(), keyword.start_offset())
        {
            return true;
        }
        let example = format!("return {inverse_keyword} {condition_source}");
        let max = (self.related_config_value("Layout/LineLength", "Enabled") != Some("false"))
            .then(|| {
                self.related_config_value("Layout/LineLength", "Max")
                    .and_then(|value| value.parse().ok())
            })
            .flatten();
        max.is_some_and(|max: usize| {
            self.source_file().column(keyword.start_offset()) + example.chars().count() > max
        }) && branch_trivial(statements)
    }

    fn register_ending(
        &mut self,
        keyword: ruby_prism::Location<'_>,
        end_keyword: ruby_prism::Location<'_>,
        condition: Node<'_>,
        statements: Option<StatementsNode<'_>>,
        inverse_keyword: &str,
    ) {
        let condition_source = self.source_file().node(&condition);
        return_if!(condition_source.contains('\n')
            || assigned_local_used(&condition, statements.as_ref()));
        let body_lines = branch_line_count(statements.as_ref(), keyword.end_offset(), end_keyword.start_offset(), self.source());
        let minimum = self.config_value("MinBodyLength").and_then(|value| value.parse::<isize>().ok()).unwrap_or(1);
        return_if!(minimum < 0 || body_lines < minimum as usize);
        return_if!(self.config_bool("AllowConsecutiveConditionals", false) && preceding_conditional(self.source(), keyword.start_offset()));

        let example = format!("return {inverse_keyword} {condition_source}");
        let max = (self.related_config_value("Layout/LineLength", "Enabled") != Some("false"))
            .then(|| self.related_config_value("Layout/LineLength", "Max").and_then(|value| value.parse().ok())).flatten();
        let too_long = max.is_some_and(|max: usize| self.source_file().column(keyword.start_offset()) + example.chars().count() > max);
        return_if!(too_long && branch_trivial(statements.as_ref()));
        let replacement = if too_long {
            format!("{inverse_keyword} {condition_source}\n  return\nend")
        } else { example.clone() };
        let shown = if too_long { format!("{inverse_keyword} {condition_source}; return; end") } else { example };
        let message = MESSAGE.replace("{example}", &shown);
        let header = keyword.start_offset()..condition.location().end_offset();
        let heredoc = statements.as_ref().is_some_and(|statements| {
            let location = statements.location();
            let body = location.start_offset()..location.end_offset();
            self.source_file()
                .heredoc_ranges()
                .iter()
                .any(|range| range.start < body.end && body.start < range.end)
        });
        let (end_edit, end_replacement) = if heredoc {
            let start = self.source_file().line_start(end_keyword.start_offset());
            let mut end = self.source_file().line_end(end_keyword.end_offset());
            if self.source().as_bytes().get(end) == Some(&b'\n') { end += 1; }
            (start..end, String::new())
        } else {
            let raw = end_keyword.start_offset()..end_keyword.end_offset();
            let offset = self
                .source_file()
                .slice(raw.clone())
                .and_then(|source| source.find("end"))
                .unwrap_or(0);
            (raw.start + offset..raw.start + offset + 3, String::new())
        };
        add_offense!(self, keyword, message: message, |corrector| {
            corrector.replace(header, replacement);
            corrector.replace(end_edit, end_replacement);
        });
    }

    #[allow(clippy::too_many_arguments)]
    fn check_guard(
        &mut self,
        location: ruby_prism::Location<'_>,
        keyword: ruby_prism::Location<'_>,
        end_keyword: ruby_prism::Location<'_>,
        condition: Node<'_>,
        conditional_keyword: &str,
        inverse_keyword: &str,
        if_statements: Option<StatementsNode<'_>>,
        else_statements: Option<StatementsNode<'_>>,
        else_keyword: ruby_prism::Location<'_>,
    ) {
        let condition_source = self.source_file().node(&condition);
        return_if!(condition_source.contains('\n') || assignment_parent(self.ancestors()));
        return_if!(assigned_local_used(&condition, if_statements.as_ref()));
        let if_guard = guard_clause(if_statements.as_ref(), self.source_file())
            .filter(|guard| guard.logical || !guard.source.contains('\n'));
        let else_guard = guard_clause(else_statements.as_ref(), self.source_file())
            .filter(|guard| guard.logical || !guard.source.contains('\n'));
        let (guard, guard_keyword, branch_range, keep_statements) = if let Some(guard) = if_guard {
            (guard, conditional_keyword, if_statements.as_ref().map(|statements| statements.location()), else_statements.as_ref())
        } else if let Some(guard) = else_guard {
            (guard, inverse_keyword, else_statements.as_ref().map(|statements| statements.location()), if_statements.as_ref())
        } else { return };
        let example = format!("{} {guard_keyword} {condition_source}", guard.source);
        let max = (self.related_config_value("Layout/LineLength", "Enabled") != Some("false"))
            .then(|| self.related_config_value("Layout/LineLength", "Max").and_then(|value| value.parse().ok())).flatten();
        let too_long = max.is_some_and(|max: usize| self.source_file().column(keyword.start_offset()) + example.chars().count() > max);
        let shown = if too_long { format!("{guard_keyword} {condition_source}; {}; end", guard.source) } else { example.clone() };
        let replacement = if too_long { format!("{guard_keyword} {condition_source}\n  {}\nend", guard.source) } else { example };
        let message = MESSAGE.replace("{example}", &shown);
        if guard.logical {
            self.report(message, keyword);
            return;
        }
        if guard.source.contains("<<") {
            let source = self.source_file().at(&location);
            if let Some(replacement) = heredoc_replacement(source, &replacement, keep_statements, self.source_file()) {
                add_offense!(self, keyword, message: message, |corrector| { corrector.replace(location, replacement); });
                return;
            }
        }
        if !source_has_newline(self.source_file().at(&location)) {
            let keep = keep_statements.map_or("", |statements| self.source_file().at(&statements.location()));
            let replacement = format!("{replacement} \n {keep}   ");
            add_offense!(self, keyword, message: message, |corrector| { corrector.replace(location, replacement); });
            return;
        }
        let header = keyword.start_offset()..condition.location().end_offset();
        add_offense!(self, keyword, message: message, |corrector| {
            corrector.replace(header, replacement);
            corrector.remove(end_keyword);
            corrector.remove(else_keyword);
            if let Some(branch_range) = branch_range { corrector.remove(branch_range); }
        });
    }
}

struct GuardClauseMatch { source: String, logical: bool }

fn guard_clause(statements: Option<&StatementsNode<'_>>, file: SourceFile<'_>) -> Option<GuardClauseMatch> {
    let statements = statements?;
    let node = (statements.body().len() == 1).then(|| statements.body().first()).flatten()?;
    if scope_exit(&node) {
        return Some(GuardClauseMatch { source: file.node(&node).to_owned(), logical: false });
    }
    if node.as_and_node().is_some() || node.as_or_node().is_some() {
        let source = file.node(&node);
        if contains_scope_exit(&node) {
            return Some(GuardClauseMatch { source: source.to_owned(), logical: true });
        }
    }
    None
}

fn scope_exit(node: &Node<'_>) -> bool {
    node.as_return_node().is_some()
        || node.as_break_node().is_some()
        || node.as_next_node().is_some()
        || node.as_call_node().is_some_and(|call| {
            call.receiver().is_none() && matches!(call.name().as_slice(), b"raise" | b"fail")
        })
}

fn contains_scope_exit(node: &Node<'_>) -> bool {
    struct ScopeExitFinder(bool);
    impl<'pr> Visit<'pr> for ScopeExitFinder {
        fn visit_return_node(&mut self, node: &ruby_prism::ReturnNode<'pr>) {
            self.0 = true;
            ruby_prism::visit_return_node(self, node);
        }
        fn visit_break_node(&mut self, node: &ruby_prism::BreakNode<'pr>) {
            self.0 = true;
            ruby_prism::visit_break_node(self, node);
        }
        fn visit_next_node(&mut self, node: &ruby_prism::NextNode<'pr>) {
            self.0 = true;
            ruby_prism::visit_next_node(self, node);
        }
        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            if node.receiver().is_none()
                && matches!(node.name().as_slice(), b"raise" | b"fail")
            {
                self.0 = true;
            }
            ruby_prism::visit_call_node(self, node);
        }
    }
    let mut finder = ScopeExitFinder(false);
    finder.visit(node);
    finder.0
}

fn branch_line_count(statements: Option<&StatementsNode<'_>>, start: usize, end: usize, source: &str) -> usize {
    statements.map_or_else(
        || source[start..end].lines().filter(|line| !line.trim().is_empty()).count(),
        |statements| statements.body().len(),
    )
}

fn branch_trivial(statements: Option<&StatementsNode<'_>>) -> bool {
    let Some(statements) = statements else { return true };
    if statements.body().len() != 1 { return false }
    statements.body().first().is_some_and(|node| {
        // Prism's explicit BeginNode maps to Parser's `kwbegin`, not its
        // implicit `begin` container. RuboCop treats `kwbegin` as trivial.
        node.as_if_node().is_none() && node.as_unless_node().is_none()
    })
}

fn source_has_newline(source: &str) -> bool { source.contains('\n') }

fn heredoc_replacement(
    conditional_source: &str,
    header: &str,
    keep: Option<&StatementsNode<'_>>,
    file: SourceFile<'_>,
) -> Option<String> {
    let marker = header.split("<<~").nth(1)?;
    let label = marker.trim_start_matches('`').split(|character: char| !character.is_ascii_alphanumeric() && character != '_').next()?;
    let introducer = conditional_source.find("<<~")?;
    let first_line_end = conditional_source[introducer..].find('\n').map(|at| introducer + at)?;
    let mut terminator_end = None;
    let mut cursor = first_line_end + 1;
    for line in conditional_source[cursor..].split_inclusive('\n') {
        if line.trim() == label {
            terminator_end = Some(cursor + line.trim_end_matches('\n').len());
            break;
        }
        cursor += line.len();
    }
    let tail = &conditional_source[first_line_end..terminator_end?];
    let keep = keep.map_or("", |statements| file.at(&statements.location()));
    Some(if keep.is_empty() { format!("{header}{tail}") } else { format!("{header}{tail}\n{keep}") })
}

fn assigned_local_used(condition: &Node<'_>, statements: Option<&StatementsNode<'_>>) -> bool {
    if condition.as_local_variable_write_node().is_some() {
        return false;
    }
    #[derive(Default)]
    struct LocalNames {
        assigned: Vec<Vec<u8>>,
        read: Vec<Vec<u8>>,
    }
    impl<'pr> Visit<'pr> for LocalNames {
        fn visit_local_variable_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableWriteNode<'pr>,
        ) {
            self.assigned.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_write_node(self, node);
        }
        fn visit_local_variable_or_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
        ) {
            self.assigned.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_or_write_node(self, node);
        }
        fn visit_local_variable_and_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
        ) {
            self.assigned.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_and_write_node(self, node);
        }
        fn visit_local_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
        ) {
            self.assigned.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_operator_write_node(self, node);
        }
        fn visit_local_variable_read_node(
            &mut self,
            node: &ruby_prism::LocalVariableReadNode<'pr>,
        ) {
            self.read.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_read_node(self, node);
        }
        fn visit_multi_write_node(&mut self, node: &ruby_prism::MultiWriteNode<'pr>) {
            for target in node
                .lefts()
                .iter()
                .chain(node.rest())
                .chain(node.rights().iter())
            {
                if let Some(target) = target.as_local_variable_target_node() {
                    self.assigned.push(target.name().as_slice().to_vec());
                }
            }
            ruby_prism::visit_multi_write_node(self, node);
        }
    }
    let mut names = LocalNames::default();
    names.visit(condition);
    let Some(statements) = statements else { return false };
    let mut branch = LocalNames::default();
    branch.visit(&statements.as_node());
    if statements.body().len() == 1 {
        if let Some(read) = statements
            .body()
            .first()
            .and_then(|node| node.as_local_variable_read_node())
        {
            branch.read.retain(|name| name.as_slice() != read.name().as_slice());
        }
    }
    names
        .assigned
        .iter()
        .any(|assigned| branch.read.contains(assigned))
}

fn assignment_parent(ancestors: &[Node<'_>]) -> bool {
    ancestors
        .iter()
        .rev()
        .find(|node| node.as_statements_node().is_none())
        .is_some_and(|node| {
        node.as_local_variable_write_node().is_some()
            || node.as_instance_variable_write_node().is_some()
            || node.as_class_variable_write_node().is_some()
            || node.as_global_variable_write_node().is_some()
            || node.as_constant_write_node().is_some()
            || node.as_constant_path_write_node().is_some()
        })
}

fn preceding_conditional(source: &str, offset: usize) -> bool {
    source[..offset].lines().rev().find(|line| !line.trim().is_empty()).is_some_and(|line| line.trim() == "end")
}
