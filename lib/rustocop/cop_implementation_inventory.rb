# frozen_string_literal: true

module Rustocop
  module CopImplementationInventory
    module_function

    def sources(root:)
      cop_root = File.join(root, "crates", "rustocop", "src", "cops")
      Dir[File.join(cop_root, "**", "*.rs")].reject do |path|
        path.include?("/tests/") || path.include?("/framework/") || path.include?("/runtime/")
      end.to_h { |path| [path, File.read(path)] }
    end

    def registration_paths(cop, sources:)
      quoted = Regexp.escape(cop)
      patterns = [
        /=>\s*"#{quoted}"\s*=>/m,
        /(?:custom|report|replace)\(\s*"#{quoted}"/m,
        /fn\s+name\s*\([^)]*\)[^{]*\{\s*"#{quoted}"\s*\}/m,
        /let\s+cop\s*=\s*"#{quoted}"/m
      ]
      paths = sources.filter_map do |path, source|
        path if patterns.any? { |pattern| source.match?(pattern) }
      end
      return paths.sort unless paths.empty?

      literal = %Q{"#{cop}"}
      paths = sources.filter_map { |path, source| path if source.include?(literal) }
      paths.reject! { |path| path.end_with?("/text/mod.rs") }
      text_paths = paths.select { |path| path.include?("/cops/text/") }
      (text_paths.empty? ? paths : text_paths).sort
    end
  end
end
