use std::collections::HashSet;
use std::fs;
use std::io;
use std::path::PathBuf;

use crate::config::InspectionConfig;
use crate::cops::{prism, text};
use crate::model::Offense;

mod diagnostic;
#[cfg(test)]
mod fixture_tests;
mod runner;
pub(crate) mod source;
#[cfg(test)]
mod unit_contract_tests;

use diagnostic::{append_prism_offenses, sort_offenses};
pub(crate) use runner::inspect_files;

const MAX_CORRECTION_ITERATIONS: usize = 200;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum CorrectionError {
    InfiniteLoop,
    MaximumIterations,
}

impl CorrectionError {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::InfiniteLoop => "infinite_loop",
            Self::MaximumIterations => "maximum_iterations",
        }
    }
}

pub(crate) struct InspectionPlan {
    prism: prism::Engine,
    text_cops_enabled: bool,
}

impl InspectionPlan {
    pub(crate) fn new(options: &InspectionConfig) -> Self {
        let prism = prism::Engine::new(&|cop| options.cop_enabled(cop));
        let text_cops_enabled = text::LEGACY_COP_NAMES
            .iter()
            .any(|cop| options.cop_enabled(cop) && !prism.implements(cop));
        Self {
            prism,
            text_cops_enabled,
        }
    }

    pub(crate) fn inspect_file(
        &self,
        path: &str,
        options: &InspectionConfig,
    ) -> io::Result<InspectionResult> {
        let original = fs::read(path)?;
        let content = source::DecodedSource::from_bytes(&original)?;
        let absolute_path = expanded_path(path);
        let (offenses, corrected_content, correction_error) =
            self.inspect_content_with_corrections(&absolute_path, content.as_str(), options);
        if let Some(error) = correction_error {
            return Err(io::Error::other(format!(
                "autocorrection failed: {}",
                error.as_str()
            )));
        }
        let corrected_bytes = content.restore(&corrected_content);
        if options.autocorrect_enabled() && corrected_bytes != original {
            fs::write(path, corrected_bytes)?;
        }
        Ok(InspectionResult {
            path: absolute_path,
            offenses,
        })
    }

    pub(crate) fn inspect_content(
        &self,
        path: &str,
        content: &str,
        options: &InspectionConfig,
    ) -> (Vec<Offense>, String) {
        if !self.text_cops_enabled {
            return self.inspect_prism_only(path, content, options);
        }

        let mut lines = source::split(content);
        let original_lines = lines.clone();
        let mut offenses = Vec::new();
        text::before_prism(&mut lines, options, &mut offenses);

        let prism_source = source::join(&lines);
        let prism_inspection = self.prism.inspect(
            path,
            &prism_source,
            options.autocorrect,
            options.target_ruby_version,
            options.cop_config.clone(),
        );
        append_prism_offenses(&mut offenses, &prism_source, prism_inspection.findings);
        text::after_prism(path, &original_lines, options, &mut offenses);
        sort_offenses(&mut offenses);
        (offenses, prism_inspection.corrected_source)
    }

    pub(crate) fn inspect_content_with_corrections(
        &self,
        path: &str,
        content: &str,
        options: &InspectionConfig,
    ) -> (Vec<Offense>, String, Option<CorrectionError>) {
        let (offenses, mut corrected) = self.inspect_content(path, content, options);
        if !options.autocorrect_enabled() || corrected == content {
            return (offenses, corrected, None);
        }

        let mut seen = HashSet::from([content.to_string()]);
        for _iteration in 1..MAX_CORRECTION_ITERATIONS {
            if !seen.insert(corrected.clone()) {
                return (offenses, corrected, Some(CorrectionError::InfiniteLoop));
            }
            let (_, next) = self.inspect_content(path, &corrected, options);
            if next == corrected {
                return (offenses, corrected, None);
            }
            corrected = next;
        }
        (
            offenses,
            corrected,
            Some(CorrectionError::MaximumIterations),
        )
    }

    fn inspect_prism_only(
        &self,
        path: &str,
        content: &str,
        options: &InspectionConfig,
    ) -> (Vec<Offense>, String) {
        let inspection = self.prism.inspect(
            path,
            content,
            options.autocorrect,
            options.target_ruby_version,
            options.cop_config.clone(),
        );
        let mut offenses = Vec::with_capacity(inspection.findings.len());
        append_prism_offenses(&mut offenses, content, inspection.findings);
        sort_offenses(&mut offenses);
        (offenses, inspection.corrected_source)
    }
}

pub(crate) fn expanded_path(path: &str) -> String {
    let path = PathBuf::from(path);
    let absolute = if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    };
    absolute.to_string_lossy().to_string()
}

#[derive(Debug)]
pub(crate) struct InspectionResult {
    pub(crate) path: String,
    pub(crate) offenses: Vec<Offense>,
}
