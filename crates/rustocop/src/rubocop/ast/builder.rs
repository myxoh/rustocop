// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/builder.rb
// Source SHA-256: c66b4e9c5f5b8ddd8af742c3111554b6f4454314351b96bd7818f74a229c1362
// Source: lib/rubocop/ast/sexp.rb
// Source SHA-256: 29ac6a29ca4841112116be055fd1cee993312e778fabbe7751d46056538059c6

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum NodeClass {
    Node,
    Alias,
    And,
    AndAssignment,
    Argument,
    Arguments,
    Array,
    Assignment,
    Block,
    Break,
    CaseMatch,
    ConstantAssignment,
    Case,
    Class,
    Complex,
    Constant,
    Definition,
    Defined,
    DynamicString,
    Ensure,
    For,
    ForwardArguments,
    KeywordSplat,
    Float,
    Hash,
    If,
    InPattern,
    Integer,
    Index,
    IndexAssignment,
    Range,
    KeywordBegin,
    Lambda,
    MultipleAssignment,
    MultipleLhs,
    Module,
    Next,
    OperatorAssignment,
    OrAssignment,
    Or,
    Pair,
    ProcArgumentZero,
    Rational,
    Regexp,
    Rescue,
    RescueBody,
    Return,
    SafeSend,
    Send,
    String,
    SelfClass,
    Super,
    Symbol,
    Until,
    Variable,
    When,
    While,
    Yield,
}

pub(crate) const EMIT_FORWARD_ARG: bool = true;
pub(crate) const EMIT_MATCH_PATTERN: bool = true;

pub(crate) fn builder_features() -> (bool, bool) {
    (EMIT_FORWARD_ARG, EMIT_MATCH_PATTERN)
}

pub(crate) fn node_class(kind: &str) -> NodeClass {
    match kind {
        "alias" => NodeClass::Alias,
        "and" => NodeClass::And,
        "and_asgn" => NodeClass::AndAssignment,
        "arg" | "blockarg" | "forward_arg" | "kwarg" | "kwoptarg" | "kwrestarg" | "optarg"
        | "restarg" | "shadowarg" => NodeClass::Argument,
        "args" => NodeClass::Arguments,
        "array" => NodeClass::Array,
        "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn" => NodeClass::Assignment,
        "block" | "numblock" | "itblock" => NodeClass::Block,
        "break" => NodeClass::Break,
        "case_match" => NodeClass::CaseMatch,
        "casgn" => NodeClass::ConstantAssignment,
        "case" => NodeClass::Case,
        "class" => NodeClass::Class,
        "complex" => NodeClass::Complex,
        "const" => NodeClass::Constant,
        "def" | "defs" => NodeClass::Definition,
        "defined?" => NodeClass::Defined,
        "dstr" => NodeClass::DynamicString,
        "ensure" => NodeClass::Ensure,
        "for" => NodeClass::For,
        "forward_args" => NodeClass::ForwardArguments,
        "forwarded_kwrestarg" | "kwsplat" => NodeClass::KeywordSplat,
        "float" => NodeClass::Float,
        "hash" | "kwargs" => NodeClass::Hash,
        "if" => NodeClass::If,
        "in_pattern" => NodeClass::InPattern,
        "int" => NodeClass::Integer,
        "index" => NodeClass::Index,
        "indexasgn" => NodeClass::IndexAssignment,
        "irange" | "erange" => NodeClass::Range,
        "kwbegin" => NodeClass::KeywordBegin,
        "lambda" => NodeClass::Lambda,
        "masgn" => NodeClass::MultipleAssignment,
        "mlhs" => NodeClass::MultipleLhs,
        "module" => NodeClass::Module,
        "next" => NodeClass::Next,
        "op_asgn" => NodeClass::OperatorAssignment,
        "or_asgn" => NodeClass::OrAssignment,
        "or" => NodeClass::Or,
        "pair" => NodeClass::Pair,
        "procarg0" => NodeClass::ProcArgumentZero,
        "rational" => NodeClass::Rational,
        "regexp" => NodeClass::Regexp,
        "rescue" => NodeClass::Rescue,
        "resbody" => NodeClass::RescueBody,
        "return" => NodeClass::Return,
        "csend" => NodeClass::SafeSend,
        "send" => NodeClass::Send,
        "str" | "xstr" => NodeClass::String,
        "sclass" => NodeClass::SelfClass,
        "super" | "zsuper" => NodeClass::Super,
        "sym" => NodeClass::Symbol,
        "until" | "until_post" => NodeClass::Until,
        "lvar" | "ivar" | "cvar" | "gvar" => NodeClass::Variable,
        "when" => NodeClass::When,
        "while" | "while_post" => NodeClass::While,
        "yield" => NodeClass::Yield,
        _ => NodeClass::Node,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum SexpValue {
    Node(Box<SexpNode>),
    Symbol(String),
    String(String),
    Integer(i64),
    Nil,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct SexpNode {
    pub(crate) kind: String,
    pub(crate) class: NodeClass,
    pub(crate) children: Vec<SexpValue>,
}

pub(crate) fn build_node(kind: &str, children: Vec<SexpValue>) -> SexpNode {
    SexpNode {
        kind: kind.to_owned(),
        class: node_class(kind),
        children,
    }
}

pub(crate) fn s(kind: &str, children: Vec<SexpValue>) -> SexpNode {
    build_node(kind, children)
}

pub(crate) fn string_value(token_value: &[u8]) -> &[u8] {
    token_value
}
