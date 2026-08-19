use ruby_prism::Location;

use super::*;

define_cops! {
    NilLambda => "Style/NilLambda" => any_node(nil_lambda),
}

fn nil_lambda(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
    if let Some(lambda) = node.as_lambda_node() {
        if always_nil(lambda.body()) {
            report_nil_callable(
                node.location(),
                lambda.opening_loc(),
                lambda.closing_loc(),
                lambda.body(),
                "lambda",
                context,
            );
        }
        return;
    }

    let Some(call) = node.as_call_node() else {
        return;
    };
    let kind = if call.receiver().is_none() && call_name(&call) == b"lambda" {
        "lambda"
    } else if call.receiver().is_none() && call_name(&call) == b"proc"
        || call_name(&call) == b"new" && root_constant(call.receiver(), b"Proc")
    {
        "proc"
    } else {
        return;
    };
    let Some(block) = call.block().and_then(|block| block.as_block_node()) else {
        return;
    };
    if !always_nil(block.body()) {
        return;
    }
    report_nil_callable(
        call.location(),
        block.opening_loc(),
        block.closing_loc(),
        block.body(),
        kind,
        context,
    );
}

fn always_nil(body: Option<Node<'_>>) -> bool {
    let Some(body) = body else {
        return false;
    };
    let Some(statements) = body.as_statements_node() else {
        return false;
    };
    let statements = statements.body();
    if statements.len() != 1 {
        return false;
    }
    let Some(statement) = statements.first() else {
        return false;
    };
    if statement.as_nil_node().is_some() {
        return true;
    }
    let arguments = if let Some(exit) = statement.as_return_node() {
        exit.arguments()
    } else if let Some(exit) = statement.as_break_node() {
        exit.arguments()
    } else if let Some(exit) = statement.as_next_node() {
        exit.arguments()
    } else {
        return false;
    };
    arguments.is_some_and(|arguments| {
        let values = arguments.arguments();
        values.len() == 1
            && values
                .first()
                .is_some_and(|value| value.as_nil_node().is_some())
    })
}

fn report_nil_callable(
    offense: Location<'_>,
    opening: Location<'_>,
    closing: Location<'_>,
    body: Option<Node<'_>>,
    kind: &str,
    context: &mut CopContext<'_, '_>,
) {
    let Some(body) = body else {
        return;
    };
    let file = context.source_file();
    let edit = if file.same_line(opening.start_offset(), closing.start_offset()) {
        opening.end_offset()..closing.start_offset()
    } else {
        file.line_start(body.location().start_offset())..file.line_start(closing.start_offset())
    };
    context.remove(
        format!("Use an empty {kind} instead of always returning nil."),
        &offense,
        edit,
    );
}
