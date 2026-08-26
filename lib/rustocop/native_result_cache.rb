# frozen_string_literal: true

require "digest"
require "json"
require_relative "artifact_store"

module Rustocop
  class NativeResultCache
    FORMAT_VERSION = 1

    def initialize(root:)
      @root = root
    end

    def fetch(metadata)
      path = path_for(metadata)
      return unless File.file?(path)

      started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      payload = ArtifactStore.read_gzip_json(path, label: "native parity cache")
      return unless payload["format_version"] == FORMAT_VERSION && payload["metadata"] == metadata

      {
        "stdout" => "cached Rustocop report",
        "stderr" => payload.fetch("stderr"),
        "exitstatus" => payload.fetch("exitstatus"),
        "seconds" => Process.clock_gettime(Process::CLOCK_MONOTONIC) - started,
        "report" => payload.fetch("report"),
        "cache_hit" => true
      }
    rescue ArtifactStore::Error, KeyError
      nil
    end

    def store(metadata, result)
      report = result["report"] || JSON.parse(result.fetch("stdout"))
      compact = report.merge(
        "files" => report.fetch("files").reject { |file| file.fetch("offenses").empty? }
      )
      ArtifactStore.write_gzip_json(
        path_for(metadata),
        {
          "format_version" => FORMAT_VERSION,
          "metadata" => metadata,
          "stderr" => result.fetch("stderr"),
          "exitstatus" => result.fetch("exitstatus"),
          "compute_seconds" => result.fetch("seconds"),
          "report" => compact
        },
        compression: Zlib::BEST_SPEED
      )
      result.merge("report" => report, "cache_hit" => false)
    rescue JSON::ParserError, KeyError
      result.merge("cache_hit" => false)
    end

    private

    def path_for(metadata)
      digest = Digest::SHA256.hexdigest(JSON.generate(metadata))
      File.join(@root, digest[0, 2], "#{digest}.json.gz")
    end
  end
end
