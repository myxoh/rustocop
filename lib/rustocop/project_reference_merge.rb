# frozen_string_literal: true

require "time"

module Rustocop
  module ProjectReferenceMerge
    class Error < StandardError; end

    module_function

    METADATA_KEYS = %w[version kind rubocop_version config_sha256 project_revisions].freeze

    def merge(references)
      raise Error, "at least two references are required" if references.length < 2

      baseline = references.first
      references.drop(1).each do |reference|
        METADATA_KEYS.each do |key|
          next if reference.fetch(key) == baseline.fetch(key)

          raise Error, "reference metadata differs for #{key}"
        end
      end

      cop_sets = references.map { |reference| reference.fetch("cops") }
      duplicates = cop_sets.flatten.tally.select { |_cop, count| count > 1 }.keys
      raise Error, "references overlap on cops: #{duplicates.sort.join(', ')}" unless duplicates.empty?

      project_names = baseline.fetch("projects").keys
      unless references.all? { |reference| reference.fetch("projects").keys == project_names }
        raise Error, "reference project sets or ordering differ"
      end

      cops = cop_sets.flatten.sort
      projects = project_names.to_h do |name|
        parts = references.map { |reference| reference.fetch("projects").fetch(name) }
        files = parts.map { |part| part.fetch("files") }.uniq
        raise Error, "reference file counts differ for #{name}" unless files.one?

        offenses = references.flat_map.with_index do |reference, index|
          decode_offenses(parts.fetch(index), reference.fetch("cops"))
        end
        [name, encode_project(parts, offenses, cops)]
      end

      baseline.slice(*METADATA_KEYS).merge(
        "generated_at" => Time.now.iso8601,
        "cops" => cops,
        "rubocop_errors" => references.flat_map { |reference| reference.fetch("rubocop_errors") }.uniq,
        "projects" => projects
      )
    end

    def decode_offenses(project, cops)
      paths = project.fetch("paths")
      messages = project.fetch("messages")
      project.fetch("offenses").map do |row|
        path_index, cop_index, severity, message_index, *position = row
        [paths.fetch(path_index), cops.fetch(cop_index), severity, messages.fetch(message_index), *position]
      end
    end

    def encode_project(parts, offenses, cops)
      offenses.sort_by! { |row| [row[0], row[4], row[5], row[1], row[3]] }
      paths = offenses.map(&:first).uniq
      messages = offenses.map { |row| row[3] }.uniq
      path_indexes = paths.each_with_index.to_h
      cop_indexes = cops.each_with_index.to_h
      message_indexes = messages.each_with_index.to_h
      rows = offenses.map do |path, cop, severity, message, *position|
        [path_indexes.fetch(path), cop_indexes.fetch(cop), severity, message_indexes.fetch(message), *position]
      end
      {
        "files" => parts.first.fetch("files"),
        "seconds" => parts.sum { |part| part.fetch("seconds") },
        "warning_count" => parts.sum { |part| part.fetch("warning_count") },
        "paths" => paths,
        "messages" => messages,
        "offenses" => rows
      }
    end
    private_class_method :decode_offenses, :encode_project
  end
end
