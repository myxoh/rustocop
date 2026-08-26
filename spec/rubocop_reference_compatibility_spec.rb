# frozen_string_literal: true

require_relative "../lib/rustocop/rubocop_reference_compatibility"

RSpec.describe Rustocop::RubocopReferenceCompatibility do
  it "allows FileWrite to inspect a missing heredoc node" do
    cop = RuboCop::Cop::Style::FileWrite.allocate

    expect(cop.send(:find_heredoc, nil)).to be_nil
  end

  it "handles a non-node ClassAndModuleChildren sibling" do
    namespace = Object.new
    identifier = Struct.new(:namespace).new(namespace)
    location = Struct.new(:keyword).new(:keyword_location)
    node = Struct.new(:left_sibling, :identifier, :loc).new(:begin, identifier, location)
    corrector = instance_double(Parser::Source::TreeRewriter)
    allow(corrector).to receive(:replace)
    cop = RuboCop::Cop::Style::ClassAndModuleChildren.allocate

    cop.send(:replace_namespace_keyword, corrector, node)

    expect(corrector).to have_received(:replace).with(:keyword_location, "module")
  end

  it "permits selected reference collection for RedundantCopDisableDirective" do
    validator = RuboCop::OptionsValidator.allocate

    expect(validator.send(:only_includes_redundant_disable?)).to be(false)
  end
end
