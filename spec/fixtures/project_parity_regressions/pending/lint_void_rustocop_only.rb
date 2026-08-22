
  def download_with_auth
    SafeFetch.fetch(
      media_url,
      http_basic_authentication: auth_credentials,
      resolver: IPV4_RESOLVER,
      validate_content_type: false
    ) { |result| retain_download(result) }
  end

  def download_without_auth
    file = SafeFetch.fetch(media_url, validate_content_type: false) { |result| retain_download(result) }
    log_download(outcome: 'success_without_auth', attempt: 1)
    file
  rescue SafeFetch::Error => e
    retry_without_auth(e)
  end

  def retry_without_auth(initial_error)
    log_download(outcome: 'retrying_without_auth', attempt: 2, error: initial_error)
    file = SafeFetch.fetch(media_url, validate_content_type: false) { |result| retain_download(result) }
    log_download(outcome: 'success_without_auth', attempt: 2)
    file
  rescue SafeFetch::Error => e
    log_failure(e, 2)
  end

  def retain_download(result)
    filename = result.original_filename
    content_type = result.content_type

    result.tempfile.dup.tap do |file|
      file.define_singleton_method(:original_filename) { filename }
      file.define_singleton_method(:content_type) { content_type }
    end
  end

  def valid_retry_url?
    uri = URI.parse(media_url)
    secure_twilio_api_uri?(uri) && valid_media_path?(uri.path)
  rescue URI::InvalidURIError
    false
  end

  def secure_twilio_api_uri?(uri)
    return false unless uri.is_a?(URI::HTTPS)
    return false unless uri.host&.match?(API_HOST_PATTERN)
    return false if uri.userinfo.present? || uri.port != 443

    uri.query.blank? && uri.fragment.blank?
  end

  def valid_media_path?(path)
    prefix = "/2010-04-01/Accounts/#{account_sid}/Messages/#{message_sid}/Media/"
    path.match?(/\A#{Regexp.escape(prefix)}ME[0-9a-f]{32}\z/i)
  end

  def http_status(error)
    error.message.to_s[/\A(\d{3})\b/, 1]&.to_i
  end

  def log_failure(error, attempt)
    log_download(outcome: 'skipped', attempt: attempt, error: error)
    nil
  end
