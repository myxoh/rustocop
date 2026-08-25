// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/node.rb
// Source SHA-256: 7f45751649ede4bc247096c1506394fd0055ebf933ba1268f23f2dc5eac17358
// Source: lib/rubocop/ast/node/mixin/descendence.rb
// Source SHA-256: ab0f800f4e1411b58baa49533cae3b321935b3798b73a49c93be33c18f501cee

use std::collections::HashMap;
use std::ops::Range;

#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub(crate) struct NodeId(usize);

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum NodeValue {
    Node(NodeId),
    Symbol(String),
    String(String),
    Integer(i64),
    Boolean(bool),
    Nil,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct NodeData {
    kind: String,
    children: Vec<NodeValue>,
    parent: Option<NodeId>,
    source_range: Option<Range<usize>>,
    locations: HashMap<String, (Range<usize>, String)>,
    complete: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Ast {
    source: String,
    nodes: Vec<NodeData>,
}

impl Ast {
    pub(crate) fn new(source: impl Into<String>) -> Self {
        Self {
            source: source.into(),
            nodes: Vec::new(),
        }
    }

    pub(crate) fn add_node(
        &mut self,
        kind: &str,
        children: Vec<NodeValue>,
        source_range: Option<Range<usize>>,
    ) -> NodeId {
        let id = NodeId(self.nodes.len());
        self.nodes.push(NodeData {
            kind: kind.into(),
            children: children.clone(),
            parent: None,
            source_range,
            locations: HashMap::new(),
            complete: false,
        });
        for child in children {
            if let NodeValue::Node(child) = child {
                if !self.nodes[child.0].complete {
                    self.nodes[child.0].parent = Some(id);
                }
            }
        }
        id
    }

    pub(crate) fn set_location(
        &mut self,
        node: NodeId,
        name: &str,
        range: Range<usize>,
        source: &str,
    ) {
        self.nodes[node.0]
            .locations
            .insert(name.into(), (range, source.into()));
    }
    pub(crate) fn append_child(&mut self, node: NodeId, child: NodeValue) {
        if let NodeValue::Node(child_id) = child {
            if !self.nodes[child_id.0].complete {
                self.nodes[child_id.0].parent = Some(node);
            }
        }
        self.nodes[node.0].children.push(child);
    }
    pub(crate) fn replace_kind(&mut self, node: NodeId, kind: &str) {
        self.nodes[node.0].kind = kind.into();
    }
    pub(crate) fn updated(
        &mut self,
        node: NodeId,
        kind: Option<&str>,
        children: Option<Vec<NodeValue>>,
    ) -> NodeId {
        let original = self.nodes[node.0].clone();
        let updated = self.add_node(
            kind.unwrap_or(&original.kind),
            children.unwrap_or(original.children),
            original.source_range,
        );
        self.nodes[updated.0].locations = original.locations;
        updated
    }
    pub(crate) fn clear_children(&mut self, node: NodeId) {
        self.nodes[node.0].children.clear();
    }
    pub(crate) fn set_source_range(&mut self, node: NodeId, range: Option<Range<usize>>) {
        self.nodes[node.0].source_range = range;
    }
    pub(crate) fn node(&self, id: NodeId) -> NodeRef<'_> {
        NodeRef { ast: self, id }
    }
    pub(crate) fn complete(&mut self, id: NodeId) {
        self.nodes[id.0].complete = true;
        let children = self.node_ids(id);
        for child in children {
            self.complete(child);
        }
    }
    fn node_ids(&self, id: NodeId) -> Vec<NodeId> {
        self.nodes[id.0]
            .children
            .iter()
            .filter_map(|value| {
                if let NodeValue::Node(id) = value {
                    Some(*id)
                } else {
                    None
                }
            })
            .collect()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct NodeRef<'ast> {
    ast: &'ast Ast,
    id: NodeId,
}

impl<'ast> NodeRef<'ast> {
    fn data(self) -> &'ast NodeData {
        &self.ast.nodes[self.id.0]
    }
    pub(crate) fn id(self) -> NodeId {
        self.id
    }
    pub(crate) fn structurally_equal(self, other: Self) -> bool {
        self.kind() == other.kind()
            && self.children().len() == other.children().len()
            && self
                .children()
                .iter()
                .zip(other.children())
                .all(|(left, right)| match (left, right) {
                    (NodeValue::Node(left), NodeValue::Node(right)) => self
                        .ast
                        .node(*left)
                        .structurally_equal(other.ast.node(*right)),
                    _ => left == right,
                })
    }
    pub(crate) fn kind(self) -> &'ast str {
        &self.data().kind
    }
    pub(crate) fn children(self) -> &'ast [NodeValue] {
        &self.data().children
    }
    pub(crate) fn node_parts(self) -> Vec<NodeValue> {
        match self.kind() {
            "defined?" => std::iter::once(NodeValue::Nil)
                .chain(std::iter::once(NodeValue::Symbol("defined?".into())))
                .chain(self.children().iter().cloned())
                .collect(),
            "super" | "zsuper" => std::iter::once(NodeValue::Nil)
                .chain(std::iter::once(NodeValue::Symbol("super".into())))
                .chain(self.children().iter().cloned())
                .collect(),
            "yield" => std::iter::once(NodeValue::Nil)
                .chain(std::iter::once(NodeValue::Symbol("yield".into())))
                .chain(self.children().iter().cloned())
                .collect(),
            "kwsplat" | "forwarded_kwrestarg" => {
                vec![NodeValue::Node(self.id), NodeValue::Node(self.id)]
            }
            "if" if self.loc_is("keyword", "unless") => {
                let mut parts = self.children().to_vec();
                if parts.len() >= 3 {
                    parts.swap(1, 2);
                }
                parts
            }
            _ => self.children().to_vec(),
        }
    }
    pub(crate) fn parent(self) -> Option<Self> {
        self.data().parent.map(|id| Self { ast: self.ast, id })
    }
    pub(crate) fn parent_exists(self) -> bool {
        self.parent().is_some()
    }
    pub(crate) fn root(self) -> bool {
        self.parent().is_none()
    }
    pub(crate) fn complete(self) -> bool {
        self.data().complete
    }
    pub(crate) fn source_range(self) -> Option<Range<usize>> {
        self.data().source_range.clone()
    }
    pub(crate) fn source(self) -> Option<&'ast str> {
        let range = self.data().source_range.as_ref()?;
        char_slice(&self.ast.source, range.clone())
    }
    pub(crate) fn source_length(self) -> usize {
        self.data()
            .source_range
            .as_ref()
            .map_or(0, |range| range.end - range.start)
    }
    pub(crate) fn empty_source(self) -> bool {
        self.source_length() == 0
    }
    pub(crate) fn first_line(self) -> usize {
        self.data()
            .source_range
            .as_ref()
            .map_or(1, |range| line_at(&self.ast.source, range.start))
    }
    pub(crate) fn last_line(self) -> usize {
        self.data().source_range.as_ref().map_or(1, |range| {
            line_at(&self.ast.source, range.end.saturating_sub(1))
        })
    }
    pub(crate) fn line_count(self) -> usize {
        self.data()
            .source_range
            .as_ref()
            .map_or(0, |_| self.last_line() - self.first_line() + 1)
    }
    pub(crate) fn column(self) -> usize {
        self.data()
            .source_range
            .as_ref()
            .map_or(0, |range| column_at(&self.ast.source, range.start))
    }
    pub(crate) fn last_column(self) -> usize {
        self.data()
            .source_range
            .as_ref()
            .map_or(0, |range| column_at(&self.ast.source, range.end))
    }
    pub(crate) fn loc_column(self, name: &str) -> Option<usize> {
        self.loc(name)
            .map(|(range, _)| column_at(&self.ast.source, range.start))
    }
    pub(crate) fn loc_last_column(self, name: &str) -> Option<usize> {
        self.loc(name)
            .map(|(range, _)| column_at(&self.ast.source, range.end))
    }
    pub(crate) fn nonempty_line_count(self) -> usize {
        self.source().map_or(0, |source| {
            source
                .lines()
                .filter(|line| line.chars().any(|character| !character.is_whitespace()))
                .count()
        })
    }
    pub(crate) fn multiline(self) -> bool {
        self.line_count() > 1
    }
    pub(crate) fn single_line(self) -> bool {
        self.line_count() == 1
    }
    pub(crate) fn type_is(self, types: &[&str]) -> bool {
        types.contains(&self.kind())
            || group_for_type(self.kind()).is_some_and(|group| types.contains(&group))
    }
    pub(crate) fn type_group_is(self, group: &str) -> bool {
        group_for_type(self.kind()) == Some(group)
    }
    pub(crate) fn loc(self, name: &str) -> Option<&'ast (Range<usize>, String)> {
        self.data().locations.get(name)
    }
    pub(crate) fn loc_is(self, name: &str, source: &str) -> bool {
        self.loc(name).is_some_and(|(_, value)| value == source)
    }
    pub(crate) fn child_nodes(self) -> Vec<Self> {
        self.data()
            .children
            .iter()
            .filter_map(|value| {
                if let NodeValue::Node(id) = value {
                    Some(Self {
                        ast: self.ast,
                        id: *id,
                    })
                } else {
                    None
                }
            })
            .collect()
    }
    pub(crate) fn each_child_node(self, types: &[&str]) -> Vec<Self> {
        self.child_nodes()
            .into_iter()
            .filter(|node| types.is_empty() || node.type_is(types))
            .collect()
    }
    pub(crate) fn descendants(self) -> Vec<Self> {
        self.each_descendant(&[])
    }
    pub(crate) fn each_descendant(self, types: &[&str]) -> Vec<Self> {
        let mut result = Vec::new();
        self.visit_descendants(types, &mut result);
        result
    }
    pub(crate) fn each_node(self, types: &[&str]) -> Vec<Self> {
        let mut result = Vec::new();
        if types.is_empty() || self.type_is(types) {
            result.push(self);
        }
        self.visit_descendants(types, &mut result);
        result
    }
    fn visit_descendants(self, types: &[&str], result: &mut Vec<Self>) {
        for child in self.child_nodes() {
            if types.is_empty() || child.type_is(types) {
                result.push(child);
            }
            child.visit_descendants(types, result);
        }
    }
    pub(crate) fn ancestors(self) -> Vec<Self> {
        self.each_ancestor(&[])
    }
    pub(crate) fn each_ancestor(self, types: &[&str]) -> Vec<Self> {
        let mut result = Vec::new();
        let mut current = self.parent();
        while let Some(node) = current {
            if types.is_empty() || node.type_is(types) {
                result.push(node);
            }
            current = node.parent();
        }
        result
    }
    pub(crate) fn sibling_index(self) -> Option<usize> {
        let parent = self.parent()?;
        parent
            .data()
            .children
            .iter()
            .position(|value| matches!(value,NodeValue::Node(id) if *id==self.id))
    }
    pub(crate) fn left_sibling(self) -> Option<Self> {
        let index = self.sibling_index()?;
        if index == 0 {
            return None;
        }
        node_value(self.ast, &self.parent()?.data().children[index - 1])
    }
    pub(crate) fn right_sibling(self) -> Option<Self> {
        let index = self.sibling_index()?;
        self.parent()?
            .data()
            .children
            .get(index + 1)
            .and_then(|value| node_value(self.ast, value))
    }
    pub(crate) fn left_siblings(self) -> Vec<Self> {
        let Some(parent) = self.parent() else {
            return Vec::new();
        };
        let Some(index) = self.sibling_index() else {
            return Vec::new();
        };
        parent.data().children[..index]
            .iter()
            .filter_map(|value| node_value(self.ast, value))
            .collect()
    }
    pub(crate) fn right_siblings(self) -> Vec<Self> {
        let Some(parent) = self.parent() else {
            return Vec::new();
        };
        let Some(index) = self.sibling_index() else {
            return Vec::new();
        };
        parent.data().children[index + 1..]
            .iter()
            .filter_map(|value| node_value(self.ast, value))
            .collect()
    }
    pub(crate) fn literal(self) -> bool {
        LITERALS.contains(&self.kind())
    }
    pub(crate) fn basic_literal(self) -> bool {
        self.literal() && !COMPOSITE_LITERALS.contains(&self.kind())
    }
    pub(crate) fn truthy_literal(self) -> bool {
        TRUTHY_LITERALS.contains(&self.kind())
    }
    pub(crate) fn falsey_literal(self) -> bool {
        matches!(self.kind(), "false" | "nil")
    }
    pub(crate) fn mutable_literal(self) -> bool {
        MUTABLE_LITERALS.contains(&self.kind())
    }
    pub(crate) fn immutable_literal(self) -> bool {
        self.literal() && !self.mutable_literal()
    }
    pub(crate) fn recursive_literal(self) -> bool {
        self.recursive_literal_kind(false)
    }
    pub(crate) fn recursive_basic_literal(self) -> bool {
        self.recursive_literal_kind(true)
    }
    fn recursive_literal_kind(self, basic: bool) -> bool {
        if self.kind() == "send" {
            return matches!(
                self.method_name(),
                Some("==" | "===" | "!=" | "<=" | ">=" | ">" | "<" | "*" | "!" | "<=>")
            ) && self
                .receiver()
                .is_some_and(|node| node.recursive_literal_kind(basic))
                && self
                    .arguments()
                    .into_iter()
                    .all(|node| node.recursive_literal_kind(basic));
        }
        if matches!(
            self.kind(),
            "and"
                | "or"
                | "dstr"
                | "xstr"
                | "dsym"
                | "array"
                | "hash"
                | "irange"
                | "erange"
                | "regexp"
                | "begin"
                | "pair"
        ) {
            return self
                .child_nodes()
                .into_iter()
                .all(|node| node.recursive_literal_kind(basic));
        }
        if basic {
            self.basic_literal()
        } else {
            self.literal()
        }
    }
    pub(crate) fn variable(self) -> bool {
        matches!(self.kind(), "ivar" | "gvar" | "cvar" | "lvar")
    }
    pub(crate) fn reference(self) -> bool {
        matches!(self.kind(), "nth_ref" | "back_ref")
    }
    pub(crate) fn equals_asgn(self) -> bool {
        matches!(
            self.kind(),
            "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn" | "casgn" | "masgn"
        )
    }
    pub(crate) fn shorthand_asgn(self) -> bool {
        matches!(self.kind(), "op_asgn" | "or_asgn" | "and_asgn")
    }
    pub(crate) fn assignment(self) -> bool {
        self.equals_asgn() || self.shorthand_asgn()
    }
    pub(crate) fn assignment_or_similar(self) -> bool {
        self.assignment() || self.kind() == "send" && self.symbol_child(1) == Some("<<")
    }
    pub(crate) fn basic_conditional(self) -> bool {
        matches!(self.kind(), "if" | "while" | "until")
    }
    pub(crate) fn conditional(self) -> bool {
        self.basic_conditional() || matches!(self.kind(), "case" | "case_match")
    }
    pub(crate) fn post_condition_loop(self) -> bool {
        matches!(self.kind(), "while_post" | "until_post")
    }
    pub(crate) fn loop_keyword(self) -> bool {
        self.post_condition_loop() || matches!(self.kind(), "while" | "until" | "for")
    }
    pub(crate) fn operator_keyword(self) -> bool {
        matches!(self.kind(), "and" | "or")
    }
    pub(crate) fn special_keyword(self) -> bool {
        self.source()
            .is_some_and(|source| matches!(source, "__FILE__" | "__LINE__" | "__ENCODING__"))
    }
    pub(crate) fn keyword(self) -> bool {
        self.special_keyword()
            || self.kind() == "send"
                && self.method_name() == Some("!")
                && self.loc_is("selector", "not")
            || KEYWORDS.contains(&self.kind())
                && (!self.operator_keyword() || self.loc_is("operator", self.kind()))
    }
    pub(crate) fn parenthesized_call(self) -> bool {
        self.loc_is("begin", "(")
    }
    pub(crate) fn call_type(self) -> bool {
        matches!(self.kind(), "send" | "csend")
    }
    pub(crate) fn chained(self) -> bool {
        self.parent().is_some_and(|parent| {
            parent.call_type()
                && parent
                    .node_child(0)
                    .is_some_and(|receiver| receiver.id == self.id)
        })
    }
    pub(crate) fn argument(self) -> bool {
        self.parent().is_some_and(|parent| {
            parent.kind() == "send"
                && parent
                    .data()
                    .children
                    .iter()
                    .skip(2)
                    .any(|value| matches!(value,NodeValue::Node(id) if *id==self.id))
        })
    }
    pub(crate) fn str_content(self) -> Option<&'ast str> {
        (self.kind() == "str")
            .then(|| self.string_child(0))
            .flatten()
    }
    pub(crate) fn const_name(self) -> Option<String> {
        if !matches!(self.kind(), "const" | "casgn") {
            return None;
        }
        let short = self.symbol_child(1)?;
        if let Some(namespace) = self.node_child(0).filter(|node| node.kind() != "cbase") {
            Some(format!("{}::{short}", namespace.const_name()?))
        } else {
            Some(short.into())
        }
    }
    pub(crate) fn defined_module(self) -> Option<Self> {
        match self.kind() {
            "class" | "module" => self.node_child(0),
            "casgn" => {
                let value = self.node_child(2)?;
                let call = if matches!(value.kind(), "block" | "numblock" | "itblock") {
                    value.send_node()?
                } else {
                    value
                };
                let receiver = call.receiver()?;
                (call.method_name() == Some("new")
                    && (receiver.global_const("Class") || receiver.global_const("Module")))
                .then_some(self)
            }
            _ => None,
        }
    }
    pub(crate) fn defined_module_parts(self) -> Option<(Option<Self>, &'ast str)> {
        let module = self.defined_module()?;
        Some((module.namespace(), module.short_name()?))
    }
    pub(crate) fn defined_module_name(self) -> Option<String> {
        self.defined_module()?.const_name()
    }
    pub(crate) fn parent_module_name(self) -> Option<String> {
        let mut parts = Vec::new();
        for ancestor in self.each_ancestor(&["class", "module", "sclass", "casgn", "block"]) {
            if let Some(part) = ancestor.parent_module_name_part()? {
                parts.push(part);
            }
        }
        parts.reverse();
        Some(if parts.is_empty() {
            "Object".into()
        } else {
            parts.join("::")
        })
    }
    fn parent_module_name_part(self) -> Option<Option<String>> {
        match self.kind() {
            "class" | "module" | "casgn" => Some(self.defined_module_name()),
            "sclass" => {
                let subject = self.node_child(0)?;
                if subject.kind() == "const" {
                    Some(subject.const_name().map(|name| format!("#<Class:{name}>")))
                } else if subject.kind() == "self" {
                    Some(
                        self.parent_module_name()
                            .map(|name| format!("#<Class:{name}>")),
                    )
                } else {
                    None
                }
            }
            "block" => {
                let send = self.send_node()?;
                if send.method_name() == Some("class_eval") {
                    match send.receiver() {
                        None => Some(None),
                        Some(receiver) if receiver.kind() == "const" => Some(receiver.const_name()),
                        Some(_) => None,
                    }
                } else if self.parent().is_some_and(|parent| {
                    parent.kind() == "casgn"
                        && parent.node_child(2) == Some(self)
                        && self.send_node().is_some_and(|call| {
                            call.method_name() == Some("new")
                                && call.receiver().is_some_and(|receiver| {
                                    receiver.global_const("Class")
                                        || receiver.global_const("Module")
                                })
                        })
                }) {
                    Some(None)
                } else {
                    None
                }
            }
            _ => None,
        }
    }
    pub(crate) fn global_const(self, name: &str) -> bool {
        self.kind() == "const"
            && self.symbol_child(1) == Some(name)
            && match self.value(0) {
                Some(NodeValue::Nil) => true,
                Some(NodeValue::Node(_)) => self
                    .node_child(0)
                    .is_some_and(|node| node.kind() == "cbase"),
                _ => false,
            }
    }
    pub(crate) fn proc_literal(self) -> bool {
        if self.kind() == "send" {
            return self
                .receiver()
                .is_some_and(|receiver| receiver.global_const("Proc"))
                && self.method_name() == Some("new");
        }
        matches!(self.kind(), "block" | "numblock" | "itblock")
            && self.send_node().is_some_and(|send| {
                send.receiver().is_none() && send.method_name() == Some("proc")
                    || send
                        .receiver()
                        .is_some_and(|receiver| receiver.global_const("Proc"))
                        && send.method_name() == Some("new")
            })
    }
    pub(crate) fn lambda(self) -> bool {
        matches!(self.kind(), "block" | "numblock" | "itblock")
            && self.send_node().is_some_and(|send| {
                send.receiver().is_none() && send.method_name() == Some("lambda")
            })
    }
    pub(crate) fn lambda_or_proc(self) -> bool {
        self.lambda() || self.proc_literal()
    }
    pub(crate) fn class_constructor(self) -> bool {
        let call = if matches!(self.kind(), "block" | "numblock" | "itblock") {
            self.send_node()
        } else {
            Some(self)
        };
        call.is_some_and(|send| {
            send.kind() == "send"
                && match send.method_name() {
                    Some("new") => send.receiver().is_some_and(|receiver| {
                        ["Class", "Module", "Struct"]
                            .iter()
                            .any(|name| receiver.global_const(name))
                    }),
                    Some("define") => send
                        .receiver()
                        .is_some_and(|receiver| receiver.global_const("Data")),
                    _ => false,
                }
        })
    }
    pub(crate) fn new_class_or_module_block(self) -> bool {
        matches!(self.kind(), "block" | "numblock" | "itblock")
            && self.parent().is_some_and(|parent| {
                parent.kind() == "casgn"
                    && parent.node_child(2) == Some(self)
                    && self.send_node().is_some_and(|call| {
                        call.method_name() == Some("new")
                            && call.receiver().is_some_and(|receiver| {
                                receiver.global_const("Class") || receiver.global_const("Module")
                            })
                    })
            })
    }
    pub(crate) fn struct_constructor(self) -> bool {
        matches!(self.kind(), "block" | "numblock" | "itblock")
            && self.send_node().is_some_and(|send| {
                send.method_name() == Some("new")
                    && send
                        .receiver()
                        .is_some_and(|receiver| receiver.global_const("Struct"))
            })
    }
    pub(crate) fn class_definition(self) -> Option<Self> {
        match self.kind() {
            "class" => self.node_child(2),
            "sclass" => self.node_child(1),
            "block" | "numblock" | "itblock"
                if self.send_node().is_some_and(|send| {
                    send.method_name() == Some("new")
                        && send.receiver().is_some_and(|receiver| {
                            receiver.global_const("Struct") || receiver.global_const("Class")
                        })
                }) =>
            {
                self.body()
            }
            _ => None,
        }
    }
    pub(crate) fn module_definition(self) -> Option<Self> {
        match self.kind() {
            "module" => self.node_child(1),
            "block" | "numblock" | "itblock"
                if self.send_node().is_some_and(|send| {
                    send.method_name() == Some("new")
                        && send
                            .receiver()
                            .is_some_and(|receiver| receiver.global_const("Module"))
                }) =>
            {
                self.body()
            }
            _ => None,
        }
    }
    pub(crate) fn guard_clause(self) -> bool {
        let node = if self.operator_keyword() {
            self.rhs().unwrap_or(self)
        } else {
            self
        };
        (matches!(node.kind(), "return" | "break" | "next")
            || node.kind() == "send"
                && node.receiver().is_none()
                && matches!(node.method_name(), Some("raise" | "fail")))
            && node.single_line()
    }
    pub(crate) fn pure(self) -> bool {
        match self.kind() {
            "__FILE__" | "__LINE__" | "const" | "cvar" | "defined?" | "false" | "float"
            | "gvar" | "int" | "ivar" | "lvar" | "nil" | "str" | "sym" | "true" | "regopt" => true,
            "and" | "array" | "begin" | "case" | "dstr" | "dsym" | "eflipflop" | "ensure"
            | "erange" | "for" | "hash" | "if" | "iflipflop" | "irange" | "kwbegin" | "not"
            | "or" | "pair" | "regexp" | "until" | "until_post" | "when" | "while"
            | "while_post" => self.child_nodes().into_iter().all(Self::pure),
            _ => false,
        }
    }
    pub(crate) fn value_used(self) -> bool {
        let Some(parent) = self.parent() else {
            return false;
        };
        match parent.kind() {
            "array" | "defined?" | "dstr" | "dsym" | "eflipflop" | "erange" | "float" | "hash"
            | "iflipflop" | "irange" | "not" | "pair" | "regexp" | "str" | "sym" | "when"
            | "xstr" => parent.value_used(),
            "begin" | "kwbegin" => {
                self.sibling_index() == Some(parent.children().len() - 1) && parent.value_used()
            }
            "for" => {
                if self.sibling_index() == Some(2) {
                    parent.value_used()
                } else {
                    true
                }
            }
            "case" | "if" => self.sibling_index() == Some(0) || parent.value_used(),
            "while" | "until" | "while_post" | "until_post" => self.sibling_index() == Some(0),
            _ => true,
        }
    }
    pub(crate) fn node_child(self, index: usize) -> Option<Self> {
        self.data()
            .children
            .get(index)
            .and_then(|value| node_value(self.ast, value))
    }
    pub(crate) fn symbol_child(self, index: usize) -> Option<&'ast str> {
        match self.data().children.get(index) {
            Some(NodeValue::Symbol(value)) => Some(value),
            _ => None,
        }
    }
    pub(crate) fn string_child(self, index: usize) -> Option<&'ast str> {
        match self.data().children.get(index) {
            Some(NodeValue::String(value)) => Some(value),
            _ => None,
        }
    }
    pub(crate) fn integer_child(self, index: usize) -> Option<i64> {
        match self.data().children.get(index) {
            Some(NodeValue::Integer(value)) => Some(*value),
            _ => None,
        }
    }
    pub(crate) fn value(self, index: usize) -> Option<&'ast NodeValue> {
        self.data().children.get(index)
    }
    pub(crate) fn last_node_child(self) -> Option<Self> {
        self.data()
            .children
            .iter()
            .rev()
            .find_map(|value| node_value(self.ast, value))
    }
}

impl PartialEq for NodeRef<'_> {
    fn eq(&self, other: &Self) -> bool {
        std::ptr::eq(self.ast, other.ast) && self.id == other.id
    }
}
impl Eq for NodeRef<'_> {}
fn node_value<'a>(ast: &'a Ast, value: &NodeValue) -> Option<NodeRef<'a>> {
    if let NodeValue::Node(id) = value {
        Some(NodeRef { ast, id: *id })
    } else {
        None
    }
}
fn char_slice(source: &str, range: Range<usize>) -> Option<&str> {
    let start = source
        .char_indices()
        .nth(range.start)
        .map_or(source.len(), |(index, _)| index);
    let end = source
        .char_indices()
        .nth(range.end)
        .map_or(source.len(), |(index, _)| index);
    source.get(start..end)
}
fn line_at(source: &str, position: usize) -> usize {
    source
        .chars()
        .take(position)
        .filter(|character| *character == '\n')
        .count()
        + 1
}
fn column_at(source: &str, position: usize) -> usize {
    source.chars().take(position).fold(
        0,
        |column, character| if character == '\n' { 0 } else { column + 1 },
    )
}
fn group_for_type(kind: &str) -> Option<&'static str> {
    Some(match kind {
        "def" | "defs" => "any_def",
        "arg" | "optarg" | "restarg" | "kwarg" | "kwoptarg" | "kwrestarg" | "blockarg"
        | "forward_arg" | "shadowarg" => "argument",
        "true" | "false" => "boolean",
        "int" | "float" | "rational" | "complex" => "numeric",
        "str" | "dstr" | "xstr" => "any_str",
        "sym" | "dsym" => "any_sym",
        "irange" | "erange" => "range",
        "send" | "csend" => "call",
        "block" | "numblock" | "itblock" => "any_block",
        "match_pattern" | "match_pattern_p" => "any_match_pattern",
        _ => return None,
    })
}
const TRUTHY_LITERALS: &[&str] = &[
    "str", "dstr", "xstr", "int", "float", "sym", "dsym", "array", "hash", "regexp", "true",
    "irange", "erange", "complex", "rational", "regopt",
];
const LITERALS: &[&str] = &[
    "str", "dstr", "xstr", "int", "float", "sym", "dsym", "array", "hash", "regexp", "true",
    "irange", "erange", "complex", "rational", "regopt", "false", "nil",
];
const COMPOSITE_LITERALS: &[&str] = &[
    "dstr", "xstr", "dsym", "array", "hash", "irange", "erange", "regexp",
];
const MUTABLE_LITERALS: &[&str] = &[
    "str", "dstr", "xstr", "array", "hash", "regexp", "irange", "erange",
];
const KEYWORDS: &[&str] = &[
    "alias", "and", "break", "case", "class", "def", "defs", "defined?", "kwbegin", "do", "else",
    "ensure", "for", "if", "module", "next", "not", "or", "postexe", "redo", "rescue", "retry",
    "return", "self", "super", "zsuper", "then", "undef", "until", "when", "while", "yield",
];
