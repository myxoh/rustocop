// rubocop-ast 1.49.1
// Source: lib/rubocop/ast/node/mixin/method_dispatch_node.rb
// Source SHA-256: 1d32445b6f24b01f566b311e31652debd32a5c7e12a6c5067bcc63b4063c4aeb

use super::method_identifier_predicates::{MethodIdentifier, ReceiverKind};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum MacroScope {
    Root,
    ClassLike,
    Wrapper(Box<Self>),
    IfBody(Box<Self>),
    IfCondition(Box<Self>),
    Other,
}

impl MacroScope {
    pub(crate) fn in_macro_scope(&self) -> bool {
        match self {
            Self::Root | Self::ClassLike => true,
            Self::Wrapper(parent) | Self::IfBody(parent) => parent.in_macro_scope(),
            Self::IfCondition(_) | Self::Other => false,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum DefModifierArgument {
    Definition,
    Dispatch {
        implicit_receiver: bool,
        argument: Box<Self>,
    },
    Other,
}

impl DefModifierArgument {
    fn definition(&self) -> bool {
        match self {
            Self::Definition => true,
            Self::Dispatch {
                implicit_receiver: true,
                argument,
            } => argument.definition(),
            Self::Dispatch { .. } | Self::Other => false,
        }
    }
}

#[derive(Clone, Debug)]
pub(crate) struct MethodDispatch<'name> {
    identifier: MethodIdentifier<'name>,
    source: &'name str,
    selector: Option<&'name str>,
    connector: Option<&'name str>,
    argument_count: usize,
    setter_operator: bool,
    block_literal: bool,
    expression_is_lambda_arrow: bool,
    selector_starts_expression: bool,
    scope: MacroScope,
    def_modifier_argument: Option<DefModifierArgument>,
}

impl<'name> MethodDispatch<'name> {
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn new(
        identifier: MethodIdentifier<'name>,
        source: &'name str,
        selector: Option<&'name str>,
        connector: Option<&'name str>,
        argument_count: usize,
        setter_operator: bool,
        block_literal: bool,
        expression_is_lambda_arrow: bool,
        selector_starts_expression: bool,
        scope: MacroScope,
        def_modifier_argument: Option<DefModifierArgument>,
    ) -> Self {
        Self {
            identifier,
            source,
            selector,
            connector,
            argument_count,
            setter_operator,
            block_literal,
            expression_is_lambda_arrow,
            selector_starts_expression,
            scope,
            def_modifier_argument,
        }
    }

    pub(crate) fn receiver(&self) -> ReceiverKind {
        self.identifier.receiver()
    }

    pub(crate) fn method_name(&self) -> &str {
        self.identifier.method_name()
    }

    pub(crate) fn selector(&self) -> Option<&str> {
        self.selector
    }

    pub(crate) fn macro_call(&self) -> bool {
        self.receiver() == ReceiverKind::Implicit && self.scope.in_macro_scope()
    }

    pub(crate) fn access_modifier(&self) -> bool {
        self.bare_access_modifier() || self.non_bare_access_modifier()
    }

    pub(crate) fn bare_access_modifier(&self) -> bool {
        self.macro_call() && access_modifier_name(self.method_name()) && self.argument_count == 0
    }

    pub(crate) fn non_bare_access_modifier(&self) -> bool {
        self.macro_call() && access_modifier_name(self.method_name()) && self.argument_count > 0
    }

    pub(crate) fn special_modifier(&self) -> bool {
        self.bare_access_modifier() && matches!(self.source, "private" | "protected")
    }

    pub(crate) fn command(&self, name: &str) -> bool {
        self.receiver() == ReceiverKind::Implicit && self.identifier.method(name)
    }

    pub(crate) fn setter_method(&self) -> bool {
        self.setter_operator
    }

    pub(crate) fn assignment(&self) -> bool {
        self.setter_method()
    }

    pub(crate) fn dot(&self) -> bool {
        self.connector == Some(".")
    }

    pub(crate) fn double_colon(&self) -> bool {
        self.connector == Some("::")
    }

    pub(crate) fn safe_navigation(&self) -> bool {
        self.connector == Some("&.")
    }

    pub(crate) fn self_receiver(&self) -> bool {
        self.identifier.self_receiver()
    }

    pub(crate) fn const_receiver(&self) -> bool {
        self.identifier.const_receiver()
    }

    pub(crate) fn implicit_call(&self) -> bool {
        self.identifier.method("call") && self.selector.is_none()
    }

    pub(crate) fn block_literal(&self) -> bool {
        self.block_literal
    }

    pub(crate) fn arithmetic_operation(&self) -> bool {
        matches!(self.method_name(), "+" | "-" | "*" | "/" | "%" | "**")
    }

    pub(crate) fn def_modifier(&self) -> Option<&DefModifierArgument> {
        self.def_modifier_argument
            .as_ref()
            .filter(|argument| argument.definition())
    }

    pub(crate) fn def_modifier_present(&self) -> bool {
        self.def_modifier().is_some()
    }

    pub(crate) fn lambda(&self) -> bool {
        self.block_literal() && self.command("lambda")
    }

    pub(crate) fn lambda_literal(&self) -> bool {
        self.expression_is_lambda_arrow && self.block_literal()
    }

    pub(crate) fn unary_operation(&self) -> bool {
        self.selector.is_some()
            && self.identifier.operator_method()
            && self.selector_starts_expression
    }

    pub(crate) fn binary_operation(&self) -> bool {
        self.selector.is_some()
            && self.identifier.operator_method()
            && !self.selector_starts_expression
    }
}

fn access_modifier_name(name: &str) -> bool {
    matches!(name, "public" | "protected" | "private" | "module_function")
}
