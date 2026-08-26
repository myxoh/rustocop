# frozen_string_literal: true

require "pathname"

module Rustocop
  module DiagnosticSignatures
    Signature = Data.define(
      :path, :cop, :severity, :message,
      :start_line, :start_column, :last_line, :last_column
    ) do
      def tuple
        members.map { |member| public_send(member) }
      end

      def location_tuple
        tuple.drop(2)
      end

      def to_h
        members.to_h { |member| [member.to_s, public_send(member)] }
      end
    end

    module_function

    def from_report(report, corpus: nil, root: Dir.pwd)
      report.fetch("files").flat_map do |file|
        path = normalize_path(file.fetch("path"), corpus:, root:)
        file.fetch("offenses").map { |offense| signature(path, offense) }
      end
    end

    def for_cop(report, cop, **options)
      from_report(report, **options).select { |signature| signature.cop == cop }
    end

    def hashes_from_report(report, **options)
      from_report(report, **options).map(&:to_h)
    end

    def encode_report(report, cops:, corpus: nil, root: Dir.pwd)
      cop_indexes = cops.each_with_index.to_h
      paths = []
      path_indexes = {}
      messages = []
      message_indexes = {}
      offenses = []

      report.fetch("files").each do |file|
        next if file.fetch("offenses").empty?

        path = normalize_path(file.fetch("path"), corpus:, root:)
        path_index = path_indexes[path] ||= begin
          paths << path
          paths.length - 1
        end
        file.fetch("offenses").each do |offense|
          message = offense.fetch("message")
          message_index = message_indexes[message] ||= begin
            messages << message
            messages.length - 1
          end
          location = offense.fetch("location")
          offenses << [
            path_index,
            cop_indexes.fetch(offense.fetch("cop_name")),
            offense.fetch("severity"),
            message_index,
            location.fetch("start_line"),
            location.fetch("start_column"),
            location.fetch("last_line"),
            location.fetch("last_column")
          ]
        end
      end

      { "paths" => paths, "cops" => cops, "messages" => messages, "offenses" => offenses }
    end

    def hashes_from_encoded(encoded_reports)
      reports = encoded_reports.is_a?(Array) ? encoded_reports : [encoded_reports]
      reports.flat_map do |encoded|
        paths = encoded.fetch("paths")
        cops = encoded.fetch("cops")
        messages = encoded.fetch("messages")
        encoded.fetch("offenses").map do |row|
          {
            "path" => paths.fetch(row.fetch(0)),
            "cop" => cops.fetch(row.fetch(1)),
            "severity" => row.fetch(2),
            "message" => messages.fetch(row.fetch(3)),
            "start_line" => row.fetch(4),
            "start_column" => row.fetch(5),
            "last_line" => row.fetch(6),
            "last_column" => row.fetch(7)
          }
        end
      end
    end

    def tuples_from_report(report, **options)
      from_report(report, **options).map(&:tuple)
    end

    def normalize_path(reported, corpus:, root:)
      return reported unless corpus

      absolute = Pathname(reported).absolute? ? reported : File.expand_path(reported, root)
      Pathname(absolute).relative_path_from(Pathname(corpus)).to_s
    end
    private_class_method :normalize_path

    def signature(path, offense)
      location = offense.fetch("location")
      Signature.new(
        path:,
        cop: offense.fetch("cop_name"),
        severity: offense.fetch("severity"),
        message: offense.fetch("message"),
        start_line: location.fetch("start_line"),
        start_column: location.fetch("start_column"),
        last_line: location.fetch("last_line"),
        last_column: location.fetch("last_column")
      )
    end
    private_class_method :signature
  end
end
