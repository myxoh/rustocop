use super::*;

define_cops! {
    Syntax => "Lint/Syntax" => parse_error_and_source(syntax, invalid_byte_syntax),
    FormatParameterMismatch => "Lint/FormatParameterMismatch" => node(as_call_node, format_parameter_mismatch),
    UnusedBlockArgument => "Lint/UnusedBlockArgument" => any_node(unused_block_argument),
    AmbiguousRange => "Lint/AmbiguousRange" => source(ambiguous_range),
    NonAtomicFileOperation => "Lint/NonAtomicFileOperation" => source(non_atomic_file_operation),
    UnmodifiedReduceAccumulator => "Lint/UnmodifiedReduceAccumulator" => node(as_block_node, unmodified_reduce_accumulator),
    DocumentationMethod => "Style/DocumentationMethod" => source(documentation_method),
    RedundantSplatExpansion => "Lint/RedundantSplatExpansion" => node(as_splat_node, redundant_splat_expansion),
}

fn syntax(error: &Diagnostic<'_>, context: &mut CopContext<'_, '_>) {
    if context.related_config_value("AllCops", "ParserEngine") == Some("parser_prism") {
        return;
    }
    if context.source().contains('\n')
        && !error.message().contains("end-of-input")
        && !context.source().trim_end().ends_with('(')
    {
        return;
    }
    let start = context.source().len().saturating_sub(1);
    let version = context.target_ruby_version();
    context.report(
        format!(
            "unexpected token $end\n(Using Ruby {}.{} parser; configure using `TargetRubyVersion` parameter, under `AllCops`)",
            version.major(),
            version.minor()
        ),
        start..context.source().len(),
    );
}

fn prism_syntax_message(error: &Diagnostic<'_>) -> String {
    let mut message = error.message().to_string();
    if message == "unexpected ',', ignoring it" {
        message = "unexpected ',', expecting end-of-input".to_string();
    } else if message
        == "unexpected constant path after `class`; class/module name must be CONSTANT"
    {
        message = "class or module name must be a constant literal".to_string();
    } else if message.ends_with(", ignoring it") {
        let token = std::str::from_utf8(error.location().as_slice()).unwrap_or_default();
        message = format!("unexpected token {token}");
    }
    message
}

fn invalid_byte_syntax(context: &mut CopContext<'_, '_>) {
    if context.related_config_value("AllCops", "ParserEngine") == Some("parser_prism") {
        let parsed = ruby_prism::parse(context.source().as_bytes());
        let errors = parsed.errors().collect::<Vec<_>>();
        let unterminated = errors
            .iter()
            .find(|error| error.message() == "unterminated string meets end of file")
            .map(|error| error.location().start_offset());
        let mut seen = std::collections::HashSet::new();
        let version = context.target_ruby_version();
        for error in errors {
            let location = error.location();
            let start = location.start_offset();
            let end = location.end_offset();
            if unterminated.is_some_and(|at| start > at) || !seen.insert((start, end)) {
                continue;
            }
            let range = if start == end {
                if end < context.source().len() {
                    start..end + 1
                } else {
                    start.saturating_sub(1)..end
                }
            } else {
                start..end
            };
            context.report(
                format!(
                    "{}\n(Using Ruby {}.{} parser; configure using `TargetRubyVersion` parameter, under `AllCops`)",
                    prism_syntax_message(&error),
                    version.major(),
                    version.minor()
                ),
                range,
            );
        }
    } else if context
        .source()
        .bytes()
        .any(|byte| byte < b' ' && !matches!(byte, b'\t' | b'\n' | b'\r'))
    {
        context.report("Invalid byte sequence in utf-8.", 0..0);
    }
}

fn format_parameter_mismatch(node: &ruby_prism::CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let name = call_name(node);
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let (format_source, dynamic_format, supplied, method) =
        if matches!(name, b"format" | b"sprintf") {
            if arguments.len() < 2
                || !literal_format_string(&arguments[0])
                || node
                    .receiver()
                    .is_some_and(|receiver| !kernel_receiver(&receiver))
                || arguments[1..]
                    .iter()
                    .any(|argument| argument.as_splat_node().is_some())
            {
                return;
            }
            (
                context.source_file().node(&arguments[0]).to_owned(),
                arguments[0].as_interpolated_string_node().is_some(),
                arguments.len() - 1,
                std::str::from_utf8(name).unwrap(),
            )
        } else if name == b"%" {
            let Some(receiver) = node.receiver() else {
                return;
            };
            let Some(argument) = arguments.first() else {
                return;
            };
            let Some(array) = argument.as_array_node() else {
                return;
            };
            if !literal_format_string(&receiver) {
                return;
            }
            (
                context.source_file().node(&receiver).to_owned(),
                receiver.as_interpolated_string_node().is_some(),
                array.elements().len(),
                "String#%",
            )
        } else {
            return;
        };
    let Some(fields) = count_format_fields(&format_source) else {
        let selector = node.message_loc().unwrap_or_else(|| node.location());
        context.report(
            "Format string is invalid because formatting sequence types (numbered, named or unnumbered) are mixed.",
            selector.start_offset()..selector.end_offset(),
        );
        return;
    };
    // RuboCop cannot determine the cardinality of a dynamic string/array used
    // as the sole `%` operand, or a dynamic format string with no fields.
    if name == b"%" && supplied == 1 {
        return;
    }
    if fields == 0 && dynamic_format {
        return;
    }
    if supplied != fields {
        let selector = node.message_loc().unwrap_or_else(|| node.location());
        context.report(
            format!(
                "Number of arguments ({supplied}) to `{method}` doesn't match the number of fields ({fields})."
            ),
            selector.start_offset()..selector.end_offset(),
        );
    }
}

fn literal_format_string(node: &Node<'_>) -> bool {
    node.as_string_node().is_some() || node.as_interpolated_string_node().is_some()
}

fn kernel_receiver(node: &Node<'_>) -> bool {
    node.as_constant_read_node()
        .is_some_and(|constant| constant.name().as_slice() == b"Kernel")
}

/// Returns the number of arguments consumed by a Ruby format string, or
/// `None` when numbered/named/unnumbered sequences are mixed.
fn count_format_fields(source: &str) -> Option<usize> {
    #[derive(Clone, Copy, PartialEq, Eq)]
    enum Mode {
        Unnumbered,
        Numbered,
        Named,
    }
    let bytes = source.as_bytes();
    let mut index = 0;
    let mut count: usize = 0;
    let mut numbered_max: usize = 0;
    let mut mode = None;
    while index < bytes.len() {
        if bytes[index] != b'%' {
            index += 1;
            continue;
        }
        index += 1;
        if bytes.get(index) == Some(&b'%') {
            index += 1;
            continue;
        }
        let mut named_probe = index;
        while bytes.get(named_probe).is_some_and(|byte| {
            byte.is_ascii_digit()
                || matches!(byte, b'0' | b'-' | b'+' | b' ' | b'.')
                || *byte == b'#' && bytes.get(named_probe + 1) != Some(&b'{')
        }) {
            named_probe += 1;
        }
        if matches!(bytes.get(named_probe), Some(b'{') | Some(b'<')) {
            index = named_probe;
        }
        if matches!(bytes.get(index), Some(b'{') | Some(b'<')) {
            let closing = if bytes[index] == b'{' { b'}' } else { b'>' };
            index += 1;
            while index < bytes.len() && bytes[index] != closing {
                index += 1;
            }
            if index == bytes.len() {
                continue;
            }
            index += 1;
            if closing == b'>' {
                while bytes
                    .get(index)
                    .is_some_and(|byte| matches!(byte, b'#' | b'0' | b'-' | b'+' | b' '))
                {
                    index += 1;
                }
                while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                    index += 1;
                }
                if bytes.get(index) == Some(&b'.') {
                    index += 1;
                    while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                        index += 1;
                    }
                }
                if !bytes
                    .get(index)
                    .is_some_and(|byte| format_conversion(*byte))
                {
                    continue;
                }
                index += 1;
            }
            if mode.is_some_and(|current| current != Mode::Named) {
                return None;
            }
            mode = Some(Mode::Named);
            count = 1;
            continue;
        }
        let digits_start = index;
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if index > digits_start && bytes.get(index) == Some(&b'$') {
            let number = std::str::from_utf8(&bytes[digits_start..index])
                .ok()?
                .parse::<usize>()
                .ok()?;
            index += 1;
            if mode.is_some_and(|current| current != Mode::Numbered) {
                return None;
            }
            mode = Some(Mode::Numbered);
            numbered_max = numbered_max.max(number);
        } else {
            index = digits_start;
            if mode.is_some_and(|current| current != Mode::Unnumbered) {
                return None;
            }
            mode = Some(Mode::Unnumbered);
            count += 1;
        }
        // Flags and width. Each `*` consumes an additional argument.
        while bytes.get(index).is_some_and(|byte| {
            matches!(byte, b'0' | b'-' | b'+' | b' ')
                || *byte == b'#' && bytes.get(index + 1) != Some(&b'{')
        }) {
            index += 1;
        }
        if bytes.get(index) == Some(&b'*') {
            count += 1;
            index += 1;
        }
        while bytes.get(index).is_some_and(u8::is_ascii_digit) {
            index += 1;
        }
        if bytes.get(index) == Some(&b'.') {
            index += 1;
            if bytes.get(index) == Some(&b'*') {
                count += 1;
                index += 1;
            }
            while bytes.get(index).is_some_and(u8::is_ascii_digit) {
                index += 1;
            }
        }
        // Interpolated widths (`%#{width}s`) still describe one conversion.
        if bytes.get(index) == Some(&b'#') && bytes.get(index + 1) == Some(&b'{') {
            index += 2;
            let mut depth = 1;
            while index < bytes.len() && depth > 0 {
                match bytes[index] {
                    b'{' => depth += 1,
                    b'}' => depth -= 1,
                    _ => {}
                }
                index += 1;
            }
        }
        if !bytes
            .get(index)
            .is_some_and(|byte| format_conversion(*byte))
        {
            // A lone `%` (or unrelated percent text) is not a field.
            if mode == Some(Mode::Unnumbered) {
                count = count.saturating_sub(1);
            }
            continue;
        }
        index += 1;
    }
    Some(if mode == Some(Mode::Numbered) {
        numbered_max
    } else {
        count
    })
}

fn format_conversion(byte: u8) -> bool {
    matches!(
        byte,
        b'b' | b'B'
            | b'c'
            | b'd'
            | b'e'
            | b'E'
            | b'f'
            | b'g'
            | b'G'
            | b'i'
            | b'o'
            | b'p'
            | b's'
            | b'u'
            | b'x'
            | b'X'
    )
}

fn unused_block_argument(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let ignore_empty = context.config_bool("IgnoreEmptyBlocks", true);
    let allow_keywords = context.config_bool("AllowUnusedKeywordArguments", false);
    let (parameters, body, lambda, define_method) = if let Some(block) = node.as_block_node() {
        let Some(block_parameters) = block
            .parameters()
            .and_then(|parameters| parameters.as_block_parameters_node())
        else {
            return;
        };
        let mut parameters = block_parameters
            .parameters()
            .map(|parameters| block_parameter_infos(&parameters))
            .unwrap_or_default();
        parameters.extend(block_parameters.locals().iter().filter_map(|local| {
            local
                .as_block_local_variable_node()
                .map(|local| BlockParameterInfo {
                    name: String::from_utf8_lossy(local.name().as_slice()).into_owned(),
                    range: local.location().start_offset()..local.location().end_offset(),
                    keyword: false,
                    local: true,
                })
        }));
        let define_method = context
            .parent()
            .and_then(Node::as_call_node)
            .is_some_and(|call| call.name().as_slice() == b"define_method");
        let lambda = context
            .parent()
            .and_then(Node::as_call_node)
            .is_some_and(|call| call.name().as_slice() == b"lambda");
        (parameters, block.body(), lambda, define_method)
    } else if let Some(lambda_node) = node.as_lambda_node() {
        let parameters = lambda_node
            .parameters()
            .and_then(|parameters| {
                parameters.as_parameters_node().or_else(|| {
                    parameters
                        .as_block_parameters_node()
                        .and_then(|block| block.parameters())
                })
            })
            .map(|parameters| block_parameter_infos(&parameters))
            .unwrap_or_default();
        (parameters, lambda_node.body(), true, false)
    } else {
        return;
    };
    if parameters.is_empty() {
        return;
    }
    let Some(body) = body else {
        if ignore_empty {
            return;
        }
        let unused = (0..parameters.len()).collect::<Vec<_>>();
        return report_unused_block_parameters(
            context,
            &parameters,
            &unused,
            lambda,
            define_method,
        );
    };
    let body_source = context.source_file().node(&body);
    if ignore_empty && body_source.trim().is_empty() {
        return;
    }
    let mut reads = BlockParameterReads::default();
    ruby_prism::Visit::visit(&mut reads, &body);
    if reads.binding {
        return;
    }
    let unused = parameters
        .iter()
        .enumerate()
        .filter_map(|(index, parameter)| {
            (!(parameter.name.starts_with('_')
                || (allow_keywords && parameter.keyword)
                || reads.names.contains(parameter.name.as_bytes())))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    report_unused_block_parameters(context, &parameters, &unused, lambda, define_method);
}

struct BlockParameterInfo {
    name: String,
    range: std::ops::Range<usize>,
    keyword: bool,
    local: bool,
}

fn block_parameter_infos(parameters: &ruby_prism::ParametersNode<'_>) -> Vec<BlockParameterInfo> {
    let mut result = Vec::new();
    for parameter in parameters
        .requireds()
        .iter()
        .chain(parameters.posts().iter())
    {
        if let Some(parameter) = parameter.as_required_parameter_node() {
            result.push(BlockParameterInfo {
                name: String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                range: parameter.location().start_offset()..parameter.location().end_offset(),
                keyword: false,
                local: false,
            });
        } else if parameter.as_multi_target_node().is_some() {
            let mut targets = ParameterTargetCollector::default();
            ruby_prism::Visit::visit(&mut targets, &parameter);
            result.extend(targets.parameters);
        }
    }
    for parameter in parameters.optionals().iter() {
        if let Some(parameter) = parameter.as_optional_parameter_node() {
            let location = parameter.name_loc();
            result.push(BlockParameterInfo {
                name: String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                range: location.start_offset()..location.end_offset(),
                keyword: false,
                local: false,
            });
        }
    }
    for parameter in parameters.keywords().iter() {
        if let Some(parameter) = parameter.as_required_keyword_parameter_node() {
            let location = parameter.name_loc();
            result.push(BlockParameterInfo {
                name: String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                range: location.start_offset()..location.end_offset().saturating_sub(1),
                keyword: true,
                local: false,
            });
        } else if let Some(parameter) = parameter.as_optional_keyword_parameter_node() {
            let location = parameter.name_loc();
            result.push(BlockParameterInfo {
                name: String::from_utf8_lossy(parameter.name().as_slice()).into_owned(),
                range: location.start_offset()..location.end_offset().saturating_sub(1),
                keyword: true,
                local: false,
            });
        }
    }
    for parameter in [parameters.rest(), parameters.keyword_rest()] {
        let Some(parameter) = parameter else { continue };
        if let Some(parameter) = parameter.as_rest_parameter_node() {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                result.push(BlockParameterInfo {
                    name: String::from_utf8_lossy(name.as_slice()).into_owned(),
                    range: location.start_offset()..location.end_offset(),
                    keyword: false,
                    local: false,
                });
            }
        } else if let Some(parameter) = parameter.as_keyword_rest_parameter_node() {
            if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
                result.push(BlockParameterInfo {
                    name: String::from_utf8_lossy(name.as_slice()).into_owned(),
                    range: location.start_offset()..location.end_offset(),
                    keyword: true,
                    local: false,
                });
            }
        }
    }
    if let Some(parameter) = parameters.block() {
        if let (Some(name), Some(location)) = (parameter.name(), parameter.name_loc()) {
            result.push(BlockParameterInfo {
                name: String::from_utf8_lossy(name.as_slice()).into_owned(),
                range: location.start_offset()..location.end_offset(),
                keyword: false,
                local: false,
            });
        }
    }
    result
}

#[derive(Default)]
struct ParameterTargetCollector {
    parameters: Vec<BlockParameterInfo>,
}

impl<'pr> ruby_prism::Visit<'pr> for ParameterTargetCollector {
    fn visit_required_parameter_node(&mut self, node: &ruby_prism::RequiredParameterNode<'pr>) {
        self.parameters.push(BlockParameterInfo {
            name: String::from_utf8_lossy(node.name().as_slice()).into_owned(),
            range: node.location().start_offset()..node.location().end_offset(),
            keyword: false,
            local: false,
        });
    }

    fn visit_local_variable_target_node(
        &mut self,
        node: &ruby_prism::LocalVariableTargetNode<'pr>,
    ) {
        self.parameters.push(BlockParameterInfo {
            name: String::from_utf8_lossy(node.name().as_slice()).into_owned(),
            range: node.location().start_offset()..node.location().end_offset(),
            keyword: false,
            local: false,
        });
    }
}

#[derive(Default)]
struct BlockParameterReads {
    names: std::collections::HashSet<Vec<u8>>,
    nested_scopes: u32,
    binding: bool,
}

impl<'pr> ruby_prism::Visit<'pr> for BlockParameterReads {
    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        if node.depth() == self.nested_scopes {
            self.names.insert(node.name().as_slice().to_vec());
        }
    }

    fn visit_local_variable_write_node(&mut self, node: &ruby_prism::LocalVariableWriteNode<'pr>) {
        if node.depth() == self.nested_scopes {
            self.names.insert(node.name().as_slice().to_vec());
        }
        ruby_prism::visit_local_variable_write_node(self, node);
    }

    fn visit_local_variable_operator_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
    ) {
        if node.depth() == self.nested_scopes {
            self.names.insert(node.name().as_slice().to_vec());
        }
        ruby_prism::visit_local_variable_operator_write_node(self, node);
    }

    fn visit_local_variable_or_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
    ) {
        if node.depth() == self.nested_scopes {
            self.names.insert(node.name().as_slice().to_vec());
        }
        ruby_prism::visit_local_variable_or_write_node(self, node);
    }

    fn visit_local_variable_and_write_node(
        &mut self,
        node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
    ) {
        if node.depth() == self.nested_scopes {
            self.names.insert(node.name().as_slice().to_vec());
        }
        ruby_prism::visit_local_variable_and_write_node(self, node);
    }

    fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
        if node.name().as_slice() == b"binding" && node.arguments().is_none() {
            self.binding = true;
        }
        ruby_prism::visit_call_node(self, node);
    }

    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        self.nested_scopes += 1;
        ruby_prism::visit_block_node(self, node);
        self.nested_scopes -= 1;
    }

    fn visit_lambda_node(&mut self, node: &ruby_prism::LambdaNode<'pr>) {
        self.nested_scopes += 1;
        ruby_prism::visit_lambda_node(self, node);
        self.nested_scopes -= 1;
    }

    fn visit_def_node(&mut self, node: &ruby_prism::DefNode<'pr>) {
        if let Some(read) = node
            .receiver()
            .and_then(|receiver| receiver.as_local_variable_read_node())
        {
            self.names.insert(read.name().as_slice().to_vec());
        }
    }
    fn visit_class_node(&mut self, _node: &ruby_prism::ClassNode<'pr>) {}
    fn visit_module_node(&mut self, _node: &ruby_prism::ModuleNode<'pr>) {}
}

fn report_unused_block_parameters(
    context: &mut CopContext<'_, '_>,
    parameters: &[BlockParameterInfo],
    unused: &[usize],
    lambda: bool,
    define_method: bool,
) {
    for &index in unused {
        let parameter = &parameters[index];
        let all_unused = unused.len()
            == parameters
                .iter()
                .filter(|candidate| !candidate.name.starts_with('_'))
                .count();
        let message = unused_block_message(
            parameter,
            lambda,
            define_method,
            all_unused,
            parameters.len(),
        );
        if parameter.keyword {
            context.report(message, parameter.range.clone());
        } else {
            context.replace(
                message,
                parameter.range.clone(),
                parameter.range.clone(),
                format!("_{}", parameter.name),
            );
        }
    }
}

fn unused_block_message(
    parameter: &BlockParameterInfo,
    lambda: bool,
    define_method: bool,
    all_unused: bool,
    parameter_count: usize,
) -> String {
    if parameter.local {
        return format!("Unused block local variable - `{}`.", parameter.name);
    }
    let prefix = format!("Unused block argument - `{}`.", parameter.name);
    if lambda && all_unused {
        return format!("{prefix} If it's necessary, use `_` or `_{}` as an argument name to indicate that it won't be used. Also consider using a proc without arguments instead of a lambda if you want it to accept any arguments but don't care about them.", parameter.name);
    }
    if define_method || !all_unused {
        return format!("{prefix} If it's necessary, use `_` or `_{}` as an argument name to indicate that it won't be used.", parameter.name);
    }
    if parameter_count == 1 {
        format!("{prefix} You can omit the argument if you don't care about it.")
    } else {
        format!("{prefix} You can omit all the arguments if you don't care about them.")
    }
}

fn ambiguous_range(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let code = line.trim_start();
        if code.starts_with(['\'', '"', '`', '/', ':']) {
            continue;
        }
        let Some((at, operator)) = line
            .find("...")
            .map(|at| (at, "..."))
            .or_else(|| line.find("..").map(|at| (at, "..")))
        else {
            continue;
        };
        let operator_end = at + operator.len();
        let unmatched_open = line[..at].rfind('(').filter(|open| {
            line[*open + 1..at].matches('(').count() >= line[*open + 1..at].matches(')').count()
        });
        let left_start = unmatched_open.map_or(0, |open| open + 1);
        let right_end = unmatched_open
            .and_then(|_| {
                line[operator_end..]
                    .find(')')
                    .map(|close| operator_end + close)
            })
            .unwrap_or(line.len());
        let require_method_chains = context.config_bool("RequireParenthesesForMethodChains", false);
        let left = trimmed_boundary(line, left_start..at);
        let right = trimmed_boundary(line, operator_end..right_end);
        for boundary in [left, right].into_iter().flatten() {
            let value = &line[boundary.clone()];
            if ambiguous_range_boundary(value, require_method_chains) {
                let absolute = offset + boundary.start..offset + boundary.end;
                context.replace_many(
                    "Wrap complex range boundaries with parentheses to avoid ambiguity.",
                    absolute.clone(),
                    vec![
                        (absolute.start..absolute.start, "(".to_string()),
                        (absolute.end..absolute.end, ")".to_string()),
                    ],
                );
            }
        }
    }
}

fn trimmed_boundary(
    source: &str,
    mut range: std::ops::Range<usize>,
) -> Option<std::ops::Range<usize>> {
    while range.start < range.end && source.as_bytes()[range.start].is_ascii_whitespace() {
        range.start += 1;
    }
    while range.end > range.start && source.as_bytes()[range.end - 1].is_ascii_whitespace() {
        range.end -= 1;
    }
    (range.start < range.end).then_some(range)
}

fn ambiguous_range_boundary(boundary: &str, require_method_chains: bool) -> bool {
    if boundary.starts_with('(') && boundary.ends_with(')') {
        return false;
    }
    if has_top_level_range_operator(boundary) {
        return true;
    }
    if !boundary.contains('.') {
        return false;
    }
    let receiver = boundary.split('.').next().unwrap_or_default();
    require_method_chains || receiver.bytes().all(|byte| byte.is_ascii_digit())
}

fn has_top_level_range_operator(source: &str) -> bool {
    let mut depths = (0usize, 0usize, 0usize);
    let mut quote = None;
    let bytes = source.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        let byte = bytes[index];
        if let Some(delimiter) = quote {
            if byte == b'\\' {
                index += 2;
                continue;
            }
            if byte == delimiter {
                quote = None;
            }
            index += 1;
            continue;
        }
        match byte {
            b'\'' | b'"' | b'`' => quote = Some(byte),
            b'(' => depths.0 += 1,
            b')' => depths.0 = depths.0.saturating_sub(1),
            b'[' => depths.1 += 1,
            b']' => depths.1 = depths.1.saturating_sub(1),
            b'{' => depths.2 += 1,
            b'}' => depths.2 = depths.2.saturating_sub(1),
            _ if depths == (0, 0, 0) => {
                if [" || ", " && ", " + ", " - ", " * ", " % "]
                    .iter()
                    .any(|operator| bytes[index..].starts_with(operator.as_bytes()))
                {
                    return true;
                }
            }
            _ => {}
        }
        index += 1;
    }
    false
}

fn non_atomic_file_operation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (condition_offset, condition_line)) in lines.iter().copied().enumerate() {
        let condition_trimmed = condition_line.trim();
        let prefix = ["unless ", "if ", "elsif "].iter().find_map(|prefix| {
            condition_trimmed
                .strip_prefix(prefix)
                .map(|value| (*prefix, value))
        });
        if let Some((keyword, mut condition)) = prefix {
            let negated = condition.starts_with('!');
            condition = condition.trim_start_matches('!');
            let Some(check) = non_atomic_existence_check(condition) else {
                continue;
            };
            let Some((operation_offset, operation_line)) = lines.get(index + 1).copied() else {
                continue;
            };
            let Some(operation) = non_atomic_operation(operation_line.trim()) else {
                continue;
            };
            if check.argument != operation.argument
                || lines
                    .get(index + 2)
                    .is_none_or(|(_, line)| line.trim() != "end")
                || condition.contains("&&")
                || condition.contains("||")
                || operation.force_false
                || operation.kind == NonAtomicKind::Excluded
                || operation.kind.is_create() != (keyword == "unless " || negated)
            {
                continue;
            }
            report_non_atomic_operation(context, operation_offset, operation_line, &operation);
            let condition_start =
                condition_offset + condition_line.find(condition_trimmed).unwrap_or(0);
            let condition_range =
                condition_start..condition_offset + condition_line.trim_end().len();
            let message = format!("Remove unnecessary existence check `{}`.", check.label);
            if keyword == "elsif " {
                context.report(message, condition_range);
            } else {
                let (end_offset, end_line) = lines[index + 2];
                context.replace_many(
                    message,
                    condition_range,
                    vec![
                        (
                            condition_offset..condition_offset + condition_line.len(),
                            String::new(),
                        ),
                        (end_offset..end_offset + end_line.len(), String::new()),
                    ],
                );
            }
            continue;
        }

        let Some((modifier_at, keyword)) = condition_line
            .find(" unless ")
            .or_else(|| condition_line.find(" unless"))
            .map(|at| (at, "unless"))
            .or_else(|| {
                condition_line
                    .find(" if ")
                    .or_else(|| condition_line.find(" if"))
                    .map(|at| (at, "if"))
            })
        else {
            continue;
        };
        let operation_source = condition_line[..modifier_at].trim();
        let Some(operation) = non_atomic_operation(operation_source) else {
            continue;
        };
        let raw_condition = condition_line[modifier_at + 1..].trim_end();
        let raw_condition = raw_condition
            .strip_suffix('}')
            .map_or(raw_condition, str::trim_end);
        let mut condition_source = raw_condition.trim_start();
        let mut condition_end = condition_offset + modifier_at + 1 + raw_condition.len();
        if condition_source == keyword || condition_source == format!("{keyword} (") {
            let Some((next_offset, next_line)) = lines.get(index + 1).copied() else {
                continue;
            };
            condition_source = next_line.trim().trim_end_matches(')');
            condition_end = next_offset + next_line.trim_end().len();
        } else {
            condition_source = condition_source
                .strip_prefix(keyword)
                .unwrap_or(condition_source)
                .trim();
        }
        let Some(check) = non_atomic_existence_check(condition_source) else {
            continue;
        };
        if check.argument != operation.argument
            || operation.force_false
            || operation.kind == NonAtomicKind::Excluded
            || operation.kind.is_create() != (keyword == "unless")
        {
            continue;
        }
        let operation_start = condition_offset
            + condition_line.find(operation_source).unwrap_or(0)
            + operation_source.find(operation.source).unwrap_or(0);
        report_non_atomic_operation_at(context, operation_start, operation.source, &operation);
        let condition_start = condition_offset + modifier_at + 1;
        context.replace_many(
            format!("Remove unnecessary existence check `{}`.", check.label),
            condition_start..condition_end,
            vec![(condition_offset + modifier_at..condition_end, String::new())],
        );
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NonAtomicKind {
    Create,
    Remove,
    RecursiveRemove,
    AtomicCreate,
    AtomicRemove,
    Excluded,
}

impl NonAtomicKind {
    fn is_create(self) -> bool {
        matches!(self, Self::Create | Self::AtomicCreate)
    }
}

struct NonAtomicOperation<'a> {
    source: &'a str,
    receiver: &'a str,
    argument: &'a str,
    method: &'a str,
    replacement: Option<&'static str>,
    kind: NonAtomicKind,
    force_false: bool,
}

struct NonAtomicCheck<'a> {
    label: &'static str,
    argument: &'a str,
}

fn non_atomic_existence_check(source: &str) -> Option<NonAtomicCheck<'_>> {
    let source = source.trim();
    let source = source
        .strip_suffix('}')
        .map_or(source, str::trim_end)
        .trim_end_matches(')');
    for (receiver, method, label) in [
        ("FileTest", "exist?", "FileTest.exist?"),
        ("FileTest", "exists?", "FileTest.exists?"),
        ("File", "exist?", "File.exist?"),
        ("File", "exists?", "File.exists?"),
        ("Dir", "exist?", "Dir.exist?"),
        ("Dir", "exists?", "Dir.exists?"),
        ("Shell", "exist?", "Shell.exist?"),
        ("Shell", "exists?", "Shell.exists?"),
    ] {
        let receiver_method = format!("{receiver}.{method}");
        let Some(argument) = source
            .trim_start_matches("::")
            .strip_prefix(&receiver_method)
        else {
            continue;
        };
        let argument = argument.trim_start();
        let argument = if let Some(argument) = argument.strip_prefix('(') {
            argument.trim_end_matches(')').trim()
        } else if !argument.is_empty() {
            argument.trim()
        } else {
            continue;
        };
        if !argument.is_empty() {
            return Some(NonAtomicCheck { label, argument });
        }
    }
    None
}

fn non_atomic_operation(source: &str) -> Option<NonAtomicOperation<'_>> {
    let source = source.trim();
    let mut operation_start = ["FileUtils.", "File.", "Dir."]
        .iter()
        .filter_map(|receiver| source.rfind(receiver))
        .max()?;
    if source.get(operation_start.saturating_sub(2)..operation_start) == Some("::") {
        operation_start -= 2;
    }
    let source = &source[operation_start..];
    let (receiver_method, arguments) = if let Some(open) = source.find('(') {
        (
            &source[..open],
            source[open + 1..].trim_end_matches(')').trim(),
        )
    } else {
        source.split_once(' ')?
    };
    let receiver_method = receiver_method.trim_start_matches("::");
    let (receiver, method) = receiver_method.split_once('.')?;
    if !matches!(receiver, "FileUtils" | "File" | "Dir") {
        return None;
    }
    let argument = arguments.split(',').next()?.trim();
    let force_false = arguments.contains("force: false");
    let (kind, replacement) = match (receiver, method) {
        ("FileUtils", "mkdir") | ("Dir", "mkdir") => (NonAtomicKind::Create, Some("mkdir_p")),
        ("FileUtils", "makedirs" | "mkdir_p" | "mkpath") => (NonAtomicKind::AtomicCreate, None),
        (
            "FileUtils" | "File",
            "remove" | "delete" | "unlink" | "remove_file" | "rm" | "rmdir" | "safe_unlink",
        ) => (NonAtomicKind::Remove, Some("rm_f")),
        ("Dir", "rmdir") => (NonAtomicKind::Remove, Some("rm_f")),
        ("FileUtils", "remove_dir" | "remove_entry" | "remove_entry_secure") => {
            (NonAtomicKind::RecursiveRemove, Some("rm_rf"))
        }
        ("FileUtils", "rm_f" | "rm_rf") => (NonAtomicKind::AtomicRemove, None),
        ("FileUtils", "rm_r" | "rmtree") => (NonAtomicKind::Excluded, None),
        _ => return None,
    };
    Some(NonAtomicOperation {
        source,
        receiver,
        argument,
        method,
        replacement,
        kind,
        force_false,
    })
}

fn report_non_atomic_operation(
    context: &mut CopContext<'_, '_>,
    line_offset: usize,
    line: &str,
    operation: &NonAtomicOperation<'_>,
) {
    let start = line_offset + line.find(operation.source).unwrap_or(0);
    report_non_atomic_operation_at(context, start, operation.source, operation);
}

fn report_non_atomic_operation_at(
    context: &mut CopContext<'_, '_>,
    start: usize,
    source: &str,
    operation: &NonAtomicOperation<'_>,
) {
    let Some(atomic_method) = operation.replacement else {
        return;
    };
    let receiver_selector = format!("{}.{}", operation.receiver, operation.method);
    let (method_start, method_end, replacement) = if operation.receiver != "FileUtils" {
        let relative = source.find(&receiver_selector).unwrap_or(0);
        (
            start + relative,
            start + relative + receiver_selector.len(),
            format!("FileUtils.{atomic_method}"),
        )
    } else {
        let method_start = source
            .find(&format!(".{}", operation.method))
            .map_or(start, |relative| start + relative + 1);
        (
            method_start,
            method_start + operation.method.len(),
            atomic_method.to_string(),
        )
    };
    let mut edits = vec![(method_start..method_end, replacement)];
    if source.trim_start_matches("::").starts_with("Dir.mkdir") {
        if let Some(comma) = source.find(',') {
            let argument_start = comma
                + 1
                + source[comma + 1..]
                    .bytes()
                    .take_while(u8::is_ascii_whitespace)
                    .count();
            edits.push((
                start + comma + 1..start + argument_start,
                " mode: ".to_string(),
            ));
        }
    }
    let mut correction = CorrectionPlan::default();
    for (range, replacement) in edits {
        correction.replace(range, replacement);
    }
    context.apply_correction_indirectly(
        format!("Use atomic file operation method `FileUtils.{atomic_method}`."),
        start..start + source.len(),
        correction,
    );
}

fn unmodified_reduce_accumulator(
    node: &ruby_prism::BlockNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let Some(call) = context.parent().and_then(Node::as_call_node) else {
        return;
    };
    let method = call_name(&call);
    if !matches!(method, b"reduce" | b"inject") {
        return;
    }
    let Some(body) = node.body() else { return };
    let names = node
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
        .and_then(|parameters| parameters.parameters())
        .map(|parameters| {
            block_parameter_infos(&parameters)
                .into_iter()
                .map(|parameter| parameter.name)
                .collect::<Vec<_>>()
        })
        .unwrap_or_else(|| {
            let source = context.source_file().node(&body);
            if source.contains("_2") {
                vec!["_1".to_string(), "_2".to_string()]
            } else {
                Vec::new()
            }
        });
    if names.len() < 2 {
        return;
    }
    let accumulator = &names[0];
    let element = &names[1];
    let Some(statements) = body.as_statements_node() else {
        return;
    };
    let mut returns = statements.body().last().into_iter().collect::<Vec<_>>();
    let mut exits = ReduceExitValues::default();
    ruby_prism::Visit::visit(&mut exits, &body);
    returns.extend(exits.values);
    if returns.is_empty() {
        return;
    }

    if let Some(index) = returns.iter().find(|value| {
        let Some(access) = value.as_call_node() else {
            return false;
        };
        if !matches!(call_name(&access), b"[]" | b"[]=") {
            return false;
        }
        let receiver_is_accumulator = access
            .receiver()
            .and_then(|receiver| receiver.as_local_variable_read_node())
            .is_some_and(|receiver| receiver.name().as_slice() == accumulator.as_bytes());
        if !receiver_is_accumulator {
            return false;
        }
        call_name(&access) == b"[]="
            || !access.arguments().is_some_and(|arguments| {
                arguments
                    .arguments()
                    .iter()
                    .any(|argument| {
                        argument
                            .as_local_variable_read_node()
                            .is_some_and(|read| read.name().as_slice() == element.as_bytes())
                    })
            })
    }) {
        context.report(
            format!(
                "Do not return an element of the accumulator in `{}`.",
                String::from_utf8_lossy(method)
            ),
            index.location(),
        );
        return;
    }
    if returns
        .iter()
        .any(|value| reduce_plain_return_uses_name(value, accumulator.as_bytes()))
        || reduce_element_modified(&body, element.as_bytes(), accumulator.as_bytes())
    {
        return;
    }
    for value in returns {
        let variables = reduce_expression_names(&value);
        if variables.is_empty()
            || variables
                .iter()
                .any(|name| name.as_slice() != element.as_bytes())
        {
            continue;
        }
        context.report(
            format!(
                "Ensure the accumulator `{accumulator}` will be modified by `{}`.",
                String::from_utf8_lossy(method)
            ),
            value.location(),
        );
    }
}

#[derive(Default)]
struct ReduceExitValues<'pr> {
    values: Vec<Node<'pr>>,
    nested: usize,
}

impl<'pr> ruby_prism::Visit<'pr> for ReduceExitValues<'pr> {
    fn visit_next_node(&mut self, node: &ruby_prism::NextNode<'pr>) {
        if self.nested == 0 {
            if let Some(value) = node
                .arguments()
                .and_then(|arguments| arguments.arguments().first())
            {
                self.values.push(value);
            }
        }
    }
    fn visit_break_node(&mut self, node: &ruby_prism::BreakNode<'pr>) {
        if self.nested == 0 {
            if let Some(value) = node
                .arguments()
                .and_then(|arguments| arguments.arguments().first())
            {
                self.values.push(value);
            }
        }
    }
    fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
        self.nested += 1;
        ruby_prism::visit_block_node(self, node);
        self.nested -= 1;
    }
}

fn reduce_node_uses_name(node: &Node<'_>, name: &[u8]) -> bool {
    struct Reads<'a> {
        name: &'a [u8],
        found: bool,
    }
    impl<'pr> ruby_prism::Visit<'pr> for Reads<'_> {
        fn visit_local_variable_read_node(
            &mut self,
            node: &ruby_prism::LocalVariableReadNode<'pr>,
        ) {
            self.found |= node.name().as_slice() == self.name;
        }
        fn visit_local_variable_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableWriteNode<'pr>,
        ) {
            self.found |= node.name().as_slice() == self.name;
            ruby_prism::visit_local_variable_write_node(self, node);
        }
    }
    let mut reads = Reads { name, found: false };
    ruby_prism::Visit::visit(&mut reads, node);
    reads.found
}

fn reduce_expression_names(node: &Node<'_>) -> Vec<Vec<u8>> {
    struct Names(Vec<Vec<u8>>);
    impl<'pr> ruby_prism::Visit<'pr> for Names {
        fn visit_local_variable_read_node(
            &mut self,
            node: &ruby_prism::LocalVariableReadNode<'pr>,
        ) {
            self.0.push(node.name().as_slice().to_vec());
        }
        fn visit_local_variable_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableWriteNode<'pr>,
        ) {
            self.0.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_write_node(self, node);
        }
        fn visit_local_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
        ) {
            self.0.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_operator_write_node(self, node);
        }
        fn visit_local_variable_or_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
        ) {
            self.0.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_or_write_node(self, node);
        }
        fn visit_local_variable_and_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
        ) {
            self.0.push(node.name().as_slice().to_vec());
            ruby_prism::visit_local_variable_and_write_node(self, node);
        }
        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            if node.receiver().is_none()
                || node
                    .receiver()
                    .is_some_and(|receiver| receiver.as_self_node().is_some())
            {
                self.0.push(b"?".to_vec());
            }
            ruby_prism::visit_call_node(self, node);
        }
        fn visit_instance_variable_read_node(
            &mut self,
            _node: &ruby_prism::InstanceVariableReadNode<'pr>,
        ) {
            self.0.push(b"?".to_vec());
        }
        fn visit_class_variable_read_node(
            &mut self,
            _node: &ruby_prism::ClassVariableReadNode<'pr>,
        ) {
            self.0.push(b"?".to_vec());
        }
        fn visit_global_variable_read_node(
            &mut self,
            _node: &ruby_prism::GlobalVariableReadNode<'pr>,
        ) {
            self.0.push(b"?".to_vec());
        }
    }
    let mut names = Names(Vec::new());
    ruby_prism::Visit::visit(&mut names, node);
    names.0
}

fn reduce_plain_return_uses_name(node: &Node<'_>, name: &[u8]) -> bool {
    node.as_local_variable_read_node()
        .is_some_and(|read| read.name().as_slice() == name)
        || node.as_call_node().is_some_and(|call| {
            call.receiver()
                .and_then(|receiver| receiver.as_local_variable_read_node())
                .is_some_and(|receiver| receiver.name().as_slice() == name)
                && !matches!(call_name(&call), b"[]" | b"[]=")
        })
}

fn reduce_element_modified(body: &Node<'_>, element: &[u8], accumulator: &[u8]) -> bool {
    struct Modified<'a> {
        element: &'a [u8],
        accumulator: &'a [u8],
        found: bool,
        nested: usize,
    }
    impl<'pr> ruby_prism::Visit<'pr> for Modified<'_> {
        fn visit_local_variable_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableWriteNode<'pr>,
        ) {
            self.found |= node.name().as_slice() == self.element;
            ruby_prism::visit_local_variable_write_node(self, node);
        }
        fn visit_local_variable_operator_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOperatorWriteNode<'pr>,
        ) {
            self.found |= node.name().as_slice() == self.element;
            ruby_prism::visit_local_variable_operator_write_node(self, node);
        }
        fn visit_local_variable_or_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableOrWriteNode<'pr>,
        ) {
            self.found |= node.name().as_slice() == self.element;
            ruby_prism::visit_local_variable_or_write_node(self, node);
        }
        fn visit_local_variable_and_write_node(
            &mut self,
            node: &ruby_prism::LocalVariableAndWriteNode<'pr>,
        ) {
            self.found |= node.name().as_slice() == self.element;
            ruby_prism::visit_local_variable_and_write_node(self, node);
        }
        fn visit_call_node(&mut self, node: &ruby_prism::CallNode<'pr>) {
            let receiver_element = node
                .receiver()
                .and_then(|receiver| receiver.as_local_variable_read_node())
                .is_some_and(|receiver| receiver.name().as_slice() == self.element);
            let receiver_accumulator = node
                .receiver()
                .and_then(|receiver| receiver.as_local_variable_read_node())
                .is_some_and(|receiver| receiver.name().as_slice() == self.accumulator);
            let element_argument = node.arguments().is_some_and(|arguments| {
                arguments
                    .arguments()
                    .iter()
                    .any(|argument| reduce_node_uses_name(&argument, self.element))
            });
            let receiver_modification = receiver_element
                && node.arguments().is_some_and(|arguments| {
                    arguments.arguments().iter().any(|argument| {
                        argument.as_local_variable_read_node().is_some()
                            || argument.as_call_node().is_some()
                    })
                });
            self.found |= self.nested == 0
                && call_name(node) != b"[]"
                && call_name(node) != b"[]="
                && (receiver_modification || element_argument && !receiver_accumulator);
            ruby_prism::visit_call_node(self, node);
        }
        fn visit_block_node(&mut self, node: &ruby_prism::BlockNode<'pr>) {
            self.nested += 1;
            ruby_prism::visit_block_node(self, node);
            self.nested -= 1;
        }
    }
    let mut modified = Modified {
        element,
        accumulator,
        found: false,
        nested: 0,
    };
    ruby_prism::Visit::visit(&mut modified, body);
    modified.found
}

fn documentation_method(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let require_non_public = context.config_bool("RequireForNonPublicMethods", false);
    let allowed_methods = context.config_values("AllowedMethods").to_vec();
    let mut public = true;
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        let trimmed = line.trim_start();
        if matches!(trimmed.trim(), "private" | "protected") {
            public = false;
            continue;
        }
        if trimmed.trim() == "public" {
            public = true;
            continue;
        }
        let Some(def_at) = trimmed.find("def ") else {
            continue;
        };
        let prefix = trimmed[..def_at].trim();
        if !prefix.is_empty()
            && !matches!(
                prefix,
                "private"
                    | "protected"
                    | "private_class_method"
                    | "module_function"
                    | "ruby2_keywords"
            )
        {
            continue;
        }
        let effective_public =
            public && !matches!(prefix, "private" | "protected" | "private_class_method");
        if !effective_public && !require_non_public {
            continue;
        }
        let definition = &trimmed[def_at + "def ".len()..];
        let name = definition
            .split(|character: char| character.is_whitespace() || matches!(character, '(' | ';'))
            .next()
            .unwrap_or_default();
        let bare_name = name.rsplit('.').next().unwrap_or(name);
        if bare_name == "initialize"
            || bare_name.starts_with('_')
            || allowed_methods.iter().any(|allowed| allowed == bare_name)
        {
            continue;
        }
        let documented = index > 0 && documentation_comment(lines[index - 1].1);
        if !documented {
            let line_indent = line.len() - trimmed.len();
            let offense_indent = if matches!(prefix, "module_function" | "ruby2_keywords") {
                line_indent
            } else {
                line_indent + def_at
            };
            let end = if trimmed[def_at..].contains("; end") {
                offset + line.len()
            } else {
                lines[index + 1..]
                    .iter()
                    .find(|(_, candidate)| {
                        candidate.trim() == "end"
                            && candidate.len() - candidate.trim_start().len() <= line_indent
                    })
                    .map_or(offset + line.len(), |(end, candidate)| {
                        end + candidate.len()
                    })
            };
            context.report(
                "Missing method documentation comment.",
                offset + offense_indent..end,
            );
        }
    }
}

fn documentation_comment(line: &str) -> bool {
    let comment = line
        .trim_start()
        .strip_prefix('#')
        .map(str::trim)
        .unwrap_or_default();
    !comment.is_empty()
        && !["TODO", "FIXME", "OPTIMIZE", "HACK", "rubocop:"]
            .iter()
            .any(|marker| comment.starts_with(marker))
}

fn redundant_splat_expansion(node: &ruby_prism::SplatNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(expression) = node.expression() else {
        return;
    };
    let array = expression.as_array_node();
    let array_new = redundant_splat_array_new(&expression, context.source_file());
    let array_new_block = expression
        .as_call_node()
        .is_some_and(|call| call.block().is_some());
    let literal = array.is_some()
        || array_new
        || expression.as_string_node().is_some()
        || expression.as_interpolated_string_node().is_some()
        || expression.as_integer_node().is_some()
        || expression.as_float_node().is_some();
    if !literal {
        return;
    }

    let parent_array = context.parent().and_then(Node::as_array_node);
    let bracketed_array = parent_array
        .as_ref()
        .is_some_and(|array| array.opening_loc().is_some());
    let mut method_argument = false;
    for ancestor in context.ancestors().iter().rev() {
        if ancestor.as_call_node().is_some() {
            method_argument = true;
            break;
        }
        if ancestor.as_local_variable_write_node().is_some()
            || ancestor.as_instance_variable_write_node().is_some()
            || ancestor.as_class_variable_write_node().is_some()
            || ancestor.as_global_variable_write_node().is_some()
            || ancestor.as_constant_write_node().is_some()
            || ancestor.as_when_node().is_some()
            || ancestor.as_rescue_node().is_some()
            || ancestor.as_array_node().is_some()
        {
            break;
        }
    }
    if array_new_block && (bracketed_array || method_argument) {
        return;
    }
    if array_new && bracketed_array {
        let parent = parent_array.unwrap();
        if parent.elements().len() > 1 {
            return;
        }
        let offense = node.location();
        context.replace(
            "Replace splat expansion with comma separated values.",
            offense.start_offset()..offense.end_offset(),
            parent.location().start_offset()..parent.location().end_offset(),
            context.source_file().node(&expression),
        );
        return;
    }
    if array_new {
        // Array constructors are only redundant in an assignment, argument,
        // or one-element array; not in `when`/`rescue` expansion lists.
        let assignment = context.ancestors().iter().any(|ancestor| {
            ancestor.as_local_variable_write_node().is_some()
                || ancestor.as_instance_variable_write_node().is_some()
                || ancestor.as_class_variable_write_node().is_some()
                || ancestor.as_global_variable_write_node().is_some()
                || ancestor.as_constant_write_node().is_some()
        });
        if !assignment && !method_argument {
            return;
        }
    }
    if let Some(array) = &array {
        let opening = array
            .opening_loc()
            .map(|opening| String::from_utf8_lossy(opening.as_slice()).into_owned())
            .unwrap_or_default();
        if method_argument
            && opening.starts_with('%')
            && context.config_bool("AllowPercentLiteralArrayArgument", true)
        {
            return;
        }
    }

    let offense = node.location();
    let offense_range = offense.start_offset()..offense.end_offset();
    let (edit_range, replacement) = if let Some(array) = &array {
        if method_argument
            || bracketed_array
            || context.ancestors().iter().any(|ancestor| {
                ancestor.as_when_node().is_some() || ancestor.as_rescue_node().is_some()
            })
        {
            (
                offense_range.clone(),
                redundant_splat_array_contents(array, context.source_file()),
            )
        } else {
            (
                node.operator_loc().start_offset()..node.operator_loc().end_offset(),
                String::new(),
            )
        }
    } else if array_new {
        (
            node.operator_loc().start_offset()..node.operator_loc().end_offset(),
            String::new(),
        )
    } else if method_argument || bracketed_array {
        (
            offense_range.clone(),
            context.source_file().node(&expression).to_string(),
        )
    } else {
        (
            offense_range.clone(),
            format!("[{}]", context.source_file().node(&expression)),
        )
    };
    let message = if array.is_some() && (method_argument || bracketed_array) {
        "Pass array contents as separate arguments."
    } else {
        "Replace splat expansion with comma separated values."
    };
    context.replace(message, offense_range, edit_range, replacement);
}

fn redundant_splat_array_new(node: &Node<'_>, file: SourceFile<'_>) -> bool {
    let Some(call) = node.as_call_node() else {
        return false;
    };
    call_name(&call) == b"new"
        && call.receiver().is_some_and(|receiver| {
            matches!(file.node(&receiver).trim_start_matches("::"), "Array")
        })
}

fn redundant_splat_array_contents(
    array: &ruby_prism::ArrayNode<'_>,
    file: SourceFile<'_>,
) -> String {
    let opening = array
        .opening_loc()
        .map(|opening| String::from_utf8_lossy(opening.as_slice()).into_owned())
        .unwrap_or_default();
    array
        .elements()
        .iter()
        .map(|element| {
            let source = file.node(&element);
            if opening.starts_with("%w") {
                format!("'{source}'")
            } else if opening.starts_with("%W") {
                format!("\"{source}\"")
            } else if opening.starts_with("%i") {
                format!(":{source}")
            } else if opening.starts_with("%I") {
                format!(":\"{source}\"")
            } else {
                source.to_string()
            }
        })
        .collect::<Vec<_>>()
        .join(", ")
}
