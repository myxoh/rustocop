# frozen_string_literal: true

require "fileutils"
require "optparse"

ROOT = File.expand_path("..", __dir__)
PRISM_ROOT = File.join(ROOT, "crates/rustocop/src/cops/prism")
COMPOSITION_ROOT = File.join(PRISM_ROOT, "mod.rs")

options = {
  dry_run: false,
  family: nil,
  node_cast: "as_call_node",
  rubocop_callbacks: ["on_send"],
  restrict_methods: []
}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/new_cop.rb Department/CopName KIND [options]"
  parser.on("--dry-run", "print the generated source without writing") { options[:dry_run] = true }
  parser.on("--family MODULE", "append the cop to an existing Prism family module") do |family|
    options[:family] = family
  end
  parser.on("--node-cast METHOD", "Prism Node cast for node cops") { |cast| options[:node_cast] = cast }
  parser.on("--callbacks LIST", "comma-separated RuboCop callbacks for rubocop cops") do |callbacks|
    options[:rubocop_callbacks] = callbacks.split(",").map(&:strip)
  end
  parser.on("--restrict-methods LIST", "comma-separated on_send method names") do |methods|
    options[:restrict_methods] = methods.split(",").map(&:strip)
  end
end.parse!

cop_name = ARGV.shift or abort "missing cop name"
kind = ARGV.shift || "call"
abort "kind must be call, node, any_node, source, or rubocop" unless %w[call node any_node source rubocop].include?(kind)
abort "cop name must contain only Ruby constant-name characters" unless cop_name.match?(/\A[A-Z][A-Za-z0-9]*\/[A-Z][A-Za-z0-9]*\z/)
department, short_name = cop_name.split("/", 2)
abort "cop name must look like Department/CopName" unless department && short_name
if kind == "node" && !options.fetch(:node_cast).match?(/\Aas_[a-z0-9_]+_node\z/)
  abort "node cast must look like as_if_node"
end

snake = short_name.gsub(/([a-z\d])([A-Z])/, '\1_\2').downcase
module_name = options[:family] || "#{department.downcase}_#{snake}"
abort "family module must use snake_case" unless module_name.match?(/\A[a-z][a-z0-9_]*\z/)
type_name = short_name.gsub(/[^A-Za-z0-9]/, "")
callback_name = options[:family] ? snake : "check"
rule_name = "#{type_name}Rule"
path = File.join(PRISM_ROOT, "#{module_name}.rs")
if options[:family]
  abort "family module does not exist: #{path}" unless File.exist?(path)
else
  abort "#{path} already exists" if File.exist?(path)
end

rubocop_node_types = {
  "on_send" => "CallNode",
  "on_if" => "IfNode",
  "on_unless" => "UnlessNode",
  "on_block" => "BlockNode",
  "on_while" => "WhileNode",
  "on_until" => "UntilNode",
  "on_for" => "ForNode",
  "on_def" => "DefNode",
  "on_class" => "ClassNode",
  "on_module" => "ModuleNode",
  "on_array" => "ArrayNode",
  "on_hash" => "HashNode",
  "on_casgn" => "Node"
}.freeze

if kind == "rubocop"
  unknown = options.fetch(:rubocop_callbacks) - rubocop_node_types.keys
  abort "unsupported RuboCop callbacks: #{unknown.join(', ')}" unless unknown.empty?
  abort "at least one RuboCop callback is required" if options.fetch(:rubocop_callbacks).empty?
  if options.fetch(:restrict_methods).any?
    abort "--restrict-methods requires callbacks to be exactly on_send" unless options.fetch(:rubocop_callbacks) == ["on_send"]
    abort "restricted method names must be Ruby identifiers or operators" unless options.fetch(:restrict_methods).all? { |method| method.match?(/\A[^\s,]+\z/) }
  end
end

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
           when "rubocop"
             methods = options.fetch(:rubocop_callbacks).map do |name|
               node_type = rubocop_node_types.fetch(name)
               body = if name == "on_send"
                        <<~RUST.chomp
                          return_unless!(match_call(node).named(b"replace_me").matches());
                          add_offense!(self, node.message_loc(), message: "Replace with the upstream RuboCop message.", |corrector| {
                              corrector.replace(node.message_loc(), "replacement");
                          });
                        RUST
                      else
                        "let _ = node;"
                      end
               <<~RUST
                 fn #{name}(&mut self, node: &ruby_prism::#{node_type}<'_>) {
                 #{body.lines.map { |line| "    #{line}" }.join.rstrip}
                 }
               RUST
             end.join("\n")
             <<~RUST
               impl #{rule_name}<'_, '_, '_> {
               #{methods.lines.map { |line| "    #{line}" }.join.rstrip}
               }
             RUST
           end

declaration = if kind == "node"
                "node(#{options.fetch(:node_cast)}, #{callback_name})"
              elsif kind == "rubocop"
                callback_list = if options.fetch(:restrict_methods).any?
                                  methods = options.fetch(:restrict_methods).map { |method| "b#{method.dump}" }.join(", ")
                                  "on_send restrict [#{methods}]"
                                else
                                  options.fetch(:rubocop_callbacks).join(", ")
                                end
                "rubocop_callbacks(#{rule_name}, [#{callback_list}])"
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
  puts "# Unit contract: spec/fixtures/cops/#{cop_name}/unit/cases.jsonl"
  puts "# Refresh with: bundle exec ruby script/generate_unit_fixtures.rb"
  exit
end

unless options[:family]
  composition = File.read(COMPOSITION_ROOT)
  module_marker = "cop_modules!(\n"
  abort "composition module marker not found" unless composition.include?(module_marker)
  composition = composition.sub(module_marker, "#{module_marker}    #{module_name},\n")
end
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
puts "Created #{path.delete_prefix("#{ROOT}/")}."
puts "Add upstream-derived examples, then run: ruby script/verify_cop.rb #{cop_name}"
