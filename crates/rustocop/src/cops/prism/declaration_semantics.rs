use super::*;

define_cops! {
    IneffectiveAccessModifier => "Lint/IneffectiveAccessModifier" => node(as_def_node, ineffective_access_modifier),
    DefWithParentheses => "Style/DefWithParentheses" => node(as_def_node, def_with_parentheses),
    MissingRespondToMissing => "Style/MissingRespondToMissing" => node(as_def_node, missing_respond_to_missing),
}

fn ineffective_access_modifier(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .receiver()
        .is_none_or(|receiver| receiver.as_self_node().is_none())
        || context
            .ancestors()
            .iter()
            .any(|ancestor| ancestor.as_singleton_class_node().is_some())
        || context.ancestors().iter().rev().any(|ancestor| {
            ancestor
                .as_call_node()
                .is_some_and(|call| call.name().as_slice() == b"private_class_method")
        })
    {
        return;
    }
    let scope = context.ancestors().iter().rev().find(|ancestor| {
        ancestor.as_block_node().is_some()
            || ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
    });
    let Some(scope) = scope.filter(|scope| scope.as_block_node().is_none()) else {
        return;
    };
    if context.ancestors().iter().rev().take_while(|ancestor| {
        ancestor.as_class_node().is_none() && ancestor.as_module_node().is_none()
    }).any(|ancestor| {
        ancestor.as_if_node().is_some()
            || ancestor.as_unless_node().is_some()
            || ancestor.as_case_node().is_some()
            || ancestor.as_while_node().is_some()
            || ancestor.as_until_node().is_some()
            || ancestor.as_for_node().is_some()
    }) {
        return;
    }
    let body = scope
        .as_class_node()
        .and_then(|scope| scope.body())
        .or_else(|| scope.as_module_node().and_then(|scope| scope.body()));
    let Some(statements) = body.and_then(|body| body.as_statements_node()) else {
        return;
    };
    let mut visibility = None;
    for sibling in statements.body().iter() {
        if sibling.location().start_offset() >= node.location().start_offset() {
            break;
        }
        let Some(call) = sibling.as_call_node() else {
            continue;
        };
        if call.receiver().is_none()
            && call.block().is_none()
            && call
                .arguments()
                .is_none_or(|arguments| arguments.arguments().is_empty())
            && matches!(call.name().as_slice(), b"private" | b"protected" | b"public")
        {
            let line = context.source()[..call.location().start_offset()]
                .bytes()
                .filter(|byte| *byte == b'\n')
                .count()
                + 1;
            visibility = Some((String::from_utf8_lossy(call.name().as_slice()).into_owned(), line));
        }
    }
    let Some((modifier, line)) = visibility.filter(|(modifier, _)| modifier != "public") else {
        return;
    };
    let method = String::from_utf8_lossy(node.name().as_slice());
    let scope = context.ancestors().iter().rev().find(|ancestor| {
        ancestor.as_class_node().is_some() || ancestor.as_module_node().is_some()
    });
    if scope.is_some_and(|scope| {
        context.source_file().at(&scope.location()).lines().any(|line| {
            line.trim_start().starts_with("private_class_method ")
                && line.contains(&format!(":{method}"))
        })
    }) {
        return;
    }
    let advice = if modifier == "private" {
        "Use `private_class_method` or `private` inside a `class << self` block instead."
    } else {
        "Use `protected` inside a `class << self` block instead."
    };
    context.report(
        format!(
            "`{modifier}` (on line {line}) does not make singleton methods {modifier}. {advice}"
        ),
        node.def_keyword_loc(),
    );
}

fn def_with_parentheses(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    let (Some(opening), Some(closing)) = (node.lparen_loc(), node.rparen_loc()) else {
        return;
    };
    if node.parameters().is_some()
        || node.equal_loc().is_none() && !context.source_file().at(&node.location()).contains('\n')
        || node
            .equal_loc()
            .is_some_and(|equal| equal.start_offset() == closing.end_offset())
    {
        return;
    }
    context.remove(
        "Omit the parentheses in defs when the method doesn't accept any arguments.",
        opening.start_offset()..closing.end_offset(),
        opening.start_offset()..closing.end_offset(),
    );
}

fn missing_respond_to_missing(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.name().as_slice() != b"method_missing" {
        return;
    }
    let mut scopes = context.ancestors().iter().rev().filter(|ancestor| {
        ancestor.as_class_node().is_some()
            || ancestor.as_module_node().is_some()
            || ancestor.as_singleton_class_node().is_some()
            || ancestor.as_block_node().is_some()
    });
    let Some(mut scope) = scopes.next() else {
        return;
    };
    // Parser AST collapses a class/module body containing one definition, so
    // RuboCop's `node.parent.parent` search lands in the surrounding statement
    // container rather than the one-member nested class itself.
    if scope_has_only_definition(scope, node) {
        if let Some(outer_scope) = scopes.next() {
            scope = outer_scope;
        }
    }
    let body = if let Some(class) = scope.as_class_node() {
        class.body()
    } else if let Some(module) = scope.as_module_node() {
        module.body()
    } else if let Some(singleton) = scope.as_singleton_class_node() {
        singleton.body()
    } else if let Some(block) = scope.as_block_node() {
        block.body()
    } else {
        None
    };
    let Some(body) = body else {
        return;
    };
    let mut methods = ScopeMethods::default();
    methods.visit(&body);
    let singleton = node.receiver().is_some();
    if !(singleton && methods.singleton_respond || !singleton && methods.instance_respond) {
        context.report(
            "When using `method_missing`, define `respond_to_missing?`.",
            node.location(),
        );
    }
}

fn scope_has_only_definition(scope: &Node<'_>, definition: &ruby_prism::DefNode<'_>) -> bool {
    let body = if let Some(class) = scope.as_class_node() {
        class.body()
    } else if let Some(module) = scope.as_module_node() {
        module.body()
    } else if let Some(singleton) = scope.as_singleton_class_node() {
        singleton.body()
    } else {
        return false;
    };
    let Some(statements) = body.and_then(|body| body.as_statements_node()) else {
        return false;
    };
    let members = statements.body();
    members.len() == 1
        && members.first().is_some_and(|member| {
            member.location().start_offset() == definition.location().start_offset()
                && member.location().end_offset() == definition.location().end_offset()
        })
}

#[derive(Default)]
struct ScopeMethods {
    instance_respond: bool,
    singleton_respond: bool,
}

impl<'pr> Visit<'pr> for ScopeMethods {
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        let singleton = node.receiver().is_some();
        match node.name().as_slice() {
            b"respond_to_missing?" if singleton => self.singleton_respond = true,
            b"respond_to_missing?" => self.instance_respond = true,
            _ => {}
        }
    }

    fn visit_class_node(&mut self, _node: &ruby_prism::ClassNode<'pr>) {}

    fn visit_module_node(&mut self, _node: &ruby_prism::ModuleNode<'pr>) {}
}
