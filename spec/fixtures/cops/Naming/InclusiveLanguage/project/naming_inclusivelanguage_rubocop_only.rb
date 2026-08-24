    return if SsrfFilter::DEFAULT_SCHEME_WHITELIST.include?(uri.scheme)
