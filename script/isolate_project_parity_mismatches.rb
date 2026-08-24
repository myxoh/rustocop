# frozen_string_literal: true

require "fileutils"
require "json"
require "optparse"
require "pathname"
require "prism"
require "rbconfig"
require "tmpdir"
require "yaml"

require_relative "../lib/rustocop/diagnostic_signatures"
require_relative "../lib/rustocop/process_runner"
require_relative "../lib/rustocop/project_corpus"
require_relative "../lib/rustocop/repository_layout"

LAYOUT = Rustocop::RepositoryLayout.default
ROOT = Pathname.new(LAYOUT.root)
FIXTURE_ROOT = Pathname.new(LAYOUT.fixture_root)
CONFIG = Pathname.new(LAYOUT.benchmark_config)
NATIVE = Pathname.new(LAYOUT.native_binary)
SUPPLEMENT = ROOT.join("tmp", "project-parity", "isolated-unit-inputs.jsonl")

options = { jobs: 8 }
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/isolate_project_parity_mismatches.rb REPORT [options]"
  parser.on("--jobs COUNT", Integer) { |value| options[:jobs] = value }
end.parse!
report_path = ARGV.shift or abort "missing project-parity report"
report = JSON.parse(File.read(report_path))

Candidate = Data.define(:cop, :kind, :project, :example)

def signatures(output, cop)
  offenses = JSON.parse(output).fetch("files", []).flat_map { |file| file.fetch("offenses", []) }
  return if cop != "Lint/Syntax" && offenses.any? { |offense| offense.fetch("cop_name") == "Lint/Syntax" }

  Rustocop::DiagnosticSignatures.for_cop({ "files" => [{ "path" => "input.rb", "offenses" => offenses }] }, cop)
                                .map(&:location_tuple)
                                .tally
rescue JSON::ParserError, KeyError
  nil
end

def run_engine(command, cop, path)
  result = Rustocop::ProcessRunner.capture(
    *command, "--config", CONFIG.to_s, "--format", "json", "--only", cop, path
  )
  return unless result.accepted?(0, 1) && result.stderr.empty?

  signatures(result.stdout, cop)
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

project_metadata = Rustocop::ProjectCorpus::PROJECTS.to_h { |project| [project.fetch("name"), project] }
unit_manifest = JSON.parse(FIXTURE_ROOT.join("unit_manifest.json").read)
existing = unit_manifest.fetch("cops").flat_map do |cop, entry|
  File.foreach(FIXTURE_ROOT.join(entry.fetch("cases"))).flat_map do |line|
    JSON.parse(line).fetch("origins", []).filter_map do |origin|
      [cop, origin["direction"]] if origin["kind"] == "project_isolation"
    end
  end
end.to_h { |key| [key, true] }

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
        corpus = Pathname.new(LAYOUT.project_corpus(project))
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

def captured_case(cop, kind, candidate, project, source_path, source, rubocop)
  report = Dir.mktmpdir("rustocop-unit-capture-") do |directory|
    path = File.join(directory, source_path)
    FileUtils.mkdir_p(File.dirname(path))
    File.write(path, source)
    result = Rustocop::ProcessRunner.capture(
      *rubocop, "--config", CONFIG.to_s, "--format", "json", "--only", cop, path
    )
    abort "RuboCop failed while capturing #{cop}" unless result.accepted?(0, 1) && result.stderr.empty?

    JSON.parse(result.stdout)
  end
  offenses = report.fetch("files").flat_map { |file| file.fetch("offenses") }.filter_map do |offense|
    next unless offense.fetch("cop_name") == cop

    location = offense.fetch("location")
    {
      "message" => offense.fetch("message"), "severity" => offense.fetch("severity"),
      "correctable" => offense.fetch("correctable"),
      "line" => location.fetch("start_line"), "column" => location.fetch("start_column"),
      "last_line" => location.fetch("last_line"), "last_column" => location.fetch("last_column")
    }
  end
  config = YAML.safe_load_file(CONFIG, aliases: true)
  {
    "cop" => cop, "selection" => cop, "source" => source, "path" => source_path,
    "ruby_version" => config.dig("AllCops", "TargetRubyVersion").to_s,
    "parser_engine" => config.dig("AllCops", "ParserEngine"),
    "default_external_encoding" => "UTF-8", "default_internal_encoding" => nil,
    "config" => config, "offenses" => offenses, "lsp" => false, "check_autocorrect" => true,
    "example" => {
      "kind" => "project_isolation", "direction" => kind,
      "repository" => project.fetch("repository"), "revision" => project.fetch("revision"),
      "path" => source_path, "line" => candidate.example.fetch(4)
    }
  }
end

captured = results.sort.filter_map do |(cop, kind), isolated|
  next unless isolated

  candidate, project, source_path, source = isolated
  captured_case(cop, kind, candidate, project, source_path, source, rubocop)
end

unless captured.empty?
  FileUtils.mkdir_p(SUPPLEMENT.dirname)
  SUPPLEMENT.write(captured.map { |item| JSON.generate(item) }.join("\n") + "\n")
  generator = ROOT.join("script", "generate_unit_fixtures.rb")
  abort "unit-contract import failed" unless system(
    RbConfig.ruby, generator.to_s, "--supplement", SUPPLEMENT.to_s, chdir: ROOT.to_s
  )
end

unresolved = results.count { |_key, isolated| isolated.nil? }
puts "Added #{captured.length} minimized mismatch directions to cop-owned unit contracts; #{unresolved} unresolved."
puts "Transient import: #{SUPPLEMENT.relative_path_from(ROOT)}" unless captured.empty?
