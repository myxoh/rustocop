# frozen_string_literal: true

require "fileutils"
require "digest"
require "json"
require "optparse"
require "pathname"
require "prism"
require "rbconfig"
require "set"
require "tmpdir"
require "yaml"

require_relative "../lib/rustocop/artifact_store"
require_relative "../lib/rustocop/diagnostic_signatures"
require_relative "../lib/rustocop/process_runner"
require_relative "../lib/rustocop/project_mismatch_inventory"
require_relative "../lib/rustocop/project_corpus"
require_relative "../lib/rustocop/repository_layout"

LAYOUT = Rustocop::RepositoryLayout.default
ROOT = Pathname.new(LAYOUT.root)
FIXTURE_ROOT = Pathname.new(LAYOUT.fixture_root)
CONFIG = Pathname.new(LAYOUT.benchmark_config)
NATIVE = Pathname.new(LAYOUT.native_binary)
SUPPLEMENT = ROOT.join("tmp", "project-parity", "isolated-unit-inputs.jsonl")

options = {
  jobs: 8, cops: [], limit_cops: nil, limit_per_cop: nil,
  dry_run: false
}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/isolate_project_parity_mismatches.rb REPORT [options]"
  parser.on("--jobs COUNT", Integer) { |value| options[:jobs] = value }
  parser.on("--cop NAME", "isolate one cop (repeatable)") { |value| options[:cops] << value }
  parser.on("--limit-cops COUNT", Integer, "isolate the largest remaining cop gaps") do |value|
    options[:limit_cops] = value
  end
  parser.on("--limit-per-cop COUNT", Integer, "process at most this many signatures per cop") do |value|
    options[:limit_per_cop] = value
  end
  parser.on("--dry-run", "list exhaustive candidate counts without running either engine") do
    options[:dry_run] = true
  end
end.parse!
report_path = ARGV.shift or abort "missing project-parity report"
report = JSON.parse(File.read(report_path))

inventory_metadata = report["mismatch_inventory"] or abort(
  "report has no exhaustive mismatch inventory; rerun audit_project_parity.rb"
)
inventory_path = Pathname.new(inventory_metadata.fetch("path"))
inventory_path = ROOT.join(inventory_path) unless inventory_path.absolute?
actual_inventory_sha = Digest::SHA256.file(inventory_path).hexdigest
abort "mismatch inventory checksum does not match report" unless
  actual_inventory_sha == inventory_metadata.fetch("sha256")
inventory = Rustocop::ArtifactStore.read_gzip_json(inventory_path, label: "mismatch inventory")
abort "unsupported mismatch inventory format" unless
  inventory.fetch("format_version") == Rustocop::ProjectMismatchInventory::FORMAT_VERSION &&
  inventory.fetch("fields") == Rustocop::ProjectMismatchInventory::ENTRY_FIELDS

Candidate = Data.define(:cop, :kind, :project, :example, :fingerprint, :source_sha256)

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

def reproduces?(source, candidate, relative_path, rubocop, rustocop)
  return false if candidate.cop != "Lint/Syntax" && !Prism.parse(source).success?

  Dir.mktmpdir("rustocop-project-isolation-") do |directory|
    path = File.join(directory, relative_path)
    FileUtils.mkdir_p(File.dirname(path))
    File.write(path, source)
    ruby = run_engine(rubocop, candidate.cop, path)
    rust = run_engine(rustocop, candidate.cop, path)
    return false unless ruby && rust

    left, right = candidate.kind == "rustocop_only" ? [rust, ruby] : [ruby, rust]
    severity_and_message = candidate.example.values_at(2, 3)
    left.any? do |signature, count|
      signature.values_at(0, 1) == severity_and_message && count > right.fetch(signature, 0)
    end
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
existing_fingerprints = Set.new
unit_manifest.fetch("cops").each_value do |entry|
  File.foreach(FIXTURE_ROOT.join(entry.fetch("cases"))).each do |line|
    JSON.parse(line).fetch("origins", []).each do |origin|
      next unless origin["kind"] == "project_isolation"

      existing_fingerprints << origin["signature_sha256"] if origin["signature_sha256"]
    end
  end
end

candidates = inventory.fetch("projects").flat_map do |project_name, project_inventory|
  project_inventory.fetch("entries").filter_map do |raw_entry|
    entry = Rustocop::ProjectMismatchInventory.entry_hash(raw_entry)
    example = Rustocop::ProjectMismatchInventory::SIGNATURE_FIELDS.map { |field| entry.fetch(field) }
    fingerprint = Digest::SHA256.hexdigest(JSON.generate([project_name, *raw_entry[0...-1]]))
    next if existing_fingerprints.include?(fingerprint)

    file_metadata = project_inventory.fetch("files").fetch(entry.fetch("path"))
    Candidate.new(
      cop: entry.fetch("cop"), kind: entry.fetch("direction"), project: project_name,
      example:, fingerprint:, source_sha256: file_metadata.fetch("sha256")
    )
  end
end

requested_cops = options.fetch(:cops)
if requested_cops.empty? && options[:limit_cops]
  requested_cops = report.fetch("combined_by_cop").filter_map do |cop, result|
    next unless result.fetch("classification") == "mismatch"

    exact = result.fetch("exact")
    [cop, result.fetch("rustocop") + result.fetch("rubocop") - (2 * exact)]
  end.sort_by { |cop, gap| [-gap, cop] }
     .first(options.fetch(:limit_cops))
     .map(&:first)
end
unless requested_cops.empty?
  unknown = requested_cops - report.fetch("combined_by_cop").keys
  abort "unknown cops: #{unknown.join(', ')}" unless unknown.empty?

  candidates.select! { |candidate| requested_cops.include?(candidate.cop) }
  warn "Isolation cops (#{requested_cops.length}): #{requested_cops.join(', ')}"
end
if options[:limit_per_cop]
  candidates = candidates.group_by(&:cop).flat_map do |_cop, cop_candidates|
    cop_candidates.first(options.fetch(:limit_per_cop))
  end
end
if options[:dry_run]
  puts "Exhaustive mismatch candidates: #{candidates.length}"
  candidates.group_by(&:cop).sort.each do |cop, cop_candidates|
    directions = cop_candidates.group_by(&:kind).transform_values(&:length)
    puts "#{cop}\t#{directions.sort.map { |kind, count| "#{kind}=#{count}" }.join(',')}"
  end
  exit
end

rubocop = [
  Gem.ruby,
  "-r", File.join(LAYOUT.root, "lib/rustocop/rubocop_reference_compatibility.rb"),
  Gem.bin_path("rubocop", "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}"),
  "--no-server", "--cache", "false"
].freeze
rustocop = [NATIVE.to_s].freeze
abort "native binary not found: #{NATIVE}" unless NATIVE.executable?

queue = Queue.new
candidates.sort_by { |candidate| [candidate.cop, candidate.kind, candidate.project, candidate.example] }
          .each { |candidate| queue << candidate }
results = []
mutex = Mutex.new
workers = Array.new([options.fetch(:jobs), candidates.length].min) do
  Thread.new do
    loop do
      candidate = queue.pop(true)
      project = project_metadata.fetch(candidate.project)
      corpus = Pathname.new(LAYOUT.project_corpus(project))
      source_path = candidate.example.fetch(0)
      full_path = corpus.join(source_path)
      isolated = if full_path.file? && Digest::SHA256.file(full_path).hexdigest == candidate.source_sha256
                   source = full_path.binread.force_encoding(Encoding::UTF_8).scrub
                   window = source_windows(source, candidate.example.fetch(4)).find do |snippet|
                     reproduces?(snippet, candidate, source_path, rubocop, rustocop)
                   end
                   [candidate, project, source_path, window] if window
                 end
      mutex.synchronize do
        results << [candidate, isolated]
        status = isolated ? "isolated" : "unresolved"
        warn "#{status}: #{candidate.cop} #{candidate.kind} #{candidate.project}:#{candidate.example.fetch(0)}:#{candidate.example.fetch(4)}"
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
      "path" => source_path, "line" => candidate.example.fetch(4),
      "signature_sha256" => candidate.fingerprint
    }
  }
end

captured = results.sort_by { |candidate, _isolated| candidate.fingerprint }.filter_map do |_candidate, isolated|
  next unless isolated

  candidate, project, source_path, source = isolated
  captured_case(candidate.cop, candidate.kind, candidate, project, source_path, source, rubocop)
end

unless captured.empty?
  FileUtils.mkdir_p(SUPPLEMENT.dirname)
  SUPPLEMENT.write(captured.map { |item| JSON.generate(item) }.join("\n") + "\n")
  generator = ROOT.join("script", "generate_unit_fixtures.rb")
  abort "unit-contract import failed" unless system(
    RbConfig.ruby, generator.to_s, "--supplement", SUPPLEMENT.to_s, chdir: ROOT.to_s
  )
end

unresolved = results.count { |_candidate, isolated| isolated.nil? }
puts "Added #{captured.length} minimized mismatch signatures to cop-owned unit contracts; #{unresolved} unresolved."
puts "Transient import: #{SUPPLEMENT.relative_path_from(ROOT)}" unless captured.empty?
