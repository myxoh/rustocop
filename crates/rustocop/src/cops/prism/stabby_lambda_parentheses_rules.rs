use super::*;
pub(super) fn cops() -> Vec<Box<dyn Cop>> { Vec::new() }


fn stabby_lambda_parentheses(node: &ruby_prism::LambdaNode<'_>, context: &mut CopContext<'_, '_>) {
    let Some(arguments) = node
        .parameters()
        .and_then(|parameters| parameters.as_block_parameters_node())
    else {
        return;
    };
    if arguments.parameters().is_none() && arguments.locals().iter().next().is_none() {
        return;
    }
    let parenthesized = arguments.opening_loc().is_some();
    let no_parentheses =
        context.policy().enforced_style("require_parentheses") == "require_no_parentheses";
    if no_parentheses && parenthesized {
        let (Some(opening), Some(closing)) = (arguments.opening_loc(), arguments.closing_loc())
        else {
            return;
        };
        let source = context.source().as_bytes();
        let nested_destructuring_delimiters = arguments.parameters().map_or(0, |parameters| {
            let mut offset = parameters.location().start_offset();
            while source.get(offset) == Some(&b'(') {
                offset += 1;
            }
            offset - parameters.location().start_offset()
        });
        let opening_end = opening.end_offset() + nested_destructuring_delimiters;
        let closing_start = closing
            .start_offset()
            .saturating_sub(nested_destructuring_delimiters);
        context.replace_many(
            "Do not wrap stabby lambda arguments with parentheses.",
            arguments.location(),
            vec![
                (opening.start_offset()..opening_end, String::new()),
                (closing_start..closing.end_offset(), String::new()),
            ],
        );
    } else if !no_parentheses && !parenthesized {
        let location = arguments.location();
        context.replace(
            "Wrap stabby lambda arguments with parentheses.",
            &location,
            &location,
            format!("({})", context.source_file().at(&location)),
        );
    }
}
