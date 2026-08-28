# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "pathname"
require "set"

ROOT = Pathname(__dir__).join("..").expand_path
GRAPH = ROOT.join("compatibility", "dependencies", "rubocop_dependency_graph.json")
ANNOTATIONS = ROOT.join("compatibility", "dependencies", "rubocop_dependency_annotations.json")
DEFAULT_SOURCE_ROOT = Pathname("/private/tmp/rustocop-dependency-spec-audit")

REPOSITORIES = {
  "rubocop-1.87.0" => ["https://github.com/rubocop/rubocop.git", "v1.87.0", "e5b788dba181ad94de30cfbad661c5d6aa08a4e5"],
  "ast-2.4.3" => ["https://github.com/whitequark/ast.git", "v2.4.3", "c8774c90c1e9e41719b92a52f5116554e9e72646"],
  "json-2.19.8" => ["https://github.com/ruby/json.git", "v2.19.8", "5233dd9b851a4924f793aec1a1658ed8b66a34c7"],
  "language_server-protocol-3.17.0.5" => ["https://github.com/mtsmfm/language_server-protocol-ruby.git", "v3.17.0.5", "174c0d148caab1089d1f0057d8139ca749bb0c59"],
  "lint_roller-1.1.0" => ["https://github.com/standardrb/lint_roller.git", "v1.1.0", "d4a5b99164efaf1d280fe890f256a9344a8bde45"],
  "parallel-2.1.0" => ["https://github.com/grosser/parallel.git", "v2.1.0", "cd5ba09894cd3a47dcf180ad9aedd0258c050863"],
  "parser-3.3.11.1" => ["https://github.com/whitequark/parser.git", "v3.3.11.1", "2edaaab2e9a19767c59cd1b3ad282fad32589e35"],
  "prism-1.9.0" => ["https://github.com/ruby/prism.git", "v1.9.0", "c0e37816e97e23e92524a4070e1b99a4025bc63f"],
  "racc-1.8.1" => ["https://github.com/ruby/racc.git", "v1.8.1", "5229883dca5b451c8bfd322272ccd2ca6d526695"],
  "rainbow-3.1.1" => ["https://github.com/sickill/rainbow.git", "v3.1.1", "d5e20718cfe890bf9fea9a435e72e9bfff2eab5a"],
  "regexp_parser-2.12.0" => ["https://github.com/ammar/regexp_parser.git", "v2.12.0", "55f48a1185c0dd569e709e065b81072ea0897a5b"],
  "rubocop-ast-1.49.1" => ["https://github.com/rubocop/rubocop-ast.git", "v1.49.1", "c249734326830f7413c78b77fa8cf1762a9da44e"],
  "ruby-progressbar-1.13.0" => ["https://github.com/jfelchner/ruby-progressbar.git", "releases/v1.13.0", "158e12d42a2a120234e153840aa9c2174f59c26c"],
  "unicode-display_width-3.2.0" => ["https://github.com/janlelis/unicode-display_width.git", "v3.2.0", "215328593f2e510923147880ac029b9e8cdc499c"],
  "unicode-emoji-4.2.0" => ["https://github.com/janlelis/unicode-emoji.git", "v4.2.0", "beff6fe9a2935a8c1bd417ae7e775c685d514f2e"]
}.freeze

VISIBILITIES = %i[public protected private].freeze
ATTRIBUTE_METHODS = {
  attr_reader: %i[reader],
  attr_writer: %i[writer],
  attr_accessor: %i[reader writer],
  attr: %i[reader]
}.freeze

def constant_parts(node)
  case node&.type
  when :constant_read_node, :constant_target_node
    [[node.name.to_s], false]
  when :constant_path_node, :constant_path_target_node
    parent = node.parent
    return [[node.name.to_s], true] unless parent

    parts, absolute = constant_parts(parent)
    [parts + [node.name.to_s], absolute]
  else
    [[], false]
  end
end

def static_name(node)
  case node&.type
  when :symbol_node, :string_node
    node.unescaped.to_s
  end
end

class ApiScanner
  def initialize(source, path)
    @source = source
    @path = path
    @types = {}
    @type_lines = {}
    @constants = Set.new
    @top_level_functions = Set.new
    @first_declaration_line = nil
  end

  def call
    result = Prism.parse(@source, filepath: @path)
    raise "#{@path}: #{result.errors.map(&:message).join('; ')}" unless result.success?

    scan_body(result.value, [], nil, :public, false)
    exposed_types = @types.values.reject do |type|
      name = type.fetch("name")
      namespace_only = type.keys.grep(/_methods\z/).all? { |key| type.fetch(key).empty? }
      namespace_only && @types.keys.any? { |candidate| candidate.start_with?("#{name}::") }
    end
    {
      "api" => {
        "types" => exposed_types.sort_by { |type| [type["name"], type["kind"]] }.map do |type|
          type.transform_values { |value| value.is_a?(Set) ? value.to_a.sort : value }
        end,
        "constants" => @constants.to_a.sort,
        "top_level_functions" => @top_level_functions.to_a.sort
      },
      "description" => description(exposed_types),
      "declared_names" => exposed_types.map { |type| type.fetch("name") } | @constants.to_a,
      "first_declaration_line" => @first_declaration_line
    }
  end

  private

  def scan_body(node, scope, owner, visibility, singleton)
    return visibility unless node

    children = node.type == :statements_node ? node.body : [node]
    children.compact.each do |child|
      visibility = scan_node(child, scope, owner, visibility, singleton)
    end
    visibility
  end

  def scan_node(node, scope, owner, visibility, singleton)
    case node.type
    when :program_node
      scan_body(node.statements, scope, owner, visibility, singleton)
    when :statements_node
      scan_body(node, scope, owner, visibility, singleton)
    when :module_node, :class_node
      scan_type(node, scope)
      visibility
    when :singleton_class_node
      if owner && node.expression.type == :self_node
        scan_body(node.body, scope, owner, :public, true)
      else
        scan_children(node, scope, owner, visibility, singleton)
      end
      visibility
    when :def_node
      record_method(owner, node.name.to_s, visibility, singleton, node.location.start_line)
      visibility
    when :call_node
      scan_call(node, scope, owner, visibility, singleton)
    when :alias_method_node
      record_method(owner, static_name(node.new_name), visibility, singleton, node.location.start_line)
      visibility
    when :constant_write_node, :constant_or_write_node, :constant_and_write_node,
         :constant_operator_write_node
      record_constant(scope + [node.name.to_s], node.location.start_line)
      scan_children(node, scope, owner, visibility, singleton)
      visibility
    when :constant_path_write_node
      parts, absolute = constant_parts(node.target)
      record_constant(absolute ? parts : scope + parts, node.location.start_line)
      scan_children(node, scope, owner, visibility, singleton)
      visibility
    else
      scan_children(node, scope, owner, visibility, singleton)
      visibility
    end
  end

  def scan_type(node, scope)
    parts, absolute = constant_parts(node.constant_path)
    full_parts = absolute ? parts : scope + parts
    return if full_parts.empty?

    name = full_parts.join("::")
    @types[name] ||= {
      "kind" => node.type == :class_node ? "class" : "module",
      "name" => name,
      "public_instance_methods" => Set.new,
      "protected_instance_methods" => Set.new,
      "private_instance_methods" => Set.new,
      "public_singleton_methods" => Set.new,
      "protected_singleton_methods" => Set.new,
      "private_singleton_methods" => Set.new
    }
    @type_lines[name] ||= node.location.start_line
    declaration(node.location.start_line)
    scan_body(node.body, full_parts, name, :public, false)
  end

  def scan_call(node, scope, owner, visibility, singleton)
    name = node.name
    arguments = node.arguments&.arguments || []
    if owner && node.receiver.nil? && VISIBILITIES.include?(name)
      names = arguments.filter_map { |argument| static_name(argument) }
      if names.empty?
        return name
      end
      names.each { |method_name| move_method_visibility(owner, method_name, name, singleton) }
    elsif owner && node.receiver.nil? && ATTRIBUTE_METHODS.key?(name)
      arguments.filter_map { |argument| static_name(argument) }.each do |attribute|
        ATTRIBUTE_METHODS.fetch(name).each do |mode|
          method_name = mode == :writer ? "#{attribute}=" : attribute
          record_method(owner, method_name, visibility, singleton, node.location.start_line)
        end
      end
    elsif owner && node.receiver.nil? && %i[define_method define_singleton_method].include?(name)
      method_name = static_name(arguments.first)
      record_method(owner, method_name, visibility, singleton || name == :define_singleton_method,
                    node.location.start_line)
    elsif owner && node.receiver.nil? && name == :alias_method
      record_method(owner, static_name(arguments.first), visibility, singleton, node.location.start_line)
    elsif owner && node.receiver.nil? && %i[private_class_method protected_class_method public_class_method].include?(name)
      target_visibility = name.to_s.delete_suffix("_class_method").to_sym
      arguments.filter_map { |argument| static_name(argument) }.each do |method_name|
        move_method_visibility(owner, method_name, target_visibility, true)
      end
    elsif owner && node.receiver.nil? && name == :module_function
      arguments.filter_map { |argument| static_name(argument) }.each do |method_name|
        move_method_visibility(owner, method_name, :private, false)
        record_method(owner, method_name, :public, true, node.location.start_line)
      end
    end
    scan_children(node, scope, owner, visibility, singleton)
    visibility
  end

  def scan_children(node, scope, owner, visibility, singleton)
    node.child_nodes.compact.each do |child|
      scan_node(child, scope, owner, visibility, singleton)
    end
  end

  def record_method(owner, name, visibility, singleton, line)
    return if name.to_s.empty?

    if owner
      type = @types.fetch(owner)
      key = "#{visibility}_#{singleton ? 'singleton' : 'instance'}_methods"
      type.fetch(key) << name
    else
      @top_level_functions << name
    end
    declaration(line)
  end

  def move_method_visibility(owner, name, visibility, singleton)
    return if name.to_s.empty?

    type = @types.fetch(owner)
    suffix = singleton ? "singleton_methods" : "instance_methods"
    VISIBILITIES.each { |candidate| type.fetch("#{candidate}_#{suffix}").delete(name) }
    type.fetch("#{visibility}_#{suffix}") << name
  end

  def record_constant(parts, line)
    return if parts.empty?

    @constants << parts.join("::")
    declaration(line)
  end

  def declaration(line)
    @first_declaration_line = [@first_declaration_line, line].compact.min
  end

  def description(exposed_types)
    primary_type = exposed_types.max_by { |type| type.fetch("name").count("::") }
    documented = preceding_comment(primary_type ? @type_lines[primary_type.fetch("name")] : @first_declaration_line)
    return documented if documented

    names = exposed_types.map { |type| type.fetch("name") }
    return "Defines #{names.join(', ')} and its public Ruby API." unless names.empty?
    return "Defines the constants #{@constants.to_a.sort.join(', ')}." unless @constants.empty?
    unless @top_level_functions.empty?
      return "Defines the top-level functions #{@top_level_functions.to_a.sort.join(', ')}."
    end

    "Provides load-time setup for #{File.basename(@path)}."
  end

  def preceding_comment(line)
    return unless line && line > 1

    lines = @source.lines
    index = line - 2
    block = []
    while index >= 0
      text = lines[index].to_s
      break unless text.match?(/^\s*#/) || text.strip.empty?
      block.unshift(text)
      index -= 1
    end
    paragraphs = block.map { |text| text.sub(/^\s*#\s?/, "").strip }
                      .reject { |text| text.empty? || text.start_with?("frozen_string_literal:", "typed:", "rubocop:", "@") }
                      .join(" ").split(/\n\s*\n/)
    text = paragraphs.find { |paragraph| paragraph.match?(/[A-Za-z]/) }
    return unless text

    text = text.gsub(/\s+/, " ").strip
    text = text[0, 397] + "..." if text.length > 400
    text
  end
end

class SpecIndex
  TEST_GLOBS = %w[spec/**/*.rb test/**/*.rb tests/**/*.rb].freeze

  def initialize(package, repository_root, revision)
    @package = package
    @root = repository_root
    @revision = revision
    @feature_index = Hash.new { |hash, key| hash[key] = Set.new }
    @constant_index = Hash.new { |hash, key| hash[key] = Set.new }
    @test_paths = TEST_GLOBS.flat_map { |glob| Dir.glob(@root.join(glob)) }.uniq.sort
    build
  end

  def specs_for(source_path, declared_names)
    feature = source_path.delete_prefix("lib/").delete_suffix(".rb")
    paths = Set.new
    conventional_paths(source_path).each { |path| paths << path if @root.join(path).file? }
    @feature_index.fetch(feature, []).each { |path| paths << path }
    declared_names.each do |name|
      @constant_index.fetch(name, []).each { |path| paths << path }
    end
    paths.to_a.sort.map do |path|
      absolute = @root.join(path)
      {
        "package" => @package,
        "path" => path,
        "md5" => Digest::MD5.file(absolute).hexdigest,
        "source_revision" => @revision
      }
    end
  end

  private

  def build
    @test_paths.each do |absolute_string|
      absolute = Pathname(absolute_string)
      relative = absolute.relative_path_from(@root).to_s
      source = absolute.read
      source.scan(/\brequire\s*\(?\s*["']([^"']+)["']/).flatten.each do |feature|
        @feature_index[feature] << relative
      end
      source.scan(/\b(?:[A-Z]\w*::)+[A-Z]\w*/).each do |name|
        @constant_index[name] << relative
      end
    rescue ArgumentError
      next
    end
  end

  def conventional_paths(source_path)
    relative = source_path.delete_prefix("lib/").delete_suffix(".rb")
    directory = File.dirname(relative)
    basename = File.basename(relative)
    [
      "spec/#{relative}_spec.rb",
      "test/#{relative}_test.rb",
      "test/#{relative}.rb",
      "test/#{directory}/test_#{basename}.rb",
      "test/test_#{basename}.rb",
      "tests/#{relative}_test.rb"
    ].uniq
  end
end

def package_source_path(logical_path)
  _package, relative = logical_path.split("/", 2)
  relative
end

options = {source_root: DEFAULT_SOURCE_ROOT, check: false}
OptionParser.new do |parser|
  parser.on("--source-root PATH") { |path| options[:source_root] = Pathname(path).expand_path }
  parser.on("--check") { options[:check] = true }
end.parse!

gem "prism", "=1.9.0"
require "prism"

graph = JSON.parse(GRAPH.read)
existing = JSON.parse(ANNOTATIONS.read)
manual_reviewed_prefix = existing.fetch("manual_reviewed_prefix", 40)
manual_paths = graph.fetch("nodes").first(manual_reviewed_prefix).map { |node| node.fetch("path") }.to_set
existing_by_path = existing.fetch("rows").select { |row| manual_paths.include?(row.fetch("path")) }
                              .to_h { |row| [row.fetch("path"), row] }

repositories = REPOSITORIES.to_h do |package, (url, tag, revision)|
  [package, {"url" => url, "tag" => tag, "revision" => revision}]
end

spec_indices = REPOSITORIES.to_h do |package, (_url, _tag, revision)|
  repository_root = options.fetch(:source_root).join(package)
  abort "missing exact source checkout #{repository_root}" unless repository_root.directory?
  actual_revision = `git -C #{repository_root.to_s.dump} rev-parse HEAD`.strip
  abort "source revision mismatch for #{package}: #{actual_revision}" unless actual_revision == revision
  [package, SpecIndex.new(package, repository_root, revision)]
end

specs_by_full_name = graph.fetch("packages").to_h do |package|
  spec = Gem::Specification.find_by_name(package.fetch("name"), "=#{package.fetch('version')}")
  [package.fetch("full_name"), spec]
end

rows = graph.fetch("nodes").map do |node|
  logical_path = node.fetch("path")
  package = logical_path.split("/", 2).first
  existing_row = existing_by_path[logical_path]
  next existing_row if existing_row

  spec = specs_by_full_name.fetch(package)
  relative = package_source_path(logical_path)
  source_path = Pathname(spec.full_gem_path).join(relative)
  source = source_path.binread.force_encoding(Encoding::UTF_8).scrub
  extracted = ApiScanner.new(source, logical_path).call
  {
    "path" => logical_path,
    "md5" => node.fetch("md5"),
    "api" => extracted.fetch("api"),
    "description" => extracted.fetch("description"),
    "associated_specs" => spec_indices.fetch(package).specs_for(
      relative,
      extracted.fetch("declared_names")
    )
  }
end

document = {
  "schema_version" => 1,
  "manual_reviewed_prefix" => manual_reviewed_prefix,
  "source_repositories" => repositories,
  "rows" => rows
}
generated = JSON.pretty_generate(document) << "\n"

if options.fetch(:check)
  unless ANNOTATIONS.binread == generated.b
    warn "dependency annotations: committed=#{Digest::SHA256.hexdigest(ANNOTATIONS.binread)} generated=#{Digest::SHA256.hexdigest(generated)}"
    abort "dependency annotations are stale"
  end
  puts "dependency annotations are current (#{rows.length} files)"
else
  ANNOTATIONS.binwrite(generated)
  puts "wrote #{ANNOTATIONS.relative_path_from(ROOT)} (#{rows.length} files)"
end
