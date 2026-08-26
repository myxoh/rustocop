// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/traversal.rb
// Source SHA-256: 94a3dc45f4bd503a7858034f0ccb0a5709a56af67674ef0fdd41284baa2c4723

use super::node::core::NodeRef;

pub(crate) fn has_compiled_child_traversal(kind: &str) -> bool {
    !matches!(
        kind,
        "true"
            | "false"
            | "nil"
            | "self"
            | "cbase"
            | "zsuper"
            | "redo"
            | "retry"
            | "forward_args"
            | "forwarded_args"
            | "match_nil_pattern"
            | "forward_arg"
            | "forwarded_restarg"
            | "forwarded_kwrestarg"
            | "lambda"
            | "empty_else"
            | "kwnilarg"
            | "blocknilarg"
            | "__FILE__"
            | "__LINE__"
            | "__ENCODING__"
            | "restarg"
            | "kwrestarg"
            | "int"
            | "float"
            | "complex"
            | "rational"
            | "str"
            | "sym"
            | "lvar"
            | "ivar"
            | "cvar"
            | "gvar"
            | "nth_ref"
            | "back_ref"
            | "arg"
            | "blockarg"
            | "shadowarg"
            | "kwarg"
            | "match_var"
    )
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct DebugError {
    pub(crate) kind: String,
    pub(crate) expected: String,
    pub(crate) actual: usize,
}

pub(crate) fn validate_shape(node: NodeRef<'_>) -> Result<(), DebugError> {
    let expected = match node.kind() {
        "true"
        | "false"
        | "nil"
        | "self"
        | "cbase"
        | "zsuper"
        | "redo"
        | "retry"
        | "forward_args"
        | "forwarded_args"
        | "match_nil_pattern"
        | "forward_arg"
        | "forwarded_restarg"
        | "forwarded_kwrestarg"
        | "lambda"
        | "empty_else"
        | "kwnilarg"
        | "blocknilarg"
        | "__FILE__"
        | "__LINE__"
        | "__ENCODING__" => Some((0, Some(0))),
        "restarg" | "kwrestarg" | "splat" | "kwsplat" | "match_rest" => Some((0, Some(1))),
        "int"
        | "float"
        | "complex"
        | "rational"
        | "str"
        | "sym"
        | "lvar"
        | "ivar"
        | "cvar"
        | "gvar"
        | "nth_ref"
        | "back_ref"
        | "arg"
        | "blockarg"
        | "shadowarg"
        | "kwarg"
        | "match_var"
        | "not"
        | "match_current_line"
        | "defined?"
        | "arg_expr"
        | "pin"
        | "if_guard"
        | "unless_guard"
        | "match_with_trailing_comma"
        | "block_pass"
        | "preexe"
        | "postexe" => Some((1, Some(1))),
        "lvasgn" | "ivasgn" | "cvasgn" | "gvasgn" => Some((1, Some(2))),
        "optarg" | "kwoptarg" | "while" | "until" | "module" | "sclass" | "const" => {
            Some((2, Some(2)))
        }
        "casgn" => Some((2, Some(3))),
        "class" | "def" | "op_asgn" | "if" | "block" | "numblock" | "itblock" => Some((3, Some(3))),
        "defs" => Some((4, Some(4))),
        _ => None,
    };
    let Some((minimum, maximum)) = expected else {
        return Ok(());
    };
    let actual = node.children().len();
    if actual < minimum || maximum.is_some_and(|max| actual > max) {
        Err(DebugError {
            kind: node.kind().into(),
            expected: maximum.filter(|max| *max == minimum).map_or_else(
                || {
                    format!(
                        "{minimum}..{}",
                        maximum.map_or_else(|| "".into(), |max| max.to_string())
                    )
                },
                |_| minimum.to_string(),
            ),
            actual,
        })
    } else {
        Ok(())
    }
}

macro_rules! traversal_callbacks {
    ($($kind:literal => $callback:ident),+ $(,)?) => {
        pub(crate) trait Traversal {
            fn walk(&mut self, node: Option<NodeRef<'_>>) { if let Some(node)=node { self.dispatch(node); } }
            fn walk_children(&mut self, node: NodeRef<'_>) { for child in node.child_nodes() { self.dispatch(child); } }
            fn dispatch(&mut self, node: NodeRef<'_>) { if std::env::var_os("RUBOCOP_DEBUG").is_some(){if let Err(error)=validate_shape(node){panic!("Expected {} children, got {} for {}",error.expected,error.actual,error.kind)}} self.on_node(node); match node.kind() { "mrasgn" => self.on_mrasgn(node), "rasgn" => self.on_rasgn(node), $($kind => self.$callback(node),)+ _ => self.on_unknown(node), } }
            fn on_node(&mut self, _node: NodeRef<'_>) {}
            fn on_unknown(&mut self, node: NodeRef<'_>) { self.walk_children(node); }
            fn on_mrasgn(&mut self, node: NodeRef<'_>) { self.walk_children(node); }
            fn on_rasgn(&mut self, node: NodeRef<'_>) { self.walk_children(node); }
            $(fn $callback(&mut self, node: NodeRef<'_>) { self.walk_children(node); })+
        }
        pub(crate) const KNOWN_NODE_TYPES: &[&str] = &[$($kind),+];
    };
}

traversal_callbacks!(
    "__ENCODING__" => on_encoding, "__FILE__" => on_file, "__LINE__" => on_line,
    "alias" => on_alias, "and" => on_and, "and_asgn" => on_and_asgn, "arg" => on_arg,
    "arg_expr" => on_arg_expr, "args" => on_args, "array" => on_array,
    "array_pattern" => on_array_pattern, "array_pattern_with_tail" => on_array_pattern_with_tail,
    "back_ref" => on_back_ref, "begin" => on_begin, "block" => on_block,
    "block_pass" => on_block_pass, "blockarg" => on_blockarg, "blockarg_expr" => on_blockarg_expr,
    "blocknilarg" => on_blocknilarg, "break" => on_break, "case" => on_case,
    "case_match" => on_case_match, "casgn" => on_casgn, "cbase" => on_cbase,
    "class" => on_class, "complex" => on_complex, "const" => on_const,
    "const_pattern" => on_const_pattern, "csend" => on_csend, "cvar" => on_cvar,
    "cvasgn" => on_cvasgn, "def" => on_def, "defined?" => on_defined,
    "defs" => on_defs, "dstr" => on_dstr, "dsym" => on_dsym,
    "eflipflop" => on_eflipflop, "empty_else" => on_empty_else, "ensure" => on_ensure,
    "erange" => on_erange, "false" => on_false, "find_pattern" => on_find_pattern,
    "float" => on_float, "for" => on_for, "forward_arg" => on_forward_arg,
    "forward_args" => on_forward_args, "forwarded_args" => on_forwarded_args,
    "forwarded_kwrestarg" => on_forwarded_kwrestarg, "forwarded_restarg" => on_forwarded_restarg,
    "gvar" => on_gvar, "gvasgn" => on_gvasgn, "hash" => on_hash,
    "hash_pattern" => on_hash_pattern, "ident" => on_ident, "if" => on_if,
    "if_guard" => on_if_guard, "iflipflop" => on_iflipflop, "in_match" => on_in_match,
    "in_pattern" => on_in_pattern, "index" => on_index, "indexasgn" => on_indexasgn,
    "int" => on_int, "irange" => on_irange, "itarg" => on_itarg, "itblock" => on_itblock,
    "ivar" => on_ivar, "ivasgn" => on_ivasgn, "kwarg" => on_kwarg, "kwargs" => on_kwargs,
    "kwbegin" => on_kwbegin, "kwnilarg" => on_kwnilarg, "kwoptarg" => on_kwoptarg,
    "kwrestarg" => on_kwrestarg, "kwsplat" => on_kwsplat, "lambda" => on_lambda,
    "lvar" => on_lvar, "lvasgn" => on_lvasgn, "masgn" => on_masgn,
    "match_alt" => on_match_alt, "match_as" => on_match_as, "match_current_line" => on_match_current_line,
    "match_nil_pattern" => on_match_nil_pattern, "match_pattern" => on_match_pattern,
    "match_pattern_p" => on_match_pattern_p, "match_rest" => on_match_rest,
    "match_var" => on_match_var, "match_with_lvasgn" => on_match_with_lvasgn,
    "match_with_trailing_comma" => on_match_with_trailing_comma, "mlhs" => on_mlhs,
    "module" => on_module, "next" => on_next, "nil" => on_nil, "not" => on_not,
    "nth_ref" => on_nth_ref, "numargs" => on_numargs, "numblock" => on_numblock,
    "objc_kwarg" => on_objc_kwarg, "objc_restarg" => on_objc_restarg,
    "objc_varargs" => on_objc_varargs, "op_asgn" => on_op_asgn, "optarg" => on_optarg,
    "or" => on_or, "or_asgn" => on_or_asgn, "pair" => on_pair, "pin" => on_pin,
    "postexe" => on_postexe, "preexe" => on_preexe, "procarg0" => on_procarg0,
    "rational" => on_rational, "redo" => on_redo, "regexp" => on_regexp,
    "regopt" => on_regopt, "resbody" => on_resbody, "rescue" => on_rescue,
    "restarg" => on_restarg, "restarg_expr" => on_restarg_expr, "retry" => on_retry,
    "return" => on_return, "sclass" => on_sclass, "self" => on_self,
    "send" => on_send, "shadowarg" => on_shadowarg, "splat" => on_splat,
    "str" => on_str, "super" => on_super, "sym" => on_sym, "true" => on_true,
    "undef" => on_undef, "unless_guard" => on_unless_guard, "until" => on_until,
    "until_post" => on_until_post, "when" => on_when, "while" => on_while,
    "while_post" => on_while_post, "xstr" => on_xstr, "yield" => on_yield, "zsuper" => on_zsuper,
);
