use super::source_helpers::source_lines;
use super::*;

define_cops! {
    OneClassPerFile => "Style/OneClassPerFile" => source(one_class_per_file),
}

fn one_class_per_file(context: &mut CopContext<'_, '_>) {
    let allowed = context.config_values("AllowedClasses").to_vec();
    let mut definitions = 0;
    for (start, line) in source_lines(context.source()) {
        if line.starts_with(char::is_whitespace) {
            continue;
        }
        let declaration = line.trim_end();
        let Some(rest) = declaration
            .strip_prefix("class ")
            .or_else(|| declaration.strip_prefix("module "))
        else {
            continue;
        };
        let name = rest.split_whitespace().next().unwrap_or_default();
        if name.is_empty()
            || name == "<<"
            || allowed.iter().any(|allowed_name| allowed_name == name)
        {
            continue;
        }
        definitions += 1;
        if definitions > 1 {
            context.report(
                "Do not define multiple classes/modules at the top level in a single file.",
                start..start + declaration.len(),
            );
        }
    }
}
