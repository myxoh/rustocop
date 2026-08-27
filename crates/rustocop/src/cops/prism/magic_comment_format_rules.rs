use regex::Regex;

use super::*;

define_compatibility_rule!(MagicCommentFormatRule);

define_cops! {
    MagicCommentFormat => "Style/MagicCommentFormat" => compatibility_source(on_new_investigation),
}

fn on_new_investigation(context: &mut CompatibilityCopContext<'_, '_, '_>) {
    MagicCommentFormatRule::new(context).on_new_investigation();
}

impl MagicCommentFormatRule<'_, '_, '_, '_> {
    fn on_new_investigation(&mut self) {
        let directive = Regex::new(
            r"(?i)(coding|encoding|frozen[-_]string[-_]literal|rbs_inline|shareable[-_]constant[-_]value|typed)\s*:",
        )
        .expect("static magic-comment regex");
        let value = Regex::new(
            r"(?i)(?:coding|encoding|frozen[-_]string[-_]literal|rbs_inline|shareable[-_]constant[-_]value|typed)\s*:\s*([^;\r\n]*)",
        )
        .expect("static magic-comment value regex");
        let style = self.policy().enforced_style("snake_case").to_string();
        let directive_case = self.config_value("DirectiveCapitalization").map(str::to_string);
        let value_case = self.config_value("ValueCapitalization").map(str::to_string);

        for (offset, line) in self.source_file().lines() {
            let trimmed = line.trim_start();
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                break;
            }
            if !trimmed.starts_with('#') || !line.contains(':') {
                continue;
            }
            if !valid_magic_comment(trimmed) {
                continue;
            }
            for captures in directive.captures_iter(line) {
                let Some(matched) = captures.get(1) else { continue };
                let original = matched.as_str();
                let separator_wrong = if style == "kebab_case" {
                    original.contains('_')
                } else {
                    original.contains('-')
                };
                let capitalization_wrong = wrong_capitalization(original, directive_case.as_deref());
                if !separator_wrong && !capitalization_wrong {
                    continue;
                }
                let range = offset + matched.start()..offset + matched.end();
                let replacement = fix_capitalization(
                    &if style == "kebab_case" {
                        original.replace('_', "-")
                    } else {
                        original.replace('-', "_")
                    },
                    directive_case.as_deref(),
                );
                let case_prefix = match directive_case.as_deref() {
                    Some("lowercase") => "lower ",
                    Some("uppercase") => "upper ",
                    _ => "",
                };
                let shape = if style == "kebab_case" { "kebab" } else { "snake" };
                add_offense!(self, range.clone(), message: format!("Prefer {case_prefix}{shape} case for magic comments."), |corrector| {
                    corrector.replace(range, replacement);
                });
            }
            if value_case.is_none() {
                continue;
            }
            for captures in value.captures_iter(line) {
                let Some(matched) = captures.get(1) else { continue };
                let original = matched.as_str();
                if !wrong_capitalization(original, value_case.as_deref()) {
                    continue;
                }
                let range = offset + matched.start()..offset + matched.end();
                let replacement = fix_capitalization(original, value_case.as_deref());
                add_offense!(self, range.clone(), message: format!("Prefer {} for magic comment values.", value_case.as_deref().unwrap_or_default()), |corrector| {
                    corrector.replace(range, replacement);
                });
            }
        }
    }
}

fn valid_magic_comment(comment: &str) -> bool {
    let simple = Regex::new(
        r"(?ix)^\#\s*(?:
            (?:en)?coding:\s+[[:alnum:]_-]+ |
            (?:frozen[-_]string[-_]literal|rbs_inline|shareable[-_]constant[-_]value|typed):\s*[[:alnum:]_-]+\s*$
        )",
    )
    .expect("static simple magic-comment regex");
    if simple.is_match(comment) {
        return true;
    }
    let editor_token = Regex::new(
        r"(?i)(?:coding|encoding|frozen[-_]string[-_]literal|shareable[-_]constant[-_]value|typed)\s*[:=]\s*[[:alnum:]_-]+",
    )
    .expect("static editor magic-comment regex");
    (comment.contains("-*-")) && editor_token.is_match(comment)
        || comment.to_ascii_lowercase().starts_with("# vim:")
            && editor_token.is_match(comment)
}

fn wrong_capitalization(source: &str, expected: Option<&str>) -> bool {
    match expected {
        Some("lowercase") => source != source.to_ascii_lowercase(),
        Some("uppercase") => source != source.to_ascii_uppercase(),
        _ => false,
    }
}

fn fix_capitalization(source: &str, expected: Option<&str>) -> String {
    match expected {
        Some("lowercase") => source.to_ascii_lowercase(),
        Some("uppercase") => source.to_ascii_uppercase(),
        _ => source.to_string(),
    }
}
