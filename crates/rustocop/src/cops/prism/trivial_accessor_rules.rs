use super::*;

define_cops! {
    TrivialAccessors => "Style/TrivialAccessors" => node(as_def_node, trivial_accessors),
}

fn trivial_accessors(node: &ruby_prism::DefNode<'_>, context: &mut CopContext<'_, '_>) {
    if !trivial_accessor_context(context.ancestors())
        || allowed_method(node.name().as_slice(), context)
    {
        return;
    }
    let class_method = node.receiver().is_some();
    if class_method && context.config_bool("IgnoreClassMethods", false) {
        return;
    }
    let correctable_class_method = node
        .receiver()
        .is_some_and(|receiver| receiver.as_self_node().is_some());
    let name = String::from_utf8_lossy(node.name().as_slice()).into_owned();
    if name == "initialize" {
        return;
    }
    let Some(body) = node.body().and_then(single_expression) else {
        return;
    };
    let candidate = if let Some(read) = body.as_instance_variable_read_node() {
        if !empty_parameters(node.parameters()) {
            return;
        }
        AccessorCandidate {
            kind: "reader",
            macro_name: "attr_reader",
            ivar: read.name().as_slice(),
            conventional: !name.ends_with('?'),
        }
    } else if let Some(write) = body.as_instance_variable_write_node() {
        let Some(parameter) = single_required_parameter(node.parameters()) else {
            return;
        };
        if write
            .value()
            .as_local_variable_read_node()
            .is_none_or(|read| read.name().as_slice() != parameter)
        {
            return;
        }
        let writer = name.ends_with('=');
        if !writer && context.config_bool("AllowDSLWriters", true) {
            return;
        }
        AccessorCandidate {
            kind: "writer",
            macro_name: "attr_writer",
            ivar: write.name().as_slice(),
            conventional: writer,
        }
    } else {
        return;
    };

    if name.ends_with('?') && context.config_bool("AllowPredicates", true) {
        return;
    }
    let attribute = name.trim_end_matches(['=', '?']);
    let exact_name = candidate
        .ivar
        .strip_prefix(b"@")
        .is_some_and(|ivar| ivar == attribute.as_bytes());
    if context.config_bool("ExactNameMatch", true) && !exact_name {
        return;
    }
    let message = format!(
        "Use `{}` to define trivial {} methods.",
        candidate.macro_name, candidate.kind
    );
    let keyword = node.def_keyword_loc();
    let prefix = context
        .source_file()
        .slice(context.source_file().line_start(keyword.start_offset())..keyword.start_offset())
        .unwrap_or_default();
    if !candidate.conventional
        || !exact_name
        || !prefix.trim().is_empty()
        || class_method && !correctable_class_method
    {
        context.report(message, &keyword);
        return;
    }
    let accessor = format!("{} :{attribute}", candidate.macro_name);
    let replacement = if class_method {
        format!("class << self\n{prefix}  {accessor}\n{prefix}end")
    } else {
        accessor
    };
    context.replace(message, &keyword, node.location(), replacement);
}

fn allowed_method(method: &[u8], context: &CopContext<'_, '_>) -> bool {
    const DEFAULT_ALLOWED_METHODS: &[&[u8]] = &[
        b"to_ary", b"to_a", b"to_c", b"to_enum", b"to_h", b"to_hash", b"to_i", b"to_int",
        b"to_io", b"to_open", b"to_path", b"to_proc", b"to_r", b"to_regexp", b"to_str",
        b"to_s", b"to_sym",
    ];

    context.policy().allows_method(method)
        || (!context.config_contains("AllowedMethods")
            && DEFAULT_ALLOWED_METHODS.contains(&method))
}

struct AccessorCandidate<'pr> {
    kind: &'static str,
    macro_name: &'static str,
    ivar: &'pr [u8],
    conventional: bool,
}

fn trivial_accessor_context(ancestors: &[Node<'_>]) -> bool {
    let mut inside_block = false;
    for ancestor in ancestors.iter().rev() {
        if ancestor.as_class_node().is_some() || ancestor.as_singleton_class_node().is_some() {
            return true;
        }
        if ancestor.as_module_node().is_some() {
            return false;
        }
        if ancestor.as_block_node().is_some() {
            inside_block = true;
        }
        if ancestor
            .as_call_node()
            .is_some_and(|call| call_name(&call) == b"instance_eval")
        {
            return false;
        }
    }
    inside_block
}

fn empty_parameters(parameters: Option<ruby_prism::ParametersNode<'_>>) -> bool {
    parameters.is_none_or(|parameters| {
        parameters.requireds().is_empty()
            && parameters.optionals().is_empty()
            && parameters.rest().is_none()
            && parameters.posts().is_empty()
            && parameters.keywords().is_empty()
            && parameters.keyword_rest().is_none()
            && parameters.block().is_none()
    })
}

fn single_required_parameter(parameters: Option<ruby_prism::ParametersNode<'_>>) -> Option<&[u8]> {
    let parameters = parameters?;
    if parameters.requireds().len() != 1
        || !parameters.optionals().is_empty()
        || parameters.rest().is_some()
        || !parameters.posts().is_empty()
        || !parameters.keywords().is_empty()
        || parameters.keyword_rest().is_some()
        || parameters.block().is_some()
    {
        return None;
    }
    parameters
        .requireds()
        .first()?
        .as_required_parameter_node()
        .map(|parameter| parameter.name().as_slice())
}
