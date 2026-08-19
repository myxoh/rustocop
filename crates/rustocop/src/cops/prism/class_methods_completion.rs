use super::*;

declare_cops!(ClassMethodsDefinitions);
define_any_node_cop!(ClassMethodsDefinitions => "Style/ClassMethodsDefinitions" => class_methods_definitions);

fn class_methods_definitions(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("def_self");
    if style == "def_self" {
        let Some(singleton) = node.as_singleton_class_node() else {
            return;
        };
        if singleton.expression().as_self_node().is_none() {
            return;
        }
        let source = context.source_file().at(&singleton.location());
        if source
            .lines()
            .any(|line| matches!(line.trim(), "private" | "protected"))
        {
            return;
        }
        let public_source = source.split("\n    private").next().unwrap_or(source);
        let has_public_instance_method = public_source
            .lines()
            .map(str::trim_start)
            .any(|line| line.starts_with("def ") && !line.starts_with("def self."));
        if !has_public_instance_method {
            return;
        }
        let replacement = def_self_correction(source);
        context.replace(
            "Do not define public methods within class << self.",
            singleton.location(),
            singleton.location(),
            replacement,
        );
        return;
    }
    let Some(definition) = node.as_def_node() else {
        return;
    };
    if definition
        .receiver()
        .is_none_or(|receiver| receiver.as_self_node().is_none())
    {
        return;
    }
    context.report(
        "Use `class << self` to define a class method.",
        definition.location(),
    );
}

fn def_self_correction(source: &str) -> String {
    let lines = source.lines().collect::<Vec<_>>();
    if lines.len() < 3 {
        return source.to_string();
    }
    let header_indent = lines[0].len() - lines[0].trim_start().len();
    let mut extracted = Vec::<Vec<String>>::new();
    let mut removed = vec![false; lines.len()];
    let mut index = 1usize;
    while index + 1 < lines.len() {
        let trimmed = lines[index].trim_start();
        if !trimmed.starts_with("def ") || trimmed.starts_with("def self.") {
            index += 1;
            continue;
        }
        let mut start = index;
        while start > 1 && lines[start - 1].trim_start().starts_with('#') {
            start -= 1;
        }
        let mut depth = 0usize;
        let mut end = index;
        for (cursor, line) in lines.iter().enumerate().skip(index) {
            let candidate = line.trim();
            if cursor > index
                && (candidate.starts_with("def ")
                    || candidate.starts_with("class ")
                    || candidate.starts_with("module ")
                    || candidate.ends_with(" do"))
            {
                depth += 1;
            } else if candidate == "end" {
                if depth == 0 {
                    end = cursor;
                    break;
                }
                depth -= 1;
            }
        }
        let mut chunk = Vec::new();
        let extracted_indent = lines[index]
            .len()
            .saturating_sub(lines[index].trim_start().len())
            .saturating_sub(2);
        for (cursor, line) in lines.iter().enumerate().take(end + 1).skip(start) {
            removed[cursor] = true;
            let dedented = line.strip_prefix("  ").unwrap_or(line);
            if cursor == index {
                let indent = " ".repeat(extracted_indent);
                let definition = dedented.trim_start().replacen("def ", "def self.", 1);
                chunk.push(format!("{indent}{definition}"));
            } else {
                chunk.push(dedented.to_string());
            }
        }
        extracted.push(chunk);
        index = end + 1;
    }
    let meaningful_remaining =
        (1..lines.len() - 1).any(|cursor| !removed[cursor] && !lines[cursor].trim().is_empty());
    let extracted = extracted
        .into_iter()
        .map(|chunk| chunk.join("\n"))
        .collect::<Vec<_>>()
        .join("\n\n");
    if !meaningful_remaining {
        return extracted
            .strip_prefix(&" ".repeat(header_indent + 2))
            .unwrap_or(&extracted)
            .to_string();
    }
    let mut singleton = Vec::new();
    for (cursor, line) in lines.iter().enumerate() {
        if !removed[cursor] {
            singleton.push((*line).to_string());
        }
    }
    while singleton.len() > 2 && singleton.get(1).is_some_and(|line| line.trim().is_empty()) {
        singleton.remove(1);
    }
    format!("{}\n\n{extracted}", singleton.join("\n"))
}
