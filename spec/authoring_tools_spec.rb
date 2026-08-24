# frozen_string_literal: true

RSpec.describe "cop authoring tools" do
  def run_script(name, *arguments, env: {})
    Open3.capture3(env, RbConfig.ruby, File.join(ROOT, "script", name), *arguments)
  end

  it "previews any-node scaffolding and the cached unit-contract destination" do
    stdout, stderr, status = run_script(
      "new_cop.rb",
      "Style/GeneratedExample",
      "any_node",
      "--dry-run"
    )

    expect(status).to be_success
    expect(stderr).to eq("")
    expect(stdout).to include(
      'GeneratedExample => "Style/GeneratedExample" => any_node(check)',
      "fn check(node: &Node<'_>",
      "spec/fixtures/cops/Style/GeneratedExample/unit/cases.jsonl",
      "generate_unit_fixtures.rb"
    )
    expect(File).not_to exist(File.join(ROOT, "crates/rustocop/src/cops/prism/style_generated_example.rs"))
  end

  it "rejects an empty upstream contract" do
    native = File.join(ROOT, "libexec", "rustocop-native")
    _stdout, stderr, status = run_script(
      "compare_upstream_cop_specs.rb",
      "--only",
      "Style/DefinitelyNotACop",
      env: { "RUSTOCOP_NATIVE_PATH" => native }
    )

    expect(status).not_to be_success
    expect(stderr).to include("no captured upstream cases matched the requested cops")
  end

  it "previews appending a cop to a capability family" do
    stdout, stderr, status = run_script(
      "new_cop.rb",
      "Style/GeneratedFamilyExample",
      "call",
      "--family",
      "style_calls",
      "--dry-run"
    )

    expect(status).to be_success
    expect(stderr).to eq("")
    expect(stdout).to include(
      "crates/rustocop/src/cops/prism/style_calls.rs",
      "Append to the existing define_cops! block",
      'GeneratedFamilyExample => "Style/GeneratedFamilyExample" => call(generated_family_example)',
      "fn generated_family_example(node: &CallNode<'_>",
      "spec/fixtures/cops/Style/GeneratedFamilyExample/unit/cases.jsonl"
    )
  end

  it "previews a Ruby-shaped cop with RuboCop callback names" do
    stdout, stderr, status = run_script(
      "new_cop.rb",
      "Style/GeneratedRubyShape",
      "rubocop",
      "--callbacks",
      "on_block,on_while,on_until",
      "--dry-run"
    )

    expect(status).to be_success
    expect(stderr).to eq("")
    expect(stdout).to include(
      'GeneratedRubyShape => "Style/GeneratedRubyShape" => rubocop_callbacks(',
      "GeneratedRubyShapeRule, [on_block, on_while, on_until]",
      "impl GeneratedRubyShapeRule<'_, '_, '_>",
      "fn on_block(&mut self, node: &ruby_prism::BlockNode<'_>)",
      "fn on_while(&mut self, node: &ruby_prism::WhileNode<'_>)",
      "fn on_until(&mut self, node: &ruby_prism::UntilNode<'_>)"
    )
  end

  it "previews a reverse-order real-project parity audit" do
    stdout, stderr, status = run_script(
      "audit_project_parity.rb",
      "--from-position",
      "391",
      "--count",
      "2",
      "--dry-run"
    )

    expect(status).to be_success
    expect(stderr).to eq("")
    expect(stdout.lines.map(&:chomp)).to eq([
      "391\tStyle/FileRead",
      "390\tStyle/FileOpen"
    ])
  end
end
