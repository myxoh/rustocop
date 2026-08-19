# frozen_string_literal: true

module Rustocop
  module GeneratedSection
    module_function

    def replace(path, name, body)
      start_marker = "<!-- generated:#{name}:start -->"
      end_marker = "<!-- generated:#{name}:end -->"
      source = File.read(path)
      pattern = /#{Regexp.escape(start_marker)}.*?#{Regexp.escape(end_marker)}/m
      raise "generated section #{name.inspect} missing from #{path}" unless source.match?(pattern)

      replacement = "#{start_marker}\n#{body.rstrip}\n#{end_marker}"
      File.write(path, source.sub(pattern, replacement))
    end
  end
end
