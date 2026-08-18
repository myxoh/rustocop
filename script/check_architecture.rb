# frozen_string_literal: true

# Keep these limits deliberately mechanical. Clippy owns function-level
# complexity; this script prevents modules from becoming dumping grounds.
LIMITS = {
  "crates/rustocop/src/main.rs" => 800,
  "crates/rustocop/src/prism_engine.rs" => 600,
}.freeze
DEFAULT_RUST_LIMIT = 600

failures = Dir.glob("crates/rustocop/src/**/*.rs").sort.filter_map do |path|
  lines = File.foreach(path).count
  limit = LIMITS.fetch(path, DEFAULT_RUST_LIMIT)
  next if lines <= limit

  "#{path}: #{lines} lines (maximum #{limit})"
end

if failures.empty?
  puts "Architecture limits passed."
else
  warn "Architecture limits failed:"
  failures.each { |failure| warn "  - #{failure}" }
  warn "Split by responsibility; do not raise a limit to land a feature."
  exit 1
end
