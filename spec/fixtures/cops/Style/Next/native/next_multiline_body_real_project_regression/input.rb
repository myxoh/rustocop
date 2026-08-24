sources.each do
  if invalid?
    raise <<~MSG
      First line.
      Second line.
      Third line.
    MSG
  end
end

keys.each do |key|
  unless valid_integer?(key) ||
         infinite?(key) ||
         valid_symbol?(key)
    raise ArgumentError, "invalid"
  end
end

groups.each do |group|
  if active?(group)
    result = condition? ? first : second
    consume(result)
    persist(result)
  end
end

items.find do |item|
  if usable?(item)
    prepare(item)
    validate(item)
    publish(item)
  end
end
