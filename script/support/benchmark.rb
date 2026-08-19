# frozen_string_literal: true

require "shellwords"
require "fileutils"

module BenchmarkSupport
  def performance_output_root(root)
    File.join(root, "tmp/performance-verification").tap { |path| FileUtils.mkdir_p(path) }
  end

  def compatibility_corpus(root, interleaved: true)
    fixture_root = File.join(root, "spec/fixtures/rubocop_builtin_examples")
    manifest = File.readlines(File.join(fixture_root, "manifest.tsv"), chomp: true).drop(1).map do |line|
      directory, cop = line.split("\t", 2)
      [cop, Dir[File.join(fixture_root, directory, "*.rb")].sort]
    end
    cops = manifest.map(&:first)
    groups = manifest.map(&:last)
    paths = if interleaved
              (0...groups.map(&:length).max).flat_map do |index|
                groups.filter_map { |group| group[index] }
              end
            else
              groups.flatten
            end
    raise "expected 500 compatibility files, got #{paths.length}" unless paths.length == 500

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
