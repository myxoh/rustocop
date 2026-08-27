use super::*;

define_cops! {
    RandomWithOffset => "Style/RandomWithOffset" => compatibility_prism_call(random_with_offset),
}

const MESSAGE: &str =
    "Prefer ranges when generating random numbers instead of integers with offsets.";

fn random_with_offset(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
    let result = match call_name(node) {
        b"succ" | b"next" | b"pred" if argument_count(node) == 0 => {
            let random = node.receiver().and_then(|receiver| receiver.as_call_node());
            random.and_then(|random| {
                let (low, high) = random_interval(&random)?;
                let offset = if call_name(node) == b"pred" { -1 } else { 1 };
                Some((random, low + offset, high + offset))
            })
        }
        b"+" | b"-" => arithmetic_interval(node),
        _ => None,
    };
    let Some((random, low, high)) = result else {
        return;
    };
    let callee = if let Some(receiver) = random.receiver() {
        let operator = random
            .call_operator_loc()
            .map(|operator| String::from_utf8_lossy(operator.as_slice()).into_owned())
            .unwrap_or_else(|| ".".to_string());
        format!("{}{operator}rand", context.source_file().node(&receiver))
    } else {
        "rand".to_string()
    };
    context.replace_call(node, MESSAGE, format!("{callee}({low}..{high})"));
}

fn arithmetic_interval<'pr>(node: &CallNode<'pr>) -> Option<(CallNode<'pr>, i32, i32)> {
    let left = node.receiver()?;
    let right = only_argument(node)?;
    if let Some(random) = left.as_call_node().filter(is_random_call) {
        let offset = integer_value(&right)?;
        let (low, high) = random_interval(&random)?;
        return if call_name(node) == b"+" {
            Some((random, low + offset, high + offset))
        } else {
            Some((random, low - offset, high - offset))
        };
    }
    let random = right.as_call_node().filter(is_random_call)?;
    let offset = integer_value(&left)?;
    let (low, high) = random_interval(&random)?;
    if call_name(node) == b"+" {
        Some((random, low + offset, high + offset))
    } else {
        Some((random, offset - high, offset - low))
    }
}

fn random_interval(node: &CallNode<'_>) -> Option<(i32, i32)> {
    if !is_random_call(node) {
        return None;
    }
    let argument = only_argument(node)?;
    if let Some(size) = integer_value(&argument) {
        return Some((0, size - 1));
    }
    let range = argument.as_range_node()?;
    let low = integer_value(&range.left()?)?;
    let mut high = integer_value(&range.right()?)?;
    if range.operator_loc().as_slice() == b"..." {
        high -= 1;
    }
    Some((low, high))
}

fn is_random_call(node: &CallNode<'_>) -> bool {
    call_name(node) == b"rand"
        && (node.receiver().is_none()
            || root_constant(node.receiver(), b"Kernel")
            || root_constant(node.receiver(), b"Random"))
        && argument_count(node) == 1
}

fn integer_value(node: &Node<'_>) -> Option<i32> {
    let integer = node.as_integer_node()?;
    TryInto::<i32>::try_into(integer.value()).ok()
}
