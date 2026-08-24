# frozen_string_literal: true

require "digest"

module Rustocop
  module SourceFingerprint
    module_function

    def cops(root: RepositoryLayout.default.root)
      source_root = File.join(root, "crates", "rustocop", "src", "cops")
      digest = Digest::SHA256.new
      Dir.glob(File.join(source_root, "**", "*.rs")).sort.each do |path|
        relative = path.delete_prefix("#{root}/")
        content = File.binread(path)
        digest << relative << "\0" << content.bytesize.to_s << "\0" << content
      end
      digest.hexdigest
    end
  end
end
