use super::*;

define_cops! {
    DigChain => "Style/DigChain" => call(dig_chain),
}

fn dig_chain(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if call_name(node) != b"dig" || is_receiver_of_dig(node, context.parent()) {
        return;
    }

    let mut current = node.as_node();
    let mut groups = Vec::<Vec<String>>::new();
    let mut first_selector = None;
    while let Some(call) = current.as_call_node() {
        if call_name(&call) != b"dig" {
            break;
        }
        let Some(arguments) = call.arguments() else {
            return;
        };
        if arguments.arguments().is_empty() {
            return;
        }
        if arguments.arguments().iter().any(|argument| {
            argument.as_hash_node().is_some() || argument.as_keyword_hash_node().is_some()
        }) {
            return;
        }
        let rendered = arguments
            .arguments()
            .iter()
            .map(|argument| context.source_file().node(&argument).to_string())
            .collect::<Vec<_>>();
        if rendered.iter().any(|argument| {
            argument.starts_with('&')
                || argument.starts_with("**")
                || argument.starts_with('{')
                || argument.contains("=>")
        }) {
            return;
        }
        first_selector = call.message_loc().map(|selector| selector.start_offset());
        groups.push(rendered);
        let Some(receiver) = call.receiver() else {
            break;
        };
        current = receiver;
    }
    if groups.len() < 2 {
        return;
    }
    groups.reverse();
    if groups[..groups.len() - 1]
        .iter()
        .flatten()
        .any(|argument| argument == "...")
    {
        return;
    }

    let arguments = groups.into_iter().flatten().collect::<Vec<_>>().join(", ");
    let replacement = format!("dig({arguments})");
    let message = format!("Use `{replacement}` instead of chaining.");
    let offense_start = first_selector.expect("dig calls have selectors");
    let offense = offense_start..node.location().end_offset();
    let offense_source = &context.source()[offense.clone()];
    if offense_source.contains('#') {
        let line_start = context.source()[..offense_start]
            .rfind('\n')
            .map_or(0, |index| index + 1);
        let indentation = &context.source()[line_start..][..context.source()[line_start..]
            .bytes()
            .take_while(u8::is_ascii_whitespace)
            .count()];
        let comments = offense_source
            .lines()
            .filter_map(|line| line.find('#').map(|index| line[index..].trim_end()))
            .map(|comment| format!("{indentation}{comment}\n"))
            .collect::<String>();
        let prefix = &context.source()[line_start..offense_start];
        context.replace(
            message,
            offense,
            line_start..node.location().end_offset(),
            format!("{comments}{prefix}{replacement}"),
        );
    } else {
        context.replace(message, offense.clone(), offense, replacement);
    }
}

fn is_receiver_of_dig(node: &CallNode<'_>, parent: Option<&Node<'_>>) -> bool {
    parent.and_then(Node::as_call_node).is_some_and(|parent| {
        call_name(&parent) == b"dig"
            && parent.receiver().is_some_and(|receiver| {
                receiver.location().start_offset() == node.location().start_offset()
                    && receiver.location().end_offset() == node.location().end_offset()
            })
    })
}
