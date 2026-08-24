  def self.unread_messages_count_arel
    messages = Message.arel_table
    conversations = arel_table
    unread_messages = messages
                      .project(messages[:id].count)
                      .where(unread_messages_condition(messages, conversations))

    Arel::Nodes::Grouping.new(unread_messages.ast)
  end

  def self.unread_messages_condition(messages, conversations)
    messages[:conversation_id].eq(conversations[:id])
                              .and(messages[:account_id].eq(conversations[:account_id]))
                              .and(messages[:message_type].eq(Message.message_types[:incoming]))
                              .and(
                                conversations[:agent_last_seen_at].eq(nil)
                                  .or(messages[:created_at].gt(conversations[:agent_last_seen_at]))
                              )
  end

  def recent_messages
    messages.chat.last(5)
  end

  def csat_survey_link
    "#{ENV.fetch('FRONTEND_URL', nil)}/survey/responses/#{uuid}"
  end

  def dispatch_conversation_updated_event(previous_changes = nil)
    dispatcher_dispatch(CONVERSATION_UPDATED, previous_changes)
  end

  private
