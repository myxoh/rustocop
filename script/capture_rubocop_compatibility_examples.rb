# frozen_string_literal: true

require "json"
require "pathname"
require "rspec/core"
require "rubocop"
require "rubocop-ast"
require "tempfile"
require "time"

PROJECT_ROOT = Pathname(__dir__).join("..").expand_path
OUTPUT = PROJECT_ROOT.join("spec/upstream/rubocop-compatibility-examples.json")
VERSIONS = { "rubocop" => "1.87.0", "rubocop-ast" => "1.49.1" }.freeze

# Three shared support objects normally come from RuboCop's development-only
# spec helper. Dry-run enumeration only needs them to exist while the example
# tree is constructed; none of their hooks execute.
module FileHelper; end
RSpec.shared_context("cli spec behavior") {}
RSpec.shared_context("mock console output") {}

paths = [
  *PROJECT_ROOT.glob("spec/upstream/rubocop-1.87.0/spec/rubocop/cop/*_spec.rb"),
  *PROJECT_ROOT.glob("spec/upstream/rubocop-1.87.0/spec/rubocop/cop/mixin/**/*_spec.rb"),
  *PROJECT_ROOT.glob("spec/upstream/rubocop-ast-1.49.1/spec/rubocop/ast/**/*_spec.rb")
].map(&:to_s).sort

raw = Tempfile.new(["rubocop-compatibility-examples", ".json"])
status = RSpec::Core::Runner.run([
  *paths,
  "--dry-run",
  "--format", "json",
  "--out", raw.path
])
abort "RSpec dry-run enumeration failed" unless status.zero?

result = JSON.parse(Pathname(raw.path).read)
summary = result.fetch("summary")
abort "RSpec dry-run had load errors" unless summary.fetch("errors_outside_of_examples_count").zero?

examples = result.fetch("examples").map do |example|
  id = example.fetch("id").sub(%r{\A\./}, "")
  package, version, source_and_id = id.match(
    %r{\Aspec/upstream/(rubocop(?:-ast)?)-(\d+\.\d+\.\d+)/(spec/.+)\z}
  ).captures
  source, rspec_id = source_and_id.split("[", 2)
  {
    "package" => package,
    "version" => version,
    "source" => source,
    "rspec_id" => "[#{rspec_id}",
    "description" => example.fetch("description"),
    "full_description" => example.fetch("full_description"),
    "definition_line" => example.fetch("line_number")
  }
end.sort_by { |example| [example.fetch("package"), example.fetch("source"), example.fetch("rspec_id")] }

payload = {
  "format_version" => 1,
  "updated_at" => Time.now.utc.iso8601,
  "versions" => VERSIONS,
  "example_count" => examples.length,
  "examples" => examples
}
OUTPUT.write("#{JSON.pretty_generate(payload)}\n")
puts "Wrote #{examples.length} expanded upstream examples to #{OUTPUT}"
