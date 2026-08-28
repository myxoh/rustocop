#[derive(Debug)]
pub(crate) struct Offense {
    pub(crate) cop_name: String,
    pub(crate) severity: String,
    pub(crate) message: String,
    pub(crate) message_bytes: Option<Vec<u8>>,
    pub(crate) corrected: bool,
    pub(crate) correctable: bool,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) last_line: usize,
    pub(crate) last_column: usize,
    pub(crate) length: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CorrectionStatus {
    Unavailable,
    Pending,
    Applied,
}

impl CorrectionStatus {
    pub(crate) const fn correctable(corrected: bool) -> Self {
        if corrected {
            Self::Applied
        } else {
            Self::Pending
        }
    }

    pub(crate) const fn from_flags(correctable: bool, corrected: bool) -> Self {
        if correctable {
            Self::correctable(corrected)
        } else {
            Self::Unavailable
        }
    }

    const fn flags(self) -> (bool, bool) {
        match self {
            Self::Unavailable => (false, false),
            Self::Pending => (true, false),
            Self::Applied => (true, true),
        }
    }
}

pub(crate) fn push_offense(
    offenses: &mut Vec<Offense>,
    cop_name: &'static str,
    message: &str,
    line: usize,
    column: usize,
    length: usize,
    correction: CorrectionStatus,
) {
    let (correctable, corrected) = correction.flags();
    offenses.push(Offense {
        cop_name: cop_name.to_string(),
        severity: default_severity(cop_name).to_string(),
        message: message.to_string(),
        message_bytes: None,
        corrected,
        correctable,
        line,
        column: column.max(1),
        last_line: line,
        last_column: column.max(1) + length.max(1) - 1,
        length: length.max(1),
    });
}

pub(crate) fn default_severity(cop_name: &str) -> &'static str {
    if cop_name == "Lint/Syntax" {
        "fatal"
    } else if cop_name.starts_with("Lint/")
        || matches!(
            cop_name,
            "Bundler/DuplicatedGem"
                | "Bundler/DuplicatedGroup"
                | "Gemspec/RequireMFA"
                | "Bundler/InsecureProtocolSource"
                | "Gemspec/RubyVersionGlobalsUsage"
                | "Gemspec/DuplicatedAssignment"
                | "Gemspec/DeprecatedAttributeAssignment"
                | "Gemspec/RequiredRubyVersion"
                | "Layout/BeginEndAlignment"
                | "Layout/DefEndAlignment"
                | "Layout/EndAlignment"
        )
    {
        "warning"
    } else {
        "convention"
    }
}

pub(crate) fn effective_severity(cop_name: &str, configured: Option<&str>) -> String {
    // Lint/Syntax overrides Base#find_severity and always reports fatal,
    // including when a configured or caller-provided severity is present.
    if cop_name == "Lint/Syntax" {
        return "fatal".to_string();
    }
    const NAMES: &[&str] = &[
        "info",
        "refactor",
        "convention",
        "warning",
        "error",
        "fatal",
    ];
    match configured {
        Some(severity) if NAMES.contains(&severity) => severity.to_string(),
        Some(severity) => {
            eprintln!(
                "Warning: Invalid severity '{severity}'. Valid severities are {}.",
                NAMES.join(", ")
            );
            base_default_severity(cop_name).to_string()
        }
        None => base_default_severity(cop_name).to_string(),
    }
}

fn base_default_severity(cop_name: &str) -> &'static str {
    if cop_name.starts_with("Lint/") {
        "warning"
    } else {
        "convention"
    }
}

#[derive(Clone, Debug)]
pub(crate) struct SourceLine {
    pub(crate) body: String,
    pub(crate) ending: String,
}

#[cfg(test)]
mod tests {
    use super::{effective_severity, CorrectionStatus};

    #[test]
    fn correction_status_preserves_the_two_output_flags() {
        assert_eq!(CorrectionStatus::Unavailable.flags(), (false, false));
        assert_eq!(CorrectionStatus::Pending.flags(), (true, false));
        assert_eq!(CorrectionStatus::Applied.flags(), (true, true));
        assert_eq!(
            CorrectionStatus::from_flags(true, false),
            CorrectionStatus::Pending
        );
        assert_eq!(
            CorrectionStatus::correctable(true),
            CorrectionStatus::Applied
        );
    }

    #[test]
    fn effective_severity_accepts_rubocop_names_and_rejects_invalid_values() {
        assert_eq!(effective_severity("Style/Example", Some("error")), "error");
        assert_eq!(
            effective_severity("Bundler/DuplicatedGem", Some("invalid")),
            "convention"
        );
        assert_eq!(
            effective_severity("Bundler/DuplicatedGem", None),
            "convention"
        );
        assert_eq!(effective_severity("Lint/Syntax", None), "fatal");
        assert_eq!(effective_severity("Lint/Syntax", Some("error")), "fatal");
    }
}
