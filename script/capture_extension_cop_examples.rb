# frozen_string_literal: true

require "digest"
require "fileutils"
require "json"
require "open3"
require "optparse"
require "pathname"
require "rubocop"
require "tmpdir"
require "yaml"

ROOT = Pathname.new(File.expand_path("..", __dir__))

options = { check: false }
OptionParser.new do |parser|
  parser.banner = "Usage: bundle exec ruby script/capture_extension_cop_examples.rb CASES_YML [options]"
  parser.on("--check", "fail when generated contracts are stale") { options[:check] = true }
end.parse!

input = Pathname.new(ARGV.shift || abort("missing cases YAML"))
input = ROOT.join(input) unless input.absolute?
definition = YAML.safe_load_file(input, aliases: true)
reference = definition.fetch("reference")
plugin = reference.fetch("plugin")
version = reference.fetch("version").to_s
rubocop_version = reference.fetch("rubocop_version").to_s
target_ruby = reference.fetch("target_ruby_version").to_s

specification = Gem::Specification.find_by_name(plugin, version)
abort "#{plugin} #{specification.version} does not match pinned #{version}" unless specification.version.to_s == version
abort "RuboCop #{RuboCop::Version::STRING} does not match pinned #{rubocop_version}" unless RuboCop::Version::STRING == rubocop_version

plugin_defaults_path = Pathname.new(specification.full_gem_path).join("config/default.yml")
plugin_defaults = YAML.safe_load_file(plugin_defaults_path, aliases: true)
output_root = input.dirname
generated = {}
manifest = {
  "rubocop_version" => "#{plugin} #{version} / rubocop #{rubocop_version}",
  "plugin" => plugin,
  "plugin_version" => version,
  "definition_sha256" => Digest::SHA256.file(input).hexdigest,
  "cops" => {}
}

def run_rubocop(arguments, accepted: [0, 1])
  command = [Gem.ruby, Gem.bin_path("rubocop", "rubocop"), *arguments]
  stdout, stderr, status = Open3.capture3(*command)
  return stdout if accepted.include?(status.exitstatus)

  abort "#{command.join(' ')} failed (#{status.exitstatus}):\n#{stderr}\n#{stdout}"
end

def diagnostics_by_path(payload)
  payload.fetch("files").to_h do |file|
    rows = file.fetch("offenses").map do |offense|
      location = offense.fetch("location")
      {
        "message" => offense.fetch("message"),
        "severity" => offense.fetch("severity"),
        "correctable" => offense.fetch("correctable"),
        "line" => location.fetch("start_line"),
        "column" => location.fetch("start_column"),
        "last_line" => location.fetch("last_line"),
        "last_column" => location.fetch("last_column")
      }
    end
    [File.expand_path(file.fetch("path")), rows]
  end
end

def materialize_cases(root, cases)
  cases.each_with_index.map do |example, index|
    relative = example.fetch("path", "example.rb").delete_prefix("/")
    path = root.join(format("%03d", index), relative)
    FileUtils.mkdir_p(path.dirname)
    path.binwrite(example.fetch("source"))
    path
  end
end

def run_correction_batch(root, cases, config_path, cop, mode)
  paths = materialize_cases(root, cases)
  run_rubocop([
    "--cache", "false", "--config", config_path.to_s, "--only", cop,
    mode, "--format", "json", *paths.map(&:to_s)
  ])
  paths.map(&:binread)
end

def cached_correction(corrected, original, all)
  return nil if corrected == original
  return "$all" if all && corrected == all

  corrected
end

definition.fetch("cops").sort.each do |cop, cases|
  defaults = plugin_defaults.fetch(cop, {}).merge("Enabled" => true)
  config_source = YAML.dump(
    "plugins" => [plugin],
    "AllCops" => { "NewCops" => "enable", "TargetRubyVersion" => target_ruby },
    cop => defaults
  )
  config_id = Digest::SHA256.hexdigest(config_source)[0, 16]
  case_rows = []

  Dir.mktmpdir("rustocop-extension-cop-") do |directory|
    directory = Pathname.new(directory)
    config_path = directory.join("rubocop.yml")
    File.write(config_path, config_source)
    diagnostic_paths = materialize_cases(directory.join("diagnostics"), cases)
    diagnostic_output = run_rubocop([
      "--cache", "false", "--config", config_path.to_s, "--only", cop,
      "--format", "json", *diagnostic_paths.map(&:to_s)
    ])
    diagnostics = diagnostics_by_path(JSON.parse(diagnostic_output))
    all_corrections = run_correction_batch(directory.join("all"), cases, config_path, cop, "-A")
    safe_corrections = run_correction_batch(directory.join("safe"), cases, config_path, cop, "-a")

    cases.each_with_index do |example, index|
      source = example.fetch("source")
      inspected_path = example.fetch("path", "example.rb")
      all = all_corrections.fetch(index)
      safe = safe_corrections.fetch(index)
      identity = Digest::SHA256.hexdigest([cop, example.fetch("id"), source, config_id].join("\0"))[0, 20]
      origin = {
        "id" => "#{plugin}-#{version}:#{cop}:#{example.fetch('id')}",
        "description" => example.fetch("id"),
        "file" => input.relative_path_from(ROOT).to_s
      }.merge(example.fetch("origin", {}))
      case_rows << {
        "id" => identity,
        "cop" => cop,
        "source" => source,
        "path" => inspected_path,
        "ruby_version" => target_ruby,
        "parser_engine" => "parser_prism",
        "external_encoding" => "UTF-8",
        "internal_encoding" => nil,
        "file_mode" => nil,
        "config" => config_id,
        "diagnostics" => diagnostics.fetch(diagnostic_paths.fetch(index).expand_path.to_s),
        "autocorrect_checked" => true,
        "autocorrect_all" => cached_correction(all, source, nil),
        "autocorrect_all_error" => nil,
        "autocorrect_safe" => cached_correction(safe, source, all),
        "autocorrect_safe_error" => nil,
        "origins" => [origin]
      }
    end
  end

  relative = Pathname.new("contracts").join(*cop.split("/"))
  cases_path = relative.join("cases.jsonl")
  configs_path = relative.join("configs.json")
  cases_content = "#{case_rows.map { |row| JSON.generate(row) }.join("\n")}\n"
  configs_content = "#{JSON.pretty_generate(config_id => config_source)}\n"
  generated[cases_path] = cases_content
  generated[configs_path] = configs_content
  manifest.fetch("cops")[cop] = {
    "cases" => cases_path.to_s,
    "cases_sha256" => Digest::SHA256.hexdigest(cases_content),
    "configs" => configs_path.to_s,
    "configs_sha256" => Digest::SHA256.hexdigest(configs_content)
  }
end

generated[Pathname.new("unit_manifest.json")] = "#{JSON.pretty_generate(manifest)}\n"
stale = generated.filter_map do |relative, content|
  path = output_root.join(relative)
  next if path.file? && path.binread == content

  if options[:check]
    relative.to_s
  else
    FileUtils.mkdir_p(path.dirname)
    path.binwrite(content)
    puts "updated #{path.relative_path_from(ROOT)}"
    nil
  end
end
abort "stale extension contracts: #{stale.join(', ')}" if stale.any?
