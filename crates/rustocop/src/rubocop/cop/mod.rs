pub(crate) mod advanced_correctors;
pub(crate) mod annotation_comment;
pub(crate) mod badge;
pub(crate) mod commissioner;
pub(crate) mod corrector;
pub(crate) mod correctors;
pub(crate) mod documentation;
pub(crate) mod exclude_limit;
pub(crate) mod force;
pub(crate) mod framework;
pub(crate) mod generator;
pub(crate) mod ignored_node;
pub(crate) mod legacy;
pub(crate) mod message_annotator;
pub(crate) mod mixin;
pub(crate) mod offense;
pub(crate) mod registry;
pub(crate) mod severity;
pub(crate) mod team;
pub(crate) mod variable_force;

#[cfg(test)]
mod advanced_correctors_spec;
#[cfg(test)]
mod alignment_corrector_spec;
#[cfg(test)]
mod annotation_comment_spec;
#[cfg(test)]
mod badge_spec;
#[cfg(test)]
mod commissioner_spec;
#[cfg(test)]
mod corrector_spec;
#[cfg(test)]
mod correctors_spec;
#[cfg(test)]
mod documentation_spec;
#[cfg(test)]
mod exclude_limit_spec;
#[cfg(test)]
mod force_spec;
#[cfg(test)]
mod framework_spec;
#[cfg(test)]
mod ignored_node_spec;
#[cfg(test)]
mod legacy_spec;
#[cfg(test)]
mod message_annotator_spec;
#[cfg(test)]
mod offense_spec;
#[cfg(test)]
mod registry_spec;
#[cfg(test)]
mod severity_spec;
#[cfg(test)]
mod util_spec;
#[cfg(test)]
mod variable_force_spec;
