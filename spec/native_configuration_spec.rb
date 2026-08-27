# frozen_string_literal: true

RSpec.describe Rustocop::NativeConfiguration do
  it "selects a discovered compiled configuration without loading RuboCop" do
    Dir.mktmpdir("rustocop-native-config") do |directory|
      config = File.join(directory, ".rustocop.yml")
      target = File.join(directory, "example.rb")
      File.write(config, "Rustocop:\n  SchemaVersion: 1\n")
      File.write(target, "puts :ok\n")

      arguments = described_class.arguments(["--format", "json", target])

      expect(arguments).to eq(["--format", "json", target, "--config=#{config}"])
    end
  end

  it "falls back to RuboCop resolution for non-native cops" do
    Dir.mktmpdir("rustocop-native-config") do |directory|
      File.write(File.join(directory, ".rustocop.yml"), "Rustocop:\n  SchemaVersion: 1\n")
      target = File.join(directory, "example.rb")
      File.write(target, "puts :ok\n")

      expect(described_class.arguments(["--included-non-native-cops", target])).to be_nil
    end
  end

  it "recognizes the conventional explicit nested native config" do
    Dir.mktmpdir("rustocop-native-config") do |directory|
      path = File.join(directory, ".config", "rustocop", "config.yml")
      FileUtils.mkdir_p(File.dirname(path))
      File.write(path, "Rustocop:\n  SchemaVersion: 1\n")

      arguments = described_class.arguments(["--config", path, "example.rb"])

      expect(arguments).to eq(["--config", path, "example.rb"])
    end
  end
end
