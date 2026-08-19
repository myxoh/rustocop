#[derive(Debug)]
pub(crate) struct Offense {
    pub(crate) cop_name: String,
    pub(crate) message: String,
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
        message: message.to_string(),
        corrected,
        correctable,
        line,
        column: column.max(1),
        last_line: line,
        last_column: column.max(1) + length.max(1) - 1,
        length: length.max(1),
    });
}

#[derive(Clone, Debug)]
pub(crate) struct SourceLine {
    pub(crate) body: String,
    pub(crate) ending: String,
}

#[cfg(test)]
mod tests {
    use super::CorrectionStatus;

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
}
