# frozen_string_literal: true

require "digest"
require "json"
require "pathname"
require "ripper"
require "rubygems"
require "set"
require "time"

PACKAGES = {
  "rubocop" => "1.87.0",
  "rubocop-ast" => "1.49.1"
}.freeze

# Activate the audited versions before either gem is required. Letting RubyGems
# choose the newest installed version made runtime API discovery depend on the
# developer machine even though source checksums were pinned below.
gem "rubocop-ast", "=#{PACKAGES.fetch('rubocop-ast')}"
gem "rubocop", "=#{PACKAGES.fetch('rubocop')}"
require "rubocop"
require "rubocop-ast"

ROOT = Pathname(__dir__).join("..").expand_path
MANIFEST = ROOT.join("crates/rustocop/rubocop-translation.json")
REPORT = ROOT.join("docs/rubocop-compatibility-progress.md")
EXAMPLE_INVENTORY = ROOT.join("spec/upstream/rubocop-compatibility-examples.json")

def generated_api_equivalences(source, api)
  case source
  when "lib/rubocop/cop/commissioner.rb"
    api.grep(/\Aon_/).to_h { |name| [name, "on_callback"] }
  when "lib/rubocop/ast/node.rb"
    api.grep(/_type\?\z/).to_h { |name| [name, "type_is"] }
  when "lib/rubocop/cop/variable_force/branch.rb"
    predicates = %w[
      collection? element? else_body? ensure_body? falsey_body? in_pattern?
      left_body? loop_body? main_body? rescue_clause? right_body? target?
      truthy_body? when_clause?
    ]
    api.intersection(predicates).to_h { |name| [name, "predicate"] }
  when "lib/rubocop/cop/variable_force.rb"
    api.grep(/\A(?:before|after)_(?:declaring_variable|entering_scope|leaving_scope)\z/)
      .to_h { |name| [name, "notify_handlers"] }
  when "lib/rubocop/cop/exclude_limit.rb"
    api.grep(/=\z/).to_h { |name| [name, "record"] }
  when "lib/rubocop/ast/node/mixin/collection_node.rb"
    # CollectionNode is exactly a SimpleForwardable facade over `to_a`; the
    # Rust node exposes the same boundary as a Vec<NodeRef>.
    api.to_h { |name| [name, "to_a"] }
  when "lib/rubocop/ast/traversal.rb"
    api.to_h do |name|
      target = case name
               when "body" then "walk_children"
               when "children_count_check_code" then "validate_shape"
               when "def_callback" then "dispatch"
               when "on_" then "on_unknown"
               when "on___ENCODING__" then "on_encoding"
               when "on___FILE__" then "on_file"
               when "on___LINE__" then "on_line"
               when "walk" then "walk"
               else rust_api_name(name)
               end
      [name, target]
    end
  when %r{\Alib/rubocop/ast/node_pattern/(?:builder|compiler(?:/[^/]+)?|method_definer|node)\.rb\z}
    # The Ruby compiler hierarchy emits Ruby source. Rust deliberately removes
    # that intermediate language: parsing produces typed Expr values and
    # NodePattern::new constructs the executable interpreter in one boundary.
    api.to_h { |name| [name, "new"] }
  when %r{\Alib/rubocop/ast/node_pattern/(?:comment|lexer(?:\.rex)?|with_meta)\.rb\z}
    api.to_h { |name| [name, "lex"] }
  when %r{\Alib/rubocop/ast/node_pattern/parser(?:\.racc)?\.rb\z}
    api.to_h { |name| [name, "parse_expression"] }
  when "lib/rubocop/ast/node_pattern/sets.rb"
    api.to_h { |name| [name, "stable_set_name"] }
  when "lib/rubocop/ast/rubocop_compatibility.rb"
    api.to_h { |name| [name, "compatibility_warning"] }
  else
    {}
  end
end

GENERATED_FOLD_TARGETS = Set.new([
  ["lib/rubocop/cop/commissioner.rb", "on_callback"],
  ["lib/rubocop/ast/node.rb", "type_is"],
  ["lib/rubocop/cop/variable_force/branch.rb", "predicate"],
  ["lib/rubocop/cop/variable_force.rb", "notify_handlers"],
  ["lib/rubocop/cop/exclude_limit.rb", "record"],
  ["lib/rubocop/ast/node/mixin/collection_node.rb", "to_a"],
  ["lib/rubocop/ast/traversal.rb", "walk_children"],
  ["lib/rubocop/ast/node_pattern/builder.rb", "new"],
  ["lib/rubocop/ast/node_pattern/comment.rb", "lex"],
  ["lib/rubocop/ast/node_pattern/compiler.rb", "new"],
  ["lib/rubocop/ast/node_pattern/compiler/atom_subcompiler.rb", "new"],
  ["lib/rubocop/ast/node_pattern/compiler/binding.rb", "new"],
  ["lib/rubocop/ast/node_pattern/compiler/debug.rb", "new"],
  ["lib/rubocop/ast/node_pattern/compiler/node_pattern_subcompiler.rb", "new"],
  ["lib/rubocop/ast/node_pattern/compiler/sequence_subcompiler.rb", "new"],
  ["lib/rubocop/ast/node_pattern/compiler/subcompiler.rb", "new"],
  ["lib/rubocop/ast/node_pattern/lexer.rb", "lex"],
  ["lib/rubocop/ast/node_pattern/lexer.rex.rb", "lex"],
  ["lib/rubocop/ast/node_pattern/method_definer.rb", "new"],
  ["lib/rubocop/ast/node_pattern/node.rb", "new"],
  ["lib/rubocop/ast/node_pattern/parser.racc.rb", "parse_expression"],
  ["lib/rubocop/ast/node_pattern/parser.rb", "parse_expression"],
  ["lib/rubocop/ast/node_pattern/sets.rb", "stable_set_name"],
  ["lib/rubocop/ast/node_pattern/with_meta.rb", "lex"]
]).freeze

expanded_example_inventory = JSON.parse(EXAMPLE_INVENTORY.read)
unless expanded_example_inventory.fetch("versions") == PACKAGES
  abort "expanded RSpec inventory versions do not match the pinned packages"
end
EXPANDED_EXAMPLES = expanded_example_inventory.fetch("examples").group_by do |example|
  [example.fetch("package"), example.fetch("source")]
end.freeze
GROUP_MAPPINGS = %w[
  allowed_identifiers
  allowed_methods
  allowed_pattern
  forbidden_identifiers
  forbidden_pattern
  min_body_length
  min_branches_count
  preferred_delimiters
  target_ruby_version
].to_h do |name|
  [
    "lib/rubocop/cop/mixin/#{name}.rb",
    {
      "status" => "translated",
      "rust" => "src/rubocop/cop/mixin/policies.rs",
      "evidence" => "Direct branch-preserving translation with focused Rust contracts for every configuration path."
    }
  ]
end.merge(
  %w[
    condition_corrector
    empty_line_corrector
    punctuation_corrector
    require_library_corrector
    string_literal_corrector
    unused_arg_corrector
  ].to_h do |name|
    [
      "lib/rubocop/cop/correctors/#{name}.rb",
      {
        "status" => "translated",
        "rust" => "src/rubocop/cop/correctors.rs",
        "evidence" => "Direct correction branch translation over source ranges with focused rewrite contracts."
      }
    ]
  end
).merge(
  %w[
    array_min_size
    configurable_enforced_style
    configurable_max
    configurable_naming
    configurable_numbering
    symbol_help
  ].to_h do |name|
    [
      "lib/rubocop/cop/mixin/#{name}.rb",
      {
        "status" => "translated",
        "rust" => "src/rubocop/cop/mixin/configuration.rs",
        "evidence" => "Direct state-machine, configuration, predicate, and auto-generation translation with complete branch contracts."
      }
    ]
  end
).merge(
  %w[
    allowed_receivers
    array_syntax
    auto_corrector
    duplication
    gem_declaration
    integer_node
    method_preference
    parentheses
    percent_literal
    rational_literal
    safe_assignment
    string_literals_help
    trailing_body
    multiline_element_line_breaks
    match_range
    nil_methods
    empty_parameter
    def_node
    dig_help
    negative_conditional
    on_normal_if_unless
  ].to_h do |name|
    [
      "lib/rubocop/cop/mixin/#{name}.rb",
      {
        "status" => "translated",
        "rust" => "src/rubocop/cop/mixin/helpers.rs",
        "evidence" => "Direct branch and predicate translation with focused contracts preserving RuboCop inputs and outcomes."
      }
    ]
  end
).merge(
  "lib/rubocop/cop/autocorrect_logic.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/framework.rs",
    "evidence" => "Direct translation of requested/correctable/enabled policy, safe and contextual modes, disable-uncorrectable behavior, multiline literal detection, and inline-versus-surrounding disable corrections with focused branch contracts."
  },
  "lib/rubocop/cop/util.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/framework.rs",
    "evidence" => "Direct translation of traversal, source-line, call-chain, argument, parentheses, escaping, string-literal, style-name, and regexp helpers; all upstream Util spec examples are ported.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/util_spec.rb",
        "source_sha256" => "8d1945f3637b5080695968de28d02efd1ec31f871718930b85cda12946444b15",
        "rust" => "src/rubocop/cop/util_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/commissioner.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/commissioner.rs",
    "evidence" => "Source-shaped facade for callback construction and mutation, restricted send dispatch, invocation, resets, begin/investigate lifecycle, error capture/re-raise policy, and mergeable investigation-report accessors, backed by the tested Rust commissioner runtime.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/commissioner_spec.rb",
        "source_sha256" => "6cb506d0d78fbbcdb0621850cc986eed14f08a33bc7dd1e75661f75bfc7dec94",
        "rust" => "src/rubocop/cop/commissioner_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/variable_force.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/variable_force/mod.rs",
    "evidence" => "Direct state-machine translation covering declaration/assignment/reference ordering, operator and multiple assignment, block capture, isolated method/class scopes, branch-aware assignment lifetime/exclusivity, post-condition traversal, and regexp traversal; all pinned smoke contracts plus focused lifetime matrices are ported.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/variable_force_spec.rb",
        "source_sha256" => "d716261b8b3bb32d910dcf82b1e6c74fe7139e619e718788e8ee270d446dc582",
        "rust" => "src/rubocop/cop/variable_force_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/mixin/annotation_comment.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/annotation_comment.rs",
    "evidence" => "Direct longest-keyword parsing, annotation classification, correction validation, and character-bound translation.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/annotation_comment_spec.rb",
        "source_sha256" => "f3a274ea61e8fe18463a6d50784cf33c1bac201a3dd04d881d29e39e7e685ad0",
        "rust" => "src/rubocop/cop/annotation_comment_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/documentation.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/documentation.rs",
    "evidence" => "Direct URL construction and configuration-fallback translation with focused contracts for builtin, custom, nested-department, and extension branches."
  },
  "lib/rubocop/cop/exclude_limit.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/exclude_limit.rs",
    "evidence" => "Direct parameter-name transformation, append-only recording, cop directory, and maximum aggregation translation with filesystem contracts."
  },
  "lib/rubocop/cop/ignored_node.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/ignored_node.rs",
    "evidence" => "Direct identity, expression containment, and heredoc containment translation with complete branch contracts."
  },
  "lib/rubocop/cop/force.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/force.rs",
    "evidence" => "Direct force naming, cop membership, responding-hook dispatch, skip, and joining-cop error translation.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/force_spec.rb",
        "source_sha256" => "86d91716e9bb5f5149e6fed45ce7f86bcb6a6d419a0c8d103eea14fc50fd1d90",
        "rust" => "src/rubocop/cop/force_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/legacy/corrections_proxy.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/legacy.rs",
    "evidence" => "Direct proxy translation covering callable application, emptiness, corrector merging, and clobber-suppressing transactions."
  },
  "lib/rubocop/cop/legacy/corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/legacy.rs",
    "evidence" => "Direct legacy-constructor and corrections-proxy adapter over the translated current Corrector."
  },
  "lib/rubocop/cop/message_annotator.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/message_annotator.rs",
    "evidence" => "Direct annotation, option precedence, department URL, details, and reference-list translation.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/message_annotator_spec.rb",
        "source_sha256" => "74047ed0fd2950bdef9dbc2f18c5d359d588dbcb3668ca81ac10758777ccc500",
        "rust" => "src/rubocop/cop/message_annotator_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/offense.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/offense.rs",
    "evidence" => "Direct value-object translation, including status predicates, source highlighting, formatting, equality, hashing, and ordering.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/offense_spec.rb",
        "source_sha256" => "7b71096ee4ce80463e77b7b82e44db00ca2c9bbbec1e6c53242b0bf386deea23",
        "rust" => "src/rubocop/cop/offense_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/registry.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/registry.rs",
    "evidence" => "Direct ordered enrollment, qualification, ambiguity, warning, lazy registration, filtering, pending/safe enablement, lookup, and sorting translation.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/registry_spec.rb",
        "source_sha256" => "f4bf5ab3323cfd211489ca4200831412ce13f69acc89e01747a89a287817a427",
        "rust" => "src/rubocop/cop/registry_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/ast/node/mixin/descendence.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/node/core.rs",
    "evidence" => "Direct child, descendant, self-inclusive, type-filtered, and depth-first traversal translation over the compatibility arena."
  },
  "lib/rubocop/ast/node/mixin/method_dispatch_node.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/node/mixin/method_dispatch_node.rs",
    "evidence" => "Direct control-flow and node-pattern translation with relevant upstream SendNode examples ported to Rust.",
    "specs" => [
      {
        "package" => "rubocop-ast",
        "source" => "spec/rubocop/ast/send_node_spec.rb",
        "source_sha256" => "a9d9b8f8c2d4e94f2f9b8297abac812f73ebd9a4c6874f81488f9fab64bf083d",
        "rust" => "src/rubocop/ast/node/mixin/method_dispatch_node_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/ast/node/mixin/method_identifier_predicates.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/node/mixin/method_identifier_predicates.rs",
    "evidence" => "Direct predicate/set translation with the relevant upstream SendNode examples ported to Rust.",
    "specs" => [
      {
        "package" => "rubocop-ast",
        "source" => "spec/rubocop/ast/send_node_spec.rb",
        "source_sha256" => "a9d9b8f8c2d4e94f2f9b8297abac812f73ebd9a4c6874f81488f9fab64bf083d",
        "rust" => "src/rubocop/ast/node/mixin/method_identifier_predicates_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  }
).freeze
BUILDER_MAPPINGS = %w[
  lib/rubocop/ast/builder.rb
  lib/rubocop/ast/sexp.rb
].to_h do |source|
  [
    source,
    {
      "status" => "translated",
      "rust" => "src/rubocop/ast/builder.rs",
      "evidence" => "Direct complete node-class dispatch, fallback, byte-preserving string, and S-expression construction translation."
    }
  ]
end.freeze
PARTIAL_MAPPINGS = {
  "lib/rubocop/cop/badge.rb" => {
    "status" => "translated", "rust" => "src/rubocop/cop/badge.rs",
    "evidence" => "Direct Badge parsing, qualification, comparison, hashing, department, naming, and class-name translation."
  },
  "lib/rubocop/cop/corrector.rb" => {
    "status" => "translated", "rust" => "src/rubocop/cop/corrector.rs",
    "evidence" => "Direct source-range correction transaction translation with insert, remove, replace, swap, merge, and rewrite behavior."
  },
  "lib/rubocop/cop/mixin/range_help.rb" => {
    "status" => "translated", "rust" => "src/rubocop/cop/mixin/range_help.rs",
    "evidence" => "Direct source-range, whole-line, comments, comma, whitespace, BOM, and argument-range helper translation."
  },
  "lib/rubocop/cop/severity.rb" => {
    "status" => "translated", "rust" => "src/rubocop/cop/severity.rs",
    "evidence" => "Direct Severity construction, validation, ordering, naming, code, and display translation."
  },
  "lib/rubocop/ast/traversal.rb" => {
    "status" => "native",
    "rust" => "src/rubocop/ast/traversal.rs",
    "evidence" => "Rust trait callbacks and macro-generated dispatch replace Ruby's runtime callback-code generator. All 137 Parser node types retain depth-first callback semantics, forward-compatible unknown-node recursion, debug child-arity validation, and the complete upstream traversal corpus.",
    "specs" => [
      {
        "package" => "rubocop-ast",
        "source" => "spec/rubocop/ast/traversal_spec.rb",
        "source_sha256" => "83cfa5999a351257de34b5d8f8be06780d03900760cc3d2418b7fa44278f9424",
        "rust" => "src/rubocop/ast/traversal_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/ast/node.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/node/core.rs",
    "evidence" => "Direct arena-backed translation of parent/completion/sibling/source/location/ancestor/descendant/type-group/literal/assignment/conditional/keyword/purity/value-use behavior, structural equality, constructors and class/module definition/name resolution, including unknown block and singleton-scope boundaries.",
    "specs" => [
      {
        "package" => "rubocop-ast",
        "source" => "spec/rubocop/ast/node_spec.rb",
        "source_sha256" => "8bc7277f33d3bc5d4b0e1f2821c3520b4700f5822ee60ed99a765e13613bde53",
        "rust" => "src/rubocop/ast/node/core_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/ast/processed_source.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/processed_source.rs",
    "evidence" => "Direct source ownership, parser-engine policy, Prism diagnostics/comments, identity-based AST comment association, line/index/range access, SHA-1, stable token navigation/predicates, __END__ handling, Unicode character offsets, file loading, and parser-shaped AST adaptation with the upstream behavior matrix consolidated into focused contracts.",
    "specs" => [
      {
        "package" => "rubocop-ast",
        "source" => "spec/rubocop/ast/processed_source_spec.rb",
        "source_sha256" => "2518178fb4a5fe278e79cb998f0bfa0cfb2a4a355a583a4d43dbb154e8dcf374",
        "rust" => "src/rubocop/ast/processed_source_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/ast/rubocop_compatibility.rb" => {
    "status" => "native",
    "rust" => "src/rubocop/ast/native.rs",
    "evidence" => "Rust startup has no Ruby require hook. The native compatibility check preserves the pinned-version thresholds and warning content, including singular/plural cop labels and the no-warning compatible path.",
    "specs" => [
      {
        "package" => "rubocop-ast",
        "source" => "spec/rubocop/ast/rubocop_compatibility_spec.rb",
        "source_sha256" => "223dbef6bc5d8d5c1b0cd9a897b5433bd363c82e84dd110632483a71a5c1b7b8",
        "rust" => "src/rubocop/ast/native.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/ast/token.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/token.rs",
    "evidence" => "Direct Token value, source-range, spacing, display, construction, and complete predicate translation.",
    "specs" => [
      {
        "package" => "rubocop-ast",
        "source" => "spec/rubocop/ast/token_spec.rb",
        "source_sha256" => "46c1f07eb32401449aafa28a2d3d72ccf5dcfe1035ebfea6a87e0f5fe01afa9d",
        "rust" => "src/rubocop/ast/token_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  }
}.freeze
NOT_APPLICABLE_MAPPINGS = {
  "lib/rubocop/cop/correctors.rb" =>
    "Ruby require aggregator; Rust modules are registered statically by cop/mod.rs.",
  "lib/rubocop/cop/internal_affairs.rb" =>
    "Ruby require aggregator for RuboCop's own development cops, which are outside the built-in cop runtime surface.",
  "lib/rubocop/cop/mixin.rb" =>
    "Ruby require aggregator; Rust mixins are registered statically by cop/mixin/mod.rs.",
  "lib/rubocop/ast/node_pattern/lexer.rex" =>
    "Rex generator input; executable lexer behavior is tracked by lexer.rex.rb.",
  "lib/rubocop/ast/node_pattern/parser.y" =>
    "Racc generator input; executable parser behavior is tracked by parser.racc.rb.",
  "lib/rubocop/ast/utilities/simple_forwardable.rb" =>
    "Ruby metaprogramming utility; Rust delegation is compile-time and has no runtime equivalent.",
  "lib/rubocop/ast/version.rb" =>
    "Ruby gem packaging constant; Cargo owns the Rust package version."
}.freeze
NATIVE_MAPPINGS = %w[
  lib/rubocop/ast/builder_prism.rb
  lib/rubocop/ast/node/break_node.rb
  lib/rubocop/ast/node/complex_node.rb
  lib/rubocop/ast/node/const_node.rb
  lib/rubocop/ast/node/float_node.rb
  lib/rubocop/ast/node/int_node.rb
  lib/rubocop/ast/node/next_node.rb
  lib/rubocop/ast/node/rational_node.rb
  lib/rubocop/ast/node/return_node.rb
  lib/rubocop/ast/node/symbol_node.rb
].to_h do |source|
  [
    source,
    {
      "status" => "native",
      "rust" => "src/rubocop/ast/native.rs",
      "evidence" => "The Ruby component only selects a node/builder class; ruby-prism exposes the corresponding typed node directly, verified by native parsing tests."
    }
  ]
end.merge(
  "lib/rubocop/ast/node/mixin/collection_node.rb" => {
    "status" => "native",
    "rust" => "src/rubocop/ast/node/specialized.rs",
    "evidence" => "Ruby delegates the complete Array facade to `to_a`; Rust exposes the same typed Vec<NodeRef> boundary and uses Rust's native collection operations."
  }
).merge(
  {
    "break_node" => ["break_node_spec.rb", "b8600988e8fb848e217a05dd2e8fa005f1eb65335958583fdbe3da831cc0a613"],
    "complex_node" => ["complex_node_spec.rb", "9c9098bdf0436aa7d181205c4db4db69e2013b7fd2d1fce9f7c49b400a981428"],
    "const_node" => ["const_node_spec.rb", "fc023b7363b03b85a79f1108f7287942c80adb3bd2587fa7be37d634e9bb1778"],
    "float_node" => ["float_node_spec.rb", "fb40020926e959e416f9eda31a0593c959c76a15130a098d16201e4b2920ece5"],
    "int_node" => ["int_node_spec.rb", "e52f8f360afb9091f922d1f2cdcf76e5a2ee2a9dd40d637d22a8a3c185ccacab"],
    "next_node" => ["next_node_spec.rb", "96c5d8ed39c4a306332a6971b6927660e08909461d8df4a03e475db8e6d88c01"],
    "rational_node" => ["rational_node_spec.rb", "4b1771c7312c2f0fcbc6e7071e7a347fa1dd31d70c04544d375bb02bf8bf3839"],
    "return_node" => ["return_node_spec.rb", "3bee619d8c57f7b82ebd54cb8db67a205c68cb4494d3c37c2e26c83b914d2877"],
    "symbol_node" => ["symbol_node_spec.rb", "8324001c80d65f496e4bcbee3984080b0f291558710b8260d45e82a266ccb4f3"]
  }.to_h do |name, (spec, sha256)|
    [
      "lib/rubocop/ast/node/#{name}.rb",
      {
        "status" => "native",
        "rust" => "src/rubocop/ast/native.rs",
        "evidence" => "Prism-native typed node adapted to the exact rubocop-ast scalar or wrapped-argument contract.",
        "specs" => [
          {
            "package" => "rubocop-ast",
            "source" => "spec/rubocop/ast/#{spec}",
            "source_sha256" => sha256,
            "rust" => "src/rubocop/ast/node/small_nodes_spec.rs",
            "status" => "translated",
            "deviations" => []
          }
        ]
      }
    ]
  end
).freeze
AST_SEMANTIC_MAPPINGS = %w[
  basic_literal_node
  binary_operator_node
  conditional_node
  constant_node
  modifier_node
  numeric_node
  parameterized_node
  predicate_operator_node
].to_h do |name|
  [
    "lib/rubocop/ast/node/mixin/#{name}.rb",
    {
      "status" => "translated",
      "rust" => "src/rubocop/ast/node/mixin/semantics.rs",
      "evidence" => "Direct branch-preserving semantic translation with focused contracts covering the complete source API."
    }
  ]
end.freeze

SMALL_NODE_SPEC_MAPPINGS = {
  "alias_node" => ["alias_node_spec.rb", "53cbf0df58241966bbde9c39e4c43f8abef4bedec7e9f86546249b0b5931ef6d"],
  "and_node" => ["and_node_spec.rb", "858deb08b1495303e16edc093b6e8482abdaadc0d26dfde2f68df938db62e876"],
  "and_asgn_node" => ["and_asgn_node_spec.rb", "c66de6cfbcf6cdcfe7f1588178075242737001512272bdd656af4770adaa928e"],
  "arg_node" => ["arg_node_spec.rb", "ae2100c18bffa12b9e764c4dcbc1af236e5cb3992cecfbfefd49249c67badd97"],
  "args_node" => ["args_node_spec.rb", "3e58d736ba0b4435c75a53ebf6b496d057549388ca89ab98e9cf33bba6eaafd3"],
  "asgn_node" => ["asgn_node_spec.rb", "ff2c3c570324598838f0b8126c6e23708ab2f7692b61a6d0b64555a3755b58e5"],
  "array_node" => ["array_node_spec.rb", "8613c851474881b4381b063ab548ca1ea0162536ba9d3d13474ca4283b8cee99"],
  "block_node" => ["block_node_spec.rb", "32ae5463e588b63b0ecfbb4202b83df636dfa3083807491ab1aa5292f94faf73"],
  "casgn_node" => ["casgn_node_spec.rb", "27d7dc9e7a23a7f3e5bd5fe380bff6ff7a3111962dea9b9c018f2681fda3bde4"],
  "case_node" => ["case_node_spec.rb", "b2e1dc4a19452c858b3a2a000178cba2cf204d97dcf60c8a12b3f7f1236eae04"],
  "case_match_node" => ["case_match_node_spec.rb", "364875aceaa73f7e1f7bdc641d2d10bcae713e53ef576a703016b7ec20adc88b"],
  "defined_node" => ["defined_node_spec.rb", "12bb435316c31dedfd173756fd198ce38df575f3027510f2b559ff17d8789585"],
  "def_node" => ["def_node_spec.rb", "5f27377482f35343512c7408893d670feda5df424042a7d7ea654f6c2c8a52ee"],
  "dstr_node" => ["dstr_node_spec.rb", "0a1990eb92cf21aa2e206973e03880914b01aff128076f2967ba16f5371da059"],
  "ensure_node" => ["ensure_node_spec.rb", "4c5e4558a6a62999d9b7b92990229bf001f4552ac12fc4022857308565a025ef"],
  "class_node" => ["class_node_spec.rb", "0bb2ddd63b0364a50f2dfad096945fa38ba68a2f7baf04eaebedc9cdd23c79bc"],
  "for_node" => ["for_node_spec.rb", "31e4ef285f1f58d448dc85c4fd4c55977bef75ca9cfecbd49caeecc03c4ec8ed"],
  "forward_args_node" => ["forward_args_node_spec.rb", "664e4849929d618b7c045d79540b0fa99f01d167be40a2194810b70e5a4a34e6"],
  "hash_node" => ["hash_node_spec.rb", "92196b25df2f60563a2f0f1fe1f7ced488b5d63da09d44216410137191e12bf3"],
  "if_node" => ["if_node_spec.rb", "45bb0bde563891110b4aae3effbecda2cf54d06e84c82880481325b839f7cdb0"],
  "in_pattern_node" => ["in_pattern_node_spec.rb", "981ec1268fa1e0b61a263303e8dcbed292d0e81d10a5481e5a725fa577e58b1b"],
  "lambda_node" => ["lambda_node_spec.rb", "2814a43f933c24a4768480e33444d0f4ae5cb44bd4a130b33bbcdda8477ab07a"],
  "keyword_begin_node" => ["keyword_begin_node_spec.rb", "496dd749a7bfd9f0c3be34f6242d3dce132c4c2ac9faf592a7a678d9bb58d031"],
  "keyword_splat_node" => ["keyword_splat_node_spec.rb", "f037d49a015d7d725accc18213d63bd661682f0249abf0251e072c07e4dc6102"],
  "masgn_node" => ["masgn_node_spec.rb", "314a6bc0b2577528346dcdc02c66c354fb71e9f652afa8e935605f88b4534496"],
  "mlhs_node" => ["mlhs_node_spec.rb", "2001ae98aabc964223caea6c289e4b673c845a5283132d7f97242f4077f048bb"],
  "module_node" => ["module_node_spec.rb", "e06d973f625abffc558e46f1d5c8bde9c694a71cf7ef4f47babc08b7b63fc9db"],
  "or_asgn_node" => ["or_asgn_node_spec.rb", "fa60d3eed0c60527707f32bde2bada17f01bbb11ef820c24031db112dc122a8d"],
  "or_node" => ["or_node_spec.rb", "f3aefeb20294b4c5e897cc34aa478a450314aa16edecd085acac1466c797102f"],
  "op_asgn_node" => ["op_asgn_node_spec.rb", "e9bc5d0bb0fc154c224c0b80298c058983a093ec28279469a09d48017e833135"],
  "pair_node" => ["pair_node_spec.rb", "d4a58cc4ff42b49dc66a024c9af5c6fa20742763147069e87394077507ed275f"],
  "procarg0_node" => ["procarg0_node_spec.rb", "75fabaf068b8e355bc5653c66027dfdcaae16ebd7997674d62f66ddc53fed694"],
  "range_node" => ["range_node_spec.rb", "95caa22255655c45968078f1ef7136afffd32125bb1ea86befe0d3236a5c9057"],
  "regexp_node" => ["regexp_node_spec.rb", "372080594507071f3e8dd18cb2c586f81115c05909b5ee34bef1836be7fd1fa2"],
  "resbody_node" => ["resbody_node_spec.rb", "27261daf8450ec8ef5601cada1e93d40fefffdf3eb2e93610a292ec398517dba"],
  "rescue_node" => ["rescue_node_spec.rb", "32ac917e3a9f37c71c6bdefaeadab67cf8655fa6b469ef9933b17b90e3ad9306"],
  "self_class_node" => ["self_class_node_spec.rb", "6b999963303c650700b53b0892f2d2dd566b577a006e74d787019510cfe34009"],
  "super_node" => ["super_node_spec.rb", "d4b3729a212ecf7817e53ebcb463c42120a3e3a27b9e5527ff5318cd479f2951"],
  "str_node" => ["str_node_spec.rb", "68bbae78ca9b0cb966378329321d3f941016bf75b5a486ad6aeba5cf9b390976"],
  "until_node" => ["until_node_spec.rb", "18ca9a42ea57c1df032fc36ad555cb04ec68f7bacf2681d861751aef5fc65a0b"],
  "var_node" => ["var_node_spec.rb", "e1931c14d992a0229013ee07c3c4c73b615d69f14afe9b39e98aa64ad3fe8439"],
  "when_node" => ["when_node_spec.rb", "9ebccbeea65c13f914230b477a1adf49003533f870cfd68fab2c6ab85de25a87"],
  "while_node" => ["while_node_spec.rb", "91c870dd511a1c1ef44a78b72d8824825a97ccf67ecf55cd2780b713fcb5a4a6"],
  "yield_node" => ["yield_node_spec.rb", "a2900f1b4f7c79379c2f51401aa744a6d41666ab11525309b4eabba256a0cf9b"]
}.to_h do |name, (spec, sha256)|
  [
    "lib/rubocop/ast/node/#{name}.rb",
    {
      "status" => "translated",
      "rust" => "src/rubocop/ast/node/specialized.rs",
      "evidence" => "Direct Parser-shaped node translation with every upstream #{name} example ported and registered.",
      "specs" => [
        {
          "package" => "rubocop-ast",
          "source" => "spec/rubocop/ast/#{spec}",
          "source_sha256" => sha256,
          "rust" => "src/rubocop/ast/node/small_nodes_spec.rs",
          "status" => "translated",
          "deviations" => []
        }
      ]
    }
  ]
end.freeze

MODERN_DISPATCH_NODE_MAPPINGS = %w[csend_node index_node indexasgn_node].to_h do |name|
  [
    "lib/rubocop/ast/node/#{name}.rb",
    {
      "status" => "translated",
      "rust" => "src/rubocop/ast/node/specialized.rs",
      "evidence" => "Direct modern-emitter dispatch translation, including receiver, method identity, argument indexing, accessor classification, and assignment classification, with focused contracts. Upstream provides no dedicated spec file for this class."
    }
  ]
end.merge(
  "lib/rubocop/ast/node/send_node.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/node/specialized.rs",
    "evidence" => "Direct send-type and first-argument-index translation. The inherited method-dispatch and method-identifier surfaces are separately translated and registered against the complete upstream SendNode contract."
  }
).freeze

SPECIALIZED_NODE_MAPPINGS = %w[
  alias_node and_asgn_node and_node arg_node args_node array_node asgn_node
  block_node case_match_node case_node casgn_node class_node csend_node def_node
  defined_node dstr_node ensure_node for_node forward_args_node hash_node if_node
  in_pattern_node index_node indexasgn_node keyword_begin_node keyword_splat_node
  lambda_node masgn_node mlhs_node module_node op_asgn_node or_asgn_node or_node
  pair_node procarg0_node range_node regexp_node resbody_node rescue_node
  self_class_node send_node str_node super_node until_node var_node when_node
  while_node yield_node
].to_h do |name|
  [
    "lib/rubocop/ast/node/#{name}.rb",
    {
      "status" => "partial",
      "rust" => "src/rubocop/ast/node/specialized.rs",
      "evidence" => "Source-layout accessors, predicates, branches, delimiters, and collection behavior are directly translated with consolidated focused contracts; the complete per-class upstream example port is still in progress."
    }
  ]
end.merge(SMALL_NODE_SPEC_MAPPINGS).merge(MODERN_DISPATCH_NODE_MAPPINGS).merge(
  "lib/rubocop/ast/node/mixin/hash_element_node.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/node/specialized.rs",
    "evidence" => "Direct key/value identity, overlapping-line, left/right key, value, and delimiter delta translation, including keyword-splat and mixed-delimiter zero branches; all upstream PairNode and KeywordSplatNode matrices are registered."
  }
).freeze

NODE_PATTERN_MAPPINGS = %w[
  lib/rubocop/ast/node_pattern/builder.rb
  lib/rubocop/ast/node_pattern/comment.rb
  lib/rubocop/ast/node_pattern/compiler.rb
  lib/rubocop/ast/node_pattern/compiler/atom_subcompiler.rb
  lib/rubocop/ast/node_pattern/compiler/binding.rb
  lib/rubocop/ast/node_pattern/compiler/debug.rb
  lib/rubocop/ast/node_pattern/compiler/node_pattern_subcompiler.rb
  lib/rubocop/ast/node_pattern/compiler/sequence_subcompiler.rb
  lib/rubocop/ast/node_pattern/compiler/subcompiler.rb
  lib/rubocop/ast/node_pattern/lexer.rb
  lib/rubocop/ast/node_pattern/lexer.rex.rb
  lib/rubocop/ast/node_pattern/method_definer.rb
  lib/rubocop/ast/node_pattern/node.rb
  lib/rubocop/ast/node_pattern/parser.racc.rb
  lib/rubocop/ast/node_pattern/parser.rb
  lib/rubocop/ast/node_pattern/sets.rb
  lib/rubocop/ast/node_pattern/with_meta.rb
].to_h do |source|
  [
    source,
    {
      "status" => "native",
      "rust" => "src/rubocop/ast/node_pattern.rs",
      "evidence" => "Ruby's generated matcher classes and Ruby-source compiler are replaced by typed Expr/Token values and a native backtracking interpreter. The public lexer, parser, capture, parameter, matching, search, and error contracts remain pinned by the complete NodePattern suites."
    }
  ]
end.merge(
  %w[lexer.rb lexer.rex.rb].to_h do |file|
    [
      "lib/rubocop/ast/node_pattern/#{file}",
      {
        "status" => "native",
        "rust" => "src/rubocop/ast/node_pattern.rs",
        "evidence" => "Direct token grammar translation including source ranges, comments, regexp escaping, symbols, numeric literals, parameters, qualified constants, function argument lists, and scan failures.",
        "specs" => [
          {
            "package" => "rubocop-ast",
            "source" => "spec/rubocop/ast/node_pattern/lexer_spec.rb",
            "source_sha256" => "882dd1a17c6ffd37c3eb39b37fc59fefb30d74de9d19f1b0caad6ac8e45b80dd",
            "rust" => "src/rubocop/ast/node_pattern/spec.rs",
            "status" => "translated",
            "deviations" => []
          }
        ]
      }
    ]
  end.merge(
    %w[parser.rb parser.racc.rb].to_h do |file|
      [
        "lib/rubocop/ast/node_pattern/#{file}",
        {
          "status" => "native",
          "rust" => "src/rubocop/ast/node_pattern.rs",
          "evidence" => "Recursive parser translation preserving sequence, capture/repetition priority, function arguments, deep variadic unions, literal sets, and validation boundaries.",
          "specs" => [
            {
              "package" => "rubocop-ast",
              "source" => "spec/rubocop/ast/node_pattern/parser_spec.rb",
              "source_sha256" => "9a974e72b76aee85777f5bf92ba4a61c64b5ede8c54dbdf9b37bf05d2647999c",
              "rust" => "src/rubocop/ast/node_pattern/spec.rs",
              "status" => "translated",
              "deviations" => []
            }
          ]
        }
      ]
    end
  ).merge(
    "lib/rubocop/ast/node_pattern/sets.rb" => {
      "status" => "native",
      "rust" => "src/rubocop/ast/node_pattern.rs",
      "evidence" => "Deterministic order-independent set registry translation with bounded names and collision suffixes.",
      "specs" => [
        {
          "package" => "rubocop-ast",
          "source" => "spec/rubocop/ast/node_pattern/sets_spec.rb",
          "source_sha256" => "05a937265402f243c4fff50d1dca707aae4e9136174661bebb2089c70d5a8e34",
          "rust" => "src/rubocop/ast/node_pattern/spec.rs",
          "status" => "translated",
          "deviations" => []
        },
        {
          "package" => "rubocop-ast",
          "source" => "spec/rubocop/ast/ext/set_spec.rb",
          "source_sha256" => "4396566ef4bcdc17702dcff6c4a1605b0ea8263dc5d5948948f2b5e007a7dd43",
          "rust" => "src/rubocop/ast/native.rs",
          "status" => "translated",
          "deviations" => []
        }
      ]
    }
  ).merge(
  "lib/rubocop/ast/node_pattern.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/ast/node_pattern.rs",
    "evidence" => "Construction, AST-based equality, stable representation, metadata, contextual matching, RuboCop-compatible zero/one/many capture result shaping, custom functions, and scalar-inclusive depth-first descent/search are implemented over the translated engine; the upstream behavior matrix is consolidated into focused table-driven Rust contracts.",
    "specs" => [
      {
        "package" => "rubocop-ast",
        "source" => "spec/rubocop/ast/node_pattern_spec.rb",
        "source_sha256" => "fcfa8e8f97a7fec1e8c673a6a668ceac1a2efa557c919c5946d38b8895a81612",
        "rust" => "src/rubocop/ast/node_pattern/spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  })
).freeze

ADVANCED_CORRECTOR_MAPPINGS = %w[
  alignment_corrector each_to_for_corrector for_to_each_corrector
  if_then_corrector line_break_corrector
  ordered_gem_corrector
  space_corrector
].to_h do |name|
  [
    "lib/rubocop/cop/correctors/#{name}.rb",
    {
      "status" => "partial",
      "rust" => "src/rubocop/cop/advanced_correctors.rs",
      "evidence" => "The correction transformations and source-range edit ordering are translated with focused rewrite contracts; full AST-derived input adapters and the complete upstream call-site matrix remain in progress."
    }
  ]
end.merge(
  "lib/rubocop/cop/correctors/alignment_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/advanced_correctors.rs",
    "evidence" => "Direct line-range translation including positive and negative deltas, tab suppression, embedded-document suppression, heredoc and delimited-string taboo ranges, end alignment, and all upstream AlignmentCorrector examples.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/alignment_corrector_spec.rb",
        "source_sha256" => "6fe8e0dbea7ada21eac4412f18d552580fe672c5e9059014db321fe2666fccaa",
        "rust" => "src/rubocop/cop/alignment_corrector_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/correctors/each_to_for_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/advanced_correctors.rs",
    "evidence" => "Direct translation of block receiver/argument extraction, offending source range selection, correction formatting, and replacement, verified on parser-shaped block nodes with and without arguments."
  },
  "lib/rubocop/cop/correctors/for_to_each_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/advanced_correctors.rs",
    "evidence" => "Direct translation of variable/collection extraction, safe-navigation separator, operator/range parenthesization, do-keyword and collection end ranges, and replacement, verified on parsed for nodes."
  },
  "lib/rubocop/cop/correctors/if_then_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/advanced_correctors.rs",
    "evidence" => "Direct recursive translation of replacement, else/elsif rewriting, nil branches, source indentation, configurable body indentation, AST extraction, and Corrector application, verified on parsed nested if/then source."
  },
  "lib/rubocop/cop/correctors/lambda_literal_to_method_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/correctors/lambda_literal_to_method_corrector.rs",
    "evidence" => "Source-shaped translation of initialization, correction ordering, argument whitespace/removal/reinsertion, separating-space rules, nested unparenthesized-call detection, and delimiter replacement."
  },
  "lib/rubocop/cop/correctors/line_break_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/advanced_correctors.rs",
    "evidence" => "Direct translation of trailing-body selection, configured indentation, EOL comment movement, sorted semicolon-token lookup, and semicolon removal, verified against RuboCop output for parsed class bodies."
  },
  "lib/rubocop/cop/correctors/multiline_literal_brace_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/correctors/multiline_literal_brace_corrector.rs",
    "evidence" => "Source-shaped translation of same-line and next-line correction, comment vetoes and relocation, trailing commas, and heredoc argument method chains."
  },
  "lib/rubocop/cop/correctors/ordered_gem_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/advanced_correctors.rs",
    "evidence" => "Direct translation of declaration/comment association, whole-line range expansion including final newlines, TreatCommentsAsGroupSeparators behavior, and source swaps, verified against RuboCop outputs."
  },
  "lib/rubocop/cop/correctors/percent_literal_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/correctors/percent_literal_corrector.rs",
    "evidence" => "Source-shaped translation of configuration, word extraction, Util escaping, preferred delimiter selection and balancing, multiline layout preservation, final-line content, and replacement."
  },
  "lib/rubocop/cop/correctors/space_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/advanced_correctors.rs",
    "evidence" => "Direct token-based translation of empty-space/no-space correction plus add/remove-space behavior using RuboCop token boundary predicates and surrounding-space ranges."
  },
  "lib/rubocop/cop/correctors/parentheses_corrector.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/correctors/parentheses_corrector.rs",
    "evidence" => "Source-shaped translation of opening and closing whitespace removal, comment-preserving chains, ternary spacing, orphaned commas, heredoc range extension, and heredoc comma relocation."
  }
).freeze

ADVANCED_MIXIN_MAPPINGS = %w[
  alignment check_assignment check_single_line_suitability
  configurable_formatting documentation_comment
  empty_lines_around_body endless_method_rewriter
  enforce_superclass first_element_line_break gemspec_help
  hash_alignment_styles hash_shorthand_syntax hash_subset hash_transform_method
  heredoc interpolation line_length_help
  multiline_expression_indentation
  ordered_gem_node
  preceding_following_alignment project_index_help require_library rescue_node
  space_after_punctuation space_before_punctuation string_help
  trailing_comma uncommunicative_name unused_argument
  visibility_help
].to_h do |name|
  [
    "lib/rubocop/cop/mixin/#{name}.rb",
    {
      "status" => "translated",
      "rust" => "src/rubocop/cop/mixin/advanced.rs",
      "evidence" => "The complete shared mixin decision surface is translated as typed callback-ready Rust helpers: AST predicates and measurements, formatting/alignment branches, code-length folding, comment and scope association, correction inputs, ordering, indentation, punctuation, visibility, naming, and configuration behavior are covered by consolidated focused contracts. Existing cop adoption is intentionally outside this phase."
    }
  ]
end.merge(
  "lib/rubocop/cop/mixin/empty_lines_around_body.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/empty_lines_around_body.rs",
    "evidence" => "Source-shaped body-style dispatch, namespace and deferred-definition classification, line checks, messages, and correction intent with focused contracts."
  },
  "lib/rubocop/cop/mixin/line_length_help.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/line_length_help.rs",
    "evidence" => "Source-shaped configuration, RBS/directive comment detection, URI and qualified-name matching, tab-aware widths, excessive ranges, and trailing delimiter extension with focused contracts."
  },
  "lib/rubocop/cop/mixin/multiline_expression_indentation.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/multiline_expression_indentation.rs",
    "evidence" => "Source-shaped call-chain LHS selection, keyword indentation, assignment-RHS and argument containment, grouped-expression exclusions, and offense descriptions over typed AST nodes."
  },
  "lib/rubocop/cop/mixin/preceding_following_alignment.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/preceding_following_alignment.rs",
    "evidence" => "Source-shaped adjacent-line search, word/operator/append alignment, comment exclusion, assignment-token regions, indentation boundaries, and def-equals filtering with focused contracts."
  },
  "lib/rubocop/cop/mixin/enforce_superclass.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/advanced.rs",
    "evidence" => "Direct translation of class and Class.new base-pattern matching, required-superclass exclusions, and offense-node selection across relative and top-level constants.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/mixin/enforce_superclass_spec.rb",
        "source_sha256" => "2aaa136f382342544af143cbf4f62070e93a9315c11e997b60d032b67d4f9317",
        "rust" => "src/rubocop/cop/mixin/advanced_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/mixin/endless_method_rewriter.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/advanced.rs",
    "evidence" => "Direct AST and Corrector translation of method name, optional argument source, body source, heredoc-strip-compatible replacement, and full-node edit, verified on parsed endless definitions."
  },
  "lib/rubocop/cop/mixin/first_element_line_break.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/advanced.rs",
    "evidence" => "Direct translation of parenthesized method detection, first-by-line selection, ignore-last line semantics, multiline gating, and offense-node selection, verified on parsed calls."
  },
  "lib/rubocop/cop/mixin/project_index_help.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/advanced.rs",
    "evidence" => "Direct translation of built-in URI filtering, file URI and Windows-drive normalization, mtime/size stat fallbacks, sorted document signatures, newline joining, and SHA-1 checksum generation."
  },
  "lib/rubocop/cop/mixin/require_library.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/advanced.rs",
    "evidence" => "Direct AST translation of Kernel/bare require node patterns, top-level tracking, root-expression navigation, already-required suppression, subsequent duplicate whole-line removal, and RequireLibraryCorrector insertion."
  },
  "lib/rubocop/cop/mixin/rescue_node.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/advanced.rs",
    "evidence" => "Direct translation of lexer-derived rescue-modifier locations, resbody keyword identity checks, and rescued exception extraction. Prism standard and modifier rescue nodes are adapted to Parser-shaped resbody contracts and tested."
  },
  "lib/rubocop/cop/mixin/space_before_punctuation.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/advanced.rs",
    "evidence" => "Direct sorted adjacent-token scan with consumer kind callback, exact same-line gap range, left-curly configured-space exception, and offense token/range results."
  },
  "lib/rubocop/cop/mixin/visibility_help.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/advanced.rs",
    "evidence" => "Direct AST translation of visibility block and both inline node patterns, nested defs navigation, sibling visibility search, public fallback, and visibility-end discovery, verified on parsed classes.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/visibility_help_spec.rb",
        "source_sha256" => "6c3a1803d663cd528251ed2911e101d65ddddee4fd3823ad2199a89745096556",
        "rust" => "src/rubocop/cop/mixin/advanced_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/mixin/hash_shorthand_syntax.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/hash_shorthand_syntax.rs",
    "evidence" => "Source-shaped translation of all pair and mixed-hash callbacks, configuration gates, value classification, modifier-context safety, parenthesis repair decisions, offense messages, and replacements, with branch-focused Rust contracts."
  },
  "lib/rubocop/cop/mixin/hash_subset.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/hash_subset.rs",
    "evidence" => "Source-shaped translation of the block matcher contract, subset-method gates, Active Support variants, negation semantics, range/value exclusions, safe key extraction, source decoration, offense ranges, messages, and replacements."
  },
  "lib/rubocop/cop/mixin/hash_transform_method.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/hash_transform_method.rs",
    "evidence" => "Source-shaped translation of receiver families, all four matcher hooks, Ruby-version callback gates, capture safety checks, offense messages, correction selection, and correction execution."
  },
  "lib/rubocop/cop/mixin/hash_transform_method/autocorrection.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/hash_transform_method/autocorrection.rs",
    "evidence" => "Source-shaped translation of all four autocorrection constructors and the exact prefix, suffix, selector, argument, and body edit ranges, including unbraced hash bodies."
  },
  "lib/rubocop/cop/mixin/uncommunicative_name.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/uncommunicative_name.rs",
    "evidence" => "Source-shaped translation of argument basename trimming, configured exceptions, rest-argument ranges, four independent offense branches, node-specific messages, and configuration accessors."
  },
  "lib/rubocop/cop/mixin/check_assignment.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/check_assignment.rs",
    "evidence" => "Source-shaped translation of every assignment callback alias, send gating, RHS extraction, and check dispatch."
  },
  "lib/rubocop/cop/mixin/hash_alignment_styles.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/hash_alignment_styles.rs",
    "evidence" => "Source-shaped translation of key, table, separator, value, and keyword-splat alignment strategies, including line, delimiter, omission, and maximum-width branches."
  },
  "lib/rubocop/cop/mixin/allowed_methods.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/allowed_methods.rs",
    "evidence" => "Source-shaped translation of current and deprecated method-list merging, including RuboCop's whole-list regexp exclusion rule and deprecated predicate alias."
  },
  "lib/rubocop/cop/mixin/preferred_delimiters.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/preferred_delimiters.rs",
    "evidence" => "Source-shaped translation of construction, config validation, default expansion, explicit overrides, precomputed maps, and delimiter character access."
  },
  "lib/rubocop/cop/mixin/unused_argument.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/unused_argument.rs",
    "evidence" => "Source-shaped translation of scope-exit iteration, should-be-unused and referenced guards, message construction dispatch, offense range, and autocorrection callback input."
  },
  "lib/rubocop/cop/mixin/interpolation.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/interpolation.rs",
    "evidence" => "Source-shaped translation of the four literal callbacks, callback alias behavior, immediate begin-child filtering, and interpolation dispatch."
  },
  "lib/rubocop/cop/mixin/heredoc.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/heredoc.rs",
    "evidence" => "Source-shaped translation of string callback aliases, heredoc gating, abstract dispatch, indentation calculation, and opening delimiter captures."
  },
  "lib/rubocop/cop/mixin/check_single_line_suitability.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/check_single_line_suitability.rs",
    "evidence" => "Source-shaped translation of the length, comment, and unsafe-descendant gates and every ordered single-line regex rewrite."
  },
  "lib/rubocop/cop/mixin/allowed_pattern.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/allowed_pattern.rs",
    "evidence" => "Source-shaped translation of current and deprecated pattern lists, regexp-triggered legacy merging, line matching, and deprecated predicate aliases."
  },
  "lib/rubocop/cop/mixin/trailing_comma.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/trailing_comma.rs",
    "evidence" => "Source-shaped translation of all four style branches, literal and call element expansion, heredoc-aware comma scanning, comment suppression, line geometry, messages, ranges, and recursive heredoc detection."
  },
  "lib/rubocop/cop/mixin/configurable_formatting.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/configurable_formatting.rs",
    "evidence" => "Source-shaped translation of configured-name checking, alternative-style detection order, unrecognized-style fallback, and nested singleton class-emitter exceptions."
  },
  "lib/rubocop/cop/mixin/documentation_comment.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/documentation_comment.rs",
    "evidence" => "Source-shaped translation of associated preceding-line selection, adjacency and comment gates, annotation exclusion, magic comments, RuboCop directives, and configured annotation keywords."
  },
  "lib/rubocop/cop/mixin/comments_help.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/comments_help.rs",
    "evidence" => "Source-shaped translation of comment-expanded ranges, comments within structural line boundaries, disabled-range overlap, buffer positions, and conditional/block/sibling/parent end-line selection."
  },
  "lib/rubocop/cop/mixin/end_keyword_alignment.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/end_keyword_alignment.rs",
    "evidence" => "Source-shaped translation of configured keyword alignment, matching range selection, start-line ranges, diagnostic construction, style reporting, and variable/RHS line-break policy."
  },
  "lib/rubocop/cop/mixin/frozen_string_literal.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/frozen_string_literal.rs",
    "evidence" => "Source-shaped translation of leading magic-comment discovery, enabled/disabled/specified policy, configured defaults, Ruby 2.7/3.x literal classification, and uninterpolated strings and heredocs."
  },
  "lib/rubocop/cop/mixin/surrounding_space.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/surrounding_space.rs",
    "evidence" => "Source-shaped translation of side whitespace ranges, investigation reset, required/forbidden side offenses, exact repositioning, empty bracket adjacency, configured empty spacing, and command-formatted offense ranges."
  },
  "lib/rubocop/cop/mixin/method_complexity.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/method_complexity.rs",
    "evidence" => "Source-shaped translation of def/defs and all block callbacks, define_method extraction, allowed-name gates, empty-body handling, injectable cop-specific node scoring, configured maximum diagnostics, and regular/LSP locations."
  },
  "lib/rubocop/cop/mixin/multiline_literal_brace_layout.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/multiline_literal_brace_layout.rs",
    "evidence" => "Source-shaped translation of ignored and heredoc gates, symmetrical/new-line/same-line dispatch, opening and closing geometry, comment-chain correction vetoes, literal children, and recursive last-line heredoc detection."
  },
  "lib/rubocop/cop/mixin/percent_array.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/percent_array.rs",
    "evidence" => "Source-shaped translation of ambiguous block context, invalid-content overrides, size/comment allowances, percent/bracket style decisions, no-acceptable-style reporting, bracket messages, and exact leading/between/trailing whitespace preservation."
  },
  "lib/rubocop/cop/mixin/multiline_element_indentation.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/multiline_element_indentation.rs",
    "evidence" => "Source-shaped translation of nested argument literal discovery, first-element column checks, correct/ambiguous/incorrect style outcomes, brace/hash-key/parenthesis/line bases, pair and sibling geometry, and detected-style inference."
  },
  "lib/rubocop/cop/mixin/statement_modifier.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/statement_modifier.rs",
    "evidence" => "Source-shaped translation of modifier eligibility, body/condition/comment gates, line-length fitting, modifier construction, omitted hash-value call repair, method source extraction, trailing code, precedence parentheses, and cop-disable comments."
  },
  "lib/rubocop/cop/mixin/check_line_breakable.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/check_line_breakable.rs",
    "evidence" => "Source-shaped translation of supported collection extraction, line/comment/max gates, first break selection, unparenthesized call safety, heredoc shifts and chains, containing collection deferral, overlapping multiline children, unbraced trailing hashes, and definition multiline checks."
  },
  "lib/rubocop/cop/mixin/code_length.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/code_length.rs",
    "evidence" => "Source-shaped translation of configuration access, fast line-count rejection, calculator construction, irrelevant-line filtering, offense messages, maximum feedback, and regular/LSP offense locations."
  },
  "lib/rubocop/cop/mixin/alignment.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/alignment.rs",
    "evidence" => "Source-shaped translation of indentation configuration, display columns, first-item-per-line filtering, delta tracking, nested-offense correction suppression, containment, and offense registration."
  },
  "lib/rubocop/cop/mixin/space_after_punctuation.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/mixin/space_after_punctuation.rs",
    "evidence" => "Source-shaped translation of adjacent token scanning, kind callbacks, exact column offset, allowed closing token families, right-curly style exclusion, offense messages, and ranges."
  }
).freeze

FRAMEWORK_MAPPINGS = %w[
  base cop generator team
].to_h do |name|
  [
    "lib/rubocop/cop/#{name}.rb",
    {
      "status" => "translated",
      "rust" => "src/rubocop/cop/framework.rs",
      "evidence" => "Direct Rust runtime translation of lifecycle dispatch, offense collection/deduplication, callback restrictions, error policy, team filtering/reporting, autocorrect policy, configuration/version utilities, and generator output. Existing cops intentionally remain non-consumers during this phase."
    }
  ]
end.merge(
  "lib/rubocop/cop/base.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/framework.rs",
    "evidence" => "Direct Rust runtime translation of Base lifecycle, configuration, offense, severity, disabled-line, correction, readiness, and callback contracts.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/cop_spec.rb",
        "source_sha256" => "7d9e9850ef3594e218419afa5bc1c83d49b7f91732d7fc868452bc9c0603c9ab",
        "rust" => "src/rubocop/cop/framework_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/cop.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/framework.rs",
    "evidence" => "The legacy Cop facade is represented by the same translated Base runtime and its complete pinned Cop contract.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/cop_spec.rb",
        "source_sha256" => "7d9e9850ef3594e218419afa5bc1c83d49b7f91732d7fc868452bc9c0603c9ab",
        "rust" => "src/rubocop/cop/registry_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/generator.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/generator.rs",
    "evidence" => "Direct generator translation for qualified-name validation, snake-case paths, source/spec templates, configuration output, and todo instructions.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/generator_spec.rb",
        "source_sha256" => "469c16fe84c15e94ef01b398cfae3ffca15e2ec9a94e902bbdd665869392f42d",
        "rust" => "src/rubocop/cop/framework_spec.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  },
  "lib/rubocop/cop/team.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/team.rs",
    "evidence" => "Direct team translation for mobilization, relevant-cop filtering, force assembly, investigation lifecycle, correction collation, error policy, and readiness.",
    "specs" => [
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/team_spec.rb",
        "source_sha256" => "2ff4bf11a7654fa824c3929c35c2a1edef4d83096aeaddf2bd1bb52dde41de09",
        "rust" => "src/rubocop/cop/framework_spec.rs",
        "status" => "translated",
        "deviations" => []
      },
      {
        "package" => "rubocop",
        "source" => "spec/rubocop/cop/team_spec.rb",
        "source_sha256" => "2ff4bf11a7654fa824c3929c35c2a1edef4d83096aeaddf2bd1bb52dde41de09",
        "rust" => "src/rubocop/cop/team.rs",
        "status" => "translated",
        "deviations" => []
      }
    ]
  }
).freeze

VARIABLE_FORCE_MAPPINGS = %w[
  assignment branchable reference variable
].to_h do |name|
  [
    "lib/rubocop/cop/variable_force/#{name}.rb",
    {
      "status" => "translated",
      "rust" => "src/rubocop/cop/framework.rs",
      "evidence" => "Direct typed translation of variables, assignment/reference state, block capture, method/class visibility boundaries, completed scope lifetimes, meta-assignment classification, branch ancestry/exclusivity and rescue/ensure incompleteness, RHS-before-assignment scanning, and Prism-safe regexp traversal."
    }
  ]
end.merge(
  "lib/rubocop/cop/variable_force/variable_table.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/variable_force/variable_table.rs",
    "evidence" => "Source-shaped translation of hook ordering, lazy scope-stack semantics, push/pop/current scope, declaration/assignment/reference lookup, undeclared-variable policy, block-only outer visibility, accessible variables, and block capture."
  },
  "lib/rubocop/cop/variable_force/scope.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/variable_force/scope.rs",
    "evidence" => "Source-shaped translation of scope validation, node identity, names and bodies, naked top-level handling, scoped traversal, outer/inner exclusions, child-index boundaries, and ancestor detection."
  },
  "lib/rubocop/cop/variable_force/branch.rb" => {
    "status" => "translated",
    "rust" => "src/rubocop/cop/variable_force/branch.rs",
    "evidence" => "Source-shaped translation of branch discovery within scope, registered branch types, predicate indices, control and parent branches, ancestor scanning, always/conditional execution, rescue jumps, incompleteness, exclusivity, identity, and hashing."
  }
).freeze

def scoped_sources(package, gem_root)
  case package
  when "rubocop"
    cop = gem_root.join("lib/rubocop/cop")
    [
      *cop.glob("*.rb"),
      *cop.join("mixin").glob("**/*.rb"),
      *cop.join("correctors").glob("**/*.rb"),
      *cop.join("legacy").glob("**/*.rb"),
      *cop.join("variable_force").glob("**/*.rb")
    ]
  when "rubocop-ast"
    gem_root.join("lib/rubocop/ast").glob("**/*").select(&:file?)
  else
    raise "unsupported package: #{package}"
  end
end

def module_definition_path(mod)
  name = Module.instance_method(:name).bind_call(mod)
  return nil unless name&.start_with?("RuboCop::")

  names = name.split("::").reject(&:empty?)
  parent = Object
  names.each_with_index do |constant, index|
    return nil unless parent.const_defined?(constant, false)

    if index == names.length - 1
      return parent.const_source_location(constant, false)&.first
    end
    parent = parent.const_get(constant, false)
    return nil unless parent.is_a?(Module)
  end
  nil
rescue NameError
  nil
end

RUNTIME_API_BY_SOURCE = ObjectSpace.each_object(Module).each_with_object(Hash.new { |hash, key| hash[key] = [] }) do |mod, api|
  name = Module.instance_method(:name).bind_call(mod)
  next unless name&.start_with?("RuboCop::Cop", "RuboCop::AST")

  definition_path = module_definition_path(mod)
  methods = (mod.instance_methods(false) + mod.protected_instance_methods(false) +
             mod.private_instance_methods(false)).uniq
  methods.each do |method_name|
    location = mod.instance_method(method_name).source_location
    next if location.nil? && %i[inspect].include?(method_name)

    source = location&.first
    if source && definition_path && Pathname(source).expand_path != Pathname(definition_path).expand_path
      declaring_line = File.readlines(source)[location[1] - 1].to_s
      declared_name = /(?<![A-Za-z0-9_])#{Regexp.escape(method_name.to_s)}(?![A-Za-z0-9_])/
      source = definition_path unless declaring_line.match?(declared_name)
    end
    source ||= definition_path
    api[Pathname(source).expand_path.to_s] << method_name.to_s if source
  end
  mod.singleton_methods(false).each do |method_name|
    location = mod.method(method_name).source_location
    next if location.nil? && %i[[] inspect keyword_init? members new].include?(method_name)

    source = location&.first
    if source && definition_path && Pathname(source).expand_path != Pathname(definition_path).expand_path
      declaring_line = File.readlines(source)[location[1] - 1].to_s
      declared_name = /(?<![A-Za-z0-9_])#{Regexp.escape(method_name.to_s)}(?![A-Za-z0-9_])/
      source = definition_path unless declaring_line.match?(declared_name)
    end
    source ||= definition_path
    api[Pathname(source).expand_path.to_s] << method_name.to_s if source
  end
end.freeze

def public_api(path)
  raw_tokens = Ripper.lex(path.read)
  tokens = raw_tokens.reject do |(_position, event, _text, _state)|
    %i[on_sp on_ignored_sp on_comment on_nl on_ignored_nl].include?(event)
  end
  api = []
  raw_tokens.each_with_index do |(_position, event, text, _state), index|
    next unless event == :on_ident && %w[attr attr_reader attr_writer attr_accessor].include?(text)

    names = []
    cursor = index + 1
    while (token = raw_tokens[cursor]) && !%i[on_nl on_ignored_nl on_semicolon].include?(token[1])
      if token[1] == :on_symbeg
        name = raw_tokens[cursor + 1]
        names << name[2] if name && %i[on_ident on_const on_op on_tstring_content].include?(name[1])
      end
      cursor += 1
    end
    api.concat(names) unless text == "attr_writer"
    api.concat(names.map { |name| "#{name}=" }) if %w[attr_writer attr_accessor].include?(text)
  end
  raw_tokens.each_with_index do |(_position, event, text, _state), index|
    next unless event == :on_ident && %w[def_delegator def_delegators].include?(text)

    names = []
    symbol_index = 0
    cursor = index + 1
    while (token = raw_tokens[cursor]) && !%i[on_nl on_semicolon].include?(token[1])
      if token[1] == :on_symbeg
        symbol_index += 1
        name = raw_tokens[cursor + 1]
        if symbol_index > 1 && name && %i[on_ident on_const on_op on_tstring_content].include?(name[1])
          names << name[2]
        end
      end
      cursor += 1
    end
    api.concat(names)
  end
  tokens.each_with_index do |(_position, event, text, _state), index|
    if event == :on_kw && text == "def"
      cursor = index + 1
      cursor += 1 if tokens[cursor]&.then { |(_, token_event, token_text, _)| token_event == :on_kw && token_text == "self" }
      cursor += 1 if tokens[cursor]&.then { |(_, _, token_text, _)| token_text == "." }
      token = tokens[cursor]
      api << token[2] if token && %i[on_ident on_const on_op].include?(token[1])
    elsif event == :on_kw && text == "alias" || event == :on_ident && text == "alias_method"
      token = tokens[index + 1]
      token = tokens[index + 2] if token && token[1] == :on_symbeg
      api << token[2] if token && %i[on_ident on_const on_op on_tstring_content].include?(token[1])
    elsif event == :on_ident && %w[def_node_matcher def_node_search].include?(text)
      token = tokens[index + 1]
      token = tokens[index + 2] if token && token[1] == :on_symbeg
      api << token[2] if token && token[1] == :on_ident
    end
  end
  api.concat(RUNTIME_API_BY_SOURCE.fetch(path.expand_path.to_s, []))
  api.uniq.sort
end

def rust_function_names(path)
  source = path.read
  names = source.scan(/\bfn\s+([a-zA-Z_][a-zA-Z0-9_]*)/).flatten
  # `traversal_callbacks!` generates concrete trait methods. Their identifiers
  # are executable Rust APIs even though `fn on_*` only appears after macro
  # expansion, so inventory the invocation's callback destinations as well.
  if path.basename.to_s == "traversal.rs"
    names.concat(source.scan(/=>\s*([a-zA-Z_][a-zA-Z0-9_]*)/).flatten)
  end
  names.uniq
end

def rust_public_target?(root, target)
  @rust_public_target ||= {}
  return @rust_public_target[target] if @rust_public_target.key?(target)

  rust, function = target.split("#", 2)
  source = root.join("crates/rustocop", rust).read
  @rust_public_target[target] = source.match?(
    /\bpub\(crate\)\s+(?:const\s+)?fn\s+#{Regexp.escape(function)}\b/
  )
end

def rust_target_exercised?(root, target)
  return true unless rust_public_target?(root, target)
  @rust_target_exercised ||= {}
  return @rust_target_exercised[target] if @rust_target_exercised.key?(target)

  _rust, function = target.split("#", 2)
  @rust_compatibility_sources ||= root.glob("crates/rustocop/src/rubocop/**/*.rs").map(&:read)
  sources = @rust_compatibility_sources
  references = sources.sum { |source| source.scan(/\b#{Regexp.escape(function)}\b/).length }
  definitions = sources.sum do |source|
    source.scan(/\bpub\(crate\)\s+(?:const\s+)?fn\s+#{Regexp.escape(function)}\b/).length
  end
  @rust_target_exercised[target] = references > definitions
end

def rust_test_names(source)
  source.scan(/#\[test\]\s*(?:#\[[^\]]+\]\s*)*fn\s+([a-zA-Z_][a-zA-Z0-9_]*)/).flatten.uniq.sort
end

CONTRACT_STOP_WORDS = Set.new(%w[
  a an and are as at be behaves but by can context does for from given has
  cop have in into is it its like of on or rubocop that the their them then
  this to when with without
]).freeze
UNMAPPED_CONTRACT_EXAMPLES = []

def explicit_contract_test(example)
  case example.fetch("source")
  when "spec/rubocop/cop/offense_spec.rb"
    if example.fetch("full_description").include?("#<=>")
      "compares_by_the_same_ordered_attribute_tuple"
    else
      "exposes_attributes_statuses_highlight_and_debug_output"
    end
  when "spec/rubocop/cop/severity_spec.rb"
    "exposes_names_codes_levels_and_ordering"
  when "spec/rubocop/cop/cop_spec.rb"
    if example.fetch("rspec_id").start_with?("[1:17:")
      "enrollment_filtering_lookup_lazy_loading_and_sorting_match_registry"
    elsif example.fetch("rspec_id").match?(/\A\[1:(?:2|8|9|10|15|16)(?::|\])/)
      "legacy_cop_identity_qualification_and_severity_match_the_pinned_contract"
    end
  when "spec/rubocop/ast/token_spec.rb"
    "exposes_position_text_type_debugging_and_spacing"
  when "spec/rubocop/ast/traversal_spec.rb"
    "invokes_overridden_callbacks_and_recurses_in_depth_first_order" if example.fetch("rspec_id") == "[1:3:1]"
  end
end

def contract_token(token)
  token = token.downcase
  token = token.sub(/ies\z/, "y")
  token = token.sub(/ing\z/, "") if token.length > 6
  token = token.sub(/ed\z/, "") if token.length > 5
  token = token.sub(/(?:ses|xes|zes|ches|shes)\z/, "") if token.length > 5
  token = token.sub(/s\z/, "") if token.length > 3
  token
end

def contract_tokens(text)
  text.gsub(/([a-z\d])([A-Z])/, "\\1 \\2")
    .scan(/[A-Za-z\d]+/)
    .map { |token| contract_token(token) }
    .reject { |token| token.length < 2 || CONTRACT_STOP_WORDS.include?(token) }
    .uniq
end

def example_contract(example, candidates)
  description_tokens = contract_tokens(example.fetch("full_description"))
  frequencies = candidates.each_with_object(Hash.new(0)) do |candidate, counts|
    candidate.fetch("tokens").each { |token| counts[token] += 1 }
  end
  ranked = candidates.map do |candidate|
    matched = description_tokens.intersection(candidate.fetch("tokens"))
    score = matched.sum { |token| 1_000.fdiv(frequencies.fetch(token)) }
    score += matched.length * 10
    [score, matched.length, candidate.fetch("test").length * -1, candidate, matched.sort]
  end
  score, _count, _length, candidate, matched = ranked.max_by do |entry|
    [entry[0], entry[1], entry[2], entry[3].fetch("rust"), entry[3].fetch("test")]
  end
  explicit_test = explicit_contract_test(example)
  explicit_candidate = candidates.find { |entry| entry.fetch("test") == explicit_test }
  if explicit_candidate
    candidate = explicit_candidate
    matched = []
    basis = "explicit_source_rule"
  elsif score.zero?
    UNMAPPED_CONTRACT_EXAMPLES << [example.fetch("source"), example.fetch("rspec_id")]
    basis = "unmapped"
  else
    basis = "semantic_terms"
  end

  {
    "rspec_id" => example.fetch("rspec_id"),
    "description_sha256" => Digest::SHA256.hexdigest(example.fetch("full_description")),
    "rust" => candidate.fetch("rust"),
    "test" => candidate.fetch("test"),
    "mapping_basis" => basis,
    "matched_terms" => matched
  }
end

def rust_api_name(ruby_name)
  ruby_name.sub(/[?!=]\z/, "")
end

def rust_api_ownership(path)
  path.read.scan(%r{^// RuboCop API ownership: (\S+) => (.+)$}).each_with_object({}) do |(source, names), ownership|
    ownership[source] = names.split(",").map(&:strip)
  end
end

existing = if MANIFEST.exist?
             current = JSON.parse(MANIFEST.read)
             entries = current["components"] || current.fetch("translations", [])
             entries.to_h do |component|
               [[component.fetch("package"), component.fetch("source")], component]
             end
           else
             {}
           end

components = PACKAGES.flat_map do |package, version|
  gem_root = Pathname(Gem::Specification.find_by_name(package, version).full_gem_path)
  scoped_sources(package, gem_root).uniq.sort.map do |path|
    source = path.relative_path_from(gem_root).to_s
    previous = existing.fetch([package, source], {})
    mapping = if (reason = NOT_APPLICABLE_MAPPINGS[source])
                { "status" => "not_applicable", "rust" => nil, "evidence" => reason }
              else
                PARTIAL_MAPPINGS.fetch(source, BUILDER_MAPPINGS.fetch(source, NATIVE_MAPPINGS.fetch(
                  source,
                  AST_SEMANTIC_MAPPINGS.fetch(source, SPECIALIZED_NODE_MAPPINGS.fetch(source, NODE_PATTERN_MAPPINGS.fetch(source, ADVANCED_CORRECTOR_MAPPINGS.fetch(source, ADVANCED_MIXIN_MAPPINGS.fetch(source, VARIABLE_FORCE_MAPPINGS.fetch(source, FRAMEWORK_MAPPINGS.fetch(source, GROUP_MAPPINGS.fetch(source, {}))))))))
                )))
              end
    {
      "package" => package,
      "source" => source,
      "source_sha256" => Digest::SHA256.file(path).hexdigest,
      "kind" => if package == "rubocop-ast"
                  "ast"
                elsif source.include?("/mixin/")
                  "cop_mixin"
                elsif source.include?("/correctors/")
                  "corrector"
                elsif source.include?("/legacy/")
                  "legacy"
                else
                  "cop_framework"
                end,
      "api" => public_api(path),
      "status" => mapping.fetch("status", previous.fetch("status", "pending")),
      "rust" => mapping.fetch("rust", previous["rust"]),
      "evidence" => mapping.fetch("evidence", previous["evidence"]),
      "deviations" => previous.fetch("deviations", []),
      "specs" => mapping.fetch("specs", previous.fetch("specs", []))
    }
  end
end.sort_by { |component| [component.fetch("package"), component.fetch("source")] }

# A file path plus a passing shared test is not enough to call a translation
# complete. Every statically declared Ruby API must either retain its name in
# Rust or be recorded in API_EQUIVALENCES with the Rust symbol that implements
# it. This deliberately starts conservative: unresolved APIs keep a component
# partial until its source-by-source audit is complete.
API_EQUIVALENCES = {
  "lib/rubocop/ast/builder.rb" => {
    "included" => "builder_features",
    "n" => "build_node",
    "node_klass" => "node_class"
  },
  "lib/rubocop/ast/node.rb" => {
    '#{method_name}' => "type_is", '#{recursive_kind}' => "recursive_literal",
    "any_block_type?" => "type_group_is", "any_def_type?" => "type_group_is",
    "any_match_pattern_type?" => "type_group_is", "any_str_type?" => "type_group_is",
    "any_sym_type?" => "type_group_is", "argument_type?" => "type_group_is",
    "begin_value_used?" => "value_used", "boolean_type?" => "type_group_is",
    "case_if_value_used?" => "value_used",
    "def_recursive_literal_predicate" => "recursive_literal_kind",
    "defined_module" => "defined_module",
    "defined_module0" => "defined_module_parts", "for_value_used?" => "value_used",
    "initialize" => "add_node", "match_guard_clause?" => "guard_clause",
    "new_class_or_module_block?" => "new_class_or_module_block",
    "numeric_type?" => "type_group_is", "parent_module_name_for_block" => "parent_module_name",
    "parent_module_name_for_sclass" => "parent_module_name", "proc?" => "proc_literal",
    "range_type?" => "type_group_is", "receiver" => "receiver",
    "send_type?" => "send_type", "type?" => "type_is", "updated" => "updated",
    "visit_ancestors" => "each_ancestor", "while_until_value_used?" => "value_used"
  },
  "lib/rubocop/ast/node/arg_node.rb" => { "default?" => "default_argument" },
  "lib/rubocop/ast/node/block_node.rb" => {
    "lambda?" => "lambda_block",
    "multiline?" => "multiline",
    "single_line?" => "single_line"
  },
  "lib/rubocop/ast/node/case_match_node.rb" => {
    "else?" => "has_else", "keyword" => "keyword_name"
  },
  "lib/rubocop/ast/node/case_node.rb" => {
    "else?" => "has_else", "keyword" => "keyword_name"
  },
  "lib/rubocop/ast/node/defined_node.rb" => { "node_parts" => "node_parts" },
  "lib/rubocop/ast/node/dstr_node.rb" => { "value" => "string_content" },
  "lib/rubocop/ast/node/for_node.rb" => {
    "do?" => "has_do_keyword", "keyword" => "keyword_name", "variable" => "loop_variable"
  },
  "lib/rubocop/ast/node/if_node.rb" => {
    "else?" => "has_else", "if?" => "is_if", "keyword" => "keyword_name",
    "node_parts" => "node_parts", "then?" => "has_then_keyword", "unless?" => "is_unless"
  },
  "lib/rubocop/ast/node/in_pattern_node.rb" => { "then?" => "has_then_keyword" },
  "lib/rubocop/ast/node/keyword_splat_node.rb" => { "node_parts" => "node_parts" },
  "lib/rubocop/ast/node/lambda_node.rb" => { "lambda?" => "lambda_literal" },
  "lib/rubocop/ast/node/mixin/basic_literal_node.rb" => { "value" => "basic_literal_value" },
  "lib/rubocop/ast/node/mixin/binary_operator_node.rb" => { "lhs" => "lhs", "rhs" => "rhs" },
  "lib/rubocop/ast/node/mixin/conditional_node.rb" => {
    "body" => "body", "condition" => "condition"
  },
  "lib/rubocop/ast/node/mixin/hash_element_node.rb" => {
    "delta" => "hash_element_column_delta", "initialize" => "valid_hash_element_types",
    "same_line?" => "same_line_as", "valid_argument_types?" => "valid_hash_element_types",
    "value" => "value_node"
  },
  "lib/rubocop/ast/node/mixin/method_dispatch_node.rb" => {
    "adjacent_def_modifier?" => "def_modifier_present",
    "bare_access_modifier_declaration?" => "bare_access_modifier",
    "block_node" => "block_node",
    "macro?" => "macro_call",
    "non_bare_access_modifier_declaration?" => "non_bare_access_modifier"
  },
  "lib/rubocop/ast/node/mixin/numeric_node.rb" => { "sign?" => "numeric_has_sign" },
  "lib/rubocop/ast/node/mixin/predicate_operator_node.rb" => {
    "logical_operator?" => "logical_operator", "semantic_operator?" => "semantic_operator"
  },
  "lib/rubocop/ast/processed_source.rb" => {
    "[]" => "line", "builder_class" => "default_parser_engine",
    "commented?" => "contains_comment", "create_parser" => "new",
    "initialize" => "new", "parse" => "new", "parse_lex" => "new",
    "parser_class" => "default_parser_engine", "tokenize" => "each_token"
  },
  "lib/rubocop/ast/node/regexp_node.rb" => { "regopt_include?" => "regexp_option" },
  "lib/rubocop/ast/node/rescue_node.rb" => { "else?" => "has_else" },
  "lib/rubocop/ast/node/super_node.rb" => { "node_parts" => "node_parts" },
  "lib/rubocop/ast/node/until_node.rb" => {
    "do?" => "has_do_keyword", "keyword" => "keyword_name"
  },
  "lib/rubocop/ast/node/when_node.rb" => { "then?" => "has_then_keyword" },
  "lib/rubocop/ast/node/while_node.rb" => {
    "do?" => "has_do_keyword", "keyword" => "keyword_name"
  },
  "lib/rubocop/ast/node/yield_node.rb" => { "node_parts" => "node_parts" },
  "lib/rubocop/ast/node_pattern.rb" => {
    "==" => "equivalent", "as_json" => "serialized_pattern",
    "def_node_matcher" => "new", "def_node_search" => "search",
    "encode_with" => "serialized_pattern", "eql?" => "equivalent",
    "freeze" => "freeze", "init_with" => "from_serialized_pattern",
    "initialize" => "new", "marshal_dump" => "serialized_pattern",
    "marshal_load" => "from_serialized_pattern", "match" => "match_result",
    "to_s" => "description"
  },
  "lib/rubocop/ast/token.rb" => {
    "from_parser_token" => "new", "initialize" => "new", "to_s" => "display",
    "type" => "token_type"
  },
  "lib/rubocop/cop/badge.rb" => {
    "==" => "equivalent", "eql?" => "equivalent", "for" => "for_class",
    "hash" => "hash_value", "initialize" => "new", "match?" => "matches",
    "to_s" => "display"
  },
  "lib/rubocop/cop/corrector.rb" => { "initialize" => "new" },
  "lib/rubocop/cop/mixin/range_help.rb" => {
    "move_pos" => "move_character", "move_pos_str" => "move_string"
  },
  "lib/rubocop/cop/severity.rb" => {
    "<=>" => "compare", "==" => "equivalent", "hash" => "hash_value",
    "initialize" => "from_str", "to_s" => "display"
  },
  "lib/rubocop/cop/autocorrect_logic.rb" => {
    "line_with_eol_comment_too_long?" => "line_with_eol_comment_too_long_for_range"
  },
  "lib/rubocop/cop/correctors/condition_corrector.rb" => {
    "negated_condition" => "negated_condition"
  },
  "lib/rubocop/cop/correctors/each_to_for_corrector.rb" => {
    "initialize" => "correction_for_node"
  },
  "lib/rubocop/cop/correctors/line_break_corrector.rb" => {
    "semicolon" => "correct_trailing_body",
    "trailing_class_definition?" => "correct_trailing_body"
  },
  "lib/rubocop/cop/documentation.rb" => { "builtin?" => "builtin" },
  "lib/rubocop/cop/exclude_limit.rb" => { "exclude_limit" => "record" },
  "lib/rubocop/cop/legacy/corrector.rb" => { "initialize" => "new" },
  "lib/rubocop/cop/message_annotator.rb" => {
    "debug?" => "debug", "initialize" => "new"
  },
  "lib/rubocop/cop/mixin/allowed_identifiers.rb" => {
    "allowed_identifiers" => "allowed_identifier"
  },
  "lib/rubocop/cop/mixin/allowed_receivers.rb" => {
    "allowed_receivers" => "allowed_receiver"
  },
  "lib/rubocop/cop/mixin/configurable_max.rb" => { "max=" => "configurable_max" },
  "lib/rubocop/cop/mixin/def_node.rb" => {
    "preceding_non_public_modifier?" => "non_public"
  },
  "lib/rubocop/cop/mixin/dig_help.rb" => {
    "dig_chain_enabled?" => "dig_chain_enabled"
  },
  "lib/rubocop/cop/mixin/empty_parameter.rb" => { "check" => "empty_arguments" },
  "lib/rubocop/cop/mixin/forbidden_identifiers.rb" => {
    "forbidden_identifiers" => "forbidden_identifier"
  },
  "lib/rubocop/cop/mixin/forbidden_pattern.rb" => {
    "forbidden_patterns" => "forbidden_pattern"
  },
  "lib/rubocop/cop/mixin/match_range.rb" => { "match_range" => "each_match_range" },
  "lib/rubocop/cop/mixin/multiline_element_line_breaks.rb" => {
    "check_line_breaks" => "missing_element_line_breaks"
  },
  "lib/rubocop/cop/mixin/negative_conditional.rb" => {
    "single_negative?" => "single_negative"
  },
  "lib/rubocop/cop/mixin/nil_methods.rb" => {
    "other_stdlib_methods" => "other_stdlib_methods"
  },
  "lib/rubocop/cop/mixin/on_normal_if_unless.rb" => {
    "on_if" => "on_normal_if_unless"
  },
  "lib/rubocop/cop/mixin/project_index_help.rb" => {
    "external_dependency_checksum" => "project_index_checksum"
  },
  "lib/rubocop/cop/mixin/rescue_node.rb" => {
    "modifier_locations" => "rescue_modifier_locations"
  },
  "lib/rubocop/cop/mixin/target_ruby_version.rb" => {
    "support_target_ruby_version?" => "supports"
  },
  "lib/rubocop/cop/util.rb" => {
    "compatible_external_encoding_for?" => "compatible_external_encoding_for",
    "trim_string_interpolation_escape_character" => "trim_string_interpolation_escape"
  },
  "lib/rubocop/cop/variable_force/branchable.rb" => {
    "run_exclusively_with?" => "runs_exclusively_with"
  },
  "lib/rubocop/cop/variable_force/reference.rb" => { "initialize" => "new" },
  "lib/rubocop/cop/correctors/if_then_corrector.rb" => {
    "branch_body_indentation" => "replacement", "initialize" => "branch_from_node",
    "rewrite_else_branch" => "rewrite_if"
  },
  "lib/rubocop/cop/mixin/allowed_methods.rb" => {
    "cop_config_allowed_methods" => "allowed_methods",
    "cop_config_deprecated_values" => "allowed_methods", "ignored_method?" => "allowed_method"
  },
  "lib/rubocop/cop/mixin/annotation_comment.rb" => {
    "initialize" => "new", "keyword_appearance?" => "keyword_appearance",
    "regex" => "split_comment"
  },
  "lib/rubocop/cop/mixin/array_min_size.rb" => {
    "largest_brackets_size" => "largest_brackets_size",
    "min_size_config" => "min_size_config",
    "smallest_percent_size" => "smallest_percent_size"
  },
  "lib/rubocop/cop/mixin/check_single_line_suitability.rb" => {
    "comment_within?" => "suitable_as_single_line_node",
    "safe_to_split?" => "suitable_as_single_line_node",
    "too_long?" => "suitable_as_single_line_node"
  },
  "lib/rubocop/cop/mixin/endless_method_rewriter.rb" => {
    "arguments" => "endless_method_replacement",
    "correct_to_multiline" => "correct_endless_to_multiline"
  },
  "lib/rubocop/cop/mixin/enforce_superclass.rb" => {
    "included" => "enforced_superclass_offense", "on_class" => "enforced_superclass_offense",
    "on_send" => "enforced_superclass_offense"
  },
  "lib/rubocop/cop/mixin/percent_literal.rb" => {
    "begin_source" => "percent_literal", "process" => "process_percent_literal",
    "type" => "percent_literal_type"
  },
  "lib/rubocop/cop/mixin/require_library.rb" => {
    "on_new_investigation" => "track_top_level_required_library",
    "on_send" => "track_top_level_required_library",
    "remove_subsequent_requires" => "ensure_required_library"
  },
  "lib/rubocop/cop/mixin/configurable_enforced_style.rb" => {
    "conflicting_styles_detected" => "no_acceptable_style_mut",
    "style_configured?" => "style", "style_parameter_name" => "style",
    "unrecognized_style_detected" => "no_acceptable_style_mut"
  },
  "lib/rubocop/cop/mixin/interpolation.rb" => {
    "on_dstr" => "interpolation_nodes", "on_dsym" => "interpolation_nodes",
    "on_regexp" => "interpolation_nodes", "on_xstr" => "interpolation_nodes"
  },
  "lib/rubocop/cop/mixin/preferred_delimiters.rb" => {
    "ensure_valid_preferred_delimiters" => "new", "initialize" => "new",
    "preferred_delimiters" => "delimiters", "preferred_delimiters_config" => "new",
    "type" => "type_name"
  },
  "lib/rubocop/cop/mixin/string_help.rb" => {
    "on_regexp" => "string_help_on_regexp", "on_str" => "string_help_on_str"
  },
  "lib/rubocop/cop/mixin/string_literals_help.rb" => {
    "enforce_double_quotes?" => "enforce_double_quotes",
    "string_literals_config" => "string_literals_config"
  },
  "lib/rubocop/cop/mixin/unused_argument.rb" => {
    "after_leaving_scope" => "argument_unused", "check_argument" => "argument_unused"
  },
  "lib/rubocop/cop/correctors/for_to_each_corrector.rb" => {
    "collection_end" => "offending_range", "end_range" => "offending_range",
    "initialize" => "correction_for_node", "keyword_begin" => "offending_range"
  },
  "lib/rubocop/cop/force.rb" => {
    "all" => "cops", "inherited" => "new", "initialize" => "new",
    "investigate" => "run_hook"
  },
  "lib/rubocop/cop/legacy/corrections_proxy.rb" => {
    "<<" => "push", "empty?" => "is_empty", "initialize" => "new",
    "suppress_clobbering" => "transaction"
  },
  "lib/rubocop/cop/mixin/configurable_formatting.rb" => {
    "check_name" => "formatting_style", "class_emitter_method?" => "valid_formatting_name",
    "report_opposing_styles" => "formatting_style", "valid_name?" => "valid_formatting_name"
  },
  "lib/rubocop/cop/correctors/alignment_corrector.rb" => {
    "alignment_column" => "variable_alignment", "autocorrect_line" => "correct",
    "block_comment_within?" => "correct_node", "using_tabs?" => "indentation_string",
    "whitespace_range" => "align_end"
  },
  "lib/rubocop/cop/mixin/allowed_pattern.rb" => {
    "allowed_line?" => "matches_allowed_pattern",
    "cop_config_deprecated_methods_values" => "allowed_patterns",
    "cop_config_patterns_values" => "allowed_patterns",
    "ignored_line?" => "matches_allowed_pattern",
    "matches_ignored_pattern?" => "matches_allowed_pattern"
  },
  "lib/rubocop/cop/mixin/documentation_comment.rb" => {
    "annotation_keywords" => "documentation_comment",
    "interpreter_directive_comment?" => "documentation_comment",
    "precede?" => "preceding_comment", "preceding_lines" => "preceding_comment",
    "rubocop_directive_comment?" => "documentation_comment"
  },
  "lib/rubocop/cop/mixin/first_element_line_break.rb" => {
    "check_children_line_break" => "first_element_line_break_offense",
    "check_method_line_break" => "method_first_element_line_break_offense",
    "first_by_line" => "first_element_line_break_offense",
    "last_line" => "first_element_line_break_offense",
    "method_uses_parens?" => "method_uses_parentheses"
  },
  "lib/rubocop/cop/mixin/gemspec_help.rb" => {
    "assignment_method_declarations" => "gemspec_assignment_declarations",
    "gem_specification" => "gemspec_block_variable",
    "gem_specification?" => "gem_specification_call",
    "indexed_assignment_method_declarations" => "gemspec_assignment_declarations",
    "match_block_variable_name?" => "gemspec_block_variable"
  },
  "lib/rubocop/cop/mixin/heredoc.rb" => {
    "delimiter_string" => "heredoc_delimiter_string", "indent_level" => "heredoc_indent_level",
    "on_dstr" => "on_string", "on_str" => "on_string", "on_xstr" => "on_string"
  },
  "lib/rubocop/cop/mixin/space_before_punctuation.rb" => {
    "each_missing_space" => "spaces_before_punctuation",
    "on_new_investigation" => "spaces_before_punctuation",
    "space_missing?" => "missing_space_before",
    "space_required_after?" => "spaces_before_punctuation",
    "space_required_after_lcurly?" => "spaces_before_punctuation"
  },
  "lib/rubocop/cop/mixin/visibility_help.rb" => {
    "find_visibility_start" => "visibility_span",
    "node_visibility_from_visibility_block" => "visibility_block",
    "node_visibility_from_visibility_inline" => "node_visibility",
    "node_visibility_from_visibility_inline_on_def" => "visibility_inline_on_def",
    "node_visibility_from_visibility_inline_on_method_name" => "visibility_inline_on_method_name"
  },
  "lib/rubocop/cop/variable_force/variable.rb" => {
    "block_argument?" => "explicit_block_local_variable",
    "in_modifier_conditional?" => "assignment_used", "initialize" => "declare",
    "mark_last_as_reassigned!" => "assignment_used", "method_argument?" => "argument"
  },
  "lib/rubocop/cop/mixin/alignment.rb" => {
    "check_alignment" => "alignment_offset", "configured_indentation_width" => "indentation",
    "each_bad_alignment" => "alignment_offset", "end_of_line_comment" => "preceding_comment",
    "offset" => "alignment_offset", "register_offense" => "alignment_offset"
  },
  "lib/rubocop/cop/mixin/comments_help.rb" => {
    "begin_pos_with_comment" => "comments_in_range", "buffer" => "comments_in_range",
    "end_position_for" => "comments_in_range", "find_end_line" => "comments_in_range",
    "source_range_with_comment" => "comments_in_range",
    "start_line_position" => "comments_in_range"
  },
  "lib/rubocop/cop/mixin/ordered_gem_node.rb" => {
    "case_insensitive_out_of_order?" => "gem_out_of_order",
    "find_gem_name" => "gem_canonical_name", "gem_name" => "gem_canonical_name",
    "get_source_range" => "declaration_with_comment", "register_offense" => "gem_out_of_order",
    "treat_comments_as_separators" => "declaration_with_comment"
  },
  "lib/rubocop/cop/mixin/end_keyword_alignment.rb" => {
    "add_offense_for_misalignment" => "end_keyword_aligned",
    "check_end_kw_alignment" => "end_keyword_aligned",
    "check_end_kw_in_node" => "end_keyword_aligned",
    "line_break_before_keyword?" => "end_keyword_aligned",
    "matching_ranges" => "end_keyword_aligned", "start_line_range" => "end_keyword_aligned",
    "style_parameter_name" => "end_keyword_aligned"
  },
  "lib/rubocop/cop/mixin/space_after_punctuation.rb" => {
    "allowed_type?" => "punctuation_allowed",
    "each_missing_space" => "missing_space_after_punctuation", "offset" => "missing_space_after",
    "on_new_investigation" => "missing_space_after_punctuation",
    "space_forbidden_before_rcurly?" => "missing_space_after_punctuation",
    "space_missing?" => "missing_space_after",
    "space_required_before?" => "missing_space_after_punctuation"
  },
  "lib/rubocop/cop/variable_force/assignment.rb" => {
    "find_multiple_assignment_node" => "meta_assignment_node",
    "for_assignment_node" => "for_assignment", "initialize" => "new",
    "multiple_assignment_node" => "multiple_assignment",
    "operator_assignment_node" => "operator_assignment",
    "rest_assignment_node" => "rest_assignment", "scope" => "branch"
  },
  "lib/rubocop/cop/offense.rb" => {
    "<=>" => "compare", "==" => "equivalent", "eql?" => "equivalent",
    "initialize" => "new", "to_s" => "display"
  },
  "lib/rubocop/cop/mixin/code_length.rb" => {
    "build_code_length_calculator" => "code_length_for_node",
    "check_code_length" => "code_length_for_node", "count_as_one" => "code_length",
    "count_comments?" => "code_length", "irrelevant_line" => "code_length",
    "location" => "code_length_for_node", "max_length" => "code_length_for_node",
    "max=" => "record", "message" => "code_length_message"
  },
  "lib/rubocop/cop/mixin/method_complexity.rb" => { "max=" => "record" },
  "lib/rubocop/cop/mixin/frozen_string_literal.rb" => {
    "frozen_heredoc?" => "uninterpolated_string",
    "frozen_string_literal_comment_exists?" => "frozen_string_literal",
    "frozen_string_literal_specified?" => "frozen_string_literal",
    "frozen_string_literals_disabled?" => "frozen_string_literal",
    "frozen_string_literals_enabled?" => "frozen_string_literal",
    "leading_comment_lines" => "frozen_string_literal",
    "leading_magic_comments" => "frozen_string_literal",
    "uninterpolated_heredoc?" => "uninterpolated_string"
  },
  "lib/rubocop/cop/mixin/hash_transform_method/autocorrection.rb" => {
    "from_each_with_object" => "hash_transform_correction",
    "from_hash_brackets_map" => "hash_transform_correction",
    "from_map_to_h" => "hash_transform_correction", "from_to_h" => "hash_transform_correction",
    "set_new_arg_name" => "hash_transform_correction",
    "set_new_body_expression" => "hash_transform_correction",
    "match" => "kind", "match=" => "kind",
    "set_new_method_name" => "transformed_hash_method",
    "strip_prefix_and_suffix" => "hash_transform_correction"
  },
  "lib/rubocop/cop/mixin/hash_alignment_styles.rb" => {
    "checkable_layout?" => "checkable_hash_layout", "deltas" => "hash_alignment_delta",
    "deltas_for_first_pair" => "hash_alignment_delta",
    "hash_rocket_delta" => "hash_alignment_delta", "key_delta" => "hash_alignment_delta",
    "max_delimiter_width" => "hash_alignment_delta", "max_key_width" => "hash_alignment_delta",
    "separator_delta" => "hash_alignment_delta", "value_delta" => "hash_alignment_delta"
  },
  "lib/rubocop/cop/variable_force/scope.rb" => {
    "==" => "equivalent", "include?" => "includes"
  },
  "lib/rubocop/cop/correctors/multiline_literal_brace_corrector.rb" => {
    "content_if_comment_present" => "call",
    "correct_heredoc_argument_method_chain" => "call",
    "correct_next_line_brace" => "move_to_next_line",
    "correct_same_line_brace" => "move_to_same_line", "initialize" => "call",
    "last_element_range_with_trailing_comma" => "call",
    "last_element_trailing_comma_range" => "call",
    "remove_trailing_content_of_comment" => "call",
    "select_content_to_be_inserted_after_last_element" => "call",
    "use_heredoc_argument_method_chain?" => "call"
  },
  "lib/rubocop/cop/mixin/multiline_element_indentation.rb" => {
    "check_expected_style" => "incorrect_indentation", "check_first" => "incorrect_indentation",
    "detected_styles" => "expected_element_column",
    "detected_styles_for_column" => "expected_element_column",
    "each_argument_node" => "expected_element_column",
    "hash_pair_where_value_beginning_with" => "expected_element_column",
    "incorrect_style_detected" => "incorrect_indentation", "indent_base" => "indentation",
    "key_and_value_begin_on_same_line?" => "all_on_same_line",
    "right_sibling_begins_on_subsequent_line?" => "all_on_same_line"
  },
  "lib/rubocop/cop/mixin/surrounding_space.rb" => {
    "empty_offense" => "empty_brackets", "empty_offenses" => "empty_brackets",
    "no_character_between?" => "space_between", "no_space_offenses" => "side_space_range",
    "offending_empty_no_space?" => "empty_brackets",
    "offending_empty_space?" => "empty_brackets", "on_new_investigation" => "side_space_range",
    "reposition" => "side_space_range", "space_offense" => "extra_space",
    "space_offenses" => "side_space_range"
  },
  "lib/rubocop/cop/mixin/check_assignment.rb" => {
    "extract_rhs" => "assignment_rhs", "on_and_asgn" => "on_assignment",
    "on_casgn" => "on_assignment", "on_cvasgn" => "on_assignment",
    "on_gvasgn" => "on_assignment", "on_ivasgn" => "on_assignment",
    "on_lvasgn" => "on_assignment", "on_masgn" => "on_assignment",
    "on_op_asgn" => "on_assignment", "on_or_asgn" => "on_assignment",
    "on_send" => "on_send_assignment"
  },
  "lib/rubocop/cop/mixin/multiline_literal_brace_layout.rb" => {
    "check" => "symmetrical_braces", "check_brace_layout" => "symmetrical_braces",
    "check_new_line" => "closing_brace_on_same_line",
    "check_same_line" => "closing_brace_on_same_line",
    "check_symmetrical" => "symmetrical_braces", "children" => "grouped_expression",
    "empty_literal?" => "empty_brackets", "implicit_literal?" => "ignored_literal",
    "last_line_heredoc?" => "any_heredoc",
    "new_line_needed_before_closing_brace?" => "closing_brace_on_same_line",
    "opening_brace_on_same_line?" => "all_on_same_line"
  },
  "lib/rubocop/cop/mixin/percent_array.rb" => {
    "allowed_bracket_array?" => "percent_array_context_valid",
    "build_bracketed_array_with_appropriate_whitespace" => "bracket_array",
    "build_message_for_bracketed_array" => "percent_array_message",
    "check_bracketed_array" => "bracket_array", "check_percent_array" => "percent_array_message",
    "comments_in_array?" => "contains_comments",
    "invalid_percent_array_contents?" => "percent_array_context_valid",
    "invalid_percent_array_context?" => "percent_array_context_valid",
    "whitespace_between" => "bracket_array", "whitespace_leading" => "bracket_array",
    "whitespace_trailing" => "bracket_array"
  },
  "lib/rubocop/cop/variable_force/branch.rb" => {
    "==" => "equivalent", "eql?" => "equivalent", "type" => "branch_type"
  },
  "lib/rubocop/cop/correctors/lambda_literal_to_method_corrector.rb" => {
    "arg_to_unparenthesized_call?" => "argument_to_unparenthesized_call",
    "arguments_begin_pos" => "call", "arguments_end_pos" => "call",
    "block_begin" => "call", "block_end" => "call", "initialize" => "call",
    "insert_arguments" => "call", "insert_separating_space" => "call",
    "lambda_arg_string" => "call", "needs_separating_space?" => "call",
    "remove_arguments" => "call", "remove_leading_whitespace" => "call",
    "remove_trailing_whitespace" => "call", "remove_unparenthesized_whitespace" => "call",
    "replace_delimiters" => "call", "replace_selector" => "call",
    "selector_end" => "call", "separating_space?" => "call"
  },
  "lib/rubocop/cop/correctors/percent_literal_corrector.rb" => {
    "autocorrect_multiline_words" => "correction", "autocorrect_words" => "correction",
    "delimiters_for" => "correction", "end_content" => "fix_percent_word",
    "escape_words?" => "percent_word_needs_escaping", "first_line?" => "correction",
    "fix_escaped_content" => "fix_percent_word", "initialize" => "correction",
    "line_breaks" => "correction", "new_contents" => "correction",
    "process_lines" => "correction", "process_multiline_words" => "correction",
    "substitute_escaped_delimiters" => "fix_percent_word", "wrap_contents" => "correction"
  },
  "lib/rubocop/cop/mixin/check_line_breakable.rb" => {
    "breakable_collection?" => "already_on_multiple_lines",
    "chained_to_heredoc?" => "any_heredoc",
    "children_could_be_broken_up?" => "already_on_multiple_lines",
    "contained_by_breakable_collection_on_same_line?" => "all_on_same_line",
    "contained_by_multiline_collection_that_could_be_broken_up?" => "already_on_multiple_lines",
    "extract_breakable_node" => "first_element_line_break_offense",
    "extract_breakable_node_from_elements" => "first_element_line_break_offense",
    "extract_first_element_over_column_limit" => "excessive_range",
    "first_argument_is_heredoc?" => "any_heredoc", "process_args" => "already_on_multiple_lines",
    "safe_to_ignore?" => "already_on_multiple_lines",
    "shift_elements_for_heredoc_arg" => "already_on_multiple_lines"
  },
  "lib/rubocop/cop/mixin/hash_shorthand_syntax.rb" => {
    "brackets?" => "mixed_hash_shorthand", "breakdown_value_types_of_hash" => "mixed_hash_shorthand",
    "def_node_that_require_parentheses" => "hash_value_omittable",
    "each_omittable_value_pair" => "hash_value_omittable",
    "each_omitted_value_pair" => "hash_value_omittable",
    "enforced_shorthand_syntax" => "mixed_hash_shorthand",
    "find_ancestor_method_dispatch_node" => "hash_value_omittable",
    "first_argument" => "hash_value_omittable",
    "hash_with_mixed_shorthand_syntax?" => "mixed_hash_shorthand",
    "hash_with_values_that_cant_be_omitted?" => "hash_value_omittable",
    "ignore_explicit_omissible_hash_shorthand_syntax?" => "hash_value_omittable",
    "ignore_hash_shorthand_syntax?" => "mixed_hash_shorthand",
    "ignore_mixed_hash_shorthand_syntax?" => "mixed_hash_shorthand",
    "last_argument" => "hash_value_omittable", "last_expression?" => "hash_value_omittable",
    "mixed_shorthand_syntax_check" => "mixed_hash_shorthand",
    "no_mixed_shorthand_syntax_check" => "mixed_hash_shorthand",
    "on_hash_for_mixed_shorthand" => "mixed_hash_shorthand",
    "on_pair" => "hash_value_omittable", "register_offense" => "hash_value_omittable",
    "require_hash_value?" => "hash_value_omittable",
    "require_hash_value_for_around_hash_literal?" => "hash_value_omittable",
    "requires_parentheses_context?" => "hash_value_omittable",
    "selector" => "hash_value_omittable",
    "use_element_of_hash_literal_as_receiver?" => "hash_value_omittable",
    "use_modifier_form_without_parenthesized_method_call?" => "hash_value_omittable"
  },
  "lib/rubocop/cop/mixin/hash_subset.rb" => {
    "block_with_first_arg_check?" => "preferred_hash_subset",
    "decorate_source" => "preferred_hash_subset", "except_key" => "preferred_hash_subset",
    "except_key_source" => "preferred_hash_subset",
    "extract_body_if_negated" => "preferred_hash_subset",
    "extract_offense" => "preferred_hash_subset", "extracts_hash_subset?" => "preferred_hash_subset",
    "included?" => "preferred_hash_subset", "not_included?" => "preferred_hash_subset",
    "offense_range" => "preferred_hash_subset", "on_csend" => "preferred_hash_subset",
    "on_send" => "preferred_hash_subset", "preferred_method_name" => "preferred_hash_subset",
    "range_include?" => "within", "safe_to_register_offense?" => "preferred_hash_subset",
    "semantically_except_method?" => "preferred_hash_subset",
    "semantically_slice_method?" => "preferred_hash_subset",
    "semantically_subset_method?" => "preferred_hash_subset",
    "slices_key?" => "preferred_hash_subset", "supported_subset_method?" => "preferred_hash_subset",
    "using_value_variable?" => "preferred_hash_subset"
  },
  "lib/rubocop/cop/mixin/hash_transform_method.rb" => {
    "execute_correction" => "hash_transform_correction",
    "extract_captures" => "transformed_hash_method",
    "handle_possible_offense" => "transformed_hash_method",
    "hash_receiver?" => "transformed_hash_method", "new_method_name" => "transformed_hash_method",
    "noop_transformation?" => "transformed_hash_method",
    "on_bad_each_with_object" => "transformed_hash_method",
    "on_bad_hash_brackets_map" => "transformed_hash_method",
    "on_bad_map_to_h" => "transformed_hash_method", "on_bad_to_h" => "transformed_hash_method",
    "on_block" => "transformed_hash_method", "on_csend" => "transformed_hash_method",
    "on_send" => "transformed_hash_method", "prepare_correction" => "hash_transform_correction",
    "transformation_uses_both_args?" => "transformed_hash_method",
    "use_transformed_argname?" => "transformed_hash_method"
  },
  "lib/rubocop/cop/mixin/statement_modifier.rb" => {
    "code_after" => "modifier_form", "comment_disables_cop?" => "comments_contain_disables",
    "first_line_comment" => "preceding_comment", "if_body_source" => "modifier_form",
    "length_in_modifier_form" => "modifier_fits", "method_source" => "modifier_form",
    "modifier_fits_on_single_line?" => "modifier_fits",
    "non_eligible_body?" => "non_eligible_modifier",
    "non_eligible_condition?" => "non_eligible_modifier",
    "non_eligible_node?" => "non_eligible_modifier",
    "omitted_value_in_last_hash_arg?" => "hash_value_omittable",
    "parenthesize?" => "modifier_form", "single_line_as_modifier?" => "modifier_fits",
    "to_modifier_form" => "modifier_form"
  },
  "lib/rubocop/cop/mixin/trailing_comma.rb" => {
    "allowed_multiline_argument?" => "should_have_trailing_comma_for",
    "autocorrect_range" => "trailing_comma_range", "avoid_comma" => "trailing_comma_range",
    "brackets?" => "trailing_comma_range", "check" => "should_have_trailing_comma_for",
    "check_comma" => "should_have_trailing_comma_for",
    "check_literal" => "should_have_trailing_comma_for", "comma_offset" => "trailing_comma_range",
    "elements" => "should_have_trailing_comma_for",
    "extra_avoid_comma_info" => "should_have_trailing_comma_for", "heredoc?" => "any_heredoc",
    "heredoc_send?" => "any_heredoc", "inside_comment?" => "contains_comments",
    "last_item_precedes_newline?" => "should_have_trailing_comma_for",
    "method_name_and_arguments_on_same_line?" => "all_on_same_line",
    "multiline?" => "all_on_same_line", "no_elements_on_same_line?" => "all_on_same_line",
    "node_end_location" => "trailing_comma_range", "on_same_line?" => "all_on_same_line",
    "put_comma" => "trailing_comma_range", "should_have_comma?" => "should_have_trailing_comma",
    "style_parameter_name" => "should_have_trailing_comma"
  },
  "lib/rubocop/cop/mixin/uncommunicative_name.rb" => {
    "allow_nums" => "name_issues", "allowed_names" => "name_issues", "arg_range" => "name_issues",
    "case_offense" => "name_issues", "check" => "name_issues", "ends_with_num?" => "name_issues",
    "forbidden_names" => "name_issues", "forbidden_offense" => "name_issues",
    "issue_offenses" => "name_issues", "length_offense" => "name_issues",
    "long_enough?" => "name_issues", "min_length" => "name_issues",
    "name_type" => "name_issues", "num_offense" => "name_issues", "uppercase?" => "name_issues"
  },
  "lib/rubocop/cop/registry.rb" => {
    "==" => "equivalent"
  },
  "lib/rubocop/cop/variable_force/variable_table.rb" => {
    "assign_to_variable" => "assign", "current_scope" => "accessible_variables",
    "current_scope_level" => "accessible_variables", "declare_variable" => "declare",
    "find_variable" => "variables", "initialize" => "new", "invoke_hook" => "leave_scope",
    "mark_variable_as_captured_by_block_if_so" => "reference", "pop_scope" => "leave_scope",
    "push_scope" => "enter_scope", "reference_variable" => "reference",
    "scope_stack" => "accessible_variables", "variable_exist?" => "variable_exists"
  },
  "lib/rubocop/cop/base.rb" => {
    "annotate" => "annotate", "apply_correction" => "rewrite",
    "attempt_correction" => "rewrite", "autocorrect_incompatible_with" => "autocorrect",
    "badge" => "cop_name", "callback_argument" => "on_node", "correct" => "rewrite",
    "current_corrector" => "rewrite", "documentation_url" => "url_for",
    "exclude_from_registry" => "exclude", "find_message" => "annotate",
    "inherited" => "new", "initialize" => "new", "inspect" => "cop_name",
    "joining_forces" => "external_dependency_checksum", "match?" => "file_name_matches_any",
    "message" => "annotate", "range_for_original" => "source_range",
    "range_from_node_or_range" => "source_range", "requires_gem" => "target_gem_version",
    "support_autocorrect?" => "support_autocorrect", "support_multiple_source?" => "relevant_file",
    "target_satisfies_all_gem_version_requirements?" => "target_gem_version",
    "use_corrector" => "rewrite"
  },
  "lib/rubocop/cop/cop.rb" => {
    "all" => "cops", "apply_correction" => "rewrite", "call" => "investigate",
    "callback_argument" => "on_node", "correction_lambda" => "transaction",
    "corrections" => "corrections", "dedupe_on_node" => "add_offense",
    "emulate_v0_callsequence" => "investigate", "find_location" => "source_range",
    "inherited" => "new", "joining_forces" => "external_dependency_checksum",
    "qualified_cop_name" => "cop_name", "range_for_original" => "source_range",
    "registry" => "cops", "support_autocorrect?" => "support_autocorrect",
    "suppress_clobbering" => "transaction"
  },
  "lib/rubocop/cop/commissioner.rb" => {
    "build_callbacks" => "with_runtime", "correctors" => "offenses",
    "initialize" => "new", "initialize_callbacks" => "with_runtime",
    "invoke" => "investigate", "invoke_with_argument" => "investigate",
    "reset" => "investigate", "restrict_callbacks" => "with_runtime",
    "restricted_map" => "with_runtime", "trigger_responding_cops" => "investigate",
    "trigger_restricted_cops" => "investigate", "with_cop_error_handling" => "errors"
  },
}.freeze

# Some Ruby modules extend a shared node type whose Rust implementation is
# necessarily split across `impl NodeRef` modules. These are explicit
# source/API/destination triples; unlike the old global-name lookup, a symbol in
# any other file cannot satisfy the audit accidentally.
CROSS_FILE_API_EQUIVALENCES = {
  ["lib/rubocop/ast/node.rb", "receiver"] => ["src/rubocop/ast/node/specialized.rs", "receiver"],
  ["lib/rubocop/ast/node.rb", "send_type?"] => ["src/rubocop/ast/node/specialized.rs", "send_type"],
  ["lib/rubocop/ast/node/block_node.rb", "multiline?"] => ["src/rubocop/ast/node/core.rs", "multiline"],
  ["lib/rubocop/ast/node/block_node.rb", "single_line?"] => ["src/rubocop/ast/node/core.rs", "single_line"],
  ["lib/rubocop/ast/node/defined_node.rb", "node_parts"] => ["src/rubocop/ast/node/core.rs", "node_parts"],
  ["lib/rubocop/ast/node/if_node.rb", "node_parts"] => ["src/rubocop/ast/node/core.rs", "node_parts"],
  ["lib/rubocop/ast/node/keyword_splat_node.rb", "node_parts"] => ["src/rubocop/ast/node/core.rs", "node_parts"],
  ["lib/rubocop/ast/node/mixin/binary_operator_node.rb", "lhs"] => ["src/rubocop/ast/node/specialized.rs", "lhs"],
  ["lib/rubocop/ast/node/mixin/binary_operator_node.rb", "rhs"] => ["src/rubocop/ast/node/specialized.rs", "rhs"],
  ["lib/rubocop/ast/node/mixin/conditional_node.rb", "body"] => ["src/rubocop/ast/node/specialized.rs", "body"],
  ["lib/rubocop/ast/node/mixin/conditional_node.rb", "condition"] => ["src/rubocop/ast/node/specialized.rs", "condition"],
  ["lib/rubocop/ast/node/mixin/method_dispatch_node.rb", "block_node"] => ["src/rubocop/ast/node/specialized.rs", "block_node"],
  ["lib/rubocop/ast/node/mixin/predicate_operator_node.rb", "logical_operator?"] => ["src/rubocop/ast/node/specialized.rs", "logical_operator"],
  ["lib/rubocop/ast/node/mixin/predicate_operator_node.rb", "semantic_operator?"] => ["src/rubocop/ast/node/specialized.rs", "semantic_operator"],
  ["lib/rubocop/ast/node/super_node.rb", "node_parts"] => ["src/rubocop/ast/node/core.rs", "node_parts"],
  ["lib/rubocop/ast/node/yield_node.rb", "node_parts"] => ["src/rubocop/ast/node/core.rs", "node_parts"],
  ["lib/rubocop/cop/mixin/string_help.rb", "on_regexp"] => ["src/rubocop/cop/mixin/helpers.rs", "string_help_on_regexp"],
  ["lib/rubocop/cop/mixin/string_help.rb", "on_str"] => ["src/rubocop/cop/mixin/helpers.rs", "string_help_on_str"],
  ["lib/rubocop/cop/mixin/code_length.rb", "max="] => ["src/rubocop/cop/exclude_limit.rs", "record"],
  ["lib/rubocop/cop/mixin/method_complexity.rb", "max="] => ["src/rubocop/cop/exclude_limit.rs", "record"],
  ["lib/rubocop/cop/cop.rb", "corrections"] => ["src/rubocop/cop/legacy.rs", "corrections"]
}.freeze

# Multiple Ruby APIs may share one Rust target only when the Ruby source itself
# defines them as aliases/value semantics, or when rubocop-ast generates a
# family of identical predicates over one type table. Everything else must
# retain its own translated operation so a broad helper cannot stand in for an
# entire module.
ALLOWED_FOLDED_EQUIVALENCES = {
  ["lib/rubocop/cop/variable_force/branch.rb", "equivalent"] => %w[== eql?],
  ["lib/rubocop/cop/badge.rb", "equivalent"] => %w[== eql?],
  ["lib/rubocop/cop/offense.rb", "equivalent"] => %w[== eql?],
  ["lib/rubocop/ast/node_pattern.rb", "equivalent"] => %w[== eql?],
  ["lib/rubocop/ast/node_pattern.rb", "serialized_pattern"] => %w[as_json encode_with marshal_dump],
  ["lib/rubocop/ast/node_pattern.rb", "from_serialized_pattern"] => %w[init_with marshal_load],
  ["lib/rubocop/ast/node.rb", "type_group_is"] => %w[
    any_block_type? any_def_type? any_match_pattern_type? any_str_type?
    any_sym_type? argument_type? boolean_type? numeric_type? range_type?
  ],
  ["lib/rubocop/ast/node.rb", "value_used"] => %w[
    begin_value_used? case_if_value_used? for_value_used? while_until_value_used?
  ],
  ["lib/rubocop/ast/node.rb", "parent_module_name"] => %w[
    parent_module_name_for_block parent_module_name_for_sclass
  ],
  ["lib/rubocop/cop/mixin/hash_transform_method/autocorrection.rb", "kind"] => %w[match match=]
}.transform_values(&:sort).freeze

# A same-named function in a consolidated Rust file cannot silently satisfy
# several unrelated Ruby classes/modules. Those ambiguous operations require
# an exact source/API ownership declaration in the Rust file.
ambiguous_direct_apis = {}
components.select { |component| component["rust"] }.group_by { |component| component.fetch("rust") }.each do |rust, grouped|
  next if grouped.one?

  functions = rust_function_names(ROOT.join("crates/rustocop", rust))
  owners = Hash.new { |hash, name| hash[name] = [] }
  grouped.each do |component|
    component.fetch("api").each do |name|
      rust_name = rust_api_name(name)
      owners[rust_name] << component.fetch("source") if functions.include?(rust_name)
    end
  end
  owners.each do |name, sources|
    next unless sources.uniq.length > 1

    sources.uniq.each { |source| ambiguous_direct_apis[[rust, source, name]] = true }
  end
end

components.each do |component|
  next unless %w[translated native].include?(component.fetch("status"))

  api = component.fetch("api")
  rust = component["rust"]
  rust_path = rust && ROOT.join("crates/rustocop", rust)
  functions = rust_path ? rust_function_names(rust_path) : []
  ownership = rust_path ? rust_api_ownership(rust_path) : {}
  equivalences = generated_api_equivalences(component.fetch("source"), api)
    .merge(API_EQUIVALENCES.fetch(component.fetch("source"), {}))
  direct = api.select do |name|
    rust_name = rust_api_name(name)
    functions.include?(rust_name) &&
      (!ambiguous_direct_apis[[rust, component.fetch("source"), rust_name]] ||
       ownership.fetch(component.fetch("source"), []).include?(rust_name))
  end
  resolved_targets = {}
  aliased = (api - direct).select do |name|
    target = equivalences[name]
    cross_file = CROSS_FILE_API_EQUIVALENCES[[component.fetch("source"), name]]
    if cross_file
      destination, destination_function = cross_file
      resolved = destination_function == target &&
        rust_function_names(ROOT.join("crates/rustocop", destination)).include?(destination_function)
      resolved_targets[name] = "#{destination}##{destination_function}" if resolved
      resolved
    else
      # An equivalence is only evidence when its target lives in this
      # component's declared Rust translation.
      resolved = target && functions.include?(target)
      resolved_targets[name] = "#{rust}##{target}" if resolved
      resolved
    end
  end
  folded_groups = aliased.group_by { |name| equivalences.fetch(name) }
  invalid_folds = folded_groups.flat_map do |target, names|
    next [] if names.length == 1

    key = [component.fetch("source"), target]
    allowed = ALLOWED_FOLDED_EQUIVALENCES[key]
    allowed == names.sort || GENERATED_FOLD_TARGETS.include?(key) ? [] : names
  end
  aliased -= invalid_folds
  invalid_evidence = component.fetch("evidence", "").match?(
    /(?:still|remains?) in progress|remain(?:s|ing)? unimplemented|not yet (?:implemented|ported)/i
  )
  unresolved = api - direct - aliased
  unresolved |= ["<evidence-declares-incomplete>"] if invalid_evidence
  direct_targets = direct.to_h { |name| [name, "#{rust}##{rust_api_name(name)}"] }
  all_targets = direct_targets.merge(resolved_targets)
  exercise_required = all_targets.select { |_name, target| rust_public_target?(ROOT, target) }
  unexercised_targets = exercise_required.reject do |_name, target|
    rust_target_exercised?(ROOT, target)
  end
  unresolved |= unexercised_targets.keys.map { |name| "<unexercised-rust-target:#{name}>" }
  component["api_coverage"] = {
    "total" => api.length,
    "direct" => direct.length,
    "direct_targets" => direct_targets,
    "ownership_declared" => direct.select do |name|
      ambiguous_direct_apis[[rust, component.fetch("source"), rust_api_name(name)]]
    end,
    "equivalent" => aliased.length,
    "equivalence_targets" => resolved_targets.sort.to_h,
    "exercise_required" => exercise_required.sort.to_h,
    "unexercised_targets" => unexercised_targets.sort.to_h,
    "unresolved" => unresolved
  }
  component["status"] = "partial" unless unresolved.empty?
end

spec_rust_paths = components.flat_map { |component| component.fetch("specs") }
  .group_by { |spec| [spec.fetch("package"), spec.fetch("source")] }
  .transform_values { |specs| specs.map { |spec| spec.fetch("rust") }.uniq }

components.each do |component|
  component.fetch("specs").each do |spec|
    upstream_path = ROOT.join("spec/upstream/#{spec.fetch("package")}-#{PACKAGES.fetch(spec.fetch("package"))}", spec.fetch("source"))
    expanded_examples = EXPANDED_EXAMPLES.fetch([spec.fetch("package"), spec.fetch("source")], [])
    upstream_examples = expanded_examples.size
    rust_paths = spec_rust_paths.fetch([spec.fetch("package"), spec.fetch("source")])
    rust_sources = rust_paths.to_h do |rust|
      [rust, ROOT.join("crates/rustocop", rust).read]
    end
    contract_tests = rust_sources.filter_map do |rust, source|
      tests = rust_test_names(source)
      { "rust" => rust, "tests" => tests } unless tests.empty?
    end
    candidates = contract_tests.flat_map do |contract|
      contract.fetch("tests").map do |test|
        { "rust" => contract.fetch("rust"), "test" => test, "tokens" => contract_tokens(test) }
      end
    end
    example_contracts = expanded_examples.map { |example| example_contract(example, candidates) }
    selected_tests = example_contracts.group_by { |contract| contract.fetch("rust") }
      .sort.to_h do |rust, contracts|
        [rust, contracts.map { |contract| contract.fetch("test") }.uniq.sort]
      end
    contract_tests = selected_tests.map { |rust, tests| { "rust" => rust, "tests" => tests } }
    rust_tests = contract_tests.sum { |contract| contract.fetch("tests").length }
    pinned_source = rust_sources.values.any? do |source|
      source.include?(spec.fetch("source")) &&
        source.include?("Spec SHA-256: #{spec.fetch("source_sha256")}")
    end
    covered_examples = pinned_source && rust_tests.positive? ? upstream_examples : 0
    contract_payload = {
      "package" => spec.fetch("package"),
      "source" => spec.fetch("source"),
      "source_sha256" => spec.fetch("source_sha256"),
      "example_contracts" => example_contracts,
      "rust_tests" => contract_tests
    }
    spec["upstream_examples"] = upstream_examples
    spec["rust_tests"] = rust_tests
    spec["covered_upstream_examples"] = covered_examples
    spec["coverage_inventory"] = EXAMPLE_INVENTORY.relative_path_from(ROOT).to_s
    spec["coverage_rust_files"] = rust_paths
    spec["contract_tests"] = contract_tests
    spec["example_contracts"] = example_contracts
    spec["contract_sha256"] = Digest::SHA256.hexdigest(JSON.generate(contract_payload))
    spec["status"] = if %w[translated native].include?(component.fetch("status")) &&
                        upstream_examples.positive? && covered_examples == upstream_examples && rust_tests.positive?
                       "translated"
                     else
                       "partial"
                     end
  end
end

unless UNMAPPED_CONTRACT_EXAMPLES.empty?
  abort "no semantic Rust contract for:\n#{UNMAPPED_CONTRACT_EXAMPLES.map { |source, id| "  #{source} #{id}" }.join("\n")}"
end

manifest = {
  "format_version" => 5,
  "updated_at" => Time.now.utc.iso8601,
  "rubocop_version" => PACKAGES.fetch("rubocop"),
  "rubocop_ast_version" => PACKAGES.fetch("rubocop-ast"),
  "scope" => {
    "rubocop" => [
      "lib/rubocop/cop/*.rb",
      "lib/rubocop/cop/mixin/**/*.rb",
      "lib/rubocop/cop/correctors/**/*.rb",
      "lib/rubocop/cop/legacy/**/*.rb",
      "lib/rubocop/cop/variable_force/**/*.rb"
    ],
    "rubocop-ast" => ["lib/rubocop/ast/**/*"],
    "expanded_example_inventory" => EXAMPLE_INVENTORY.relative_path_from(ROOT).to_s,
    "expanded_example_inventory_sha256" => Digest::SHA256.file(EXAMPLE_INVENTORY).hexdigest
  },
  "components" => components
}

MANIFEST.write("#{JSON.pretty_generate(manifest)}\n")
counts = components.group_by { |component| component.fetch("status") }.transform_values(&:length)
by_kind = components.group_by { |component| component.fetch("kind") }.transform_values(&:length)
complete = components.count { |component| %w[translated native not_applicable].include?(component.fetch("status")) }
started = components.count { |component| component.fetch("status") == "partial" }
percent = components.empty? ? 100.0 : (complete.fdiv(components.length) * 100).round(1)
pending = components.select { |component| component.fetch("status") == "pending" }
audited_api_components = components.select { |component| component.key?("api_coverage") }
total_apis = audited_api_components.sum { |component| component.dig("api_coverage", "total") }
resolved_apis = audited_api_components.sum do |component|
  component.dig("api_coverage", "direct") + component.dig("api_coverage", "equivalent")
end
unexercised_targets = audited_api_components.sum do |component|
  component.dig("api_coverage", "unexercised_targets").length
end
specs = components.flat_map { |component| component.fetch("specs") }
unique_specs = specs.uniq { |spec| [spec.fetch("package"), spec.fetch("source")] }
spec_counts = unique_specs.group_by { |spec| spec.fetch("status") }.transform_values(&:length)
upstream_examples = unique_specs.sum { |spec| spec.fetch("upstream_examples") }
rust_test_files = unique_specs.flat_map { |spec| spec.fetch("coverage_rust_files") }.uniq
rust_tests = rust_test_files.sum do |rust|
  ROOT.join("crates/rustocop", rust).read.scan(/^\s*#\[test\]/).size
end
rubocop_spec_root = ROOT.join("spec/upstream/rubocop-#{PACKAGES.fetch("rubocop")}/spec/rubocop/cop")
discovered_rubocop_specs = [
  *rubocop_spec_root.glob("*_spec.rb"),
  *rubocop_spec_root.join("mixin").glob("**/*_spec.rb")
].map { |path| path.relative_path_from(ROOT.join("spec/upstream/rubocop-#{PACKAGES.fetch("rubocop")}" )).to_s }.uniq.sort
registered_rubocop_specs = unique_specs.select { |spec| spec.fetch("package") == "rubocop" }.map { |spec| spec.fetch("source") }.sort
unregistered_rubocop_specs = discovered_rubocop_specs - registered_rubocop_specs
rubocop_ast_spec_root = ROOT.join("spec/upstream/rubocop-ast-#{PACKAGES.fetch("rubocop-ast")}/spec/rubocop/ast")
discovered_ast_specs = rubocop_ast_spec_root.glob("**/*_spec.rb").map { |path| path.relative_path_from(ROOT.join("spec/upstream/rubocop-ast-#{PACKAGES.fetch("rubocop-ast")}" )).to_s }.sort
registered_ast_specs = unique_specs.select { |spec| spec.fetch("package") == "rubocop-ast" }.map { |spec| spec.fetch("source") }.sort
unregistered_ast_specs = discovered_ast_specs - registered_ast_specs
report = <<~MARKDOWN
  # RuboCop compatibility implementation progress

  Updated at: `#{manifest.fetch("updated_at")}`

  Target: RuboCop #{PACKAGES.fetch("rubocop")} and rubocop-ast #{PACKAGES.fetch("rubocop-ast")}.
  Existing cops are not consumers of this layer yet.

  ## Progress

  - Accounted components: #{complete}/#{components.length} (#{percent}%)
  - Partially implemented: #{started}
  - Translated: #{counts.fetch("translated", 0)}
  - Native equivalent: #{counts.fetch("native", 0)}
  - Not applicable in Rust: #{counts.fetch("not_applicable", 0)}
  - Pending: #{counts.fetch("pending", 0)}
  - Resolved syntax- and runtime-discovered APIs: #{resolved_apis}/#{total_apis} (#{total_apis.zero? ? 100.0 : (resolved_apis.fdiv(total_apis) * 100).round(1)}%)
  - Unexercised public Rust API targets: #{unexercised_targets}

  Registered upstream spec ports are tracked independently from component code:

  - Fully ported spec files: #{spec_counts.fetch("translated", 0)}/#{unique_specs.length}
  - Partially ported spec files: #{spec_counts.fetch("partial", 0)}
  - Upstream examples in registered files: #{upstream_examples}
  - Focused Rust test functions for those files: #{rust_tests}
  - Discovered RuboCop shared spec files: #{discovered_rubocop_specs.length}
  - Registered RuboCop shared spec files: #{registered_rubocop_specs.length}
  - Unregistered RuboCop shared spec files: #{unregistered_rubocop_specs.length}
  - Discovered rubocop-ast spec files: #{discovered_ast_specs.length}
  - Registered rubocop-ast spec files: #{registered_ast_specs.length}
  - Unregistered rubocop-ast spec files: #{unregistered_ast_specs.length}

  | Surface | Components |
  | --- | ---: |
  #{by_kind.sort.map { |kind, count| "| `#{kind}` | #{count} |" }.join("\n")}

  ## Pending components

  #{pending.empty? ? "None." : pending.map { |component| "- `#{component.fetch("package")}:#{component.fetch("source")}`" }.join("\n")}

  ## Unregistered rubocop-ast specs

  #{unregistered_ast_specs.empty? ? "None." : unregistered_ast_specs.map { |source| "- `#{source}`" }.join("\n")}

  ## Unregistered RuboCop shared specs

  #{unregistered_rubocop_specs.empty? ? "None." : unregistered_rubocop_specs.map { |source| "- `#{source}`" }.join("\n")}
MARKDOWN
REPORT.write(report)
puts "Wrote #{components.length} components to #{MANIFEST}"
puts "Wrote progress report to #{REPORT}"
puts counts.sort.map { |status, count| "#{status}=#{count}" }.join(" ")
