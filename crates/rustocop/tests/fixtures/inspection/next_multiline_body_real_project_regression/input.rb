sources.each do
  if invalid?
    raise <<~MSG
      First line.
      Second line.
      Third line.
    MSG
  end
end
