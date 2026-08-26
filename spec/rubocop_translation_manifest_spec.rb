# frozen_string_literal: true

require "digest"
require "json"
require "time"

RSpec.describe "RuboCop translation manifest" do
  root = Pathname(__dir__).join("..").expand_path
  manifest_path = root.join("crates/rustocop/rubocop-translation.json")
  manifest = JSON.parse(manifest_path.read)
  versions = {
    "rubocop" => manifest.fetch("rubocop_version"),
    "rubocop-ast" => manifest.fetch("rubocop_ast_version")
  }.freeze

  def expect_matching_sha256(path, recorded_sha256)
    expect(path).to exist, "missing upstream source: #{path}"
    expect(Digest::SHA256.file(path).hexdigest).to eq(recorded_sha256)
  end

  def expect_rust_function(root, target)
    rust, function = target.split("#", 2)
    source = root.join("crates/rustocop", rust).read
    generated_callback = rust.end_with?("/traversal.rs") && source.match?(/=>\s*#{Regexp.escape(function)}\b/)
    expect(source.match?(/\bfn\s+#{Regexp.escape(function)}\b/) || generated_callback).to be(true), target
  end

  it "pins every component and registered spec to the fixed upstream versions" do
    manifest.fetch("components").each do |translation|
      package = translation.fetch("package")
      source = translation.fetch("source")
      gem_root = Pathname(Gem::Specification.find_by_name(package, versions.fetch(package)).full_gem_path)
      expect_matching_sha256(gem_root.join(source), translation.fetch("source_sha256"))

      translation.fetch("specs").each do |spec|
        spec_package = spec.fetch("package")
        spec_source = spec.fetch("source")
        upstream_root = root.join("spec/upstream/#{spec_package}-#{versions.fetch(spec_package)}")
        expect_matching_sha256(upstream_root.join(spec_source), spec.fetch("source_sha256"))
      end
    end
  end

  it "registers every pinned shared RuboCop and rubocop-ast spec" do
    registered = manifest.fetch("components").flat_map do |translation|
      translation.fetch("specs").map { |spec| [spec.fetch("package"), spec.fetch("source")] }
    end.uniq

    rubocop_root = root.join("spec/upstream/rubocop-#{versions.fetch("rubocop")}")
    rubocop_cop_specs = rubocop_root.join("spec/rubocop/cop").then do |cop_root|
      [*cop_root.glob("*_spec.rb"), *cop_root.join("mixin").glob("**/*_spec.rb")]
    end
    rubocop_ast_root = root.join("spec/upstream/rubocop-ast-#{versions.fetch("rubocop-ast")}")
    rubocop_ast_specs = rubocop_ast_root.join("spec/rubocop/ast").glob("**/*_spec.rb")

    discovered = rubocop_cop_specs.map do |path|
      ["rubocop", path.relative_path_from(rubocop_root).to_s]
    end + rubocop_ast_specs.map do |path|
      ["rubocop-ast", path.relative_path_from(rubocop_ast_root).to_s]
    end

    expect(registered.sort).to eq(discovered.uniq.sort)
  end

  it "keeps the pinned compatibility inventory complete" do
    expect(manifest.fetch("format_version")).to eq(5)
    expect { Time.iso8601(manifest.fetch("updated_at")) }.not_to raise_error
    expect(manifest.fetch("components").length).to eq(228)
    expect(manifest.fetch("components").map { |component| component.fetch("status") }.uniq)
      .to contain_exactly("translated", "native", "not_applicable")
    translated_api_total = manifest.fetch("components").sum do |component|
      component.dig("api_coverage", "total").to_i
    end
    expect(translated_api_total).to eq(2_586)

    generated_api_samples = {
      "lib/rubocop/cop/base.rb" => %w[config gem_requirements project_index=],
      "lib/rubocop/cop/commissioner.rb" => %w[cop_reports errors on_send on_array_pattern],
      "lib/rubocop/cop/variable_force.rb" => %w[before_entering_scope after_declaring_variable],
      "lib/rubocop/ast/node.rb" => %w[array_type? match_pattern_type? zsuper_type?],
      "lib/rubocop/ast/node_pattern.rb" => %w[ast captures match_code pattern]
    }
    generated_api_samples.each do |source, expected_api|
      component = manifest.fetch("components").find { |entry| entry.fetch("source") == source }
      expect(component.fetch("api")).to include(*expected_api), source
    end
    expect(
      manifest.fetch("components").flat_map { |component| component.fetch("specs") }
        .map { |spec| spec.fetch("status") }.uniq
    ).to eq(["translated"])

    example_inventory_path = root.join(manifest.dig("scope", "expanded_example_inventory"))
    expect(Digest::SHA256.file(example_inventory_path).hexdigest)
      .to eq(manifest.dig("scope", "expanded_example_inventory_sha256"))
    example_inventory = JSON.parse(example_inventory_path.read)
    expect(example_inventory.fetch("versions")).to eq(versions)
    expect(example_inventory.fetch("example_count")).to eq(3_139)
    expect(example_inventory.fetch("examples").length).to eq(3_139)

    specs = manifest.fetch("components").flat_map { |component| component.fetch("specs") }
    specs.uniq { |spec| [spec.fetch("package"), spec.fetch("source")] }.each do |spec|
      expect(spec.fetch("upstream_examples")).to be_positive
      expect(spec.fetch("covered_upstream_examples")).to eq(spec.fetch("upstream_examples"))
      expect(spec.fetch("coverage_inventory")).to eq(
        manifest.dig("scope", "expanded_example_inventory")
      )
      expect(spec.fetch("coverage_rust_files")).to include(spec.fetch("rust"))
      contract_tests = spec.fetch("contract_tests")
      expect(contract_tests).not_to be_empty
      expect(contract_tests.sum { |contract| contract.fetch("tests").length })
        .to eq(spec.fetch("rust_tests"))
      contract_tests.each do |contract|
        rust_source = root.join("crates/rustocop", contract.fetch("rust")).read
        contract.fetch("tests").each do |test|
          expect(rust_source).to match(/\bfn\s+#{Regexp.escape(test)}\b/)
        end
      end

      examples = example_inventory.fetch("examples").select do |example|
        example.fetch("package") == spec.fetch("package") &&
          example.fetch("source") == spec.fetch("source")
      end
      example_contracts = spec.fetch("example_contracts")
      expect(example_contracts.length).to eq(examples.length)
      expect(example_contracts.map { |contract| contract.fetch("rspec_id") })
        .to eq(examples.map { |example| example.fetch("rspec_id") })
      example_contracts.zip(examples).each do |contract, example|
        expect(contract.fetch("description_sha256"))
          .to eq(Digest::SHA256.hexdigest(example.fetch("full_description")))
        expect(%w[semantic_terms explicit_source_rule]).to include(contract.fetch("mapping_basis"))
        if contract.fetch("mapping_basis") == "semantic_terms"
          expect(contract.fetch("matched_terms")).not_to be_empty
        end
        rust_source = root.join("crates/rustocop", contract.fetch("rust")).read
        expect(rust_source).to match(/\bfn\s+#{Regexp.escape(contract.fetch('test'))}\b/)
      end
      derived_contract_tests = example_contracts.group_by { |contract| contract.fetch("rust") }
        .sort.to_h do |rust, contracts|
          [rust, contracts.map { |contract| contract.fetch("test") }.uniq.sort]
        end
        .map { |rust, tests| { "rust" => rust, "tests" => tests } }
      expect(contract_tests).to eq(derived_contract_tests)
      contract_payload = {
        "package" => spec.fetch("package"),
        "source" => spec.fetch("source"),
        "source_sha256" => spec.fetch("source_sha256"),
        "example_contracts" => example_contracts,
        "rust_tests" => contract_tests
      }
      expect(spec.fetch("contract_sha256")).to eq(Digest::SHA256.hexdigest(JSON.generate(contract_payload)))
    end

    manifest.fetch("components").select do |component|
      %w[translated native].include?(component.fetch("status"))
    end
      .each do |component|
        coverage = component.fetch("api_coverage")
        expect(coverage.fetch("unresolved")).to be_empty, component.fetch("source")
        expect(coverage.fetch("unexercised_targets")).to be_empty, component.fetch("source")
        expect(coverage.fetch("direct") + coverage.fetch("equivalent"))
          .to eq(coverage.fetch("total")), component.fetch("source")
        expect(coverage.fetch("direct_targets").length).to eq(coverage.fetch("direct"))
        coverage.fetch("direct_targets").each_value do |target|
          expect_rust_function(root, target)
        end
        coverage.fetch("ownership_declared").each do |name|
          marker = "// RuboCop API ownership: #{component.fetch('source')} =>"
          rust_source = root.join("crates/rustocop", component.fetch("rust")).read
          declaration = rust_source.lines.find { |line| line.start_with?(marker) }
          expect(declaration).not_to be_nil, "#{component.fetch('source')}##{name}"
          owned_names = declaration.split("=>", 2).last.split(",").map(&:strip)
          expect(owned_names).to include(name.sub(/[?!=]\z/, ""))
        end
        coverage.fetch("equivalence_targets").each_value do |target|
          expect_rust_function(root, target)
        end
      end
  end
end
