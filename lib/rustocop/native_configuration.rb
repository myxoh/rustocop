# frozen_string_literal: true

module Rustocop
  class NativeConfiguration
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

    CANDIDATES = [
      ".rustocop.yml",
      File.join(".config", "rustocop", "config.yml"),
      "rustocop.yml"
    ].freeze

    def self.arguments(arguments)
      new(arguments).arguments
    end

    def initialize(arguments)
      @arguments = arguments.dup
    end

    def arguments
      return if @arguments.include?("--included-non-native-cops")
      return if explicit_ruby_loaders?

      if (explicit = explicit_config_path)
        return @arguments if rustocop_config?(explicit)
        return
      end

      path = discovered_config_path
      path ? [*@arguments, "--config=#{path}"] : nil
    end

    private

    def explicit_config_path
      index = @arguments.index { |argument| %w[--config -c].include?(argument) }
      inline = @arguments.find { |argument| argument.start_with?("--config=") }
      index ? @arguments[index + 1] : inline&.delete_prefix("--config=")
    end

    def explicit_ruby_loaders?
      @arguments.any? do |argument|
        %w[--require --plugin].include?(argument) || argument.start_with?("--require=", "--plugin=")
      end
    end

    def discovered_config_path
      directory = target_directory
      loop do
        CANDIDATES.each do |candidate|
          path = File.join(directory, candidate)
          return path if File.file?(path)
        end
        parent = File.dirname(directory)
        break if parent == directory

        directory = parent
      end
      nil
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
      @arguments.each do |argument|
        if after_separator
          paths << argument
        elsif skip_value
          skip_value = false
        elsif argument == "--"
          after_separator = true
        elsif OPTIONS_WITH_VALUES.include?(argument)
          skip_value = true
        elsif !argument.start_with?("-")
          paths << argument
        end
      end
      paths
    end

    def rustocop_config?(path)
      basename = File.basename(path)
      normalized = File.expand_path(path).tr("\\", "/")
      (basename.include?("rustocop") && !basename.include?("rubocop")) ||
        normalized.end_with?("/.config/rustocop/config.yml")
    end
  end
end
