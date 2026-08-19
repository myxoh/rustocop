/// Defines the per-cop object used while a callback is running. The object
/// behaves like RuboCop's cop instance: callback helpers are inherent methods
/// and inspection/configuration/reporting are available through `self`.
macro_rules! define_rule {
    ($rule:ident) => {
        struct $rule<'rule, 'context, 'pr> {
            context: &'rule mut CopContext<'context, 'pr>,
        }

        impl<'rule, 'context, 'pr> $rule<'rule, 'context, 'pr> {
            fn new(context: &'rule mut CopContext<'context, 'pr>) -> Self {
                Self { context }
            }
        }

        impl<'rule, 'context, 'pr> std::ops::Deref for $rule<'rule, 'context, 'pr> {
            type Target = CopContext<'context, 'pr>;

            fn deref(&self) -> &Self::Target {
                self.context
            }
        }

        impl<'rule, 'context, 'pr> std::ops::DerefMut for $rule<'rule, 'context, 'pr> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.context
            }
        }
    };
}

macro_rules! define_stateful_rule {
    ($rule:ident, $state:ident) => {
        struct $rule<'rule, 'context, 'pr> {
            context: &'rule mut CopContext<'context, 'pr>,
            state: &'rule mut $state,
        }

        impl<'rule, 'context, 'pr> $rule<'rule, 'context, 'pr> {
            fn new(
                context: &'rule mut CopContext<'context, 'pr>,
                state: &'rule mut $state,
            ) -> Self {
                Self { context, state }
            }
        }

        impl<'rule, 'context, 'pr> std::ops::Deref for $rule<'rule, 'context, 'pr> {
            type Target = CopContext<'context, 'pr>;

            fn deref(&self) -> &Self::Target {
                self.context
            }
        }

        impl<'rule, 'context, 'pr> std::ops::DerefMut for $rule<'rule, 'context, 'pr> {
            fn deref_mut(&mut self) -> &mut Self::Target {
                self.context
            }
        }
    };
}

macro_rules! define_call_rule_cop {
    ($type:ident => $name:literal => $rule:ident::$callback:ident) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }

            fn on_node<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context) {
                let Some(call) = node.as_call_node() else { return; };
                let mut context = context.cop_context(self.name(), source, ancestors);
                $rule::new(&mut context).$callback(&call);
            }
        }
    };
    ($type:ident => $name:literal => $rule:ident::$callback:ident restrict [$($method:literal),+ $(,)?]) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }

            fn on_node<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context) {
                let Some(call) = node.as_call_node() else { return; };
                if ![$($method.as_slice()),+].contains(&call.method_name()) { return; }
                let mut context = context.cop_context(self.name(), source, ancestors);
                $rule::new(&mut context).$callback(&call);
            }
        }
    };
}

macro_rules! define_stateful_call_rule_cop {
    ($type:ident => $name:literal => $rule:ident<$state:ident>::$callback:ident restrict [$($method:literal),+ $(,)?]) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }
            fn investigation_state(&self) -> Box<dyn Any> { Box::new($state::default()) }
            fn on_new_investigation(&self, state: &mut dyn Any) {
                *state.downcast_mut::<$state>().expect("cop state type") = $state::default();
            }
            fn on_node_with_state<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context, state: &mut dyn Any) {
                let Some(call) = node.as_call_node() else { return; };
                if ![$($method.as_slice()),+].contains(&call.method_name()) { return; }
                let state = state.downcast_mut::<$state>().expect("cop state type");
                let mut context = context.cop_context(self.name(), source, ancestors);
                $rule::new(&mut context, state).$callback(&call);
            }
        }
    };
}

macro_rules! define_node_rule_cop {
    ($type:ident => $name:literal => $cast:ident => $rule:ident::$callback:ident) => {
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
                let mut context = context.cop_context(self.name(), source, ancestors);
                $rule::new(&mut context).$callback(&typed_node);
            }
        }
    };
}

macro_rules! define_stateful_node_rule_cop {
    ($type:ident => $name:literal => $cast:ident => $rule:ident<$state:ident>::$callback:ident) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str {
                $name
            }
            fn investigation_state(&self) -> Box<dyn Any> {
                Box::new($state::default())
            }
            fn on_new_investigation(&self, state: &mut dyn Any) {
                *state.downcast_mut::<$state>().expect("cop state type") = $state::default();
            }
            fn on_node_with_state<'pr>(
                &self,
                node: &Node<'pr>,
                ancestors: &[Node<'pr>],
                source: &str,
                context: &mut Context,
                state: &mut dyn Any,
            ) {
                let Some(typed_node) = node.$cast() else {
                    return;
                };
                let state = state.downcast_mut::<$state>().expect("cop state type");
                let mut context = context.cop_context(self.name(), source, ancestors);
                $rule::new(&mut context, state).$callback(&typed_node);
            }
        }
    };
}

macro_rules! define_any_node_rule_cop {
    ($type:ident => $name:literal => $rule:ident::$callback:ident) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }
            fn on_node<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context) {
                let mut context = context.cop_context(self.name(), source, ancestors);
                $rule::new(&mut context).$callback(node);
            }
        }
    };
    ($type:ident => $name:literal => $rule:ident::$callback:ident aliases [$($cast:ident),+ $(,)?]) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }
            fn on_node<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context) {
                if !($(node.$cast().is_some())||+) { return; }
                let mut context = context.cop_context(self.name(), source, ancestors);
                $rule::new(&mut context).$callback(node);
            }
        }
    };
}

pub(super) use {
    define_any_node_rule_cop, define_call_rule_cop, define_node_rule_cop, define_rule,
    define_stateful_call_rule_cop, define_stateful_node_rule_cop, define_stateful_rule,
};
