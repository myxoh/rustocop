//! Converts Prism's native tree into the Parser-shaped node contract exposed by
//! `rubocop-ast`. Prism intentionally has a different tree shape (notably calls
//! with blocks and wrapper nodes), so this is a semantic adapter rather than a
//! direct node-for-node copy.

use std::ops::Range;

use ruby_prism::{
    AndNode, AssocNode, AssocSplatNode, BeginNode, BlockNode, BlockParametersNode, BreakNode,
    CallNode, CallOperatorWriteNode, CallTargetNode, CaseMatchNode, CaseNode, ClassNode,
    ConstantPathNode, ConstantPathWriteNode, DefNode, ForNode, ForwardingSuperNode, IfNode, InNode,
    IndexTargetNode, InterpolatedRegularExpressionNode, InterpolatedStringNode,
    InterpolatedXStringNode, LambdaNode, MatchWriteNode, ModuleNode, MultiTargetNode,
    MultiWriteNode, NextNode, Node, OrNode, ParametersNode, RangeNode as PrismRangeNode,
    RegularExpressionNode, RescueModifierNode, RescueNode, ReturnNode, SingletonClassNode,
    SplatNode, StatementsNode, StringNode, SuperNode, SymbolNode, UnlessNode, UntilNode, Visit,
    WhenNode, WhileNode, XStringNode, YieldNode,
};

use super::node::core::{Ast, NodeId, NodeValue};

#[derive(Clone, Copy)]
enum Frame {
    Transparent,
    Node(NodeId),
    Call { outer: NodeId, send: NodeId },
    Super { outer: NodeId, super_node: NodeId },
}

pub(crate) fn convert(source: &str, root: &Node<'_>) -> (Ast, Option<NodeId>) {
    let mut converter = Converter {
        ast: Ast::new(source),
        source,
        frames: Vec::new(),
        roots: Vec::new(),
        pattern_depth: 0,
        multiple_assignment_depth: 0,
    };
    converter.visit(root);
    let root = match converter.roots.as_slice() {
        [] => None,
        [root] => Some(*root),
        roots => {
            let range = roots
                .iter()
                .filter_map(|id| converter.ast.node(*id).source_range())
                .fold(None, |range: Option<Range<usize>>, item| {
                    Some(range.map_or(item.clone(), |current| {
                        current.start.min(item.start)..current.end.max(item.end)
                    }))
                });
            Some(converter.ast.add_node(
                "begin",
                roots.iter().copied().map(NodeValue::Node).collect(),
                range,
            ))
        }
    };
    if let Some(root) = root {
        converter.ast.complete(root);
    }
    (converter.ast, root)
}

struct Converter<'source> {
    ast: Ast,
    source: &'source str,
    frames: Vec<Frame>,
    roots: Vec<NodeId>,
    pattern_depth: usize,
    multiple_assignment_depth: usize,
}

impl Converter<'_> {
    fn current_parent(&self) -> Option<NodeId> {
        self.frames.iter().rev().find_map(|frame| match *frame {
            Frame::Node(id) => Some(id),
            Frame::Call { outer, .. } => Some(outer),
            Frame::Super { outer, .. } => Some(outer),
            Frame::Transparent => None,
        })
    }

    fn attach(&mut self, id: NodeId) {
        if let Some(parent) = self.current_parent() {
            self.ast.append_child(parent, NodeValue::Node(id));
        } else {
            self.roots.push(id);
        }
    }

    fn location(&self, node: &Node<'_>) -> Range<usize> {
        let location = node.location();
        byte_range_to_character(self.source, location.start_offset()..location.end_offset())
    }

    fn source_for(&self, node: &Node<'_>) -> &str {
        let location = node.location();
        self.source
            .get(location.start_offset()..location.end_offset())
            .unwrap_or("")
    }

    #[allow(clippy::too_many_lines)] // One dispatch table mirrors Prism node families.
    fn enter(&mut self, node: Node<'_>) {
        let raw = raw_name(&node);
        if transparent(&raw) {
            self.frames.push(Frame::Transparent);
            return;
        }

        if let Some(lambda) = node.as_lambda_node() {
            let block_kind = implicit_block_kind(lambda.parameters());
            let outer = self
                .ast
                .add_node(block_kind, Vec::new(), Some(self.location(&node)));
            self.attach(outer);
            let operator = lambda.operator_loc();
            let operator_range = byte_range_to_character(
                self.source,
                operator.start_offset()..operator.end_offset(),
            );
            let send = self.ast.add_node(
                "send",
                vec![NodeValue::Nil, NodeValue::Symbol("lambda".into())],
                Some(operator_range),
            );
            self.ast.append_child(outer, NodeValue::Node(send));
            self.set_location(send, "selector", lambda.operator_loc());
            self.set_location(outer, "begin", lambda.opening_loc());
            self.set_location(outer, "end", lambda.closing_loc());
            self.frames.push(Frame::Call { outer, send });
            return;
        }

        if raw.ends_with("AndWriteNode")
            || raw.ends_with("OrWriteNode")
            || raw.ends_with("OperatorWriteNode")
        {
            let source = self.source_for(&node).to_owned();
            let kind = if raw.ends_with("AndWriteNode") {
                "and_asgn"
            } else if raw.ends_with("OrWriteNode") {
                "or_asgn"
            } else {
                "op_asgn"
            };
            let outer = self
                .ast
                .add_node(kind, Vec::new(), Some(self.location(&node)));
            self.attach(outer);
            if raw.starts_with("Call") {
                self.frames.push(Frame::Node(outer));
                return;
            }
            let name = source
                .split(|character: char| {
                    character.is_whitespace() || character == '&' || character == '|'
                })
                .next()
                .unwrap_or(&source)
                .to_owned();
            let lhs_kind = if raw.starts_with("InstanceVariable") {
                "ivasgn"
            } else if raw.starts_with("ClassVariable") {
                "cvasgn"
            } else if raw.starts_with("GlobalVariable") {
                "gvasgn"
            } else {
                "lvasgn"
            };
            let lhs = self.ast.add_node(
                lhs_kind,
                vec![NodeValue::Symbol(name.clone())],
                Some(self.location(&node).start..self.location(&node).start + name.chars().count()),
            );
            self.ast.append_child(outer, NodeValue::Node(lhs));
            if kind == "op_asgn" {
                let operator = source
                    .split_once('=')
                    .map(|(left, _)| left.split_whitespace().last().unwrap_or("").to_owned())
                    .unwrap_or_default();
                self.ast.append_child(outer, NodeValue::Symbol(operator));
            }
            self.frames.push(Frame::Node(outer));
            return;
        }

        if let Some(call) = node.as_call_node() {
            let range = self.location(&node);
            if let Some(block) = call.block().and_then(|block| block.as_block_node()) {
                let mut send_end = block.opening_loc().start_offset();
                while send_end > node.location().start_offset()
                    && self.source.as_bytes()[send_end - 1].is_ascii_whitespace()
                {
                    send_end -= 1;
                }
                let send_range =
                    byte_range_to_character(self.source, node.location().start_offset()..send_end);
                let outer = self.ast.add_node(
                    implicit_block_kind(block.parameters()),
                    Vec::new(),
                    Some(range.clone()),
                );
                self.attach(outer);
                let send = self.ast.add_node(
                    if call.is_safe_navigation() {
                        "csend"
                    } else {
                        "send"
                    },
                    Vec::new(),
                    Some(send_range),
                );
                self.ast.append_child(outer, NodeValue::Node(send));
                self.frames.push(Frame::Call { outer, send });
            } else {
                let send = self.ast.add_node(
                    if call.is_safe_navigation() {
                        "csend"
                    } else {
                        "send"
                    },
                    Vec::new(),
                    Some(range),
                );
                self.attach(send);
                self.frames.push(Frame::Node(send));
            }
            return;
        }

        if let Some(super_node) = node.as_super_node() {
            if let Some(block) = super_node.block().and_then(|block| block.as_block_node()) {
                let mut super_end = block.opening_loc().start_offset();
                while super_end > node.location().start_offset()
                    && self.source.as_bytes()[super_end - 1].is_ascii_whitespace()
                {
                    super_end -= 1;
                }
                let outer = self.ast.add_node(
                    implicit_block_kind(block.parameters()),
                    Vec::new(),
                    Some(self.location(&node)),
                );
                self.attach(outer);
                let inner = self.ast.add_node(
                    "super",
                    Vec::new(),
                    Some(byte_range_to_character(
                        self.source,
                        node.location().start_offset()..super_end,
                    )),
                );
                self.ast.append_child(outer, NodeValue::Node(inner));
                self.frames.push(Frame::Super {
                    outer,
                    super_node: inner,
                });
                return;
            }
        }

        let kind = if self.pattern_depth > 0 && raw.ends_with("TargetNode") {
            "match_var"
        } else if self.multiple_assignment_depth > 0 {
            match raw.as_str() {
                "ConstantTargetNode" | "ConstantPathTargetNode" => "casgn",
                "CallTargetNode" => "send",
                "IndexTargetNode" => "indexasgn",
                "ImplicitRestNode" => "splat",
                _ => parser_kind(&raw, self.source_for(&node)),
            }
        } else if raw == "SplatNode"
            && self
                .current_parent()
                .is_some_and(|parent| self.ast.node(parent).kind() == "mlhs")
        {
            "restarg"
        } else {
            parser_kind(&raw, self.source_for(&node))
        };
        let children = scalar_children(&raw, kind, self.source_for(&node));
        let id = self
            .ast
            .add_node(kind, children, Some(self.location(&node)));
        self.attach(id);
        let node_source = self.source_for(&node).to_owned();
        self.set_delimiter_locations(id, &raw, &node_source);
        if raw == "UnlessNode" {
            self.set_keyword_location(id, "unless");
        }
        self.frames.push(Frame::Node(id));
    }

    fn leave(&mut self) {
        self.frames.pop().expect("balanced Prism traversal");
    }

    fn set_keyword_location(&mut self, id: NodeId, keyword: &str) {
        let Some(range) = self.ast.node(id).source_range() else {
            return;
        };
        let start = range.start;
        self.ast.set_location(
            id,
            "keyword",
            start..start + keyword.chars().count(),
            keyword,
        );
    }

    fn set_delimiter_locations(&mut self, id: NodeId, raw: &str, source: &str) {
        let Some(range) = self.ast.node(id).source_range() else {
            return;
        };
        let (opening, closing) = match raw {
            "ArrayNode" if source.starts_with('[') => {
                (Some("["), source.ends_with(']').then_some("]"))
            }
            "ArrayNode" if source.starts_with('%') => {
                let opening_end = source
                    .char_indices()
                    .find(|(_, character)| matches!(character, '(' | '[' | '{' | '<'))
                    .map(|(index, character)| index + character.len_utf8());
                let opening = opening_end.and_then(|end| source.get(..end));
                let closing = source
                    .chars()
                    .last()
                    .map(|character| match character {
                        ')' => ")",
                        ']' => "]",
                        '}' => "}",
                        '>' => ">",
                        _ => "",
                    })
                    .filter(|value| !value.is_empty());
                (opening, closing)
            }
            "HashNode" | "KeywordHashNode" if source.starts_with('{') => {
                (Some("{"), source.ends_with('}').then_some("}"))
            }
            "ParenthesesNode" => (Some("("), source.ends_with(')').then_some(")")),
            _ => (None, None),
        };
        if let Some(opening) = opening {
            self.ast.set_location(
                id,
                "begin",
                range.start..range.start + opening.chars().count(),
                opening,
            );
        }
        if let Some(closing) = closing {
            self.ast.set_location(
                id,
                "end",
                range.end.saturating_sub(closing.chars().count())..range.end,
                closing,
            );
        }
    }

    fn with_parent(&mut self, parent: NodeId, operation: impl FnOnce(&mut Self)) {
        self.frames.push(Frame::Node(parent));
        operation(self);
        self.frames.pop();
    }

    fn populate_call(&mut self, node: &CallNode<'_>, send: NodeId) {
        self.with_parent(send, |this| {
            if let Some(receiver) = node.receiver() {
                this.visit(&receiver);
            } else {
                this.ast.append_child(send, NodeValue::Nil);
            }
            this.ast.append_child(
                send,
                NodeValue::Symbol(String::from_utf8_lossy(node.name().as_slice()).into_owned()),
            );
            if let Some(arguments) = node.arguments() {
                for argument in &arguments.arguments() {
                    this.visit(&argument);
                }
            }
            if let Some(block_pass) = node.block().filter(|block| block.as_block_node().is_none()) {
                this.visit(&block_pass);
            }
        });
        if let Some(location) = node.message_loc() {
            self.set_location(send, "selector", location);
        }
        if let Some(location) = node.opening_loc() {
            self.set_location(send, "begin", location);
        }
        if let Some(location) = node.closing_loc() {
            self.set_location(send, "end", location);
        }
        if let Some(location) = node.call_operator_loc() {
            self.set_location(send, "dot", location);
        }
    }

    fn populate_block(&mut self, node: &BlockNode<'_>, outer: NodeId) {
        if let Some(parameters) = node.parameters() {
            self.populate_block_parameters(parameters, outer);
        } else {
            let args = self.ast.add_node("args", Vec::new(), None);
            self.ast.append_child(outer, NodeValue::Node(args));
        }
        if let Some(body) = node.body() {
            self.visit(&body);
        } else {
            self.ast.append_child(outer, NodeValue::Nil);
        }
        self.set_location(outer, "begin", node.opening_loc());
        self.set_location(outer, "end", node.closing_loc());
    }

    fn populate_block_parameters(&mut self, parameters: Node<'_>, outer: NodeId) {
        if let Some(numbered) = parameters.as_numbered_parameters_node() {
            self.ast
                .append_child(outer, NodeValue::Integer(i64::from(numbered.maximum())));
        } else if parameters.as_it_parameters_node().is_some() {
            self.ast.append_child(outer, NodeValue::Symbol("it".into()));
        } else {
            self.visit(&parameters);
        }
    }

    fn set_location(&mut self, id: NodeId, name: &str, location: ruby_prism::Location<'_>) {
        let bytes = location.start_offset()..location.end_offset();
        let range = byte_range_to_character(self.source, bytes.clone());
        let text = self.source.get(bytes).unwrap_or("").to_owned();
        self.ast.set_location(id, name, range, &text);
    }

    fn set_heredoc_end(&mut self, id: NodeId, location: ruby_prism::Location<'_>) {
        let start = location.start_offset();
        let mut end = location.end_offset();
        while end > start && matches!(self.source.as_bytes()[end - 1], b'\n' | b'\r') {
            end -= 1;
        }
        let range = byte_range_to_character(self.source, start..end);
        let value = self.source.get(start..end).unwrap_or("").to_owned();
        self.ast.set_location(id, "heredoc_end", range, &value);
    }

    fn append_regopt(
        &mut self,
        regexp: NodeId,
        closing: ruby_prism::Location<'_>,
        options: Vec<char>,
    ) {
        let bytes = closing.start_offset()..closing.end_offset();
        let closing_source = self.source.get(bytes.clone()).unwrap_or("");
        let option_count = options.len();
        let closing_range = byte_range_to_character(self.source, bytes);
        let option_start = closing_range.end.saturating_sub(option_count);
        let regopt = self.ast.add_node(
            "regopt",
            options
                .into_iter()
                .map(|option| NodeValue::Symbol(option.to_string()))
                .collect(),
            Some(option_start..closing_range.end),
        );
        self.ast.append_child(regexp, NodeValue::Node(regopt));
        self.ast
            .set_location(regexp, "end", closing_range, closing_source);
    }

    fn populate_literal(
        &mut self,
        literal: NodeId,
        value: NodeValue,
        opening: Option<ruby_prism::Location<'_>>,
        closing: Option<ruby_prism::Location<'_>>,
    ) {
        self.ast.clear_children(literal);
        self.ast.append_child(literal, value);
        if let Some(opening) = opening {
            self.set_location(literal, "begin", opening);
        }
        if let Some(closing) = closing {
            self.set_location(literal, "end", closing);
        }
    }
}

impl<'pr> Visit<'pr> for Converter<'_> {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        self.enter(node);
    }
    fn visit_branch_node_leave(&mut self) {
        self.leave();
    }
    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        self.enter(node);
    }
    fn visit_leaf_node_leave(&mut self) {
        self.leave();
    }

    fn visit_call_node(&mut self, node: &CallNode<'pr>) {
        let frame = *self.frames.last().expect("call frame");
        match frame {
            Frame::Call { outer, send } => {
                self.populate_call(node, send);
                let block = node
                    .block()
                    .and_then(|block| block.as_block_node())
                    .expect("block call frame");
                self.populate_block(&block, outer);
            }
            Frame::Node(send) => self.populate_call(node, send),
            Frame::Transparent | Frame::Super { .. } => {
                unreachable!("calls are never transparent or super frames")
            }
        }
    }

    fn visit_block_node(&mut self, node: &BlockNode<'pr>) {
        let Frame::Node(block) = *self.frames.last().expect("block frame") else {
            unreachable!()
        };
        self.ast.clear_children(block);
        self.populate_block(node, block);
    }

    fn visit_call_operator_write_node(&mut self, node: &CallOperatorWriteNode<'pr>) {
        let Frame::Node(assignment) = *self.frames.last().expect("call operator-write frame")
        else {
            unreachable!()
        };
        self.ast.clear_children(assignment);
        let lhs = self.ast.add_node(
            if node.is_safe_navigation() {
                "csend"
            } else {
                "send"
            },
            Vec::new(),
            Some(self.location(&node.as_node())),
        );
        self.ast.append_child(assignment, NodeValue::Node(lhs));
        self.with_parent(lhs, |this| {
            if let Some(receiver) = node.receiver() {
                this.visit(&receiver);
            } else {
                this.ast.append_child(lhs, NodeValue::Nil);
            }
            this.ast.append_child(
                lhs,
                NodeValue::Symbol(
                    String::from_utf8_lossy(node.read_name().as_slice()).into_owned(),
                ),
            );
        });
        if let Some(location) = node.message_loc() {
            self.set_location(lhs, "selector", location);
        }
        if let Some(location) = node.call_operator_loc() {
            self.set_location(lhs, "dot", location);
        }
        self.ast.append_child(
            assignment,
            NodeValue::Symbol(
                String::from_utf8_lossy(node.binary_operator().as_slice()).into_owned(),
            ),
        );
        self.set_location(assignment, "operator", node.binary_operator_loc());
        self.visit(&node.value());
    }

    fn visit_call_target_node(&mut self, node: &CallTargetNode<'pr>) {
        let Frame::Node(send) = *self.frames.last().expect("call-target frame") else {
            unreachable!()
        };
        self.ast.clear_children(send);
        self.visit(&node.receiver());
        let mut name = String::from_utf8_lossy(node.name().as_slice()).into_owned();
        if !name.ends_with('=') {
            name.push('=');
        }
        self.ast.append_child(send, NodeValue::Symbol(name));
        self.set_location(send, "dot", node.call_operator_loc());
        self.set_location(send, "selector", node.message_loc());
    }

    fn visit_break_node(&mut self, node: &BreakNode<'pr>) {
        let Frame::Node(keyword) = *self.frames.last().expect("break frame") else {
            unreachable!()
        };
        self.ast.clear_children(keyword);
        if let Some(arguments) = node.arguments() {
            for argument in &arguments.arguments() {
                if let Some(parentheses) = argument.as_parentheses_node() {
                    if let Some(body) = parentheses.body() {
                        self.visit(&body);
                    }
                } else {
                    self.visit(&argument);
                }
            }
        }
        self.set_location(keyword, "keyword", node.keyword_loc());
    }

    fn visit_next_node(&mut self, node: &NextNode<'pr>) {
        let Frame::Node(keyword) = *self.frames.last().expect("next frame") else {
            unreachable!()
        };
        self.ast.clear_children(keyword);
        if let Some(arguments) = node.arguments() {
            for argument in &arguments.arguments() {
                if let Some(parentheses) = argument.as_parentheses_node() {
                    if let Some(body) = parentheses.body() {
                        self.visit(&body);
                    }
                } else {
                    self.visit(&argument);
                }
            }
        }
        self.set_location(keyword, "keyword", node.keyword_loc());
    }

    fn visit_return_node(&mut self, node: &ReturnNode<'pr>) {
        let Frame::Node(keyword) = *self.frames.last().expect("return frame") else {
            unreachable!()
        };
        self.ast.clear_children(keyword);
        if let Some(arguments) = node.arguments() {
            for argument in &arguments.arguments() {
                if let Some(parentheses) = argument.as_parentheses_node() {
                    if let Some(body) = parentheses.body() {
                        self.visit(&body);
                    }
                } else {
                    self.visit(&argument);
                }
            }
        }
        self.set_location(keyword, "keyword", node.keyword_loc());
    }

    fn visit_index_target_node(&mut self, node: &IndexTargetNode<'pr>) {
        let Frame::Node(index) = *self.frames.last().expect("index-target frame") else {
            unreachable!()
        };
        self.ast.clear_children(index);
        self.visit(&node.receiver());
        if let Some(arguments) = node.arguments() {
            for argument in &arguments.arguments() {
                self.visit(&argument);
            }
        }
        if let Some(block) = node.block() {
            self.visit(&block.as_node());
        }
        self.set_location(index, "begin", node.opening_loc());
        self.set_location(index, "end", node.closing_loc());
    }

    fn visit_multi_write_node(&mut self, node: &MultiWriteNode<'pr>) {
        let Frame::Node(masgn) = *self.frames.last().expect("multi-write frame") else {
            unreachable!()
        };
        self.ast.clear_children(masgn);
        let operator = node.operator_loc();
        let lhs_start = node.location().start_offset();
        let lhs_end = operator.start_offset();
        let mlhs = self.ast.add_node(
            "mlhs",
            Vec::new(),
            Some(byte_range_to_character(self.source, lhs_start..lhs_end)),
        );
        self.ast.append_child(masgn, NodeValue::Node(mlhs));
        self.multiple_assignment_depth += 1;
        self.with_parent(mlhs, |this| {
            for target in &node.lefts() {
                this.visit(&target);
            }
            if let Some(rest) = node.rest() {
                this.visit(&rest);
            }
            for target in &node.rights() {
                this.visit(&target);
            }
        });
        self.multiple_assignment_depth -= 1;
        if let Some(location) = node.lparen_loc() {
            self.set_location(mlhs, "begin", location);
        }
        if let Some(location) = node.rparen_loc() {
            self.set_location(mlhs, "end", location);
        }
        self.visit(&node.value());
        self.set_location(masgn, "operator", operator);
    }

    fn visit_multi_target_node(&mut self, node: &MultiTargetNode<'pr>) {
        let Frame::Node(mlhs) = *self.frames.last().expect("multi-target frame") else {
            unreachable!()
        };
        self.ast.clear_children(mlhs);
        for target in &node.lefts() {
            self.visit(&target);
        }
        if let Some(rest) = node.rest() {
            self.visit(&rest);
        }
        for target in &node.rights() {
            self.visit(&target);
        }
        if let Some(location) = node.lparen_loc() {
            self.set_location(mlhs, "begin", location);
        }
        if let Some(location) = node.rparen_loc() {
            self.set_location(mlhs, "end", location);
        }
    }

    fn visit_lambda_node(&mut self, node: &LambdaNode<'pr>) {
        let Frame::Call { outer, .. } = *self.frames.last().expect("lambda frame") else {
            unreachable!()
        };
        if let Some(parameters) = node.parameters() {
            self.populate_block_parameters(parameters, outer);
        } else {
            let args = self.ast.add_node("args", Vec::new(), None);
            self.ast.append_child(outer, NodeValue::Node(args));
        }
        if let Some(body) = node.body() {
            self.visit(&body);
        } else {
            self.ast.append_child(outer, NodeValue::Nil);
        }
    }

    fn visit_match_write_node(&mut self, node: &MatchWriteNode<'pr>) {
        let Frame::Node(_) = *self.frames.last().expect("match-write frame") else {
            unreachable!()
        };
        let call = node.call();
        if let Some(receiver) = call.receiver() {
            self.visit(&receiver);
        }
        if let Some(arguments) = call.arguments() {
            for argument in &arguments.arguments() {
                self.visit(&argument);
            }
        }
        if let Some(operator) = call.message_loc() {
            let current = self.current_parent().expect("match-write node");
            self.set_location(current, "operator", operator);
        }
    }

    fn visit_and_node(&mut self, node: &AndNode<'pr>) {
        let Frame::Node(binary) = *self.frames.last().expect("and frame") else {
            unreachable!()
        };
        self.visit(&node.left());
        self.visit(&node.right());
        self.set_location(binary, "operator", node.operator_loc());
    }

    fn visit_assoc_node(&mut self, node: &AssocNode<'pr>) {
        let Frame::Node(pair) = *self.frames.last().expect("association frame") else {
            unreachable!()
        };
        self.ast.clear_children(pair);
        self.visit(&node.key());
        self.visit(&node.value());
        if let Some(location) = node.operator_loc() {
            self.set_location(pair, "operator", location);
        } else {
            let location = node.location();
            let bytes = location.start_offset()..location.end_offset();
            let source = self.source.get(bytes.clone()).unwrap_or("");
            if let Some(relative) = source.find(':') {
                let operator_bytes = bytes.start + relative..bytes.start + relative + 1;
                self.ast.set_location(
                    pair,
                    "operator",
                    byte_range_to_character(self.source, operator_bytes),
                    ":",
                );
            }
        }
    }

    fn visit_assoc_splat_node(&mut self, node: &AssocSplatNode<'pr>) {
        let Frame::Node(splat) = *self.frames.last().expect("association splat frame") else {
            unreachable!()
        };
        self.ast.clear_children(splat);
        if let Some(value) = node.value() {
            self.visit(&value);
        } else {
            self.ast.replace_kind(splat, "forwarded_kwrestarg");
        }
        self.set_location(splat, "operator", node.operator_loc());
    }

    fn visit_or_node(&mut self, node: &OrNode<'pr>) {
        let Frame::Node(binary) = *self.frames.last().expect("or frame") else {
            unreachable!()
        };
        self.visit(&node.left());
        self.visit(&node.right());
        self.set_location(binary, "operator", node.operator_loc());
    }

    fn visit_begin_node(&mut self, node: &BeginNode<'pr>) {
        let Frame::Node(begin) = *self.frames.last().expect("begin frame") else {
            unreachable!()
        };
        // Parser/rubocop-ast reserve `kwbegin` for an explicit `begin ... end`.
        // Prism also uses BeginNode for implicit rescue/ensure wrappers.
        self.ast.replace_kind(
            begin,
            if node.begin_keyword_loc().is_some() {
                "kwbegin"
            } else {
                "begin"
            },
        );
        if node.rescue_clause().is_none() && node.ensure_clause().is_none() {
            self.ast.clear_children(begin);
            if let Some(statements) = node.statements() {
                for statement in &statements.body() {
                    self.visit(&statement);
                }
            }
            if let Some(keyword) = node.begin_keyword_loc() {
                self.set_location(begin, "begin", keyword);
            }
            if let Some(keyword) = node.end_keyword_loc() {
                self.set_location(begin, "end", keyword);
            }
            return;
        }

        self.ast.clear_children(begin);
        if let Some(keyword) = node.begin_keyword_loc() {
            self.set_location(begin, "begin", keyword);
        }
        if let Some(keyword) = node.end_keyword_loc() {
            self.set_location(begin, "end", keyword);
        }

        let protected_parent = if let Some(ensure_clause) = node.ensure_clause() {
            // Parser/rubocop-ast model `ensure` as `[protected_body,
            // ensure_body]`. Prism instead places both parts beside one another
            // on BeginNode, so assemble the legacy shape explicitly.
            let ensure =
                self.ast
                    .add_node("ensure", Vec::new(), Some(self.location(&node.as_node())));
            self.ast.append_child(begin, NodeValue::Node(ensure));
            self.set_location(ensure, "keyword", ensure_clause.ensure_keyword_loc());
            self.set_location(ensure, "end", ensure_clause.end_keyword_loc());
            ensure
        } else {
            begin
        };

        self.with_parent(protected_parent, |this| {
            if node.rescue_clause().is_some() {
                let rescue =
                    this.ast
                        .add_node("rescue", Vec::new(), Some(this.location(&node.as_node())));
                this.ast
                    .append_child(protected_parent, NodeValue::Node(rescue));
                this.with_parent(rescue, |this| {
                    if let Some(statements) = node.statements() {
                        this.visit_statements_node(&statements);
                    } else {
                        this.ast.append_child(rescue, NodeValue::Nil);
                    }
                    if let Some(clause) = node.rescue_clause() {
                        this.visit_rescue_node(&clause);
                    }
                    if let Some(otherwise) = node.else_clause() {
                        this.set_location(rescue, "else", otherwise.else_keyword_loc());
                        if let Some(statements) = otherwise.statements() {
                            this.visit_statements_node(&statements);
                        } else {
                            this.ast.append_child(rescue, NodeValue::Nil);
                        }
                    }
                });
            } else if let Some(statements) = node.statements() {
                this.visit_statements_node(&statements);
            } else {
                this.ast.append_child(protected_parent, NodeValue::Nil);
            }
        });

        if let Some(ensure_clause) = node.ensure_clause() {
            self.with_parent(protected_parent, |this| {
                if let Some(statements) = ensure_clause.statements() {
                    this.visit_statements_node(&statements);
                } else {
                    this.ast.append_child(protected_parent, NodeValue::Nil);
                }
            });
        }
    }

    fn visit_rescue_modifier_node(&mut self, node: &RescueModifierNode<'pr>) {
        let Frame::Node(rescue) = *self.frames.last().expect("rescue modifier frame") else {
            unreachable!()
        };
        self.ast.clear_children(rescue);
        self.visit(&node.expression());
        let body_range = self.location(&node.rescue_expression());
        let keyword_range = byte_range_to_character(
            self.source,
            node.keyword_loc().start_offset()..node.keyword_loc().end_offset(),
        );
        let resbody = self.ast.add_node(
            "resbody",
            vec![NodeValue::Nil, NodeValue::Nil],
            Some(keyword_range.start..body_range.end),
        );
        self.ast.append_child(rescue, NodeValue::Node(resbody));
        self.set_location(resbody, "keyword", node.keyword_loc());
        // Preserve Prism's structural distinction so consumers that mirror
        // Parser token callbacks can exclude rescue modifiers exactly.
        self.set_location(resbody, "modifier", node.keyword_loc());
        self.with_parent(resbody, |this| this.visit(&node.rescue_expression()));
    }

    fn visit_rescue_node(&mut self, node: &RescueNode<'pr>) {
        let Frame::Node(frame_node) = *self.frames.last().expect("resbody frame") else {
            unreachable!()
        };
        let resbody = if self.ast.node(frame_node).kind() == "resbody" {
            frame_node
        } else {
            let resbody =
                self.ast
                    .add_node("resbody", Vec::new(), Some(self.location(&node.as_node())));
            self.ast.append_child(frame_node, NodeValue::Node(resbody));
            resbody
        };
        self.ast.clear_children(resbody);
        self.frames.push(Frame::Node(resbody));
        if node.exceptions().is_empty() {
            self.ast.append_child(resbody, NodeValue::Nil);
        } else {
            let exceptions = self.ast.add_node("array", Vec::new(), None);
            self.ast.append_child(resbody, NodeValue::Node(exceptions));
            self.with_parent(exceptions, |this| {
                for exception in &node.exceptions() {
                    this.visit(&exception);
                }
            });
        }
        if let Some(reference) = node.reference() {
            self.visit(&reference);
        } else {
            self.ast.append_child(resbody, NodeValue::Nil);
        }
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        } else {
            self.ast.append_child(resbody, NodeValue::Nil);
        }
        self.set_location(resbody, "keyword", node.keyword_loc());
        if let Some(operator) = node.operator_loc() {
            self.set_location(resbody, "operator", operator);
        }
        let clause_start = node.keyword_loc().start_offset();
        let clause_end = node
            .statements()
            .map(|statements| statements.location().end_offset())
            .or_else(|| {
                node.reference()
                    .map(|reference| reference.location().end_offset())
            })
            .or_else(|| {
                node.exceptions()
                    .last()
                    .map(|exception| exception.location().end_offset())
            })
            .unwrap_or_else(|| node.keyword_loc().end_offset());
        self.ast.set_source_range(
            resbody,
            Some(byte_range_to_character(
                self.source,
                clause_start..clause_end,
            )),
        );
        self.frames.pop();
        if let Some(subsequent) = node.subsequent() {
            self.with_parent(frame_node, |this| this.visit_rescue_node(&subsequent));
        }
    }

    fn visit_range_node(&mut self, node: &PrismRangeNode<'pr>) {
        let Frame::Node(range) = *self.frames.last().expect("range frame") else {
            unreachable!()
        };
        self.ast.clear_children(range);
        if let Some(left) = node.left() {
            self.visit(&left);
        } else {
            self.ast.append_child(range, NodeValue::Nil);
        }
        if let Some(right) = node.right() {
            self.visit(&right);
        } else {
            self.ast.append_child(range, NodeValue::Nil);
        }
        self.set_location(range, "operator", node.operator_loc());
    }

    fn visit_block_parameters_node(&mut self, node: &BlockParametersNode<'pr>) {
        let args = self
            .ast
            .add_node("args", Vec::new(), Some(self.location(&node.as_node())));
        self.attach(args);
        if let Some(opening) = node.opening_loc() {
            self.set_location(args, "begin", opening);
        }
        if let Some(closing) = node.closing_loc() {
            self.set_location(args, "end", closing);
        }
        self.with_parent(args, |this| {
            if let Some(parameters) = node.parameters() {
                this.visit_parameters_node(&parameters);
            }
            for local in &node.locals() {
                this.visit(&local);
            }
        });
    }

    fn visit_parameters_node(&mut self, node: &ParametersNode<'pr>) {
        for parameter in &node.requireds() {
            self.visit(&parameter);
        }
        for parameter in &node.optionals() {
            self.visit(&parameter);
        }
        if let Some(parameter) = node.rest() {
            self.visit(&parameter);
        }
        for parameter in &node.posts() {
            self.visit(&parameter);
        }
        for parameter in &node.keywords() {
            self.visit(&parameter);
        }
        if let Some(parameter) = node.keyword_rest() {
            self.visit(&parameter);
        }
        if let Some(parameter) = node.block() {
            self.visit(&parameter.as_node());
        }
    }

    fn visit_statements_node(&mut self, node: &StatementsNode<'pr>) {
        let body = node.body();
        if body.len() == 1 {
            self.visit(&body.first().expect("single statement"));
            return;
        }
        if body.is_empty() {
            return;
        }
        let begin = self
            .ast
            .add_node("begin", Vec::new(), Some(self.location(&node.as_node())));
        self.attach(begin);
        self.with_parent(begin, |this| {
            for statement in &body {
                this.visit(&statement);
            }
        });
    }

    fn visit_def_node(&mut self, node: &DefNode<'pr>) {
        let Frame::Node(definition) = *self.frames.last().expect("definition frame") else {
            unreachable!()
        };
        if let Some(receiver) = node.receiver() {
            self.ast.replace_kind(definition, "defs");
            self.visit(&receiver);
        }
        self.ast.append_child(
            definition,
            NodeValue::Symbol(String::from_utf8_lossy(node.name().as_slice()).into_owned()),
        );
        if let Some(parameters) = node.parameters() {
            self.visit(&parameters.as_node());
            if let Some(args) = self
                .ast
                .node(definition)
                .child_nodes()
                .last()
                .map(|node| node.id())
            {
                if let (Some(opening), Some(closing)) = (node.lparen_loc(), node.rparen_loc()) {
                    let range = byte_range_to_character(
                        self.source,
                        opening.start_offset()..closing.end_offset(),
                    );
                    self.ast.set_source_range(args, Some(range));
                    self.set_location(args, "begin", opening);
                    self.set_location(args, "end", closing);
                }
            }
        } else {
            let args = self.ast.add_node("args", Vec::new(), None);
            self.ast.append_child(definition, NodeValue::Node(args));
            if let (Some(opening), Some(closing)) = (node.lparen_loc(), node.rparen_loc()) {
                self.ast.set_source_range(
                    args,
                    Some(byte_range_to_character(
                        self.source,
                        opening.start_offset()..closing.end_offset(),
                    )),
                );
                self.set_location(args, "begin", opening);
                self.set_location(args, "end", closing);
            }
        }
        if let Some(body) = node.body() {
            self.visit(&body);
        } else {
            self.ast.append_child(definition, NodeValue::Nil);
        }
        self.set_location(definition, "keyword", node.def_keyword_loc());
        self.set_location(definition, "name", node.name_loc());
        if let Some(location) = node.operator_loc() {
            self.set_location(definition, "operator", location);
        }
        if let Some(location) = node.end_keyword_loc() {
            self.set_location(definition, "end", location);
        }
    }

    fn visit_if_node(&mut self, node: &IfNode<'pr>) {
        let Frame::Node(conditional) = *self.frames.last().expect("if frame") else {
            unreachable!()
        };
        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        } else {
            self.ast.append_child(conditional, NodeValue::Nil);
        }
        if let Some(subsequent) = node.subsequent() {
            if let Some(otherwise) = subsequent.as_else_node() {
                if node.if_keyword_loc().is_none() {
                    self.set_location(conditional, "colon", otherwise.else_keyword_loc());
                } else {
                    self.set_location(conditional, "else", otherwise.else_keyword_loc());
                }
            } else if let Some(elsif) = subsequent.as_if_node() {
                if let Some(keyword) = elsif.if_keyword_loc() {
                    self.set_location(conditional, "else", keyword);
                }
            }
            self.visit(&subsequent);
        } else {
            self.ast.append_child(conditional, NodeValue::Nil);
        }
        if let Some(location) = node.if_keyword_loc() {
            self.set_location(conditional, "keyword", location);
        }
        if let Some(location) = node.then_keyword_loc() {
            if node.if_keyword_loc().is_none() {
                self.set_location(conditional, "question", location);
            } else {
                self.set_location(conditional, "begin", location);
            }
        }
        if let Some(location) = node.end_keyword_loc() {
            self.set_location(conditional, "end", location);
        }
    }

    fn visit_unless_node(&mut self, node: &UnlessNode<'pr>) {
        let Frame::Node(conditional) = *self.frames.last().expect("unless frame") else {
            unreachable!()
        };
        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        } else {
            self.ast.append_child(conditional, NodeValue::Nil);
        }
        if let Some(otherwise) = node.else_clause() {
            self.set_location(conditional, "else", otherwise.else_keyword_loc());
            self.visit(&otherwise.as_node());
        } else {
            self.ast.append_child(conditional, NodeValue::Nil);
        }
        self.set_location(conditional, "keyword", node.keyword_loc());
        if let Some(location) = node.then_keyword_loc() {
            self.set_location(conditional, "begin", location);
        }
        if let Some(location) = node.end_keyword_loc() {
            self.set_location(conditional, "end", location);
        }
    }

    fn visit_class_node(&mut self, node: &ClassNode<'pr>) {
        let Frame::Node(class) = *self.frames.last().expect("class frame") else {
            unreachable!()
        };
        self.visit(&node.constant_path());
        if let Some(superclass) = node.superclass() {
            self.visit(&superclass);
        } else {
            self.ast.append_child(class, NodeValue::Nil);
        }
        if let Some(body) = node.body() {
            self.visit(&body);
        } else {
            self.ast.append_child(class, NodeValue::Nil);
        }
        self.set_location(class, "keyword", node.class_keyword_loc());
        if let Some(location) = node.inheritance_operator_loc() {
            self.set_location(class, "operator", location);
        }
        self.set_location(class, "end", node.end_keyword_loc());
    }

    fn visit_case_node(&mut self, node: &CaseNode<'pr>) {
        let Frame::Node(case_node) = *self.frames.last().expect("case frame") else {
            unreachable!()
        };
        self.ast.clear_children(case_node);
        if let Some(predicate) = node.predicate() {
            self.visit(&predicate);
        } else {
            self.ast.append_child(case_node, NodeValue::Nil);
        }
        for condition in &node.conditions() {
            self.visit(&condition);
        }
        if let Some(otherwise) = node.else_clause() {
            self.set_location(case_node, "else", otherwise.else_keyword_loc());
            if let Some(statements) = otherwise.statements() {
                self.visit_statements_node(&statements);
            } else {
                self.ast.append_child(case_node, NodeValue::Nil);
            }
        } else {
            self.ast.append_child(case_node, NodeValue::Nil);
        }
        self.set_location(case_node, "keyword", node.case_keyword_loc());
        self.set_location(case_node, "end", node.end_keyword_loc());
    }

    fn visit_case_match_node(&mut self, node: &CaseMatchNode<'pr>) {
        let Frame::Node(case_node) = *self.frames.last().expect("case-match frame") else {
            unreachable!()
        };
        self.ast.clear_children(case_node);
        if let Some(predicate) = node.predicate() {
            self.visit(&predicate);
        } else {
            self.ast.append_child(case_node, NodeValue::Nil);
        }
        for condition in &node.conditions() {
            self.visit(&condition);
        }
        if let Some(otherwise) = node.else_clause() {
            self.set_location(case_node, "else", otherwise.else_keyword_loc());
            if let Some(statements) = otherwise.statements() {
                self.visit_statements_node(&statements);
            } else {
                let empty_else = self.ast.add_node(
                    "empty_else",
                    Vec::new(),
                    Some(byte_range_to_character(
                        self.source,
                        otherwise.else_keyword_loc().start_offset()
                            ..otherwise.else_keyword_loc().end_offset(),
                    )),
                );
                self.ast
                    .append_child(case_node, NodeValue::Node(empty_else));
            }
        } else {
            self.ast.append_child(case_node, NodeValue::Nil);
        }
        self.set_location(case_node, "keyword", node.case_keyword_loc());
        self.set_location(case_node, "end", node.end_keyword_loc());
    }

    fn visit_in_node(&mut self, node: &InNode<'pr>) {
        let Frame::Node(in_pattern) = *self.frames.last().expect("in-pattern frame") else {
            unreachable!()
        };
        self.ast.clear_children(in_pattern);
        let pattern = node.pattern();
        if let Some(guard) = pattern.as_if_node() {
            self.pattern_depth += 1;
            if let Some(statements) = guard.statements() {
                self.visit_statements_node(&statements);
            } else {
                self.ast.append_child(in_pattern, NodeValue::Nil);
            }
            self.pattern_depth -= 1;
            let guard_kind = guard
                .if_keyword_loc()
                .and_then(|location| {
                    self.source
                        .get(location.start_offset()..location.end_offset())
                })
                .map_or("if_guard", |keyword| {
                    if keyword == "unless" {
                        "unless_guard"
                    } else {
                        "if_guard"
                    }
                });
            let guard_node =
                self.ast
                    .add_node(guard_kind, Vec::new(), Some(self.location(&pattern)));
            self.ast
                .append_child(in_pattern, NodeValue::Node(guard_node));
            self.with_parent(guard_node, |this| this.visit(&guard.predicate()));
        } else if let Some(guard) = pattern.as_unless_node() {
            self.pattern_depth += 1;
            if let Some(statements) = guard.statements() {
                self.visit_statements_node(&statements);
            } else {
                self.ast.append_child(in_pattern, NodeValue::Nil);
            }
            self.pattern_depth -= 1;
            let guard_node =
                self.ast
                    .add_node("unless_guard", Vec::new(), Some(self.location(&pattern)));
            self.ast
                .append_child(in_pattern, NodeValue::Node(guard_node));
            self.with_parent(guard_node, |this| this.visit(&guard.predicate()));
        } else {
            self.pattern_depth += 1;
            self.visit(&pattern);
            self.pattern_depth -= 1;
            self.ast.append_child(in_pattern, NodeValue::Nil);
        }
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        } else {
            self.ast.append_child(in_pattern, NodeValue::Nil);
        }
        self.set_location(in_pattern, "keyword", node.in_loc());
        if let Some(location) = node.then_loc() {
            self.set_location(in_pattern, "begin", location);
        }
    }

    fn visit_when_node(&mut self, node: &WhenNode<'pr>) {
        let Frame::Node(when_node) = *self.frames.last().expect("when frame") else {
            unreachable!()
        };
        self.ast.clear_children(when_node);
        for condition in &node.conditions() {
            self.visit(&condition);
        }
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        } else {
            self.ast.append_child(when_node, NodeValue::Nil);
        }
        self.set_location(when_node, "keyword", node.keyword_loc());
        if let Some(location) = node.then_keyword_loc() {
            self.set_location(when_node, "begin", location);
        }
    }

    fn visit_for_node(&mut self, node: &ForNode<'pr>) {
        let Frame::Node(for_node) = *self.frames.last().expect("for frame") else {
            unreachable!()
        };
        self.visit(&node.index());
        self.visit(&node.collection());
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        } else {
            self.ast.append_child(for_node, NodeValue::Nil);
        }
        self.set_location(for_node, "keyword", node.for_keyword_loc());
        self.set_location(for_node, "operator", node.in_keyword_loc());
        if let Some(location) = node.do_keyword_loc() {
            self.set_location(for_node, "begin", location);
        }
        self.set_location(for_node, "end", node.end_keyword_loc());
    }

    fn visit_while_node(&mut self, node: &WhileNode<'pr>) {
        let Frame::Node(loop_node) = *self.frames.last().expect("while frame") else {
            unreachable!()
        };
        if node.is_begin_modifier() {
            self.ast.replace_kind(loop_node, "while_post");
        }
        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        } else {
            self.ast.append_child(loop_node, NodeValue::Nil);
        }
        self.set_location(loop_node, "keyword", node.keyword_loc());
        if let Some(location) = node.do_keyword_loc() {
            self.set_location(loop_node, "begin", location);
        }
        if let Some(location) = node.closing_loc() {
            self.set_location(loop_node, "end", location);
        }
    }

    fn visit_until_node(&mut self, node: &UntilNode<'pr>) {
        let Frame::Node(loop_node) = *self.frames.last().expect("until frame") else {
            unreachable!()
        };
        if node.is_begin_modifier() {
            self.ast.replace_kind(loop_node, "until_post");
        }
        self.visit(&node.predicate());
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
        } else {
            self.ast.append_child(loop_node, NodeValue::Nil);
        }
        self.set_location(loop_node, "keyword", node.keyword_loc());
        if let Some(location) = node.do_keyword_loc() {
            self.set_location(loop_node, "begin", location);
        }
        if let Some(location) = node.closing_loc() {
            self.set_location(loop_node, "end", location);
        }
    }

    fn visit_module_node(&mut self, node: &ModuleNode<'pr>) {
        let Frame::Node(module) = *self.frames.last().expect("module frame") else {
            unreachable!()
        };
        self.visit(&node.constant_path());
        if let Some(body) = node.body() {
            self.visit(&body);
        } else {
            self.ast.append_child(module, NodeValue::Nil);
        }
        self.set_location(module, "keyword", node.module_keyword_loc());
        self.set_location(module, "end", node.end_keyword_loc());
    }

    fn visit_singleton_class_node(&mut self, node: &SingletonClassNode<'pr>) {
        let Frame::Node(class) = *self.frames.last().expect("singleton class frame") else {
            unreachable!()
        };
        self.visit(&node.expression());
        if let Some(body) = node.body() {
            self.visit(&body);
        } else {
            self.ast.append_child(class, NodeValue::Nil);
        }
        self.set_location(class, "keyword", node.class_keyword_loc());
        self.set_location(class, "operator", node.operator_loc());
        self.set_location(class, "end", node.end_keyword_loc());
    }

    fn visit_constant_path_node(&mut self, node: &ConstantPathNode<'pr>) {
        let Frame::Node(constant) = *self.frames.last().expect("constant path frame") else {
            unreachable!()
        };
        self.ast.clear_children(constant);
        if let Some(parent) = node.parent() {
            self.visit(&parent);
        } else if self.source_for(&node.as_node()).starts_with("::") {
            let start = self.location(&node.as_node()).start;
            let cbase = self
                .ast
                .add_node("cbase", Vec::new(), Some(start..start + 2));
            self.ast.append_child(constant, NodeValue::Node(cbase));
        } else {
            self.ast.append_child(constant, NodeValue::Nil);
        }
        let name = node
            .name()
            .map(|name| String::from_utf8_lossy(name.as_slice()).into_owned())
            .unwrap_or_default();
        self.ast.append_child(constant, NodeValue::Symbol(name));
        self.set_location(constant, "double_colon", node.delimiter_loc());
        self.set_location(constant, "name", node.name_loc());
    }

    fn visit_constant_path_write_node(&mut self, node: &ConstantPathWriteNode<'pr>) {
        let Frame::Node(assignment) = *self.frames.last().expect("constant path write frame")
        else {
            unreachable!()
        };
        self.ast.clear_children(assignment);
        let target = node.target();
        if let Some(parent) = target.parent() {
            self.visit(&parent);
        } else if self.source_for(&target.as_node()).starts_with("::") {
            let start = self.location(&target.as_node()).start;
            let cbase = self
                .ast
                .add_node("cbase", Vec::new(), Some(start..start + 2));
            self.ast.append_child(assignment, NodeValue::Node(cbase));
        } else {
            self.ast.append_child(assignment, NodeValue::Nil);
        }
        self.ast.append_child(
            assignment,
            NodeValue::Symbol(
                target
                    .name()
                    .map(|name| String::from_utf8_lossy(name.as_slice()).into_owned())
                    .unwrap_or_default(),
            ),
        );
        self.visit(&node.value());
        self.set_location(assignment, "operator", node.operator_loc());
    }

    fn visit_regular_expression_node(&mut self, node: &RegularExpressionNode<'pr>) {
        let Frame::Node(regexp) = *self.frames.last().expect("regexp frame") else {
            unreachable!()
        };
        self.ast.clear_children(regexp);
        let content_location = node.content_loc();
        let content_range = byte_range_to_character(
            self.source,
            content_location.start_offset()..content_location.end_offset(),
        );
        let content = String::from_utf8_lossy(node.unescaped()).into_owned();
        let string =
            self.ast
                .add_node("str", vec![NodeValue::String(content)], Some(content_range));
        self.ast.append_child(regexp, NodeValue::Node(string));
        self.set_location(regexp, "begin", node.opening_loc());
        self.append_regopt(regexp, node.closing_loc(), regexp_options_regular(node));
    }

    fn visit_interpolated_string_node(&mut self, node: &InterpolatedStringNode<'pr>) {
        let Frame::Node(string) = *self.frames.last().expect("interpolated string frame") else {
            unreachable!()
        };
        self.ast.clear_children(string);
        for part in &node.parts() {
            self.visit(&part);
        }
        let opening = node.opening_loc();
        let heredoc = opening.as_ref().is_some_and(|location| {
            self.source
                .get(location.start_offset()..location.end_offset())
                .is_some_and(|source| source.starts_with("<<"))
        });
        if heredoc {
            if let Some(closing) = node.closing_loc() {
                self.set_heredoc_end(string, closing);
            }
            if let Some(opening) = opening {
                self.ast.set_source_range(
                    string,
                    Some(byte_range_to_character(
                        self.source,
                        opening.start_offset()..opening.end_offset(),
                    )),
                );
            }
        } else {
            if let Some(opening) = opening {
                self.set_location(string, "begin", opening);
            }
            if let Some(closing) = node.closing_loc() {
                self.set_location(string, "end", closing);
            }
        }
    }

    fn visit_interpolated_x_string_node(&mut self, node: &InterpolatedXStringNode<'pr>) {
        let Frame::Node(string) = *self.frames.last().expect("interpolated xstring frame") else {
            unreachable!()
        };
        self.ast.clear_children(string);
        for part in &node.parts() {
            self.visit(&part);
        }
        let opening = node.opening_loc();
        let opening_source = self
            .source
            .get(opening.start_offset()..opening.end_offset())
            .unwrap_or("");
        if opening_source.starts_with("<<") {
            self.set_heredoc_end(string, node.closing_loc());
            self.ast.set_source_range(
                string,
                Some(byte_range_to_character(
                    self.source,
                    opening.start_offset()..opening.end_offset(),
                )),
            );
        } else {
            self.set_location(string, "begin", opening);
            self.set_location(string, "end", node.closing_loc());
        }
    }

    fn visit_interpolated_regular_expression_node(
        &mut self,
        node: &InterpolatedRegularExpressionNode<'pr>,
    ) {
        let Frame::Node(regexp) = *self.frames.last().expect("regexp frame") else {
            unreachable!()
        };
        self.ast.clear_children(regexp);
        self.with_parent(regexp, |this| {
            for part in &node.parts() {
                this.visit(&part);
            }
        });
        self.set_location(regexp, "begin", node.opening_loc());
        self.append_regopt(
            regexp,
            node.closing_loc(),
            regexp_options_interpolated(node),
        );
    }

    fn visit_string_node(&mut self, node: &StringNode<'pr>) {
        let Frame::Node(string) = *self.frames.last().expect("string frame") else {
            unreachable!()
        };
        let heredoc = node.opening_loc().as_ref().is_some_and(|location| {
            self.source
                .get(location.start_offset()..location.end_offset())
                .is_some_and(|source| source.starts_with("<<"))
        });
        self.populate_literal(
            string,
            NodeValue::String(String::from_utf8_lossy(node.unescaped()).into_owned()),
            if heredoc { None } else { node.opening_loc() },
            if heredoc { None } else { node.closing_loc() },
        );
        if heredoc {
            self.set_location(string, "heredoc_body", node.content_loc());
            if let Some(closing) = node.closing_loc() {
                self.set_heredoc_end(string, closing);
            }
            if let Some(opening) = node.opening_loc() {
                self.ast.set_source_range(
                    string,
                    Some(byte_range_to_character(
                        self.source,
                        opening.start_offset()..opening.end_offset(),
                    )),
                );
            }
        }
    }

    fn visit_symbol_node(&mut self, node: &SymbolNode<'pr>) {
        let Frame::Node(symbol) = *self.frames.last().expect("symbol frame") else {
            unreachable!()
        };
        self.populate_literal(
            symbol,
            NodeValue::Symbol(String::from_utf8_lossy(node.unescaped()).into_owned()),
            node.opening_loc(),
            node.closing_loc(),
        );
    }

    fn visit_splat_node(&mut self, node: &SplatNode<'pr>) {
        if self.frames.last().is_some_and(
            |frame| matches!(frame, Frame::Node(id) if self.ast.node(*id).kind() == "restarg"),
        ) {
            return;
        }
        ruby_prism::visit_splat_node(self, node);
    }

    fn visit_super_node(&mut self, node: &SuperNode<'pr>) {
        let frame = *self.frames.last().expect("super frame");
        let super_node = match frame {
            Frame::Node(super_node) | Frame::Super { super_node, .. } => super_node,
            Frame::Transparent | Frame::Call { .. } => unreachable!(),
        };
        self.ast.clear_children(super_node);
        self.with_parent(super_node, |this| {
            if let Some(arguments) = node.arguments() {
                for argument in &arguments.arguments() {
                    this.visit(&argument);
                }
            }
        });
        self.set_location(super_node, "keyword", node.keyword_loc());
        if let Some(location) = node.lparen_loc() {
            self.set_location(super_node, "begin", location);
        }
        if let Some(location) = node.rparen_loc() {
            self.set_location(super_node, "end", location);
        }
        if let Frame::Super { outer, .. } = frame {
            let block = node
                .block()
                .and_then(|block| block.as_block_node())
                .expect("block super frame");
            self.populate_block(&block, outer);
        } else if let Some(block) = node.block() {
            self.visit(&block);
        }
    }

    fn visit_forwarding_super_node(&mut self, node: &ForwardingSuperNode<'pr>) {
        let Frame::Node(super_node) = *self.frames.last().expect("forwarding-super frame") else {
            unreachable!()
        };
        self.ast.clear_children(super_node);
        if let Some(block) = node.block() {
            self.visit(&block.as_node());
        }
        let start = node.location().start_offset();
        self.ast.set_location(
            super_node,
            "keyword",
            byte_range_to_character(self.source, start..start + "super".len()),
            "super",
        );
    }

    fn visit_yield_node(&mut self, node: &YieldNode<'pr>) {
        let Frame::Node(yield_node) = *self.frames.last().expect("yield frame") else {
            unreachable!()
        };
        self.ast.clear_children(yield_node);
        if let Some(arguments) = node.arguments() {
            for argument in &arguments.arguments() {
                self.visit(&argument);
            }
        }
        self.set_location(yield_node, "keyword", node.keyword_loc());
        if let Some(location) = node.lparen_loc() {
            self.set_location(yield_node, "begin", location);
        }
        if let Some(location) = node.rparen_loc() {
            self.set_location(yield_node, "end", location);
        }
    }

    fn visit_x_string_node(&mut self, node: &XStringNode<'pr>) {
        let Frame::Node(string) = *self.frames.last().expect("xstring frame") else {
            unreachable!()
        };
        self.ast.clear_children(string);
        let content_location = node.content_loc();
        let content_range = byte_range_to_character(
            self.source,
            content_location.start_offset()..content_location.end_offset(),
        );
        let content = self.ast.add_node(
            "str",
            vec![NodeValue::String(
                String::from_utf8_lossy(node.unescaped()).into_owned(),
            )],
            Some(content_range),
        );
        self.ast.append_child(string, NodeValue::Node(content));
        let opening = node.opening_loc();
        let opening_source = self
            .source
            .get(opening.start_offset()..opening.end_offset())
            .unwrap_or("");
        if opening_source.starts_with("<<") {
            self.set_location(string, "heredoc_body", node.content_loc());
            self.set_heredoc_end(string, node.closing_loc());
            self.ast.set_source_range(
                string,
                Some(byte_range_to_character(
                    self.source,
                    opening.start_offset()..opening.end_offset(),
                )),
            );
        } else {
            self.set_location(string, "begin", opening);
            self.set_location(string, "end", node.closing_loc());
        }
    }
}

fn implicit_block_kind(parameters: Option<Node<'_>>) -> &'static str {
    match parameters {
        Some(parameters) if parameters.as_numbered_parameters_node().is_some() => "numblock",
        Some(parameters) if parameters.as_it_parameters_node().is_some() => "itblock",
        _ => "block",
    }
}

fn regexp_options_regular(node: &RegularExpressionNode<'_>) -> Vec<char> {
    [
        ('i', node.is_ignore_case()),
        ('m', node.is_multi_line()),
        ('x', node.is_extended()),
        ('o', node.is_once()),
        ('n', node.is_ascii_8bit()),
        ('u', node.is_utf_8()),
        ('e', node.is_euc_jp()),
        ('s', node.is_windows_31j()),
    ]
    .into_iter()
    .filter_map(|(option, enabled)| enabled.then_some(option))
    .collect()
}

fn regexp_options_interpolated(node: &InterpolatedRegularExpressionNode<'_>) -> Vec<char> {
    [
        ('i', node.is_ignore_case()),
        ('m', node.is_multi_line()),
        ('x', node.is_extended()),
        ('o', node.is_once()),
        ('n', node.is_ascii_8bit()),
        ('u', node.is_utf_8()),
        ('e', node.is_euc_jp()),
        ('s', node.is_windows_31j()),
    ]
    .into_iter()
    .filter_map(|(option, enabled)| enabled.then_some(option))
    .collect()
}

fn transparent(raw: &str) -> bool {
    matches!(
        raw,
        "ProgramNode"
            | "StatementsNode"
            | "ArgumentsNode"
            | "BlockParametersNode"
            | "ElseNode"
            | "ImplicitNode"
    )
}

fn raw_name(node: &Node<'_>) -> String {
    // `Node` deliberately exposes typed downcasts rather than a public numeric
    // kind. These tests cover the complete 151-variant Prism 1.9 node enum; the
    // fallback is only for a future Prism release adding a new variant.
    if node.as_call_node().is_some() {
        return "CallNode".into();
    }
    let debug = format!("{node:?}");
    let name = debug
        .split_once('(')
        .map_or(debug.as_str(), |(name, _)| name);
    name.to_owned()
}

fn parser_kind(raw: &str, source: &str) -> &'static str {
    match raw {
        "AliasGlobalVariableNode" | "AliasMethodNode" => "alias",
        "AlternationPatternNode" => "match_alt",
        "AndNode" => "and",
        "ArrayNode" => "array",
        "ArrayPatternNode" => "array_pattern",
        "AssocNode" => "pair",
        "AssocSplatNode" => "kwsplat",
        "BackReferenceReadNode" => "back_ref",
        "BeginNode" => "kwbegin",
        "BlockArgumentNode" => "block_pass",
        "BlockLocalVariableNode" => "shadowarg",
        "BlockNode" => "block",
        "BlockParameterNode" => "blockarg",
        "BreakNode" => "break",
        "CapturePatternNode" => "match_as",
        "CaseMatchNode" => "case_match",
        "CaseNode" => "case",
        "ClassNode" => "class",
        "ClassVariableReadNode" => "cvar",
        "ClassVariableTargetNode" | "ClassVariableWriteNode" => "cvasgn",
        "ConstantPathNode" | "ConstantReadNode" | "ConstantTargetNode" => "const",
        "ConstantPathWriteNode" | "ConstantWriteNode" => "casgn",
        "DefNode" => "def",
        "DefinedNode" => "defined?",
        "EmbeddedStatementsNode" => "begin",
        "EmbeddedVariableNode" => "begin",
        "EnsureNode" => "ensure",
        "FalseNode" => "false",
        "FindPatternNode" => "find_pattern",
        "FlipFlopNode" => {
            if source.contains("...") {
                "eflipflop"
            } else {
                "iflipflop"
            }
        }
        "FloatNode" => "float",
        "ForNode" => "for",
        "ForwardingArgumentsNode" => "forwarded_args",
        "ForwardingParameterNode" => "forward_arg",
        "ForwardingSuperNode" => "zsuper",
        "GlobalVariableReadNode" => "gvar",
        "GlobalVariableTargetNode" | "GlobalVariableWriteNode" => "gvasgn",
        "HashNode" | "KeywordHashNode" => "hash",
        "HashPatternNode" => "hash_pattern",
        "IfNode" | "UnlessNode" => "if",
        "ImaginaryNode" => "complex",
        "InNode" => "in_pattern",
        "InstanceVariableReadNode" => "ivar",
        "InstanceVariableTargetNode" | "InstanceVariableWriteNode" => "ivasgn",
        "IntegerNode" => "int",
        "InterpolatedMatchLastLineNode" | "InterpolatedRegularExpressionNode" => "regexp",
        "InterpolatedStringNode" => "dstr",
        "InterpolatedSymbolNode" => "dsym",
        "InterpolatedXStringNode" => "xstr",
        "ItLocalVariableReadNode" => "lvar",
        "ItParametersNode" | "ParametersNode" => "args",
        "KeywordRestParameterNode" => "kwrestarg",
        "LambdaNode" => "lambda",
        "LocalVariableReadNode" => "lvar",
        "LocalVariableTargetNode" | "LocalVariableWriteNode" => "lvasgn",
        "MatchLastLineNode" | "RegularExpressionNode" => "regexp",
        "MatchPredicateNode" => "match_pattern_p",
        "MatchRequiredNode" => "match_pattern",
        "MatchWriteNode" => "match_with_lvasgn",
        "ModuleNode" => "module",
        "MultiTargetNode" => "mlhs",
        "MultiWriteNode" => "masgn",
        "NextNode" => "next",
        "NilNode" => "nil",
        "NoKeywordsParameterNode" => "kwrestarg",
        "NumberedReferenceReadNode" => "nth_ref",
        "OptionalKeywordParameterNode" => "kwoptarg",
        "OptionalParameterNode" => "optarg",
        "OrNode" => "or",
        "ParenthesesNode" => "begin",
        "PinnedExpressionNode" | "PinnedVariableNode" => "pin",
        "PostExecutionNode" => "postexe",
        "PreExecutionNode" => "preexe",
        "RangeNode" => {
            if source.contains("...") {
                "erange"
            } else {
                "irange"
            }
        }
        "RationalNode" => "rational",
        "RedoNode" => "redo",
        "RequiredKeywordParameterNode" => "kwarg",
        "RequiredParameterNode" => "arg",
        "RescueModifierNode" => "rescue",
        "RescueNode" => "resbody",
        "RestParameterNode" => "restarg",
        "RetryNode" => "retry",
        "ReturnNode" => "return",
        "SelfNode" => "self",
        "SingletonClassNode" => "sclass",
        "SourceEncodingNode" => "__ENCODING__",
        "SourceFileNode" => "__FILE__",
        "SourceLineNode" => "__LINE__",
        "SplatNode" => "splat",
        "StringNode" => "str",
        "SuperNode" => "super",
        "SymbolNode" => "sym",
        "TrueNode" => "true",
        "UndefNode" => "undef",
        "UntilNode" => "until",
        "WhenNode" => "when",
        "WhileNode" => "while",
        "XStringNode" => "xstr",
        "YieldNode" => "yield",
        name if name.ends_with("AndWriteNode") => "and_asgn",
        name if name.ends_with("OrWriteNode") => "or_asgn",
        name if name.ends_with("OperatorWriteNode") => "op_asgn",
        name if name.ends_with("TargetNode") => "lvasgn",
        _ => "unknown",
    }
}

fn scalar_children(raw: &str, kind: &str, source: &str) -> Vec<NodeValue> {
    let name = || {
        source
            .split(|character: char| {
                character.is_whitespace() || matches!(character, '=' | ',' | ')' | '|' | ';')
            })
            .next()
            .unwrap_or(source)
            .trim_start_matches(['*', '&'])
            .trim_end_matches(':')
            .to_owned()
    };
    match kind {
        "kwrestarg" if raw == "NoKeywordsParameterNode" => vec![NodeValue::Nil],
        "lvar" | "ivar" | "cvar" | "gvar" | "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn"
        | "match_var" | "arg" | "optarg" | "restarg" | "kwarg" | "kwoptarg" | "kwrestarg"
        | "blockarg" | "shadowarg" => {
            let name = name();
            vec![if name.is_empty() {
                NodeValue::Nil
            } else {
                NodeValue::Symbol(name)
            }]
        }
        "const" => vec![
            NodeValue::Nil,
            NodeValue::Symbol(
                source
                    .rsplit("::")
                    .next()
                    .unwrap_or(source)
                    .trim()
                    .to_owned(),
            ),
        ],
        "casgn" => vec![NodeValue::Nil, NodeValue::Symbol(name())],
        "int" => vec![NodeValue::Integer(parse_integer(source).unwrap_or(0))],
        "float" | "rational" | "complex" => {
            vec![NodeValue::String(source.replace('_', ""))]
        }
        "str" | "xstr" => vec![NodeValue::String(unquote(source))],
        "sym" => vec![NodeValue::Symbol(unquote(source.trim_start_matches(':')))],
        "nth_ref" => vec![NodeValue::Integer(
            source.trim_start_matches('$').parse().unwrap_or(0),
        )],
        "false" => Vec::new(),
        "true" => Vec::new(),
        "nil" => Vec::new(),
        _ => Vec::new(),
    }
}

fn parse_integer(source: &str) -> Option<i64> {
    let source = source.replace('_', "");
    if let Some(value) = source.strip_prefix("0x") {
        i64::from_str_radix(value, 16).ok()
    } else if let Some(value) = source.strip_prefix("0b") {
        i64::from_str_radix(value, 2).ok()
    } else if let Some(value) = source.strip_prefix("0o") {
        i64::from_str_radix(value, 8).ok()
    } else {
        source.parse().ok()
    }
}

fn unquote(source: &str) -> String {
    source
        .strip_prefix(['\'', '"', '`'])
        .and_then(|value| value.strip_suffix(['\'', '"', '`']))
        .unwrap_or(source)
        .to_owned()
}

fn byte_range_to_character(source: &str, range: Range<usize>) -> Range<usize> {
    source[..range.start].chars().count()..source[..range.end].chars().count()
}
