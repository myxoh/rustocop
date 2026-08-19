# frozen_string_literal: true

require "shellwords"
require "fileutils"
require "json"
require_relative "../../lib/rustocop/compatibility_status"

module BenchmarkSupport
  def performance_output_root(root)
    File.join(root, "tmp/performance-verification").tap { |path| FileUtils.mkdir_p(path) }
  end

  def benchmark_corpus(root, interleaved: true)
    manifest = JSON.parse(File.read(File.join(root, "benchmark/corpus.json")))
    raise "unsupported benchmark corpus version" unless manifest.fetch("version") == 1

    corpus_root = File.join(performance_output_root(root), "benchmark-corpus")
    groups = manifest.fetch("groups").each_with_index.map do |group, group_index|
      paths = group.fetch("sources").each_with_index.map do |source, source_index|
        path = File.join(corpus_root, format("%02d", group_index), format("%02d.rb", source_index))
        FileUtils.mkdir_p(File.dirname(path))
        File.write(path, source) unless File.file?(path) && File.binread(path) == source
        path
      end
      [group.fetch("cop"), paths]
    end
    cops = groups.map(&:first)
    file_groups = groups.map(&:last)
    paths = if interleaved
              (0...file_groups.map(&:length).max).flat_map do |index|
                file_groups.filter_map { |group| group[index] }
              end
            else
              file_groups.flatten
            end
    raise "expected 20 benchmark cops, got #{cops.length}" unless cops.length == 20
    raise "expected 500 benchmark files, got #{paths.length}" unless paths.length == 500
    Rustocop::CompatibilityStatus.load(root: root).validate_verified!(cops, label: "benchmark corpus")

    [cops, paths]
  end

  def prism_config(output_root)
    File.join(output_root, "rubocop-prism.yml").tap do |path|
      File.write(path, <<~YAML)
        AllCops:
          ParserEngine: parser_prism
          TargetRubyVersion: 3.4
          NewCops: enable
      YAML
    end
  end

  def duration(command)
    started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
    pid = Process.spawn(*command, out: File::NULL, err: File::NULL)
    _finished_pid, status = Process.wait2(pid)
    unless [0, 1].include?(status.exitstatus)
      raise "benchmark command failed with #{status.exitstatus}: #{command.shelljoin}"
    end

    Process.clock_gettime(Process::CLOCK_MONOTONIC) - started
  end

  def percentile(values, fraction)
    raise ArgumentError, "cannot take a percentile of no samples" if values.empty?

    sorted = values.sort
    sorted[((sorted.length - 1) * fraction).round]
  end
end
