# frozen_string_literal: true

require "fileutils"
require "optparse"

ROOT = File.expand_path("..", __dir__)
PRISM_ROOT = File.join(ROOT, "crates/rustocop/src/cops/prism")
COMPOSITION_ROOT = File.join(PRISM_ROOT, "mod.rs")
FIXTURE_TESTS = File.join(ROOT, "crates/rustocop/src/engine/fixture_tests.rs")

options = {
  autocorrect: false,
  dry_run: false,
  family: nil,
  fixture_path: "/project/example.rb",
  node_cast: "as_call_node"
}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/new_cop.rb Department/CopName KIND [options]"
  parser.on("--dry-run", "print the generated source without writing") { options[:dry_run] = true }
  parser.on("--autocorrect", "run the generated fixture with autocorrection enabled") do
    options[:autocorrect] = true
  end
  parser.on("--fixture-path PATH", "path exposed to the generated fixture") do |path|
    options[:fixture_path] = path
  end
  parser.on("--family MODULE", "append the cop to an existing Prism family module") do |family|
    options[:family] = family
  end
  parser.on("--node-cast METHOD", "Prism Node cast for node cops") { |cast| options[:node_cast] = cast }
end.parse!

cop_name = ARGV.shift or abort "missing cop name"
kind = ARGV.shift || "call"
abort "kind must be call, node, any_node, or source" unless %w[call node any_node source].include?(kind)
abort "cop name must contain only Ruby constant-name characters" unless cop_name.match?(/\A[A-Z][A-Za-z0-9]*\/[A-Z][A-Za-z0-9]*\z/)
department, short_name = cop_name.split("/", 2)
abort "cop name must look like Department/CopName" unless department && short_name
if kind == "node" && !options.fetch(:node_cast).match?(/\Aas_[a-z0-9_]+_node\z/)
  abort "node cast must look like as_if_node"
end

snake = short_name.gsub(/([a-z\d])([A-Z])/, '\1_\2').downcase
fixture_name = "#{department.downcase}_#{snake}"
module_name = options[:family] || fixture_name
abort "family module must use snake_case" unless module_name.match?(/\A[a-z][a-z0-9_]*\z/)
type_name = short_name.gsub(/[^A-Za-z0-9]/, "")
callback_name = options[:family] ? snake : "check"
path = File.join(PRISM_ROOT, "#{module_name}.rs")
if options[:family]
  abort "family module does not exist: #{path}" unless File.exist?(path)
else
  abort "#{path} already exists" if File.exist?(path)
end
fixture = File.join(ROOT, "crates/rustocop/tests/fixtures/inspection", fixture_name)
abort "#{fixture} already exists" if File.exist?(fixture)

callback = case kind
           when "call"
             <<~RUST
               fn #{callback_name}(node: &CallNode<'_>, context: &mut CopContext<'_, '_>) {
                   if !match_call(node).named(b"replace_me").matches() {
                       return;
                   }
                   context.report_call(node, "Replace with the upstream RuboCop message.");
               }
             RUST
           when "node"
             node_type = options.fetch(:node_cast).delete_prefix("as_").delete_suffix("_node")
               .split("_").map(&:capitalize).join + "Node"
             <<~RUST
               fn #{callback_name}(node: &ruby_prism::#{node_type}<'_>, context: &mut CopContext<'_, '_>) {
                   let _ = (node, context);
               }
             RUST
           when "any_node"
             <<~RUST
               fn #{callback_name}(node: &Node<'_>, context: &mut CopContext<'_, '_>) {
                   // Use this only when the cop genuinely handles several node kinds.
                   let _ = (node, context);
               }
             RUST
           when "source"
             <<~RUST
               fn #{callback_name}(context: &mut CopContext<'_, '_>) {
                   // Source callbacks are for genuinely lexical or file-level rules.
                   // Prefer `call` or typed `node` for Ruby syntax.
                   let source = context.source_file();
                   let _ = source;
               }
             RUST
           end

declaration = if kind == "node"
                "node(#{options.fetch(:node_cast)}, #{callback_name})"
              else
                "#{kind}(#{callback_name})"
              end
source = <<~RUST
  use super::*;

  define_cops! {
      #{type_name} => "#{cop_name}" => #{declaration},
  }

  #{callback}
RUST

fixture_input = "# Replace this with upstream-derived offending and clean examples.\n"
fixture_header = "cop\tline\tcolumn\tlast_line\tlast_column\tcorrectable\tcorrected\tmessage\n"
fixture_test = <<~RUST
  fixture_test!(
      checks_#{fixture_name},
      #{fixture_name.dump},
      #{options.fetch(:fixture_path).dump},
      #{cop_name.dump},
      #{options.fetch(:autocorrect)},
      RubyVersion::default()
  );
RUST
if options[:dry_run]
  puts "# #{path}"
  if options[:family]
    puts "# Append to the existing define_cops! block:"
    puts "    #{type_name} => #{cop_name.dump} => #{declaration},"
    puts
    puts callback
  else
    puts source
  end
  puts "# #{File.join(fixture, "input.rb")}"
  puts fixture_input
  puts "# #{File.join(fixture, "offenses.tsv")}"
  puts fixture_header
  if options.fetch(:autocorrect)
    puts "# #{File.join(fixture, "corrected.rb")}"
    puts fixture_input
  end
  puts "# #{FIXTURE_TESTS}"
  puts fixture_test
  exit
end

unless options[:family]
  composition = File.read(COMPOSITION_ROOT)
  module_line = "mod #{module_name};\n"
  abort "composition module marker not found" unless composition.include?("mod source_helpers;\n")
  composition = composition.sub("mod source_helpers;\n", "#{module_line}mod source_helpers;\n")
  chain = "            .chain(#{module_name}::cops())\n"
  abort "registry chain marker not found" unless composition.include?("            .chain(lint_control_flow::cops())\n")
  composition = composition.sub(
    "            .chain(lint_control_flow::cops())\n",
    "#{chain}            .chain(lint_control_flow::cops())\n"
  )
end
fixture_tests = File.read(FIXTURE_TESTS)
fixture_marker = "// New-cop generator registrations are inserted directly below this line.\n"
abort "fixture registration marker not found" unless fixture_tests.include?(fixture_marker)
fixture_tests = fixture_tests.sub(fixture_marker, "#{fixture_marker}#{fixture_test}\n")

if options[:family]
  family_source = File.read(path)
  declaration_end = family_source.index("\n}", family_source.index("define_cops! {") || 0)
  abort "family module has no define_cops! block: #{path}" unless declaration_end
  family_source.insert(
    declaration_end + 1,
    "    #{type_name} => #{cop_name.dump} => #{declaration},\n"
  )
  File.write(path, "#{family_source.rstrip}\n\n#{callback}")
else
  File.write(path, source)
  File.write(COMPOSITION_ROOT, composition)
end
File.write(FIXTURE_TESTS, fixture_tests)
FileUtils.mkdir_p(fixture)
File.write(File.join(fixture, "input.rb"), fixture_input)
File.write(File.join(fixture, "offenses.tsv"), fixture_header)
File.write(File.join(fixture, "corrected.rb"), fixture_input) if options.fetch(:autocorrect)

puts "Created #{path.delete_prefix("#{ROOT}/")}."
puts "Add upstream-derived examples, then run: ruby script/verify_cop.rb #{cop_name}"
