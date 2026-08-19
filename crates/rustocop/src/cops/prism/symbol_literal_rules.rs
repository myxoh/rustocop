use super::*;

define_cops! {
    SymbolLiteral => "Style/SymbolLiteral" => node(as_symbol_node, symbol_literal),
}

fn symbol_literal(node: &ruby_prism::SymbolNode<'_>, context: &mut CopContext<'_, '_>) {
    let location = node.location();
    let source = context.source_file().at(&location);
    let bytes = source.as_bytes();
    if bytes.len() < 4
        || bytes[0] != b':'
        || !matches!(bytes[1], b'\'' | b'"')
        || bytes.last() != Some(&bytes[1])
    {
        return;
    }
    let word = &bytes[2..bytes.len() - 1];
    if word.is_empty()
        || !(word[0].is_ascii_alphabetic() || word[0] == b'_')
        || word
            .iter()
            .any(|byte| !byte.is_ascii_alphanumeric() && *byte != b'_')
    {
        return;
    }
    context.replace(
        "Do not use strings for word-like symbol literals.",
        &location,
        &location,
        format!(":{}", String::from_utf8_lossy(word)),
    );
}
