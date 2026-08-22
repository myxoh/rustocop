# Minimized from gitlabhq/gitlabhq@67a526442c20d20b6e80ebf916bd766b54018c5e
# app/models/doorkeeper/concerns/token_fallback.rb.

source = <<~RUBY
  begin
    operation
  rescue StandardError => error
    warn error
  end
RUBY

begin
  operation
rescue StandardError => err
  warn err
end

begin
  operation
rescue HandledError => handled
  e = handled.cause
  warn e
rescue FatalError => exception
  warn exception
end
