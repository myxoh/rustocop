compact = ->() { "compact" }
spaced = -> () { "spaced" }
argument = ->(value) { value }
keyword = lambda { || "keyword lambda" }

[compact.call, spaced.call, argument.call("argument"), keyword.call]
