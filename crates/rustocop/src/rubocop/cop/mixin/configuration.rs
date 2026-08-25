// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/array_min_size.rb
// Source SHA-256: 7110ee9b0a7738d9757c0a4991fb2c7b88c8dfb3a7b312835b262064425fff59
// Source: lib/rubocop/cop/mixin/configurable_enforced_style.rb
// Source SHA-256: b0c252d3d11dff73d6dcfdd77acd552bdc48f18423c2c530934b1e6feb4948f1
// Source: lib/rubocop/cop/mixin/configurable_max.rb
// Source SHA-256: 97a87efd086509d9c88a6194cd849c44952fee4d7a34b98abe30cb6646ea5c32
// Source: lib/rubocop/cop/mixin/configurable_naming.rb
// Source SHA-256: e3217a6747bae391c7881368154f3c807e63bfaa7c7d1f9ebb5e30109b1ed4f7
// Source: lib/rubocop/cop/mixin/configurable_numbering.rb
// Source SHA-256: 341a70d448ef754a1165b57fae1b01c83577e0df3a6752b1641d750df854172b
// Source: lib/rubocop/cop/mixin/symbol_help.rb
// Source SHA-256: 585801993819ff53ade687f36f064ccad2d6a455825d5eec58eede0de2e286e4

use std::collections::BTreeMap;
use std::io;

use crate::rubocop::cop::exclude_limit::ExcludeLimit;

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum AutoConfigValue {
    Bool(bool),
    Integer(usize),
    Style(String),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArrayStyle {
    Brackets,
    Percent,
}

#[derive(Clone, Debug)]
pub(crate) struct ArrayMinSize {
    min_size: usize,
    largest_brackets: Option<usize>,
    smallest_percent: Option<usize>,
    config_to_allow_offenses: BTreeMap<String, AutoConfigValue>,
}

impl ArrayMinSize {
    pub(crate) fn new(min_size: usize) -> Self {
        Self {
            min_size,
            largest_brackets: None,
            smallest_percent: None,
            config_to_allow_offenses: BTreeMap::new(),
        }
    }

    pub(crate) fn below_array_length(&self, length: usize) -> bool {
        length < self.min_size
    }

    pub(crate) fn min_size_config(&self) -> usize {
        self.min_size
    }

    pub(crate) fn largest_brackets_size(
        &mut self,
        style: ArrayStyle,
        size: usize,
    ) -> Option<usize> {
        if style == ArrayStyle::Brackets {
            self.largest_brackets =
                Some(self.largest_brackets.map_or(size, |value| value.max(size)));
        }
        self.largest_brackets
    }

    pub(crate) fn smallest_percent_size(
        &mut self,
        style: ArrayStyle,
        size: usize,
    ) -> Option<usize> {
        if style == ArrayStyle::Percent {
            self.smallest_percent =
                Some(self.smallest_percent.map_or(size, |value| value.min(size)));
        }
        self.smallest_percent
    }

    pub(crate) fn array_style_detected(&mut self, style: ArrayStyle, array_size: usize) {
        if self.config_to_allow_offenses.get("Enabled") == Some(&AutoConfigValue::Bool(false)) {
            return;
        }
        if style == ArrayStyle::Brackets {
            self.largest_brackets = Some(
                self.largest_brackets
                    .map_or(array_size, |value| value.max(array_size)),
            );
        }
        if style == ArrayStyle::Percent {
            self.smallest_percent = Some(
                self.smallest_percent
                    .map_or(array_size, |value| value.min(array_size)),
            );
        }
        let largest = self
            .largest_brackets
            .map_or(f64::NEG_INFINITY, |value| value as f64);
        let smallest = self
            .smallest_percent
            .map_or(f64::INFINITY, |value| value as f64);
        let detected = match style {
            ArrayStyle::Brackets => "brackets",
            ArrayStyle::Percent => "percent",
        };
        match self.config_to_allow_offenses.get("EnforcedStyle") {
            Some(AutoConfigValue::Style(current)) if current == detected => {}
            None => {
                self.config_to_allow_offenses.insert(
                    "EnforcedStyle".into(),
                    AutoConfigValue::Style(detected.into()),
                );
            }
            _ if smallest <= largest => {
                self.config_to_allow_offenses =
                    BTreeMap::from([("Enabled".into(), AutoConfigValue::Bool(false))]);
            }
            _ => {
                self.config_to_allow_offenses.insert(
                    "EnforcedStyle".into(),
                    AutoConfigValue::Style("percent".into()),
                );
                self.config_to_allow_offenses.insert(
                    "MinSize".into(),
                    AutoConfigValue::Integer(largest as usize + 1),
                );
            }
        }
    }

    pub(crate) fn config_to_allow_offenses(&self) -> &BTreeMap<String, AutoConfigValue> {
        &self.config_to_allow_offenses
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum StyleError {
    Unknown(String),
    NotBinary,
}

#[derive(Clone, Debug)]
pub(crate) struct ConfigurableEnforcedStyle {
    parameter_name: String,
    style: String,
    supported_styles: Vec<String>,
    detected_style: Option<Vec<String>>,
    config_to_allow_offenses: BTreeMap<String, AutoConfigValue>,
}

impl ConfigurableEnforcedStyle {
    pub(crate) fn new(
        parameter_name: &str,
        style: &str,
        supported_styles: Vec<String>,
    ) -> Result<Self, StyleError> {
        if !supported_styles.iter().any(|candidate| candidate == style) {
            return Err(StyleError::Unknown(style.to_owned()));
        }
        Ok(Self {
            parameter_name: parameter_name.to_owned(),
            style: style.to_owned(),
            supported_styles,
            detected_style: None,
            config_to_allow_offenses: BTreeMap::new(),
        })
    }

    pub(crate) fn style(&self) -> &str {
        &self.style
    }

    pub(crate) fn supported_styles(&self) -> &[String] {
        &self.supported_styles
    }

    pub(crate) fn alternative_style(&self) -> Result<&str, StyleError> {
        if self.supported_styles.len() != 2 {
            return Err(StyleError::NotBinary);
        }
        Ok(self.alternative_styles()[0])
    }

    pub(crate) fn alternative_styles(&self) -> Vec<&str> {
        self.supported_styles
            .iter()
            .map(String::as_str)
            .filter(|candidate| *candidate != self.style)
            .collect()
    }

    pub(crate) fn opposite_style_detected(&mut self) {
        if let Ok(style) = self.alternative_style().map(str::to_owned) {
            self.style_detected(&[style]);
        }
    }

    pub(crate) fn correct_style_detected(&mut self) {
        let style = self.style.clone();
        self.style_detected(std::slice::from_ref(&style));
    }

    pub(crate) fn unexpected_style_detected(&mut self, unexpected: &str) {
        self.style_detected(&[unexpected.to_owned()]);
    }

    pub(crate) fn ambiguous_style_detected(&mut self, possibilities: &[String]) {
        self.style_detected(possibilities);
    }

    pub(crate) fn style_detected(&mut self, detected: &[String]) {
        if self.no_acceptable_style() {
            return;
        }
        let updated = self.detected_style.as_ref().map_or_else(
            || detected.to_vec(),
            |current| {
                current
                    .iter()
                    .filter(|style| detected.contains(style))
                    .cloned()
                    .collect()
            },
        );
        if updated.is_empty() {
            self.no_acceptable_style_mut();
        } else {
            self.config_to_allow_offenses.insert(
                self.parameter_name.clone(),
                AutoConfigValue::Style(updated[0].clone()),
            );
            self.detected_style = Some(updated);
        }
    }

    pub(crate) fn no_acceptable_style(&self) -> bool {
        self.config_to_allow_offenses.get("Enabled") == Some(&AutoConfigValue::Bool(false))
    }

    pub(crate) fn no_acceptable_style_mut(&mut self) {
        self.config_to_allow_offenses =
            BTreeMap::from([("Enabled".into(), AutoConfigValue::Bool(false))]);
    }

    pub(crate) fn conflicting_styles_detected(&mut self) {
        self.no_acceptable_style_mut();
    }

    pub(crate) fn unrecognized_style_detected(&mut self) {
        self.no_acceptable_style_mut();
    }

    pub(crate) fn style_configured(&self) -> bool {
        true
    }

    pub(crate) fn style_parameter_name(&self) -> &str {
        &self.parameter_name
    }

    pub(crate) fn detected_style(&self) -> Option<&[String]> {
        self.detected_style.as_deref()
    }

    pub(crate) fn config_to_allow_offenses(&self) -> &BTreeMap<String, AutoConfigValue> {
        &self.config_to_allow_offenses
    }
}

pub(crate) fn valid_naming(name: &str, style: &str) -> bool {
    let core = name.trim_start_matches('@');
    let core = core.strip_suffix(['!', '?', '=']).unwrap_or(core);
    if name.len() - core.len() > 4 || core.is_empty() {
        return false;
    }
    match style {
        "snake_case" => core.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        }),
        "camelCase" => {
            let rest = core.strip_prefix('_').unwrap_or(core);
            rest.starts_with(|character: char| character.is_ascii_lowercase())
                && rest
                    .chars()
                    .all(|character| character.is_ascii_alphanumeric() || character == '_')
        }
        _ => false,
    }
}

pub(crate) fn valid_numbering(name: &str, style: &str) -> bool {
    if name.len() > 1
        && name.starts_with('_')
        && name[1..]
            .chars()
            .all(|character| character.is_ascii_digit())
    {
        return true;
    }
    let Some(last) = name.chars().last() else {
        return false;
    };
    let digit_start = name
        .char_indices()
        .rev()
        .find(|(_, character)| !character.is_ascii_digit())
        .map_or(0, |(index, character)| index + character.len_utf8());
    let before_digits = digit_start
        .checked_sub(1)
        .and_then(|index| name.as_bytes().get(index))
        .copied();
    match style {
        "snake_case" => {
            !last.is_ascii_digit()
                || name.chars().all(|character| character.is_ascii_digit())
                || before_digits == Some(b'_')
        }
        "normalcase" => {
            !last.is_ascii_digit()
                || name.chars().all(|character| character.is_ascii_digit())
                || before_digits
                    .is_some_and(|character| character != b'_' && !character.is_ascii_digit())
        }
        "non_integer" => {
            !last.is_ascii_digit() || name.chars().all(|character| character.is_ascii_digit())
        }
        _ => false,
    }
}

pub(crate) fn hash_key(parent_is_pair: bool, is_first_child: bool) -> bool {
    parent_is_pair && is_first_child
}

pub(crate) fn configurable_max(
    exclude_limit: &ExcludeLimit,
    cop_name: &str,
    value: i64,
) -> io::Result<()> {
    exclude_limit.record(cop_name, max_parameter_name(), value)
}

pub(crate) const fn max_parameter_name() -> &'static str {
    "Max"
}
