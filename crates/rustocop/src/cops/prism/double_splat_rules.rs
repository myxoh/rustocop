use super::*;

define_cops! {
    RedundantDoubleSplatHashBraces => "Style/RedundantDoubleSplatHashBraces" => node(as_assoc_splat_node, redundant_double_splat_hash_braces),
}

const MESSAGE: &str =
    "Remove the redundant double splat and braces, use keyword arguments directly.";

fn redundant_double_splat_hash_braces(
    node: &ruby_prism::AssocSplatNode<'_>,
    context: &mut CopContext<'_, '_>,
) {
    let Some(value) = node.value() else {
        return;
    };
    let Some(parts) = flatten_hash_expression(&value, context.source_file()) else {
        return;
    };
    context.replace(MESSAGE, node.location(), node.location(), parts.join(", "));
}

fn flatten_hash_expression(node: &Node<'_>, file: SourceFile<'_>) -> Option<Vec<String>> {
    if let Some(hash) = node.as_hash_node() {
        return flatten_hash(&hash, file);
    }
    let call = node.as_call_node()?;
    if !matches!(call_name(&call), b"merge" | b"merge!") || call.block().is_some() {
        return None;
    }
    let mut parts = flatten_hash_expression(&call.receiver()?, file)?;
    let arguments = call.arguments()?;
    for argument in arguments.arguments().iter() {
        if let Some(keywords) = argument.as_keyword_hash_node() {
            parts.extend(flatten_keyword_hash(&keywords, file)?);
        } else {
            parts.push(format!("**{}", file.node(&argument)));
        }
    }
    Some(parts)
}

fn flatten_hash(hash: &ruby_prism::HashNode<'_>, file: SourceFile<'_>) -> Option<Vec<String>> {
    if hash.elements().is_empty() {
        return None;
    }
    flatten_elements(hash.elements().iter(), file)
}

fn flatten_keyword_hash(
    hash: &ruby_prism::KeywordHashNode<'_>,
    file: SourceFile<'_>,
) -> Option<Vec<String>> {
    flatten_elements(hash.elements().iter(), file)
}

fn flatten_elements<'pr>(
    elements: impl Iterator<Item = Node<'pr>>,
    file: SourceFile<'pr>,
) -> Option<Vec<String>> {
    let mut parts = Vec::new();
    for element in elements {
        if let Some(pair) = element.as_assoc_node() {
            if pair.operator_loc().is_some() {
                return None;
            }
            parts.push(file.node(&element).trim().to_string());
        } else if let Some(splat) = element.as_assoc_splat_node() {
            let value = splat.value()?;
            if let Some(nested) = flatten_hash_expression(&value, file) {
                parts.extend(nested);
            } else {
                parts.push(format!("**{}", file.node(&value)));
            }
        } else {
            return None;
        }
    }
    Some(parts)
}
