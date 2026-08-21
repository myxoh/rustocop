use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        Box::new(FileTouch),
        Box::new(GlobalStdStream),
        Box::new(MinMax),
        Box::new(RedundantFileExtensionInRequire),
        Box::new(SuperWithArgsParentheses),
        Box::new(TrailingCommaInBlockArgs),
    ]
}

struct MinMax;

impl Cop for MinMax {
    fn name(&self) -> &'static str {
        "Style/MinMax"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let (elements, offense_start, offense_end) = if let Some(array) = node.as_array_node() {
            let elements = array.elements().iter().collect::<Vec<_>>();
            let location = array.location();
            (elements, location.start_offset(), location.end_offset())
        } else if let Some(return_node) = node.as_return_node() {
            let Some(arguments) = return_node.arguments() else {
                return;
            };
            let elements = arguments.arguments().iter().collect::<Vec<_>>();
            let (Some(first), Some(last)) = (elements.first(), elements.last()) else {
                return;
            };
            let start = first.location().start_offset();
            let end = last.location().end_offset();
            (elements, start, end)
        } else {
            return;
        };
        if elements.len() != 2 {
            return;
        }
        let (Some(minimum), Some(maximum)) =
            (elements[0].as_call_node(), elements[1].as_call_node())
        else {
            return;
        };
        let (Some(min_receiver), Some(max_receiver)) = (minimum.receiver(), maximum.receiver())
        else {
            return;
        };
        if call_name(&minimum) != b"min"
            || call_name(&maximum) != b"max"
            || source_at(source, &min_receiver.location())
                != source_at(source, &max_receiver.location())
        {
            return;
        }
        let receiver = source_at(source, &min_receiver.location());
        let offender = &source[offense_start..offense_end];
        context.replace(
            self.name(),
            format!("Use `{receiver}.minmax` instead of `{offender}`."),
            (offense_start, offense_end),
            (offense_start, offense_end),
            format!("{receiver}.minmax"),
        );
    }
}

struct FileTouch;

impl Cop for FileTouch {
    fn name(&self) -> &'static str {
        "Style/FileTouch"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(call) = node.as_call_node() else {
            return;
        };
        let Some(arguments) = call.arguments() else {
            return;
        };
        let values = arguments.arguments().iter().collect::<Vec<_>>();
        let (Some(filename), Some(mode)) = (values.first(), values.last()) else {
            return;
        };
        let empty_block = call
            .block()
            .and_then(|block| block.as_block_node())
            .is_some_and(|block| block.body().is_none());
        let append_mode = mode.as_string_node().is_some_and(|mode| {
            matches!(
                mode.unescaped(),
                b"a" | b"a+" | b"ab" | b"a+b" | b"at" | b"a+t"
            )
        });
        if call_name(&call) != b"open"
            || !root_constant(call.receiver(), b"File")
            || values.len() < 2
            || !empty_block
            || !append_mode
        {
            return;
        }

        let argument = source_at(source, &filename.location());
        let location = call.location();
        context.replace(
            self.name(),
            format!(
                "Use `FileUtils.touch({argument})` instead of `File.open` in append mode with empty block."
            ),
            &location,
            &location,
            format!("FileUtils.touch({argument})"),
        );
    }
}

struct GlobalStdStream;

impl Cop for GlobalStdStream {
    fn name(&self) -> &'static str {
        "Style/GlobalStdStream"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let (constant, location) = if let Some(read) = node.as_constant_read_node() {
            (read.name().as_slice(), read.location())
        } else if let Some(path) = node.as_constant_path_node() {
            if path.parent().is_some() {
                return;
            }
            let Some(name) = path.name() else {
                return;
            };
            (name.as_slice(), path.location())
        } else {
            return;
        };
        let global = match constant {
            b"STDIN" => b"$stdin".as_slice(),
            b"STDOUT" => b"$stdout".as_slice(),
            b"STDERR" => b"$stderr".as_slice(),
            _ => return,
        };
        if ancestors.iter().rev().any(|ancestor| {
            ancestor
                .as_global_variable_write_node()
                .is_some_and(|write| write.name().as_slice() == global)
        }) {
            return;
        }

        let constant = String::from_utf8_lossy(constant);
        let global = String::from_utf8_lossy(global);
        context.replace(
            self.name(),
            format!("Use `{global}` instead of `{constant}`."),
            &location,
            &location,
            global.into_owned(),
        );
    }
}

struct RedundantFileExtensionInRequire;

impl Cop for RedundantFileExtensionInRequire {
    fn name(&self) -> &'static str {
        "Style/RedundantFileExtensionInRequire"
    }

    fn on_call(&self, node: &CallNode<'_>, context: &mut Context) {
        if !match_call(node)
            .named_any(&[b"require", b"require_relative"])
            .without_receiver()
            .with_argument_count(1)
            .matches()
        {
            return;
        }
        let Some(string) = only_argument(node).and_then(|argument| argument.as_string_node())
        else {
            return;
        };
        if !string.unescaped().ends_with(b".rb") {
            return;
        }
        let end = string.location().end_offset().saturating_sub(1);
        let start = end.saturating_sub(3);
        context.remove(
            self.name(),
            "Redundant `.rb` file extension detected.",
            (start, end),
            (start, end),
        );
    }
}

struct SuperWithArgsParentheses;

impl Cop for SuperWithArgsParentheses {
    fn name(&self) -> &'static str {
        "Style/SuperWithArgsParentheses"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        _ancestors: &[Node<'pr>],
        _source: &str,
        context: &mut Context,
    ) {
        let Some(super_node) = node.as_super_node() else {
            return;
        };
        let Some(arguments) = super_node.arguments() else {
            return;
        };
        if super_node.lparen_loc().is_some() || arguments.arguments().is_empty() {
            return;
        }
        let argument_nodes = arguments.arguments();
        let Some(first) = argument_nodes.first() else {
            return;
        };
        let Some(last) = argument_nodes.last() else {
            return;
        };
        let last_end = super_node
            .block()
            .and_then(|block| block.as_block_argument_node())
            .map_or(last.location().end_offset(), |block| {
                block.location().end_offset()
            });
        let keyword = super_node.keyword_loc();
        let offense = keyword.start_offset()..last_end;
        context.replace_many(
            self.name(),
            "Use parentheses for `super` with arguments.",
            offense,
            vec![
                (
                    keyword.end_offset()..first.location().start_offset(),
                    "(".to_string(),
                ),
                (
                    last_end..last_end,
                    ")".to_string(),
                ),
            ],
        );
    }
}

struct TrailingCommaInBlockArgs;

impl Cop for TrailingCommaInBlockArgs {
    fn name(&self) -> &'static str {
        "Style/TrailingCommaInBlockArgs"
    }

    fn on_node<'pr>(
        &self,
        node: &Node<'pr>,
        ancestors: &[Node<'pr>],
        source: &str,
        context: &mut Context,
    ) {
        let Some(block) = node.as_block_node() else {
            return;
        };
        let Some(parameters) = block.parameters() else {
            return;
        };
        let Some(block_parameters) = parameters.as_block_parameters_node() else {
            return;
        };
        let Some(parameter_nodes) = block_parameters.parameters() else {
            return;
        };
        let argument_count = block_argument_count(&parameter_nodes);
        let text = source_at(source, &parameters.location());
        let Some(first_pipe) = text.find('|') else {
            return;
        };
        let Some(last_pipe) = text.rfind('|').filter(|last| *last > first_pipe) else {
            return;
        };
        let arguments = &text[first_pipe + 1..last_pipe];
        if argument_count > 1 && !arguments.contains(';') && arguments.trim_end().ends_with(',') {
            let comma = parameters.location().start_offset()
                + first_pipe
                + 1
                + arguments.trim_end().len()
                - 1;
            context.remove(
                self.name(),
                "Useless trailing comma present in block arguments.",
                (comma, comma + 1),
                (comma, comma + 1),
            );
            return;
        }
        if argument_count <= 1 {
            return;
        }
        let Some(inner_parameters) = ancestors.iter().rev().find_map(|ancestor| {
            let call = ancestor.as_call_node()?;
            let receiver = call.receiver()?.as_call_node()?;
            receiver
                .block()?
                .as_block_node()?
                .parameters()?
                .as_block_parameters_node()
        }) else {
            return;
        };
        let inner_text = source_at(source, &inner_parameters.location());
        let Some(comma) = inner_text.rfind(',') else {
            return;
        };
        if inner_text[comma + 1..].trim() != "|" {
            return;
        }
        let comma = inner_parameters.location().start_offset() + comma;
        context.remove(
            self.name(),
            "Useless trailing comma present in block arguments.",
            (comma, comma + 1),
            (comma, comma + 1),
        );
    }
}

fn block_argument_count(parameters: &ruby_prism::ParametersNode<'_>) -> usize {
    parameters
        .requireds()
        .iter()
        .chain(parameters.posts().iter())
        .map(|parameter| required_block_argument_count(&parameter))
        .sum::<usize>()
        + parameters.optionals().len()
        + parameters
            .keywords()
            .iter()
            .filter(|parameter| parameter.as_optional_keyword_parameter_node().is_some())
            .count()
}

fn required_block_argument_count(parameter: &Node<'_>) -> usize {
    if parameter.as_required_parameter_node().is_some() {
        return 1;
    }
    let Some(group) = parameter.as_multi_target_node() else {
        return 0;
    };
    group
        .lefts()
        .iter()
        .chain(group.rights().iter())
        .map(|target| required_block_argument_count(&target))
        .sum()
}
