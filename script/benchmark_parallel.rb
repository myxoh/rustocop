# frozen_string_literal: true

require "json"
require "open3"
require "time"
require_relative "support/benchmark"

extend BenchmarkSupport

root = File.expand_path("..", __dir__)
output_root = performance_output_root(root)
cops, paths = compatibility_corpus(root)

native = File.join(root, "libexec/rustocop-native")
config_path = prism_config(output_root)
common = ["--cache", "false", "--no-server", "--format", "json", "--only", cops.join(",")]
variants = {
  "sequential" => [],
  "jobs_2" => ["--jobs", "2"],
  "jobs_4" => ["--jobs", "4"],
  "jobs_8" => ["--jobs", "8"],
  "automatic" => ["--parallel"]
}

def command_output(command)
  stdout, stderr, status = Open3.capture3(*command)
  raise "command failed with #{status.exitstatus}: #{stderr}" unless [0, 1].include?(status.exitstatus)
  raise "command wrote to stderr: #{stderr}" unless stderr.empty?

  stdout
end

sizes = [25, 100, 500]
runs = 15
results = sizes.map do |size|
  selected = paths.first(size)
  commands = variants.transform_values do |arguments|
    [native, *common, *arguments, "--config", config_path, *selected]
  end
  expected = command_output(commands.fetch("sequential"))
  commands.each do |name, command|
    raise "parallel output mismatch for #{name} at #{size} files" unless command_output(command) == expected
  end

  commands.each_value { |command| duration(command) }
  samples = variants.to_h { |name, _arguments| [name, []] }
  runs.times do |iteration|
    variants.keys.rotate(iteration % variants.length).each do |name|
      samples.fetch(name) << duration(commands.fetch(name))
    end
  end
  medians = samples.transform_values { |values| percentile(values, 0.5) }
  sequential = medians.fetch("sequential")

  {
    "files" => size,
    "runs" => runs,
    "verified_equal" => true,
    "variants" => samples.to_h do |name, values|
      [
        name,
        {
          "median_seconds" => medians.fetch(name),
          "p95_seconds" => percentile(values, 0.95),
          "speedup_vs_sequential" => sequential.fdiv(medians.fetch(name))
        }
      ]
    end
  }
end

report = {
  "generated_at" => Time.now.iso8601,
  "scope" => {
    "corpus_files" => paths.length,
    "cops" => cops,
    "runs" => runs,
    "cache" => false,
    "server" => false,
    "formatter" => "json"
  },
  "results" => results
}
json_path = File.join(output_root, "parallel-benchmark.json")
File.write(json_path, JSON.pretty_generate(report))

puts "files\tsequential_ms\tjobs_2\tjobs_4\tjobs_8\tautomatic"
results.each do |result|
  row = [result.fetch("files")]
  variants.each_key do |name|
    variant = result.dig("variants", name)
    row << format(
      "%.3fms/%.2fx",
      variant.fetch("median_seconds") * 1000,
      variant.fetch("speedup_vs_sequential")
    )
  end
  puts row.join("\t")
end
puts "Report: #{json_path}"
