use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(Eval),
        Box::new(CompoundHash),
        Box::new(JsonLoad),
        Box::new(MarshalLoad),
        Box::new(Open),
        Box::new(IoMethods),
        Box::new(YamlLoad),
    ]
}

struct YamlLoad;

impl Cop for YamlLoad {
    fn name(&self) -> &'static str {
        "Security/YAMLLoad"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if context.target_ruby_version().at_least(3, 1)
            || !CallMatcher::new(node)
                .named(b"load")
                .on_root_constant(b"YAML")
                .matches()
        {
            return;
        }
        let Some(selector) = node.message_loc() else {
            return;
        };
        context.replace(
            self.name(),
            "Prefer using `YAML.safe_load` over `YAML.load`.",
            &selector,
            &selector,
            "safe_load",
        );
    }
}

struct Eval;

impl Cop for Eval {
    fn name(&self) -> &'static str {
        "Security/Eval"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if call_name(node) != b"eval" || !eval_receiver(node.receiver()) {
            return;
        }

        let Some(code) = first_argument(node) else {
            return;
        };
        if code.as_string_node().is_some() || recursive_literal_string(&code) {
            return;
        }

        if let Some(selector) = node.message_loc() {
            context.report(
                self.name(),
                "The use of `eval` is a serious security risk.",
                selector,
            );
        }
    }
}

struct JsonLoad;

struct CompoundHash;

impl Cop for CompoundHash {
    fn name(&self) -> &'static str {
        "Security/CompoundHash"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        if !matches!(call_name(&call), b"^" | b"+" | b"*" | b"|") || first_argument(&call).is_none()
        {
            return;
        }
        let inside_hash_method = ancestors.iter().rev().any(|ancestor| {
            ancestor.as_def_node().is_some_and(|definition| {
                definition.name().as_slice() == b"hash" && definition.parameters().is_none()
            })
        });
        let nested_combinator = ancestors.iter().rev().any(|ancestor| {
            ancestor
                .as_call_node()
                .is_some_and(|parent| matches!(call_name(&parent), b"^" | b"+" | b"*" | b"|"))
        });
        if inside_hash_method && !nested_combinator {
            context.report(
                self.name(),
                "Use `[...].hash` instead of combining hash values manually.",
                call.location(),
            );
        }
    }
}

impl Cop for JsonLoad {
    fn name(&self) -> &'static str {
        "Security/JSONLoad"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        let method = call_name(node);
        if !CallMatcher::new(node)
            .named_any(&[b"load", b"restore"])
            .on_root_constant(b"JSON")
            .matches()
            || has_keyword(node, b"create_additions")
        {
            return;
        }

        if let Some(selector) = node.message_loc() {
            context.replace(
                self.name(),
                format!(
                    "Prefer `JSON.parse` over `JSON.{}`.",
                    String::from_utf8_lossy(method)
                ),
                &selector,
                &selector,
                "parse",
            );
        }
    }
}

struct MarshalLoad;

impl Cop for MarshalLoad {
    fn name(&self) -> &'static str {
        "Security/MarshalLoad"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        let method = call_name(node);
        if !CallMatcher::new(node)
            .named_any(&[b"load", b"restore"])
            .on_root_constant(b"Marshal")
            .matches()
        {
            return;
        }

        let Some(argument) = first_argument(node) else {
            return;
        };
        if marshal_dump(&argument) {
            return;
        }

        if let Some(selector) = node.message_loc() {
            context.report(
                self.name(),
                format!("Avoid using `Marshal.{}`.", String::from_utf8_lossy(method)),
                selector,
            );
        }
    }
}

struct Open;

impl Cop for Open {
    fn name(&self) -> &'static str {
        "Security/Open"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if call_name(node) != b"open" {
            return;
        }

        let receiver = node.receiver();
        let receiver_name = if receiver.is_none() {
            "Kernel#"
        } else if root_constant(receiver, b"URI") {
            "URI."
        } else {
            return;
        };

        let Some(argument) = first_argument(node) else {
            return;
        };
        if safe_open_argument(&argument) {
            return;
        }

        if let Some(selector) = node.message_loc() {
            context.report(
                self.name(),
                format!(
                    "The use of `{}open` is a serious security risk.",
                    receiver_name
                ),
                selector,
            );
        }
    }
}

struct IoMethods;

impl Cop for IoMethods {
    fn name(&self) -> &'static str {
        "Security/IoMethods"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        let method = call_name(node);
        if !CallMatcher::new(node)
            .named_any(&[
                b"read",
                b"binread",
                b"write",
                b"binwrite",
                b"foreach",
                b"readlines",
            ])
            .on_constant_read(b"IO")
            .matches()
        {
            return;
        }

        if first_argument(node).is_some_and(|argument| string_starts_with_pipe(&argument)) {
            return;
        }

        let Some(receiver_location) = node.receiver().map(|receiver| receiver.location()) else {
            return;
        };
        context.replace(
            self.name(),
            format!(
                "`File.{}` is safer than `IO.{}`.",
                String::from_utf8_lossy(method),
                String::from_utf8_lossy(method)
            ),
            node.location(),
            receiver_location,
            "File",
        );
    }
}
