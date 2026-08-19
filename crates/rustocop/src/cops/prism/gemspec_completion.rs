use std::collections::HashMap;

use super::*;

define_cops! {
    OrderedDependencies => "Gemspec/OrderedDependencies" => source(ordered_dependencies),
    DuplicatedAssignment => "Gemspec/DuplicatedAssignment" => source(duplicated_assignment),
}

#[derive(Clone)]
struct Dependency {
    method: String,
    name: String,
    line_start: usize,
    line_end: usize,
    chunk_start: usize,
}

fn ordered_dependencies(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let comments_separate = context.config_bool("TreatCommentsAsGroupSeparators", false);
    let punctuation = context.config_bool("ConsiderPunctuation", false);
    let lines = context.source_file().lines().collect::<Vec<_>>();
    let mut dependencies = Vec::new();
    for (index, (offset, line)) in lines.iter().enumerate() {
        let trimmed = line.trim();
        let Some(method_at) = trimmed.find(".add_") else {
            continue;
        };
        let method_end = trimmed[method_at + 1..]
            .find(' ')
            .map(|at| method_at + 1 + at)
            .unwrap_or(trimmed.len());
        let method = trimmed[method_at + 1..method_end].to_string();
        let argument = trimmed[method_end..].trim();
        let name = argument
            .trim_matches(['\'', '"'])
            .split([',', ' '])
            .next()
            .unwrap_or_default()
            .to_string();
        if name.is_empty() {
            continue;
        }
        let mut chunk_start = *offset;
        if !comments_separate {
            let mut previous = index;
            while previous > 0 && lines[previous - 1].1.trim_start().starts_with('#') {
                previous -= 1;
                chunk_start = lines[previous].0;
            }
        }
        dependencies.push(Dependency {
            method,
            name,
            line_start: *offset,
            line_end: offset
                + line.len()
                + usize::from(source.as_bytes().get(offset + line.len()) == Some(&b'\n')),
            chunk_start,
        });
    }
    for pair in dependencies.windows(2) {
        let (previous, current) = (&pair[0], &pair[1]);
        if previous.method != current.method
            || comments_separate && current.chunk_start > previous.line_end
            || source[previous.line_end..current.chunk_start].contains('\n')
        {
            continue;
        }
        let normalized = |name: &str| {
            if punctuation {
                name.to_string()
            } else {
                name.chars()
                    .filter(|character| character.is_alphanumeric())
                    .collect()
            }
        };
        if normalized(&previous.name).to_lowercase() <= normalized(&current.name).to_lowercase() {
            continue;
        }
        let current_line = &source[current.line_start..current.line_end];
        let indentation = current_line.len() - current_line.trim_start().len();
        let current_chunk = &source[current.chunk_start..current.line_end];
        let previous_chunk = &source[previous.chunk_start..previous.line_end];
        context.replace_many(
            format!("Dependencies should be sorted in an alphabetical order within their section of the gemspec. Dependency `{}` should appear before `{}`.", current.name, previous.name),
            current.line_start + indentation
                ..current.line_end - usize::from(current_line.ends_with('\n')),
            vec![
                (previous.chunk_start..previous.line_end, current_chunk.to_string()),
                (current.chunk_start..current.line_end, previous_chunk.to_string()),
            ],
        );
    }
}

fn duplicated_assignment(context: &mut CopContext<'_, '_>) {
    let mut seen: HashMap<String, usize> = HashMap::new();
    let specification = context
        .source()
        .lines()
        .find_map(|line| line.split('|').nth(1))
        .unwrap_or("spec")
        .trim();
    for (line_number, (offset, line)) in context.source_file().lines().enumerate() {
        let trimmed = line.trim();
        let Some((left, _)) = trimmed.split_once(" = ") else {
            continue;
        };
        let receiver = left.split('.').next().unwrap_or_default();
        if !matches!(receiver, "_1" | "it") && receiver != specification {
            continue;
        }
        let (identity, key) = if let Some(metadata) = left.split(".metadata[").nth(1) {
            let raw = metadata.trim_end_matches(']');
            if !raw.starts_with(['\'', '"', ':']) {
                continue;
            }
            let value = raw.trim_start_matches(':').trim_matches(['\'', '"']);
            if raw.starts_with(':') {
                (
                    format!("metadata:symbol:{value}"),
                    format!("metadata[:{value}]="),
                )
            } else {
                (
                    format!("metadata:string:{value}"),
                    format!("metadata['{value}']="),
                )
            }
        } else {
            let Some(attribute) = left.rsplit('.').next() else {
                continue;
            };
            if attribute.is_empty() {
                continue;
            }
            let key = format!("{attribute}=");
            (key.clone(), key)
        };
        if let Some(first_line) = seen.get(&identity) {
            let start = offset + line.len() - line.trim_start().len();
            context.report(
                format!("`{key}` method calls already given on line {first_line} of the gemspec."),
                start..offset + line.len(),
            );
        } else {
            seen.insert(identity, line_number + 1);
        }
    }
}
