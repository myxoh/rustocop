  def perform(channel, interval = 1)
    Rails.logger.info "[IMAP::FETCH_EMAIL_SERVICE] Job started for inbox #{channel.inbox.id}"

    return log_skipped_fetch(channel) unless should_fetch_email?(channel)

    fetch_mails_with_lock(channel, interval)
  rescue *ExceptionList::IMAP_EXCEPTIONS => e
    Rails.logger.error "Authorization error for email channel - #{channel.inbox.id} : #{e.message}"
  rescue IOError, OpenSSL::SSL::SSLError, Net::IMAP::NoResponseError, Net::IMAP::BadResponseError, Net::IMAP::InvalidResponseError,
         Net::IMAP::ResponseParseError, Net::IMAP::ResponseReadError, Net::IMAP::ResponseTooLargeError => e
    Rails.logger.error "Error for email channel - #{channel.inbox.id} : #{e.message}"
  rescue LockAcquisitionError
    Rails.logger.error "Lock failed for #{channel.inbox.id}"
  rescue StandardError => e
    handle_unexpected_error(e, channel)
  end
