query = <<~GRAPHQL
  rules {
    if
    options
  }
GRAPHQL

raise "missing" if
  missing?

if
  ready?
  run
end

query
