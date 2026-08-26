# frozen_string_literal: true

require "json"

module Rustocop
  module BatchedNativeReports
    module_function

    def capture(cops:, batch_size:, run:)
      reports = []
      encoded_offenses = []
      stderr = +""
      seconds = 0.0
      cache_hits = 0
      cache_misses = 0
      cops.each_slice(batch_size) do |batch|
        result = run.call(batch)
        seconds += result.fetch("seconds")
        cache_hits += 1 if result["cache_hit"] == true
        cache_misses += 1 if result["cache_hit"] == false
        stderr << result.fetch("stderr")
        return result.merge("failed_cops" => batch) unless accepted?(result)

        begin
          if result["encoded_offenses"]
            encoded_offenses << result.fetch("encoded_offenses")
          else
            reports << (result["report"] || JSON.parse(result.fetch("stdout")))
          end
        rescue JSON::ParserError => e
          return result.merge(
            "exitstatus" => 2,
            "stderr" => "#{stderr}invalid Rustocop JSON for #{batch.join(', ')}: #{e.message}\n",
            "failed_cops" => batch
          )
        end
      end

      report = merge(reports) unless reports.empty?
      offense_count = encoded_offenses.sum { |encoded| encoded.fetch("offenses").length }
      offense_count += report.fetch("summary").fetch("offense_count") if report
      result = {
        "stdout" => "batched Rustocop report",
        "stderr" => stderr,
        "exitstatus" => offense_count.zero? ? 0 : 1,
        "seconds" => seconds,
        "cache_hits" => cache_hits,
        "cache_misses" => cache_misses
      }
      result["report"] = report if report
      result["encoded_offenses"] = encoded_offenses unless encoded_offenses.empty?
      result
    end

    def merge(reports)
      files_by_path = {}
      file_order = []
      reports.each do |report|
        report.fetch("files").each do |file|
          path = file.fetch("path")
          unless files_by_path.key?(path)
            files_by_path[path] = { "path" => path, "offenses" => [] }
            file_order << path
          end
          files_by_path.fetch(path).fetch("offenses").concat(file.fetch("offenses"))
        end
      end

      first = reports.fetch(0)
      {
        "metadata" => first.fetch("metadata"),
        "files" => file_order.map { |path| files_by_path.fetch(path) },
        "summary" => first.fetch("summary").merge(
          "offense_count" => files_by_path.sum { |_path, file| file.fetch("offenses").length }
        )
      }
    end

    def accepted?(result)
      [0, 1].include?(result.fetch("exitstatus")) &&
        !result.fetch("stdout").empty? &&
        !result.fetch("stderr").include?("An error occurred while")
    end
    private_class_method :accepted?
  end
end
