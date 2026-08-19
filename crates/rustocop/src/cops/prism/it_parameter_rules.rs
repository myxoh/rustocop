use super::*;

define_cops! {
    ItBlockParameter => "Style/ItBlockParameter" => node(as_block_node, it_block_parameter),
}

fn it_block_parameter(node: &ruby_prism::BlockNode<'_>, context: &mut CopContext<'_, '_>) {
    if !context.target_ruby_version().at_least(3, 4) {
        return;
    }
    let Some(parameters) = node.parameters() else {
        return;
    };
    let style = context
        .policy()
        .enforced_style("allow_single_line")
        .to_string();
    let mut reads = ParameterReads::default();
    if let Some(body) = node.body() {
        reads.visit(&body);
    }

    if parameters.as_it_parameters_node().is_some() {
        check_it_parameter(node, &style, &reads.it, context);
    } else if parameters
        .as_numbered_parameters_node()
        .is_some_and(|parameters| parameters.maximum() == 1)
    {
        if style != "disallow" {
            correct_parameter_reads(node, &reads.numbered, None, context);
        }
    } else if style == "always" {
        let Some(parameters) = parameters.as_block_parameters_node() else {
            return;
        };
        let Some(name) = single_named_parameter(&parameters) else {
            return;
        };
        let matching = reads
            .named
            .iter()
            .filter(|(read_name, _)| read_name == &name)
            .map(|(_, range)| range.clone())
            .collect::<Vec<_>>();
        if !matching.is_empty() {
            correct_parameter_reads(node, &matching, Some(parameters.location()), context);
        }
    }
}

fn check_it_parameter(
    block: &ruby_prism::BlockNode<'_>,
    style: &str,
    reads: &[std::ops::Range<usize>],
    context: &mut CopContext<'_, '_>,
) {
    if style == "disallow" {
        for read in reads {
            context.report("Avoid using `it` block parameter.", read.clone());
        }
    } else if style == "allow_single_line"
        && context.source_file().node(&block.as_node()).contains('\n')
    {
        let offense = context.parent().map_or_else(
            || block.location().start_offset()..block.location().end_offset(),
            |parent| parent.location().start_offset()..parent.location().end_offset(),
        );
        context.report(
            "Avoid using `it` block parameter for multi-line blocks.",
            offense,
        );
    }
}

fn correct_parameter_reads(
    block: &ruby_prism::BlockNode<'_>,
    reads: &[std::ops::Range<usize>],
    parameters: Option<ruby_prism::Location<'_>>,
    context: &mut CopContext<'_, '_>,
) {
    if reads.is_empty() {
        return;
    }
    let location = block.location();
    let block_range = location.start_offset()..location.end_offset();
    let file = context.source_file();
    let mut edits = reads
        .iter()
        .cloned()
        .map(|range| SourceEdit::replace(range, "it"))
        .collect::<Vec<_>>();
    if let Some(parameters) = parameters {
        edits.push(SourceEdit::remove(
            parameters.start_offset()..parameters.end_offset(),
        ));
    }
    let Some(replacement) = file.rewrite(block_range.clone(), edits) else {
        return;
    };
    for read in reads {
        context.replace(
            "Use `it` block parameter.",
            read.clone(),
            block_range.clone(),
            replacement.clone(),
        );
    }
}

fn single_named_parameter(parameters: &ruby_prism::BlockParametersNode<'_>) -> Option<Vec<u8>> {
    let parameters = parameters.parameters()?;
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    parameters
        .requireds()
        .first()?
        .as_required_parameter_node()
        .map(|parameter| parameter.name().as_slice().to_vec())
}

#[derive(Default)]
struct ParameterReads {
    it: Vec<std::ops::Range<usize>>,
    numbered: Vec<std::ops::Range<usize>>,
    named: Vec<(Vec<u8>, std::ops::Range<usize>)>,
}

impl<'pr> Visit<'pr> for ParameterReads {
    fn visit_it_local_variable_read_node(
        &mut self,
        node: &ruby_prism::ItLocalVariableReadNode<'pr>,
    ) {
        self.it
            .push(node.location().start_offset()..node.location().end_offset());
    }

    fn visit_local_variable_read_node(&mut self, node: &ruby_prism::LocalVariableReadNode<'pr>) {
        let range = node.location().start_offset()..node.location().end_offset();
        if node.depth() == 0 && node.name().as_slice() == b"_1" {
            self.numbered.push(range.clone());
        }
        self.named.push((node.name().as_slice().to_vec(), range));
    }

    fn visit_block_node(&mut self, _node: &ruby_prism::BlockNode<'pr>) {}
}
