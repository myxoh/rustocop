# frozen_string_literal: true

require "digest"
require "json"
require_relative "artifact_store"

module Rustocop
  class NativeResultCache
    FORMAT_VERSION = 2

    def initialize(root:)
      @root = root
    end

    def cached?(metadata)
      File.file?(path_for(metadata))
    end

    def fetch(metadata)
      path = path_for(metadata)
      return unless File.file?(path)

      started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
      payload = ArtifactStore.read_gzip_json(path, label: "native parity cache")
      return unless [1, FORMAT_VERSION].include?(payload["format_version"]) && payload["metadata"] == metadata

      result = {
        "stdout" => "cached Rustocop report",
        "stderr" => payload.fetch("stderr"),
        "exitstatus" => payload.fetch("exitstatus"),
        "seconds" => Process.clock_gettime(Process::CLOCK_MONOTONIC) - started,
        "cache_hit" => true
      }
      if payload["format_version"] == FORMAT_VERSION
        result.merge("encoded_offenses" => payload.fetch("encoded_offenses"))
      else
        result.merge("report" => payload.fetch("report"))
      end
    rescue ArtifactStore::Error, KeyError
      nil
    end

    def store(metadata, result)
      ArtifactStore.write_gzip_json(
        path_for(metadata),
        {
          "format_version" => FORMAT_VERSION,
          "metadata" => metadata,
          "stderr" => result.fetch("stderr"),
          "exitstatus" => result.fetch("exitstatus"),
          "compute_seconds" => result.fetch("seconds"),
          "encoded_offenses" => result.fetch("encoded_offenses")
        },
        compression: Zlib::BEST_SPEED
      )
      result.merge("cache_hit" => false)
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
