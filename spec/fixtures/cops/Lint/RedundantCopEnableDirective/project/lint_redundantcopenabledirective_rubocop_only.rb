  let(:response) { double(Faraday::Response, body: body, parsed: parsed_response) }
  # rubocop:enable RSpec/VerifiedDoubles
  let(:response_caller) { -> { response } }
