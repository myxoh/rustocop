use std::fs;
use std::io;

use crate::diagnostic::{append_prism_offenses, sort_offenses, Offense};
use crate::source_lines;
use crate::Options;
use crate::{cop_enabled, cop_registry::SUPPORTED_COPS, expanded_path, line_cops, prism_engine};

pub(crate) struct InspectionPlan {
    engine: prism_engine::Engine,
    line_cops_enabled: bool,
}

impl InspectionPlan {
    pub(crate) fn new(options: &Options) -> Self {
        Self {
            engine: prism_engine::Engine::new(&|cop| cop_enabled(options, cop)),
            line_cops_enabled: SUPPORTED_COPS
                .iter()
                .any(|cop| !prism_engine::PRISM_COPS.contains(cop) && cop_enabled(options, cop)),
        }
    }

    pub(crate) fn inspect_file(
        &self,
        path: &str,
        options: &Options,
    ) -> io::Result<InspectionResult> {
        let content = fs::read_to_string(path)?;
        let absolute_path = expanded_path(path);
        let (offenses, corrected_content) = self.inspect_content(&absolute_path, &content, options);
        if options.autocorrect && corrected_content != content {
            fs::write(path, corrected_content)?;
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
        options: &Options,
    ) -> (Vec<Offense>, String) {
        if !self.line_cops_enabled {
            return self.inspect_prism_only(content, options);
        }

        let mut lines = source_lines::split(content);
        let original_lines = lines.clone();
        let mut offenses = Vec::new();
        line_cops::before_prism(path, &mut lines, options, &mut offenses);

        let prism_source = source_lines::join(&lines);
        let prism_inspection = self.engine.inspect(
            &prism_source,
            options.autocorrect,
            options.target_ruby_version,
        );
        append_prism_offenses(&mut offenses, &prism_source, prism_inspection.findings);
        line_cops::after_prism(path, &original_lines, options, &mut offenses);
        sort_offenses(&mut offenses);
        (offenses, prism_inspection.corrected_source)
    }

    fn inspect_prism_only(&self, content: &str, options: &Options) -> (Vec<Offense>, String) {
        let inspection =
            self.engine
                .inspect(content, options.autocorrect, options.target_ruby_version);
        let mut offenses = Vec::with_capacity(inspection.findings.len());
        append_prism_offenses(&mut offenses, content, inspection.findings);
        sort_offenses(&mut offenses);
        (offenses, inspection.corrected_source)
    }
}

#[derive(Debug)]
pub(crate) struct InspectionResult {
    pub(crate) path: String,
    pub(crate) offenses: Vec<Offense>,
}
