# frozen_string_literal: true

require "fileutils"

ROOT = File.expand_path("..", __dir__)
DESTINATION = File.join(ROOT, "spec", "fixtures", "rubocop_builtin_examples")

CaseSet = Struct.new(:directory, :cop, :offense, :clean)

cases = [
  CaseSet.new("lint_boolean_symbol", "Lint/BooleanSymbol",
    ->(i) { i.odd? ? ":true\n" : ":false\n" },
    ->(i) { ":boolean_#{i}\n" }),
  CaseSet.new("lint_empty_expression", "Lint/EmptyExpression",
    ->(i) { "value_#{i} = ()\n" },
    ->(i) { "value_#{i} = (#{i})\n" }),
  CaseSet.new("lint_float_comparison", "Lint/FloatComparison",
    ->(i) { "value_#{i} == #{i}.5\n" },
    ->(i) { "value_#{i} == 0.0\n" }),
  CaseSet.new("lint_self_assignment", "Lint/SelfAssignment",
    ->(i) { "value_#{i} = value_#{i}\n" },
    ->(i) { "value_#{i} = other_#{i}\n" }),
  CaseSet.new("security_compound_hash", "Security/CompoundHash",
    ->(i) { "def hash\n  @left_#{i} ^ @right_#{i}\nend\n" },
    ->(i) { "def hash\n  [@left_#{i}, @right_#{i}].hash\nend\n" }),
  CaseSet.new("security_eval", "Security/Eval",
    ->(i) { "eval(code_#{i})\n" },
    ->(i) { "eval(\"literal_#{i}\")\n" }),
  CaseSet.new("security_json_load", "Security/JSONLoad",
    ->(i) { "JSON.#{i.odd? ? "load" : "restore"}(payload_#{i})\n" },
    ->(i) { "JSON.parse(payload_#{i})\n" }),
  CaseSet.new("security_marshal_load", "Security/MarshalLoad",
    ->(i) { "Marshal.#{i.odd? ? "load" : "restore"}(payload_#{i})\n" },
    ->(i) { "Marshal.load(Marshal.dump(value_#{i}))\n" }),
  CaseSet.new("security_open", "Security/Open",
    ->(i) { "open(path_#{i})\n" },
    ->(i) { "open(\"file_#{i}.txt\")\n" }),
  CaseSet.new("security_io_methods", "Security/IoMethods",
    ->(i) { "IO.#{i.odd? ? "read" : "write"}(path_#{i})\n" },
    ->(i) { "File.#{i.odd? ? "read" : "write"}(path_#{i})\n" }),
  CaseSet.new("style_character_literal", "Style/CharacterLiteral",
    ->(i) { "?#{(96 + i).chr}\n" },
    ->(i) { "'#{(96 + i).chr}'\n" }),
  CaseSet.new("style_def_with_parentheses", "Style/DefWithParentheses",
    ->(i) { "def value_#{i}()\n  #{i}\nend\n" },
    ->(i) { "def value_#{i}\n  #{i}\nend\n" }),
  CaseSet.new("style_method_call_without_args_parentheses", "Style/MethodCallWithoutArgsParentheses",
    ->(i) { "action_#{i}()\n" },
    ->(i) { "action_#{i}\n" }),
  CaseSet.new("style_nil_comparison", "Style/NilComparison",
    ->(i) { "value_#{i} == nil\n" },
    ->(i) { "value_#{i}.nil?\n" }),
  CaseSet.new("style_not", "Style/Not",
    ->(i) { "not value_#{i}\n" },
    ->(i) { "!value_#{i}\n" }),
  CaseSet.new("style_redundant_array_constructor", "Style/RedundantArrayConstructor",
    ->(i) { "Array([#{i}])\n" },
    ->(i) { "[#{i}]\n" }),
  CaseSet.new("style_redundant_freeze", "Style/RedundantFreeze",
    ->(i) { "#{i}.freeze\n" },
    ->(i) { "object_#{i}.freeze\n" }),
  CaseSet.new("style_semicolon", "Style/Semicolon",
    ->(i) { "first_#{i}; second_#{i}\n" },
    ->(i) { "first_#{i}\nsecond_#{i}\n" }),
  CaseSet.new("style_string_chars", "Style/StringChars",
    ->(i) { "value_#{i}.split(#{i.odd? ? %q("") : %q('')})\n" },
    ->(i) { "value_#{i}.chars\n" }),
  CaseSet.new("style_unless_else", "Style/UnlessElse",
    ->(i) { "unless ready_#{i}\n  work_#{i}\nelse\n  wait_#{i}\nend\n" },
    ->(i) { "if ready_#{i}\n  wait_#{i}\nelse\n  work_#{i}\nend\n" })
].freeze

FileUtils.mkdir_p(DESTINATION)

cases.each do |case_set|
  directory = File.join(DESTINATION, case_set.directory)
  FileUtils.mkdir_p(directory)

  1.upto(25) do |number|
    kind = number <= 12 ? "offense" : "clean"
    source = number <= 12 ? case_set.offense.call(number) : case_set.clean.call(number)
    File.write(File.join(directory, format("%02d_%s.rb", number, kind)), source)
  end
end

manifest = cases.map { |case_set| "#{case_set.directory}\t#{case_set.cop}" }.join("\n")
File.write(File.join(DESTINATION, "manifest.tsv"), "directory\tcop\n#{manifest}\n")

puts "generated #{cases.length * 25} examples across #{cases.length} cops"
