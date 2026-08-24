# frozen_string_literal: true

require "fileutils"
require "open3"
require "pathname"
require "tmpdir"

ROOT = Pathname.new(__dir__).join("..").expand_path
FIXTURE_ROOT = ROOT.join("spec", "fixtures", "cops")
REQUIRED_INPUTS = %w[input.rb rubocop.yml].freeze

def rubocop_command(*arguments)
  [
    Gem.ruby,
    Gem.bin_path("rubocop", "rubocop"),
    "--no-server",
    "--cache",
    "false",
    "--config",
    "rubocop.yml",
    *arguments
  ]
end

def run_rubocop(directory, *arguments)
  environment = { "RUBOCOP_CACHE_ROOT" => File.join(Dir.tmpdir, "rustocop-rubocop-cache") }
  stdout, stderr, status = Open3.capture3(environment, *rubocop_command(*arguments), chdir: directory)
  unless stderr.empty?
    warn stderr
    abort "RuboCop wrote to stderr in #{directory}"
  end
  return stdout if status.success? || status.exitstatus == 1

  warn stderr
  abort "RuboCop failed in #{directory} (exit #{status.exitstatus})"
end

requested = ARGV
fixtures = FIXTURE_ROOT.glob("*/*/end_to_end/*").select(&:directory?).sort
unless requested.empty?
  fixtures.select! do |fixture|
    cop = fixture.relative_path_from(FIXTURE_ROOT).each_filename.first(2).to_a.join("/")
    requested.include?(cop) || requested.include?(fixture.basename.to_s)
  end
end
abort "No matching cop-owned end-to-end fixtures" if fixtures.empty?

fixtures.each do |fixture|
  missing = REQUIRED_INPUTS.reject { |name| fixture.join(name).file? }
  abort "#{fixture}: missing #{missing.join(', ')}" unless missing.empty?

  diagnostics = run_rubocop(fixture, "--format", "simple", "--no-color", "input.rb")
  fixture.join("output.out").write(diagnostics)

  Dir.mktmpdir("rustocop-real-fixture-") do |temporary_directory|
    FileUtils.cp(fixture.join("input.rb"), File.join(temporary_directory, "input.rb"))
    FileUtils.cp(fixture.join("rubocop.yml"), File.join(temporary_directory, "rubocop.yml"))
    run_rubocop(temporary_directory, "--autocorrect-all", "--format", "quiet", "input.rb")
    FileUtils.cp(File.join(temporary_directory, "input.rb"), fixture.join("output.rb"))
  end

  puts "updated #{fixture.relative_path_from(ROOT)}"
end
