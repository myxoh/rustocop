use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(ArrayJoin),
        Box::new(NestedFileDirname),
        Box::new(ProcLiteral),
        Box::new(StderrPuts),
        Box::new(Strip),
    ]
}

struct ArrayJoin;

impl Cop for ArrayJoin {
    fn name(&self) -> &'static str {
        "Style/ArrayJoin"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let (Some(receiver), Some(argument), Some(selector)) =
            (call.receiver(), only_argument(&call), call.message_loc())
        else {
            return;
        };
        if call_name(&call) != b"*"
            || receiver.as_array_node().is_none()
            || argument.as_string_node().is_none()
        {
            return;
        }
        let replacement = format!(
            "{}.join({})",
            node_source(source, &receiver),
            node_source(source, &argument)
        );
        context.replace(
            self.name(),
            "Favor `Array#join` over `Array#*`.",
            selector,
            call.location(),
            replacement,
        );
    }
}

struct ProcLiteral;

impl Cop for ProcLiteral {
    fn name(&self) -> &'static str {
        "Style/Proc"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if !match_call(node)
            .named(b"new")
            .on_root_constant(b"Proc")
            .with_block()
            .matches()
        {
            return;
        }
        let (Some(receiver), Some(selector)) = (node.receiver(), node.message_loc()) else {
            return;
        };
        let offense = receiver.location().start_offset()..selector.end_offset();
        context.replace(
            self.name(),
            "Use `proc` instead of `Proc.new`.",
            offense.clone(),
            offense,
            "proc",
        );
    }
}

struct StderrPuts;

impl Cop for StderrPuts {
    fn name(&self) -> &'static str {
        "Style/StderrPuts"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if call_name(node) != b"puts" || node.is_safe_navigation() || first_argument(node).is_none()
        {
            return;
        }
        let Some(receiver) = node.receiver() else {
            return;
        };
        let stderr = receiver
            .as_global_variable_read_node()
            .is_some_and(|global| global.name().as_slice() == b"$stderr")
            || node_is_root_constant(&receiver, b"STDERR");
        if !stderr {
            return;
        }
        let Some(selector) = node.message_loc() else {
            return;
        };
        let offense = receiver.location().start_offset()..selector.end_offset();
        let receiver_name = String::from_utf8_lossy(receiver.location().as_slice());
        context.replace(
            self.name(),
            format!(
                "Use `warn` instead of `{receiver_name}.puts` to allow such output to be disabled."
            ),
            offense.clone(),
            offense,
            "warn",
        );
    }
}

struct Strip;

impl Cop for Strip {
    fn name(&self) -> &'static str {
        "Style/Strip"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        let outer = call_name(node);
        if !match_call(node)
            .named_any(&[b"lstrip", b"rstrip"])
            .without_arguments()
            .matches()
        {
            return;
        }
        let Some(inner) = receiver_call(node) else {
            return;
        };
        let expected_inner = if outer == b"lstrip" {
            b"rstrip"
        } else {
            b"lstrip"
        };
        if !match_call(&inner)
            .named(expected_inner)
            .without_arguments()
            .matches()
            || inner.block().is_some()
        {
            return;
        }
        let (Some(start), Some(selector)) = (inner.message_loc(), node.message_loc()) else {
            return;
        };
        let end = node.closing_loc().unwrap_or(selector);
        let message_chain = start
            .join(&end)
            .expect("locations from one call chain can be joined");
        let offense = start.start_offset()..end.end_offset();
        context.replace(
            self.name(),
            format!(
                "Use `strip` instead of `{}`.",
                String::from_utf8_lossy(message_chain.as_slice())
            ),
            offense.clone(),
            offense,
            "strip",
        );
    }
}

struct NestedFileDirname;

impl Cop for NestedFileDirname {
    fn name(&self) -> &'static str {
        "Style/NestedFileDirname"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        if !context.target_ruby_version().at_least(3, 1) {
            return;
        }
        let Some(call) = node.as_call_node() else {
            return;
        };
        if !file_dirname_call(&call) || nested_in_file_dirname(&call, ancestors) {
            return;
        }
        let Some(mut argument) = only_argument(&call) else {
            return;
        };
        let mut depth = 1;
        while let Some(inner) = argument.as_call_node().filter(file_dirname_call) {
            let Some(next) = only_argument(&inner) else {
                break;
            };
            depth += 1;
            argument = next;
        }
        if depth < 2 {
            return;
        }
        let Some(selector) = call.message_loc() else {
            return;
        };
        let offense = selector.start_offset()..call.location().end_offset();
        let argument = node_source(source, &argument);
        context.replace(
            self.name(),
            format!("Use `dirname({argument}, {depth})` instead."),
            offense.clone(),
            offense,
            format!("dirname({argument}, {depth})"),
        );
    }
}

fn file_dirname_call(call: &CallNode<'_>) -> bool {
    match_call(call)
        .named(b"dirname")
        .on_root_constant(b"File")
        .with_argument_count(1)
        .matches()
}

fn nested_in_file_dirname(call: &CallNode<'_>, ancestors: &[Node<'_>]) -> bool {
    ancestors.iter().rev().any(|ancestor| {
        ancestor.as_call_node().is_some_and(|parent| {
            file_dirname_call(&parent)
                && only_argument(&parent)
                    .is_some_and(|argument| same_location(&argument, &call.as_node()))
        })
    })
}
