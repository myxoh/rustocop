# frozen_string_literal: true

require "set"
require "yaml"

module Rustocop
  class RubocopConfiguration
    WARNING = "Warning - non native cops are ignored by default, to include them use " \
              "--included-non-native-cops NOTE performance is severely degraded when using non native cops."

    RESOLVED_CONFIG_ENV = "RUSTOCOP_RESOLVED_CONFIG_SOURCE"

    Resolution = Struct.new(:arguments, :environment, :warn_about_non_native_cops, keyword_init: true)

    OPTIONS_WITH_VALUES = %w[
      --cache
      --config
      --format
      --jobs
      --only
      --plugin
      --require
      --stdin
      -c
      -f
    ].freeze

    def self.resolve(arguments)
      new(arguments).resolve
    end

    def initialize(arguments)
      @arguments = arguments.dup
    end

    def resolve
      return unchanged if metadata_command? || explicit_custom_cop_selection?
      return unchanged if explicit_cop_selection? && !configuration_present?

      require "rubocop"

      base_cops = RuboCop::Cop::Registry.global.names.to_set
      config = effective_config
      resolved_arguments = arguments.dup
      add_discovered_config(resolved_arguments, config)

      non_native_cops = []
      unless explicit_cop_selection?
        enabled_cops = RuboCop::Cop::Registry.global.enabled(config).map(&:cop_name)
        base_enabled_cops, non_native_cops = enabled_cops.partition { |name| base_cops.include?(name) }
        resolved_arguments << "--resolved-enabled-cops=#{base_enabled_cops.sort.join(',')}"
        resolved_arguments << "--resolved-non-native-cops=#{non_native_cops.sort.join(',')}"
      end

      Resolution.new(
        arguments: resolved_arguments,
        environment: { RESOLVED_CONFIG_ENV => resolved_config_source(config, base_cops) },
        warn_about_non_native_cops: !explicit_cop_selection? &&
          non_native_cops.any? && !include_non_native_cops?
      )
    rescue LoadError => e
      raise "RuboCop is required to resolve project configuration: #{e.message}"
    rescue RuboCop::Error, Psych::SyntaxError => e
      raise "could not resolve RuboCop configuration: #{e.message}"
    end

    private

    attr_reader :arguments

    def unchanged
      Resolution.new(arguments:, environment: {}, warn_about_non_native_cops: false)
    end

    def metadata_command?
      arguments.any? { |argument| %w[--version -V --show-cops].include?(argument) }
    end

    def explicit_cop_selection?
      arguments.any? { |argument| argument == "--only" || argument.start_with?("--only=") }
    end

    def explicit_custom_cop_selection?
      explicit_cop_selection? && arguments.any? do |argument|
        %w[--require --plugin].include?(argument) ||
          argument.start_with?("--require=", "--plugin=")
      end
    end

    def include_non_native_cops?
      arguments.include?("--included-non-native-cops")
    end

    def configuration_present?
      return true if explicit_config_path

      directory = target_directory
      loop do
        return true if config_candidates(directory).any? { |path| File.file?(path) }

        parent = File.dirname(directory)
        break if parent == directory

        directory = parent
      end

      user_config_candidates.any? { |path| File.file?(path) }
    end

    def config_candidates(directory)
      [
        File.join(directory, ".rubocop.yml"),
        File.join(directory, ".config", ".rubocop.yml"),
        File.join(directory, ".config", "rubocop", "config.yml")
      ]
    end

    def user_config_candidates
      home = ENV["HOME"]
      xdg = ENV["XDG_CONFIG_HOME"] || (File.join(home, ".config") if home)
      [
        (File.join(home, ".rubocop.yml") if home),
        (File.join(xdg, "rubocop", "config.yml") if xdg)
      ].compact
    end

    def effective_config
      store = RuboCop::ConfigStore.new
      store.options_config = explicit_config_path if explicit_config_path
      store.for_dir(target_directory)
    end

    def explicit_config_path
      return @explicit_config_path if defined?(@explicit_config_path)

      option_index = arguments.index { |argument| %w[--config -c].include?(argument) }
      inline = arguments.find { |argument| argument.start_with?("--config=") }
      @explicit_config_path = option_index ? arguments[option_index + 1] : inline&.delete_prefix("--config=")
    end

    def target_directory
      target = target_paths.first
      return Dir.pwd unless target

      expanded = File.expand_path(target)
      File.directory?(expanded) ? expanded : File.dirname(expanded)
    end

    def target_paths
      paths = []
      skip_value = false
      after_separator = false

      arguments.each do |argument|
        if after_separator
          paths << argument
        elsif skip_value
          skip_value = false
        elsif argument == "--"
          after_separator = true
        elsif OPTIONS_WITH_VALUES.include?(argument)
          skip_value = true
        elsif argument.start_with?("-")
          next
        else
          paths << argument
        end
      end
      paths
    end

    def add_discovered_config(resolved_arguments, config)
      return if explicit_config_path

      loaded_path = config.loaded_path
      return unless loaded_path && File.file?(loaded_path)
      return if File.basename(loaded_path) == "default.yml" && loaded_path.include?("/rubocop-")

      resolved_arguments << "--config=#{loaded_path}"
    end

    def resolved_config_source(config, base_cops)
      defaults = RuboCop::ConfigLoader.default_configuration.to_h
      overrides = config.to_h.each_with_object({}) do |(name, settings), result|
        next unless name == "AllCops" || base_cops.include?(name)
        next unless settings.is_a?(Hash)

        default_settings = defaults.fetch(name, {})
        changed = settings.reject { |key, value| default_settings[key] == value }
        result[name] = changed unless changed.empty?
      end
      YAML.dump(overrides)
    end
  end
end
