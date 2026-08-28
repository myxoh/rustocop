mod code_action_kind;
mod code_action_trigger_kind;
mod completion_item_kind;
mod completion_item_tag;
mod completion_trigger_kind;
mod diagnostic_severity;
mod diagnostic_tag;
mod document_diagnostic_report_kind;
mod document_highlight_kind;
mod error_codes;
mod failure_handling_kind;
mod file_change_type;
mod file_operation_pattern_kind;
mod folding_range_kind;
mod initialize_error_codes;
mod inlay_hint_kind;
mod insert_text_format;
mod insert_text_mode;
mod markup_kind;
mod message_type;
mod moniker_kind;
mod notebook_cell_kind;
mod position_encoding_kind;
mod prepare_support_default_behavior;
mod resource_operation_kind;
mod semantic_token_modifiers;
mod semantic_token_types;
mod signature_help_trigger_kind;
mod symbol_kind;
mod symbol_tag;
mod text_document_save_reason;
mod text_document_sync_kind;
mod token_format;
mod uniqueness_level;
mod watch_kind;

#[allow(non_snake_case)]
pub(crate) mod CodeActionKind {
    pub(crate) use super::code_action_kind::*;
}

#[allow(non_snake_case)]
pub(crate) mod CodeActionTriggerKind {
    pub(crate) use super::code_action_trigger_kind::{AUTOMATIC, INVOKED};
}

#[allow(non_snake_case)]
pub(crate) mod CompletionItemKind {
    pub(crate) use super::completion_item_kind::*;
}

#[allow(non_snake_case)]
pub(crate) mod CompletionItemTag {
    pub(crate) use super::completion_item_tag::DEPRECATED;
}

#[allow(non_snake_case)]
pub(crate) mod CompletionTriggerKind {
    pub(crate) use super::completion_trigger_kind::{
        INVOKED, TRIGGER_CHARACTER, TRIGGER_FOR_INCOMPLETE_COMPLETIONS,
    };
}

#[allow(non_snake_case)]
pub(crate) mod DiagnosticSeverity {
    pub(crate) use super::diagnostic_severity::{ERROR, HINT, INFORMATION, WARNING};
}

#[allow(non_snake_case)]
pub(crate) mod DiagnosticTag {
    pub(crate) use super::diagnostic_tag::{DEPRECATED, UNNECESSARY};
}

#[allow(non_snake_case)]
pub(crate) mod DocumentDiagnosticReportKind {
    pub(crate) use super::document_diagnostic_report_kind::{FULL, UNCHANGED};
}

#[allow(non_snake_case)]
pub(crate) mod DocumentHighlightKind {
    pub(crate) use super::document_highlight_kind::{READ, TEXT, WRITE};
}

#[allow(non_snake_case)]
pub(crate) mod ErrorCodes {
    pub(crate) use super::error_codes::*;
}

#[allow(non_snake_case)]
pub(crate) mod FailureHandlingKind {
    pub(crate) use super::failure_handling_kind::{
        ABORT, TEXT_ONLY_TRANSACTIONAL, TRANSACTIONAL, UNDO,
    };
}

#[allow(non_snake_case)]
pub(crate) mod FileChangeType {
    pub(crate) use super::file_change_type::{CHANGED, CREATED, DELETED};
}

#[allow(non_snake_case)]
pub(crate) mod FileOperationPatternKind {
    pub(crate) use super::file_operation_pattern_kind::{FILE, FOLDER};
}

#[allow(non_snake_case)]
pub(crate) mod FoldingRangeKind {
    pub(crate) use super::folding_range_kind::{COMMENT, IMPORTS, REGION};
}

#[allow(non_snake_case)]
pub(crate) mod InitializeErrorCodes {
    pub(crate) use super::initialize_error_codes::UNKNOWN_PROTOCOL_VERSION;
}

#[allow(non_snake_case)]
pub(crate) mod InlayHintKind {
    pub(crate) use super::inlay_hint_kind::{PARAMETER, TYPE};
}

#[allow(non_snake_case)]
pub(crate) mod InsertTextFormat {
    pub(crate) use super::insert_text_format::{PLAIN_TEXT, SNIPPET};
}

#[allow(non_snake_case)]
pub(crate) mod InsertTextMode {
    pub(crate) use super::insert_text_mode::{ADJUST_INDENTATION, AS_IS};
}

#[allow(non_snake_case)]
pub(crate) mod MarkupKind {
    pub(crate) use super::markup_kind::{MARKDOWN, PLAIN_TEXT};
}

#[allow(non_snake_case)]
pub(crate) mod MessageType {
    pub(crate) use super::message_type::{ERROR, INFO, LOG, WARNING};
}

#[allow(non_snake_case)]
pub(crate) mod MonikerKind {
    pub(crate) use super::moniker_kind::{EXPORT, IMPORT, LOCAL};
}

#[allow(non_snake_case)]
pub(crate) mod NotebookCellKind {
    pub(crate) use super::notebook_cell_kind::{CODE, MARKUP};
}

#[allow(non_snake_case)]
pub(crate) mod PositionEncodingKind {
    pub(crate) use super::position_encoding_kind::{UTF8, UTF16, UTF32};
}

#[allow(non_snake_case)]
pub(crate) mod PrepareSupportDefaultBehavior {
    pub(crate) use super::prepare_support_default_behavior::IDENTIFIER;
}

#[allow(non_snake_case)]
pub(crate) mod ResourceOperationKind {
    pub(crate) use super::resource_operation_kind::{CREATE, DELETE, RENAME};
}

#[allow(non_snake_case)]
pub(crate) mod SemanticTokenModifiers {
    pub(crate) use super::semantic_token_modifiers::*;
}

#[allow(non_snake_case)]
pub(crate) mod SemanticTokenTypes {
    pub(crate) use super::semantic_token_types::*;
}

#[allow(non_snake_case)]
pub(crate) mod SignatureHelpTriggerKind {
    pub(crate) use super::signature_help_trigger_kind::{
        CONTENT_CHANGE, INVOKED, TRIGGER_CHARACTER,
    };
}

#[allow(non_snake_case)]
pub(crate) mod SymbolKind {
    pub(crate) use super::symbol_kind::*;
}

#[allow(non_snake_case)]
pub(crate) mod SymbolTag {
    pub(crate) use super::symbol_tag::DEPRECATED;
}

#[allow(non_snake_case)]
pub(crate) mod TextDocumentSaveReason {
    pub(crate) use super::text_document_save_reason::{AFTER_DELAY, FOCUS_OUT, MANUAL};
}

#[allow(non_snake_case)]
pub(crate) mod TextDocumentSyncKind {
    pub(crate) use super::text_document_sync_kind::{FULL, INCREMENTAL, NONE};
}

#[allow(non_snake_case)]
pub(crate) mod TokenFormat {
    pub(crate) use super::token_format::RELATIVE;
}

#[allow(non_snake_case)]
pub(crate) mod UniquenessLevel {
    pub(crate) use super::uniqueness_level::{DOCUMENT, GLOBAL, GROUP, PROJECT, SCHEME};
}

#[allow(non_snake_case)]
pub(crate) mod WatchKind {
    pub(crate) use super::watch_kind::{CHANGE, CREATE, DELETE};
}

#[cfg(test)]
mod code_action_kind_spec;

#[cfg(test)]
mod code_action_trigger_kind_spec;

#[cfg(test)]
mod completion_item_kind_spec;

#[cfg(test)]
mod completion_item_tag_spec;

#[cfg(test)]
mod completion_trigger_kind_spec;

#[cfg(test)]
mod diagnostic_severity_spec;

#[cfg(test)]
mod diagnostic_tag_spec;

#[cfg(test)]
mod document_diagnostic_report_kind_spec;

#[cfg(test)]
mod document_highlight_kind_spec;

#[cfg(test)]
mod error_codes_spec;

#[cfg(test)]
mod failure_handling_kind_spec;

#[cfg(test)]
mod file_change_type_spec;

#[cfg(test)]
mod file_operation_pattern_kind_spec;

#[cfg(test)]
mod folding_range_kind_spec;

#[cfg(test)]
mod initialize_error_codes_spec;

#[cfg(test)]
mod inlay_hint_kind_spec;

#[cfg(test)]
mod insert_text_format_spec;

#[cfg(test)]
mod insert_text_mode_spec;

#[cfg(test)]
mod markup_kind_spec;

#[cfg(test)]
mod message_type_spec;

#[cfg(test)]
mod moniker_kind_spec;

#[cfg(test)]
mod notebook_cell_kind_spec;

#[cfg(test)]
mod position_encoding_kind_spec;

#[cfg(test)]
mod prepare_support_default_behavior_spec;

#[cfg(test)]
mod resource_operation_kind_spec;
#[cfg(test)]
mod semantic_token_modifiers_spec;
#[cfg(test)]
mod semantic_token_types_spec;
#[cfg(test)]
mod signature_help_trigger_kind_spec;
#[cfg(test)]
mod symbol_kind_spec;
#[cfg(test)]
mod symbol_tag_spec;
#[cfg(test)]
mod text_document_save_reason_spec;
#[cfg(test)]
mod text_document_sync_kind_spec;
#[cfg(test)]
mod token_format_spec;
#[cfg(test)]
mod uniqueness_level_spec;
#[cfg(test)]
mod watch_kind_spec;
