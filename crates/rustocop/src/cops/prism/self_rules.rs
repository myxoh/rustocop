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
        || local_name_conflicts(call_name(node), receiver.location().start_offset(), context)
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

#[allow(clippy::too_many_lines)]
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

fn local_name_conflicts(name: &[u8], offense_start: usize, context: &CopContext<'_, '_>) -> bool {
    if let Some(branch) = context.ancestors().iter().rev().find_map(Node::as_in_node) {
        let mut bindings = PatternBindings::default();
        bindings.visit(&branch.pattern());
        return bindings.names.iter().any(|binding| binding == name);
    }
    if conditional_assignment_conflict(name, offense_start, context.ancestors()) {
        return true;
    }
    context.ancestors().iter().rev().any(|scope| {
        if let Some(block) = scope.as_block_node() {
            block.locals().iter().any(|local| local.as_slice() == name)
                || block.parameters().is_some_and(|parameters| {
                    explicit_block_parameters_include(&parameters, name, context)
                })
                || block
                    .body()
                    .is_some_and(|body| subtree_binds_name(&body, name, true, None))
        } else if let Some(lambda) = scope.as_lambda_node() {
            lambda.locals().iter().any(|local| local.as_slice() == name)
                || lambda.parameters().is_some_and(|parameters| {
                    explicit_block_parameters_include(&parameters, name, context)
                })
        } else if let Some(definition) = scope.as_def_node() {
            definition.parameters().is_some_and(|parameters| {
                subtree_binds_name(&parameters.as_node(), name, true, None)
            }) || definition.body().is_some_and(|body| {
                subtree_binds_name(&body, name, true, Some(offense_start))
            })
        } else if let Some(program) = scope.as_program_node() {
            program
                .locals()
                .iter()
                .any(|local| local.as_slice() == name)
        } else {
            false
        }
    })
}

fn conditional_assignment_conflict(
    name: &[u8],
    offense_start: usize,
    ancestors: &[Node<'_>],
) -> bool {
    ancestors.iter().rev().any(|ancestor| {
        let (predicate, statements) = if let Some(condition) = ancestor.as_if_node() {
            (condition.predicate(), condition.statements())
        } else if let Some(condition) = ancestor.as_unless_node() {
            (condition.predicate(), condition.statements())
        } else if let Some(condition) = ancestor.as_while_node() {
            (condition.predicate(), condition.statements())
        } else if let Some(condition) = ancestor.as_until_node() {
            (condition.predicate(), condition.statements())
        } else {
            return false;
        };
        let predicate = predicate.location();
        predicate.start_offset() <= offense_start
            && offense_start < predicate.end_offset()
            && statements.is_some_and(|body| {
                subtree_binds_name(&body.as_node(), name, true, None)
            })
    })
}

fn explicit_block_parameters_include(
    parameters: &Node<'_>,
    name: &[u8],
    context: &CopContext<'_, '_>,
) -> bool {
    parameters.as_block_parameters_node().is_some()
        && context
            .source_file()
            .node(parameters)
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .any(|parameter| parameter.as_bytes() == name)
}

fn subtree_binds_name(
    node: &Node<'_>,
    name: &[u8],
    include_writes: bool,
    parameter_before: Option<usize>,
) -> bool {
    struct BindingFinder<'a> {
        name: &'a [u8],
        found: bool,
        include_writes: bool,
        parameter_before: Option<usize>,
    }

    impl BindingFinder<'_> {
        fn check(&mut self, candidate: &[u8]) {
            self.found |= candidate == self.name;
        }

        fn before_cutoff(&self, at: usize) -> bool {
            self.parameter_before.is_none_or(|cutoff| at < cutoff)
        }
    }

    impl<'pr> Visit<'pr> for BindingFinder<'_> {
        fn visit_def_node(&mut self, _node: &ruby_prism::DefNode<'pr>) {}

        fn visit_required_parameter_node(
            &mut self,
            node: &ruby_prism::RequiredParameterNode<'pr>,
        ) {
            if self
                .parameter_before
                .is_none_or(|cutoff| node.location().start_offset() < cutoff)
            {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_required_parameter_node(self, node);
        }

        fn visit_optional_parameter_node(
            &mut self,
            node: &ruby_prism::OptionalParameterNode<'pr>,
        ) {
            if self.before_cutoff(node.location().start_offset()) {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_optional_parameter_node(self, node);
        }

        fn visit_rest_parameter_node(&mut self, node: &ruby_prism::RestParameterNode<'pr>) {
            if self.before_cutoff(node.location().start_offset()) {
                if let Some(name) = node.name() {
                    self.check(name.as_slice());
                }
            }
            ruby_prism::visit_rest_parameter_node(self, node);
        }

        fn visit_required_keyword_parameter_node(
            &mut self,
            node: &ruby_prism::RequiredKeywordParameterNode<'pr>,
        ) {
            if self.before_cutoff(node.location().start_offset()) {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_required_keyword_parameter_node(self, node);
        }

        fn visit_optional_keyword_parameter_node(
            &mut self,
            node: &ruby_prism::OptionalKeywordParameterNode<'pr>,
        ) {
            if self.before_cutoff(node.location().start_offset()) {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_optional_keyword_parameter_node(self, node);
        }

        fn visit_keyword_rest_parameter_node(
            &mut self,
            node: &ruby_prism::KeywordRestParameterNode<'pr>,
        ) {
            if self.before_cutoff(node.location().start_offset()) {
                if let Some(name) = node.name() {
                    self.check(name.as_slice());
                }
            }
            ruby_prism::visit_keyword_rest_parameter_node(self, node);
        }

        fn visit_block_parameter_node(&mut self, node: &ruby_prism::BlockParameterNode<'pr>) {
            if self.before_cutoff(node.location().start_offset()) {
                if let Some(name) = node.name() {
                    self.check(name.as_slice());
                }
            }
            ruby_prism::visit_block_parameter_node(self, node);
        }

        fn visit_local_variable_target_node(
            &mut self,
            node: &ruby_prism::LocalVariableTargetNode<'pr>,
        ) {
            if self.include_writes && self.before_cutoff(node.location().start_offset()) {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_target_node(self, node);
        }

        fn visit_local_variable_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableWriteNode<'pr>,
        ) {
            if self.include_writes && self.before_cutoff(node.location().start_offset()) {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_write_node(self, node);
        }

        fn visit_local_variable_or_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
        ) {
            if self.include_writes && self.before_cutoff(node.location().start_offset()) {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_or_write_node(self, node);
        }

        fn visit_local_variable_and_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
        ) {
            if self.include_writes && self.before_cutoff(node.location().start_offset()) {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_and_write_node(self, node);
        }

        fn visit_local_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
        ) {
            if self.include_writes && self.before_cutoff(node.location().start_offset()) {
                self.check(node.name().as_slice());
            }
            ruby_prism::visit_local_variable_operator_write_node(self, node);
        }
    }

    let mut finder = BindingFinder {
        name,
        found: false,
        include_writes,
        parameter_before,
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
