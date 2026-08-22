use super::catalog_cop::custom;
use super::*;

mod registry;

pub(super) fn cops() -> Vec<Box<dyn Cop>> {
    let mut cops = vec![
        custom("Layout/LineContinuationSpacing", line_continuation_spacing),
        custom(
            "Layout/MultilineMethodDefinitionBraceLayout",
            method_definition_brace_layout,
        ),
        custom("Layout/ArrayAlignment", align_continuation),
        custom("Layout/SpaceInsideParens", space_inside_parens),
        custom("Layout/ClosingParenthesisIndentation", closing_parenthesis),
        custom("Layout/IndentationStyle", indentation_style),
        custom("Layout/LeadingCommentSpace", leading_comment_space),
        custom("Layout/CommentIndentation", comment_indentation),
        custom("Layout/ElseAlignment", keyword_alignment),
        custom(
            "Layout/AccessModifierIndentation",
            access_modifier_indentation,
        ),
        custom("Layout/MultilineHashBraceLayout", hash_brace_layout),
        custom("Layout/CaseIndentation", case_indentation),
    ];
    cops.extend(registry::cops());
    cops
}

fn leading_comment_space(context: &mut CopContext<'_, '_>) {
    let shebang_file = context.source().lines().next().is_some_and(|line| line.starts_with("#!"));
    let config_ru = context.path().rsplit('/').next() == Some("config.ru");
    let gemfile = context.path().rsplit('/').next() == Some("Gemfile");
    let parsed = parse(context.source().as_bytes());
    for parsed_comment in parsed.comments() {
        if parsed_comment.type_() != ruby_prism::CommentType::InlineComment {
            continue;
        }
        let location = parsed_comment.location();
        let comment = context.source_file().at(&location);
        let content = &comment[1..];
        let hashes = comment.bytes().take_while(|byte| *byte == b'#').count();
        let multiple_hash_comment = hashes > 1
            && (hashes == comment.len()
                || comment[hashes..].starts_with(char::is_whitespace));
        let first_line = context.source_file().line_start(location.start_offset()) == 0;
        if content.is_empty()
            || content.starts_with(char::is_whitespace)
            || multiple_hash_comment
            || content.starts_with('=')
            || content.starts_with("++")
            || content.starts_with("--")
            || shebang_file && content.starts_with('!')
            || first_line && config_ru && content.starts_with('\\')
            || context.config_bool("AllowDoxygenCommentStyle", false) && content.starts_with('*')
            || context.config_bool("AllowGemfileRubyComment", false)
                && gemfile
                && (content.starts_with("ruby=") || content.starts_with("ruby-gemset="))
            || context.config_bool("AllowRBSInlineAnnotation", false)
                && content.starts_with(['[', ':', '|'])
            || context.config_bool("AllowSteepAnnotation", false)
                && content.starts_with(['$', ':'])
        {
            continue;
        }
        context.insert(
            "Missing space after `#`.",
            location.start_offset()..location.end_offset(),
            location.start_offset() + 1,
            " ",
        );
    }
}

fn array_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '[', ']');
}

fn hash_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '{', '}');
}

fn method_call_brace_layout(context: &mut CopContext<'_, '_>) {
    brace_layout(context, '(', ')');
}

fn method_definition_brace_layout(context: &mut CopContext<'_, '_>) {
    if context
        .source()
        .lines()
        .next()
        .is_some_and(|line| line.trim_start().starts_with("def "))
    {
        brace_layout(context, '(', ')');
    }
}

fn brace_layout(context: &mut CopContext<'_, '_>, open: char, close: char) {
    let style = context.policy().enforced_style("symmetrical").to_string();
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if !window[0].1.trim_end().ends_with(open) || window[1].1.trim() == close.to_string() {
            continue;
        }
        if matches!(style.as_str(), "symmetrical" | "new_line")
            && lines.iter().any(|(_, line)| {
                line.trim_end().ends_with(close) && line.trim() != close.to_string()
            })
        {
            context.report(
                "Closing brace must be on a new line.",
                window[0].0..window[1].0 + window[1].1.len(),
            );
        }
    }
}

fn align_continuation(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if window[0].1.trim_end().ends_with([',', '+', '\\'])
            && !window[1].1.trim().is_empty()
            && window[1].1.len() - window[1].1.trim_start().len() == 0
        {
            context.insert(
                "Use configured indentation for a continuation line.",
                window[1].0..window[1].0,
                window[1].0,
                "  ",
            );
        }
    }
}

fn line_continuation_spacing(context: &mut CopContext<'_, '_>) {
    let trimmed_source = context.source().trim_start();
    if ["%i", "%I", "%q", "%Q", "%r", "%x", "%W", "%w", "/", "`"]
            .iter()
            .any(|prefix| trimmed_source.starts_with(prefix))
    {
        return;
    }
    let space_style = context.policy().enforced_style("space") == "space";
    let literal_ranges = context.source_file().literal_ranges();
    for (offset, line) in context.source_file().lines() {
        if line.trim() == "__END__" {
            break;
        }
        if !line.trim_end().ends_with('\\') || line.trim_start().starts_with('#') {
            continue;
        }
        let slash = line.rfind('\\').unwrap_or(0);
        if literal_ranges
            .iter()
            .any(|range| range.start <= offset + slash && offset + slash < range.end)
        {
            continue;
        }
        if !context
            .source_file()
            .code_offsets("\\")
            .contains(&(offset + slash))
        {
            continue;
        }
        let spaces = line[..slash].len() - line[..slash].trim_end().len();
        if space_style && spaces != 1 {
            context.replace(
                "Use one space in front of backslash.",
                offset + slash - spaces..offset + slash + 1,
                offset + slash - spaces..offset + slash,
                " ",
            );
        } else if !space_style && spaces > 0 {
            context.remove(
                "Use zero spaces in front of backslash.",
                offset + slash - spaces..offset + slash + 1,
                offset + slash - spaces..offset + slash,
            );
        }
    }
}

fn space_inside_parens(context: &mut CopContext<'_, '_>) {
    let style = context.policy().enforced_style("no_space");
    let no_space = style == "no_space";
    let compact = style == "compact";
    let source = context.source();
    let file = context.source_file();
    for opening in file.code_offsets("(") {
        let whitespace_end = opening
            + 1
            + source[opening + 1..]
                .bytes()
                .take_while(|byte| matches!(byte, b' ' | b'\t'))
                .count();
        let next = source.as_bytes().get(whitespace_end).copied();
        if source.as_bytes().get(opening + 1) == Some(&b'\n') || next == Some(b'#') {
            continue;
        }
        let whitespace = opening + 1..whitespace_end;
        let unwanted = !whitespace.is_empty() && (no_space || next == Some(b')') || compact && next == Some(b'('));
        if unwanted {
            context.remove(
                "Space inside parentheses detected.",
                whitespace.clone(),
                whitespace,
            );
        } else if !no_space
            && whitespace.is_empty()
            && !matches!(next, Some(b')'))
            && !(compact && next == Some(b'('))
        {
            context.insert(
                "No space inside parentheses detected.",
                opening + 1..opening + 2,
                opening + 1,
                " ",
            );
        }
    }
    for closing in file.code_offsets(")") {
        let line_start = file.line_start(closing);
        let whitespace_start = source[line_start..closing].trim_end_matches([' ', '\t']).len() + line_start;
        let previous = source.as_bytes().get(whitespace_start.wrapping_sub(1)).copied();
        if whitespace_start == line_start
            || source.as_bytes().get(closing.wrapping_sub(1)) == Some(&b'\n')
            || previous == Some(b'(')
        {
            continue;
        }
        let whitespace = whitespace_start..closing;
        let unwanted = !whitespace.is_empty() && (no_space || compact && previous == Some(b')'));
        if unwanted {
            context.remove(
                "Space inside parentheses detected.",
                whitespace.clone(),
                whitespace,
            );
        } else if !no_space
            && whitespace.is_empty()
            && !(compact && previous == Some(b')'))
        {
            context.insert(
                "No space inside parentheses detected.",
                closing..closing + 1,
                closing,
                " ",
            );
        }
    }
}

fn space_inside_block(context: &mut CopContext<'_, '_>) {
    let no_space = context.policy().enforced_style("space") == "no_space";
    if !no_space {
        return;
    }
    for (offset, line) in context.source_file().lines() {
        if let Some(at) = line.find("{ ") {
            if !line.contains('}')
                || (line[at + 2..].starts_with('|')
                    && context.config_bool("SpaceBeforeBlockParameters", true))
            {
                continue;
            }
            context.remove(
                "Space inside block braces detected.",
                offset + at + 1..offset + at + 2,
                offset + at + 1..offset + at + 2,
            );
        }
    }
}

fn multiline_assignment(context: &mut CopContext<'_, '_>) {
    if context.policy().enforced_style("same_line") != "same_line" {
        return;
    }
    for start in context.source_file().code_offsets(" =\n") {
        context.report(
            "Right hand side of a multi-line assignment must be on the same line.",
            start..start + 2,
        );
    }
}

fn closing_parenthesis(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if line.starts_with(' ') && line.trim() == ")" && line.len() == 2 {
            let spaces = line.len() - 1;
            context.replace(
                "Indent `)` to match the opening parenthesis.",
                offset..offset + spaces,
                offset..offset + spaces,
                " ".repeat(spaces.saturating_sub(1)),
            );
        }
    }
}

fn indentation_style(context: &mut CopContext<'_, '_>) {
    if context.source().contains("<<") {
        return;
    }
    let spaces_style = context.policy().enforced_style("spaces") == "spaces";
    let width = context.config_usize("IndentationWidth", 2);
    for (offset, line) in context.source_file().lines() {
        if line.trim() == "__END__" {
            break;
        }
        let indentation = line.len() - line.trim_start_matches([' ', '\t']).len();
        let trimmed = &line[indentation..];
        if indentation == 0
            || trimmed.is_empty()
            || !context
                .source_file()
                .code_offsets(trimmed)
                .contains(&(offset + indentation))
        {
            continue;
        }
        let leading = &line[..indentation];
        if spaces_style && leading.contains('\t') {
            let offense_end = leading.rfind('\t').map_or(indentation, |tab| tab + 1);
            context.replace(
                "Tab detected in indentation.",
                offset..offset + offense_end,
                offset..offset + indentation,
                leading.replace('\t', &" ".repeat(width)),
            );
        } else if !spaces_style && leading.contains(' ') {
            let spaces = leading.bytes().take_while(|byte| *byte == b' ').count();
            let offense_end = if spaces > 0 { spaces } else { indentation };
            context.replace(
                "Space detected in indentation.",
                offset..offset + offense_end,
                offset..offset + indentation,
                "\t".repeat(indentation / width),
            );
        }
    }
}

fn comment_indentation(context: &mut CopContext<'_, '_>) {
    if context.config_bool("AllowForAlignment", false) {
        return;
    }
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(3) {
        if window[1].1.trim_start().starts_with('#')
            && !window[1].1.contains("rubocop:")
            && !window[0].1.trim_start().starts_with('#')
            && ![
                "if ", "unless ", "elsif ", "else", "case ", "when ", "in ", "rescue", "ensure",
                "begin", "def ",
            ]
            .iter()
            .any(|keyword| window[0].1.trim_start().starts_with(keyword))
            && !window[0].1.trim_end().ends_with(['{', '[', '('])
            && !["elsif ", "else", "when ", "in ", "rescue", "ensure"]
                .iter()
                .any(|keyword| window[2].1.trim_start().starts_with(keyword))
        {
            let expected = window[0].1.len() - window[0].1.trim_start().len();
            let actual = window[1].1.len() - window[1].1.trim_start().len();
            if expected != actual && !window[0].1.trim().is_empty() {
                context.replace(
                    "Incorrect indentation detected.",
                    window[1].0..window[1].0 + actual,
                    window[1].0..window[1].0 + actual,
                    " ".repeat(expected),
                );
            }
        }
    }
}

fn keyword_alignment(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if matches!(line.trim(), "else" | "ensure" | "rescue")
            && line.len() - line.trim_start().len() % 2 == 1
        {
            context.report(
                "Incorrect indentation detected.",
                offset..offset + line.len(),
            );
        }
    }
}

fn access_modifier_indentation(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if matches!(line.trim(), "private" | "protected" | "public")
            && (line.len() - line.trim_start().len()) % 2 == 1
        {
            context.report(
                "Incorrect indentation detected.",
                offset..offset + line.len(),
            );
        }
    }
}

fn case_indentation(context: &mut CopContext<'_, '_>) {
    for (offset, line) in context.source_file().lines() {
        if line.trim_start().starts_with("when ")
            && (line.len() - line.trim_start().len()) % 2 == 1
            && line.len() - line.trim_start().len() < 5
        {
            context.report(
                "Indent `when` as deep as `case`.",
                offset..offset + line.len(),
            );
        }
    }
}

fn empty_after_guard(context: &mut CopContext<'_, '_>) {
    let lines = context.source_file().lines().collect::<Vec<_>>();
    for window in lines.windows(2) {
        if (window[0].1.contains("return if ") || window[0].1.contains("next if "))
            && !window[1].1.trim().is_empty()
            && !window[1].1.trim_start().starts_with('#')
            && !matches!(window[1].1.trim(), "end" | "else" | "ensure" | "rescue")
            && !window[0].1.contains([';', '<'])
            && !window[0].1.contains("and return")
            && !window[1].1.trim_start().starts_with("if ")
        {
            context.insert(
                "Add empty line after guard clause.",
                window[0].0..window[0].0 + window[0].1.len(),
                window[0].0 + window[0].1.len() + 1,
                "\n",
            );
        }
    }
}
