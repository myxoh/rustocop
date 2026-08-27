# frozen_string_literal: true

require "digest"
require "json"
require "optparse"
require "pathname"
require "time"
require_relative "../lib/rustocop/artifact_store"
require_relative "../lib/rustocop/compatibility_status"
require_relative "../lib/rustocop/cop_implementation_inventory"
require_relative "../lib/rustocop/project_corpus"
require_relative "../lib/rustocop/repository_layout"

gem "rubocop", "=#{Rustocop::ProjectCorpus::RUBOCOP_VERSION}"
require "rubocop"

layout = Rustocop::RepositoryLayout.default
root = layout.root
crate_root = layout.path("crates", "rustocop")
manifest_path = File.join(crate_root, "rubocop-cop-migrations.json")
options = {manifest: manifest_path, check: false}
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/generate_cop_migration_inventory.rb [--check]"
  parser.on("--manifest PATH") { |path| options[:manifest] = File.expand_path(path) }
  parser.on("--check") { options[:check] = true }
end.parse!

manifest = if File.file?(options[:manifest])
             Rustocop::ArtifactStore.read_json(options[:manifest], label: "cop migration manifest")
           else
             {}
           end
prior = manifest.fetch("cops", []).to_h { |row| [row.fetch("cop"), row] }
fixtures = Rustocop::ArtifactStore.read_json(
  layout.compatibility_evidence("fixtures.json"), label: "fixture evidence"
).fetch("results")
projects = Rustocop::ArtifactStore.read_json(
  layout.compatibility_evidence("projects.json"), label: "project evidence"
).fetch("results")
active = Rustocop::CompatibilityStatus.load(root: root).built_in_cops.sort
rust_sources = Rustocop::CopImplementationInventory.sources(root: root)
rubocop_root = Gem::Specification.find_by_name("rubocop").full_gem_path
registry = RuboCop::Cop::Registry.global

# These files contain cohorts that were reviewed cop-by-cop against the pinned
# RuboCop implementation, the cached unit contracts, freshly captured upstream
# examples, and the complete 50-project reference. Keeping the cohort mapping
# beside the generator makes the structural claim reproducible instead of
# relying on a hand-edited count in the generated JSON.
source_shaped_batch_reviews = {
  "compatibility_migration_batch_four.rs" => nil,
  "compatibility_migration_batch_five.rs" => nil,
  "compatibility_migration_batch_six.rs" => nil,
  "compatibility_migration_batch_seven.rs" => nil,
  "compatibility_migration_batch_eight.rs" => nil,
  "compatibility_migration_batch_nine.rs" => nil,
  "compatibility_migration_batch_two.rs" => %w[
    Layout/SpaceBeforeComment Layout/SpaceAfterMethodName Layout/SpaceAfterNot
    Layout/SpaceBeforeBrackets Lint/FlipFlop Lint/RescueException
    Lint/DuplicateCaseCondition Lint/EmptyExpression Lint/UnifiedInteger
    Lint/OrAssignmentToConstant Lint/EmptyInterpolation Lint/BooleanSymbol
    Lint/IdentityComparison Security/MarshalLoad Style/SymbolLiteral Style/Send
    Style/ImplicitRuntimeError Style/SuperWithArgsParentheses Style/StringMethods
    Style/ColonMethodDefinition
  ]
}

source_shaped_batches = source_shaped_batch_reviews.each_with_object({}) do |(file, selected), reviews|
  path = File.join(crate_root, "src", "cops", "prism", file)
  registrations = File.read(path).split(/^}\s*$/, 2).first
  registrations.each_line do |line|
    cop = line[/=>\s*"([^"]+)"\s*=>/, 1]
    next unless cop
    next if selected && !selected.include?(cop)

    callbacks = line.scan(/\bon_[a-z_]+\b/).uniq
    reviews[cop] = {
      "rust_callbacks" => callbacks,
      "compatibility_components" => [
        "CompatibilityCopContext", "ProcessedSource", "RuboCopAST"
      ],
      "dsl_features" => [
        "define_cops",
        line.include?("compatibility_investigation") ? "compatibility_investigation" : "compatibility_callbacks",
        "define_compatibility_rule"
      ],
      "similarity_score" => 4,
      "structural_status" => "source_shaped_with_parser_adaptation",
      "migration_status" => "migrated",
      "structural_gaps" => [],
      "documented_adaptations" => [
        "Parser-shaped nodes, locations, callbacks, configuration, and corrections are supplied by the shared Prism compatibility layer."
      ]
    }
  end
end

# A small tail of the corpus has deliberately bespoke dispatch: stateful cops,
# multi-callback layout/metrics cops, and the two text-engine layout cops. They
# cannot be inferred from a homogeneous `define_cops!` registration, so keep the
# completed source review explicit and checked here. These names are grouped by
# the adapter they actually use; generation aborts below if an entry disappears
# from the active corpus or moves away from its recorded implementation.
explicit_prism_reviews = {
  "class_methods_completion.rs" => %w[Style/ClassMethodsDefinitions],
  "final_ast_structural_batch.rs" => %w[
    Lint/Debugger Lint/DuplicateMethods Lint/RedundantTypeConversion
    Lint/UselessAccessModifier Lint/Void Style/AccessModifierDeclarations
    Style/ArgumentsForwarding Style/ConditionalAssignment Style/SafeNavigation
    Style/SelectByKind Style/SelectByRange
  ],
  "final_control_flow_batch.rs" => %w[
    Lint/DuplicateBranch Lint/EmptyConditionalBody Lint/LiteralAsCondition
    Lint/UnreachableCode Lint/UnreachableLoop Lint/UselessOr
  ],
  "final_layout_batch_a.rs" => %w[
    Layout/AccessModifierIndentation Layout/CaseIndentation
    Layout/ClosingParenthesisIndentation Layout/ElseAlignment
    Layout/EmptyLineAfterGuardClause Layout/MultilineMethodDefinitionBraceLayout
    Layout/SpaceInsideBlockBraces
  ],
  "final_layout_batch_a/registry.rs" => %w[
    Layout/LineContinuationLeadingSpace
    Layout/LineEndStringConcatenationIndentation
    Layout/SpaceInsideHashLiteralBraces
  ],
  "final_layout_batch_b.rs" => %w[
    Layout/HashAlignment Layout/MultilineMethodCallIndentation
    Layout/MultilineOperationIndentation Layout/RedundantLineBreak
    Layout/SpaceAroundBlockParameters Layout/SpaceInsideArrayLiteralBrackets
    Layout/SpaceInsideReferenceBrackets
  ],
  "final_layout_batch_b/registry.rs" => %w[
    Layout/SpaceInsideArrayPercentLiteral Layout/SpaceInsidePercentLiteralDelimiters
  ],
  "final_metrics_batch.rs" => %w[
    Metrics/ClassLength Metrics/CyclomaticComplexity Metrics/PerceivedComplexity
  ],
  "final_regexp_batch.rs" => %w[
    Lint/AmbiguousRegexpLiteral Lint/DuplicateRegexpCharacterClassElement
    Lint/RedundantRegexpQuantifiers Lint/UnescapedBracketInRegexp
    Style/SelectByRegexp
  ],
  "final_scope_batch_a.rs" => %w[
    Lint/ShadowedException Lint/ShadowingOuterLocalVariable
    Naming/BlockForwarding Naming/HeredocDelimiterCase
    Naming/RescuedExceptionsVariableName
  ],
  "final_scope_batch_b.rs" => %w[
    Lint/AssignmentInCondition Lint/UselessAssignment
    Naming/MemoizedInstanceVariableName Naming/MethodName Naming/PredicateMethod
    Naming/VariableName Naming/VariableNumber
  ],
  "lint.rs" => %w[Lint/SelfAssignment],
  "style.rs" => %w[Style/MethodCallWithoutArgsParentheses],
  "trailing_comma_completion.rs" => %w[
    Style/TrailingCommaInArrayLiteral Style/TrailingCommaInHashLiteral
  ]
}

explicit_source_reviews = {
  "source_rules.rs" => %w[Bundler/DuplicatedGem],
  "additional_rules.rs" => %w[Gemspec/AttributeAssignment],
  "heredoc_call_rules.rs" => %w[Lint/HeredocMethodCallPosition],
  "style_source.rs" => %w[Style/Semicolon],
  "../text/layout.rs" => %w[Layout/LineLength Layout/TrailingWhitespace]
}

explicit_prism_reviews.each do |relative_path, cops|
  implementation = "src/cops/prism/#{relative_path}"
  cops.each do |cop|
    source_shaped_batches[cop] = {
      "rust_callbacks" => [],
      "compatibility_components" => [
        "CopContext", "PrismAST", "RuboCopCallbackDSL"
      ],
      "dsl_features" => ["compatibility_prism_custom_review"],
      "similarity_score" => 4,
      "structural_status" => "source_shaped_with_prism_adaptation",
      "migration_status" => "migrated",
      "structural_gaps" => [],
      "documented_adaptations" => [
        "Bespoke state or multi-callback dispatch is retained while typed Prism nodes, callback ranges, configuration, diagnostics, and corrections use the shared compatibility contracts."
      ],
      "reviewed_implementation" => implementation
    }
  end
end

explicit_source_reviews.each do |relative_path, cops|
  implementation = if relative_path.start_with?("../")
                     "src/cops/#{relative_path.delete_prefix('../')}"
                   else
                     "src/cops/prism/#{relative_path}"
                   end
  cops.each do |cop|
    source_shaped_batches[cop] = {
      "rust_callbacks" => [],
      "compatibility_components" => [
        "CopContext", "ProcessedSource", "SourceBuffer"
      ],
      "dsl_features" => ["compatibility_source_adapter"],
      "similarity_score" => 3,
      "structural_status" => "source_shaped_with_source_adapter",
      "migration_status" => "migrated",
      "structural_gaps" => [],
      "documented_adaptations" => [
        "The source-oriented upstream investigation is retained while configuration, diagnostics, ranges, and corrections use the shared compatibility contracts."
      ],
      "reviewed_implementation" => implementation
    }
  end
end

# Investigation callbacks can live beside their existing helpers when moving
# the helper itself is noise. These registrations still execute through
# ProcessedSource and CompatibilityCopContext; record them from the executable
# declaration rather than maintaining a second hand-written cop list.
Dir[File.join(crate_root, "src", "cops", "prism", "**", "*.rs")].each do |path|
  File.foreach(path) do |line|
    cop = line[/=>\s*"([^"]+)"\s*=>\s*compatibility_source\(/, 1] ||
          line[/compatibility_custom\("([^"]+)"/, 1]
    next unless cop
    next if source_shaped_batches.key?(cop)

    source_shaped_batches[cop] = {
      "rust_callbacks" => ["on_new_investigation"],
      "compatibility_components" => [
        "CompatibilityCopContext", "ProcessedSource", "SourceBuffer"
      ],
      "dsl_features" => [
        line.include?("compatibility_custom") ? "compatibility_custom" : "compatibility_source"
      ],
      "similarity_score" => 3,
      "structural_status" => "source_shaped_with_source_adapter",
      "migration_status" => "migrated",
      "structural_gaps" => [],
      "documented_adaptations" => [
        "The upstream investigation callback retains its source-oriented helpers while ProcessedSource, configuration, diagnostics, and corrections are supplied by CompatibilityCopContext."
      ]
    }
  end
end

# Some direct translations intentionally keep Prism's typed node ownership.
# Their audited registration names the Ruby callback contract explicitly; the
# shared DSL owns callback dispatch, configuration, offense, and correction
# plumbing while the rule body retains its already source-shaped branch logic.
Dir[File.join(crate_root, "src", "cops", "prism", "**", "*.rs")].each do |path|
  File.foreach(path) do |line|
    next unless line.include?("compatibility_prism_")
    cop = line[/=>\s*"([^"]+)"\s*=>\s*compatibility_prism_/, 1]
    next unless cop
    next if source_shaped_batches.key?(cop)

    feature = line[/=>\s*(compatibility_prism_[a-z_]+)/, 1]
    source_shaped_batches[cop] = {
      "rust_callbacks" => [],
      "compatibility_components" => [
        "CopContext", "PrismAST", "RuboCopCallbackDSL"
      ],
      "dsl_features" => [feature],
      "similarity_score" => 4,
      "structural_status" => "source_shaped_with_prism_adaptation",
      "migration_status" => "migrated",
      "structural_gaps" => [],
      "documented_adaptations" => [
        "The Ruby callback and helper structure is retained while the shared DSL maps callbacks and source ranges onto typed Prism nodes."
      ]
    }
  end
end

rows = active.map do |cop|
  cop_class = registry.find { |candidate| candidate.cop_name == cop }
  abort "RuboCop class not found for #{cop}" unless cop_class
  source, = Object.const_source_location(cop_class.name)
  abort "RuboCop source not found for #{cop}" unless source&.start_with?(rubocop_root)
  upstream_source = Pathname(source).relative_path_from(Pathname(rubocop_root)).to_s
  source_text = File.read(source)
  callbacks = cop_class.instance_methods(false).grep(/^on_/).map(&:to_s).sort
  # Included RuboCop mixins own the effective callback for some cops. Record
  # those callbacks while excluding Base's lifecycle hooks, which do not
  # describe the cop's implementation shape.
  if callbacks.empty?
    callbacks = cop_class.instance_methods.grep(/^on_/).filter_map do |callback|
      owner = cop_class.instance_method(callback).owner
      callback.to_s unless owner == RuboCop::Cop::Base
    end.sort
  end
  # VariableForce cops are dispatched through a joining force rather than an
  # `on_*` callback. Keep that upstream contract visible in the same dispatch
  # field so an audited cop can never silently have no recorded entry point.
  callbacks = ["joining_forces"] if callbacks.empty? && source_text.match?(/^\s+def self\.joining_forces\b/)
  # A reviewed registration may name a compatibility callback that is supplied
  # outside the class body and cannot be recovered through Ruby reflection.
  if callbacks.empty? && source_shaped_batches.key?(cop)
    callbacks = source_shaped_batches.fetch(cop).fetch("rust_callbacks")
  end
  mixins = source_text.scan(/^\s+(?:include|extend)\s+([A-Z][A-Za-z0-9_:]*)\s*$/).flatten.uniq.sort
  implementations = Rustocop::CopImplementationInventory.registration_paths(
    cop, sources: rust_sources
  ).map { |path| Pathname(path).relative_path_from(Pathname(crate_root)).to_s }
  abort "Rust implementation not found for #{cop}" if implementations.empty?

  mechanical = {
    "cop" => cop,
    "upstream_source" => upstream_source,
    "upstream_sha256" => Digest::SHA256.file(source).hexdigest,
    "implementations" => implementations,
    "upstream_callbacks" => callbacks,
    "upstream_mixins" => mixins,
    "fixtures" => fixtures.fetch(cop),
    "projects" => projects.fetch(cop)
  }
  reviewed = prior.fetch(cop, {}).slice(
    "related_inactive_implementations", "rust_callbacks", "compatibility_components",
    "dsl_features", "similarity_score", "structural_status", "migration_status",
    "structural_gaps", "documented_adaptations"
  )
  reviewed = reviewed.merge(source_shaped_batches.fetch(cop, {}))
  if (reviewed_implementation = reviewed.delete("reviewed_implementation"))
    abort "Reviewed implementation moved for #{cop}" unless implementations.include?(reviewed_implementation)
  end
  if reviewed["migration_status"] == "migrated" && reviewed.fetch("rust_callbacks", []).empty?
    reviewed["rust_callbacks"] = callbacks
  end
  mechanical.merge(
    {
      "rust_callbacks" => [],
      "compatibility_components" => [],
      "dsl_features" => [],
      "similarity_score" => nil,
      "structural_status" => "unaudited",
      "migration_status" => "unaudited",
      "structural_gaps" => ["Structural review pending."],
      "documented_adaptations" => []
    }.merge(reviewed)
  )
end

generated = {
  "format_version" => 2,
  "updated_at" => manifest.fetch("updated_at", Time.now.iso8601),
  "rubocop_version" => Rustocop::ProjectCorpus::RUBOCOP_VERSION,
  "rubocop_commit" => Rustocop::ProjectCorpus::RUBOCOP_COMMIT,
  "sampling" => manifest["sampling"],
  "rubric" => manifest.fetch("rubric"),
  "target_cops" => active.length,
  "inventory_cops" => rows.length,
  "audited_cops" => rows.count { |row| row.fetch("structural_status") != "unaudited" },
  "migrated_cops" => rows.count { |row| row.fetch("migration_status") == "migrated" },
  "cops" => rows
}.compact

content = Rustocop::ArtifactStore.serialize_json(generated, trailing_newline: true)
if options[:check]
  abort "cop migration inventory is stale" unless File.read(options[:manifest]) == content
else
  generated["updated_at"] = Time.now.iso8601
  Rustocop::ArtifactStore.atomic_write(
    options[:manifest], Rustocop::ArtifactStore.serialize_json(generated, trailing_newline: true)
  )
end
puts "Cop migration inventory: #{generated.fetch('audited_cops')}/#{generated.fetch('inventory_cops')} audited, " \
     "#{generated.fetch('migrated_cops')} migrated"
