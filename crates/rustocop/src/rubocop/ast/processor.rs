// ast 2.4.3
// Source: lib/ast/processor/mixin.rb
// Source MD5: 9186c2d8a7a97a918db4efbdcf962937

use super::node::core::{Ast, NodeId};

macro_rules! processor_callbacks {
    ($($kind:literal => $callback:ident),+ $(,)?) => {
        /// Typed Rust adaptation of `AST::Processor::Mixin`.
        ///
        /// A callback returns `Some(node)` to replace the input node and
        /// `None` to preserve it, matching the upstream nil-return contract.
        pub(crate) trait Processor {
            fn process(&mut self, ast: &mut Ast, node: Option<NodeId>) -> Option<NodeId> {
                let node = node?;
                Some(self.dispatch(ast, node).unwrap_or(node))
            }

            fn process_all(&mut self, ast: &mut Ast, nodes: &[NodeId]) -> Vec<NodeId> {
                nodes
                    .iter()
                    .filter_map(|node| self.process(ast, Some(*node)))
                    .collect()
            }

            fn dispatch(&mut self, ast: &mut Ast, node: NodeId) -> Option<NodeId> {
                let kind = ast.node(node).kind().to_string();
                match kind.as_str() {
                    $($kind => self.$callback(ast, node),)+
                    _ => self.handler_missing(ast, node),
                }
            }

            fn handler_missing(&mut self, _ast: &mut Ast, _node: NodeId) -> Option<NodeId> {
                None
            }

            $(fn $callback(&mut self, ast: &mut Ast, node: NodeId) -> Option<NodeId> {
                self.handler_missing(ast, node)
            })+
        }
    };
}

processor_callbacks!(
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
    "match_alt" => on_match_alt, "match_as" => on_match_as,
    "match_current_line" => on_match_current_line, "match_nil_pattern" => on_match_nil_pattern,
    "match_pattern" => on_match_pattern, "match_pattern_p" => on_match_pattern_p,
    "match_rest" => on_match_rest, "match_var" => on_match_var,
    "match_with_lvasgn" => on_match_with_lvasgn,
    "match_with_trailing_comma" => on_match_with_trailing_comma, "mlhs" => on_mlhs,
    "module" => on_module, "mrasgn" => on_mrasgn, "next" => on_next, "nil" => on_nil,
    "not" => on_not, "nth_ref" => on_nth_ref, "numargs" => on_numargs,
    "numblock" => on_numblock, "objc_kwarg" => on_objc_kwarg,
    "objc_restarg" => on_objc_restarg, "objc_varargs" => on_objc_varargs,
    "op_asgn" => on_op_asgn, "optarg" => on_optarg, "or" => on_or,
    "or_asgn" => on_or_asgn, "pair" => on_pair, "pin" => on_pin,
    "postexe" => on_postexe, "preexe" => on_preexe, "procarg0" => on_procarg0,
    "rasgn" => on_rasgn, "rational" => on_rational, "redo" => on_redo,
    "regexp" => on_regexp, "regopt" => on_regopt, "resbody" => on_resbody,
    "rescue" => on_rescue, "restarg" => on_restarg, "restarg_expr" => on_restarg_expr,
    "retry" => on_retry, "return" => on_return, "sclass" => on_sclass,
    "self" => on_self, "send" => on_send, "sequence" => on_sequence,
    "shadowarg" => on_shadowarg, "splat" => on_splat, "str" => on_str,
    "subsequence" => on_subsequence, "super" => on_super, "sym" => on_sym,
    "true" => on_true, "undef" => on_undef, "unless_guard" => on_unless_guard,
    "until" => on_until, "until_post" => on_until_post, "when" => on_when,
    "while" => on_while, "while_post" => on_while_post, "xstr" => on_xstr,
    "yield" => on_yield, "zsuper" => on_zsuper,
);
