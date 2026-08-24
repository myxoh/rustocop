  def self.arm_anchor_for(conversation, message)
    if message.nil?
      conversation.status_changed_at
    elsif message.incoming?
      conversation.waiting_since.presence || message.created_at
    else
      message.created_at
    end
  end

  # Arming keys differ from the strict fire-time keys wherever current state can already reflect the
  # event the row waits for: MESSAGE_CREATED dispatches asynchronously, so this can run long after
  # the message it arms.
  def self.arm_episode_key_for(conversation, message)
    return episode_key_for(conversation, message) if message.nil?

    if message.incoming?
      # waiting_since is written just after MESSAGE_CREATED dispatches, so it can still be nil here.
      # It becomes the starting message's created_at, so use that; the strict fire-time key then
      # matches once waiting_since is settled.
      return episode_key_for(conversation, message) if conversation.waiting_since.present?

      "awaiting_agent:#{microsecond_stamp(message.created_at)}"
    else
      # Count only the replies that predate the agent message being chased. A customer reply that
      # landed while this job queued must end the episode at fire time, not be baked into its key.
      "reply_chase:#{conversation.messages.incoming.where(id: ...message.id).maximum(:id) || 0}"
    end
  end

  # Microsecond integer, not a float: epoch seconds carry ~16 significant digits, past float64's
  # precision, so an in-memory timestamp (arm time) and its DB-reloaded value (fire time) would
  # round to different floats. strftime is exact on both. Sub-second distinguishes rapid episodes.
