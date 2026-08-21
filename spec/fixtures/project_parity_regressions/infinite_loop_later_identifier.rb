def invoke
  callback = proc do
    while true
      current = next_item
      break current
    end
  end
  callback.call
end

def unrelated(current)
  current
end

def leaking_value
  while true
    suffix = next_item
    break
  end
  use(suffix)
end

def nested_block_local(rows)
  while true
    rows.each do |row|
      id = row.id
      use(id)
    end
    break
  end
  rows.each { |row| use(row.id) }
end
