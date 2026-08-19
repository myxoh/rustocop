use super::*;

pub(super) fn redundant_self_assignment(context: &mut CopContext<'_, '_>) {
    let mutating = [
        "concat", "collect!", "compact!", "delete", "delete_if", "fill", "flatten!", "insert",
        "keep_if", "map!", "merge!", "prepend", "push", "reject!", "replace", "reverse!",
        "rotate!", "select!", "shift", "shuffle!", "slice!", "sort!", "sort_by!", "store",
        "uniq!", "unshift", "update",
    ];
    for (offset, line) in context.source_file().lines() {
        let trimmed = line.trim();
        let Some((left, right)) = trimmed.split_once(" = ") else {
            continue;
        };
        if !right.starts_with(&format!("{left}.")) && !right.starts_with(&format!("{left}&.")) {
            continue;
        }
        let separator = usize::from(right[left.len()..].starts_with("&.")) + 1;
        let method = right[left.len() + separator..]
            .split(['(', ' ', '{'])
            .next()
            .unwrap_or_default();
        if !mutating.contains(&method) {
            continue;
        }
        let equals = offset + line.find('=').unwrap_or(0);
        let edit_start = offset + line.find(left).unwrap_or(0);
        context.remove(
            format!("Redundant self assignment detected. Method `{method}` modifies its receiver in place."),
            equals..equals + 1,
            edit_start..equals + 2,
        );
    }
}

pub(super) fn top_level_method_definition(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for (index, (offset, line)) in lines.iter().copied().enumerate() {
        if line.starts_with("def ") {
            context.report(
                "Do not define methods at the top-level.",
                offset..offset + line.len(),
            );
        } else if line.starts_with("define_method(") {
            let mut end = offset + line.len();
            if line.contains(" do") || line.ends_with("do") {
                for (candidate_offset, candidate) in &lines[index + 1..] {
                    if candidate.trim() == "end" {
                        end = candidate_offset + candidate.len();
                        break;
                    }
                }
            }
            context.report("Do not define methods at the top-level.", offset..end);
        }
    }
}
