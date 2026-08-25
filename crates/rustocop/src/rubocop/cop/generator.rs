// RuboCop 1.87.0
// Source: lib/rubocop/cop/generator.rb
// Source SHA-256: cfe84001c8a5c023786f90662376528a912ed7fb5716d40f556ade9173b951ea

use std::cell::{Ref, RefCell};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use regex::Regex;

use super::badge::Badge;

const SOURCE_TEMPLATE: &str = r#"# frozen_string_literal: true

module RuboCop
  module Cop
    module %<department>s
      # TODO: Write cop description and example of bad / good code. For every
      # `SupportedStyle` and unique configuration, there needs to be examples.
      # Examples must have valid Ruby syntax. Do not use upticks.
      #
      # @safety
      #   Delete this section if the cop is not unsafe (`Safe: false` or
      #   `SafeAutoCorrect: false`), or use it to explain how the cop is
      #   unsafe.
      #
      # @example EnforcedStyle: bar (default)
      #   # Description of the `bar` style.
      #
      #   # bad
      #   bad_bar_method
      #
      #   # bad
      #   bad_bar_method(args)
      #
      #   # good
      #   good_bar_method
      #
      #   # good
      #   good_bar_method(args)
      #
      # @example EnforcedStyle: foo
      #   # Description of the `foo` style.
      #
      #   # bad
      #   bad_foo_method
      #
      #   # bad
      #   bad_foo_method(args)
      #
      #   # good
      #   good_foo_method
      #
      #   # good
      #   good_foo_method(args)
      #
      class %<cop_name>s < Base
        # TODO: Implement the cop in here.
        #
        # In many cases, you can use a node matcher for matching node pattern.
        # See https://github.com/rubocop/rubocop-ast/blob/master/lib/rubocop/ast/node_pattern.rb
        #
        # For example
        MSG = 'Use `#good_method` instead of `#bad_method`.'

        # TODO: Don't call `on_send` unless the method name is in this list
        # If you don't need `on_send` in the cop you created, remove it.
        RESTRICT_ON_SEND = %i[bad_method].freeze

        # @!method bad_method?(node)
        def_node_matcher :bad_method?, <<~PATTERN
          (send nil? :bad_method ...)
        PATTERN

        # Called on every `send` node (method call) while walking the AST.
        # TODO: remove this method if inspecting `send` nodes is unneeded for your cop.
        # By default, this is aliased to `on_csend` as well to handle method calls
        # with safe navigation, remove the alias if this is unnecessary.
        # If kept, ensure your tests cover safe navigation as well!
        def on_send(node)
          return unless bad_method?(node)

          add_offense(node)
        end
        alias on_csend on_send
      end
    end
  end
end
"#;

const SPEC_TEMPLATE: &str = r#"# frozen_string_literal: true

RSpec.describe RuboCop::Cop::%<department>s::%<cop_name>s, :config do
  let(:config) { RuboCop::Config.new }

  # TODO: Write test code
  #
  # For example
  it 'registers an offense when using `#bad_method`' do
    expect_offense(<<~RUBY)
      bad_method
      ^^^^^^^^^^ Use `#good_method` instead of `#bad_method`.
    RUBY
  end

  it 'does not register an offense when using `#good_method`' do
    expect_no_offenses(<<~RUBY)
      good_method
    RUBY
  end
end
"#;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct Generator {
    badge: Badge,
    output: RefCell<Vec<String>>,
}

impl Generator {
    pub(crate) fn initialize(name: &str) -> Result<Self, String> {
        let badge = Badge::parse(name);
        if !badge.qualified() {
            return Err("Specify a cop name with Department/Name style".into());
        }
        Ok(Self {
            badge,
            output: RefCell::new(Vec::new()),
        })
    }

    pub(crate) fn badge(&self) -> &Badge {
        &self.badge
    }

    pub(crate) fn output(&self) -> Ref<'_, Vec<String>> {
        self.output.borrow()
    }

    pub(crate) fn write_source(&self, root: &Path) -> io::Result<PathBuf> {
        self.write_unless_file_exists(root, &self.source_path(), &self.generated_source())
    }

    pub(crate) fn write_spec(&self, root: &Path) -> io::Result<PathBuf> {
        self.write_unless_file_exists(root, &self.spec_path(), &self.generated_spec())
    }

    pub(crate) fn inject_require(&self, root_source: &str) -> String {
        let require = format!(
            "require_relative '{}'",
            self.source_path()
                .trim_start_matches("lib/")
                .trim_end_matches(".rb")
        );
        if root_source.lines().any(|line| line == require) {
            return root_source.to_owned();
        }

        let mut lines = root_source.lines().map(str::to_owned).collect::<Vec<_>>();
        let prefix = format!(
            "require_relative '{}/",
            self.source_path()
                .trim_start_matches("lib/")
                .rsplit_once('/')
                .map_or("", |(directory, _)| directory)
        );
        let same_department = lines
            .iter()
            .enumerate()
            .filter(|(_, line)| line.starts_with(&prefix))
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        let insertion = same_department
            .iter()
            .copied()
            .find(|index| lines[*index].as_str() > require.as_str())
            .or_else(|| same_department.last().map(|last| last + 1))
            .unwrap_or(lines.len());
        lines.insert(insertion, require);
        let mut result = lines.join("\n");
        if root_source.ends_with('\n') {
            result.push('\n');
        }
        result
    }

    pub(crate) fn inject_config(&self, source: &str, version_added: &str) -> String {
        let entry = format!(
            "{}:\n  Description: 'TODO: Write a description of the cop.'\n  Enabled: pending\n  VersionAdded: '{version_added}'",
            self.badge
        );
        let mut sections = source
            .trim_end()
            .split("\n\n")
            .map(str::to_owned)
            .collect::<Vec<_>>();
        sections.push(entry);
        sections.sort_by(|left, right| {
            left.lines()
                .next()
                .unwrap_or_default()
                .cmp(right.lines().next().unwrap_or_default())
        });
        format!("{}\n", sections.join("\n\n"))
    }

    pub(crate) fn todo(&self) -> String {
        format!(
            "Do 4 steps:\n  1. Modify the description of {} in config/default.yml\n  2. Implement your new cop in the generated file!\n  3. Commit your new cop with a message such as\n     e.g. \"Add new `{}` cop\"\n  4. Run `bundle exec rake changelog:new` to generate a changelog entry\n     for your new cop.\n",
            self.badge, self.badge
        )
    }

    fn write_unless_file_exists(
        &self,
        root: &Path,
        relative_path: &str,
        contents: &str,
    ) -> io::Result<PathBuf> {
        let path = root.join(relative_path);
        if path.exists() {
            return Err(io::Error::new(
                io::ErrorKind::AlreadyExists,
                format!("rake new_cop: {relative_path} already exists!"),
            ));
        }
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(&path, contents)?;
        self.output
            .borrow_mut()
            .push(format!("[create] {}", path.display()));
        Ok(path)
    }

    fn generated_source(&self) -> String {
        self.generate(SOURCE_TEMPLATE)
    }

    fn generated_spec(&self) -> String {
        self.generate(SPEC_TEMPLATE)
    }

    fn generate(&self, template: &str) -> String {
        template
            .replace(
                "%<department>s",
                &self
                    .badge
                    .department()
                    .unwrap_or_default()
                    .replace('/', "::"),
            )
            .replace("%<cop_name>s", self.badge.cop_name())
    }

    fn spec_path(&self) -> String {
        format!(
            "spec/rubocop/cop/{}/{}_spec.rb",
            Self::snake_case(self.badge.department().unwrap_or_default()),
            Self::snake_case(self.badge.cop_name())
        )
    }

    fn source_path(&self) -> String {
        format!(
            "lib/rubocop/cop/{}/{}.rb",
            Self::snake_case(self.badge.department().unwrap_or_default()),
            Self::snake_case(self.badge.cop_name())
        )
    }

    fn snake_case(camel_case_string: &str) -> String {
        let value = camel_case_string.replace("RSpec", "Rspec");
        let value = Regex::new(r"([^A-Z/])([A-Z]+)")
            .expect("static regex")
            .replace_all(&value, "${1}_${2}");
        Regex::new(r"([A-Z])([A-Z][^A-Z\d/]+)")
            .expect("static regex")
            .replace_all(&value, "${1}_${2}")
            .to_ascii_lowercase()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ports_paths_templates_configuration_and_todo() {
        let generator = Generator::initialize("Style/FakeCop").unwrap();
        assert_eq!(generator.source_path(), "lib/rubocop/cop/style/fake_cop.rb");
        assert_eq!(
            generator.spec_path(),
            "spec/rubocop/cop/style/fake_cop_spec.rb"
        );
        assert!(generator
            .generated_source()
            .contains("class FakeCop < Base"));
        assert!(generator
            .generated_spec()
            .contains("RuboCop::Cop::Style::FakeCop"));
        assert!(generator
            .todo()
            .contains("Modify the description of Style/FakeCop"));
        assert!(Generator::initialize("FakeCop").is_err());
        assert_eq!(Generator::snake_case("RSpecFoo/Bar"), "rspec_foo/bar");

        let config = "Style/Alias:\n  Enabled: true\n\nStyle/Lambda:\n  Enabled: true\n";
        let injected = generator.inject_config(config, "<<next>>");
        assert!(injected.find("Style/Alias:").unwrap() < injected.find("Style/FakeCop:").unwrap());
        assert!(injected.find("Style/FakeCop:").unwrap() < injected.find("Style/Lambda:").unwrap());
    }

    #[test]
    fn ports_require_insertion_and_filesystem_refusal() {
        let generator = Generator::initialize("Style/FakeCop").unwrap();
        let root = "require_relative 'rubocop/cop/style/end_block'\nrequire_relative 'rubocop/cop/style/file_name'\nrequire_relative 'rubocop/cop/team'\n";
        let injected = generator.inject_require(root);
        assert!(injected.contains("require_relative 'rubocop/cop/style/fake_cop'"));
        assert!(injected.find("fake_cop").unwrap() < injected.find("file_name").unwrap());
        assert_eq!(generator.inject_require(&injected), injected);

        let directory = std::env::temp_dir().join(format!(
            "rustocop-generator-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        if directory.exists() {
            fs::remove_dir_all(&directory).unwrap();
        }
        generator.write_source(&directory).unwrap();
        assert_eq!(
            generator.write_source(&directory).unwrap_err().kind(),
            io::ErrorKind::AlreadyExists
        );
        fs::remove_dir_all(directory).unwrap();
    }
}
