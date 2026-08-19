use std::ops::Range;

use super::*;

declare_source_cops! {
    RequireOrder => "Style/RequireOrder" => require_order,
}

#[derive(Clone)]
struct RequireItem {
    kind: &'static str,
    key: String,
    chunk: Range<usize>,
    offense: Range<usize>,
}

fn require_order(source: &str, reporter: &mut Reporter<'_>) {
    let lines = SourceFile::new(source)
        .lines()
        .map(|(start, line)| {
            let end = source[start..]
                .find('\n')
                .map_or(source.len(), |relative| start + relative + 1);
            (start, end, line)
        })
        .collect::<Vec<_>>();
    let mut group = Vec::new();
    let mut pending_comment = None;

    for (start, end, line) in lines {
        let trimmed = line.trim_start();
        if trimmed.starts_with('#') {
            pending_comment.get_or_insert(start);
            continue;
        }
        if trimmed.is_empty() {
            inspect_group(source, &mut group, reporter);
            pending_comment = None;
            continue;
        }
        let Some((kind, key, code_length)) = parse_require(trimmed) else {
            inspect_group(source, &mut group, reporter);
            pending_comment = None;
            continue;
        };
        if group
            .first()
            .is_some_and(|item: &RequireItem| item.kind != kind)
        {
            inspect_group(source, &mut group, reporter);
        }
        let indentation = line.len() - trimmed.len();
        group.push(RequireItem {
            kind,
            key,
            chunk: pending_comment.take().unwrap_or(start)..end,
            offense: start + indentation..start + indentation + code_length,
        });
    }
    inspect_group(source, &mut group, reporter);
}

fn parse_require(line: &str) -> Option<(&'static str, String, usize)> {
    let (kind, rest) = if let Some(rest) = line.strip_prefix("require_relative ") {
        ("require_relative", rest)
    } else if let Some(rest) = line.strip_prefix("require ") {
        ("require", rest)
    } else {
        return None;
    };
    let quote = rest.as_bytes().first().copied()?;
    if !matches!(quote, b'\'' | b'"') {
        return None;
    }
    let closing = rest[1..].find(char::from(quote))? + 1;
    let key = rest[1..closing].to_string();
    Some((kind, key, kind.len() + 1 + closing + 1))
}

fn inspect_group(source: &str, group: &mut Vec<RequireItem>, reporter: &mut Reporter<'_>) {
    if group.len() < 2 {
        group.clear();
        return;
    }
    let mut greatest = &group[0].key;
    let mut offending = Vec::new();
    for (index, item) in group.iter().enumerate().skip(1) {
        if item.key < *greatest {
            offending.push(index);
        } else {
            greatest = &item.key;
        }
    }
    if offending.is_empty() {
        group.clear();
        return;
    }

    let mut sorted = group.clone();
    sorted.sort_by(|left, right| left.key.cmp(&right.key));
    let edits = group
        .iter()
        .zip(&sorted)
        .map(|(destination, item)| {
            (
                destination.chunk.clone(),
                source[item.chunk.clone()].to_string(),
            )
        })
        .collect::<Vec<_>>();
    let message = format!("Sort `{}` in alphabetical order.", group[0].kind);
    let first = offending[0];
    reporter.replace_many(message.clone(), group[first].offense.clone(), edits);
    for index in offending.into_iter().skip(1) {
        let offense = group[index].offense.clone();
        reporter.replace(
            message.clone(),
            offense.clone(),
            offense.clone(),
            source[offense].to_string(),
        );
    }
    group.clear();
}
