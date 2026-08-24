# rubocop:disable Style/TrailingCommaInArrayLiteral -- let the last element have a comma for simpler diffs
values = [
  :one,
  :two,
]
# rubocop:enable Style/TrailingCommaInArrayLiteral

# rubocop: disable Style/Semicolon
expect { iterations << i; raise }
# rubocop: enable Style/Semicolon
