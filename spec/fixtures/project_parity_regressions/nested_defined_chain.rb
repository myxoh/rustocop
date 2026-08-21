def wrap
  return yield unless defined?(Rails) && defined?(Rails.application) && Rails.application

  Rails.application.executor.wrap { yield }
end
