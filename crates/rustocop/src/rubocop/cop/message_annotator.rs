// RuboCop 1.87.0
// Source: lib/rubocop/cop/message_annotator.rb
// Source SHA-256: c1fc65fad14bcd11da2f8d266a74e0f2493d74aaa9db32eb5e9b8d82eb4fd502

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum Urls {
    One(String),
    Many(Vec<String>),
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct MessageConfig {
    pub(crate) all_cops: HashMap<String, String>,
    pub(crate) departments: HashMap<String, HashMap<String, String>>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CopMessageConfig {
    pub(crate) details: Option<String>,
    pub(crate) style_guide: Option<String>,
    pub(crate) references: Option<Urls>,
    pub(crate) reference: Option<Urls>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct MessageOptions {
    pub(crate) display_style_guide: bool,
    pub(crate) extra_details: bool,
    pub(crate) debug: bool,
    pub(crate) display_cop_names: Option<bool>,
    pub(crate) format: Option<String>,
}

pub(crate) struct MessageAnnotator<'a> {
    config: &'a MessageConfig,
    cop_name: &'a str,
    cop_config: &'a CopMessageConfig,
    options: &'a MessageOptions,
}

impl<'a> MessageAnnotator<'a> {
    pub(crate) fn options(&self) -> &MessageOptions {
        self.options
    }

    pub(crate) fn config(&self) -> &MessageConfig {
        self.config
    }

    pub(crate) fn cop_name(&self) -> &str {
        self.cop_name
    }

    pub(crate) fn cop_config(&self) -> &CopMessageConfig {
        self.cop_config
    }

    pub(crate) fn style_guide_urls() -> &'static Mutex<HashMap<String, String>> {
        static URLS: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
        URLS.get_or_init(|| Mutex::new(HashMap::new()))
    }

    pub(crate) fn new(
        config: &'a MessageConfig,
        cop_name: &'a str,
        cop_config: &'a CopMessageConfig,
        options: &'a MessageOptions,
    ) -> Self {
        Self {
            config,
            cop_name,
            cop_config,
            options,
        }
    }

    pub(crate) fn annotate(&self, message: &str) -> String {
        let mut message = if self.display_cop_names() {
            format!("{}: {message}", self.cop_name)
        } else {
            message.to_owned()
        };
        if self.extra_details() {
            if let Some(details) = self.details() {
                message.push(' ');
                message.push_str(details);
            }
        }
        let urls = self.urls();
        if self.display_style_guide(&urls) {
            message.push_str(" (");
            message.push_str(&urls.join(", "));
            message.push(')');
        }
        message
    }

    pub(crate) fn debug(&self) -> bool {
        self.options.debug
    }

    pub(crate) fn urls(&self) -> Vec<String> {
        self.style_guide_url()
            .into_iter()
            .chain(self.reference_urls())
            .collect()
    }

    fn style_guide_url(&self) -> Option<String> {
        let url = self.cop_config.style_guide.as_deref()?;
        if url.is_empty() {
            return None;
        }
        if let Some(cached) = Self::style_guide_urls().lock().unwrap().get(url).cloned() {
            return Some(cached);
        }
        let Some(base) = self.style_guide_base_url() else {
            return Some(url.to_owned());
        };
        let resolved = resolve_url(base, url);
        Self::style_guide_urls()
            .lock()
            .unwrap()
            .insert(url.to_owned(), resolved.clone());
        Some(resolved)
    }

    fn style_guide_base_url(&self) -> Option<&str> {
        let department = self
            .cop_name
            .rsplit_once('/')
            .map_or("", |(department, _)| department);
        self.config
            .departments
            .get(department)
            .and_then(|config| config.get("StyleGuideBaseURL"))
            .or_else(|| self.config.all_cops.get("StyleGuideBaseURL"))
            .map(String::as_str)
    }

    fn display_style_guide(&self, urls: &[String]) -> bool {
        (self.options.display_style_guide || self.all_cops_bool("DisplayStyleGuide"))
            && !urls.is_empty()
    }

    fn reference_urls(&self) -> Vec<String> {
        [&self.cop_config.references, &self.cop_config.reference]
            .into_iter()
            .flatten()
            .flat_map(|urls| match urls {
                Urls::One(url) => vec![url.clone()],
                Urls::Many(urls) => urls.clone(),
            })
            .filter(|url| !url.is_empty())
            .collect()
    }

    fn extra_details(&self) -> bool {
        self.options.extra_details || self.all_cops_bool("ExtraDetails")
    }

    fn display_cop_names(&self) -> bool {
        if self.options.debug {
            return true;
        }
        if let Some(value) = self.options.display_cop_names {
            return value;
        }
        if self.options.format.as_deref() == Some("json") {
            return false;
        }
        self.all_cops_bool("DisplayCopNames")
    }

    fn details(&self) -> Option<&str> {
        self.cop_config
            .details
            .as_deref()
            .filter(|details| !details.is_empty())
    }

    fn all_cops_bool(&self, key: &str) -> bool {
        self.config
            .all_cops
            .get(key)
            .is_some_and(|value| value == "true")
    }
}

fn resolve_url(base: &str, url: &str) -> String {
    if url.contains("://") {
        return url.to_owned();
    }
    if url.starts_with('#') {
        return format!("{}{}", base.trim_end_matches('/'), url);
    }
    let (origin, mut path) = match base.find("://") {
        Some(scheme) => {
            let path_start = base[scheme + 3..]
                .find('/')
                .map_or(base.len(), |index| scheme + 3 + index);
            (&base[..path_start], base[path_start..].to_owned())
        }
        None => ("", base.to_owned()),
    };
    if !path.ends_with('/') {
        if let Some(index) = path.rfind('/') {
            path.truncate(index + 1);
        } else {
            path.clear();
        }
    }
    for part in url.split('/') {
        match part {
            ".." => {
                let trimmed = path.trim_end_matches('/');
                path.truncate(trimmed.rfind('/').map_or(0, |index| index + 1));
            }
            "." | "" => {}
            _ => {
                if !path.ends_with('/') {
                    path.push('/');
                }
                path.push_str(part);
            }
        }
    }
    format!("{origin}{path}")
}
