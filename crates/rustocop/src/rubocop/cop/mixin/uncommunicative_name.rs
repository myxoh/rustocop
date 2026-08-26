// RuboCop 1.87.0
// Source: lib/rubocop/cop/mixin/uncommunicative_name.rb
// Source SHA-256: 4050d3508e49cd64552f02caeb6780ce47ca6be4dc9fb3f599184a28bab55c23

use std::ops::Range;

const CASE_MSG: &str = "Only use lowercase characters for %s.";
const NUM_MSG: &str = "Do not end %s with a number.";
const LENGTH_MSG: &str = "%s must be at least %s characters long.";
const FORBIDDEN_MSG: &str = "Do not use %s as a name for a %s.";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ArgumentKind {
    Argument,
    RestArgument,
    KeywordRestArgument,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Argument {
    pub(crate) name: Option<String>,
    pub(crate) begin: usize,
    pub(crate) kind: ArgumentKind,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum ParameterOwner {
    Block,
    Method,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct NameOffense {
    pub(crate) range: Range<usize>,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UncommunicativeName {
    allowed_names: Vec<String>,
    forbidden_names: Vec<String>,
    allow_nums: bool,
    min_length: usize,
}

impl UncommunicativeName {
    pub(crate) fn new(
        allowed_names: Vec<String>,
        forbidden_names: Vec<String>,
        allow_nums: bool,
        min_length: usize,
    ) -> Self {
        Self {
            allowed_names,
            forbidden_names,
            allow_nums,
            min_length,
        }
    }

    pub(crate) fn check(&self, node: ParameterOwner, args: &[Argument]) -> Vec<NameOffense> {
        let mut offenses = Vec::new();
        for arg in args {
            let Some(full_name) = arg.name.as_deref() else {
                continue;
            };
            if full_name == "_" {
                continue;
            }
            let name = full_name.trim_start_matches('_');
            if self.allowed_names().iter().any(|allowed| allowed == name) {
                continue;
            }

            let mut length = full_name.chars().count();
            length += usize::from(arg.kind == ArgumentKind::RestArgument);
            length += 2 * usize::from(arg.kind == ArgumentKind::KeywordRestArgument);
            let range = self.arg_range(arg, length);
            offenses.extend(self.issue_offenses(node, range, name));
        }
        offenses
    }

    pub(crate) fn issue_offenses(
        &self,
        node: ParameterOwner,
        range: Range<usize>,
        name: &str,
    ) -> Vec<NameOffense> {
        let mut offenses = Vec::new();
        if self
            .forbidden_names()
            .iter()
            .any(|forbidden| forbidden == name)
        {
            offenses.push(self.forbidden_offense(node, range.clone(), name));
        }
        if self.uppercase(name) {
            offenses.push(self.case_offense(node, range.clone()));
        }
        if !self.long_enough(name) {
            offenses.push(self.length_offense(node, range.clone()));
        }
        if !self.allow_nums() && self.ends_with_num(name) {
            offenses.push(self.num_offense(node, range));
        }
        offenses
    }

    pub(crate) fn case_offense(&self, node: ParameterOwner, range: Range<usize>) -> NameOffense {
        NameOffense {
            range,
            message: CASE_MSG.replacen("%s", self.name_type(node), 1),
        }
    }

    pub(crate) fn uppercase(&self, name: &str) -> bool {
        name.chars().any(char::is_uppercase)
    }

    pub(crate) fn name_type(&self, node: ParameterOwner) -> &'static str {
        match node {
            ParameterOwner::Block => "block parameter",
            ParameterOwner::Method => "method parameter",
        }
    }

    pub(crate) fn num_offense(&self, node: ParameterOwner, range: Range<usize>) -> NameOffense {
        NameOffense {
            range,
            message: NUM_MSG.replacen("%s", self.name_type(node), 1),
        }
    }

    pub(crate) fn ends_with_num(&self, name: &str) -> bool {
        name.chars()
            .next_back()
            .is_some_and(|character| character.is_ascii_digit())
    }

    pub(crate) fn length_offense(&self, node: ParameterOwner, range: Range<usize>) -> NameOffense {
        let name_type = self.name_type(node);
        let capitalized = format!("{}{}", &name_type[..1].to_uppercase(), &name_type[1..]);
        NameOffense {
            range,
            message: LENGTH_MSG.replacen("%s", &capitalized, 1).replacen(
                "%s",
                &self.min_length().to_string(),
                1,
            ),
        }
    }

    pub(crate) fn long_enough(&self, name: &str) -> bool {
        name.chars().count() >= self.min_length()
    }

    pub(crate) fn arg_range(&self, arg: &Argument, length: usize) -> Range<usize> {
        arg.begin..arg.begin + length
    }

    pub(crate) fn forbidden_offense(
        &self,
        node: ParameterOwner,
        range: Range<usize>,
        name: &str,
    ) -> NameOffense {
        NameOffense {
            range,
            message: FORBIDDEN_MSG
                .replacen("%s", name, 1)
                .replacen("%s", self.name_type(node), 1),
        }
    }

    pub(crate) fn allowed_names(&self) -> &[String] {
        &self.allowed_names
    }

    pub(crate) fn forbidden_names(&self) -> &[String] {
        &self.forbidden_names
    }

    pub(crate) fn allow_nums(&self) -> bool {
        self.allow_nums
    }

    pub(crate) fn min_length(&self) -> usize {
        self.min_length
    }
}
