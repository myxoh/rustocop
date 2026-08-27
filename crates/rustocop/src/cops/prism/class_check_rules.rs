use super::*;

pub(super) fn cops() -> Vec<Box<dyn Cop>> { Vec::new() }

fn on_send(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let current = call_name(node);
    if !matches!(current, b"is_a?" | b"kind_of?") {
        return;
    }
    let preferred = if current == b"is_a?" {
        b"kind_of?".as_slice()
    } else {
        b"is_a?".as_slice()
    };
    if context.policy().enforced_style("is_a?").as_bytes() == current {
        return;
    }
    let selector = node.message_loc().expect("class check selector");
    let current = String::from_utf8_lossy(current);
    let preferred = String::from_utf8_lossy(preferred);
    context.replace(
        format!("Prefer `Object#{preferred}` over `Object#{current}`."),
        &selector,
        &selector,
        preferred,
    );
}
