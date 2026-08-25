// RuboCop 1.87.0
// Source: lib/rubocop/cop/correctors/punctuation_corrector.rb
// Source SHA-256: 18b0b332f1ad8a65770c2e2f1e3c1a941bb5d40a07976110cb0431e9d7342450
// Source: lib/rubocop/cop/correctors/empty_line_corrector.rb
// Source SHA-256: 2bfaceb4403fec7563cbf4acfbc0405ab4b2466b99f7e11383355184b59dbca7
// Source: lib/rubocop/cop/correctors/string_literal_corrector.rb
// Source SHA-256: a52570a6155dfd4a704d7ccfee0454887cf08b3ee792d2d6e28c8a48510cff95
// Source: lib/rubocop/cop/correctors/unused_arg_corrector.rb
// Source SHA-256: 2a52b6887c63733fe34efd4cd6d858ab58ba4eec6a39b29bbf207aeaaf0ef5c4
// Source: lib/rubocop/cop/correctors/require_library_corrector.rb
// Source SHA-256: 6c4563d95ea2562e55f27ab30ba897d5d3846b04bfba0ee4c5055eccf48fbb4e
// Source: lib/rubocop/cop/correctors/condition_corrector.rb
// Source SHA-256: e971828765783126d58632a1d79e7b7d8b1d09c82a659b4bb8a860d3455d8b9b

use crate::rubocop::ast::node::core::NodeRef;
use crate::rubocop::ast::processed_source::ProcessedSource;
use crate::rubocop::ast::source::SourceRange;

use super::corrector::Corrector;

pub(crate) mod lambda_literal_to_method_corrector;
pub(crate) mod multiline_literal_brace_corrector;
pub(crate) mod parentheses_corrector;
pub(crate) mod percent_literal_corrector;

pub(crate) struct PunctuationCorrector;

impl PunctuationCorrector {
    pub(crate) fn remove_space<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        space_before: SourceRange<'buffer, 'source>,
    ) {
        corrector.remove(space_before);
    }

    pub(crate) fn add_space<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        token: SourceRange<'buffer, 'source>,
    ) {
        corrector.replace(token, format!("{} ", token.source()));
    }

    pub(crate) fn swap_comma<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        range: Option<SourceRange<'buffer, 'source>>,
    ) {
        let Some(range) = range else { return };
        if range.source() == "," {
            corrector.remove(range);
        } else {
            corrector.insert_after(range, ",");
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EmptyLineStyle {
    NoEmptyLines,
    EmptyLines,
}

pub(crate) struct EmptyLineCorrector;

impl EmptyLineCorrector {
    pub(crate) fn correct<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        offense_style: EmptyLineStyle,
        range: SourceRange<'buffer, 'source>,
    ) {
        match offense_style {
            EmptyLineStyle::NoEmptyLines => corrector.remove(range),
            EmptyLineStyle::EmptyLines => corrector.insert_before(range, "\n"),
        }
    }

    pub(crate) fn insert_before<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        node: SourceRange<'buffer, 'source>,
    ) {
        corrector.insert_before(node, "\n");
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum StringStyle {
    SingleQuotes,
    DoubleQuotes,
}

pub(crate) struct StringLiteralCorrector;

impl StringLiteralCorrector {
    pub(crate) fn correct<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        range: SourceRange<'buffer, 'source>,
        content: &str,
        interpolated: bool,
        style: StringStyle,
    ) {
        if interpolated {
            return;
        }
        let literal = match style {
            StringStyle::SingleQuotes => to_single_quoted_literal(content),
            StringStyle::DoubleQuotes => ruby_inspect(content),
        };
        corrector.replace(range, literal);
    }
}

fn to_single_quoted_literal(content: &str) -> String {
    if content.contains('\'')
        || content.chars().any(|character| character.is_control())
        || has_unpaired_backslash(content)
    {
        return ruby_inspect(content);
    }
    format!("'{}'", content.replace('\\', "\\\\"))
}

fn has_unpaired_backslash(content: &str) -> bool {
    let mut slash_run = 0;
    for character in content.chars().chain(['\0']) {
        if character == '\\' {
            slash_run += 1;
        } else {
            if slash_run % 2 == 1 && character != '"' {
                return true;
            }
            slash_run = 0;
        }
    }
    false
}

fn ruby_inspect(content: &str) -> String {
    let mut output = String::from("\"");
    for character in content.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                output.push_str(&format!("\\u{{{:x}}}", u32::from(character)));
            }
            character => output.push(character),
        }
    }
    output.push('"');
    output
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArgumentKind {
    Keyword,
    OptionalKeyword,
    Block,
    Optional,
    Positional,
}

pub(crate) struct UnusedArgCorrector;

impl UnusedArgCorrector {
    pub(crate) fn processed_source<'source>(
        processed_source: &'source ProcessedSource<'source>,
    ) -> &'source ProcessedSource<'source> {
        processed_source
    }

    pub(crate) fn correct<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        kind: ArgumentKind,
        source_range: SourceRange<'buffer, 'source>,
        name_range: SourceRange<'buffer, 'source>,
        optional_name: Option<&str>,
    ) {
        if matches!(kind, ArgumentKind::Keyword | ArgumentKind::OptionalKeyword) {
            return;
        }
        if kind == ArgumentKind::Block {
            let buffer = source_range.buffer();
            let mut begin = source_range.begin_pos();
            while begin > 0 && buffer.character(begin - 1).is_some_and(char::is_whitespace) {
                begin -= 1;
            }
            if begin > 0 && buffer.character(begin - 1) == Some(',') {
                begin -= 1;
            }
            corrector.remove(SourceRange::new(buffer, begin, source_range.end_pos()));
            return;
        }
        let variable_name =
            optional_name.unwrap_or_else(|| source_range.source().trim_start_matches('*'));
        corrector.replace(name_range, format!("_{variable_name}"));
    }

    pub(crate) fn correct_for_blockarg_type<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        source_range: SourceRange<'buffer, 'source>,
    ) {
        let buffer = source_range.buffer();
        let mut begin = source_range.begin_pos();
        while begin > 0 && buffer.character(begin - 1).is_some_and(char::is_whitespace) {
            begin -= 1;
        }
        if begin > 0 && buffer.character(begin - 1) == Some(',') {
            begin -= 1;
        }
        corrector.remove(SourceRange::new(buffer, begin, source_range.end_pos()));
    }
}

pub(crate) struct RequireLibraryCorrector;

impl RequireLibraryCorrector {
    pub(crate) fn correct<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        root_first_expression: SourceRange<'buffer, 'source>,
        library_name: &str,
    ) {
        corrector.insert_before(root_first_expression, Self::require_statement(library_name));
    }

    pub(crate) fn require_statement(library_name: &str) -> String {
        format!("require '{library_name}'\n")
    }
}

pub(crate) struct ConditionCorrector;

impl ConditionCorrector {
    pub(crate) fn negated_condition(mut node: NodeRef<'_>) -> Option<NodeRef<'_>> {
        node = node.condition()?;
        while node.kind() == "begin" {
            node = node.child_nodes().last().copied()?;
        }
        Some(node)
    }

    pub(crate) fn correct_negative_condition<'buffer, 'source>(
        corrector: &mut Corrector<'buffer, 'source>,
        keyword: SourceRange<'buffer, 'source>,
        inverse_keyword: &str,
        negated_condition: SourceRange<'buffer, 'source>,
        inner_condition_source: &str,
    ) {
        corrector.replace(keyword, inverse_keyword);
        corrector.replace(negated_condition, inner_condition_source);
    }
}
// RuboCop API ownership: lib/rubocop/cop/correctors/empty_line_corrector.rb => correct
// RuboCop API ownership: lib/rubocop/cop/correctors/require_library_corrector.rb => correct
// RuboCop API ownership: lib/rubocop/cop/correctors/string_literal_corrector.rb => correct
// RuboCop API ownership: lib/rubocop/cop/correctors/unused_arg_corrector.rb => correct, processed_source
