// RuboCop 1.87.0
// Source: lib/rubocop/cop/offense.rb
// Source SHA-256: dd34d19e99f1f94adcda2eeb4ff9cf943b91e23a93d437644624f9fa03340339

use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Range;

use crate::rubocop::ast::source::SourceBuffer;
use crate::rubocop::ast::source::SourceRange;

use super::corrector::Corrector;
use super::severity::Severity;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum OffenseStatus {
    Uncorrected,
    Unsupported,
    Corrected,
    CorrectedWithTodo,
    Disabled,
    Todo,
}

#[derive(Clone)]
pub(crate) struct Offense<'buffer, 'source> {
    severity: Severity,
    location: Option<SourceRange<'buffer, 'source>>,
    message: String,
    cop_name: String,
    status: OffenseStatus,
    corrector: Option<Corrector<'buffer, 'source>>,
}

impl<'buffer, 'source> Offense<'buffer, 'source> {
    pub(crate) fn new(
        severity: Severity,
        location: SourceRange<'buffer, 'source>,
        message: impl Into<String>,
        cop_name: impl Into<String>,
        status: OffenseStatus,
        corrector: Option<Corrector<'buffer, 'source>>,
    ) -> Self {
        Self {
            severity,
            location: Some(location),
            message: message.into(),
            cop_name: cop_name.into(),
            status,
            corrector,
        }
    }

    pub(crate) fn no_location(
        severity: Severity,
        message: impl Into<String>,
        cop_name: impl Into<String>,
        status: OffenseStatus,
    ) -> Self {
        Self {
            severity,
            location: None,
            message: message.into(),
            cop_name: cop_name.into(),
            status,
            corrector: None,
        }
    }

    pub(crate) fn severity(&self) -> Severity {
        self.severity
    }

    pub(crate) fn location(&self) -> Option<SourceRange<'buffer, 'source>> {
        self.location
    }

    pub(crate) fn message(&self) -> &str {
        &self.message
    }

    pub(crate) fn cop_name(&self) -> &str {
        &self.cop_name
    }

    pub(crate) fn status(&self) -> OffenseStatus {
        self.status
    }

    pub(crate) fn has_corrector(&self) -> bool {
        self.corrector.is_some()
    }

    pub(crate) fn corrector(&self) -> Option<&Corrector<'buffer, 'source>> {
        self.corrector.as_ref()
    }

    pub(crate) fn source_buffer(&self) -> Option<&SourceBuffer<'source>> {
        self.location.map(SourceRange::buffer)
    }

    pub(crate) fn correctable(&self) -> bool {
        self.status != OffenseStatus::Unsupported
    }

    pub(crate) fn corrected(&self) -> bool {
        matches!(
            self.status,
            OffenseStatus::Corrected | OffenseStatus::CorrectedWithTodo
        )
    }

    pub(crate) fn corrected_with_todo(&self) -> bool {
        self.status == OffenseStatus::CorrectedWithTodo
    }

    pub(crate) fn disabled(&self) -> bool {
        matches!(self.status, OffenseStatus::Disabled | OffenseStatus::Todo)
    }

    pub(crate) fn line(&self) -> usize {
        self.location.map_or(1, SourceRange::line)
    }

    pub(crate) fn column(&self) -> usize {
        self.location.map_or(0, SourceRange::column)
    }

    pub(crate) fn source_line(&self) -> &'source str {
        self.location.map_or("", |location| {
            location.buffer().source_line(location.line())
        })
    }

    pub(crate) fn column_length(&self) -> usize {
        if self.location.is_none() {
            return 0;
        }
        if self.first_line() == self.last_line() {
            self.last_column().saturating_sub(self.column())
        } else {
            self.source_line()
                .chars()
                .count()
                .saturating_sub(self.column())
        }
    }

    pub(crate) fn first_line(&self) -> usize {
        self.line()
    }

    pub(crate) fn last_line(&self) -> usize {
        self.location.map_or(1, SourceRange::last_line)
    }

    pub(crate) fn last_column(&self) -> usize {
        self.location.map_or(0, SourceRange::last_column)
    }

    pub(crate) fn column_range(&self) -> Range<usize> {
        self.column()..self.last_column()
    }

    pub(crate) fn real_column(&self) -> usize {
        self.column() + 1
    }

    pub(crate) fn highlighted_source(&self) -> &'source str {
        let Some(location) = self.location else {
            return "";
        };
        let line_start = location.buffer().line_start(location.line());
        location
            .buffer()
            .slice(line_start + self.column()..line_start + self.column() + self.column_length())
    }

    pub(crate) fn highlighted_area(&self) -> Range<usize> {
        self.column()..self.column() + self.column_length()
    }

    pub(crate) fn size(&self) -> usize {
        self.location
            .map_or(0, |range| range.end_pos().saturating_sub(range.begin_pos()))
    }

    pub(crate) fn begin_pos(&self) -> usize {
        self.location.map_or(0, SourceRange::begin_pos)
    }

    pub(crate) fn end_pos(&self) -> usize {
        self.location.map_or(0, SourceRange::end_pos)
    }

    pub(crate) fn length(&self) -> usize {
        self.size()
    }

    pub(crate) fn equivalent(&self, other: &Self) -> bool {
        self == other
    }

    pub(crate) fn compare(&self, other: &Self) -> Ordering {
        self.cmp(other)
    }

    pub(crate) fn display(&self) -> String {
        self.to_string()
    }

    pub(crate) fn marshal_dump(
        &self,
    ) -> (
        Severity,
        Option<SourceRange<'buffer, 'source>>,
        String,
        String,
        OffenseStatus,
    ) {
        (
            self.severity,
            self.location,
            self.message.clone(),
            self.cop_name.clone(),
            self.status,
        )
    }

    pub(crate) fn marshal_load(
        dump: (
            Severity,
            Option<SourceRange<'buffer, 'source>>,
            String,
            String,
            OffenseStatus,
        ),
    ) -> Self {
        Self {
            severity: dump.0,
            location: dump.1,
            message: dump.2,
            cop_name: dump.3,
            status: dump.4,
            corrector: None,
        }
    }

    fn comparison_tuple(&self) -> (usize, usize, &str, &str, Severity) {
        (
            self.line(),
            self.column(),
            &self.cop_name,
            &self.message,
            self.severity,
        )
    }
}

impl fmt::Display for Offense<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{}:{:>3}:{:>3}: {}",
            self.severity.code(),
            self.line(),
            self.real_column(),
            self.message
        )
    }
}

impl fmt::Debug for Offense<'_, '_> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("Offense")
            .field("severity", &self.severity)
            .field("location", &self.location)
            .field("message", &self.message)
            .field("cop_name", &self.cop_name)
            .field("status", &self.status)
            .field("has_corrector", &self.has_corrector())
            .finish()
    }
}

impl PartialEq for Offense<'_, '_> {
    fn eq(&self, other: &Self) -> bool {
        self.comparison_tuple() == other.comparison_tuple()
    }
}

impl Eq for Offense<'_, '_> {}

impl Hash for Offense<'_, '_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.comparison_tuple().hash(state);
    }
}

impl PartialOrd for Offense<'_, '_> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Offense<'_, '_> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.comparison_tuple().cmp(&other.comparison_tuple())
    }
}
