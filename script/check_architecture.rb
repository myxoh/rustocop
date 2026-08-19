# frozen_string_literal: true

# Keep these limits deliberately mechanical. Clippy owns function-level
# complexity; this script prevents modules from becoming dumping grounds.
LIMITS = {
  "crates/rustocop/src/main.rs" => 50
}.freeze
DEFAULT_RUST_LIMIT = 400
SOURCE_ROOT = "crates/rustocop/src"
ROOT_MODULES = %w[config.rs main.rs model.rs].freeze
PACKAGE_DIRECTORIES = %w[app cops engine].freeze

DEPENDENCY_RULES = {
  "#{SOURCE_ROOT}/cops/" => %w[app engine],
  "#{SOURCE_ROOT}/engine/" => %w[app],
  "#{SOURCE_ROOT}/config.rs" => %w[app cops engine model],
  "#{SOURCE_ROOT}/model.rs" => %w[app config cops engine]
}.freeze

rust_files = Dir.glob("#{SOURCE_ROOT}/**/*.rs").sort
failures = rust_files.filter_map do |path|
  lines = File.foreach(path).count
  limit = LIMITS.fetch(path, DEFAULT_RUST_LIMIT)
  next if lines <= limit

  "#{path}: #{lines} lines (maximum #{limit})"
end

root_modules = Dir.glob("#{SOURCE_ROOT}/*.rs").map { |path| File.basename(path) }.sort
unexpected_modules = root_modules - ROOT_MODULES
failures << "unexpected root modules: #{unexpected_modules.join(', ')}" unless unexpected_modules.empty?

package_directories = Dir.children(SOURCE_ROOT).filter { |entry| File.directory?(File.join(SOURCE_ROOT, entry)) }.sort
unexpected_packages = package_directories - PACKAGE_DIRECTORIES
failures << "unexpected package directories: #{unexpected_packages.join(', ')}" unless unexpected_packages.empty?

DEPENDENCY_RULES.each do |prefix, forbidden_modules|
  paths = prefix.end_with?("/") ? rust_files.grep(/^#{Regexp.escape(prefix)}/) : [prefix]
  paths.each do |path|
    source = File.read(path)
    forbidden_modules.each do |mod|
      next unless source.match?(/crate::#{Regexp.escape(mod)}\b/)

      failures << "#{path}: forbidden dependency on crate::#{mod}"
    end
  end
end

if failures.empty?
  puts "Architecture limits passed."
else
  warn "Architecture limits failed:"
  failures.each { |failure| warn "  - #{failure}" }
  warn "Split by responsibility; do not raise a limit to land a feature."
  exit 1
end
