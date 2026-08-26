// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/node/mixin/method_identifier_predicates.rb
// Source SHA-256: 4179be9db5846a3447b49dad3d9a39b8ab170cb8a2b34f86324cb7dfcef1dff9

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ReceiverKind {
    Implicit,
    SelfValue,
    Constant,
    Other,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct MethodIdentifier<'name> {
    method_name: &'name str,
    receiver: ReceiverKind,
    selector: Option<&'name str>,
}

impl<'name> MethodIdentifier<'name> {
    pub(crate) fn new(
        method_name: &'name str,
        receiver: ReceiverKind,
        selector: Option<&'name str>,
    ) -> Self {
        Self {
            method_name,
            receiver,
            selector,
        }
    }

    pub(crate) fn method(&self, name: &str) -> bool {
        self.method_name == name
    }

    pub(crate) fn method_name(&self) -> &'name str {
        self.method_name
    }

    pub(crate) fn receiver(&self) -> ReceiverKind {
        self.receiver
    }

    pub(crate) fn operator_method(&self) -> bool {
        contains(OPERATOR_METHODS, self.method_name)
    }

    pub(crate) fn nonmutating_binary_operator_method(&self) -> bool {
        contains(NONMUTATING_BINARY_OPERATOR_METHODS, self.method_name)
    }

    pub(crate) fn nonmutating_unary_operator_method(&self) -> bool {
        contains(NONMUTATING_UNARY_OPERATOR_METHODS, self.method_name)
    }

    pub(crate) fn nonmutating_operator_method(&self) -> bool {
        self.nonmutating_binary_operator_method() || self.nonmutating_unary_operator_method()
    }

    pub(crate) fn nonmutating_array_method(&self) -> bool {
        contains(NONMUTATING_ARRAY_METHODS, self.method_name)
    }

    pub(crate) fn nonmutating_hash_method(&self) -> bool {
        contains(NONMUTATING_HASH_METHODS, self.method_name)
    }

    pub(crate) fn nonmutating_string_method(&self) -> bool {
        contains(NONMUTATING_STRING_METHODS, self.method_name)
    }

    pub(crate) fn comparison_method(&self) -> bool {
        contains(COMPARISON_OPERATORS, self.method_name)
    }

    pub(crate) fn assignment_method(&self) -> bool {
        !self.comparison_method() && self.method_name.ends_with('=')
    }

    pub(crate) fn enumerator_method(&self) -> bool {
        contains(ENUMERATOR_METHODS, self.method_name) || self.method_name.starts_with("each_")
    }

    pub(crate) fn enumerable_method(&self) -> bool {
        contains(ENUMERABLE_METHODS, self.method_name)
    }

    pub(crate) fn predicate_method(&self) -> bool {
        self.method_name.ends_with('?')
    }

    pub(crate) fn bang_method(&self) -> bool {
        self.method_name.ends_with('!')
    }

    pub(crate) fn camel_case_method(&self) -> bool {
        self.method_name
            .chars()
            .next()
            .is_some_and(|character| character.is_ascii_uppercase())
    }

    pub(crate) fn self_receiver(&self) -> bool {
        self.receiver == ReceiverKind::SelfValue
    }

    pub(crate) fn const_receiver(&self) -> bool {
        self.receiver == ReceiverKind::Constant
    }

    pub(crate) fn negation_method(&self) -> bool {
        self.receiver != ReceiverKind::Implicit && self.method_name == "!"
    }

    pub(crate) fn prefix_not(&self) -> bool {
        self.negation_method() && self.selector == Some("not")
    }

    pub(crate) fn prefix_bang(&self) -> bool {
        self.negation_method() && self.selector == Some("!")
    }
}

fn contains(values: &[&str], method_name: &str) -> bool {
    values.contains(&method_name)
}

const COMPARISON_OPERATORS: &[&str] = &["==", "===", "!=", "<=", ">=", ">", "<"];
const OPERATOR_METHODS: &[&str] = &[
    "|", "^", "&", "<=>", "==", "===", "=~", ">", ">=", "<", "<=", "<<", ">>", "+", "-", "*", "/",
    "%", "**", "~", "+@", "-@", "!@", "~@", "[]", "[]=", "!", "!=", "!~", "`",
];
const NONMUTATING_BINARY_OPERATOR_METHODS: &[&str] = &[
    "*", "/", "%", "+", "-", "==", "===", "!=", "<", ">", "<=", ">=", "<=>",
];
const NONMUTATING_UNARY_OPERATOR_METHODS: &[&str] = &["+@", "-@", "~", "!"];
const ENUMERATOR_METHODS: &[&str] = &[
    "collect",
    "collect_concat",
    "detect",
    "downto",
    "each",
    "find",
    "find_all",
    "find_index",
    "inject",
    "loop",
    "map!",
    "map",
    "reduce",
    "reject",
    "reject!",
    "reverse_each",
    "select",
    "select!",
    "times",
    "upto",
];
const ENUMERABLE_METHODS: &[&str] = &[
    "all?",
    "any?",
    "chain",
    "chunk",
    "chunk_while",
    "collect",
    "collect_concat",
    "compact",
    "count",
    "cycle",
    "detect",
    "drop",
    "drop_while",
    "each",
    "each_cons",
    "each_entry",
    "each_slice",
    "each_with_index",
    "each_with_object",
    "entries",
    "filter",
    "filter_map",
    "find",
    "find_all",
    "find_index",
    "first",
    "flat_map",
    "grep",
    "grep_v",
    "group_by",
    "include?",
    "inject",
    "lazy",
    "map",
    "max",
    "max_by",
    "member?",
    "min",
    "min_by",
    "minmax",
    "minmax_by",
    "none?",
    "one?",
    "partition",
    "reduce",
    "reject",
    "reverse_each",
    "select",
    "slice_after",
    "slice_before",
    "slice_when",
    "sort",
    "sort_by",
    "sum",
    "take",
    "take_while",
    "tally",
    "to_a",
    "to_h",
    "to_set",
    "uniq",
    "zip",
];
const NONMUTATING_ARRAY_METHODS: &[&str] = &[
    "all?",
    "any?",
    "assoc",
    "at",
    "bsearch",
    "bsearch_index",
    "collect",
    "combination",
    "compact",
    "count",
    "cycle",
    "deconstruct",
    "difference",
    "dig",
    "drop",
    "drop_while",
    "each",
    "each_index",
    "empty?",
    "eql?",
    "fetch",
    "filter",
    "find_index",
    "first",
    "flatten",
    "hash",
    "include?",
    "index",
    "inspect",
    "intersection",
    "join",
    "last",
    "length",
    "map",
    "max",
    "min",
    "minmax",
    "none?",
    "one?",
    "pack",
    "permutation",
    "product",
    "rassoc",
    "reject",
    "repeated_combination",
    "repeated_permutation",
    "reverse",
    "reverse_each",
    "rindex",
    "rotate",
    "sample",
    "select",
    "shuffle",
    "size",
    "slice",
    "sort",
    "sum",
    "take",
    "take_while",
    "to_a",
    "to_ary",
    "to_h",
    "to_s",
    "transpose",
    "union",
    "uniq",
    "values_at",
    "zip",
    "|",
];
const NONMUTATING_HASH_METHODS: &[&str] = &[
    "any?",
    "assoc",
    "compact",
    "dig",
    "each",
    "each_key",
    "each_pair",
    "each_value",
    "empty?",
    "eql?",
    "fetch",
    "fetch_values",
    "filter",
    "flatten",
    "has_key?",
    "has_value?",
    "hash",
    "include?",
    "inspect",
    "invert",
    "key",
    "key?",
    "keys?",
    "length",
    "member?",
    "merge",
    "rassoc",
    "rehash",
    "reject",
    "select",
    "size",
    "slice",
    "to_a",
    "to_h",
    "to_hash",
    "to_proc",
    "to_s",
    "transform_keys",
    "transform_values",
    "value?",
    "values",
    "values_at",
];
const NONMUTATING_STRING_METHODS: &[&str] = &[
    "ascii_only?",
    "b",
    "bytes",
    "bytesize",
    "byteslice",
    "capitalize",
    "casecmp",
    "casecmp?",
    "center",
    "chars",
    "chomp",
    "chop",
    "chr",
    "codepoints",
    "count",
    "crypt",
    "delete",
    "delete_prefix",
    "delete_suffix",
    "downcase",
    "dump",
    "each_byte",
    "each_char",
    "each_codepoint",
    "each_grapheme_cluster",
    "each_line",
    "empty?",
    "encode",
    "encoding",
    "end_with?",
    "eql?",
    "getbyte",
    "grapheme_clusters",
    "gsub",
    "hash",
    "hex",
    "include",
    "index",
    "inspect",
    "intern",
    "length",
    "lines",
    "ljust",
    "lstrip",
    "match",
    "match?",
    "next",
    "oct",
    "ord",
    "partition",
    "reverse",
    "rindex",
    "rjust",
    "rpartition",
    "rstrip",
    "scan",
    "scrub",
    "size",
    "slice",
    "squeeze",
    "start_with?",
    "strip",
    "sub",
    "succ",
    "sum",
    "swapcase",
    "to_a",
    "to_c",
    "to_f",
    "to_i",
    "to_r",
    "to_s",
    "to_str",
    "to_sym",
    "tr",
    "tr_s",
    "unicode_normalize",
    "unicode_normalized?",
    "unpack",
    "unpack1",
    "upcase",
    "upto",
    "valid_encoding?",
];
