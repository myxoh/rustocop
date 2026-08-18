# frozen_string_literal: true

require 'fileutils'
require 'json'
require 'time'

root = File.expand_path('..', __dir__)
fixture_root = File.join(root, 'spec/fixtures/rubocop_builtin_examples')
output_root = File.join(root, 'tmp/performance-verification')
FileUtils.mkdir_p(output_root)

manifest = File.readlines(File.join(fixture_root, 'manifest.tsv'), chomp: true).drop(1).map do |line|
  directory, cop = line.split("\t", 2)
  [cop, Dir[File.join(fixture_root, directory, '*.rb')].sort]
end
cops = manifest.map(&:first)
paths = manifest.flat_map(&:last)
raise 'expected 500 files' unless paths.length == 500

native = File.join(root, 'libexec/rustocop-native')
variants = {
  'none' => ['NoSuch/Cop'],
  'one' => cops.first(1),
  'five' => cops.first(5),
  'twenty' => cops
}
runs = 30

def duration(command)
  started = Process.clock_gettime(Process::CLOCK_MONOTONIC)
  pid = Process.spawn(*command, out: File::NULL, err: File::NULL)
  _finished_pid, status = Process.wait2(pid)
  raise "benchmark command failed with #{status.exitstatus}" unless [0, 1].include?(status.exitstatus)

  Process.clock_gettime(Process::CLOCK_MONOTONIC) - started
end

def percentile(values, fraction)
  sorted = values.sort
  sorted[((sorted.length - 1) * fraction).round]
end

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
