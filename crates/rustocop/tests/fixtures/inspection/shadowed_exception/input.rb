begin
  work
rescue StandardError, RuntimeError
  handle
end
begin
  work
rescue Exception
  handle
rescue StandardError
  handle
end
begin
  work
rescue RuntimeError
  handle
rescue StandardError
  handle
end
