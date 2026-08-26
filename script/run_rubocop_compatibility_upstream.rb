# frozen_string_literal: true

require "fileutils"
require "json"
require "pathname"
require "rbconfig"
require "tmpdir"

ROOT = Pathname(__dir__).join("..").expand_path
MANIFEST = JSON.parse(ROOT.join("crates/rustocop/rubocop-translation.json").read)

def registered_specs(package)
  MANIFEST.fetch("components").flat_map { |component| component.fetch("specs") }
    .select { |spec| spec.fetch("package") == package }
    .map { |spec| spec.fetch("source") }
    .uniq
end

def run_rspec(root, specs, expected_examples:, load_paths: [], gems: {})
  runner = <<~'RUBY'
    require "json"
    require "rspec/core"
    root = ARGV.shift
    expected_examples = Integer(ARGV.shift)
    JSON.parse(ARGV.shift).each { |name, version| gem name, "=#{version}" }
    require File.join(root, "spec/spec_helper")
    RSpec.configuration.example_status_persistence_file_path = nil
    Dir.chdir(root)
    status = RSpec::Core::Runner.run(["--format", "progress", *ARGV])
    actual_examples = RSpec.world.filtered_examples.values.sum(&:length)
    warn "Expected #{expected_examples} runnable examples, got #{actual_examples}" if actual_examples != expected_examples
    exit(status.zero? && actual_examples == expected_examples ? 0 : 1)
  RUBY
  command = [
    RbConfig.ruby, *load_paths.map { |path| "-I#{path}" }, "-e", runner,
    root.to_s, expected_examples.to_s, JSON.generate(gems), *specs
  ]
  system(*command)
end

ast_root = ROOT.join("spec/upstream/rubocop-ast-#{MANIFEST.fetch("rubocop_ast_version")}")
abort "rubocop-ast upstream compatibility suite failed" unless run_rspec(
  ast_root,
  registered_specs("rubocop-ast"),
  expected_examples: 2_719,
  gems: { "rubocop-ast" => MANIFEST.fetch("rubocop_ast_version") }
)

# RuboCop's development helper unconditionally loads WebMock and MCP even
# though the registered shared-cop suites use neither. It also expects both
# gem lib/ and spec/ to be writable siblings for generator tests. Build that
# exact disposable layout without adding unrelated runtime dependencies.
Dir.mktmpdir("rustocop-rubocop-upstream-") do |temporary|
  overlay = Pathname(temporary).join("rubocop")
  shim = Pathname(temporary).join("shim")
  FileUtils.mkdir_p(overlay)
  gem_root = Gem::Specification.find_by_name(
    "rubocop", MANIFEST.fetch("rubocop_version")
  ).full_gem_path
  FileUtils.cp_r("#{gem_root}/.", overlay)
  vendored = ROOT.join("spec/upstream/rubocop-#{MANIFEST.fetch("rubocop_version")}")
  FileUtils.cp_r(vendored.join("spec"), overlay)
  FileUtils.cp_r(vendored.join("config"), overlay)
  FileUtils.mkdir_p(shim.join("webmock"))
  FileUtils.mkdir_p(shim.join("rubocop/mcp"))
  shim.join("webmock/rspec.rb").write("# Not used by the registered compatibility specs.\n")
  shim.join("rubocop/mcp/server.rb").write("# Not used by the registered compatibility specs.\n")

  abort "RuboCop upstream compatibility suite failed" unless run_rspec(
    overlay,
    registered_specs("rubocop"),
    expected_examples: 416,
    load_paths: [shim, overlay.join("lib")],
    gems: {
      "rubocop-ast" => MANIFEST.fetch("rubocop_ast_version"),
      "rubocop" => MANIFEST.fetch("rubocop_version")
    }
  )
end

warn "Upstream compatibility suites passed (3,135 runnable examples; " \
     "4 parser-tagged examples remain represented in the Rust contracts)."
