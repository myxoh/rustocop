use super::*;

define_cops! {
    IneffectiveAccessModifier => "Lint/IneffectiveAccessModifier" => node(as_def_node, ineffective_access_modifier),
    DefWithParentheses => "Style/DefWithParentheses" => node(as_def_node, def_with_parentheses),
    MissingRespondToMissing => "Style/MissingRespondToMissing" => any_node(missing_respond_to_missing),
}

fn ineffective_access_modifier(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if node
        .receiver()
        .is_none_or(|receiver| receiver.as_self_node().is_none())
        || context
            .ancestors()
            .iter()
            .any(|ancestor| ancestor.as_singleton_class_node().is_some())
    {
        return;
    }
    let Some(class) = context
        .ancestors()
        .iter()
        .rev()
        .find_map(Node::as_class_node)
    else {
        return;
    };
    let before = context
        .source()
        .get(class.location().start_offset()..node.location().start_offset())
        .unwrap_or_default();
    let Some((line_index, modifier)) = before
        .lines()
        .enumerate()
        .filter(|(_, line)| matches!(line.trim(), "private" | "protected"))
        .last()
    else {
        return;
    };
    let method = String::from_utf8_lossy(node.name().as_slice());
    if modifier.trim() == "private" {
        let class_source = context.source_file().at(&class.location());
        if class_source.contains(&format!("private_class_method :{method}")) {
            return;
        }
    }
    let line = context.source()[..class.location().start_offset()]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + line_index
        + 1;
    let modifier = modifier.trim();
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

fn missing_respond_to_missing(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let body = if let Some(class) = node.as_class_node() {
        class.body()
    } else if let Some(module) = node.as_module_node() {
        module.body()
    } else {
        return;
    };
    let Some(body) = body else {
        return;
    };
    let mut methods = ScopeMethods::default();
    methods.visit(&body);
    for (location, singleton) in methods.missing {
        if singleton && methods.singleton_respond || !singleton && methods.instance_respond {
            continue;
        }
        context.report(
            "When using `method_missing`, define `respond_to_missing?`.",
            location,
        );
    }
}

#[derive(Default)]
struct ScopeMethods<'pr> {
    missing: Vec<(ruby_prism::Location<'pr>, bool)>,
    instance_respond: bool,
    singleton_respond: bool,
}

impl<'pr> Visit<'pr> for ScopeMethods<'pr> {
    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        let singleton = node.receiver().is_some();
        match node.name().as_slice() {
            b"method_missing" => self.missing.push((node.location(), singleton)),
            b"respond_to_missing?" if singleton => self.singleton_respond = true,
            b"respond_to_missing?" => self.instance_respond = true,
            _ => {}
        }
    }

    fn visit_class_node(&mut self, _node: &ruby_prism::ClassNode<'pr>) {}

    fn visit_module_node(&mut self, _node: &ruby_prism::ModuleNode<'pr>) {}
}
