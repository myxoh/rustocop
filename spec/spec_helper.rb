# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "rbconfig"
require "tmpdir"

require_relative "../lib/rustocop"

ROOT = File.expand_path("..", __dir__)

CommandResult = Struct.new(:stdout, :stderr, :status, keyword_init: true)

def run_rustocop(*args, stdin: nil, env: {}, chdir: nil)
  options = { stdin_data: stdin }
  options[:chdir] = chdir if chdir
  stdout, stderr, status = Open3.capture3(env, RbConfig.ruby, File.join(ROOT, "exe", "rustocop"), *args, options)
  CommandResult.new(stdout:, stderr:, status:)
end

def run_rubocop(*args, stdin: nil, chdir: nil)
  args = ["--cache", "false", *args] unless args.any? { |arg| arg == "--cache" || arg.start_with?("--cache=") }
  options = { stdin_data: stdin }
  options[:chdir] = chdir if chdir
  stdout, stderr, status = Open3.capture3(RbConfig.ruby, Gem.bin_path("rubocop", "rubocop"), *args, options)
  CommandResult.new(stdout:, stderr:, status:)
end

def parsed_json(result)
  JSON.parse(result.stdout)
end

def normalize_rubocop_report(report)
  report = Marshal.load(Marshal.dump(report))
  report.fetch("metadata")["rubocop_version"] = "normalized"
  report.fetch("files").each { |file| file["path"] = File.basename(file.fetch("path")) }
  report
end
