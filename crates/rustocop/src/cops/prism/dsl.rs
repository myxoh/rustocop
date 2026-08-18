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
/// `fn(&CallNode<'_>, &mut Reporter<'_>)`.
macro_rules! define_call_cop {
    ($type:ident => $name:literal => $check:path) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str {
                $name
            }

            fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
                let mut reporter = context.reporter(self.name());
                $check(node, &mut reporter);
            }
        }
    };
}

pub(super) use {declare_cops, declare_source_cops, define_call_cop};
