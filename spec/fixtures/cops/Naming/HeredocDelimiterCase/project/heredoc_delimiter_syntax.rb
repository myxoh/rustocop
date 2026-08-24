# Minimized from rails/rails@ba4f7369aee71f9f38d67bdbf0e8571fb372b535
# actionmailbox/lib/generators/action_mailbox/install/install_generator.rb.

def configure(environment, buffer, chunk)
  buffer << chunk

  environment <<~end_of_config
    config.action_mailbox.ingress = :relay
  end_of_config
end
