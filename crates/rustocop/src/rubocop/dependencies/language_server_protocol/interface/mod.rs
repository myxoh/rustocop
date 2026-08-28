mod annotated_text_edit;
mod apply_workspace_edit_params;
mod apply_workspace_edit_result;
mod attributes;
mod call_hierarchy_client_capabilities;
mod call_hierarchy_incoming_call;
mod call_hierarchy_incoming_calls_params;
mod call_hierarchy_item;
mod call_hierarchy_options;
mod call_hierarchy_outgoing_call;
mod call_hierarchy_outgoing_calls_params;
mod call_hierarchy_prepare_params;
mod call_hierarchy_registration_options;
mod cancel_params;
mod change_annotation;
mod client_capabilities;
mod code_action;
mod code_action_client_capabilities;
mod code_action_context;
mod code_action_options;
mod code_action_params;
mod code_action_registration_options;
mod code_description;
mod code_lens;
mod code_lens_client_capabilities;
mod code_lens_options;
mod code_lens_params;
mod code_lens_registration_options;
mod code_lens_workspace_client_capabilities;
mod color;
mod color_information;
mod color_presentation;
mod color_presentation_params;
mod command;
mod completion_client_capabilities;
mod completion_context;
mod completion_item;
mod completion_item_label_details;
mod completion_list;
mod completion_options;

pub(crate) use annotated_text_edit::AnnotatedTextEdit;
pub(crate) use apply_workspace_edit_params::ApplyWorkspaceEditParams;
pub(crate) use apply_workspace_edit_result::ApplyWorkspaceEditResult;
pub(crate) use call_hierarchy_client_capabilities::CallHierarchyClientCapabilities;
pub(crate) use call_hierarchy_incoming_call::CallHierarchyIncomingCall;
pub(crate) use call_hierarchy_incoming_calls_params::CallHierarchyIncomingCallsParams;
pub(crate) use call_hierarchy_item::CallHierarchyItem;
pub(crate) use call_hierarchy_options::CallHierarchyOptions;
pub(crate) use call_hierarchy_outgoing_call::CallHierarchyOutgoingCall;
pub(crate) use call_hierarchy_outgoing_calls_params::CallHierarchyOutgoingCallsParams;
pub(crate) use call_hierarchy_prepare_params::CallHierarchyPrepareParams;
pub(crate) use call_hierarchy_registration_options::CallHierarchyRegistrationOptions;
pub(crate) use cancel_params::CancelParams;
pub(crate) use change_annotation::ChangeAnnotation;
pub(crate) use client_capabilities::ClientCapabilities;
pub(crate) use code_action::CodeAction;
pub(crate) use code_action_client_capabilities::CodeActionClientCapabilities;
pub(crate) use code_action_context::CodeActionContext;
pub(crate) use code_action_options::CodeActionOptions;
pub(crate) use code_action_params::CodeActionParams;
pub(crate) use code_action_registration_options::CodeActionRegistrationOptions;
pub(crate) use code_description::CodeDescription;
pub(crate) use code_lens::CodeLens;
pub(crate) use code_lens_client_capabilities::CodeLensClientCapabilities;
pub(crate) use code_lens_options::CodeLensOptions;
pub(crate) use code_lens_params::CodeLensParams;
pub(crate) use code_lens_registration_options::CodeLensRegistrationOptions;
pub(crate) use code_lens_workspace_client_capabilities::CodeLensWorkspaceClientCapabilities;
pub(crate) use color::Color;
pub(crate) use color_information::ColorInformation;
pub(crate) use color_presentation::ColorPresentation;
pub(crate) use color_presentation_params::ColorPresentationParams;
pub(crate) use command::Command;
pub(crate) use completion_client_capabilities::CompletionClientCapabilities;
pub(crate) use completion_context::CompletionContext;
pub(crate) use completion_item::CompletionItem;
pub(crate) use completion_item_label_details::CompletionItemLabelDetails;
pub(crate) use completion_list::CompletionList;
pub(crate) use completion_options::CompletionOptions;

#[cfg(test)]
mod annotated_text_edit_spec;
#[cfg(test)]
mod apply_workspace_edit_params_spec;
#[cfg(test)]
mod apply_workspace_edit_result_spec;
#[cfg(test)]
mod call_hierarchy_client_capabilities_spec;
#[cfg(test)]
mod call_hierarchy_incoming_call_spec;
#[cfg(test)]
mod call_hierarchy_incoming_calls_params_spec;
#[cfg(test)]
mod call_hierarchy_item_spec;
#[cfg(test)]
mod call_hierarchy_options_spec;
#[cfg(test)]
mod call_hierarchy_outgoing_call_spec;
#[cfg(test)]
mod call_hierarchy_outgoing_calls_params_spec;
#[cfg(test)]
mod call_hierarchy_prepare_params_spec;
#[cfg(test)]
mod call_hierarchy_registration_options_spec;
#[cfg(test)]
mod cancel_params_spec;
#[cfg(test)]
mod change_annotation_spec;
#[cfg(test)]
mod client_capabilities_spec;
#[cfg(test)]
mod code_action_client_capabilities_spec;
#[cfg(test)]
mod code_action_context_spec;
#[cfg(test)]
mod code_action_options_spec;
#[cfg(test)]
mod code_action_params_spec;
#[cfg(test)]
mod code_action_registration_options_spec;
#[cfg(test)]
mod code_action_spec;
#[cfg(test)]
mod code_description_spec;
#[cfg(test)]
mod code_lens_client_capabilities_spec;
#[cfg(test)]
mod code_lens_options_spec;
#[cfg(test)]
mod code_lens_params_spec;
#[cfg(test)]
mod code_lens_registration_options_spec;
#[cfg(test)]
mod code_lens_spec;
#[cfg(test)]
mod code_lens_workspace_client_capabilities_spec;
#[cfg(test)]
mod color_information_spec;
#[cfg(test)]
mod color_presentation_params_spec;
#[cfg(test)]
mod color_presentation_spec;
#[cfg(test)]
mod color_spec;
#[cfg(test)]
mod command_spec;
#[cfg(test)]
mod completion_client_capabilities_spec;
#[cfg(test)]
mod completion_context_spec;
#[cfg(test)]
mod completion_item_label_details_spec;
#[cfg(test)]
mod completion_item_spec;
#[cfg(test)]
mod completion_list_spec;
#[cfg(test)]
mod completion_options_spec;
