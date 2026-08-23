    raise ArgumentError, "Invalid URL scheme: #{uri.scheme}. Only http and https are allowed" unless %w[http https].include?(uri.scheme)
