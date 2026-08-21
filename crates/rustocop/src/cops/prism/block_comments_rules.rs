use super::*;

define_cops! {
    BlockComments => "Style/BlockComments" => source(on_new_investigation),
}

fn on_new_investigation(context: &mut CopContext<'_, '_>) {
    let parsed = ruby_prism::parse(context.source().as_bytes());

    for comment in parsed.comments() {
        let location = comment.location();
        let source = context.source_file().at(&location);

        // Prism only exposes an embdoc as a comment when the markers are Ruby
        // syntax. Checking the parsed comment text distinguishes it from
        // `=begin` embedded in heredocs, strings, and ordinary `#` comments.
        let Some(body_and_end) = source.strip_prefix("=begin\n") else {
            continue;
        };
        let Some(body) = body_and_end.strip_suffix("=end\n").or_else(|| body_and_end.strip_suffix("=end")) else {
            continue;
        };

        let replacement = body
            .strip_suffix('\n')
            .unwrap_or(body)
            .lines()
            .map(|line| {
                if line.is_empty() {
                    "#".to_string()
                } else {
                    format!("# {line}")
                }
            })
            .collect::<Vec<_>>()
            .join("\n");
        let replacement = if replacement.is_empty() {
            replacement
        } else {
            format!("{replacement}\n")
        };

        context.replace(
            "Do not use block comments.",
            &location,
            &location,
            replacement,
        );
    }
}
