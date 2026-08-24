use super::*;

/// A diagnostic context already scoped to one cop. Rule helpers use this
/// instead of accepting and forwarding a separate cop-name argument.
pub(in super::super) struct Reporter<'context> {
    pub(super) cop_name: &'static str,
    pub(super) context: &'context mut Context,
}

impl Reporter<'_> {
    pub(in super::super) fn autocorrect_enabled(&self) -> bool {
        self.context
            .autocorrect
            .enabled_for(&self.context.cop_config, self.cop_name)
    }

    #[allow(dead_code)]
    pub(in super::super) fn path(&self) -> &str {
        &self.context.path
    }

    pub(in super::super) fn target_ruby_version(&self) -> RubyVersion {
        self.context.target_ruby_version()
    }

    pub(in super::super) fn config_value(&self, key: &str) -> Option<&str> {
        self.context.config_value(self.cop_name, key)
    }

    pub(in super::super) fn config_bool(&self, key: &str, default: bool) -> bool {
        self.context
            .cop_config
            .bool(self.cop_name, key)
            .unwrap_or(default)
    }

    #[allow(dead_code)]
    pub(in super::super) fn config_usize(&self, key: &str, default: usize) -> usize {
        self.context
            .cop_config
            .usize(self.cop_name, key)
            .unwrap_or(default)
    }

    #[allow(dead_code)]
    pub(in super::super) fn config_values(&self, key: &str) -> &[String] {
        self.context.cop_config.values(self.cop_name, key)
    }

    pub(in super::super) fn config_contains(&self, key: &str) -> bool {
        self.context.cop_config.contains(self.cop_name, key)
    }

    pub(in super::super) fn config_map(
        &self,
        key: &str,
    ) -> Option<&std::collections::HashMap<String, String>> {
        self.context.cop_config.map(self.cop_name, key)
    }

    pub(in super::super) fn config_symbol_map(
        &self,
        key: &str,
    ) -> Option<&std::collections::HashMap<String, String>> {
        self.context.cop_config.symbol_map(self.cop_name, key)
    }

    pub(in super::super) fn policy(&self) -> CopPolicy<'_> {
        CopPolicy::new(&self.context.cop_config, self.cop_name)
    }

    pub(in super::super) fn related_config_value(&self, cop_name: &str, key: &str) -> Option<&str> {
        self.context.config_value(cop_name, key)
    }

    pub(in super::super) fn related_config_values(&self, cop_name: &str, key: &str) -> &[String] {
        self.context.cop_config.values(cop_name, key)
    }

    pub(in super::super) fn related_config_explicit(&self, cop_name: &str, key: &str) -> bool {
        self.context.cop_config.explicitly_contains(cop_name, key)
    }

    pub(in super::super) fn related_config_map(
        &self,
        cop_name: &str,
        key: &str,
    ) -> Option<&std::collections::HashMap<String, String>> {
        self.context.cop_config.map(cop_name, key)
    }

    pub(in super::super) fn report(&mut self, message: impl Into<String>, offense: impl ByteRange) {
        self.context.report(self.cop_name, message, offense);
    }

    pub(in super::super) fn replace(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
        replacement: impl Into<String>,
    ) {
        self.context
            .replace(self.cop_name, message, offense, edit, replacement);
    }

    /// Records one atomic correction. Either every edit is accepted or none
    /// are, so a finding is never marked corrected after a partial rewrite.
    pub(in super::super) fn replace_many(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edits: Vec<(Range<usize>, String)>,
    ) {
        self.context
            .replace_many(self.cop_name, message, offense, edits);
    }

    pub(in super::super) fn remove(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
    ) {
        self.context.remove(self.cop_name, message, offense, edit);
    }

    pub(in super::super) fn insert(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        offset: usize,
        text: impl Into<String>,
    ) {
        self.context
            .insert(self.cop_name, message, offense, offset, text);
    }

    /// Records an offense that cannot be corrected in isolation but is
    /// resolved by the supplied broader correction transaction.
    pub(in super::super) fn replace_indirectly(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edit: impl ByteRange,
        replacement: impl Into<String>,
    ) {
        self.context
            .replace_indirectly(self.cop_name, message, offense, edit, replacement);
    }

    pub(in super::super) fn replace_many_indirectly(
        &mut self,
        message: impl Into<String>,
        offense: impl ByteRange,
        edits: Vec<(Range<usize>, String)>,
    ) {
        self.context
            .replace_many_indirectly(self.cop_name, message, offense, edits);
    }
}
