use super::*;

define_cops! {
    SwapValues => "Style/SwapValues" => source(swap_values),
}

fn swap_values(context: &mut CopContext<'_, '_>) {
    let source = context.source();
    let assignments = context
        .source_file()
        .lines()
        .enumerate()
        .filter_map(|(index, (start, line))| {
            let code = line.split('#').next().unwrap_or_default().trim();
            let (left, right) = code.split_once(" = ")?;
            (!left.contains(',') && !right.contains(',')).then_some((
                index + 1,
                start,
                code,
                left,
                right,
            ))
        })
        .collect::<Vec<_>>();
    if assignments.len() != 3 {
        return;
    }
    let (first_line, first_start, first_code, temporary, first_value) = assignments[0];
    let (second_line, _, _, second_left, second_value) = assignments[1];
    let (third_line, _, _, third_left, third_value) = assignments[2];
    if second_left != first_value || third_left != second_value || third_value != temporary {
        return;
    }
    let replacement = format!("{second_left}, {third_left} = {third_left}, {second_left}");
    context.replace(
        format!(
            "Replace this and assignments at lines {second_line} and {third_line} with `{replacement}`."
        ),
        first_start..first_start + first_code.len(),
        0..source.len(),
        format!("{replacement}\n"),
    );
    let _ = first_line;
}
