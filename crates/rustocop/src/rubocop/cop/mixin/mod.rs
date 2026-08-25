pub(crate) mod advanced;
pub(crate) mod alignment;
pub(crate) mod allowed_methods;
pub(crate) mod allowed_pattern;
pub(crate) mod check_assignment;
pub(crate) mod check_line_breakable;
pub(crate) mod check_single_line_suitability;
pub(crate) mod code_length;
pub(crate) mod comments_help;
pub(crate) mod configurable_formatting;
pub(crate) mod configuration;
pub(crate) mod documentation_comment;
pub(crate) mod empty_lines_around_body;
pub(crate) mod end_keyword_alignment;
pub(crate) mod frozen_string_literal;
pub(crate) mod hash_alignment_styles;
pub(crate) mod hash_shorthand_syntax;
pub(crate) mod hash_subset;
pub(crate) mod hash_transform_method;
pub(crate) mod helpers;
pub(crate) mod heredoc;
pub(crate) mod interpolation;
pub(crate) mod line_length_help;
pub(crate) mod method_complexity;
pub(crate) mod multiline_element_indentation;
pub(crate) mod multiline_expression_indentation;
pub(crate) mod multiline_literal_brace_layout;
pub(crate) mod percent_array;
pub(crate) mod policies;
pub(crate) mod preceding_following_alignment;
pub(crate) mod preferred_delimiters;
pub(crate) mod range_help;
pub(crate) mod space_after_punctuation;
pub(crate) mod statement_modifier;
pub(crate) mod surrounding_space;
pub(crate) mod trailing_comma;
pub(crate) mod uncommunicative_name;
pub(crate) mod unused_argument;

#[cfg(test)]
mod advanced_spec;
#[cfg(test)]
mod alignment_spec;
#[cfg(test)]
mod allowed_methods_spec;
#[cfg(test)]
mod allowed_pattern_spec;
#[cfg(test)]
mod check_assignment_spec;
#[cfg(test)]
mod check_single_line_suitability_spec;
#[cfg(test)]
mod configurable_formatting_spec;
#[cfg(test)]
mod configuration_spec;
#[cfg(test)]
mod documentation_comment_spec;
#[cfg(test)]
mod hash_alignment_styles_spec;
#[cfg(test)]
mod hash_shorthand_syntax_spec;
#[cfg(test)]
mod hash_subset_spec;
#[cfg(test)]
mod hash_transform_method_spec;
#[cfg(test)]
mod helpers_spec;
#[cfg(test)]
mod heredoc_spec;
#[cfg(test)]
mod interpolation_spec;
#[cfg(test)]
mod policies_spec;
#[cfg(test)]
mod preferred_delimiters_spec;
#[cfg(test)]
mod range_help_spec;
#[cfg(test)]
mod space_after_punctuation_spec;
#[cfg(test)]
mod trailing_comma_spec;
#[cfg(test)]
mod uncommunicative_name_spec;
#[cfg(test)]
mod unused_argument_spec;
