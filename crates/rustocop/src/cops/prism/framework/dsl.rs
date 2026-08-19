/// Declares a module's registry without repeating `Box::new` and trait-object
/// coercions for every cop.
macro_rules! declare_cops {
    ($($cop:expr),+ $(,)?) => {
        pub(super) fn cops() -> Vec<Box<dyn Cop>> {
            vec![$(Box::new($cop) as Box<dyn Cop>),+]
        }
    };
}

/// Declares source-wide cops whose rule bodies use a cop-scoped `Reporter`.
///
/// The handler signature is `fn(&str, &mut Reporter<'_>)`; registration, the
/// marker type, `Cop::name`, and reporter scoping are generated here.
macro_rules! declare_source_cops {
    ($($type:ident => $name:literal => $check:path),+ $(,)?) => {
        declare_cops!($($type),+);

        $(
            struct $type;

            impl Cop for $type {
                fn name(&self) -> &'static str {
                    $name
                }

                fn on_source(&self, source: &str, context: &mut Context) {
                    let mut reporter = context.reporter(self.name());
                    $check(source, &mut reporter);
                }
            }
        )+
    };
}

/// Defines a call-only cop around a function with the signature
/// `fn(&CallNode<'_>, &mut CopContext<'_, '_>)`.
macro_rules! define_call_cop {
    ($type:ident => $name:literal => $check:path) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn on_node<'pr>(
                &self,
                node: &Node<'pr>,
                ancestors: &[Node<'pr>],
                source: &str,
                context: &mut Context,
            ) {
                let Some(call) = node.as_call_node() else {
                    return;
                };
                let mut cop_context = context.cop_context(self.name(), source, ancestors);
                $check(&call, &mut cop_context);
            }
        }
    };
}

/// Defines a cop callback for one Prism node type. The cast is the Prism
/// `Node::as_*_node` method used to select that type.
macro_rules! define_node_cop {
    ($type:ident => $name:literal => $cast:ident => $check:path) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn on_node<'pr>(
                &self,
                node: &Node<'pr>,
                ancestors: &[Node<'pr>],
                source: &str,
                context: &mut Context,
            ) {
                let Some(typed_node) = node.$cast() else {
                    return;
                };
                let mut cop_context = context.cop_context(self.name(), source, ancestors);
                $check(&typed_node, &mut cop_context);
            }
        }
    };
}

/// Defines a cop that intentionally examines more than one Prism node type.
macro_rules! define_any_node_cop {
    ($type:ident => $name:literal => $check:path) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn on_node<'pr>(
                &self,
                node: &Node<'pr>,
                ancestors: &[Node<'pr>],
                source: &str,
                context: &mut Context,
            ) {
                let mut cop_context = context.cop_context(self.name(), source, ancestors);
                $check(node, &mut cop_context);
            }
        }
    };
}

/// Defines a source-wide cop whose callback receives the same rich context as
/// AST callbacks.
macro_rules! define_source_context_cop {
    ($type:ident => $name:literal => $check:path) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn on_source(&self, source: &str, context: &mut Context) {
                let mut cop_context = context.cop_context(self.name(), source, &[]);
                $check(&mut cop_context);
            }
        }
    };
}

/// Declares and registers a homogeneous set of concise cop callbacks. A cop's
/// name appears exactly once and no public inventory needs a matching edit.
macro_rules! define_cops {
    ($($type:ident => $name:literal => $kind:ident($($arguments:tt)*)),+ $(,)?) => {
        declare_cops!($($type),+);
        $(define_cop_entry!($type => $name => $kind($($arguments)*));)+
    };
}

macro_rules! define_cop_entry {
    ($type:ident => $name:literal => call($check:path)) => {
        define_call_cop!($type => $name => $check);
    };
    ($type:ident => $name:literal => node($cast:ident, $check:path)) => {
        define_node_cop!($type => $name => $cast => $check);
    };
    ($type:ident => $name:literal => any_node($check:path)) => {
        define_any_node_cop!($type => $name => $check);
    };
    ($type:ident => $name:literal => source($check:path)) => {
        define_source_context_cop!($type => $name => $check);
    };
}

pub(super) use {
    declare_cops, declare_source_cops, define_any_node_cop, define_call_cop, define_cop_entry,
    define_cops, define_node_cop, define_source_context_cop,
};
