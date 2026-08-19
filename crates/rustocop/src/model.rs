#[derive(Debug)]
pub(crate) struct Offense {
    pub(crate) cop_name: &'static str,
    pub(crate) message: String,
    pub(crate) corrected: bool,
    pub(crate) correctable: bool,
    pub(crate) line: usize,
    pub(crate) column: usize,
    pub(crate) last_line: usize,
    pub(crate) last_column: usize,
    pub(crate) length: usize,
}

pub(crate) fn push_offense(
    offenses: &mut Vec<Offense>,
    cop_name: &'static str,
    message: &str,
    line: usize,
    column: usize,
    length: usize,
    correction: (bool, bool),
) {
    let (correctable, corrected) = correction;
    offenses.push(Offense {
        cop_name,
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
