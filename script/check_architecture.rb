# frozen_string_literal: true

require "yaml"

# Keep these limits deliberately mechanical. Clippy owns function-level
# complexity; this script prevents modules from becoming dumping grounds.
LIMITS = {
  "crates/rustocop/src/main.rs" => 50
}.freeze
DEFAULT_RUST_LIMIT = 350
DEBT_PATH = "spec/architecture_debt.yml"
MAX_COPS_PER_MODULE = 16
SOURCE_ROOT = "crates/rustocop/src"
ROOT_MODULES = %w[config.rs main.rs model.rs].freeze
PACKAGE_DIRECTORIES = %w[app config cops engine].freeze

DEPENDENCY_RULES = {
  "#{SOURCE_ROOT}/cops/" => %w[app engine],
  "#{SOURCE_ROOT}/engine/" => %w[app],
  "#{SOURCE_ROOT}/config/" => %w[app cops engine model],
  "#{SOURCE_ROOT}/config.rs" => %w[app cops engine model],
  "#{SOURCE_ROOT}/model.rs" => %w[app config cops engine]
}.freeze

rust_files = Dir.glob("#{SOURCE_ROOT}/**/*.rs").sort
module_debt = YAML.safe_load_file(DEBT_PATH, aliases: false) || {}
failures = rust_files.filter_map do |path|
  lines = File.foreach(path).count
  limit = LIMITS.fetch(path, DEFAULT_RUST_LIMIT)
  debt_limit = module_debt[path]
  if lines <= limit
    next unless debt_limit

    next "#{path}: now #{lines} lines; remove its obsolete architecture debt entry"
  end
  next "#{path}: #{lines} lines (maximum #{limit}); add no new architecture debt" unless debt_limit
  next "#{path}: grew to #{lines} lines (debt ceiling #{debt_limit})" if lines > debt_limit
  next "#{path}: reduced to #{lines} lines; lower its debt ceiling from #{debt_limit}" if lines < debt_limit

  nil
end

(module_debt.keys - rust_files).each do |path|
  failures << "#{path}: architecture debt entry does not name a Rust source file"
end

cop_declaration = /=>\s*"[A-Z][^"]+\/[^"]+"|(?:replace|report|custom)\(\s*"[A-Z][^"]+\/[^"]+"/m
rust_files.grep(%r{/cops/}).each do |path|
  cop_count = File.read(path).scan(cop_declaration).length
  next if cop_count <= MAX_COPS_PER_MODULE

  failures << "#{path}: #{cop_count} cops (maximum #{MAX_COPS_PER_MODULE})"
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
  warn "Split by responsibility; never raise a debt ceiling to land a feature."
  exit 1
end
