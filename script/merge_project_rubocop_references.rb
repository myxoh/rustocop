#!/usr/bin/env ruby
# frozen_string_literal: true

require "optparse"
require_relative "../lib/rustocop/artifact_store"
require_relative "../lib/rustocop/project_reference_merge"

output = nil
OptionParser.new do |parser|
  parser.banner = "Usage: ruby script/merge_project_rubocop_references.rb --output PATH REFERENCE..."
  parser.on("--output PATH") { |path| output = File.expand_path(path) }
end.parse!

abort "--output is required" unless output
abort "at least two reference paths are required" if ARGV.length < 2

references = ARGV.map do |path|
  Rustocop::ArtifactStore.read_gzip_json(File.expand_path(path), label: "RuboCop reference")
end
merged = Rustocop::ProjectReferenceMerge.merge(references)
Rustocop::ArtifactStore.write_gzip_json(output, merged)
puts "Merged #{references.length} references into #{output} (#{merged.fetch('cops').length} cops)"
