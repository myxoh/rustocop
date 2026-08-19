# frozen_string_literal: true

require "json"
require "open3"
require "time"
require_relative "support/benchmark"

extend BenchmarkSupport

root = File.expand_path("..", __dir__)
native = File.join(root, "libexec/rustocop-native")
output_root = performance_output_root(root)
status = Rustocop::CompatibilityStatus.load(root: root)
all_cops = status.built_in_cops.sort
pinned_cops, pinned_paths = benchmark_corpus(root)
chatwoot_root = Dir[File.join(root, "tmp/project-benchmarks/corpora/chatwoot-*")].first
abort "cached Chatwoot corpus is required; run script/benchmark_projects.rb first" unless chatwoot_root

real_paths = Dir[File.join(chatwoot_root, "**/*.rb")].sort_by { |path| -File.size(path) }
workloads = {
  "tiny_500" => { "cops" => pinned_cops, "paths" => pinned_paths },
  "largest_real_file" => { "cops" => all_cops, "paths" => real_paths.first(1) },
  "largest_100_real_files" => { "cops" => all_cops, "paths" => real_paths.first(100) },
  "chatwoot_all" => { "cops" => all_cops, "paths" => real_paths }
}
variants = {
  "sequential" => ["--no-parallel"],
  "file_parallel" => ["--parallel"],
  "cop_jobs_2" => ["--cop-jobs", "2"],
  "cop_jobs_4" => ["--cop-jobs", "4"],
  "cop_jobs_8" => ["--cop-jobs", "8"],
  "cop_automatic" => ["--parallel-cops"]
}
runs = Integer(ENV.fetch("COP_PARALLEL_POC_RUNS", "7"))
config = File.join(root, "benchmark/project-rubocop.yml")

def output(command)
  stdout, stderr, status = Open3.capture3(*command)
  raise "command failed with #{status.exitstatus}: #{stderr}" unless [0, 1].include?(status.exitstatus)
  raise "command wrote to stderr: #{stderr}" unless stderr.empty?

  stdout
end

results = workloads.map do |name, workload|
  warn "Benchmarking #{name} (#{workload.fetch("paths").length} files)..."
  common = [
    native, "--format", "json", "--config", config,
    "--only", workload.fetch("cops").join(","), *workload.fetch("paths")
  ]
  commands = variants.transform_values { |arguments| [*common, *arguments] }
  expected = output(commands.fetch("sequential"))
  commands.each do |variant, command|
    raise "output mismatch for #{name}/#{variant}" unless output(command) == expected
  end
  commands.each_value { |command| duration(command) }
  samples = variants.to_h { |variant, _arguments| [variant, []] }
  runs.times do |iteration|
    variants.keys.rotate(iteration % variants.length).each do |variant|
      samples.fetch(variant) << duration(commands.fetch(variant))
    end
  end
  medians = samples.transform_values { |values| percentile(values, 0.5) }
  baseline = medians.fetch("sequential")
  {
    "workload" => name,
    "files" => workload.fetch("paths").length,
    "bytes" => workload.fetch("paths").sum { |path| File.size(path) },
    "cops" => workload.fetch("cops").length,
    "runs" => runs,
    "verified_equal" => true,
    "variants" => samples.to_h do |variant, values|
      median = medians.fetch(variant)
      [
        variant,
        {
          "median_seconds" => median,
          "p95_seconds" => percentile(values, 0.95),
          "speedup_vs_sequential" => baseline.fdiv(median)
        }
      ]
    end
  }
end

report = {
  "generated_at" => Time.now.iso8601,
  "design" => "source cops parallel with single-threaded Prism parse and AST traversal",
  "results" => results
}
json_path = File.join(output_root, "cop-parallel-poc.json")
File.write(json_path, JSON.pretty_generate(report))

header = ["workload", "files", "bytes", "cops", *variants.keys]
puts header.join("\t")
results.each do |result|
  values = result.fetch("variants").map do |_name, measurement|
    format("%.2fms/%.2fx", measurement.fetch("median_seconds") * 1000,
      measurement.fetch("speedup_vs_sequential"))
  end
  puts [result.fetch("workload"), result.fetch("files"), result.fetch("bytes"), result.fetch("cops"), *values].join("\t")
end
puts "Report: #{json_path}"
