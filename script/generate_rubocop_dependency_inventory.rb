# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "pathname"
require "set"

ROOT = Pathname(__dir__).join("..").expand_path
DEFAULT_OUTPUT_DIR = ROOT.join("compatibility", "dependencies")
DEFAULT_ANNOTATIONS = DEFAULT_OUTPUT_DIR.join("rubocop_dependency_annotations.json")
DEFAULT_RUST_EQUIVALENTS = DEFAULT_OUTPUT_DIR.join("rubocop_dependency_rust_equivalents.json")
ROOT_PACKAGE = "rubocop"
ROOT_VERSION = "1.87.0"
LOCKED_PACKAGE_VERSIONS = {
  "rubocop" => "1.87.0",
  "ast" => "2.4.3",
  "json" => "2.19.8",
  "language_server-protocol" => "3.17.0.5",
  "lint_roller" => "1.1.0",
  "parallel" => "2.1.0",
  "parser" => "3.3.11.1",
  "prism" => "1.9.0",
  "racc" => "1.8.1",
  "rainbow" => "3.1.1",
  "regexp_parser" => "2.12.0",
  "rubocop-ast" => "1.49.1",
  "ruby-progressbar" => "1.13.0",
  "unicode-display_width" => "3.2.0",
  "unicode-emoji" => "4.2.0"
}.freeze

# The report deliberately includes the complete runtime gem closure rooted at
# the locked RuboCop release, while stopping at Ruby's standard library. Files
# are limited to loadable Ruby library sources under each gem's require paths.
CSV_HEADERS = [
  "RuboCop file",
  "MD5 hash",
  "path",
  "known number of dependencies",
  "actual dependency paths",
  "classes and interfaces exposed",
  "description",
  "associated spec",
  "Rust equivalent (if it exists)",
  "Detailed reason why a Rust equivalent cannot be built",
  "Known workarounds if you need the current file to implement your cop or higher level library"
].freeze

# Capture Ruby's built-in and already-loaded standard-library constants before
# the audited package closure is activated. JSON is removed because RuboCop
# declares the json gem as a runtime dependency and its files belong in this
# inventory. The explicit tail covers common optional standard-library classes
# that may not have been loaded by this generator yet.
RUBY_BOUNDARY_CONSTANTS = (
  Object.constants.map(&:to_s).to_set |
  Set.new(%w[BigDecimal Date DateTime OpenStruct SecureRandom StringIO Tempfile URI YAML])
).subtract(%w[JSON]).freeze

FileEntry = Data.define(
  :spec,
  :absolute_path,
  :logical_path,
  :require_relative_path,
  :content,
  :md5
)

Reference = Data.define(:parts, :absolute, :lexical_scope)

class RubySourceScanner
  LOAD_METHODS = %i[require require_relative autoload].freeze
  CONSTANT_WRITE_TYPES = %i[
    constant_write_node
    constant_or_write_node
    constant_and_write_node
    constant_operator_write_node
  ].freeze

  attr_reader :definitions, :references, :loads, :dynamic_loads, :parse_errors

  def initialize(source, path)
    @source = source
    @path = path
    @definitions = Set.new
    @references = []
    @loads = []
    @dynamic_loads = []
    @parse_errors = []
  end

  def scan
    result = Prism.parse(@source, filepath: @path)
    result.errors.each { |error| @parse_errors << error.message }
    scan_definitions(result.value, [])
    scan_references(result.value, [])
    self
  rescue StandardError => e
    @parse_errors << "#{e.class}: #{e.message}"
    self
  end

  private

  def scan_definitions(node, scope)
    return unless node

    case node.type
    when :module_node, :class_node
      name = qualified_definition_name(node.constant_path, scope)
      @definitions << name.join("::") unless name.empty?
      scan_definitions(node.body, name.empty? ? scope : name)
      return
    when *CONSTANT_WRITE_TYPES
      name = scope + [node.name.to_s]
      @definitions << name.join("::")
      scan_definitions(node.value, scope)
      return
    when :constant_path_write_node
      name = qualified_definition_name(node.target, scope)
      @definitions << name.join("::") unless name.empty?
      scan_definitions(node.value, scope)
      return
    end

    node.child_nodes.compact.each { |child| scan_definitions(child, scope) }
  end

  def scan_references(node, scope)
    return unless node

    case node.type
    when :module_node
      name = qualified_definition_name(node.constant_path, scope)
      scan_references(node.body, name.empty? ? scope : name)
      return
    when :class_node
      name = qualified_definition_name(node.constant_path, scope)
      scan_references(node.superclass, scope)
      scan_references(node.body, name.empty? ? scope : name)
      return
    when *CONSTANT_WRITE_TYPES, :constant_path_write_node
      scan_references(node.value, scope)
      return
    when :constant_path_node, :constant_read_node
      record_constant_reference(node, scope)
      return
    when :call_node
      record_load(node)
    end

    node.child_nodes.compact.each { |child| scan_references(child, scope) }
  end

  def qualified_definition_name(node, scope)
    parts, absolute = constant_parts(node)
    return [] if parts.empty?
    return parts if absolute

    scope + parts
  end

  def record_constant_reference(node, scope)
    parts, absolute = constant_parts(node)
    return if parts.empty?

    @references << Reference.new(parts:, absolute:, lexical_scope: scope.dup)
  end

  def constant_parts(node)
    case node&.type
    when :constant_read_node, :constant_target_node
      [[node.name.to_s], false]
    when :constant_path_node, :constant_path_target_node
      parent = node.parent
      if parent.nil?
        [[node.name.to_s], true]
      else
        parent_parts, parent_absolute = constant_parts(parent)
        return [[], false] if parent_parts.empty?

        [parent_parts + [node.name.to_s], parent_absolute]
      end
    else
      [[], false]
    end
  end

  def record_load(node)
    return unless LOAD_METHODS.include?(node.name)
    return unless node.receiver.nil? || kernel_receiver?(node.receiver)

    arguments = node.arguments&.arguments || []
    argument = node.name == :autoload ? arguments[1] : arguments[0]
    relative, value = static_load_path(argument)
    if value
      kind = relative ? "#{node.name}_relative" : node.name.to_s
      @loads << [kind, value]
    else
      @dynamic_loads << node.location.slice.lines.first.to_s.strip
    end
  end

  def kernel_receiver?(node)
    node.type == :constant_read_node && node.name == :Kernel
  end

  def static_string(node)
    return unless node

    case node.type
    when :string_node
      node.unescaped
    when :symbol_node
      node.unescaped.to_s
    end
  end

  def static_load_path(node)
    value = static_string(node)
    return [false, value] if value
    return [false, nil] unless node&.type == :call_node
    return [false, nil] unless node.name == :expand_path
    return [false, nil] unless node.receiver&.type == :constant_read_node && node.receiver.name == :File

    arguments = node.arguments&.arguments || []
    path = static_string(arguments[0])
    base = arguments[1]
    relative_to_directory = base&.type == :call_node && base.name == :__dir__ && base.receiver.nil?
    return [false, nil] unless path && relative_to_directory

    [true, path]
  end
end

class StronglyConnectedComponents
  def initialize(nodes, dependencies)
    @nodes = nodes
    @dependencies = dependencies
    @next_index = 0
    @indices = {}
    @low_links = {}
    @stack = []
    @on_stack = Set.new
    @components = []
  end

  def call
    @nodes.sort.each { |node| connect(node) unless @indices.key?(node) }
    @components
  end

  private

  def connect(node)
    @indices[node] = @next_index
    @low_links[node] = @next_index
    @next_index += 1
    @stack << node
    @on_stack << node

    @dependencies.fetch(node, []).sort.each do |dependency|
      unless @indices.key?(dependency)
        connect(dependency)
        @low_links[node] = [@low_links[node], @low_links[dependency]].min
        next
      end

      if @on_stack.include?(dependency)
        @low_links[node] = [@low_links[node], @indices[dependency]].min
      end
    end

    return unless @low_links[node] == @indices[node]

    component = []
    loop do
      member = @stack.pop
      @on_stack.delete(member)
      component << member
      break if member == node
    end
    @components << component.sort
  end
end

class DependencyInventory
  attr_reader :entries, :dependencies, :edge_kinds, :external_loads, :dynamic_loads,
              :parse_errors, :packages, :ranks, :components

  def initialize(
    annotation_path: DEFAULT_ANNOTATIONS,
    rust_equivalence_path: DEFAULT_RUST_EQUIVALENTS
  )
    @packages = runtime_package_closure
    activate_prism!
    @entries = collect_entries
    @entry_by_absolute_path = @entries.to_h { |entry| [entry.absolute_path, entry] }
    @entry_by_logical_path = @entries.to_h { |entry| [entry.logical_path, entry] }
    @feature_index = build_feature_index
    @package_entrypoints = build_package_entrypoints
    @scanners = scan_sources
    @dependencies = @entries.to_h { |entry| [entry.logical_path, Set.new] }
    @edge_kinds = Hash.new { |hash, key| hash[key] = Set.new }
    @external_loads = Hash.new { |hash, key| hash[key] = Set.new }
    @dynamic_loads = Hash.new { |hash, key| hash[key] = Set.new }
    @parse_errors = {}
    build_edges
    build_ordering
    @annotation_document = load_annotations(annotation_path)
    @annotations = @annotation_document.fetch("rows").to_h do |row|
      [row.fetch("path"), row]
    end
    @rust_equivalence_document = load_json_document(
      rust_equivalence_path,
      default: {"schema_version" => 1, "rows" => []}
    )
    @rust_equivalents = @rust_equivalence_document.fetch("rows").to_h do |row|
      [row.fetch("path"), row]
    end
  end

  def ordered_entries
    @entries.sort_by do |entry|
      [@ranks.fetch(entry.logical_path), @dependencies.fetch(entry.logical_path).length,
       entry.logical_path]
    end
  end

  def validate!
    paths = @entries.map(&:logical_path)
    errors = []
    errors << "duplicate inventory paths" unless paths.uniq.length == paths.length
    errors << "source parse failures: #{@parse_errors.keys.sort.join(', ')}" unless @parse_errors.empty?

    known_paths = paths.to_set
    @entries.each do |entry|
      errors << "invalid MD5 for #{entry.logical_path}" unless entry.md5.match?(/\A[0-9a-f]{32}\z/)
      actual_md5 = Digest::MD5.hexdigest(File.binread(entry.absolute_path))
      errors << "stale MD5 for #{entry.logical_path}" unless actual_md5 == entry.md5

      unknown = @dependencies.fetch(entry.logical_path).reject { |path| known_paths.include?(path) }
      errors << "unknown dependencies for #{entry.logical_path}: #{unknown.to_a.sort.join(', ')}" unless unknown.empty?
      if @dependencies.fetch(entry.logical_path).include?(entry.logical_path)
        errors << "self dependency for #{entry.logical_path}"
      end
    end

    ordered_ranks = ordered_entries.map { |entry| @ranks.fetch(entry.logical_path) }
    errors << "dependency ranks are not monotonically ordered" unless ordered_ranks == ordered_ranks.sort
    errors << "missing dependency ranks" unless @ranks.keys.to_set == known_paths
    validate_annotations(errors)
    validate_rust_equivalents(errors)

    raise errors.join("\n") unless errors.empty?

    true
  end

  def to_csv
    rows = [CSV_HEADERS]
    ordered_entries.each do |entry|
      dependency_paths = @dependencies.fetch(entry.logical_path).to_a.sort
      annotation = @annotations[entry.logical_path]
      rust_equivalence = @rust_equivalents[entry.logical_path]
      rows << [
        File.basename(entry.logical_path),
        entry.md5,
        entry.logical_path,
        dependency_paths.length,
        JSON.generate(dependency_paths),
        annotation ? JSON.generate(annotation.fetch("api")) : "",
        annotation ? annotation.fetch("description") : "",
        annotation ? JSON.generate(annotation.fetch("associated_specs")) : "",
        format_rust_equivalent(rust_equivalence),
        rust_equivalence&.fetch("cannot_build_reason", ""),
        format_known_workarounds(rust_equivalence)
      ]
    end
    rows.map { |row| csv_line(row) }.join
  end

  def to_graph_json
    component_by_path = {}
    @components.each_with_index do |component, index|
      component.each { |path| component_by_path[path] = index }
    end

    package_rows = @packages.map do |spec|
      {
        "name" => spec.name,
        "version" => spec.version.to_s,
        "full_name" => spec.full_name,
        "runtime_dependencies" => spec.runtime_dependencies.sort_by(&:name).map do |dependency|
          {
            "name" => dependency.name,
            "requirement" => dependency.requirement.to_s
          }
        end
      }
    end

    nodes = ordered_entries.map do |entry|
      path = entry.logical_path
      dependency_paths = @dependencies.fetch(path).to_a.sort
      {
        "file" => File.basename(path),
        "md5" => entry.md5,
        "path" => path,
        "dependency_count" => dependency_paths.length,
        "dependencies" => dependency_paths,
        "dependency_rank" => @ranks.fetch(path),
        "strongly_connected_component" => component_by_path.fetch(path)
      }
    end

    edges = @edge_kinds.keys.sort.map do |source, target|
      {
        "source" => source,
        "target" => target,
        "kinds" => @edge_kinds.fetch([source, target]).to_a.sort
      }
    end

    document = {
      "schema_version" => 1,
      "root" => {
        "package" => ROOT_PACKAGE,
        "version" => ROOT_VERSION
      },
      "scope" => {
        "included" => "Ruby files in require paths of RuboCop's complete declared runtime gem closure",
        "excluded" => [
          "Ruby standard-library and default-library files not in the declared runtime gem closure",
          "specification, test, documentation, executable, generated native binary, and non-Ruby files",
          "optional undeclared integrations"
        ],
        "edge_sources" => [
          "static require",
          "static require_relative",
          "static autoload",
          "resolvable Ruby constant reference"
        ]
      },
      "ordering" => {
        "direction" => "most independent to least independent",
        "algorithm" => "dependency rank over the strongly connected component DAG; package-entrypoint bootstrap loads are retained as graph edges but excluded from ranking; ties by direct dependency count and path"
      },
      "summary" => {
        "packages" => package_rows.length,
        "files" => nodes.length,
        "edges" => edges.length,
        "strongly_connected_components" => @components.length,
        "cyclic_components" => @components.count { |component| component.length > 1 },
        "package_bootstrap_edges" => @edge_kinds.count do |_edge, kinds|
          kinds.any? { |kind| kind.end_with?(":package_bootstrap") }
        end,
        "parse_error_files" => @parse_errors.length,
        "external_static_load_features" => @external_loads.length,
        "dynamic_load_sites" => @dynamic_loads.values.sum(&:length),
        "annotated_files" => @annotations.length,
        "unannotated_files" => nodes.length - @annotations.length,
        "rust_equivalence_audited_files" => @rust_equivalents.values.count do |row|
          row.fetch("review_status", "complete") == "complete"
        end,
        "rust_equivalence_candidate_files" => @rust_equivalents.values.count do |row|
          row["review_status"] == "candidate_pending"
        end,
        "rust_equivalence_unaudited_files" => nodes.length - @rust_equivalents.values.count do |row|
          row.fetch("review_status", "complete") == "complete"
        end
      },
      "packages" => package_rows,
      "external_static_loads" => @external_loads.keys.sort.to_h do |feature|
        [feature, @external_loads.fetch(feature).to_a.sort]
      end,
      "dynamic_loads" => @dynamic_loads.keys.sort.to_h do |path|
        [path, @dynamic_loads.fetch(path).to_a.sort]
      end,
      "parse_errors" => @parse_errors.sort.to_h,
      "nodes" => nodes,
      "edges" => edges
    }

    JSON.pretty_generate(document) << "\n"
  end

  private

  def load_annotations(path)
    load_json_document(
      path,
      default: {"schema_version" => 1, "source_repositories" => {}, "rows" => []}
    )
  end

  def load_json_document(path, default:)
    return default unless path.file?

    JSON.parse(path.read)
  rescue JSON::ParserError => e
    raise "invalid inventory JSON at #{path}: #{e.message}"
  end

  def validate_annotations(errors)
    unless @annotation_document.fetch("schema_version", nil) == 1
      errors << "annotation schema_version must be 1"
    end

    rows = @annotation_document.fetch("rows", [])
    expected_prefix = ordered_entries.first(rows.length).map(&:logical_path)
    actual_paths = rows.map { |row| row.fetch("path", nil) }
    unless actual_paths == expected_prefix
      errors << "annotations must be a contiguous dependency-ordered prefix"
    end
    errors << "duplicate annotation paths" unless actual_paths.uniq.length == actual_paths.length

    repositories = @annotation_document.fetch("source_repositories", {})
    rows.each do |row|
      path = row.fetch("path", nil)
      entry = @entry_by_logical_path[path]
      unless entry
        errors << "annotation references unknown path #{path.inspect}"
        next
      end

      errors << "annotation MD5 mismatch for #{path}" unless row["md5"] == entry.md5
      errors << "annotation API missing for #{path}" unless valid_api?(row["api"])
      if row["description"].to_s.strip.empty?
        errors << "annotation description missing for #{path}"
      end
      validate_associated_specs(row, repositories, errors)
    end
  end

  def valid_api?(api)
    api.is_a?(Hash) && api["types"].is_a?(Array) && api["constants"].is_a?(Array) &&
      api["top_level_functions"].is_a?(Array)
  end

  def validate_associated_specs(row, repositories, errors)
    path = row.fetch("path", nil)
    specs = row["associated_specs"]
    unless specs.is_a?(Array)
      errors << "associated_specs must be an array for #{path}"
      return
    end

    specs.each do |spec|
      package = spec["package"]
      repository = repositories[package]
      errors << "unknown spec package #{package.inspect} for #{path}" unless repository
      errors << "spec path missing for #{path}" if spec["path"].to_s.empty?
      errors << "spec MD5 invalid for #{path}" unless spec["md5"].to_s.match?(/\A[0-9a-f]{32}\z/)
      unless repository && spec["source_revision"] == repository["revision"]
        errors << "spec revision mismatch for #{path}"
      end
    end
  end

  def validate_rust_equivalents(errors)
    unless @rust_equivalence_document.fetch("schema_version", nil) == 1
      errors << "Rust equivalence schema_version must be 1"
    end

    rows = @rust_equivalence_document.fetch("rows", [])
    annotation_paths = @annotation_document.fetch("rows").map { |row| row["path"] }
    actual_paths = rows.map { |row| row.fetch("path", nil) }
    unless actual_paths == annotation_paths.first(rows.length)
      errors << "Rust equivalence audits must follow source-annotation order"
    end
    errors << "duplicate Rust equivalence paths" unless actual_paths.uniq.length == actual_paths.length

    rows.each do |row|
      path = row.fetch("path", nil)
      annotation = @annotations[path]
      unless annotation
        errors << "Rust equivalence audit references unannotated path #{path.inspect}"
        next
      end

      errors << "Rust equivalence MD5 mismatch for #{path}" unless row["md5"] == annotation["md5"]
      review_status = row.fetch("review_status", "complete")
      if review_status == "candidate_pending"
        candidates = row["candidates"]
        unless candidates.is_a?(Array) && !candidates.empty? && candidates.all? do |candidate|
          candidate.is_a?(Hash) && candidate["paths"].is_a?(Array) &&
            candidate["paths"].all? { |candidate_path| ROOT.join(candidate_path).file? }
        end
          errors << "pending Rust equivalence candidate evidence invalid for #{path}"
        end
        next
      end
      unless review_status == "complete"
        errors << "unknown Rust equivalence review status #{review_status.inspect} for #{path}"
        next
      end

      value = row["rust_equivalent"]
      reason = row["cannot_build_reason"]
      if reason && (value != "N/A" || reason.to_s.strip.empty?)
        errors << "cannot_build_reason is valid only for an N/A row and must be nonblank for #{path}"
      end
      workarounds = row["known_workarounds"]
      if workarounds && (!workarounds.is_a?(Array) || workarounds.empty? ||
          workarounds.any? { |workaround| workaround.to_s.strip.empty? })
        errors << "known_workarounds must be a nonempty array of nonblank descriptions for #{path}"
      end
      next if %w[N/A not_necessary].include?(value)

      unless valid_positive_rust_equivalent?(value)
        errors << "Rust equivalent for #{path} must be N/A, not_necessary, or a verified equivalence object"
      end
    end
  end

  def valid_positive_rust_equivalent?(value)
    return false unless value.is_a?(Hash)

    paths = value["paths"]
    verification = value["verification"]
    paths.is_a?(Array) && !paths.empty? && paths.all? do |path|
      path.is_a?(String) && !path.empty? && ROOT.join(path).file?
    end && verification.is_a?(Hash) &&
      !verification["api_identity_evidence"].to_s.strip.empty? &&
      !verification["behavior_identity_evidence"].to_s.strip.empty?
  end

  def format_rust_equivalent(row)
    return "" unless row
    return "" unless row.fetch("review_status", "complete") == "complete"

    value = row.fetch("rust_equivalent")
    %w[N/A not_necessary].include?(value) ? value : value.fetch("paths").join("; ")
  end

  def format_known_workarounds(row)
    return "" unless row

    workarounds = row.fetch("known_workarounds", [])
    workarounds.empty? ? "" : JSON.generate(workarounds)
  end

  def csv_line(fields)
    fields.map do |field|
      value = field.to_s
      if value.match?(/[",\r\n]/)
        %Q{"#{value.gsub('"', '""')}"}
      else
        value
      end
    end.join(",") << "\n"
  end

  def runtime_package_closure
    root = Gem::Specification.find_by_name(ROOT_PACKAGE, "=#{ROOT_VERSION}")
    found = {}
    queue = [root]

    until queue.empty?
      spec = queue.shift
      next if found.key?(spec.name)

      found[spec.name] = spec
      spec.runtime_dependencies.sort_by(&:name).each do |dependency|
        locked_version = LOCKED_PACKAGE_VERSIONS.fetch(dependency.name) do
          abort "runtime dependency #{dependency.name.inspect} is absent from LOCKED_PACKAGE_VERSIONS"
        end
        unless dependency.requirement.satisfied_by?(Gem::Version.new(locked_version))
          abort "locked #{dependency.name} #{locked_version} does not satisfy #{dependency.requirement}"
        end

        child = Gem::Specification.find_by_name(dependency.name, "=#{locked_version}")
        queue << child
      rescue Gem::MissingSpecError => e
        abort "missing runtime dependency #{dependency.name.inspect}: #{e.message}"
      end
    end

    unexpected = LOCKED_PACKAGE_VERSIONS.keys - found.keys
    abort "locked packages are outside the RuboCop runtime closure: #{unexpected.sort.join(', ')}" unless unexpected.empty?

    found.values.sort_by { |spec| [spec.name == ROOT_PACKAGE ? 0 : 1, spec.name] }
  end

  def activate_prism!
    prism = @packages.find { |spec| spec.name == "prism" }
    abort "RuboCop runtime dependency closure does not contain prism" unless prism

    gem "prism", "=#{prism.version}"
    require "prism"
  end

  def collect_entries
    @packages.flat_map do |spec|
      gem_root = Pathname(spec.full_gem_path).expand_path
      spec.require_paths.flat_map do |require_path|
        root = Pathname(require_path)
        root = gem_root.join(root) unless root.absolute?
        root = root.expand_path
        next [] unless root.to_s.start_with?("#{gem_root}/") || root == gem_root
        next [] unless root.directory?

        Dir.glob(root.join("**", "*.rb")).sort.filter_map do |path_string|
          path = Pathname(path_string).expand_path
          next unless path.file?

          content = path.binread
          relative_to_gem = path.relative_path_from(gem_root).to_s
          relative_to_require = path.relative_path_from(root).to_s
          FileEntry.new(
            spec:,
            absolute_path: path.to_s,
            logical_path: "#{spec.full_name}/#{relative_to_gem}",
            require_relative_path: relative_to_require,
            content:,
            md5: Digest::MD5.hexdigest(content)
          )
        end
      end
    end.uniq { |entry| entry.absolute_path }.sort_by(&:logical_path)
  end

  def build_feature_index
    index = Hash.new { |hash, key| hash[key] = [] }
    @entries.each do |entry|
      feature = entry.require_relative_path.delete_suffix(".rb")
      index[feature] << entry
      index["#{feature}.rb"] << entry
    end
    index.each_value { |entries| entries.sort_by! { |entry| package_priority(entry.spec) } }
    index
  end

  def build_package_entrypoints
    @packages.each_with_object(Set.new) do |spec, entrypoints|
      candidates = [
        spec.name,
        spec.name.tr("-", "_"),
        spec.name.tr("-", "/")
      ]
      candidates.each do |feature|
        entry = @feature_index[feature]&.find { |candidate| candidate.spec.name == spec.name }
        entrypoints << entry.logical_path if entry
      end
    end
  end

  def package_priority(spec)
    [spec.name == ROOT_PACKAGE ? 0 : 1, spec.name, spec.version.to_s]
  end

  def scan_sources
    @entries.to_h do |entry|
      source = entry.content.dup.force_encoding(Encoding::UTF_8)
      source = source.scrub
      scanner = RubySourceScanner.new(source, entry.logical_path).scan
      [entry.logical_path, scanner]
    end
  end

  def build_edges
    definition_candidates = Hash.new { |hash, key| hash[key] = Set.new }
    @scanners.each do |path, scanner|
      scanner.definitions.each { |definition| definition_candidates[definition] << path }
      @parse_errors[path] = scanner.parse_errors unless scanner.parse_errors.empty?
    end
    definition_index = definition_candidates.to_h do |definition, paths|
      [definition, canonical_definition_paths(definition, paths)]
    end

    @entries.each do |entry|
      source_path = entry.logical_path
      scanner = @scanners.fetch(source_path)

      scanner.loads.each do |kind, feature|
        target = resolve_load(entry, kind, feature)
        if target
          add_edge(source_path, target.logical_path, kind)
        else
          @external_loads[feature] << source_path
        end
      end
      resolve_static_globs(entry).each do |target|
        add_edge(source_path, target.logical_path, "require_relative:glob")
      end
      scanner.dynamic_loads.each { |site| @dynamic_loads[source_path] << site }

      scanner.references.each do |reference|
        resolve_constant(reference, definition_index).each do |target_path|
          add_edge(source_path, target_path, "constant")
        end
      end
    end
  end

  def resolve_load(entry, kind, feature)
    if kind.end_with?("_relative")
      candidate = Pathname(entry.absolute_path).dirname.join(feature)
      candidates = [candidate, Pathname("#{candidate}.rb")]
      target = candidates.find(&:file?)
      return @entry_by_absolute_path[target.expand_path.to_s] if target
    end

    @feature_index[feature]&.first
  end

  def resolve_static_globs(entry)
    source = entry.content.dup.force_encoding(Encoding::UTF_8).scrub
    patterns = source.scan(
      /Dir\[\s*File\.expand_path\(\s*(['"])(.*?)\1\s*,\s*__FILE__\s*\)\s*\]/m
    ).map { |_quote, pattern| pattern }

    patterns.flat_map do |pattern|
      Dir.glob(File.expand_path(pattern, entry.absolute_path)).filter_map do |path|
        @entry_by_absolute_path[Pathname(path).expand_path.to_s]
      end
    end.uniq(&:logical_path)
  end

  def resolve_constant(reference, definition_index)
    candidates = if reference.absolute
                   [reference.parts.join("::")]
                 else
                   reference.lexical_scope.length.downto(0).map do |length|
                     (reference.lexical_scope.first(length) + reference.parts).join("::")
                   end
                 end

    match = candidates.find { |candidate| definition_index.key?(candidate) }
    return [] if match && !match.include?("::") && RUBY_BOUNDARY_CONSTANTS.include?(match)

    match ? definition_index.fetch(match) : []
  end

  # Ruby namespace modules are reopened throughout RuboCop. Treating every
  # reopening as the implementation of `RuboCop`, `Cop`, or `AST` creates
  # thousands of false edges. Prefer the conventional file that owns the full
  # constant path (for example RuboCop::Cop::Base -> rubocop/cop/base.rb), then
  # fall back to the shallowest matching basename. The complete candidate set
  # remains derivable from the source scanner; the graph edge names the
  # canonical implementation file a developer would need to translate first.
  def canonical_definition_paths(definition, paths)
    expected_feature = definition.split("::").map { |part| underscore_constant(part) }.join("/")
    entries = paths.map { |path| @entry_by_logical_path.fetch(path) }
    exact = entries.select do |entry|
      entry.require_relative_path.delete_suffix(".rb") == expected_feature
    end
    selected = exact

    if selected.empty?
      expected_basename = "#{underscore_constant(definition.split('::').last)}.rb"
      selected = entries.select do |entry|
        File.basename(entry.require_relative_path) == expected_basename
      end
    end

    selected = entries if selected.empty?
    minimum_depth = selected.map { |entry| entry.require_relative_path.count("/") }.min
    selected.select { |entry| entry.require_relative_path.count("/") == minimum_depth }
            .sort_by(&:logical_path)
            .first(1)
            .map(&:logical_path)
  end

  def underscore_constant(name)
    return "rubocop" if name == "RuboCop"

    name.gsub(/([A-Z\d]+)([A-Z][a-z])/, '\\1_\\2')
        .gsub(/([a-z\d])([A-Z])/, '\\1_\\2')
        .tr("-", "_")
        .downcase
  end

  def add_edge(source, target, kind)
    return if source == target

    @dependencies.fetch(source) << target
    edge_kind = bootstrap_edge?(source, target, kind) ? "#{kind}:package_bootstrap" : kind
    @edge_kinds[[source, target]] << edge_kind
  end

  def bootstrap_edge?(source, target, kind)
    return false unless %w[require require_relative autoload require_relative_relative autoload_relative].include?(kind)
    return false unless @package_entrypoints.include?(target)

    @entry_by_logical_path.fetch(source).spec.name == @entry_by_logical_path.fetch(target).spec.name
  end

  def build_ordering
    paths = @entries.map(&:logical_path)
    ordering_dependencies = @dependencies.to_h do |source, targets|
      ordered_targets = targets.reject do |target|
        kinds = @edge_kinds.fetch([source, target])
        kinds.all? { |kind| kind.end_with?(":package_bootstrap") }
      end
      [source, ordered_targets]
    end
    @components = StronglyConnectedComponents.new(paths, ordering_dependencies).call
    component_by_path = {}
    @components.each_with_index do |component, index|
      component.each { |path| component_by_path[path] = index }
    end

    component_dependencies = Hash.new { |hash, key| hash[key] = Set.new }
    ordering_dependencies.each do |source, targets|
      source_component = component_by_path.fetch(source)
      targets.each do |target|
        target_component = component_by_path.fetch(target)
        component_dependencies[source_component] << target_component unless source_component == target_component
      end
    end

    component_ranks = {}
    calculate_rank = lambda do |component|
      component_ranks[component] ||= begin
        dependencies = component_dependencies[component]
        dependencies.empty? ? 0 : 1 + dependencies.map { |dependency| calculate_rank.call(dependency) }.max
      end
    end
    @components.each_index { |component| calculate_rank.call(component) }

    @ranks = paths.to_h do |path|
      [path, component_ranks.fetch(component_by_path.fetch(path))]
    end
  end
end

options = {
  check: false,
  output_dir: DEFAULT_OUTPUT_DIR,
  annotations: DEFAULT_ANNOTATIONS,
  rust_equivalents: DEFAULT_RUST_EQUIVALENTS
}
OptionParser.new do |parser|
  parser.banner = "Usage: bundle exec ruby script/generate_rubocop_dependency_inventory.rb [--check]"
  parser.on("--check", "Fail if committed inventory differs from a fresh generation") do
    options[:check] = true
  end
  parser.on("--output-dir PATH", "Write artifacts to PATH") do |path|
    options[:output_dir] = Pathname(path).expand_path
  end
  parser.on("--annotations PATH", "Read reviewed annotations from PATH") do |path|
    options[:annotations] = Pathname(path).expand_path
  end
  parser.on("--rust-equivalents PATH", "Read reviewed Rust equivalence judgments from PATH") do |path|
    options[:rust_equivalents] = Pathname(path).expand_path
  end
end.parse!

inventory = DependencyInventory.new(
  annotation_path: options.fetch(:annotations),
  rust_equivalence_path: options.fetch(:rust_equivalents)
)
inventory.validate!
csv = inventory.to_csv
graph = inventory.to_graph_json
output_dir = options.fetch(:output_dir)
csv_path = output_dir.join("rubocop_dependency_inventory.csv")
graph_path = output_dir.join("rubocop_dependency_graph.json")

if options.fetch(:check)
  mismatches = []
  mismatches << csv_path.to_s unless csv_path.file? && csv_path.binread == csv.b
  mismatches << graph_path.to_s unless graph_path.file? && graph_path.binread == graph.b
  mismatches.each do |path|
    file = Pathname(path)
    expected = file == csv_path ? csv : graph
    actual = file.file? ? file.binread : ""
    warn "#{file.basename}: committed=#{Digest::SHA256.hexdigest(actual)} generated=#{Digest::SHA256.hexdigest(expected)}"
  end
  abort "dependency inventory is stale: #{mismatches.join(', ')}" unless mismatches.empty?

  puts "dependency inventory is current (#{inventory.entries.length} files)"
else
  output_dir.mkpath
  csv_path.binwrite(csv)
  graph_path.binwrite(graph)
  puts "wrote #{csv_path.relative_path_from(ROOT)}"
  puts "wrote #{graph_path.relative_path_from(ROOT)}"
  puts "inventoried #{inventory.entries.length} files across #{inventory.packages.length} packages"
end
