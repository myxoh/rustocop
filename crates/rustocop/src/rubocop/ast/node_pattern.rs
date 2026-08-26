#![allow(clippy::too_many_lines)]
// rubocop-ast 1.49.1 NodePattern compatibility.
// Source: lib/rubocop/ast/node_pattern.rb
// Source SHA-256: 1cd2bb1dc8484a40492370ca47aebb965acac325dab66cfbb6aed27037240d3f
// Source: lib/rubocop/ast/node_pattern/builder.rb
// Source SHA-256: ae7e2d70526ba5a5c663d022de02ef647fa654c648dccef421c0c53acecc9ebc
// Source: lib/rubocop/ast/node_pattern/comment.rb
// Source SHA-256: dc5c742be96778dd8b506bb83a098a99ef54412f5b479a56273646737e7c288f
// Source: lib/rubocop/ast/node_pattern/compiler.rb
// Source SHA-256: 0ad2c839a5ce4ece480d97513fc1df55d2adf99f9b6e3919fae2bea183e783e1
// Source: lib/rubocop/ast/node_pattern/compiler/atom_subcompiler.rb
// Source SHA-256: 8ff7b6ea8ff9bccf6ac15076b52c49705eff021c4b68db569c1dcc224b423366
// Source: lib/rubocop/ast/node_pattern/compiler/binding.rb
// Source SHA-256: 057bb5ac8566ee91d646d044a17cb52542549a44c8c6c28b7a08096adc497949
// Source: lib/rubocop/ast/node_pattern/compiler/debug.rb
// Source SHA-256: 5b8b63c105124b07e20bf38198e8fe6a5ddaaf83a35a42cc100da936c4a9a253
// Source: lib/rubocop/ast/node_pattern/compiler/node_pattern_subcompiler.rb
// Source SHA-256: b7710c570754542b1de10b985bf28a65f1dfaa1e687bdce2b9b130753f6678f9
// Source: lib/rubocop/ast/node_pattern/compiler/sequence_subcompiler.rb
// Source SHA-256: a0d6239ae88b86e624c5ea3eb28d33d69258adfaa4828d5c444dcc151eccb2dc
// Source: lib/rubocop/ast/node_pattern/compiler/subcompiler.rb
// Source SHA-256: 60f6c4f97f00673a3d6279e8a48d3928e69bb9f906de07668e77f8177b509f56
// Source: lib/rubocop/ast/node_pattern/lexer.rb
// Source SHA-256: f2b9e34bb2e3f63cac678499e99d4ab106b4cf4ae029eb93e13a115a2d78979c
// Source: lib/rubocop/ast/node_pattern/lexer.rex.rb
// Source SHA-256: 9982d0f3af4057657ea57e31c5c21de1bf46b2e45637d1bd13608d128a1b69e6
// Source: lib/rubocop/ast/node_pattern/method_definer.rb
// Source SHA-256: 0c8e1931b95945abcbca56cc38897877e79fe66a3cf6b3828ce102d9f4dc3861
// Source: lib/rubocop/ast/node_pattern/node.rb
// Source SHA-256: eec9b586ac12e0688d8fca787a7898f5728f40633b154edc64b565c7860f457b
// Source: lib/rubocop/ast/node_pattern/parser.racc.rb
// Source SHA-256: 5fdbc06a741b3b58f5191a4c07ce41a86094c55ef7b699f6848b4ed3bdaf34ab
// Source: lib/rubocop/ast/node_pattern/parser.rb
// Source SHA-256: 322580bc34966e915ad518b799fd192a115723c8d319d23f89115d69e71fda81
// Source: lib/rubocop/ast/node_pattern/sets.rb
// Source SHA-256: 7a4410797166348f3b66416dc176e6475774c6e7b15a009fcb087b08ec74b9ac
// Source: lib/rubocop/ast/node_pattern/with_meta.rb
// Source SHA-256: 9156f80feba8ae69b49e15d4d17e1a9e74590adeb753dc6cad884ad5e691662a

use std::collections::{HashMap, HashSet};
use std::fmt;
use std::ops::Range;

use regex::Regex;

use super::node::core::{NodeRef, NodeValue};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PatternError {
    pub(crate) message: String,
    pub(crate) position: usize,
}

impl fmt::Display for PatternError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} at {}", self.message, self.position)
    }
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expr {
    Sequence(Vec<Expr>),
    Intersection(Vec<Expr>),
    Union(Vec<Vec<Expr>>),
    AnyOrder(Vec<Expr>, bool, bool),
    Negation(Box<Expr>),
    Ascend(Box<Expr>),
    Descend(Box<Expr>),
    Capture(Box<Expr>),
    Repetition(Box<Expr>, Repeat),
    Rest(bool),
    Function(String, Vec<Expr>),
    Predicate(String, Vec<Expr>),
    NodeType(String),
    Symbol(String),
    Number(String),
    String(String),
    Regexp(String, String),
    Constant(String),
    NamedParameter(String),
    PositionalParameter(usize),
    Wildcard,
    Unify(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum Repeat {
    ZeroOrOne,
    ZeroOrMore,
    OneOrMore,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TokenKind {
    Punctuation(char),
    Rest,
    Symbol(String),
    Number(String),
    String(String),
    Regexp(String, String),
    Constant(String),
    NamedParameter(String),
    PositionalParameter(usize),
    Function(String),
    Predicate(String),
    NodeType(String),
    Wildcard,
    Unify(String),
    ArgumentList,
    Comment(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) range: Range<usize>,
}

#[allow(clippy::cognitive_complexity)] // Kept linear to mirror NodePattern's token precedence.
pub(crate) fn lex(source: &str) -> Result<Vec<Token>, PatternError> {
    let bytes = source.as_bytes();
    let mut tokens = Vec::new();
    let mut index = 0;
    let mut function_may_take_args = false;
    while index < bytes.len() {
        let byte = bytes[index];
        if byte.is_ascii_whitespace() {
            index += 1;
            function_may_take_args = false;
            continue;
        }
        if byte == b'#' {
            if index + 1 < bytes.len() && is_call_start(bytes[index + 1]) {
                let start = index;
                index += 1;
                let name_start = index;
                while index < bytes.len() && is_call_char(bytes[index]) {
                    index += 1;
                }
                let name = source[name_start..index].to_string();
                tokens.push(Token {
                    kind: TokenKind::Function(name),
                    range: start..index,
                });
                function_may_take_args = true;
            } else {
                let start = index;
                while index < bytes.len() && bytes[index] != b'\n' {
                    index += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Comment(source[start + 1..index].to_string()),
                    range: start..index,
                });
                function_may_take_args = false;
            }
            continue;
        }
        if byte == b'(' && function_may_take_args {
            tokens.push(Token {
                kind: TokenKind::ArgumentList,
                range: index..index + 1,
            });
            index += 1;
            function_may_take_args = false;
            continue;
        }
        function_may_take_args = false;
        if source[index..].starts_with("...") {
            tokens.push(Token {
                kind: TokenKind::Rest,
                range: index..index + 3,
            });
            index += 3;
            continue;
        }
        if b"(){}|[]<>$!^`+*?,".contains(&byte) {
            tokens.push(Token {
                kind: TokenKind::Punctuation(byte as char),
                range: index..index + 1,
            });
            index += 1;
            continue;
        }
        if source[index..].starts_with("::") {
            let start = index;
            index += 2;
            while index < bytes.len() && is_constant_char(bytes[index]) {
                index += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Constant(source[start..index].into()),
                range: start..index,
            });
            continue;
        }
        if byte == b':' {
            let start = index;
            index += 1;
            let value_start = index;
            while index < bytes.len() && is_symbol_char(bytes[index]) {
                index += 1;
            }
            if value_start == index {
                return Err(error("expected symbol name", start));
            }
            tokens.push(Token {
                kind: TokenKind::Symbol(source[value_start..index].to_string()),
                range: start..index,
            });
            continue;
        }
        if byte == b'"' {
            let start = index;
            index += 1;
            let mut value = String::new();
            let mut closed = false;
            while index < bytes.len() {
                match bytes[index] {
                    b'"' => {
                        index += 1;
                        closed = true;
                        break;
                    }
                    b'\\' if index + 1 < bytes.len() => {
                        index += 1;
                        value.push(bytes[index] as char);
                        index += 1
                    }
                    other => {
                        value.push(other as char);
                        index += 1
                    }
                }
            }
            if !closed {
                return Err(error("unterminated string", start));
            }
            tokens.push(Token {
                kind: TokenKind::String(value),
                range: start..index,
            });
            continue;
        }
        if byte == b'/' {
            let start = index;
            index += 1;
            let body_start = index;
            let mut escaped = false;
            while index < bytes.len() {
                if !escaped && bytes[index] == b'/' {
                    break;
                }
                escaped = bytes[index] == b'\\' && !escaped;
                index += 1;
            }
            if index >= bytes.len() || escaped {
                return Err(error("unterminated regexp", start));
            }
            let body = source[body_start..index].to_string();
            index += 1;
            let option_start = index;
            while index < bytes.len() && b"imxo".contains(&bytes[index]) {
                index += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Regexp(body, source[option_start..index].to_string()),
                range: start..index,
            });
            continue;
        }
        if byte == b'%' {
            let start = index;
            index += 1;
            if index < bytes.len() && bytes[index].is_ascii_digit() {
                let digit_start = index;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
                let number = source[digit_start..index].parse().unwrap_or(1);
                tokens.push(Token {
                    kind: TokenKind::PositionalParameter(number),
                    range: start..index,
                });
            } else if index < bytes.len()
                && (bytes[index].is_ascii_lowercase() || bytes[index] == b'_')
            {
                let name_start = index;
                while index < bytes.len()
                    && (bytes[index].is_ascii_lowercase()
                        || bytes[index].is_ascii_digit()
                        || bytes[index] == b'_')
                {
                    index += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::NamedParameter(source[name_start..index].into()),
                    range: start..index,
                });
            } else if index < bytes.len() && is_constant_start(bytes[index]) {
                let name_start = index;
                while index < bytes.len() && is_constant_char(bytes[index]) {
                    index += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Constant(source[name_start..index].into()),
                    range: start..index,
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::PositionalParameter(1),
                    range: start..index,
                });
            }
            continue;
        }
        if byte == b'_' {
            let start = index;
            index += 1;
            if index < bytes.len() && bytes[index].is_ascii_lowercase() {
                let name_start = index;
                while index < bytes.len() && is_identifier_char(bytes[index]) {
                    index += 1;
                }
                tokens.push(Token {
                    kind: TokenKind::Unify(source[name_start..index].into()),
                    range: start..index,
                });
            } else {
                tokens.push(Token {
                    kind: TokenKind::Wildcard,
                    range: start..index,
                });
            }
            continue;
        }
        if byte == b'-' || byte == b'+' || byte.is_ascii_digit() {
            let start = index;
            if matches!(byte, b'-' | b'+') {
                index += 1;
            }
            let digits = index;
            while index < bytes.len() && bytes[index].is_ascii_digit() {
                index += 1;
            }
            if index < bytes.len() && bytes[index] == b'.' {
                index += 1;
                while index < bytes.len() && bytes[index].is_ascii_digit() {
                    index += 1;
                }
            }
            if digits < index && source[start..index].chars().any(|c| c.is_ascii_digit()) {
                tokens.push(Token {
                    kind: TokenKind::Number(source[start..index].into()),
                    range: start..index,
                });
                continue;
            }
            index = start;
        }
        if is_constant_start(byte) {
            let start = index;
            while index < bytes.len() && is_constant_char(bytes[index]) {
                index += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Constant(source[start..index].into()),
                range: start..index,
            });
            continue;
        }
        if byte.is_ascii_lowercase() {
            let start = index;
            while index < bytes.len() && (is_identifier_char(bytes[index]) || bytes[index] == b'-')
            {
                index += 1;
            }
            let mut name = source[start..index].to_string();
            if index < bytes.len() && bytes[index] == b'?' {
                index += 1;
                name.push('?');
                tokens.push(Token {
                    kind: TokenKind::Predicate(name),
                    range: start..index,
                });
                function_may_take_args = true;
            } else {
                tokens.push(Token {
                    kind: TokenKind::NodeType(name),
                    range: start..index,
                });
            }
            continue;
        }
        return Err(error("unexpected character", index));
    }
    Ok(tokens
        .into_iter()
        .filter(|token| !matches!(token.kind, TokenKind::Comment(_)))
        .collect())
}

fn is_call_start(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_uppercase() || byte == b':'
}
fn is_call_char(byte: u8) -> bool {
    is_identifier_char(byte)
        || byte.is_ascii_uppercase()
        || matches!(byte, b':' | b'.' | b'?' | b'!')
}
fn is_identifier_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || byte == b'_'
}
fn is_constant_start(byte: u8) -> bool {
    byte.is_ascii_uppercase() || byte == b':'
}
fn is_constant_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b':')
}
fn is_symbol_char(byte: u8) -> bool {
    is_identifier_char(byte) || b"+@*/?!<>=~|%^&-[]".contains(&byte)
}
fn error(message: &str, position: usize) -> PatternError {
    PatternError {
        message: message.into(),
        position,
    }
}

pub(crate) fn stable_set_name(values: &[String]) -> String {
    let mut elements = values.to_vec();
    elements.sort();
    elements.dedup();
    if elements.len() > 4 {
        elements.truncate(3);
        elements.push("etc".into());
    }
    let name = elements
        .join("_")
        .to_ascii_uppercase()
        .chars()
        .filter(|character| character.is_ascii_alphanumeric() || *character == '_')
        .collect::<String>();
    format!("SET_{name}")
}

#[derive(Default)]
pub(crate) struct SetRegistry {
    names: HashMap<Vec<String>, String>,
    used: HashSet<String>,
}

impl SetRegistry {
    pub(crate) fn name_for(&mut self, values: &[String]) -> String {
        let mut key = values.to_vec();
        key.sort();
        key.dedup();
        if let Some(name) = self.names.get(&key) {
            return name.clone();
        }
        let base = stable_set_name(&key);
        let mut name = base.clone();
        let mut suffix = 2;
        while self.used.contains(&name) {
            name = format!("{base}_{suffix}");
            suffix += 1;
        }
        self.used.insert(name.clone());
        self.names.insert(key, name.clone());
        name
    }
}

pub(crate) fn parse_expression(source: &str) -> Result<Expr, PatternError> {
    let tokens = lex(source)?;
    let mut parser = Parser { tokens, index: 0 };
    let expression = parser.parse_pattern()?;
    if parser.peek().is_some() {
        return Err(error("unexpected trailing token", parser.position()));
    }
    Ok(expression)
}

#[derive(Clone, Debug)]
pub(crate) struct NodePattern {
    pattern: String,
    expression: Expr,
}

impl NodePattern {
    pub(crate) fn initialize(pattern: impl Into<String>) -> Result<Self, PatternError> {
        Self::new(pattern)
    }

    pub(crate) fn def_node_matcher(&self) -> NodeMatcher {
        NodeMatcher {
            pattern: self.clone(),
        }
    }

    pub(crate) fn new(pattern: impl Into<String>) -> Result<Self, PatternError> {
        let pattern = pattern.into();
        let expression = parse_expression(&pattern)?;
        validate(&expression)?;
        Ok(Self {
            pattern,
            expression,
        })
    }
    pub(crate) fn pattern(&self) -> &str {
        &self.pattern
    }
    pub(crate) fn ast(&self) -> &Expr {
        &self.expression
    }
    pub(crate) fn match_code(&self) -> &Expr {
        // The Ruby implementation exposes generated Ruby source. The Rust
        // executable equivalent is the validated expression consumed directly
        // by the matcher, avoiding an eval/code-generation stage.
        &self.expression
    }
    pub(crate) fn equivalent(&self, other: &Self) -> bool {
        self == other
    }
    pub(crate) fn serialized_pattern(&self) -> &str {
        &self.pattern
    }
    pub(crate) fn from_serialized_pattern(
        pattern: impl Into<String>,
    ) -> Result<Self, PatternError> {
        Self::new(pattern)
    }
    pub(crate) fn freeze(&self) -> &Self {
        self
    }
    pub(crate) fn description(&self) -> String {
        format!("#<RuboCop::AST::NodePattern {}>", self.pattern)
    }
    pub(crate) fn expression(&self) -> &Expr {
        &self.expression
    }
    pub(crate) fn captures(&self) -> usize {
        capture_count(&self.expression)
    }
    pub(crate) fn named_parameters(&self) -> Vec<&str> {
        let mut named = Vec::new();
        collect_parameters(&self.expression, &mut named, &mut Vec::new());
        named.sort_unstable();
        named.dedup();
        named
    }
    pub(crate) fn positional_parameters(&self) -> Vec<usize> {
        let mut positional = Vec::new();
        collect_parameters(&self.expression, &mut Vec::new(), &mut positional);
        positional.sort_unstable();
        positional.dedup();
        positional
    }
    pub(crate) fn matches<'ast>(&self, node: NodeRef<'ast>) -> Option<Vec<MatchValue<'ast>>> {
        self.matches_with(node, &MatchContext::default())
    }
    pub(crate) fn matches_with<'ast>(
        &self,
        node: NodeRef<'ast>,
        context: &MatchContext<'ast>,
    ) -> Option<Vec<MatchValue<'ast>>> {
        let mut state = MatchState {
            target: Some(node),
            ..MatchState::default()
        };
        match_expr(
            &self.expression,
            MatchValue::Node(node),
            context,
            &mut state,
        )
        .then_some(state.captures)
    }
    pub(crate) fn search<'ast>(
        &self,
        node: NodeRef<'ast>,
    ) -> Vec<(NodeRef<'ast>, Vec<MatchValue<'ast>>)> {
        node.each_node(&[])
            .into_iter()
            .filter_map(|candidate| {
                self.matches(candidate)
                    .map(|captures| (candidate, captures))
            })
            .collect()
    }
    pub(crate) fn search_with<'ast>(
        &self,
        node: NodeRef<'ast>,
        context: &MatchContext<'ast>,
    ) -> Vec<(NodeRef<'ast>, Vec<MatchValue<'ast>>)> {
        node.each_node(&[])
            .into_iter()
            .filter_map(|candidate| {
                self.matches_with(candidate, context)
                    .map(|captures| (candidate, captures))
            })
            .collect()
    }
    pub(crate) fn match_result<'ast>(&self, node: NodeRef<'ast>) -> Option<MatchResult<'ast>> {
        self.matches(node).map(|mut captures| match captures.len() {
            0 => MatchResult::Matched,
            1 => MatchResult::Capture(captures.pop().unwrap()),
            _ => MatchResult::Captures(captures),
        })
    }
    pub(crate) fn descend<'ast>(node: NodeRef<'ast>) -> Vec<MatchValue<'ast>> {
        fn visit<'ast>(value: MatchValue<'ast>, output: &mut Vec<MatchValue<'ast>>) {
            output.push(value.clone());
            if let MatchValue::Node(node) = value {
                for child in node.children() {
                    visit(MatchValue::from_node_value(node, child), output);
                }
            }
        }
        let mut output = Vec::new();
        visit(MatchValue::Node(node), &mut output);
        output
    }
}

#[derive(Clone, Debug)]
pub(crate) struct NodeMatcher {
    pattern: NodePattern,
}

impl NodeMatcher {
    pub(crate) fn call<'ast>(&self, node: NodeRef<'ast>) -> Option<Vec<MatchValue<'ast>>> {
        self.pattern.matches(node)
    }
}

impl fmt::Display for NodePattern {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.description())
    }
}

impl PartialEq for NodePattern {
    fn eq(&self, other: &Self) -> bool {
        self.expression == other.expression
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MatchResult<'ast> {
    Matched,
    Capture(MatchValue<'ast>),
    Captures(Vec<MatchValue<'ast>>),
}

struct Parser {
    tokens: Vec<Token>,
    index: usize,
}
impl Parser {
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.index)
    }
    fn position(&self) -> usize {
        self.peek().map_or_else(
            || self.tokens.last().map_or(0, |t| t.range.end),
            |t| t.range.start,
        )
    }
    fn take(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.index).cloned()?;
        self.index += 1;
        Some(token)
    }
    fn punctuation(&mut self, expected: char) -> bool {
        if self
            .peek()
            .is_some_and(|t| t.kind == TokenKind::Punctuation(expected))
        {
            self.index += 1;
            true
        } else {
            false
        }
    }
    fn require_punctuation(&mut self, expected: char) -> Result<(), PatternError> {
        self.punctuation(expected)
            .then_some(())
            .ok_or_else(|| error(&format!("expected {expected}"), self.position()))
    }
    fn parse_pattern(&mut self) -> Result<Expr, PatternError> {
        if self
            .peek()
            .is_some_and(|t| t.kind == TokenKind::Punctuation('{'))
        {
            self.parse_union()
        } else {
            self.parse_no_union()
        }
    }
    fn parse_no_union(&mut self) -> Result<Expr, PatternError> {
        let Some(token) = self.take() else {
            return Err(error("expected pattern", self.position()));
        };
        Ok(match token.kind {
            TokenKind::Punctuation('(') => {
                let mut items = Vec::new();
                loop {
                    if self.punctuation(')') {
                        break;
                    }
                    if self.peek().is_none() {
                        return Err(error("expected )", self.position()));
                    }
                    items.push(self.parse_variadic()?);
                }
                if items.is_empty() {
                    return Err(error("empty sequence", token.range.start));
                }
                Expr::Sequence(items)
            }
            TokenKind::Punctuation('[') => {
                let mut items = Vec::new();
                loop {
                    if self.punctuation(']') {
                        break;
                    }
                    if self.peek().is_none() {
                        return Err(error("expected ]", self.position()));
                    }
                    items.push(self.parse_pattern()?);
                }
                if items.is_empty() {
                    return Err(error("empty intersection", token.range.start));
                }
                Expr::Intersection(items)
            }
            TokenKind::Punctuation('!') => Expr::Negation(Box::new(self.parse_pattern()?)),
            TokenKind::Punctuation('^') => Expr::Ascend(Box::new(self.parse_pattern()?)),
            TokenKind::Punctuation('`') => Expr::Descend(Box::new(self.parse_pattern()?)),
            TokenKind::Punctuation('$') => Expr::Capture(Box::new(self.parse_pattern()?)),
            TokenKind::Function(name) => Expr::Function(name, self.parse_args()?),
            TokenKind::Predicate(name) => Expr::Predicate(name, self.parse_args()?),
            TokenKind::NodeType(name) => Expr::NodeType(name.replace('-', "_")),
            TokenKind::Symbol(value) => Expr::Symbol(value),
            TokenKind::Number(value) => Expr::Number(value),
            TokenKind::String(value) => Expr::String(value),
            TokenKind::Regexp(body, options) => Expr::Regexp(body, options),
            TokenKind::Constant(value) => Expr::Constant(value),
            TokenKind::NamedParameter(value) => Expr::NamedParameter(value),
            TokenKind::PositionalParameter(value) => Expr::PositionalParameter(value),
            TokenKind::Wildcard => Expr::Wildcard,
            TokenKind::Unify(value) => Expr::Unify(value),
            other => {
                return Err(error(
                    &format!("unexpected token {other:?}"),
                    token.range.start,
                ))
            }
        })
    }
    fn parse_variadic(&mut self) -> Result<Expr, PatternError> {
        let captured = self.punctuation('$');
        if self
            .peek()
            .is_some_and(|t| t.kind == TokenKind::Punctuation('<'))
        {
            self.index += 1;
            let mut items = Vec::new();
            let mut rest = false;
            let mut capture_rest = false;
            loop {
                if self.punctuation('>') {
                    break;
                }
                if self
                    .peek()
                    .is_some_and(|token| token.kind == TokenKind::Punctuation('$'))
                    && self
                        .tokens
                        .get(self.index + 1)
                        .is_some_and(|token| matches!(token.kind, TokenKind::Rest))
                {
                    self.index += 2;
                    rest = true;
                    capture_rest = true;
                    self.require_punctuation('>')?;
                    break;
                }
                if matches!(self.peek().map(|t| &t.kind), Some(TokenKind::Rest)) {
                    self.index += 1;
                    rest = true;
                    self.require_punctuation('>')?;
                    break;
                }
                items.push(self.parse_pattern()?);
            }
            let expr = Expr::AnyOrder(items, rest, capture_rest);
            return Ok(if captured {
                Expr::Capture(Box::new(expr))
            } else {
                expr
            });
        }
        let mut expr = if self
            .peek()
            .is_some_and(|t| matches!(t.kind, TokenKind::Rest))
        {
            self.index += 1;
            Expr::Rest(captured)
        } else {
            let base = self.parse_pattern()?;
            if captured {
                Expr::Capture(Box::new(base))
            } else {
                base
            }
        };
        if let Some(repeat) = match self.peek().map(|t| &t.kind) {
            Some(TokenKind::Punctuation('?')) => Some(Repeat::ZeroOrOne),
            Some(TokenKind::Punctuation('*')) => Some(Repeat::ZeroOrMore),
            Some(TokenKind::Punctuation('+')) => Some(Repeat::OneOrMore),
            _ => None,
        } {
            self.index += 1;
            expr = Expr::Repetition(Box::new(expr), repeat);
        }
        Ok(expr)
    }
    fn parse_union(&mut self) -> Result<Expr, PatternError> {
        self.require_punctuation('{')?;
        let mut branches = vec![Vec::new()];
        loop {
            if self.punctuation('}') {
                break;
            }
            if self.punctuation('|') {
                branches.push(Vec::new());
                continue;
            }
            if self.peek().is_none() {
                return Err(error("expected }", self.position()));
            }
            branches.last_mut().unwrap().push(self.parse_variadic()?);
        }
        if branches.len() == 1 {
            return Ok(Expr::Union(
                branches
                    .pop()
                    .unwrap()
                    .into_iter()
                    .map(|item| vec![item])
                    .collect(),
            ));
        }
        Ok(Expr::Union(branches))
    }
    fn parse_args(&mut self) -> Result<Vec<Expr>, PatternError> {
        if !self
            .peek()
            .is_some_and(|t| t.kind == TokenKind::ArgumentList)
        {
            return Ok(Vec::new());
        }
        self.index += 1;
        let mut args = Vec::new();
        if self.punctuation(')') {
            return Err(error("empty argument list", self.position()));
        }
        loop {
            args.push(self.parse_pattern()?);
            if self.punctuation(')') {
                break;
            }
            self.require_punctuation(',')?;
        }
        Ok(args)
    }
}

fn validate(expr: &Expr) -> Result<(), PatternError> {
    match expr {
        Expr::Sequence(items) => {
            if matches!(
                items.first(),
                Some(Expr::Repetition(..) | Expr::AnyOrder(..))
            ) {
                return Err(error("sequence cannot start with variadic term", 0));
            }
            if items
                .iter()
                .filter(|item| matches!(item, Expr::Rest(_)))
                .count()
                > 1
            {
                return Err(error("sequence has multiple rests", 0));
            }
            for item in items {
                validate(item)?;
            }
        }
        Expr::Union(branches) => {
            let captures: Vec<_> = branches
                .iter()
                .map(|branch| branch.iter().map(capture_count).sum::<usize>())
                .collect();
            if captures.windows(2).any(|pair| pair[0] != pair[1]) {
                return Err(error("union branches must capture equally", 0));
            }
            for branch in branches {
                for item in branch {
                    validate(item)?;
                }
            }
        }
        Expr::Intersection(items) | Expr::AnyOrder(items, _, _) => {
            if matches!(expr, Expr::AnyOrder(items, _, _) if items.iter().any(|item| matches!(item, Expr::AnyOrder(..))))
            {
                return Err(error("any-order patterns cannot be nested", 0));
            }
            for item in items {
                validate(item)?
            }
        }
        Expr::Negation(child) => {
            if capture_count(child) > 0 {
                return Err(error("capture in negation", 0));
            }
            validate(child)?
        }
        Expr::Ascend(child)
        | Expr::Descend(child)
        | Expr::Capture(child)
        | Expr::Repetition(child, _) => validate(child)?,
        _ => {}
    }
    Ok(())
}
fn capture_count(expr: &Expr) -> usize {
    match expr {
        Expr::Capture(child) => 1 + capture_count(child),
        Expr::Rest(true) => 1,
        Expr::Sequence(items) | Expr::Intersection(items) => items.iter().map(capture_count).sum(),
        Expr::AnyOrder(items, _, capture_rest) => {
            items.iter().map(capture_count).sum::<usize>() + usize::from(*capture_rest)
        }
        Expr::Union(branches) => branches
            .first()
            .map_or(0, |b| b.iter().map(capture_count).sum()),
        Expr::Negation(child)
        | Expr::Ascend(child)
        | Expr::Descend(child)
        | Expr::Repetition(child, _) => capture_count(child),
        Expr::Function(_, args) | Expr::Predicate(_, args) => args.iter().map(capture_count).sum(),
        _ => 0,
    }
}

fn collect_parameters<'expr>(
    expr: &'expr Expr,
    named: &mut Vec<&'expr str>,
    positional: &mut Vec<usize>,
) {
    match expr {
        Expr::NamedParameter(name) => named.push(name),
        Expr::PositionalParameter(index) => positional.push(*index),
        Expr::Sequence(items) | Expr::Intersection(items) | Expr::AnyOrder(items, _, _) => {
            for item in items {
                collect_parameters(item, named, positional);
            }
        }
        Expr::Union(branches) => {
            for item in branches.iter().flatten() {
                collect_parameters(item, named, positional);
            }
        }
        Expr::Negation(child)
        | Expr::Ascend(child)
        | Expr::Descend(child)
        | Expr::Capture(child)
        | Expr::Repetition(child, _) => collect_parameters(child, named, positional),
        Expr::Function(_, args) | Expr::Predicate(_, args) => {
            for argument in args {
                collect_parameters(argument, named, positional);
            }
        }
        _ => {}
    }
}

#[derive(Clone, Debug)]
pub(crate) enum MatchValue<'ast> {
    Node(NodeRef<'ast>),
    Symbol(&'ast str),
    OwnedSymbol(String),
    String(&'ast str),
    OwnedString(String),
    Integer(i64),
    Boolean(bool),
    Nil,
    Array(Vec<MatchValue<'ast>>),
}
impl PartialEq for MatchValue<'_> {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Node(left), Self::Node(right)) => left.structurally_equal(*right),
            (Self::Symbol(left), Self::Symbol(right))
            | (Self::String(left), Self::String(right)) => left == right,
            (Self::OwnedSymbol(left), Self::OwnedSymbol(right))
            | (Self::OwnedString(left), Self::OwnedString(right)) => left == right,
            (Self::Symbol(left), Self::OwnedSymbol(right))
            | (Self::String(left), Self::OwnedString(right)) => *left == right,
            (Self::OwnedSymbol(left), Self::Symbol(right))
            | (Self::OwnedString(left), Self::String(right)) => left == *right,
            (Self::Integer(left), Self::Integer(right)) => left == right,
            (Self::Integer(left), Self::String(right)) => right.parse::<f64>() == Ok(*left as f64),
            (Self::String(left), Self::Integer(right)) => left.parse::<f64>() == Ok(*right as f64),
            (Self::Boolean(left), Self::Boolean(right)) => left == right,
            (Self::Nil, Self::Nil) => true,
            (Self::Array(left), Self::Array(right)) => left == right,
            _ => false,
        }
    }
}
impl Eq for MatchValue<'_> {}
impl<'ast> MatchValue<'ast> {
    fn from_node_value(node: NodeRef<'ast>, value: &'ast NodeValue) -> Self {
        match value {
            NodeValue::Node(id) => node
                .each_node(&[])
                .into_iter()
                .find(|n| n.id() == *id)
                .map_or(Self::Nil, Self::Node),
            NodeValue::Symbol(v) => Self::Symbol(v),
            NodeValue::String(v) => Self::String(v),
            NodeValue::Integer(v) => Self::Integer(*v),
            NodeValue::Boolean(v) => Self::Boolean(*v),
            NodeValue::Nil => Self::Nil,
        }
    }
}

pub(crate) trait PatternFunctions<'ast> {
    fn call(&self, name: &str, value: MatchValue<'ast>, arguments: &[MatchValue<'ast>]) -> bool;
}
#[derive(Default)]
struct NoFunctions;
impl<'ast> PatternFunctions<'ast> for NoFunctions {
    fn call(&self, _: &str, _: MatchValue<'ast>, _: &[MatchValue<'ast>]) -> bool {
        false
    }
}
#[derive(Default)]
pub(crate) struct MatchContext<'ast> {
    pub(crate) positional: Vec<MatchValue<'ast>>,
    pub(crate) named: HashMap<String, MatchValue<'ast>>,
    pub(crate) constants: HashMap<String, Vec<MatchValue<'ast>>>,
    pub(crate) functions: Option<&'ast dyn PatternFunctions<'ast>>,
}
#[derive(Clone, Default)]
struct MatchState<'ast> {
    captures: Vec<MatchValue<'ast>>,
    unified: HashMap<String, MatchValue<'ast>>,
    target: Option<NodeRef<'ast>>,
}

fn match_expr<'ast>(
    expr: &Expr,
    value: MatchValue<'ast>,
    context: &MatchContext<'ast>,
    state: &mut MatchState<'ast>,
) -> bool {
    match expr {
        Expr::Wildcard => true,
        Expr::NodeType(kind) => matches!(value,MatchValue::Node(node) if node.kind()==kind),
        Expr::Symbol(expected) => matches!(value,MatchValue::Symbol(actual) if actual==expected),
        Expr::String(expected) => matches!(value,MatchValue::String(actual) if actual==expected),
        Expr::Number(expected) => match value {
            MatchValue::Integer(actual) => expected.parse::<i64>() == Ok(actual),
            MatchValue::String(actual) => expected
                .parse::<f64>()
                .is_ok_and(|expected| actual.parse::<f64>() == Ok(expected)),
            _ => false,
        },
        Expr::Regexp(body, options) => {
            let text = match value {
                MatchValue::String(v) | MatchValue::Symbol(v) => v,
                _ => return false,
            };
            let prefix = if options.contains('i') { "(?i)" } else { "" };
            Regex::new(&format!("{prefix}{body}")).is_ok_and(|r| r.is_match(text))
        }
        Expr::Constant(name) => context
            .constants
            .get(name)
            .is_some_and(|values| values.contains(&value)),
        Expr::NamedParameter(name) => context.named.get(name) == Some(&value),
        Expr::PositionalParameter(0) => state
            .target
            .is_some_and(|target| MatchValue::Node(target) == value),
        Expr::PositionalParameter(index) => context.positional.get(index - 1) == Some(&value),
        Expr::Unify(name) => {
            if let Some(bound) = state.unified.get(name) {
                *bound == value
            } else {
                state.unified.insert(name.clone(), value);
                true
            }
        }
        Expr::Capture(child) => {
            let checkpoint = state.clone();
            if match_expr(child, value.clone(), context, state) {
                state.captures.push(value);
                true
            } else {
                *state = checkpoint;
                false
            }
        }
        Expr::Negation(child) => {
            let checkpoint = state.clone();
            let matched = match_expr(child, value, context, state);
            *state = checkpoint;
            !matched
        }
        Expr::Intersection(items) => items
            .iter()
            .all(|item| match_expr(item, value.clone(), context, state)),
        Expr::Union(branches) => branches.iter().any(|branch| {
            let checkpoint = state.clone();
            let ok = branch
                .iter()
                .all(|item| match_expr(item, value.clone(), context, state));
            if !ok {
                *state = checkpoint;
            }
            ok
        }),
        Expr::Ascend(child) => {
            matches!(value,MatchValue::Node(node) if node.parent().is_some_and(|p|match_expr(child,MatchValue::Node(p),context,state)))
        }
        Expr::Descend(child) => {
            matches!(value,MatchValue::Node(node) if node.descendants().into_iter().any(|d|{let checkpoint=state.clone();let ok=match_expr(child,MatchValue::Node(d),context,state);if !ok{*state=checkpoint;}ok}))
        }
        Expr::Sequence(items) => match_sequence(items, value, context, state),
        Expr::Predicate(name, args) => match_predicate(name, args, value, context, state),
        Expr::Function(name, args) => {
            let evaluated = args
                .iter()
                .map(|arg| {
                    literal_value(arg, context, state).unwrap_or_else(|| {
                        let mut argument_state = state.clone();
                        MatchValue::Boolean(match_expr(
                            arg,
                            value.clone(),
                            context,
                            &mut argument_state,
                        ))
                    })
                })
                .collect::<Vec<_>>();
            context
                .functions
                .unwrap_or(&NoFunctions)
                .call(name, value, &evaluated)
        }
        Expr::Repetition(child, _) => match_expr(child, value, context, state),
        Expr::Rest(_) | Expr::AnyOrder(..) => false,
    }
}

fn match_sequence<'ast>(
    items: &[Expr],
    value: MatchValue<'ast>,
    context: &MatchContext<'ast>,
    state: &mut MatchState<'ast>,
) -> bool {
    let MatchValue::Node(node) = value else {
        return false;
    };
    let Some((head, terms)) = items.split_first() else {
        return false;
    };
    if !match_expr(head, value, context, state) {
        return false;
    }
    let children = node
        .children()
        .iter()
        .map(|v| MatchValue::from_node_value(node, v))
        .collect::<Vec<_>>();
    match_terms(terms, &children, 0, 0, context, state)
}
fn match_terms<'ast>(
    terms: &[Expr],
    values: &[MatchValue<'ast>],
    term: usize,
    value: usize,
    context: &MatchContext<'ast>,
    state: &mut MatchState<'ast>,
) -> bool {
    if term == terms.len() {
        return value == values.len();
    }
    match &terms[term] {
        Expr::Rest(capture) => {
            for count in (0..=values.len().saturating_sub(value)).rev() {
                let checkpoint = state.clone();
                if *capture {
                    state
                        .captures
                        .push(MatchValue::Array(values[value..value + count].to_vec()));
                }
                if match_terms(terms, values, term + 1, value + count, context, state) {
                    return true;
                }
                *state = checkpoint;
            }
            false
        }
        Expr::Repetition(child, repeat) => {
            let (min, max) = match repeat {
                Repeat::ZeroOrOne => (0, 1),
                Repeat::ZeroOrMore => (0, values.len() - value),
                Repeat::OneOrMore => (1, values.len() - value),
            };
            for count in (min..=max).rev() {
                let checkpoint = state.clone();
                let capture_start = state.captures.len();
                let captures_per_item = capture_count(child);
                let mut ok = true;
                for item in &values[value..value + count] {
                    if !match_expr(child, item.clone(), context, state) {
                        ok = false;
                        break;
                    }
                }
                if ok && captures_per_item > 0 {
                    let repeated = state.captures.split_off(capture_start);
                    for capture_index in 0..captures_per_item {
                        state.captures.push(MatchValue::Array(
                            repeated
                                .iter()
                                .skip(capture_index)
                                .step_by(captures_per_item)
                                .cloned()
                                .collect(),
                        ));
                    }
                }
                if ok && match_terms(terms, values, term + 1, value + count, context, state) {
                    return true;
                }
                *state = checkpoint;
            }
            false
        }
        Expr::AnyOrder(patterns, has_rest, capture_rest) => match_any_order_term(
            patterns,
            *has_rest,
            *capture_rest,
            false,
            terms,
            values,
            term,
            value,
            context,
            state,
        ),
        Expr::Capture(child) if matches!(child.as_ref(), Expr::AnyOrder(..)) => {
            let Expr::AnyOrder(patterns, has_rest, capture_rest) = child.as_ref() else {
                unreachable!()
            };
            match_any_order_term(
                patterns,
                *has_rest,
                *capture_rest,
                true,
                terms,
                values,
                term,
                value,
                context,
                state,
            )
        }
        current => {
            if value >= values.len() {
                return false;
            }
            let checkpoint = state.clone();
            if match_expr(current, values[value].clone(), context, state)
                && match_terms(terms, values, term + 1, value + 1, context, state)
            {
                true
            } else {
                *state = checkpoint;
                false
            }
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn match_any_order_term<'ast>(
    patterns: &[Expr],
    has_rest: bool,
    capture_rest: bool,
    capture_whole: bool,
    terms: &[Expr],
    values: &[MatchValue<'ast>],
    term: usize,
    value: usize,
    context: &MatchContext<'ast>,
    state: &mut MatchState<'ast>,
) -> bool {
    let available = values.len().saturating_sub(value);
    if available < patterns.len() {
        return false;
    }
    let minimum = patterns.len();
    let maximum = if has_rest { available } else { minimum };
    for length in (minimum..=maximum).rev() {
        let checkpoint = state.clone();
        let segment = &values[value..value + length];
        let mut used = vec![false; segment.len()];
        if match_any_order(patterns, segment, 0, &mut used, context, state) {
            if capture_rest {
                state.captures.push(MatchValue::Array(
                    segment
                        .iter()
                        .zip(&used)
                        .filter(|(_, used)| !*used)
                        .map(|(item, _)| item.clone())
                        .collect(),
                ));
            }
            if capture_whole {
                state.captures.push(MatchValue::Array(segment.to_vec()));
            }
            if match_terms(terms, values, term + 1, value + length, context, state) {
                return true;
            }
        }
        *state = checkpoint;
    }
    false
}
fn match_any_order<'ast>(
    patterns: &[Expr],
    values: &[MatchValue<'ast>],
    index: usize,
    used: &mut [bool],
    context: &MatchContext<'ast>,
    state: &mut MatchState<'ast>,
) -> bool {
    if index == patterns.len() {
        return true;
    }
    for i in 0..values.len() {
        if used[i] {
            continue;
        }
        let checkpoint = state.clone();
        if match_expr(&patterns[index], values[i].clone(), context, state) {
            used[i] = true;
            if match_any_order(patterns, values, index + 1, used, context, state) {
                return true;
            }
            used[i] = false;
        }
        *state = checkpoint;
    }
    false
}
fn match_predicate<'ast>(
    name: &str,
    args: &[Expr],
    value: MatchValue<'ast>,
    context: &MatchContext<'ast>,
    state: &mut MatchState<'ast>,
) -> bool {
    if !args.is_empty() {
        let evaluated = args
            .iter()
            .map(|arg| {
                literal_value(arg, context, state).unwrap_or_else(|| {
                    let mut argument_state = state.clone();
                    MatchValue::Boolean(match_expr(
                        arg,
                        value.clone(),
                        context,
                        &mut argument_state,
                    ))
                })
            })
            .collect::<Vec<_>>();
        if name == "equal?" && evaluated.len() == 1 {
            return value == evaluated[0];
        }
        if name == "between?" && evaluated.len() == 2 {
            return between(&value, &evaluated[0], &evaluated[1]);
        }
        return context
            .functions
            .unwrap_or(&NoFunctions)
            .call(name, value, &evaluated);
    }
    match name {
        "nil?" => value == MatchValue::Nil,
        "node?" => matches!(value, MatchValue::Node(_)),
        "odd?" => matches!(value, MatchValue::Integer(number) if number % 2 != 0),
        "zero?" => matches!(value, MatchValue::Integer(0)),
        "truthy_literal?" => matches!(value,MatchValue::Node(n) if n.truthy_literal()),
        "falsey_literal?" => matches!(value,MatchValue::Node(n) if n.falsey_literal()),
        "literal?" => matches!(value,MatchValue::Node(n) if n.literal()),
        "parenthesized?" => matches!(value,MatchValue::Node(n) if n.parenthesized_call()),
        _ if name.ends_with("_type?") => {
            matches!(value,MatchValue::Node(n) if n.kind()==&name[..name.len()-6])
        }
        _ => {
            let checkpoint = state.clone();
            let result = context
                .functions
                .unwrap_or(&NoFunctions)
                .call(name, value, &[]);
            if !result {
                *state = checkpoint;
            }
            result
        }
    }
}

fn between(value: &MatchValue<'_>, lower: &MatchValue<'_>, upper: &MatchValue<'_>) -> bool {
    match (value, lower, upper) {
        (MatchValue::Integer(value), MatchValue::Integer(lower), MatchValue::Integer(upper)) => {
            lower <= value && value <= upper
        }
        _ => {
            let text = |value: &MatchValue<'_>| match value {
                MatchValue::String(value) | MatchValue::Symbol(value) => Some((*value).to_owned()),
                MatchValue::OwnedString(value) | MatchValue::OwnedSymbol(value) => {
                    Some(value.clone())
                }
                _ => None,
            };
            text(value)
                .zip(text(lower))
                .zip(text(upper))
                .is_some_and(|((value, lower), upper)| lower <= value && value <= upper)
        }
    }
}
fn literal_value<'ast>(
    expr: &Expr,
    context: &MatchContext<'ast>,
    state: &MatchState<'ast>,
) -> Option<MatchValue<'ast>> {
    match expr {
        Expr::PositionalParameter(0) => state.target.map(MatchValue::Node),
        Expr::PositionalParameter(i) => context.positional.get(i - 1).cloned(),
        Expr::NamedParameter(n) => context.named.get(n).cloned(),
        Expr::Constant(n) => context
            .constants
            .get(n)
            .and_then(|values| values.first())
            .cloned(),
        Expr::Symbol(value) => Some(MatchValue::OwnedSymbol(value.clone())),
        Expr::String(value) => Some(MatchValue::OwnedString(value.clone())),
        Expr::Number(value) => value.parse().ok().map(MatchValue::Integer),
        _ => None,
    }
}

#[cfg(test)]
mod spec;
// RuboCop API ownership: lib/rubocop/ast/node_pattern.rb => ast, captures, def_node_matcher, initialize, named_parameters, pattern, positional_parameters
