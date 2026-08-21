Parallel.map(items, in_threads: 4) do |item|
  [item.id, item]
end.to_h

items.map do |item|
  [item.id, item]
end.to_h
