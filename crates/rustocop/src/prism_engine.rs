use ruby_prism::{parse, CallNode, Location, Node, Visit};

mod layout;
mod lint;
mod security;
mod style;
mod style_compat;

pub const PRISM_COPS: &[&str] = &[
    "Lint/BooleanSymbol",
    "Lint/EmptyExpression",
    "Lint/FlipFlop",
    "Lint/FloatComparison",
    "Lint/IdentityComparison",
    "Lint/SelfAssignment",
    "Lint/ToJSON",
    "Layout/SpaceAfterColon",
    "Security/Eval",
    "Security/CompoundHash",
    "Security/JSONLoad",
    "Security/MarshalLoad",
    "Security/Open",
    "Security/IoMethods",
    "Style/CharacterLiteral",
    "Style/BeginBlock",
    "Style/DefWithParentheses",
    "Style/MethodCallWithoutArgsParentheses",
    "Style/NilComparison",
    "Style/Not",
    "Style/RedundantArrayConstructor",
    "Style/RedundantFreeze",
    "Style/Semicolon",
    "Style/StringChars",
    "Style/StringMethods",
    "Style/UnlessElse",
    "Style/FileTouch",
    "Style/GlobalStdStream",
    "Style/MinMax",
    "Style/RedundantFileExtensionInRequire",
    "Style/SuperWithArgsParentheses",
    "Style/TrailingCommaInBlockArgs",
    "Style/WhileUntilDo",
];

#[derive(Debug)]
pub struct Finding {
    pub cop_name: &'static str,
    pub message: String,
    pub correctable: bool,
    pub corrected: bool,
    pub start_offset: usize,
    pub end_offset: usize,
}

#[derive(Debug)]
pub struct Inspection {
    pub findings: Vec<Finding>,
    pub corrected_source: String,
}

#[derive(Debug)]
struct Edit {
    start_offset: usize,
    end_offset: usize,
    replacement: String,
}

pub(super) struct Context {
    autocorrect: bool,
    findings: Vec<Finding>,
    edits: Vec<Edit>,
}

impl Context {
    pub(super) fn add_offense(
        &mut self,
        cop_name: &'static str,
        message: String,
        location: Location<'_>,
        correction: Option<(Location<'_>, &str)>,
    ) {
        let correctable = correction.is_some();
        let corrected = self.autocorrect && correctable;

        if let Some((edit_location, replacement)) = correction.filter(|_| self.autocorrect) {
            self.edits.push(Edit {
                start_offset: edit_location.start_offset(),
                end_offset: edit_location.end_offset(),
                replacement: replacement.to_string(),
            });
        }

        self.findings.push(Finding {
            cop_name,
            message,
            correctable,
            corrected,
            start_offset: location.start_offset(),
            end_offset: location.end_offset(),
        });
    }

    pub(super) fn add_offense_offsets(
        &mut self,
        cop_name: &'static str,
        message: String,
        start_offset: usize,
        end_offset: usize,
        correction: Option<(usize, usize, String)>,
    ) {
        let correctable = correction.is_some();
        let corrected = self.autocorrect && correctable;

        if let Some((edit_start, edit_end, replacement)) = correction.filter(|_| self.autocorrect) {
            self.edits.push(Edit {
                start_offset: edit_start,
                end_offset: edit_end,
                replacement,
            });
        }

        self.findings.push(Finding {
            cop_name,
            message,
            correctable,
            corrected,
            start_offset,
            end_offset,
        });
    }
}

pub(super) trait Cop {
    fn name(&self) -> &'static str;

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        if let Some(call) = node.as_call_node() {
            self.on_call(&call, context);
        }
    }

    fn on_call(&self, _node: &CallNode<'_>, _context: &mut Context) {}
}

struct Registry {
    cops: Vec<Box<dyn Cop>>,
}

impl Registry {
    fn enabled(enabled: &dyn Fn(&str) -> bool) -> Self {
        let cops = lint::cops()
            .into_iter()
            .chain(layout::cops())
            .chain(security::cops())
            .chain(style::cops())
            .chain(style_compat::cops());

        Self {
            cops: cops.filter(|cop| enabled(cop.name())).collect(),
        }
    }
}

struct Runner<'registry, 'context> {
    registry: &'registry Registry,
    context: &'context mut Context,
    source: &'context str,
    ancestors: Vec<Node<'context>>,
}

impl<'pr> Visit<'pr> for Runner<'_, 'pr> {
    fn visit_branch_node_enter(&mut self, node: Node<'pr>) {
        for cop in &self.registry.cops {
            cop.on_node(&node, &self.ancestors, self.source, self.context);
        }
        self.ancestors.push(node);
    }

    fn visit_branch_node_leave(&mut self) {
        self.ancestors.pop();
    }

    fn visit_leaf_node_enter(&mut self, node: Node<'pr>) {
        for cop in &self.registry.cops {
            cop.on_node(&node, &self.ancestors, self.source, self.context);
        }
    }
}

pub fn inspect(source: &str, autocorrect: bool, enabled: &dyn Fn(&str) -> bool) -> Inspection {
    let parsed = parse(source.as_bytes());
    let registry = Registry::enabled(enabled);
    debug_assert!(registry
        .cops
        .iter()
        .all(|cop| PRISM_COPS.contains(&cop.name())));
    let mut context = Context {
        autocorrect,
        findings: Vec::new(),
        edits: Vec::new(),
    };

    let mut runner = Runner {
        registry: &registry,
        context: &mut context,
        source,
        ancestors: Vec::new(),
    };
    runner.visit(&parsed.node());

    context
        .findings
        .sort_by_key(|finding| (finding.start_offset, finding.end_offset, finding.cop_name));

    let corrected_source = apply_edits(source, context.edits);

    Inspection {
        findings: context.findings,
        corrected_source,
    }
}

fn apply_edits(source: &str, mut edits: Vec<Edit>) -> String {
    edits.sort_by_key(|edit| (edit.start_offset, edit.end_offset));

    let mut accepted = Vec::with_capacity(edits.len());
    let mut previous_end = 0;
    for edit in edits {
        if edit.start_offset < previous_end || edit.end_offset > source.len() {
            continue;
        }
        previous_end = edit.end_offset;
        accepted.push(edit);
    }

    let mut corrected = source.to_string();
    for edit in accepted.into_iter().rev() {
        corrected.replace_range(edit.start_offset..edit.end_offset, &edit.replacement);
    }
    corrected
}

pub(super) fn call_name<'pr>(node: &CallNode<'pr>) -> &'pr [u8] {
    node.name().as_slice()
}

pub(super) fn first_argument<'pr>(node: &CallNode<'pr>) -> Option<Node<'pr>> {
    node.arguments()?.arguments().first()
}

pub(super) fn eval_receiver(receiver: Option<Node<'_>>) -> bool {
    let Some(receiver) = receiver else {
        return true;
    };

    if node_is_root_constant(&receiver, b"Kernel") {
        return true;
    }

    receiver.as_call_node().is_some_and(|call| {
        call.receiver().is_none() && call.arguments().is_none() && call_name(&call) == b"binding"
    })
}

pub(super) fn root_constant(receiver: Option<Node<'_>>, expected: &[u8]) -> bool {
    receiver
        .as_ref()
        .is_some_and(|receiver| node_is_root_constant(receiver, expected))
}

pub(super) fn node_is_root_constant(receiver: &Node<'_>, expected: &[u8]) -> bool {
    if let Some(constant) = receiver.as_constant_read_node() {
        return constant.name().as_slice() == expected;
    }

    receiver.as_constant_path_node().is_some_and(|constant| {
        constant.parent().is_none()
            && constant
                .name()
                .is_some_and(|name| name.as_slice() == expected)
    })
}

pub(super) fn constant_read(receiver: Option<Node<'_>>, expected: &[u8]) -> bool {
    receiver
        .and_then(|node| node.as_constant_read_node())
        .is_some_and(|constant| constant.name().as_slice() == expected)
}

pub(super) fn marshal_dump(node: &Node<'_>) -> bool {
    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"dump" && root_constant(call.receiver(), b"Marshal")
    })
}

pub(super) fn has_keyword(node: &CallNode<'_>, expected: &[u8]) -> bool {
    let Some(arguments) = node.arguments() else {
        return false;
    };

    arguments.arguments().iter().any(|argument| {
        argument
            .as_keyword_hash_node()
            .is_some_and(|hash| keyword_hash_contains(&hash, expected))
    })
}

pub(super) fn keyword_hash_contains(
    hash: &ruby_prism::KeywordHashNode<'_>,
    expected: &[u8],
) -> bool {
    hash.elements().iter().any(|element| {
        element
            .as_assoc_node()
            .and_then(|association| association.key().as_symbol_node())
            .is_some_and(|symbol| symbol.unescaped() == expected)
    })
}

pub(super) fn recursive_literal_string(node: &Node<'_>) -> bool {
    node.as_interpolated_string_node().is_some_and(|string| {
        string.parts().iter().all(|part| {
            part.as_string_node().is_some()
                || part.as_embedded_statements_node().is_some_and(|embedded| {
                    embedded
                        .statements()
                        .is_some_and(|statements| statements.body().iter().all(literal_node))
                })
        })
    })
}

pub(super) fn literal_node(node: Node<'_>) -> bool {
    node.as_string_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
}

pub(super) fn safe_open_argument(node: &Node<'_>) -> bool {
    if let Some(string) = node.as_string_node() {
        return safe_open_text(string.unescaped());
    }

    if let Some(string) = node.as_interpolated_string_node() {
        return string
            .parts()
            .first()
            .is_some_and(|part| safe_open_argument(&part));
    }

    node.as_call_node().is_some_and(|call| {
        call_name(&call) == b"+"
            && call
                .receiver()
                .is_some_and(|receiver| safe_open_argument(&receiver))
    })
}

pub(super) fn safe_open_text(text: &[u8]) -> bool {
    !text.is_empty() && !text.starts_with(b"|")
}

pub(super) fn string_starts_with_pipe(node: &Node<'_>) -> bool {
    node.as_string_node()
        .is_some_and(|string| trim_ascii_whitespace(string.unescaped()).starts_with(b"|"))
}

pub(super) fn trim_ascii_whitespace(mut value: &[u8]) -> &[u8] {
    while value.first().is_some_and(u8::is_ascii_whitespace) {
        value = &value[1..];
    }
    while value.last().is_some_and(u8::is_ascii_whitespace) {
        value = &value[..value.len() - 1];
    }
    value
}

pub(super) fn source_at<'source>(source: &'source str, location: &Location<'_>) -> &'source str {
    &source[location.start_offset()..location.end_offset()]
}

pub(super) fn literal_zero(node: Option<&Node<'_>>) -> bool {
    let Some(node) = node else {
        return false;
    };
    if let Some(integer) = node.as_integer_node() {
        return integer
            .value()
            .to_u32_digits()
            .1
            .iter()
            .all(|digit| *digit == 0);
    }
    node.as_float_node()
        .is_some_and(|float| float.value() == 0.0)
}

pub(super) fn float_expression(node: Option<&Node<'_>>) -> bool {
    let Some(node) = node else {
        return false;
    };
    if node.as_float_node().is_some() {
        return true;
    }
    node.as_call_node().is_some_and(|call| {
        matches!(call_name(&call), b"to_f" | b"fdiv" | b"Float")
            || matches!(call_name(&call), b"+" | b"-" | b"*" | b"**" | b"/" | b"%")
                && (float_expression(call.receiver().as_ref())
                    || first_argument(&call)
                        .as_ref()
                        .is_some_and(|arg| float_expression(Some(arg))))
    })
}

pub(super) fn immutable_literal(node: &Node<'_>) -> bool {
    node.as_integer_node().is_some()
        || node.as_float_node().is_some()
        || node.as_symbol_node().is_some()
        || node.as_true_node().is_some()
        || node.as_false_node().is_some()
        || node.as_nil_node().is_some()
        || node.as_regular_expression_node().is_some()
        || node.as_range_node().is_some()
}

pub(super) fn semicolon_offsets(source: &str) -> Vec<usize> {
    let bytes = source.as_bytes();
    let mut offsets = Vec::new();
    let mut index = 0;
    let mut quote = None;
    let mut escaped = false;
    let mut comment = false;

    while index < bytes.len() {
        let byte = bytes[index];
        if comment {
            if byte == b'\n' {
                comment = false;
            }
            index += 1;
            continue;
        }
        if let Some(delimiter) = quote {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' => quote = Some(byte),
            b'#' => comment = true,
            b';' => offsets.push(index),
            _ => {}
        }
        index += 1;
    }
    offsets
}

pub(super) fn correct_unless_else(source: &str) -> String {
    let Some((before_else, after_else)) = source.split_once("\nelse\n") else {
        return source.replacen("unless", "if", 1);
    };
    let Some((header, body)) = before_else.split_once('\n') else {
        return source.replacen("unless", "if", 1);
    };
    let else_body = after_else.strip_suffix("\nend").unwrap_or(after_else);
    format!(
        "{}\n{}\nelse\n{}\nend",
        header.replacen("unless", "if", 1),
        else_body,
        body
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_once_and_dispatches_to_enabled_cops() {
        let inspection = inspect("eval(code)\nJSON.load(payload)\n", false, &|cop| {
            matches!(cop, "Security/Eval" | "Security/JSONLoad")
        });

        assert_eq!(inspection.findings.len(), 2);
        assert_eq!(inspection.findings[0].cop_name, "Security/Eval");
        assert_eq!(inspection.findings[1].cop_name, "Security/JSONLoad");
    }

    #[test]
    fn applies_non_overlapping_autocorrections_from_the_shared_tree() {
        let inspection = inspect("JSON.load(payload)\nIO.read(path)\n", true, &|cop| {
            matches!(cop, "Security/JSONLoad" | "Security/IoMethods")
        });

        assert_eq!(
            inspection.corrected_source,
            "JSON.parse(payload)\nFile.read(path)\n"
        );
        assert!(inspection.findings.iter().all(|finding| finding.corrected));
    }

    #[test]
    fn applies_compatibility_batch_corrections_from_one_tree() {
        let source = concat!(
            "{a:3}\n",
            "STDOUT.puts('hello')\n",
            "require 'foo.rb'\n",
            "super name, age\n",
            "test { |a, b,| a + b }\n",
            "while cond do\nend\n",
        );
        let inspection = inspect(source, true, &|cop| {
            matches!(
                cop,
                "Layout/SpaceAfterColon"
                    | "Style/GlobalStdStream"
                    | "Style/RedundantFileExtensionInRequire"
                    | "Style/SuperWithArgsParentheses"
                    | "Style/TrailingCommaInBlockArgs"
                    | "Style/WhileUntilDo"
            )
        });

        assert_eq!(inspection.findings.len(), 6);
        assert_eq!(
            inspection.corrected_source,
            concat!(
                "{a: 3}\n",
                "$stdout.puts('hello')\n",
                "require 'foo'\n",
                "super(name, age)\n",
                "test { |a, b| a + b }\n",
                "while cond\nend\n",
            )
        );
    }

    #[test]
    fn replaces_empty_append_file_open_block() {
        let inspection = inspect("File.open(filename, 'a') {}\n", true, &|cop| {
            cop == "Style/FileTouch"
        });

        assert_eq!(inspection.findings.len(), 1);
        assert_eq!(inspection.corrected_source, "FileUtils.touch(filename)\n");
    }
}
