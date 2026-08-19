use super::catalog_cop::{custom, replace, report};
use super::*;
use std::collections::HashSet;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    vec![
        custom(
            "Lint/DuplicateRegexpCharacterClassElement",
            duplicate_character_class,
        ),
        replace(
            "Lint/RedundantRegexpQuantifiers",
            "{1}",
            "",
            "Use a single regexp atom instead of a `{1}` quantifier.",
        ),
        report(
            "Lint/UnescapedBracketInRegexp",
            "/[/",
            "Regular expression has an unescaped open bracket.",
        ),
        report(
            "Lint/AmbiguousRegexpLiteral",
            "puts /",
            "Ambiguous regexp literal. Parenthesize the method arguments.",
        ),
        custom(
            "Style/RedundantRegexpCharacterClass",
            redundant_character_class,
        ),
        report(
            "Lint/ArrayLiteralInRegexp",
            "Regexp.new([",
            "Passing an array to `Regexp.new` is invalid.",
        ),
        replace(
            "Style/RedundantRegexpEscape",
            "\\:",
            ":",
            "Redundant escape inside regexp literal.",
        ),
        custom("Style/RegexpLiteral", regexp_literal),
        replace(
            "Style/RedundantRegexpArgument",
            ".match(/foo/, 0)",
            ".match(/foo/)",
            "Remove the redundant regexp match position argument.",
        ),
        custom("Lint/OutOfRangeRegexpRef", out_of_range_ref),
        report(
            "Style/SelectByRegexp",
            ".select { |x| x =~ /",
            "Prefer `grep` to selecting by regexp.",
        ),
    ]
}

fn regexp_ranges(source: &str) -> impl Iterator<Item = (usize, usize)> + '_ {
    source.match_indices('/').filter_map(|(start, _)| {
        source[start + 1..]
            .find('/')
            .map(|relative| (start, start + 1 + relative))
    })
}

fn duplicate_character_class(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    for (regexp_start, regexp_end) in regexp_ranges(&source) {
        let regexp = &source[regexp_start + 1..regexp_end];
        if regexp.contains('\\') || regexp.contains("&&") || regexp.contains("#{") {
            continue;
        }
        let Some(open) = regexp.find('[') else {
            continue;
        };
        let Some(close) = regexp[open + 1..].find(']').map(|at| open + 1 + at) else {
            continue;
        };
        let mut seen = HashSet::new();
        for (relative, character) in regexp[open + 1..close].char_indices() {
            if !seen.insert(character) {
                let start = regexp_start + 1 + open + 1 + relative;
                context.remove(
                    "Duplicate element inside regexp character class.",
                    start..start + character.len_utf8(),
                    start..start + character.len_utf8(),
                );
            }
        }
    }
}

fn redundant_character_class(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    for (regexp_start, regexp_end) in regexp_ranges(&source) {
        let regexp = &source[regexp_start + 1..regexp_end];
        if regexp.starts_with('[')
            && regexp.ends_with(']')
            && regexp[1..regexp.len() - 1].chars().count() == 1
            && !matches!(&regexp[1..regexp.len() - 1], "+" | " " | "-" | "#")
        {
            context.replace(
                "Redundant single-element regexp character class.",
                regexp_start + 1..regexp_end,
                regexp_start + 1..regexp_end,
                &regexp[1..regexp.len() - 1],
            );
        }
    }
}

fn regexp_literal(context: &mut CopContext<'_, '_>) {
    let source = context.source().to_string();
    for quote in ['\'', '"'] {
        let needle = format!("Regexp.new({quote}");
        let mut search = 0;
        while let Some(relative) = source[search..].find(&needle) {
            let start = search + relative;
            let body_start = start + needle.len();
            let Some(close) = source[body_start..].find(quote).map(|at| body_start + at) else {
                break;
            };
            if source.as_bytes().get(close + 1) == Some(&b')') {
                context.replace(
                    "Use a regexp literal instead of `Regexp.new`.",
                    start..close + 2,
                    start..close + 2,
                    format!("/{}/", &source[body_start..close]),
                );
            }
            search = close + 1;
        }
    }
}

fn out_of_range_ref(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        for (at, _) in line.match_indices('$') {
            let digits = line[at + 1..]
                .bytes()
                .take_while(u8::is_ascii_digit)
                .count();
            if digits > 0
                && line[at + 1..at + 1 + digits]
                    .parse::<usize>()
                    .is_ok_and(|value| value > 9)
            {
                context.report(
                    "Back reference is out of range.",
                    offset + at..offset + at + digits + 1,
                );
            }
        }
    }
}
