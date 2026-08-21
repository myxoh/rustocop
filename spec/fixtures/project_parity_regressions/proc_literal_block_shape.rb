literal = Proc.new { |value| value }
numbered = ::Proc.new { _1 }

symbol_proc = Proc.new(&:to_s)
bare = Proc.new

[literal, numbered, symbol_proc, bare]
