# frozen_string_literal: true

require "yaml"
require_relative "../lib/rustocop/config_serialization"

RSpec.describe Rustocop::ConfigSerialization do
  it "restores Ruby symbol semantics only for inverse-method maps" do
    rendered = described_class.rubocop_yaml(
      "Style/Example" => {
        "InverseMethods" => { "include?" => "exclude?" },
        "PreferredMethods" => { "intern" => "to_sym" }
      }
    )
    loaded = YAML.unsafe_load(rendered)

    expect(loaded.dig("Style/Example", "InverseMethods")).to eq(include?: :exclude?)
    expect(loaded.dig("Style/Example", "PreferredMethods")).to eq("intern" => "to_sym")
  end
end
