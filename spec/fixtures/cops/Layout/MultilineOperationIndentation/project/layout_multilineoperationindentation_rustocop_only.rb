  def format_time(time_string)
    return 'not specified' if time_string.blank?

    Time.zone.parse(time_string).strftime('%B %d, %Y %H:%M:%S %Z')
  end

  def subject_for(account)
    "Account Deletion Notice for #{account.id} - #{account.name}"
  end

  def instance_admin_email
    GlobalConfig.get('CHATWOOT_INSTANCE_ADMIN_EMAIL')['CHATWOOT_INSTANCE_ADMIN_EMAIL']
  end

  def instance_url
    ENV.fetch('FRONTEND_URL', 'not available')
  end
