// RuboCop 1.87.0
// Source: lib/rubocop/cop/registry.rb
// Source SHA-256: 26b447bb1678935ce6275f071fb0485a89d3bfefc241d61a5dbb51f74e207d71

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fmt;
use std::sync::{Mutex, OnceLock};

use super::badge::Badge;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CopDescriptor {
    badge: Badge,
}

impl CopDescriptor {
    pub(crate) fn new(name: &str) -> Self {
        Self {
            badge: Badge::parse(name),
        }
    }

    pub(crate) fn badge(&self) -> &Badge {
        &self.badge
    }

    pub(crate) fn cop_name(&self) -> String {
        self.badge.to_string()
    }

    pub(crate) fn department(&self) -> Option<&str> {
        self.badge.department()
    }

    pub(crate) fn matches(&self, names: &[String]) -> bool {
        names
            .iter()
            .any(|name| self.badge.matches(&Badge::parse(name)))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum EnabledValue {
    Enabled,
    Disabled,
    Pending,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CopRegistryConfig {
    pub(crate) enabled: EnabledValue,
    pub(crate) safe: bool,
}

impl Default for CopRegistryConfig {
    fn default() -> Self {
        Self {
            enabled: EnabledValue::Enabled,
            safe: true,
        }
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegistryConfig {
    cops: HashMap<String, CopRegistryConfig>,
    enabled_new_cops: bool,
}

impl RegistryConfig {
    pub(crate) fn new(cops: HashMap<String, CopRegistryConfig>, enabled_new_cops: bool) -> Self {
        Self {
            cops,
            enabled_new_cops,
        }
    }

    fn for_cop(&self, name: &str) -> CopRegistryConfig {
        self.cops.get(name).cloned().unwrap_or_default()
    }
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct RegistryOptions {
    pub(crate) only: BTreeSet<String>,
    pub(crate) safe: bool,
    pub(crate) disable_pending_cops: bool,
    pub(crate) enable_pending_cops: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AmbiguousCopName {
    name: String,
    origin: String,
    badges: Vec<Badge>,
}

impl fmt::Display for AmbiguousCopName {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "Ambiguous cop name `{}` used in {} needs department qualifier. Did you mean {}?",
            self.name,
            self.origin,
            self.badges
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join(" or ")
        )
    }
}

#[derive(Clone, Debug)]
pub(crate) struct Registry {
    departments: Vec<String>,
    cops_by_badge: Vec<(Badge, CopDescriptor)>,
    lazy_loaded_cops_by_badge: Vec<(Badge, CopDescriptor)>,
    enrollment_queue: Vec<CopDescriptor>,
    options: RegistryOptions,
    warnings: BTreeMap<String, Vec<String>>,
}

impl Registry {
    pub(crate) fn global() -> &'static Mutex<Registry> {
        static GLOBAL: OnceLock<Mutex<Registry>> = OnceLock::new();
        GLOBAL.get_or_init(|| Mutex::new(Registry::new(Vec::new(), RegistryOptions::default())))
    }

    pub(crate) fn options(&self) -> &RegistryOptions {
        &self.options
    }

    pub(crate) fn initialize(cops: Vec<CopDescriptor>, options: RegistryOptions) -> Self {
        Self::new(cops, options)
    }

    pub(crate) fn new(cops: Vec<CopDescriptor>, options: RegistryOptions) -> Self {
        Self {
            departments: Vec::new(),
            cops_by_badge: Vec::new(),
            lazy_loaded_cops_by_badge: Vec::new(),
            enrollment_queue: cops,
            options,
            warnings: BTreeMap::new(),
        }
    }

    pub(crate) fn lazy_load(&mut self, cop_name: &str) {
        let descriptor = CopDescriptor::new(cop_name);
        if let Some(department) = descriptor.department() {
            insert_department(&mut self.departments, department);
        }
        replace_by_badge(&mut self.lazy_loaded_cops_by_badge, descriptor);
    }

    pub(crate) fn enlist(&mut self, cop: CopDescriptor) {
        self.enrollment_queue.push(cop);
    }

    pub(crate) fn dismiss(&mut self, cop: &CopDescriptor) -> Result<(), String> {
        let Some(index) = self
            .enrollment_queue
            .iter()
            .position(|queued| queued == cop)
        else {
            return Err(format!("Cop {} could not be dismissed", cop.cop_name()));
        };
        self.enrollment_queue.remove(index);
        Ok(())
    }

    pub(crate) fn departments(&mut self) -> Vec<String> {
        self.clear_enrollment_queue();
        self.departments.clone()
    }

    pub(crate) fn with_department(&mut self, department: &str) -> Self {
        let cops = self
            .cops()
            .into_iter()
            .filter(|cop| cop.department() == Some(department))
            .collect();
        Self::new(cops, RegistryOptions::default())
    }

    pub(crate) fn without_department(&mut self, department: &str) -> Self {
        let cops = self
            .cops()
            .into_iter()
            .filter(|cop| cop.department() != Some(department))
            .collect();
        Self::new(cops, RegistryOptions::default())
    }

    pub(crate) fn all(&mut self) -> Vec<CopDescriptor> {
        self.without_department("Test").cops()
    }

    pub(crate) fn with_temporary_global<R>(
        &mut self,
        temporary: Self,
        action: impl FnOnce(&mut Self) -> R,
    ) -> R {
        let previous = std::mem::replace(self, temporary);
        let result = action(self);
        *self = previous;
        result
    }

    pub(crate) fn reset(&mut self) {
        *self = Self::new(Vec::new(), RegistryOptions::default());
    }

    pub(crate) fn qualified_cop(&mut self, name: &str) -> bool {
        let badge = Badge::parse(name);
        self.qualify_badge(&badge).first() == Some(&badge)
    }

    pub(crate) fn department(&mut self, name: &str) -> bool {
        self.departments()
            .iter()
            .any(|department| department == name)
    }

    pub(crate) fn contains_cop_matching(&mut self, names: &[String]) -> bool {
        self.cops().iter().any(|cop| cop.matches(names))
    }

    pub(crate) fn qualified_cop_name(
        &mut self,
        name: &str,
        path: &str,
        warn: bool,
    ) -> Result<String, AmbiguousCopName> {
        let badge = Badge::parse(name);
        if warn && self.department_missing(&badge, name) {
            self.print_department_missing_warning(name, path);
        }
        if self.registered(&badge) {
            return Ok(name.to_owned());
        }
        let potential_badges = self.qualify_badge(&badge);
        match potential_badges.as_slice() {
            [] => Ok(name.to_owned()),
            [real] => Ok(self.resolve_badge(&badge, real, path, warn)),
            _ => Err(AmbiguousCopName {
                name: badge.to_string(),
                origin: path.to_owned(),
                badges: potential_badges,
            }),
        }
    }

    pub(crate) fn department_missing(&mut self, badge: &Badge, name: &str) -> bool {
        !badge.qualified() && self.unqualified_cop_names().contains(name)
    }

    pub(crate) fn unqualified_cop_names(&mut self) -> BTreeSet<String> {
        self.clear_enrollment_queue();
        self.cops_by_badge
            .iter()
            .chain(&self.lazy_loaded_cops_by_badge)
            .map(|(badge, _)| badge.cop_name().to_owned())
            .chain(["RedundantCopDisableDirective".to_owned()])
            .collect()
    }

    pub(crate) fn qualify_badge(&mut self, badge: &Badge) -> Vec<Badge> {
        self.clear_enrollment_queue();
        let departments = self.departments.to_vec();
        departments
            .into_iter()
            .map(|department| badge.with_department(&department))
            .filter(|potential| self.registered(potential))
            .collect()
    }

    pub(crate) fn cops(&mut self) -> Vec<CopDescriptor> {
        self.clear_enrollment_queue();
        self.load_all_lazy_cops();
        self.cops_by_badge
            .iter()
            .map(|(_, cop)| cop.clone())
            .collect()
    }

    pub(crate) fn length(&mut self) -> usize {
        self.clear_enrollment_queue();
        self.cops_by_badge.len() + self.lazy_loaded_cops_by_badge.len()
    }

    pub(crate) fn enabled(&mut self, config: &RegistryConfig) -> Vec<CopDescriptor> {
        let cops = self.cops();
        cops.into_iter()
            .filter(|cop| self.enabled_cop(cop, config))
            .collect()
    }

    pub(crate) fn disabled(&mut self, config: &RegistryConfig) -> Vec<CopDescriptor> {
        let cops = self.cops();
        cops.into_iter()
            .filter(|cop| !self.enabled_cop(cop, config))
            .collect()
    }

    pub(crate) fn enabled_cop(&self, cop: &CopDescriptor, config: &RegistryConfig) -> bool {
        if self.options.only.contains(&cop.cop_name()) {
            return true;
        }
        let cop_config = config.for_cop(&cop.cop_name());
        let enabled = cop_config.enabled == EnabledValue::Enabled
            || self.enabled_pending_cop(&cop_config, config);
        if self.options.safe {
            enabled && cop_config.safe
        } else {
            enabled
        }
    }

    pub(crate) fn names(&mut self) -> Vec<String> {
        self.clear_enrollment_queue();
        self.cops_by_badge
            .iter()
            .chain(&self.lazy_loaded_cops_by_badge)
            .map(|(badge, _)| badge.to_string())
            .collect()
    }

    pub(crate) fn cops_for_department(&mut self, department: &str) -> Vec<CopDescriptor> {
        self.cops()
            .into_iter()
            .filter(|cop| cop.department() == Some(department))
            .collect()
    }

    pub(crate) fn names_for_department(&mut self, department: &str) -> Vec<String> {
        self.cops_for_department(department)
            .iter()
            .map(CopDescriptor::cop_name)
            .collect()
    }

    pub(crate) fn equivalent(&mut self, other: &mut Self) -> bool {
        self.cops() == other.cops()
    }

    pub(crate) fn select(
        &mut self,
        predicate: impl Fn(&CopDescriptor) -> bool,
    ) -> Vec<CopDescriptor> {
        self.cops().into_iter().filter(predicate).collect()
    }

    pub(crate) fn each(&mut self) -> Vec<CopDescriptor> {
        self.cops()
    }

    #[allow(clippy::wrong_self_convention)] // Preserve RuboCop's public `to_h` API name.
    pub(crate) fn to_h(&mut self) -> BTreeMap<String, Vec<CopDescriptor>> {
        self.cops()
            .into_iter()
            .map(|cop| (cop.cop_name(), vec![cop]))
            .collect()
    }

    pub(crate) fn sort(&mut self) {
        self.clear_enrollment_queue();
        self.load_all_lazy_cops();
        self.cops_by_badge
            .sort_by(|(left, _), (right, _)| left.cop_name().cmp(right.cop_name()));
    }

    pub(crate) fn find_by_cop_name(&mut self, cop_name: &str) -> Option<CopDescriptor> {
        self.clear_enrollment_queue();
        let badge = Badge::parse(cop_name);
        self.cops_by_badge
            .iter()
            .find(|(candidate, _)| candidate == &badge)
            .map(|(_, cop)| cop.clone())
            .or_else(|| self.load_lazy_cop(&badge))
    }

    pub(crate) fn find_cops_by_directive(&mut self, directive: &str) -> Vec<CopDescriptor> {
        self.find_by_cop_name(directive)
            .map_or_else(|| self.cops_for_department(directive), |cop| vec![cop])
    }

    pub(crate) fn warnings(&self, path: &str) -> Option<&[String]> {
        self.warnings.get(path).map(Vec::as_slice)
    }

    pub(crate) fn freeze(&mut self) {
        self.clear_enrollment_queue();
        self.load_all_lazy_cops();
        let _ = self.unqualified_cop_names();
    }

    pub(crate) fn initialize_copy(registry: &mut Self) -> Self {
        Self::initialize(registry.cops(), registry.options.clone())
    }

    pub(crate) fn with(cops: Vec<CopDescriptor>) -> Self {
        Self::new(cops, RegistryOptions::default())
    }

    fn enabled_pending_cop(&self, cop_config: &CopRegistryConfig, config: &RegistryConfig) -> bool {
        !self.options.disable_pending_cops
            && cop_config.enabled == EnabledValue::Pending
            && (self.options.enable_pending_cops || config.enabled_new_cops)
    }

    fn clear_enrollment_queue(&mut self) {
        for cop in std::mem::take(&mut self.enrollment_queue) {
            if let Some(department) = cop.department() {
                insert_department(&mut self.departments, department);
            }
            replace_by_badge(&mut self.cops_by_badge, cop);
        }
    }

    fn load_all_lazy_cops(&mut self) {
        for (_, cop) in std::mem::take(&mut self.lazy_loaded_cops_by_badge) {
            replace_by_badge(&mut self.cops_by_badge, cop);
        }
    }

    fn load_lazy_cop(&mut self, badge: &Badge) -> Option<CopDescriptor> {
        let index = self
            .lazy_loaded_cops_by_badge
            .iter()
            .position(|(candidate, _)| candidate == badge)?;
        let (_, cop) = self.lazy_loaded_cops_by_badge.remove(index);
        replace_by_badge(&mut self.cops_by_badge, cop.clone());
        Some(cop)
    }

    fn registered(&mut self, badge: &Badge) -> bool {
        self.clear_enrollment_queue();
        self.cops_by_badge
            .iter()
            .chain(&self.lazy_loaded_cops_by_badge)
            .any(|(candidate, _)| candidate == badge)
    }

    fn resolve_badge(&mut self, given: &Badge, real: &Badge, path: &str, warn: bool) -> String {
        if warn && !given.matches(real) {
            self.emit_warning(
                path,
                format!(
                    "{given} has the wrong namespace - replace it with {}",
                    given.with_department(real.department().unwrap_or_default())
                ),
            );
        }
        real.to_string()
    }

    fn print_department_missing_warning(&mut self, name: &str, path: &str) {
        let mut message = format!("no department given for {name}.");
        if path.ends_with(".rb") {
            message.push_str(" Run `rubocop -a --only Migration/DepartmentName` to fix.");
        }
        self.emit_warning(path, message);
    }

    fn emit_warning(&mut self, path: &str, message: String) {
        self.warnings
            .entry(path.to_owned())
            .or_default()
            .push(message);
    }
}

impl PartialEq for Registry {
    fn eq(&self, other: &Self) -> bool {
        let mut left = self.clone();
        let mut right = other.clone();
        left.cops() == right.cops()
    }
}

impl Eq for Registry {}

fn replace_by_badge(entries: &mut Vec<(Badge, CopDescriptor)>, cop: CopDescriptor) {
    if let Some((_, current)) = entries.iter_mut().find(|(badge, _)| badge == cop.badge()) {
        *current = cop;
    } else {
        entries.push((cop.badge.clone(), cop));
    }
}

fn insert_department(departments: &mut Vec<String>, department: &str) {
    if !departments.iter().any(|current| current == department) {
        departments.push(department.to_owned());
    }
}
