use ruby_prism::Node;

use super::*;

define_rule!(RegexpLiteralRule);

const MSG_USE_SLASHES: &str = "Use `//` around regular expression.";
const MSG_USE_PERCENT_R: &str = "Use `%r` around regular expression.";

define_cops! {
    RegexpLiteral => "Style/RegexpLiteral" => node_rule_aliases(RegexpLiteralRule, on_regexp => [as_regular_expression_node, as_interpolated_regular_expression_node]),
}

impl RegexpLiteralRule<'_, '_, '_> {
    fn on_regexp(&mut self, node: &Node<'_>) {
        let Some(regexp) = RegexpView::new(node, self.source()) else {
            return;
        };
        let slash_literal = self.source_file().at(&regexp.opening) == "/";
        return_if!(slash_literal && self.percent_r_delimiters_conflict(&regexp.body));

        let style = self.policy().enforced_style("slashes");
        let disallowed_slash = !self.config_bool("AllowInnerSlashes", false)
            && regexp.body.contains('/');
        let multiline = self
            .source()
            .get(regexp.location.clone())
            .is_some_and(|source| source.contains('\n'));
        let message = if slash_literal {
            let allowed = style == "slashes" && !disallowed_slash
                || style == "mixed" && !multiline && !disallowed_slash;
            (!allowed).then_some(MSG_USE_PERCENT_R)
        } else {
            let allowed = style == "percent_r"
                || style == "slashes" && disallowed_slash
                || style == "mixed" && multiline
                || disallowed_slash
                || self.allowed_omit_parentheses(&regexp.body);
            (!allowed).then_some(MSG_USE_SLASHES)
        };
        let Some(message) = message else {
            return;
        };

        let (opening, closing, target_uses_slashes) = if slash_literal {
            let (open, close) = self.preferred_delimiters();
            (format!("%r{open}"), close.to_string(), open == '/')
        } else {
            ("/".to_string(), "/".to_string(), true)
        };
        let closing_source = self.source_file().at(&regexp.closing);
        let closing = format!(
            "{closing}{}",
            closing_source.get(1..).unwrap_or_default()
        );
        let part_edits = regexp
            .parts
            .iter()
            .filter_map(|part| {
                let source = self.source().get(part.clone()).unwrap_or_default();
                let replacement = if slash_literal && !target_uses_slashes {
                    source.replace("\\/", "/")
                } else if !slash_literal && target_uses_slashes {
                    escape_unescaped_slashes(source)
                } else {
                    source.to_string()
                };
                (replacement != source).then(|| (part.clone(), replacement))
            })
            .collect::<Vec<_>>();
        add_offense!(self, regexp.location.clone(), message: message, |corrector| {
            corrector.replace(regexp.opening, opening);
            corrector.replace(regexp.closing, closing);
            for (part, replacement) in part_edits {
                corrector.replace(part, replacement);
            }
        });
    }

    fn preferred_delimiters(&self) -> (char, char) {
        let configured = self
            .related_config_map("Style/PercentLiteralDelimiters", "PreferredDelimiters")
            .and_then(|values| values.get("%r").or_else(|| values.get("default")))
            .map(String::as_str)
            .unwrap_or("{}");
        let mut characters = configured.chars();
        let opening = characters.next().unwrap_or('{');
        let closing = characters.next().unwrap_or(opening);
        (opening, closing)
    }

    fn percent_r_delimiters_conflict(&self, body: &str) -> bool {
        let (opening, closing) = self.preferred_delimiters();
        if !matches!((opening, closing), ('(', ')') | ('[', ']') | ('{', '}') | ('<', '>')) {
            return false;
        }
        let mut depth = 0_i32;
        let mut characters = body.chars();
        while let Some(character) = characters.next() {
            if character == '\\' {
                characters.next();
            } else if character == opening {
                depth += 1;
            } else if character == closing {
                depth -= 1;
                if depth < 0 {
                    return true;
                }
            }
        }
        depth != 0
    }

    fn allowed_omit_parentheses(&self, body: &str) -> bool {
        let call_parent = self.parent().and_then(Node::as_call_node).is_some();
        call_parent
            && (body.starts_with([' ', '='])
                || self.related_config_value(
                    "Style/MethodCallWithArgsParentheses",
                    "EnforcedStyle",
                ) == Some("omit_parentheses"))
    }
}

struct RegexpView<'pr> {
    location: std::ops::Range<usize>,
    opening: ruby_prism::Location<'pr>,
    closing: ruby_prism::Location<'pr>,
    parts: Vec<std::ops::Range<usize>>,
    body: String,
}

impl<'pr> RegexpView<'pr> {
    fn new(node: &Node<'pr>, source: &str) -> Option<Self> {
        let (location, opening, closing, parts) = if let Some(regexp) = node.as_regular_expression_node() {
            let content = regexp.content_loc();
            (
                regexp.location(),
                regexp.opening_loc(),
                regexp.closing_loc(),
                vec![content.start_offset()..content.end_offset()],
            )
        } else {
            let regexp = node.as_interpolated_regular_expression_node()?;
            let parts = regexp
                .parts()
                .iter()
                .filter_map(|part| {
                    part.as_string_node().map(|string| {
                        let location = string.location();
                        location.start_offset()..location.end_offset()
                    })
                })
                .collect();
            (
                regexp.location(),
                regexp.opening_loc(),
                regexp.closing_loc(),
                parts,
            )
        };
        let body = parts
            .iter()
            .filter_map(|part| source.get(part.clone()))
            .collect::<String>();
        Some(Self {
            location: location.start_offset()..location.end_offset(),
            opening,
            closing,
            parts,
            body,
        })
    }
}

fn escape_unescaped_slashes(source: &str) -> String {
    let mut output = String::with_capacity(source.len());
    let mut escaped = false;
    for character in source.chars() {
        if character == '/' && !escaped {
            output.push('\\');
        }
        output.push(character);
        if character == '\\' {
            escaped = !escaped;
        } else {
            escaped = false;
        }
    }
    output
}
