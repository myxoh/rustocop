# frozen_string_literal: true

require 'json'
require 'time'
require_relative 'support/benchmark'

extend BenchmarkSupport

root = File.expand_path('..', __dir__)
output_root = performance_output_root(root)
cops, paths = benchmark_corpus(root, interleaved: false)

native = File.join(root, 'libexec/rustocop-native')
variants = {
  'none' => ['NoSuch/Cop'],
  'one' => cops.first(1),
  'five' => cops.first(5),
  'twenty' => cops
}
runs = 30

commands = variants.transform_values do |selected_cops|
  [native, '--no-parallel', '--format', 'json', '--only', selected_cops.join(','), *paths]
end
commands.each_value { |command| duration(command) }
samples = variants.to_h { |name, _cops| [name, []] }
runs.times do |iteration|
  variants.keys.rotate(iteration % variants.length).each do |name|
    samples.fetch(name) << duration(commands.fetch(name))
  end
end

results = variants.map do |name, selected_cops|
  values = samples.fetch(name)
  {
    'variant' => name,
    'enabled_cops' => name == 'none' ? 0 : selected_cops.length,
    'median_seconds' => percentile(values, 0.5),
    'p95_seconds' => percentile(values, 0.95)
  }
end
report = {
  'generated_at' => Time.now.iso8601,
  'scope' => { 'corpus_files' => paths.length, 'runs' => runs, 'formatter' => 'json' },
  'results' => results
}
json_path = File.join(output_root, 'cop-scaling-benchmark.json')
File.write(json_path, JSON.pretty_generate(report))

puts "enabled_cops\tmedian_ms\tp95_ms"
results.each do |result|
  puts [
    result.fetch('enabled_cops'),
    format('%.3f', result.fetch('median_seconds') * 1000),
    format('%.3f', result.fetch('p95_seconds') * 1000)
  ].join("\t")
end
puts "Report: #{json_path}"
