# frozen_string_literal: true

module Rustocop
  module ProjectMismatchInventory
    FORMAT_VERSION = 1
    SIGNATURE_FIELDS = %w[
      path cop severity message start_line start_column last_line last_column
    ].freeze
    ENTRY_FIELDS = ["direction", *SIGNATURE_FIELDS, "count"].freeze
    EXAMPLE_LIMIT = 3

    Comparison = Data.define(:by_cop, :entries)

    module_function

    def compare(rustocop_offenses, rubocop_offenses, cops)
      rust_by_cop = rustocop_offenses.group_by { |offense| offense.fetch("cop") }
      ruby_by_cop = rubocop_offenses.group_by { |offense| offense.fetch("cop") }
      entries = []
      by_cop = cops.to_h do |cop|
        rust_rows = rust_by_cop.fetch(cop, [])
        ruby_rows = ruby_by_cop.fetch(cop, [])
        rust_tally = signature_tally(rust_rows)
        ruby_tally = signature_tally(ruby_rows)
        rust_only = unmatched(rust_tally, ruby_tally)
        ruby_only = unmatched(ruby_tally, rust_tally)
        entries.concat(inventory_entries("rustocop_only", rust_only))
        entries.concat(inventory_entries("rubocop_only", ruby_only))
        exact = rust_tally.sum { |signature, count| [count, ruby_tally.fetch(signature, 0)].min }
        [cop, {
          "rustocop" => rust_rows.length,
          "rubocop" => ruby_rows.length,
          "exact" => exact,
          "rustocop_only_examples" => examples(rust_only),
          "rubocop_only_examples" => examples(ruby_only)
        }]
      end
      Comparison.new(by_cop:, entries: entries.sort)
    end

    def entry_hash(entry)
      ENTRY_FIELDS.zip(entry).to_h
    end

    def signature_tally(rows)
      rows.map { |offense| SIGNATURE_FIELDS.map { |field| offense.fetch(field) } }.tally
    end
    private_class_method :signature_tally

    def unmatched(left, right)
      left.filter_map do |signature, count|
        missing = count - right.fetch(signature, 0)
        [signature, missing] if missing.positive?
      end
    end
    private_class_method :unmatched

    def examples(unmatched)
      unmatched.each_with_object([]) do |(signature, count), result|
        [count, EXAMPLE_LIMIT - result.length].min.times { result << signature }
        break result if result.length == EXAMPLE_LIMIT
      end
    end
    private_class_method :examples

    def inventory_entries(direction, unmatched)
      unmatched.map { |signature, count| [direction, *signature, count] }
    end
    private_class_method :inventory_entries
  end
end
