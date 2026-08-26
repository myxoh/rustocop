use super::*;

define_cops! {
    EnvHome => "Style/EnvHome" => call(env_home),
    AsciiComments => "Style/AsciiComments" => source(ascii_comments),
}

fn env_home(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    if !root_constant(node.receiver(), b"ENV") {
        return;
    }
    let arguments = node
        .arguments()
        .map(|arguments| arguments.arguments().iter().collect::<Vec<_>>())
        .unwrap_or_default();
    let home = arguments
        .first()
        .and_then(static_string)
        .is_some_and(|value| value == b"HOME");
    let supported = match call_name(node) {
        b"[]" => arguments.len() == 1 && home,
        b"fetch" => {
            home && (arguments.len() == 1
                || arguments.len() == 2 && arguments[1].as_nil_node().is_some())
        }
        _ => false,
    };
    if supported {
        let range = context.source_file().node_range(&node.as_node());
        context.replace(
            "Use `Dir.home` instead.",
            range.clone(),
            range,
            "Dir.home",
        );
    }
}

fn ascii_comments(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let allowed = context.config_values("AllowedChars").to_vec();
    let parsed = ruby_prism::parse(source.as_bytes());
    for comment in parsed.comments() {
        let location = comment.location();
        let text = source
            .get(location.start_offset()..location.end_offset())
            .unwrap_or_default();
        let disallowed = |character: char| {
            !character.is_ascii()
                && character != '©'
                && !allowed.iter().any(|item| item == &character.to_string())
        };
        if !text.chars().any(disallowed) {
            continue;
        }
        let Some((relative, _)) = text.char_indices().find(|(_, character)| !character.is_ascii())
        else {
            continue;
        };
        let start = location.start_offset() + relative;
        let end = source[start..location.end_offset()]
            .char_indices()
            .take_while(|(_, character)| !character.is_ascii())
            .last()
            .map_or(start, |(relative, character)| {
                start + relative + character.len_utf8()
            });
        context.report("Use only ascii symbols in comments.", start..end);
    }
}
