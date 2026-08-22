# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "prism"
require "tmpdir"

require_relative "../lib/rustocop/project_corpus"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("spec/fixtures/project_parity_regressions")
MANIFEST = FIXTURE_ROOT.join("mismatches.tsv")
CONFIG = ROOT.join("benchmark/project-rubocop.yml")
NATIVE = ROOT.join("crates/rustocop/target/release/rustocop")

options = { jobs: 8, refresh_invalid: false }
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/isolate_project_parity_mismatches.rb REPORT [options]"
  parser.on("--jobs COUNT", Integer) { |value| options[:jobs] = value }
  parser.on("--refresh-invalid", "replace syntactically incomplete generated fixtures") do
    options[:refresh_invalid] = true
  end
end.parse!
report_path = ARGV.shift or abort "missing project-parity report"
report = JSON.parse(File.read(report_path))

Offense = Data.define(:severity, :message, :line, :column, :last_line, :last_column)
Candidate = Data.define(:cop, :kind, :project, :example)

def signatures(output, cop)
  offenses = JSON.parse(output).fetch("files", []).flat_map { |file| file.fetch("offenses", []) }
  return if cop != "Lint/Syntax" && offenses.any? { |offense| offense.fetch("cop_name") == "Lint/Syntax" }

  offenses.select { |offense| offense.fetch("cop_name") == cop }
      .map do |offense|
    location = offense.fetch("location")
    Offense.new(
      offense.fetch("severity"), offense.fetch("message"),
      location.fetch("start_line"), location.fetch("start_column"),
      location.fetch("last_line"), location.fetch("last_column")
    )
  end.tally
rescue JSON::ParserError, KeyError
  nil
end

def run_engine(command, cop, path)
  stdout, stderr, status = Open3.capture3(
    *command, "--config", CONFIG.to_s, "--format", "json", "--only", cop, path
  )
  return unless [0, 1].include?(status.exitstatus) && stderr.empty?

  signatures(stdout, cop)
end

def reproduces?(source, cop, kind, relative_path, rubocop, rustocop)
  return false if cop != "Lint/Syntax" && !Prism.parse(source).success?

  Dir.mktmpdir("rustocop-project-isolation-") do |directory|
    path = File.join(directory, relative_path)
    FileUtils.mkdir_p(File.dirname(path))
    File.write(path, source)
    ruby = run_engine(rubocop, cop, path)
    rust = run_engine(rustocop, cop, path)
    return false unless ruby && rust

    left, right = kind == "rustocop_only" ? [rust, ruby] : [ruby, rust]
    left.any? { |signature, count| count > right.fetch(signature, 0) }
  end
end

def source_windows(source, target_line)
  lines = source.lines
  index = [[target_line.to_i - 1, 0].max, lines.length - 1].min
  [0, 1, 2, 4, 8, 16, 32, 64].filter_map do |radius|
    first = [index - radius, 0].max
    last = [index + radius, lines.length - 1].min
    lines[first..last]&.join
  end.uniq << source
end

def slug(cop, kind)
  "pending/#{cop.downcase.tr('/', '_').gsub(/[^a-z0-9_]+/, '_')}_#{kind}.rb"
end

project_metadata = Rustocop::ProjectCorpus::PROJECTS.to_h { |project| [project.fetch("name"), project] }
existing_rows = File.readlines(MANIFEST, chomp: true).drop(1).map { |line| line.split("\t", 7) }
if options[:refresh_invalid]
  existing_rows.reject! do |row|
    cop, file = row.values_at(0, 1)
    next false if cop == "Lint/Syntax"

    path = FIXTURE_ROOT.join(file)
    invalid = path.file? && !Prism.parse(path.binread).success?
    FileUtils.rm_f(path) if invalid && path.to_s.start_with?(FIXTURE_ROOT.join("pending").to_s)
    invalid
  end
end
existing = existing_rows.to_h { |row| [[row.fetch(0), row.fetch(5)], true] }

candidates = Hash.new { |hash, key| hash[key] = [] }
report.fetch("projects").each do |project_name, project_result|
  project_result.fetch("by_cop").each do |cop, result|
    %w[rustocop_only rubocop_only].each do |kind|
      key = [cop, kind]
      next if existing.key?(key)

      result.fetch("#{kind}_examples").each do |example|
        candidates[key] << Candidate.new(cop, kind, project_name, example)
      end
    end
  end
end

rubocop = [
  Gem.ruby,
  Gem.bin_path("rubocop", "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}"),
  "--no-server", "--cache", "false"
].freeze
rustocop = [NATIVE.to_s].freeze
abort "native binary not found: #{NATIVE}" unless NATIVE.executable?

queue = Queue.new
candidates.sort.each { |item| queue << item }
results = []
mutex = Mutex.new
workers = Array.new([options.fetch(:jobs), candidates.length].min) do
  Thread.new do
    loop do
      key, examples = queue.pop(true)
      isolated = examples.lazy.filter_map do |candidate|
        project = project_metadata.fetch(candidate.project)
        corpus = ROOT.join(
          "tmp/project-benchmarks/corpora",
          "#{candidate.project}-#{project.fetch('revision')}"
        )
        source_path = candidate.example.fetch(0)
        full_path = corpus.join(source_path)
        next unless full_path.file?

        source = full_path.binread.force_encoding(Encoding::UTF_8).scrub
        window = source_windows(source, candidate.example.fetch(4)).find do |snippet|
          reproduces?(snippet, candidate.cop, candidate.kind, source_path, rubocop, rustocop)
        end
        next unless window

        [candidate, project, source_path, window]
      end.first
      mutex.synchronize do
        results << [key, isolated]
        status = isolated ? "isolated" : "unresolved"
        warn "#{status}: #{key.join(' ')}"
      end
    rescue ThreadError
      break
    end
  end
end
workers.each(&:join)

new_rows = []
FileUtils.mkdir_p(FIXTURE_ROOT.join("pending"))
results.sort.each do |(cop, kind), isolated|
  next unless isolated

  candidate, project, source_path, source = isolated
  file = slug(cop, kind)
  FIXTURE_ROOT.join(file).write(source)
  new_rows << [
    cop, file, project.fetch("repository"), project.fetch("revision"), source_path, kind
  ]
end

all_rows = (existing_rows + new_rows).sort_by { |row| [row.fetch(0), row.fetch(5)] }
content = ["cop\tfile\trepository\trevision\tsource_path\tkind\tselection", *all_rows.map { |row| row.join("\t") }]
MANIFEST.write("#{content.join("\n")}\n")

unresolved = results.count { |_key, isolated| isolated.nil? }
puts "Isolated #{new_rows.length} new mismatch directions; #{unresolved} unresolved."
puts "Manifest: #{MANIFEST.relative_path_from(ROOT)}"
