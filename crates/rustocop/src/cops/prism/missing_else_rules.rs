use ruby_prism::{CaseNode, IfNode, UnlessNode};

use super::*;

define_cops! {
    MissingElse => "Style/MissingElse" => rubocop_callbacks(MissingElseRule, [on_if, on_unless, on_case]),
}

impl MissingElseRule<'_, '_, '_> {
    fn on_if(&mut self, node: &IfNode<'_>) {
        return_if!(self.policy().enforced_style("both") == "case");
        return_if!(node.end_keyword_loc().is_none() || node.subsequent().is_some());
        let end = node.end_keyword_loc().expect("checked end");
        let keyword = node.if_keyword_loc().expect("normal if has keyword");
        let offense = if self.source_file().at(&keyword) == "elsif" {
            let start = node.location().start_offset();
            let mut finish = end.start_offset();
            while finish > start && self.source().as_bytes()[finish - 1].is_ascii_whitespace() {
                finish -= 1;
            }
            if finish > start && self.source().as_bytes()[finish - 1] == b';' {
                finish -= 1;
            }
            start..finish
        } else {
            node.location().start_offset()..node.location().end_offset()
        };
        self.add_missing_else(offense, end, "if");
    }

    fn on_unless(&mut self, node: &UnlessNode<'_>) {
        return_if!(self.policy().enforced_style("both") == "case");
        return_if!(self.related_config_value("Style/UnlessElse", "Enabled") != Some("false"));
        return_if!(node.end_keyword_loc().is_none() || node.else_clause().is_some());
        let location = node.location();
        self.add_missing_else(location.start_offset()..location.end_offset(), node.end_keyword_loc().expect("checked end"), "if");
    }

    fn on_case(&mut self, node: &CaseNode<'_>) {
        return_if!(self.policy().enforced_style("both") == "if" || node.else_clause().is_some());
        let location = node.location();
        self.add_missing_else(location.start_offset()..location.end_offset(), node.end_keyword_loc(), "case");
    }

    fn add_missing_else(&mut self, offense: std::ops::Range<usize>, end: ruby_prism::Location<'_>, kind: &str) {
        let empty_else = self.related_config_value("Style/EmptyElse", "EnforcedStyle").map(str::to_string);
        let (message, correction) = match empty_else.as_deref() {
            Some("empty") => (format!("`{kind}` condition requires an `else`-clause with `nil` in it."), Some("else; nil; ")),
            Some("nil") => (format!("`{kind}` condition requires an empty `else`-clause."), Some("else; ")),
            _ => (format!("`{kind}` condition requires an `else`-clause."), None),
        };
        if let Some(correction) = correction {
            add_offense!(self, offense, message: message, |corrector| {
                corrector.replace(end.start_offset()..end.start_offset(), correction);
            });
        } else {
            self.report(message, offense);
        }
    }
}
