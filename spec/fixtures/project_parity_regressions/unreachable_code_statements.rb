def raise_before
  raise Before
  yield
end

transaction do
  throw :not_an_error
  assert_predicate record, :approved?
end

raise Exception, 'loading failed'
$load_count += 1

prompt = <<~MARKDOWN
  return
  This remains ordinary documentation.
MARKDOWN

def rescued_return
  begin
    return run_remotely
  rescue ConnectionError
    warn 'falling back'
  end
  run_locally
end
