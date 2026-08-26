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

/// Maps RuboCop callback names to their Prism node representation. A callback
/// remains visible under its Ruby name while this adapter owns the parser-
/// specific cast. Callbacks such as `on_casgn` deliberately cover several
/// Prism write nodes because Parser exposes them as one RuboCop event.
macro_rules! dispatch_rubocop_callback {
    ($rule:ident, $node:ident, on_send) => {
        if let Some(typed_node) = $node.as_call_node() {
            $rule.on_send(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_if) => {
        if let Some(typed_node) = $node.as_if_node() {
            $rule.on_if(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_unless) => {
        if let Some(typed_node) = $node.as_unless_node() {
            $rule.on_unless(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_str) => {
        if let Some(typed_node) = $node.as_string_node() {
            $rule.on_str(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_block) => {
        if let Some(typed_node) = $node.as_block_node() {
            $rule.on_block(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_lambda) => {
        if let Some(typed_node) = $node.as_lambda_node() {
            $rule.on_lambda(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_while) => {
        if let Some(typed_node) = $node.as_while_node() {
            $rule.on_while(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_until) => {
        if let Some(typed_node) = $node.as_until_node() {
            $rule.on_until(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_for) => {
        if let Some(typed_node) = $node.as_for_node() {
            $rule.on_for(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_def) => {
        if let Some(typed_node) = $node.as_def_node() {
            $rule.on_def(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_class) => {
        if let Some(typed_node) = $node.as_class_node() {
            $rule.on_class(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_module) => {
        if let Some(typed_node) = $node.as_module_node() {
            $rule.on_module(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_array) => {
        if let Some(typed_node) = $node.as_array_node() {
            $rule.on_array(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_hash) => {
        if let Some(typed_node) = $node.as_hash_node() {
            $rule.on_hash(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_keyword_hash) => {
        if let Some(typed_node) = $node.as_keyword_hash_node() {
            $rule.on_keyword_hash(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_case) => {
        if let Some(typed_node) = $node.as_case_node() {
            $rule.on_case(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_case_match) => {
        if let Some(typed_node) = $node.as_case_match_node() {
            $rule.on_case_match(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_yield) => {
        if let Some(typed_node) = $node.as_yield_node() {
            $rule.on_yield(&typed_node);
        }
    };
    ($rule:ident, $node:ident, on_casgn) => {
        if $node.as_constant_write_node().is_some()
            || $node.as_constant_path_write_node().is_some()
            || $node.as_constant_or_write_node().is_some()
            || $node.as_constant_path_or_write_node().is_some()
            || $node.as_constant_and_write_node().is_some()
            || $node.as_constant_path_and_write_node().is_some()
            || $node.as_constant_operator_write_node().is_some()
            || $node.as_constant_path_operator_write_node().is_some()
        {
            $rule.on_casgn($node);
        }
    };
}

macro_rules! rubocop_callback_matches {
    ($node:ident, on_send) => {
        $node.as_call_node().is_some()
    };
    ($node:ident, on_if) => {
        $node.as_if_node().is_some()
    };
    ($node:ident, on_unless) => {
        $node.as_unless_node().is_some()
    };
    ($node:ident, on_str) => {
        $node.as_string_node().is_some()
    };
    ($node:ident, on_block) => {
        $node.as_block_node().is_some()
    };
    ($node:ident, on_lambda) => {
        $node.as_lambda_node().is_some()
    };
    ($node:ident, on_while) => {
        $node.as_while_node().is_some()
    };
    ($node:ident, on_until) => {
        $node.as_until_node().is_some()
    };
    ($node:ident, on_for) => {
        $node.as_for_node().is_some()
    };
    ($node:ident, on_def) => {
        $node.as_def_node().is_some()
    };
    ($node:ident, on_class) => {
        $node.as_class_node().is_some()
    };
    ($node:ident, on_module) => {
        $node.as_module_node().is_some()
    };
    ($node:ident, on_array) => {
        $node.as_array_node().is_some()
    };
    ($node:ident, on_hash) => {
        $node.as_hash_node().is_some()
    };
    ($node:ident, on_keyword_hash) => {
        $node.as_keyword_hash_node().is_some()
    };
    ($node:ident, on_case) => {
        $node.as_case_node().is_some()
    };
    ($node:ident, on_case_match) => {
        $node.as_case_match_node().is_some()
    };
    ($node:ident, on_yield) => {
        $node.as_yield_node().is_some()
    };
    ($node:ident, on_casgn) => {
        $node.as_constant_write_node().is_some()
            || $node.as_constant_path_write_node().is_some()
            || $node.as_constant_or_write_node().is_some()
            || $node.as_constant_path_or_write_node().is_some()
            || $node.as_constant_and_write_node().is_some()
            || $node.as_constant_path_and_write_node().is_some()
            || $node.as_constant_operator_write_node().is_some()
            || $node.as_constant_path_operator_write_node().is_some()
    };
}

/// Defines one cop adapter from the same callback names visible in RuboCop.
/// Multiple callbacks share one rule object and one cop-scoped context.
macro_rules! define_rubocop_callback_rule_cop {
    ($type:ident => $name:literal => $rule:ident [on_block, on_send restrict [$($method:literal),+ $(,)?]]) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }

            fn on_node_with_state<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context, _state: &mut dyn Any) {
                let is_restricted_send = node.as_call_node().is_some_and(|call| {
                    [$($method.as_slice()),+].contains(&call.method_name())
                });
                if node.as_block_node().is_none() && !is_restricted_send { return; }
                let mut context = context.cop_context(self.name(), source, ancestors);
                let mut rule = $rule::new(&mut context);
                if let Some(block) = node.as_block_node() { rule.on_block(&block); }
                if let Some(call) = node.as_call_node() { rule.on_send(&call); }
            }
        }
    };
    ($type:ident => $name:literal => $rule:ident [$($callback:ident),+ $(,)?]) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }

            fn on_node_with_state<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context, _state: &mut dyn Any) {
                if !($(rubocop_callback_matches!(node, $callback))||+) { return; }
                let mut context = context.cop_context(self.name(), source, ancestors);
                let mut rule = $rule::new(&mut context);
                $(dispatch_rubocop_callback!(rule, node, $callback);)+
            }
        }
    };
    ($type:ident => $name:literal => $rule:ident [on_send restrict [$($method:literal),+ $(,)?]]) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }

            fn on_node_with_state<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context, _state: &mut dyn Any) {
                let Some(call) = node.as_call_node() else { return; };
                if ![$($method.as_slice()),+].contains(&call.method_name()) { return; }
                let mut context = context.cop_context(self.name(), source, ancestors);
                $rule::new(&mut context).on_send(&call);
            }
        }
    };
}

macro_rules! define_recovery_rubocop_callback_rule_cop {
    ($type:ident => $name:literal => $rule:ident [$($callback:ident),+ $(,)?]) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }
            fn visits_recovered_nodes(&self) -> bool { true }

            fn on_node_with_state<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context, _state: &mut dyn Any) {
                if !($(rubocop_callback_matches!(node, $callback))||+) { return; }
                let mut context = context.cop_context(self.name(), source, ancestors);
                let mut rule = $rule::new(&mut context);
                $(dispatch_rubocop_callback!(rule, node, $callback);)+
            }
        }
    };
}

macro_rules! define_stateful_rubocop_callback_rule_cop {
    ($type:ident => $name:literal => $rule:ident<$state:ident> [$($callback:ident),+ $(,)?]) => {
        struct $type;

        impl Cop for $type {
            fn name(&self) -> &'static str { $name }
            fn investigation_state(&self) -> Box<dyn Any> { Box::new($state::default()) }
            fn on_new_investigation(&self, state: &mut dyn Any) {
                *state.downcast_mut::<$state>().expect("cop state type") = $state::default();
            }
            fn on_node_with_state<'pr>(&self, node: &Node<'pr>, ancestors: &[Node<'pr>], source: &str, context: &mut Context, state: &mut dyn Any) {
                if !($(rubocop_callback_matches!(node, $callback))||+) { return; }
                let state = state.downcast_mut::<$state>().expect("cop state type");
                let mut context = context.cop_context(self.name(), source, ancestors);
                let mut rule = $rule::new(&mut context, state);
                $(dispatch_rubocop_callback!(rule, node, $callback);)+
            }
        }
    };
}

pub(super) use {
    define_any_node_rule_cop, define_call_rule_cop, define_node_rule_cop,
    define_recovery_rubocop_callback_rule_cop, define_rubocop_callback_rule_cop, define_rule,
    define_stateful_call_rule_cop, define_stateful_node_rule_cop,
    define_stateful_rubocop_callback_rule_cop, define_stateful_rule, dispatch_rubocop_callback,
    rubocop_callback_matches,
};
