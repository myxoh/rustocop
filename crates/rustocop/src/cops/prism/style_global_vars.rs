use super::*;

define_cops! {
    GlobalVars => "Style/GlobalVars" => any_node(global_vars),
    PerlBackrefs => "Style/PerlBackrefs" => any_node(perl_backrefs),
}

const BUILT_INS: &[&[u8]] = &[
    b"$:",
    b"$LOAD_PATH",
    b"$\"",
    b"$LOADED_FEATURES",
    b"$0",
    b"$PROGRAM_NAME",
    b"$!",
    b"$ERROR_INFO",
    b"$@",
    b"$ERROR_POSITION",
    b"$;",
    b"$FS",
    b"$FIELD_SEPARATOR",
    b"$\x2c",
    b"$OFS",
    b"$OUTPUT_FIELD_SEPARATOR",
    b"$/",
    b"$RS",
    b"$INPUT_RECORD_SEPARATOR",
    b"$\\",
    b"$ORS",
    b"$OUTPUT_RECORD_SEPARATOR",
    b"$.",
    b"$NR",
    b"$INPUT_LINE_NUMBER",
    b"$_",
    b"$LAST_READ_LINE",
    b"$>",
    b"$DEFAULT_OUTPUT",
    b"$<",
    b"$DEFAULT_INPUT",
    b"$$",
    b"$PID",
    b"$PROCESS_ID",
    b"$?",
    b"$CHILD_STATUS",
    b"$~",
    b"$LAST_MATCH_INFO",
    b"$=",
    b"$IGNORECASE",
    b"$*",
    b"$ARGV",
    b"$&",
    b"$MATCH",
    b"$`",
    b"$PREMATCH",
    b"$'",
    b"$POSTMATCH",
    b"$+",
    b"$LAST_PAREN_MATCH",
    b"$stdin",
    b"$stdout",
    b"$stderr",
    b"$DEBUG",
    b"$FILENAME",
    b"$VERBOSE",
    b"$SAFE",
    b"$-0",
    b"$-a",
    b"$-d",
    b"$-F",
    b"$-i",
    b"$-I",
    b"$-l",
    b"$-p",
    b"$-v",
    b"$-w",
    b"$CLASSPATH",
    b"$JRUBY_VERSION",
    b"$JRUBY_REVISION",
    b"$ENV_JAVA",
];

fn global_vars(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let variable = if let Some(read) = node.as_global_variable_read_node() {
        Some((read.name().as_slice(), read.location()))
    } else if let Some(write) = node.as_global_variable_write_node() {
        Some((write.name().as_slice(), write.name_loc()))
    } else if let Some(write) = node.as_global_variable_and_write_node() {
        Some((write.name().as_slice(), write.name_loc()))
    } else if let Some(write) = node.as_global_variable_or_write_node() {
        Some((write.name().as_slice(), write.name_loc()))
    } else if let Some(write) = node.as_global_variable_operator_write_node() {
        Some((write.name().as_slice(), write.name_loc()))
    } else {
        node.as_global_variable_target_node()
            .map(|target| (target.name().as_slice(), target.location()))
    };
    let Some((name, location)) = variable else {
        return;
    };
    if BUILT_INS.contains(&name)
        || context
            .config_values("AllowedVariables")
            .iter()
            .any(|allowed| allowed.as_bytes() == name)
    {
        return;
    }
    context.report("Do not introduce global variables.", location);
}

fn perl_backrefs(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    let (name, location) = if let Some(read) = node.as_numbered_reference_read_node() {
        (format!("${}", read.number()), read.location())
    } else if let Some(read) = node.as_back_reference_read_node() {
        (
            String::from_utf8_lossy(read.name().as_slice()).into_owned(),
            read.location(),
        )
    } else if let Some(read) = node.as_global_variable_read_node() {
        (
            String::from_utf8_lossy(read.name().as_slice()).into_owned(),
            read.location(),
        )
    } else {
        return;
    };
    let suffix = match name.as_str() {
        "$&" | "$MATCH" => "(0)".to_string(),
        "$`" | "$PREMATCH" => ".pre_match".to_string(),
        "$'" | "$POSTMATCH" => ".post_match".to_string(),
        "$+" | "$LAST_PAREN_MATCH" => "(-1)".to_string(),
        name if name.strip_prefix('$').is_some_and(|digits| {
            !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
        }) =>
        {
            format!("({})", &name[1..])
        }
        _ => return,
    };
    let qualified = context
        .ancestors()
        .iter()
        .any(|ancestor| ancestor.as_module_node().is_some())
        && context
            .source()
            .lines()
            .any(|line| line.trim_start().starts_with("class Regexp"));
    let replacement = format!(
        "{}Regexp.last_match{suffix}",
        if qualified { "::" } else { "" }
    );
    let message = format!("Prefer `{replacement}` over `{name}`.");
    if let Some(embedded) = context.parent().and_then(Node::as_embedded_variable_node) {
        context.replace(
            message,
            &location,
            embedded.location(),
            format!("#{{{replacement}}}"),
        );
    } else {
        let range = location.start_offset()..location.end_offset();
        context.replace(message, range.clone(), range, replacement);
    }
}
