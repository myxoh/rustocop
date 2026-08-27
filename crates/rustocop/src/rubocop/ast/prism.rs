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
use super::source_position::SourcePositionIndex;

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
        positions: SourcePositionIndex::new(source),
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
    positions: SourcePositionIndex,
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
        self.positions
            .character_range(location.start_offset()..location.end_offset())
    }

    fn character_range(&self, range: Range<usize>) -> Range<usize> {
        self.positions.character_range(range)
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
        if transparent(raw) {
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
            let operator_range =
                self.character_range(operator.start_offset()..operator.end_offset());
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
            let lhs_kind = if raw.starts_with("Constant") {
                "casgn"
            } else if raw.starts_with("InstanceVariable") {
                "ivasgn"
            } else if raw.starts_with("ClassVariable") {
                "cvasgn"
            } else if raw.starts_with("GlobalVariable") {
                "gvasgn"
            } else {
                "lvasgn"
            };
            let lhs_start = self.location(&node).start;
            let lhs_children = if lhs_kind == "casgn" {
                vec![
                    NodeValue::Nil,
                    NodeValue::Symbol(name.rsplit("::").next().unwrap_or(&name).to_owned()),
                ]
            } else {
                vec![NodeValue::Symbol(name.clone())]
            };
            let lhs = self.ast.add_node(
                lhs_kind,
                lhs_children,
                Some(lhs_start..lhs_start + name.chars().count()),
            );
            if lhs_kind == "casgn" {
                let short_name = name.rsplit("::").next().unwrap_or(&name);
                let name_start = lhs_start + name.chars().count() - short_name.chars().count();
                self.ast.set_location(
                    lhs,
                    "name",
                    name_start..name_start + short_name.chars().count(),
                    short_name,
                );
            }
            self.ast.append_child(outer, NodeValue::Node(lhs));
            let operator_bytes = if kind == "and_asgn" {
                source.find("&&=").map(|start| start..start + 3)
            } else if kind == "or_asgn" {
                source.find("||=").map(|start| start..start + 3)
            } else {
                source.find('=').map(|equal| {
                    let mut start = equal;
                    while start > 0
                        && matches!(
                            source.as_bytes()[start - 1],
                            b'&' | b'|' | b'+' | b'-' | b'*' | b'/' | b'%' | b'^' | b'<' | b'>'
                        )
                    {
                        start -= 1;
                    }
                    start..equal + 1
                })
            };
            if let Some(operator_bytes) = operator_bytes {
                let byte_start = node.location().start_offset() + operator_bytes.start;
                let byte_end = node.location().start_offset() + operator_bytes.end;
                let range = self.character_range(byte_start..byte_end);
                self.ast
                    .set_location(outer, "operator", range, &source[operator_bytes]);
            }
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
                let send_range = self.character_range(node.location().start_offset()..send_end);
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
                    Some(self.character_range(node.location().start_offset()..super_end)),
                );
                self.ast.append_child(outer, NodeValue::Node(inner));
                self.frames.push(Frame::Super {
                    outer,
                    super_node: inner,
                });
                return;
            }
        }

        if let Some(super_node) = node.as_forwarding_super_node() {
            if let Some(block) = super_node.block() {
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
                    "zsuper",
                    Vec::new(),
                    Some(self.character_range(node.location().start_offset()..super_end)),
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
            match raw {
                "ConstantTargetNode" | "ConstantPathTargetNode" => "casgn",
                "CallTargetNode" => "send",
                "IndexTargetNode" => "indexasgn",
                "ImplicitRestNode" => "splat",
                _ => parser_kind(raw, self.source_for(&node)),
            }
        } else if raw == "SplatNode"
            && self
                .current_parent()
                .is_some_and(|parent| self.ast.node(parent).kind() == "mlhs")
        {
            "restarg"
        } else {
            parser_kind(raw, self.source_for(&node))
        };
        // Prism can retain MatchLastLineNode for the regexp nested under `!`
        // after RuboCop has corrected `if !/foo/` to `if !/foo/ =~ $_`.
        // Parser classifies that regexp as an ordinary `regexp`; preserving
        // the MatchLastLine kind would make a correction loop add `=~ $_`
        // again. Normalize the explicit match operand at the adapter boundary.
        let kind = if kind == "match_current_line"
            && self
                .source
                .get(node.location().end_offset()..)
                .is_some_and(|tail| tail.trim_start().starts_with("=~"))
        {
            "regexp"
        } else {
            kind
        };
        let children = scalar_children(raw, kind, self.source_for(&node));
        let id = self
            .ast
            .add_node(kind, children, Some(self.location(&node)));
        self.attach(id);
        let node_source = self.source_for(&node).to_owned();
        self.set_delimiter_locations(id, raw, &node_source);
        if kind == "const" {
            let range = self.location(&node);
            let name = node_source.rsplit("::").next().unwrap_or(&node_source);
            let start = range.end.saturating_sub(name.chars().count());
            self.ast.set_location(id, "name", start..range.end, name);
        }
        if matches!(
            kind,
            "lvar"
                | "ivar"
                | "cvar"
                | "gvar"
                | "lvasgn"
                | "ivasgn"
                | "cvasgn"
                | "gvasgn"
                | "arg"
                | "optarg"
                | "restarg"
                | "kwarg"
                | "kwoptarg"
                | "kwrestarg"
                | "blockarg"
                | "shadowarg"
        ) {
            if let Some(name) = self.ast.node(id).symbol_child(0).map(str::to_owned) {
                let range = self.location(&node);
                self.ast.set_location(
                    id,
                    "name",
                    range.start..range.start + name.chars().count(),
                    &name,
                );
            }
        }
        if matches!(kind, "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn" | "casgn")
            && !raw.ends_with("TargetNode")
        {
            if let Some(relative) = node_source.find('=') {
                let begin = self.location(&node).start + node_source[..relative].chars().count();
                self.ast.set_location(id, "operator", begin..begin + 1, "=");
            }
        }
        if raw == "UnlessNode" {
            self.set_keyword_location(id, "unless");
        }
        if raw == "PreExecutionNode" {
            self.set_keyword_location(id, "BEGIN");
        }
        if raw == "PostExecutionNode" {
            self.set_keyword_location(id, "END");
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
                // Percent arrays accept both paired delimiters (`%W(...)`) and
                // arbitrary single-character delimiters (`%W'...'`). Prism's
                // opening location includes the `%W`/`%w` sigil and delimiter.
                let delimiter = source.char_indices().nth(2);
                let opening_end = delimiter.map(|(index, character)| index + character.len_utf8());
                let opening = opening_end.and_then(|end| source.get(..end));
                let closing_character = delimiter.map(|(_, character)| match character {
                    '(' => ')',
                    '[' => ']',
                    '{' => '}',
                    '<' => '>',
                    other => other,
                });
                let closing = closing_character.and_then(|expected| {
                    let (index, actual) = source.char_indices().next_back()?;
                    (actual == expected).then(|| &source[index..])
                });
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
            let location_end = location.end_offset();
            self.set_location(send, "end", location);
            // Prism extends a call containing a heredoc argument through the
            // heredoc body. Parser's send expression ends at the call's
            // closing delimiter; heredoc body/end remain separate locations.
            let has_heredoc_argument = self
                .ast
                .node(send)
                .arguments()
                .into_iter()
                .any(|argument| argument.heredoc());
            if has_heredoc_argument {
                if let Some(current) = self.ast.node(send).source_range() {
                    let closing_end = self.character_range(location_end..location_end).end;
                    if closing_end < current.end {
                        self.ast
                            .set_source_range(send, Some(current.start..closing_end));
                    }
                }
            }
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
        let range = self.character_range(bytes.clone());
        let text = self.source.get(bytes).unwrap_or("").to_owned();
        self.ast.set_location(id, name, range, &text);
    }

    fn set_heredoc_end(&mut self, id: NodeId, location: ruby_prism::Location<'_>) {
        let start = location.start_offset();
        let mut end = location.end_offset();
        while end > start && matches!(self.source.as_bytes()[end - 1], b'\n' | b'\r') {
            end -= 1;
        }
        let range = self.character_range(start..end);
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
        let closing_range = self.character_range(bytes);
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
            Some(self.character_range(lhs_start..lhs_end)),
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
                self.ast
                    .set_location(pair, "operator", self.character_range(operator_bytes), ":");
            }
        }
        if self.ast.node(pair).loc_is("operator", ":") {
            if let Some(key) = self.ast.node(pair).first_node() {
                if key.kind() == "sym" {
                    if let Some(range) = key.source_range().filter(|range| range.end > range.start)
                    {
                        self.ast
                            .set_source_range(key.id(), Some(range.start..range.end - 1));
                    }
                }
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
        let keyword_range = self
            .character_range(node.keyword_loc().start_offset()..node.keyword_loc().end_offset());
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
            let exception_range =
                node.exceptions()
                    .first()
                    .zip(node.exceptions().last())
                    .map(|(first, last)| {
                        self.character_range(
                            first.location().start_offset()..last.location().end_offset(),
                        )
                    });
            let exceptions = self.ast.add_node("array", Vec::new(), exception_range);
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
        if let Some(assoc) = node.operator_loc() {
            self.set_location(resbody, "assoc", assoc);
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
            Some(self.character_range(clause_start..clause_end)),
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
        // Parser stores interpolation statements directly under the `begin`
        // representing `#{...}`. Prism's StatementsNode would otherwise add
        // a second synthetic `begin`, changing RuboCop callback semantics.
        if self.current_parent().is_some_and(|parent| {
            self.ast.node(parent).kind() == "begin"
                && self
                    .ast
                    .node(parent)
                    .source()
                    .is_some_and(|source| source.starts_with("#{"))
        }) {
            for statement in &body {
                self.visit(&statement);
            }
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
                    let range = self.character_range(opening.start_offset()..closing.end_offset());
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
                    Some(self.character_range(opening.start_offset()..closing.end_offset())),
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
        // Parser represents `unless condition; body; else other; end` using
        // the `if` node shape `(if condition other body)`. Preserve that
        // branch order so rubocop-ast's `if_branch`/`else_branch` queries
        // remain authoritative over the Prism tree.
        if let Some(otherwise) = node.else_clause() {
            self.set_location(conditional, "else", otherwise.else_keyword_loc());
            self.visit(&otherwise.as_node());
        } else {
            self.ast.append_child(conditional, NodeValue::Nil);
        }
        if let Some(statements) = node.statements() {
            self.visit_statements_node(&statements);
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
                    Some(self.character_range(
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
        } else if let Some(first_statement) = node
            .statements()
            .and_then(|statements| statements.body().first())
        {
            let gap = pattern.location().end_offset()..first_statement.location().start_offset();
            if let Some(relative) = self
                .source
                .get(gap.clone())
                .and_then(|source| source.find(';'))
            {
                let bytes = gap.start + relative..gap.start + relative + 1;
                let range = self.character_range(bytes.clone());
                let text = self.source.get(bytes).unwrap_or("");
                self.ast.set_location(in_pattern, "begin", range, text);
            }
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
            // Parser's empty `when` expression ends at the final condition;
            // Prism extends the node through a trailing comment. RuboCop uses
            // the Parser-shaped range for both offenses and CommentsHelp.
            if let Some(end) = self
                .ast
                .node(when_node)
                .conditions()
                .last()
                .and_then(|condition| condition.source_range())
                .map(|range| range.end)
            {
                let start = self
                    .character_range(
                        node.keyword_loc().start_offset()..node.keyword_loc().start_offset(),
                    )
                    .start;
                self.ast.set_source_range(when_node, Some(start..end));
            }
        }
        self.set_location(when_node, "keyword", node.keyword_loc());
        if let Some(location) = node.then_keyword_loc() {
            self.set_location(when_node, "begin", location);
        } else if let Some(condition_end) = self
            .ast
            .node(when_node)
            .conditions()
            .last()
            .and_then(|condition| condition.source_range())
            .map(|range| range.end)
        {
            let source_end = self
                .ast
                .node(when_node)
                .source_range()
                .map_or(condition_end, |range| range.end);
            let between = self.ast.node(when_node).source().unwrap_or_default();
            let node_start = self
                .ast
                .node(when_node)
                .source_range()
                .map_or(0, |range| range.start);
            let relative_start = condition_end.saturating_sub(node_start);
            if let Some(relative) = between
                .chars()
                .skip(relative_start)
                .position(|character| character == ';')
            {
                let start = condition_end + relative;
                if start < source_end {
                    self.ast
                        .set_location(when_node, "begin", start..start + 1, ";");
                }
            }
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
        let content_range =
            self.character_range(content_location.start_offset()..content_location.end_offset());
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
        let mut body_offsets = None;
        for part in &node.parts() {
            let location = part.location();
            body_offsets = Some(body_offsets.map_or(
                location.start_offset()..location.end_offset(),
                |current: Range<usize>| {
                    current.start.min(location.start_offset())
                        ..current.end.max(location.end_offset())
                },
            ));
            self.visit(&part);
        }
        let opening = node.opening_loc();
        let heredoc = opening.as_ref().is_some_and(|location| {
            self.source
                .get(location.start_offset()..location.end_offset())
                .is_some_and(|source| source.starts_with("<<"))
        });
        if heredoc {
            let body_offsets = body_offsets.unwrap_or_else(|| {
                let start = node.closing_loc().map_or_else(
                    || opening.as_ref().map_or(0, ruby_prism::Location::end_offset),
                    |location| location.start_offset(),
                );
                start..start
            });
            let body_range = self.character_range(body_offsets.clone());
            let body = self.source.get(body_offsets).unwrap_or("").to_owned();
            self.ast
                .set_location(string, "heredoc_body", body_range, &body);
            if let Some(closing) = node.closing_loc() {
                self.set_heredoc_end(string, closing);
            }
            if let Some(opening) = opening {
                self.ast.set_source_range(
                    string,
                    Some(self.character_range(opening.start_offset()..opening.end_offset())),
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
        let mut body_offsets = None;
        for part in &node.parts() {
            let location = part.location();
            body_offsets = Some(body_offsets.map_or(
                location.start_offset()..location.end_offset(),
                |current: Range<usize>| {
                    current.start.min(location.start_offset())
                        ..current.end.max(location.end_offset())
                },
            ));
            self.visit(&part);
        }
        let opening = node.opening_loc();
        let opening_source = self
            .source
            .get(opening.start_offset()..opening.end_offset())
            .unwrap_or("");
        if opening_source.starts_with("<<") {
            let body_offsets = body_offsets.unwrap_or_else(|| {
                let start = node.closing_loc().start_offset();
                start..start
            });
            let body_range = self.character_range(body_offsets.clone());
            let body = self.source.get(body_offsets).unwrap_or("").to_owned();
            self.ast
                .set_location(string, "heredoc_body", body_range, &body);
            self.set_heredoc_end(string, node.closing_loc());
            self.ast.set_source_range(
                string,
                Some(self.character_range(opening.start_offset()..opening.end_offset())),
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
        let value = String::from_utf8_lossy(node.unescaped()).into_owned();
        let heredoc = node.opening_loc().as_ref().is_some_and(|location| {
            self.source
                .get(location.start_offset()..location.end_offset())
                .is_some_and(|source| source.starts_with("<<"))
        });
        self.populate_literal(
            string,
            NodeValue::String(value.clone()),
            if heredoc { None } else { node.opening_loc() },
            if heredoc { None } else { node.closing_loc() },
        );
        let embedded_in_interpolation = heredoc
            && self.ast.node(string).parent().is_some_and(|parent| {
                parent.kind() == "begin"
                    && parent
                        .source()
                        .is_some_and(|source| source.starts_with("#{"))
            });
        let content = node.content_loc();
        let physically_multiline = self
            .source
            .get(content.start_offset()..content.end_offset())
            .is_some_and(|source| source.split_inclusive('\n').count() > 1);
        // Parser represents any physically multi-line StringNode as a dynamic string
        // with literal string parts, even without interpolation. Prism keeps
        // it as one StringNode. Normalize the shared AST once so callbacks
        // and node patterns see the Parser contract expected by RuboCop. An
        // escaped `\n` changes the runtime value but remains a plain `str`.
        if embedded_in_interpolation || physically_multiline {
            self.ast.replace_kind(string, "dstr");
            self.ast.clear_children(string);
            let child = self.ast.add_node(
                "str",
                vec![NodeValue::String(value)],
                Some(self.character_range(content.start_offset()..content.end_offset())),
            );
            self.ast.append_child(string, NodeValue::Node(child));
        }
        if heredoc {
            self.set_location(string, "heredoc_body", node.content_loc());
            if let Some(closing) = node.closing_loc() {
                self.set_heredoc_end(string, closing);
            }
            if let Some(opening) = node.opening_loc() {
                self.ast.set_source_range(
                    string,
                    Some(self.character_range(opening.start_offset()..opening.end_offset())),
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
        let frame = *self.frames.last().expect("forwarding-super frame");
        let super_node = match frame {
            Frame::Node(super_node) | Frame::Super { super_node, .. } => super_node,
            Frame::Transparent | Frame::Call { .. } => unreachable!(),
        };
        self.ast.clear_children(super_node);
        if let Frame::Super { outer, .. } = frame {
            let block = node.block().expect("block forwarding-super frame");
            self.populate_block(&block, outer);
        } else if let Some(block) = node.block() {
            self.visit(&block.as_node());
        }
        let start = node.location().start_offset();
        self.ast.set_location(
            super_node,
            "keyword",
            self.character_range(start..start + "super".len()),
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
        let content_range =
            self.character_range(content_location.start_offset()..content_location.end_offset());
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
                Some(self.character_range(opening.start_offset()..opening.end_offset())),
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

macro_rules! prism_node_name {
    ($node:expr, $($cast:ident => $name:literal),+ $(,)?) => {{
        $(if $node.$cast().is_some() { return $name; })+
        unreachable!("ruby-prism added a node variant without updating prism_node_name")
    }};
}

fn raw_name(node: &Node<'_>) -> &'static str {
    // ruby-prism exposes typed downcasts but no public node-kind scalar. Keep
    // this exhaustive list in generated binding order so classification never
    // recursively Debug-formats an AST merely to recover its variant name.
    prism_node_name!(
        node,
        as_call_node => "CallNode",
        as_alias_global_variable_node => "AliasGlobalVariableNode",
        as_alias_method_node => "AliasMethodNode",
        as_alternation_pattern_node => "AlternationPatternNode",
        as_and_node => "AndNode",
        as_arguments_node => "ArgumentsNode",
        as_array_node => "ArrayNode",
        as_array_pattern_node => "ArrayPatternNode",
        as_assoc_node => "AssocNode",
        as_assoc_splat_node => "AssocSplatNode",
        as_back_reference_read_node => "BackReferenceReadNode",
        as_begin_node => "BeginNode",
        as_block_argument_node => "BlockArgumentNode",
        as_block_local_variable_node => "BlockLocalVariableNode",
        as_block_node => "BlockNode",
        as_block_parameter_node => "BlockParameterNode",
        as_block_parameters_node => "BlockParametersNode",
        as_break_node => "BreakNode",
        as_call_and_write_node => "CallAndWriteNode",
        as_call_operator_write_node => "CallOperatorWriteNode",
        as_call_or_write_node => "CallOrWriteNode",
        as_call_target_node => "CallTargetNode",
        as_capture_pattern_node => "CapturePatternNode",
        as_case_match_node => "CaseMatchNode",
        as_case_node => "CaseNode",
        as_class_node => "ClassNode",
        as_class_variable_and_write_node => "ClassVariableAndWriteNode",
        as_class_variable_operator_write_node => "ClassVariableOperatorWriteNode",
        as_class_variable_or_write_node => "ClassVariableOrWriteNode",
        as_class_variable_read_node => "ClassVariableReadNode",
        as_class_variable_target_node => "ClassVariableTargetNode",
        as_class_variable_write_node => "ClassVariableWriteNode",
        as_constant_and_write_node => "ConstantAndWriteNode",
        as_constant_operator_write_node => "ConstantOperatorWriteNode",
        as_constant_or_write_node => "ConstantOrWriteNode",
        as_constant_path_and_write_node => "ConstantPathAndWriteNode",
        as_constant_path_node => "ConstantPathNode",
        as_constant_path_operator_write_node => "ConstantPathOperatorWriteNode",
        as_constant_path_or_write_node => "ConstantPathOrWriteNode",
        as_constant_path_target_node => "ConstantPathTargetNode",
        as_constant_path_write_node => "ConstantPathWriteNode",
        as_constant_read_node => "ConstantReadNode",
        as_constant_target_node => "ConstantTargetNode",
        as_constant_write_node => "ConstantWriteNode",
        as_def_node => "DefNode",
        as_defined_node => "DefinedNode",
        as_else_node => "ElseNode",
        as_embedded_statements_node => "EmbeddedStatementsNode",
        as_embedded_variable_node => "EmbeddedVariableNode",
        as_ensure_node => "EnsureNode",
        as_false_node => "FalseNode",
        as_find_pattern_node => "FindPatternNode",
        as_flip_flop_node => "FlipFlopNode",
        as_float_node => "FloatNode",
        as_for_node => "ForNode",
        as_forwarding_arguments_node => "ForwardingArgumentsNode",
        as_forwarding_parameter_node => "ForwardingParameterNode",
        as_forwarding_super_node => "ForwardingSuperNode",
        as_global_variable_and_write_node => "GlobalVariableAndWriteNode",
        as_global_variable_operator_write_node => "GlobalVariableOperatorWriteNode",
        as_global_variable_or_write_node => "GlobalVariableOrWriteNode",
        as_global_variable_read_node => "GlobalVariableReadNode",
        as_global_variable_target_node => "GlobalVariableTargetNode",
        as_global_variable_write_node => "GlobalVariableWriteNode",
        as_hash_node => "HashNode",
        as_hash_pattern_node => "HashPatternNode",
        as_if_node => "IfNode",
        as_imaginary_node => "ImaginaryNode",
        as_implicit_node => "ImplicitNode",
        as_implicit_rest_node => "ImplicitRestNode",
        as_in_node => "InNode",
        as_index_and_write_node => "IndexAndWriteNode",
        as_index_operator_write_node => "IndexOperatorWriteNode",
        as_index_or_write_node => "IndexOrWriteNode",
        as_index_target_node => "IndexTargetNode",
        as_instance_variable_and_write_node => "InstanceVariableAndWriteNode",
        as_instance_variable_operator_write_node => "InstanceVariableOperatorWriteNode",
        as_instance_variable_or_write_node => "InstanceVariableOrWriteNode",
        as_instance_variable_read_node => "InstanceVariableReadNode",
        as_instance_variable_target_node => "InstanceVariableTargetNode",
        as_instance_variable_write_node => "InstanceVariableWriteNode",
        as_integer_node => "IntegerNode",
        as_interpolated_match_last_line_node => "InterpolatedMatchLastLineNode",
        as_interpolated_regular_expression_node => "InterpolatedRegularExpressionNode",
        as_interpolated_string_node => "InterpolatedStringNode",
        as_interpolated_symbol_node => "InterpolatedSymbolNode",
        as_interpolated_x_string_node => "InterpolatedXStringNode",
        as_it_local_variable_read_node => "ItLocalVariableReadNode",
        as_it_parameters_node => "ItParametersNode",
        as_keyword_hash_node => "KeywordHashNode",
        as_keyword_rest_parameter_node => "KeywordRestParameterNode",
        as_lambda_node => "LambdaNode",
        as_local_variable_and_write_node => "LocalVariableAndWriteNode",
        as_local_variable_operator_write_node => "LocalVariableOperatorWriteNode",
        as_local_variable_or_write_node => "LocalVariableOrWriteNode",
        as_local_variable_read_node => "LocalVariableReadNode",
        as_local_variable_target_node => "LocalVariableTargetNode",
        as_local_variable_write_node => "LocalVariableWriteNode",
        as_match_last_line_node => "MatchLastLineNode",
        as_match_predicate_node => "MatchPredicateNode",
        as_match_required_node => "MatchRequiredNode",
        as_match_write_node => "MatchWriteNode",
        as_missing_node => "MissingNode",
        as_module_node => "ModuleNode",
        as_multi_target_node => "MultiTargetNode",
        as_multi_write_node => "MultiWriteNode",
        as_next_node => "NextNode",
        as_nil_node => "NilNode",
        as_no_keywords_parameter_node => "NoKeywordsParameterNode",
        as_numbered_parameters_node => "NumberedParametersNode",
        as_numbered_reference_read_node => "NumberedReferenceReadNode",
        as_optional_keyword_parameter_node => "OptionalKeywordParameterNode",
        as_optional_parameter_node => "OptionalParameterNode",
        as_or_node => "OrNode",
        as_parameters_node => "ParametersNode",
        as_parentheses_node => "ParenthesesNode",
        as_pinned_expression_node => "PinnedExpressionNode",
        as_pinned_variable_node => "PinnedVariableNode",
        as_post_execution_node => "PostExecutionNode",
        as_pre_execution_node => "PreExecutionNode",
        as_program_node => "ProgramNode",
        as_range_node => "RangeNode",
        as_rational_node => "RationalNode",
        as_redo_node => "RedoNode",
        as_regular_expression_node => "RegularExpressionNode",
        as_required_keyword_parameter_node => "RequiredKeywordParameterNode",
        as_required_parameter_node => "RequiredParameterNode",
        as_rescue_modifier_node => "RescueModifierNode",
        as_rescue_node => "RescueNode",
        as_rest_parameter_node => "RestParameterNode",
        as_retry_node => "RetryNode",
        as_return_node => "ReturnNode",
        as_self_node => "SelfNode",
        as_shareable_constant_node => "ShareableConstantNode",
        as_singleton_class_node => "SingletonClassNode",
        as_source_encoding_node => "SourceEncodingNode",
        as_source_file_node => "SourceFileNode",
        as_source_line_node => "SourceLineNode",
        as_splat_node => "SplatNode",
        as_statements_node => "StatementsNode",
        as_string_node => "StringNode",
        as_super_node => "SuperNode",
        as_symbol_node => "SymbolNode",
        as_true_node => "TrueNode",
        as_undef_node => "UndefNode",
        as_unless_node => "UnlessNode",
        as_until_node => "UntilNode",
        as_when_node => "WhenNode",
        as_while_node => "WhileNode",
        as_x_string_node => "XStringNode",
        as_yield_node => "YieldNode",
    )
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
        "ConstantPathNode" | "ConstantReadNode" => "const",
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
        "InterpolatedMatchLastLineNode" => "match_current_line",
        "InterpolatedRegularExpressionNode" => "regexp",
        "InterpolatedStringNode" => "dstr",
        "InterpolatedSymbolNode" => "dsym",
        "InterpolatedXStringNode" => "xstr",
        "ItLocalVariableReadNode" => "lvar",
        "ItParametersNode" | "ParametersNode" => "args",
        "KeywordRestParameterNode" => "kwrestarg",
        "LambdaNode" => "lambda",
        "LocalVariableReadNode" => "lvar",
        "LocalVariableTargetNode" | "LocalVariableWriteNode" => "lvasgn",
        "ConstantTargetNode" | "ConstantPathTargetNode" => "casgn",
        "MatchLastLineNode" => "match_current_line",
        "RegularExpressionNode" => "regexp",
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
