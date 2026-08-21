def read
  work
rescue => error # rubocop:disable Style/RescueStandardError
  warn error
end
