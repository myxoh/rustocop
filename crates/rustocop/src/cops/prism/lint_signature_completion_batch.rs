use super::*;

define_cops! {
    Syntax => "Lint/Syntax" => parse_error(syntax),
    FormatParameterMismatch => "Lint/FormatParameterMismatch" => source(format_parameter_mismatch),
    UnusedBlockArgument => "Lint/UnusedBlockArgument" => any_node(unused_block_argument),
    AmbiguousRange => "Lint/AmbiguousRange" => source(ambiguous_range),
    NonAtomicFileOperation => "Lint/NonAtomicFileOperation" => source(non_atomic_file_operation),
    UnmodifiedReduceAccumulator => "Lint/UnmodifiedReduceAccumulator" => source(unmodified_reduce_accumulator),
    DocumentationMethod => "Style/DocumentationMethod" => source(documentation_method),
    RedundantSplatExpansion => "Lint/RedundantSplatExpansion" => source(redundant_splat_expansion),
}

fn syntax(error: &Diagnostic<'_>, context: &mut CopContext<'_, '_>) {
    let location = error.location();
    let start = location.start_offset().min(context.source().len());
    let end = location.end_offset().max(start).min(context.source().len());
    context.report(error.message(), start..end);
}

fn format_parameter_mismatch(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let unescaped = line.replace("%%", "");
        let placeholders = unescaped.matches("%s").count()
            + unescaped.matches("%d").count()
            + unescaped.matches("%f").count();
        if placeholders == 0 {
            continue;
        }
        let Some(percent) = line.find(" % ") else {
            continue;
        };
        let arguments = line[percent + 3..].trim();
        let supplied = if arguments.starts_with('[') {
            arguments.split(',').count()
        } else {
            1
        };
        if placeholders != supplied {
            context.report(format!("Number of arguments ({supplied}) to format string differs from number of fields ({placeholders})."), offset + percent..offset + line.len());
        }
    }
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
            local.as_block_local_variable_node().map(|local| BlockParameterInfo {
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
    report_unused_block_parameters(
        context,
        &parameters,
        &unused,
        lambda,
        define_method,
    );
}

struct BlockParameterInfo {
    name: String,
    range: std::ops::Range<usize>,
    keyword: bool,
    local: bool,
}

fn block_parameter_infos(parameters: &ruby_prism::ParametersNode<'_>) -> Vec<BlockParameterInfo> {
    let mut result = Vec::new();
    for parameter in parameters.requireds().iter().chain(parameters.posts().iter()) {
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
    fn visit_required_parameter_node(
        &mut self,
        node: &ruby_prism::RequiredParameterNode<'pr>,
    ) {
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
        if let Some(read) = node.receiver().and_then(|receiver| receiver.as_local_variable_read_node()) {
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
    if [" || ", " && ", " + ", " - ", " * ", " % "]
        .iter()
        .any(|operator| boundary.contains(operator))
    {
        return true;
    }
    if !boundary.contains('.') {
        return false;
    }
    let receiver = boundary.split('.').next().unwrap_or_default();
    require_method_chains || receiver.bytes().all(|byte| byte.is_ascii_digit())
}

fn non_atomic_file_operation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if (window[0].1.contains("File.exist?") || window[0].1.contains("File.exists?"))
            && ["File.delete", "File.rename", "FileUtils.rm", "FileUtils.mv"]
                .iter()
                .any(|operation| window[1].1.contains(operation))
        {
            context.report(
                "File operation is not atomic.",
                window[0].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn unmodified_reduce_accumulator(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        let Some(method) = line
            .find(".reduce")
            .or_else(|| line.find(".inject"))
        else {
            continue;
        };
        let pipes = line[method..]
            .match_indices('|')
            .map(|(at, _)| method + at)
            .collect::<Vec<_>>();
        let candidates = pipes
            .windows(2)
            .filter_map(|pair| {
                let prefix = line[..pair[0]].trim_end();
                let brace_block = prefix.ends_with('{');
                let do_block = prefix
                    .strip_suffix("do")
                    .is_some_and(|before| before.is_empty() || before.ends_with(char::is_whitespace));
                let parameters = &line[pair[0] + 1..pair[1]];
                ((brace_block || do_block)
                    && parameters.bytes().all(|byte| {
                        byte.is_ascii_alphanumeric()
                            || matches!(byte, b'_' | b',' | b' ' | b'*' | b';')
                    }))
                .then_some((pair[0], pair[1], do_block))
            })
            .collect::<Vec<_>>();
        let selected = candidates
            .iter()
            .rev()
            .find(|(_, _, do_block)| *do_block)
            .or_else(|| candidates.first());
        let Some(&(pipe, close, _)) = selected else {
            continue;
        };
        let mut parameters = line[pipe + 1..close].split(',').map(str::trim);
        let accumulator = parameters.next().unwrap_or("");
        let element = parameters.next().unwrap_or("");
        let body = &line[close + 1..];
        if accumulator.is_empty() {
            continue;
        }
        let expression_start = body.rfind(';').map_or(0, |at| at + 1);
        let expression = &body[expression_start..];
        let leading = expression.len() - expression.trim_start().len();
        let returned = expression.trim().trim_end_matches('}').trim_end();
        if let Some(relative_index) = returned
            .strip_prefix(accumulator)
            .filter(|suffix| suffix.starts_with('['))
            .map(|_| expression_start + leading)
        {
            let Some(end) = body[relative_index..]
                .find(']')
                .map(|at| relative_index + at + 1)
            else {
                continue;
            };
            let index_argument = &body[relative_index + accumulator.len() + 1..end - 1];
            let tail = returned[end - relative_index..].trim();
            let assignment = tail.starts_with('=') && !tail.starts_with("==");
            if !tail.is_empty() && !assignment {
                continue;
            }
            if !assignment && index_argument.trim() == element {
                continue;
            }
            let method_name = if line[method..].starts_with(".inject") {
                "inject"
            } else {
                "reduce"
            };
            context.report(
                format!("Do not return an element of the accumulator in `{method_name}`."),
                offset + close + 1 + relative_index..offset + close + 1 + end,
            );
        } else if body.contains('}')
            && !body.trim_start().starts_with('}')
            && !body.contains(accumulator)
        {
            let start = offset + pipe + 1;
            context.report(
                "Ensure the reduce accumulator is modified in each iteration.",
                start..start + accumulator.len(),
            );
        }
    }
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
        let Some(def_at) = trimmed.find("def ") else { continue };
        let prefix = trimmed[..def_at].trim();
        if !prefix.is_empty() && !matches!(prefix, "private" | "protected" | "private_class_method" | "module_function" | "ruby2_keywords") { continue; }
        let effective_public = public && !matches!(prefix, "private" | "protected" | "private_class_method");
        if !effective_public && !require_non_public { continue; }
        let definition = &trimmed[def_at + "def ".len()..];
        let name = definition.split(|character: char| character.is_whitespace() || matches!(character, '(' | ';')).next().unwrap_or_default();
        let bare_name = name.rsplit('.').next().unwrap_or(name);
        if bare_name == "initialize" || bare_name.starts_with('_') || allowed_methods.iter().any(|allowed| allowed == bare_name) { continue; }
        let documented = index > 0 && documentation_comment(lines[index - 1].1);
        if !documented {
            let line_indent = line.len() - trimmed.len();
            let offense_indent = if matches!(prefix, "module_function" | "ruby2_keywords") { line_indent } else { line_indent + def_at };
            let end = if trimmed[def_at..].contains("; end") {
                offset + line.len()
            } else {
                lines[index + 1..]
                    .iter()
                    .find(|(_, candidate)| candidate.trim() == "end" && candidate.len() - candidate.trim_start().len() <= line_indent)
                    .map_or(offset + line.len(), |(end, candidate)| end + candidate.len())
            };
            context.report(
                "Missing method documentation comment.",
                offset + offense_indent..end,
            );
        }
    }
}

fn documentation_comment(line: &str) -> bool {
    let comment = line.trim_start().strip_prefix('#').map(str::trim).unwrap_or_default();
    !comment.is_empty()
        && !["TODO", "FIXME", "OPTIMIZE", "HACK", "rubocop:"].iter().any(|marker| comment.starts_with(marker))
}

fn redundant_splat_expansion(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    let mut search = 0;
    while let Some(relative) = source[search..].find("[*") {
        let start = search + relative;
        let Some(close) = source[start + 2..].find(']').map(|at| start + 2 + at) else {
            break;
        };
        let value = source[start + 2..close].trim();
        if value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'_')
        {
            context.replace(
                "Redundant splat expansion.",
                start..close + 1,
                start..close + 1,
                value,
            );
        }
        search = close + 1;
    }
}
