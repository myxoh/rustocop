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
        if !source.starts_with("=begin") || source.len() < "=begin\n".len() {
            continue;
        }
        let begin_end = "=begin\n".len();
        let (end_start, end_end) = if source.ends_with('\n') {
            (source.len().saturating_sub("=end\n".len()), source.len())
        } else {
            (
                source.len().saturating_sub("\n=end".len() + 1),
                source.len().saturating_sub(2),
            )
        };
        if begin_end > end_start || end_start > end_end {
            continue;
        }
        let contents = &source[begin_end..end_start];
        let mut replacement = String::new();
        if !contents.is_empty() {
            replacement.push_str("# ");
            let bytes = contents.as_bytes();
            for (index, character) in contents.char_indices() {
                replacement.push(character);
                if character == '\n' {
                    if bytes.get(index + 1) == Some(&b'\n') {
                        replacement.push('#');
                    } else if bytes.get(index + 1).is_some_and(|byte| *byte != b'#') {
                        replacement.push_str("# ");
                    }
                }
            }
        }
        replacement.push_str(&source[end_end..]);

        context.replace(
            "Do not use block comments.",
            &location,
            &location,
            replacement,
        );
    }
}
