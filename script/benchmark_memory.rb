# frozen_string_literal: true

require "json"
require "open3"
require "rbconfig"
require "shellwords"
require "time"
require_relative "support/benchmark"

extend BenchmarkSupport

root = File.expand_path("..", __dir__)
output_root = performance_output_root(root)

abort "memory benchmark currently requires macOS /usr/bin/time -l" unless RbConfig::CONFIG.fetch("host_os").include?("darwin")

cops, paths = compatibility_corpus(root)
config_path = prism_config(output_root)

native = File.join(root, "libexec/rustocop-native")
rubocop = [RbConfig.ruby, Gem.bin_path("rubocop", "rubocop")]
common = ["--cache", "false", "--no-server", "--format", "json", "--only", cops.join(",")]

def normalized_report(command)
  stdout, stderr, status = Open3.capture3(*command)
  raise "command failed (#{status.exitstatus}): #{command.shelljoin}\n#{stderr}" unless [0, 1].include?(status.exitstatus)
  raise "command wrote to stderr: #{command.shelljoin}\n#{stderr}" unless stderr.empty?

  report = JSON.parse(stdout)
  report.fetch("metadata").transform_values! { "normalized" }
  report.fetch("files").each { |file| file["path"] = File.basename(file.fetch("path")) }
  report
end

def peak_rss_bytes(command)
  reader, writer = IO.pipe
  pid = Process.spawn("/usr/bin/time", "-l", *command, out: File::NULL, err: writer)
  writer.close
  accounting = reader.read
  reader.close
  _finished_pid, status = Process.wait2(pid)
  raise "benchmark command failed with #{status.exitstatus}: #{command.shelljoin}" unless [0, 1].include?(status.exitstatus)

  match = accounting.match(/^\s*(\d+)\s+maximum resident set size$/)
  raise "could not read peak RSS from /usr/bin/time output:\n#{accounting}" unless match

  Integer(match[1])
end

sizes = [1, 25, 100, 500]
runs = 7
results = sizes.map do |size|
  selected = paths.first(size)
  commands = {
    "rustocop" => [native, *common, "--config", config_path, *selected],
    "rustocop_parallel" => [native, *common, "--parallel", "--config", config_path, *selected],
    "rubocop_prism" => [*rubocop, *common, "--config", config_path, *selected]
  }

  expected = normalized_report(commands.fetch("rustocop"))
  commands.each do |name, command|
    raise "output mismatch for #{name} at #{size} files" unless normalized_report(command) == expected
  end

  commands.each_value { |command| peak_rss_bytes(command) }
  samples = commands.to_h { |name, _command| [name, []] }
  runs.times do |iteration|
    commands.keys.rotate(iteration % commands.length).each do |name|
      samples.fetch(name) << peak_rss_bytes(commands.fetch(name))
    end
  end

  medians = samples.transform_values { |values| percentile(values, 0.5) }
  {
    "files" => size,
    "runs" => runs,
    "verified_equal" => true,
    "rustocop" => {
      "median_peak_rss_bytes" => medians.fetch("rustocop"),
      "p95_peak_rss_bytes" => percentile(samples.fetch("rustocop"), 0.95)
    },
    "rustocop_parallel" => {
      "median_peak_rss_bytes" => medians.fetch("rustocop_parallel"),
      "p95_peak_rss_bytes" => percentile(samples.fetch("rustocop_parallel"), 0.95)
    },
    "rubocop_prism" => {
      "median_peak_rss_bytes" => medians.fetch("rubocop_prism"),
      "p95_peak_rss_bytes" => percentile(samples.fetch("rubocop_prism"), 0.95)
    },
    "rss_ratio" => medians.fetch("rubocop_prism").fdiv(medians.fetch("rustocop"))
  }
end

report = {
  "generated_at" => Time.now.iso8601,
  "scope" => {
    "rubocop_version" => "1.87.0",
    "ruby_version" => RUBY_VERSION,
    "target_ruby_version" => "3.4",
    "parser_engine" => "parser_prism",
    "cops" => cops,
    "corpus_files" => paths.length,
    "cache" => false,
    "server" => false,
    "formatter" => "json",
    "process_models" => ["single process, sequential files", "single process, file worker threads"],
    "measurement" => "macOS /usr/bin/time -l maximum resident set size"
  },
  "results" => results
}

json_path = File.join(output_root, "memory-benchmark.json")
File.write(json_path, JSON.pretty_generate(report))

puts "files\trustocop_mib\tparallel_mib\trubocop_prism_mib\tratio\tverified"
results.each do |result|
  puts format(
    "%d\t%.2f\t%.2f\t%.2f\t%.2fx\t%s",
    result.fetch("files"),
    result.dig("rustocop", "median_peak_rss_bytes").fdiv(1024**2),
    result.dig("rustocop_parallel", "median_peak_rss_bytes").fdiv(1024**2),
    result.dig("rubocop_prism", "median_peak_rss_bytes").fdiv(1024**2),
    result.fetch("rss_ratio"),
    result.fetch("verified_equal")
  )
end
puts "Report: #{json_path}"
