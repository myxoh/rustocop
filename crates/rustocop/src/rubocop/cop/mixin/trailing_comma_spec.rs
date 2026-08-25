use super::trailing_comma::*;

fn location(start: usize, end: usize, line: usize, last_line: usize, source: &str) -> Location {
    Location {
        bytes: start..end,
        line,
        last_line,
        source: source.into(),
        begins_its_line: true,
    }
}

fn item(kind: &str, source: &str, line: usize) -> Item {
    Item {
        kind: kind.into(),
        source_range: location(0, source.len(), line, line, source),
        children: vec![],
        arguments: vec![],
        call_type: false,
        multiline: false,
        braces: false,
        block_pass: false,
        heredoc_body: false,
        end_location: None,
        selector_line: None,
    }
}

#[test]
fn comma_detection_changes_whitespace_rule_for_heredocs_and_respects_comments() {
    let cop = TrailingComma {
        style: TrailingCommaStyle::NoComma,
    };
    let plain = item("int", "1", 1);
    let range = location(10, 14, 1, 2, "\n ,");
    assert_eq!(
        cop.comma_offset(std::slice::from_ref(&plain), &range),
        Some(2)
    );
    let mut heredoc = plain;
    heredoc.heredoc_body = true;
    assert_eq!(cop.comma_offset(&[heredoc], &range), None);
    assert!(cop.inside_comment(&range, 2, Some(11)));
}

#[test]
fn each_style_retains_its_distinct_multiline_condition() {
    let mut node = item("array", "[\n1\n]", 1);
    node.multiline = true;
    let mut child = item("int", "1", 2);
    child.source_range.bytes = 2..3;
    node.children = vec![child];
    node.end_location = Some(location(4, 5, 3, 3, "]"));
    let comma = TrailingComma {
        style: TrailingCommaStyle::Comma,
    };
    assert!(comma.should_have_comma(TrailingCommaStyle::Comma, &node));
    assert!(comma.should_have_comma(TrailingCommaStyle::ConsistentComma, &node));
    node.source_range.source = "[\n1\n]".into();
    assert!(comma.should_have_comma(TrailingCommaStyle::DiffComma, &node));
    assert!(comma
        .check_literal(&node, "element in an array", None)
        .is_some());
}

#[test]
fn offenses_preserve_messages_ranges_actions_and_block_pass_suppression() {
    let cop = TrailingComma {
        style: TrailingCommaStyle::NoComma,
    };
    let offense = cop.avoid_comma("element in an array", 7, "");
    assert_eq!(offense.range, 7..8);
    assert_eq!(offense.action, CommaAction::Remove);
    let mut last = item("int", "value", 1);
    last.source_range.bytes = 4..9;
    assert_eq!(
        cop.put_comma(&[last.clone()], "%<article>s argument")
            .unwrap()
            .action,
        CommaAction::Add
    );
    last.block_pass = true;
    assert!(cop.put_comma(&[last], "%<article>s argument").is_none());
}

#[test]
fn nested_send_and_hash_heredocs_follow_last_child_recursion() {
    let cop = TrailingComma {
        style: TrailingCommaStyle::NoComma,
    };
    let mut body = item("str", "<<~SQL", 1);
    body.heredoc_body = true;
    let mut send = item("send", "value.strip", 1);
    send.call_type = true;
    send.children = vec![body.clone(), item("sym", "strip", 1)];
    assert!(cop.heredoc_send(&send));
    let mut pair = item("pair", "key: value", 1);
    pair.children = vec![item("sym", "key", 1), body];
    assert!(cop.heredoc(&pair));
}
