# frozen_string_literal: true

require "fileutils"
require "json"
require "open3"
require "rbconfig"
require "tmpdir"

require_relative "../lib/rustocop"

ROOT = Rustocop::RepositoryLayout.default.root

def run_rustocop(*args, stdin: nil, env: {}, chdir: nil)
  Rustocop::ProcessRunner.capture(
    RbConfig.ruby, File.join(ROOT, "exe", "rustocop"), *args,
    env:, chdir:, stdin_data: stdin
  )
end

def run_rubocop(*args, stdin: nil, chdir: nil, env: {})
  args = ["--cache", "false", *args] unless args.any? { |arg| arg == "--cache" || arg.start_with?("--cache=") }
  Rustocop::ProcessRunner.capture(
    RbConfig.ruby, Gem.bin_path("rubocop", "rubocop"), *args,
    env:, chdir:, stdin_data: stdin
  )
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
