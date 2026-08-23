# frozen_string_literal: true

require "fileutils"
require "json"
require "zlib"

module Rustocop
  module ArtifactStore
    class Error < StandardError; end

    module_function

    def read_json(path, label: "JSON artifact")
      JSON.parse(File.read(path))
    rescue Errno::ENOENT
      raise Error, "#{label} not found: #{path}"
    rescue JSON::ParserError => e
      raise Error, "invalid #{label} #{path}: #{e.message}"
    end

    def read_gzip_json(path, label: "compressed JSON artifact")
      Zlib::GzipReader.open(path) { |gzip| JSON.parse(gzip.read) }
    rescue Errno::ENOENT
      raise Error, "#{label} not found: #{path}"
    rescue Zlib::GzipFile::Error, JSON::ParserError => e
      raise Error, "invalid #{label} #{path}: #{e.message}"
    end

    def write_json(path, value, pretty: true, trailing_newline: false)
      atomic_write(path, serialize_json(value, pretty:, trailing_newline:))
    end

    def serialize_json(value, pretty: true, trailing_newline: false)
      content = pretty ? JSON.pretty_generate(value) : JSON.generate(value)
      trailing_newline ? "#{content}\n" : content
    end

    def write_gzip_json(path, value)
      atomic_replace(path) do |temporary|
        Zlib::GzipWriter.open(temporary) do |gzip|
          gzip.mtime = 0
          gzip.write(JSON.generate(value))
        end
      end
    end

    def atomic_write(path, content)
      atomic_replace(path) { |temporary| File.binwrite(temporary, content) }
    end

    def atomic_replace(path)
      FileUtils.mkdir_p(File.dirname(path))
      temporary = "#{path}.#{Process.pid}.tmp"
      yield temporary
      File.rename(temporary, path)
      path
    ensure
      FileUtils.rm_f(temporary) if defined?(temporary)
    end
    private_class_method :atomic_replace
  end
end
