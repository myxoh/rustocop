// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/node/mixin/basic_literal_node.rb
// Source SHA-256: 9fab7928d9a744735dd180f2da2c0f5c83ea8ab27dc19b5d95c98502b42999e4
// Source: lib/rubocop/ast/node/mixin/binary_operator_node.rb
// Source SHA-256: 0a97df29c7fdabfbfcb63efec2eac6d7947e48ce3459c60af33bf803977df1bf
// Source: lib/rubocop/ast/node/mixin/conditional_node.rb
// Source SHA-256: 1acd6ed5bfcab155163b357d14ff13879597872e43da81787db6597b1a463179
// Source: lib/rubocop/ast/node/mixin/constant_node.rb
// Source SHA-256: 6e4c39ecce59138d964ddcf73be4b0768993173dc01658d5ca89becdd28a5fab
// Source: lib/rubocop/ast/node/mixin/modifier_node.rb
// Source SHA-256: eeaa25284aaf5111335014dd8fe99e97e6989bf1060e25326424d4af08298ac2
// Source: lib/rubocop/ast/node/mixin/numeric_node.rb
// Source SHA-256: b2c24f2c025292e028e866515d450b30022f00def433f91428c3b8d2f43b6cb4
// Source: lib/rubocop/ast/node/mixin/parameterized_node.rb
// Source SHA-256: 996f74f46cf9a70a118d78642932ec33a5edc3bd567b5ba0e87cbd0b7a14f392
// Source: lib/rubocop/ast/node/mixin/predicate_operator_node.rb
// Source SHA-256: 61291696f6d3737aaf6e0c52928a179ad01ac401008b94b8dcc768bad07449f1

pub(crate) fn basic_literal_value<T: Copy>(node_parts: &[T]) -> Option<T> {
    node_parts.first().copied()
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum BinaryCondition<T> {
    Atom(T),
    Begin(Box<Self>),
    Operator(Box<Self>, Box<Self>),
}

impl<T> BinaryCondition<T> {
    pub(crate) fn conditions(&self) -> Vec<&T> {
        let mut conditions = Vec::new();
        self.collect_conditions(&mut conditions);
        conditions
    }

    fn collect_conditions<'node>(&'node self, conditions: &mut Vec<&'node T>) {
        match self {
            Self::Atom(value) => conditions.push(value),
            Self::Begin(value) => value.collect_conditions(conditions),
            Self::Operator(left, right) => {
                left.collect_conditions(conditions);
                right.collect_conditions(conditions);
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct ConditionalLines {
    pub(crate) keyword_line: usize,
    pub(crate) condition_line: usize,
}

impl ConditionalLines {
    pub(crate) fn single_line_condition(self) -> bool {
        self.keyword_line == self.condition_line
    }

    pub(crate) fn multiline_condition(self) -> bool {
        !self.single_line_condition()
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ConstantPath<'name> {
    absolute: bool,
    segments: Vec<&'name str>,
}

impl<'name> ConstantPath<'name> {
    pub(crate) fn new(absolute: bool, segments: Vec<&'name str>) -> Self {
        Self { absolute, segments }
    }

    pub(crate) fn namespace(&self) -> Option<String> {
        (self.segments.len() > 1).then(|| {
            let prefix = self.segments[..self.segments.len() - 1].join("::");
            if self.absolute {
                format!("::{prefix}")
            } else {
                prefix
            }
        })
    }

    pub(crate) fn short_name(&self) -> Option<&'name str> {
        self.segments.last().copied()
    }

    pub(crate) fn module_name(&self) -> bool {
        self.short_name()
            .is_some_and(|name| name.chars().any(|character| character.is_lowercase()))
    }

    pub(crate) fn class_name(&self) -> bool {
        self.module_name()
    }

    pub(crate) fn absolute(&self) -> bool {
        self.absolute && !self.segments.is_empty()
    }

    pub(crate) fn relative(&self) -> bool {
        !self.absolute()
    }

    pub(crate) fn each_path(&self) -> Vec<String> {
        let mut paths = Vec::with_capacity(self.segments.len() + usize::from(self.absolute));
        if self.absolute {
            paths.push("::".to_owned());
        }
        for end in 1..self.segments.len() {
            let prefix = self.segments[..end].join("::");
            paths.push(if self.absolute {
                format!("::{prefix}")
            } else {
                prefix
            });
        }
        paths
    }
}

pub(crate) fn modifier_form(has_end_location: bool) -> bool {
    !has_end_location
}

pub(crate) fn numeric_has_sign(source: &str) -> bool {
    source
        .chars()
        .next()
        .is_some_and(|character| matches!(character, '+' | '-'))
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArgumentKind {
    Splat,
    RestArgument,
    BlockPass,
    BlockArgument,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Parameterized<T> {
    arguments: Vec<(ArgumentKind, T)>,
    closing_delimiter: Option<char>,
}

impl<T> Parameterized<T> {
    pub(crate) fn new(arguments: Vec<(ArgumentKind, T)>, closing_delimiter: Option<char>) -> Self {
        Self {
            arguments,
            closing_delimiter,
        }
    }

    pub(crate) fn parenthesized(&self) -> bool {
        self.closing_delimiter == Some(')')
    }

    pub(crate) fn arguments(&self) -> &[(ArgumentKind, T)] {
        &self.arguments
    }

    pub(crate) fn first_argument(&self) -> Option<&T> {
        self.arguments.first().map(|(_, value)| value)
    }

    pub(crate) fn last_argument(&self) -> Option<&T> {
        self.arguments.last().map(|(_, value)| value)
    }

    pub(crate) fn has_arguments(&self) -> bool {
        !self.arguments.is_empty()
    }

    pub(crate) fn splat_argument(&self) -> bool {
        self.arguments
            .iter()
            .any(|(kind, _)| matches!(kind, ArgumentKind::Splat | ArgumentKind::RestArgument))
    }

    pub(crate) fn rest_argument(&self) -> bool {
        self.splat_argument()
    }

    pub(crate) fn block_argument(&self) -> bool {
        self.arguments.last().is_some_and(|(kind, _)| {
            matches!(kind, ArgumentKind::BlockPass | ArgumentKind::BlockArgument)
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum PredicateOperator {
    LogicalAnd,
    SemanticAnd,
    LogicalOr,
    SemanticOr,
}

impl PredicateOperator {
    pub(crate) fn operator(self) -> &'static str {
        match self {
            Self::LogicalAnd => "&&",
            Self::SemanticAnd => "and",
            Self::LogicalOr => "||",
            Self::SemanticOr => "or",
        }
    }

    pub(crate) fn logical(self) -> bool {
        matches!(self, Self::LogicalAnd | Self::LogicalOr)
    }

    pub(crate) fn semantic(self) -> bool {
        matches!(self, Self::SemanticAnd | Self::SemanticOr)
    }
}
