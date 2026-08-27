use super::*;
use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::cop::mixin::range_help::SurroundingSpace;

define_cops! {
    EmptyFileCompatibility => "Lint/EmptyFile" => compatibility_investigation(EmptyFileRule, on_new_investigation),
    StringHashKeysCompatibility => "Style/StringHashKeys" => compatibility_callbacks(StringHashKeysRule, [on_pair]),
    NestedTernaryOperatorCompatibility => "Style/NestedTernaryOperator" => compatibility_callbacks(NestedTernaryOperatorRule, [on_if]),
    ToJSONCompatibility => "Lint/ToJSON" => compatibility_callbacks(ToJSONRule, [on_def]),
    NumberedParametersCompatibility => "Style/NumberedParameters" => compatibility_callbacks(NumberedParametersRule, [on_numblock]),
    ColonMethodCallCompatibility => "Style/ColonMethodCall" => compatibility_callbacks(ColonMethodCallRule, [on_send]),
    EnsureReturnCompatibility => "Lint/EnsureReturn" => compatibility_callbacks(EnsureReturnRule, [on_ensure]),
    RedundantRegexpConstructorCompatibility => "Style/RedundantRegexpConstructor" => compatibility_callbacks(RedundantRegexpConstructorRule, [on_send restrict ["new", "compile"]]),
}

define_compatibility_rule!(ToJSONRule);
impl ToJSONRule<'_, '_, '_, '_> {
    fn on_def(&mut self, node: NodeRef<'_>) {
        if node.method_name() != Some("to_json") || !node.arguments().is_empty() {
            return;
        }
        let Some(name) = self.location_range(node, "name") else { return; };
        add_offense!(self, node, message: "`#to_json` requires an optional argument to be parsable via JSON.generate(obj).", |corrector| {
            corrector.insert_after(name, "(*_args)");
        });
    }
}

define_compatibility_rule!(NumberedParametersRule);
impl NumberedParametersRule<'_, '_, '_, '_> {
    fn on_numblock(&mut self, node: NodeRef<'_>) {
        if !self.target_ruby_version().at_least(2, 7) { return; }
        let style = self.policy().enforced_style("allow_single_line");
        if style == "disallow" {
            self.report("Avoid using numbered parameters.", node);
        } else if node.multiline() {
            self.report("Avoid using numbered parameters for multi-line blocks.", node);
        }
    }
}

define_compatibility_rule!(ColonMethodCallRule);
impl ColonMethodCallRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(receiver) = node.receiver() else { return; };
        if !node.loc_is("dot", "::")
            || node.method_name().is_some_and(|method| method.chars().next().is_some_and(char::is_uppercase))
            || root_constant(receiver, "Java")
            || receiver.call_type() && receiver.receiver().is_some_and(|node| root_constant(node, "Java"))
        {
            return;
        }
        let Some(operator) = self.location_range(node, "dot") else { return; };
        add_offense!(self, operator, message: "Do not use `::` for method calls.", |corrector| {
            corrector.replace(operator, ".");
        });
    }
}

define_compatibility_rule!(EnsureReturnRule);
impl EnsureReturnRule<'_, '_, '_, '_> {
    fn on_ensure(&mut self, node: NodeRef<'_>) {
        let Some(branch) = node.branch() else { return; };
        for return_node in branch.each_node(&["return"]) {
            self.report("Do not return from an `ensure` block.", return_node);
        }
    }
}

define_compatibility_rule!(RedundantRegexpConstructorRule);
impl RedundantRegexpConstructorRule<'_, '_, '_, '_> {
    fn on_send(&mut self, node: NodeRef<'_>) {
        let Some(method @ ("new" | "compile")) = node.method_name() else { return; };
        if !node.receiver().is_some_and(|node| root_constant(node, "Regexp")) { return; }
        let arguments = node.arguments();
        let Some(regexp) = (arguments.len() == 1).then(|| arguments[0]).filter(|node| node.kind() == "regexp") else { return; };
        let replacement = regexp.source().unwrap_or_default().to_owned();
        add_offense!(self, node, message: format!("Remove the redundant `Regexp.{method}`."), |corrector| {
            corrector.replace(node, replacement);
        });
    }
}

define_compatibility_rule!(EmptyFileRule);
impl EmptyFileRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let offending = self.source().is_empty()
            || !self.config_bool("AllowComments", true)
                && self
                    .processed_source()
                    .lines()
                    .iter()
                    .all(|line| line.trim().is_empty() || line.trim_start().starts_with('#'));
        if offending {
            let range = self.range_between(0, 0);
            self.report("Empty file detected.", range);
        }
    }
}

define_compatibility_rule!(StringHashKeysRule);
impl StringHashKeysRule<'_, '_, '_, '_> {
    fn on_pair(&mut self, node: NodeRef<'_>) {
        let Some(key) = node
            .key()
            .filter(|key| matches!(key.kind(), "str" | "__FILE__"))
        else {
            return;
        };
        let Some(key_content) = self.string_content(key) else {
            return;
        };
        let invalid_symbol_encoding = key_content.contains('\u{fffd}')
            && key.source().is_some_and(|source| {
                !source.contains("\\u") && !source.contains('\u{fffd}')
            });
        if invalid_symbol_encoding || receive_environments_method(node) {
            return;
        }
        let replacement = ruby_symbol_inspect(&key_content);
        add_offense!(self, key, message: "Prefer symbols instead of strings as hash keys.", |corrector| {
            corrector.replace(key, replacement);
        });
    }
}

fn receive_environments_method(pair: NodeRef<'_>) -> bool {
    let Some(hash) = pair.parent().filter(|node| node.kind() == "hash") else {
        return false;
    };
    if let Some(call) = hash.parent().filter(|node| node.call_type()) {
        let method = call.method_name().unwrap_or_default();
        let receiver = call.receiver();
        return (method == "popen" && receiver.is_some_and(|node| root_constant(node, "IO")))
            || (["capture2", "capture2e", "capture3", "popen2", "popen2e", "popen3"]
                .contains(&method)
                && receiver.is_some_and(|node| root_constant(node, "Open3")))
            || (["spawn", "system"].contains(&method)
                && receiver.is_none_or(|node| root_constant(node, "Kernel")))
            || ["gsub", "gsub!"].contains(&method);
    }
    let Some(array) = hash.parent().filter(|node| node.kind() == "array") else {
        return false;
    };
    let Some(call) = array.parent().filter(|node| node.call_type()) else {
        return false;
    };
    ["pipeline", "pipeline_r", "pipeline_rw", "pipeline_start", "pipeline_w"]
        .contains(&call.method_name().unwrap_or_default())
        && call.receiver().is_some_and(|node| root_constant(node, "Open3"))
}

fn root_constant(node: NodeRef<'_>, name: &str) -> bool {
    node.kind() == "const"
        && node.short_name() == Some(name)
        && node
            .namespace()
            .is_none_or(|namespace| namespace.kind() == "cbase")
}

fn ruby_symbol_inspect(value: &str) -> String {
    let identifier = |text: &str| {
        let mut characters = text.chars();
        characters
            .next()
            .is_some_and(|character| character == '_' || character.is_alphabetic())
            && characters.all(|character| character == '_' || character.is_alphanumeric())
    };
    let method = value
        .strip_suffix(['!', '?', '='])
        .filter(|method| identifier(method));
    let bare = identifier(value)
        || method.is_some()
        || matches!(
            value,
            "|" | "^" | "&" | "<=>" | "==" | "===" | "=~" | ">" | ">=" | "<" | "<="
                | "<<" | ">>" | "+" | "-" | "*" | "/" | "%" | "**" | "~" | "+@" | "-@"
                | "[]" | "[]=" | "`" | "!" | "!=" | "!~"
        );
    if bare {
        format!(":{value}")
    } else {
        format!(":{}", inspect_string(value))
    }
}

fn inspect_string(value: &str) -> String {
    let escaped = value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t");
    format!("\"{escaped}\"")
}

define_compatibility_rule!(NestedTernaryOperatorRule);
impl NestedTernaryOperatorRule<'_, '_, '_, '_> {
    fn on_if(&mut self, node: NodeRef<'_>) {
        if !node.ternary() {
            return;
        }
        let nested = node
            .each_descendant(&["if"])
            .into_iter()
            .filter(|descendant| descendant.ternary())
            .collect::<Vec<_>>();
        for (index, nested_ternary) in nested.into_iter().enumerate() {
            if index == 0 {
                let Some(question) = self.location_range(node, "question") else {
                    continue;
                };
                let Some(colon) = self.location_range(node, "colon") else {
                    continue;
                };
                let Some(if_branch) = node.if_branch() else {
                    continue;
                };
                let question = self.range_help().range_with_surrounding_space(
                    self.range_help()
                        .range_between(question.begin_pos(), question.end_pos()),
                    SurroundingSpace {
                        whitespace: true,
                        ..SurroundingSpace::default()
                    },
                );
                let question = self.owned_range(question);
                let colon = self.range_help().range_with_surrounding_space(
                    self.range_help()
                        .range_between(colon.begin_pos(), colon.end_pos()),
                    SurroundingSpace {
                        whitespace: true,
                        ..SurroundingSpace::default()
                    },
                );
                let colon = self.owned_range(colon);
                let branch_source = if_branch.source().unwrap_or_default();
                let branch_replacement = branch_source
                    .strip_prefix('(')
                    .and_then(|source| source.strip_suffix(')'))
                    .unwrap_or(branch_source)
                    .to_owned();
                add_offense!(self, nested_ternary, message: "Ternary operators must not be nested. Prefer `if` or `else` constructs instead.", |corrector| {
                    corrector.replace(question, "\n");
                    corrector.replace(colon, "\nelse\n");
                    corrector.replace(if_branch, branch_replacement);
                    corrector.wrap(node, "if ", "\nend");
                });
            } else {
                self.report(
                    "Ternary operators must not be nested. Prefer `if` or `else` constructs instead.",
                    nested_ternary,
                );
            }
        }
    }
}
