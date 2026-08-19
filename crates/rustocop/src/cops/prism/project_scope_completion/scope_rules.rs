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
