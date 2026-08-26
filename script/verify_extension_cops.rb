# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "rbconfig"

ROOT = File.expand_path("..", __dir__)

options = { refresh: false }
OptionParser.new do |parser|
  parser.banner = "Usage: bundle exec ruby script/verify_extension_cops.rb [--refresh] PACK COP [...]"
  parser.on("--refresh", "refresh the pinned RuboCop extension oracle first") { options[:refresh] = true }
end.parse!
pack = ARGV.shift || abort("Pass an extension pack directory")
pack = File.expand_path(pack, ROOT)
cases_path = File.join(pack, "cases.yml")
manifest_path = File.join(pack, "unit_manifest.json")
cops = ARGV
abort "Pass at least one extension cop" if cops.empty?

if options[:refresh]
  capture = File.join(ROOT, "script/capture_extension_cop_examples.rb")
  success = system(RbConfig.ruby, capture, cases_path)
  exit 1 unless success
end

manifest = JSON.parse(File.read(manifest_path))
definition_sha = Digest::SHA256.file(cases_path).hexdigest
abort "extension cases changed; rerun with --refresh" unless manifest.fetch("definition_sha256") == definition_sha
manifest.fetch("cops").each_value do |entry|
  %w[cases configs].each do |kind|
    path = File.join(File.dirname(manifest_path), entry.fetch(kind))
    actual = Digest::SHA256.file(path).hexdigest
    abort "stale extension #{kind}: #{path}; rerun with --refresh" unless
      actual == entry.fetch("#{kind}_sha256")
  end
end

environment = {
  "RUSTOCOP_UNIT_MANIFEST" => manifest_path,
  "RUSTOCOP_UNIT_COP" => cops.join(",")
}
success = system(
  environment,
  "cargo", "test", "--manifest-path", File.join(ROOT, "crates/rustocop/Cargo.toml"),
  "--profile", "fixture", "cached_unit_contracts_match", "--", "--ignored", "--nocapture"
)
exit(success ? 0 : 1)
