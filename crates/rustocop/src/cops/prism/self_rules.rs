use super::*;

define_cops! {
    RedundantSelf => "Style/RedundantSelf" => call(redundant_self),
}

fn redundant_self(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(receiver) = node.receiver() else {
        return;
    };
    if receiver.as_self_node().is_none() || excluded_method(call_name(node)) {
        return;
    }
    let Some(selector) = node.message_loc() else {
        return;
    };
    if selector
        .as_slice()
        .first()
        .is_some_and(u8::is_ascii_uppercase)
        || local_name_conflicts(call_name(node), context)
        || implicit_it_block(call_name(node), context)
    {
        return;
    }
    context.remove(
        "Redundant `self` detected.",
        receiver.location(),
        receiver.location().start_offset()..selector.start_offset(),
    );
}

fn excluded_method(name: &[u8]) -> bool {
    name.ends_with(b"=")
        || matches!(
            name,
            b"+" | b"-"
                | b"*"
                | b"/"
                | b"%"
                | b"**"
                | b"|"
                | b"^"
                | b"&"
                | b"<=>"
                | b">"
                | b">="
                | b"<"
                | b"<="
                | b"=="
                | b"==="
                | b"!="
                | b"=~"
                | b"!~"
                | b"<<"
                | b">>"
                | b"[]"
                | b"~"
                | b"+@"
                | b"-@"
                | b"!"
        )
        || matches!(
            name,
            b"__callee__"
                | b"__dir__"
                | b"__method__"
                | b"`"
                | b"abort"
                | b"at_exit"
                | b"autoload"
                | b"autoload?"
                | b"binding"
                | b"block_given?"
                | b"caller"
                | b"caller_locations"
                | b"catch"
                | b"eval"
                | b"exec"
                | b"exit"
                | b"exit!"
                | b"fail"
                | b"fork"
                | b"format"
                | b"gets"
                | b"global_variables"
                | b"iterator?"
                | b"lambda"
                | b"load"
                | b"local_variables"
                | b"loop"
                | b"no_warning_require"
                | b"open"
                | b"p"
                | b"print"
                | b"printf"
                | b"proc"
                | b"putc"
                | b"puts"
                | b"raise"
                | b"rand"
                | b"readline"
                | b"readlines"
                | b"require"
                | b"require_relative"
                | b"select"
                | b"set_trace_func"
                | b"sleep"
                | b"spawn"
                | b"sprintf"
                | b"srand"
                | b"syscall"
                | b"system"
                | b"test"
                | b"throw"
                | b"trace_var"
                | b"trap"
                | b"untrace_var"
                | b"warn"
        )
        || matches!(
            name,
            b"class"
                | b"for"
                | b"and"
                | b"or"
                | b"alias"
                | b"begin"
                | b"break"
                | b"case"
                | b"def"
                | b"defined?"
                | b"do"
                | b"else"
                | b"elsif"
                | b"end"
                | b"ensure"
                | b"false"
                | b"if"
                | b"in"
                | b"module"
                | b"next"
                | b"nil"
                | b"not"
                | b"redo"
                | b"rescue"
                | b"retry"
                | b"return"
                | b"self"
                | b"super"
                | b"then"
                | b"true"
                | b"undef"
                | b"unless"
                | b"until"
                | b"when"
                | b"while"
                | b"yield"
                | b"__FILE__"
                | b"__LINE__"
                | b"__ENCODING__"
        )
}

fn local_name_conflicts(name: &[u8], context: &CopContext<'_, '_>) -> bool {
    if let Some(branch) = context.ancestors().iter().rev().find_map(Node::as_in_node) {
        let mut bindings = PatternBindings::default();
        bindings.visit(&branch.pattern());
        return bindings.names.iter().any(|binding| binding == name);
    }
    context.ancestors().iter().rev().any(|scope| {
        if let Some(block) = scope.as_block_node() {
            block.locals().iter().any(|local| local.as_slice() == name)
                || block
                    .body()
                    .is_some_and(|body| subtree_binds_name(&body, name, true))
        } else if let Some(definition) = scope.as_def_node() {
            definition
                .locals()
                .iter()
                .any(|local| local.as_slice() == name)
                || definition
                    .body()
                    .is_some_and(|body| subtree_binds_name(&body, name, false))
        } else if let Some(program) = scope.as_program_node() {
            program
                .locals()
                .iter()
                .any(|local| local.as_slice() == name)
                || subtree_binds_name(&program.statements().as_node(), name, true)
        } else {
            false
        }
    })
}

fn subtree_binds_name(node: &Node<'_>, name: &[u8], include_writes: bool) -> bool {
    struct BindingFinder<'a> {
        name: &'a [u8],
        found: bool,
        include_writes: bool,
    }

    impl BindingFinder<'_> {
        fn check(&mut self, candidate: &[u8]) {
            self.found |= candidate == self.name;
        }
    }

    impl<'pr> Visit<'pr> for BindingFinder<'_> {
        fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}

        fn visit_required_parameter_node(
            &mut self,
            node: &ruby_prism::RequiredParameterNode<'pr>,
        ) {
            self.check(node.name().as_slice());
            ruby_prism::visit_required_parameter_node(self, node);
        }

        fn visit_local_variable_target_node(
            &mut self,
            node: &ruby_prism::LocalVariableTargetNode<'pr>,
        ) {
            self.check(node.name().as_slice());
            ruby_prism::visit_local_variable_target_node(self, node);
        }

        fn visit_local_variable_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableWriteNode<'pr>,
        ) {
            if self.include_writes {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_write_node(self, node);
        }

        fn visit_local_variable_or_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
        ) {
            if self.include_writes {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_or_write_node(self, node);
        }

        fn visit_local_variable_and_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
        ) {
            if self.include_writes {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_and_write_node(self, node);
        }

        fn visit_local_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
        ) {
            if self.include_writes {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_operator_write_node(self, node);
        }
    }

    let mut finder = BindingFinder {
        name,
        found: false,
        include_writes,
    };
    finder.visit(node);
    finder.found
}

fn implicit_it_block(name: &[u8], context: &CopContext<'_, '_>) -> bool {
    name == b"it"
        && context
            .ancestors()
            .iter()
            .rev()
            .find_map(Node::as_block_node)
            .is_some_and(|block| block.parameters().is_none())
}

#[derive(Default)]
struct PatternBindings {
    names: Vec<Vec<u8>>,
}

impl<'pr> Visit<'pr> for PatternBindings {
    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        self.names.push(node.name().as_slice().to_vec());
    }
}
