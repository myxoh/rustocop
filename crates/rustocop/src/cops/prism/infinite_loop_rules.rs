use ruby_prism::{Location, Node, UntilNode, WhileNode};

use super::*;

define_cops! {
    InfiniteLoop => "Style/InfiniteLoop" => rubocop_callbacks(InfiniteLoopRule, [on_while, on_until]),
}

impl InfiniteLoopRule<'_, '_, '_> {
    fn on_while(&mut self, node: &WhileNode<'_>) {
        let predicate = node.predicate();
        return_unless!(truthy_literal(&predicate));
        self.check_loop(
            node.location(),
            node.keyword_loc(),
            node.do_keyword_loc(),
            node.closing_loc(),
            predicate,
        );
    }

    fn on_until(&mut self, node: &UntilNode<'_>) {
        let predicate = node.predicate();
        return_unless!(predicate.as_false_node().is_some() || predicate.as_nil_node().is_some());
        self.check_loop(
            node.location(),
            node.keyword_loc(),
            node.do_keyword_loc(),
            node.closing_loc(),
            predicate,
        );
    }

    fn check_loop(
        &mut self,
        location: Location<'_>,
        keyword: Location<'_>,
        do_keyword: Option<Location<'_>>,
        closing: Option<Location<'_>>,
        predicate: Node<'_>,
    ) {
        let range = location.start_offset()..location.end_offset();
        return_if!(changes_local_scope(range.clone(), self.ancestors()));
        let modifier = keyword.start_offset() > location.start_offset();
        let source = self.source_file().slice(range.clone()).unwrap_or_default();
        let post_condition = modifier && source.trim_start().starts_with("begin");
        let correction = if post_condition {
            let mut suffix_start = keyword.start_offset();
            while suffix_start > location.start_offset()
                && matches!(self.source().as_bytes()[suffix_start - 1], b' ' | b'\t')
            {
                suffix_start -= 1;
            }
            LoopCorrection::PostCondition {
                begin: location.start_offset()..location.start_offset() + "begin".len(),
                suffix: suffix_start..predicate.location().end_offset(),
            }
        } else if modifier {
            let body = self.source()[location.start_offset()..keyword.start_offset()].trim_end();
            let replacement = if body.contains('\n') {
                let width = self
                    .related_config_value("Layout/IndentationWidth", "Width")
                    .and_then(|value| value.parse::<usize>().ok())
                    .unwrap_or(2);
                let indent = " ".repeat(width);
                let body = body
                    .lines()
                    .map(|line| format!("{indent}{line}"))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("loop do\n{body}\nend")
            } else {
                format!("loop {{ {body} }}")
            };
            LoopCorrection::Replace(range, replacement)
        } else {
            let header_end = do_keyword
                .map_or(predicate.location().end_offset(), |location| location.end_offset());
            LoopCorrection::Replace(keyword.start_offset()..header_end, "loop do".to_string())
        };
        let _ = closing;
        add_offense!(self, &keyword, message: "Use `Kernel#loop` for infinite loops.", |corrector| {
            match correction {
                LoopCorrection::Replace(range, replacement) => corrector.replace(range, replacement),
                LoopCorrection::PostCondition { begin, suffix } => {
                    corrector.replace(begin, "loop do");
                    corrector.remove(suffix);
                }
            }
        });
    }
}

enum LoopCorrection {
    Replace(std::ops::Range<usize>, String),
    PostCondition {
        begin: std::ops::Range<usize>,
        suffix: std::ops::Range<usize>,
    },
}

fn truthy_literal(node: &Node<'_>) -> bool {
    node.as_true_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_array_node().is_some()
        || node.as_hash_node().is_some()
        || node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_regular_expression_node().is_some()
}

fn changes_local_scope(loop_range: std::ops::Range<usize>, ancestors: &[Node<'_>]) -> bool {
    let Some(body) = ancestors.iter().rev().find_map(|ancestor| {
        ancestor
            .as_def_node()
            .and_then(|definition| definition.body())
            .or_else(|| ancestor.as_block_node().and_then(|block| block.body()))
            .or_else(|| ancestor.as_lambda_node().and_then(|lambda| lambda.body()))
            .or_else(|| ancestor.as_class_node().and_then(|class| class.body()))
            .or_else(|| ancestor.as_module_node().and_then(|module| module.body()))
            .or_else(|| {
                ancestor
                    .as_program_node()
                    .map(|program| program.statements().as_node())
            })
    }) else {
        return false;
    };
    let mut uses = LocalVariableUses::default();
    uses.visit(&body);
    uses.assignments.iter().any(|(name, assignment)| {
        loop_range.contains(assignment)
            && !uses
                .assignments
                .iter()
                .any(|(candidate, offset)| candidate == name && *offset < loop_range.start)
            && uses
                .references
                .iter()
                .any(|(candidate, offset)| candidate == name && *offset > loop_range.end)
    })
}

#[derive(Default)]
struct LocalVariableUses {
    assignments: Vec<(Vec<u8>, usize)>,
    references: Vec<(Vec<u8>, usize)>,
}

impl LocalVariableUses {
    fn assignment(&mut self, name: &[u8], location: Location<'_>) {
        self.assignments
            .push((name.to_vec(), location.start_offset()));
    }
}

impl<'pr> ruby_prism::Visit<'pr> for LocalVariableUses {
    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        self.assignment(node.name().as_slice(), node.location());
        ruby_prism::visit_local_variable_write_node(self, node);
    }

    fn visit_local_variable_or_write_node(&mut self, node: &ruby_prism::LocalVariableOrWriteNode<'pr>) {
        self.assignment(node.name().as_slice(), node.location());
        ruby_prism::visit_local_variable_or_write_node(self, node);
    }

    fn visit_local_variable_and_write_node(&mut self, node: &ruby_prism::LocalVariableAndWriteNode<'pr>) {
        self.assignment(node.name().as_slice(), node.location());
        ruby_prism::visit_local_variable_and_write_node(self, node);
    }

    fn visit_local_variable_operator_write_node(&mut self, node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>) {
        self.assignment(node.name().as_slice(), node.location());
        ruby_prism::visit_local_variable_operator_write_node(self, node);
    }

    fn visit_local_variable_target_node(&mut self, node: &ruby_prism::LocalVariableTargetNode<'pr>) {
        self.assignment(node.name().as_slice(), node.location());
    }

    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        self.references
            .push((node.name().as_slice().to_vec(), node.location().start_offset()));
    }

    fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}
    fn visit_class_node(&mut self, _node: &ruby_prism::ClassNode<'pr>) {}
    fn visit_module_node(&mut self, _node: &ruby_prism::ModuleNode<'pr>) {}
}
