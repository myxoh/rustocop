body.each do
  latch.wait
  body.close
  break
end

fragment.each do |element|
  element.css("a").each do |inner|
    return
  end
  return
end

items.each do |item|
  next if item.skip?
  return item
end

items.each do |item|
  break if item.done?
end

while host
  begin
    return host.call
  rescue StandardError
    retry
  end
end
