use super::*;

define_cops! {
    HashAsLastArrayItem => "Style/HashAsLastArrayItem" => compatibility_prism_node(as_array_node, hash_as_last_array_item),
}

fn hash_as_last_array_item(node: &ruby_prism::ArrayNode<'_>, context: &mut CopContext<'_, '_>) {
    if node.opening_loc().is_none() || node.closing_loc().is_none() {
        return;
    }
    let elements = node.elements().iter().collect::<Vec<_>>();
    let Some(last) = elements.last() else {
        return;
    };
    if elements.len() > 1
        && (elements[elements.len() - 2].as_hash_node().is_some()
            || elements[elements.len() - 2]
                .as_keyword_hash_node()
                .is_some())
    {
        return;
    }
    let style = context.policy().enforced_style("braces").to_string();
    if style == "braces" {
        let Some(hash) = last.as_keyword_hash_node() else {
            return;
        };
        if hash.elements().is_empty()
            || hash
                .elements()
                .iter()
                .any(|element| element.as_assoc_splat_node().is_some())
        {
            return;
        }
        let location = hash.location();
        let file = context.source_file();
        let multiline_only_hash = elements.len() == 1
            && node.opening_loc().is_some_and(|opening| {
                !file.same_line(opening.start_offset(), location.start_offset())
            });
        let (opening, closing) = if multiline_only_hash {
            let indentation = file.column(location.start_offset());
            (
                format!("{{\n{}", " ".repeat(indentation)),
                format!("\n{}}}", " ".repeat(indentation)),
            )
        } else {
            ("{".to_string(), "}".to_string())
        };
        context.replace_many(
            "Wrap hash in `{` and `}`.",
            &location,
            vec![
                (location.start_offset()..location.start_offset(), opening),
                (location.end_offset()..location.end_offset(), closing),
            ],
        );
    } else {
        let Some(hash) = last.as_hash_node() else {
            return;
        };
        if hash.elements().is_empty()
            || elements.len() > 1
                && elements
                    .iter()
                    .all(|element| element.as_hash_node().is_some())
        {
            return;
        }
        let (opening, closing) = (hash.opening_loc(), hash.closing_loc());
        let closing_end = if context.source().as_bytes().get(closing.end_offset()) == Some(&b',') {
            closing.end_offset() + 1
        } else {
            closing.end_offset()
        };
        context.replace_many(
            "Omit the braces around the hash.",
            hash.location(),
            vec![
                (opening.start_offset()..opening.end_offset(), String::new()),
                (closing.start_offset()..closing_end, String::new()),
            ],
        );
    }
}
