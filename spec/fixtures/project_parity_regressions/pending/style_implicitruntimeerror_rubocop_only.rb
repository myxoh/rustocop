      raise 'Conversation already present' if @contact_inbox.reload.conversations.present?
