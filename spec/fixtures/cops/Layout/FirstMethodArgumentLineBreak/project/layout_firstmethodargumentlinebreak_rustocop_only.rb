    # Use database query to find the message efficiently
    # Search for exact match or with angle brackets
    conversation.messages
                .where.not(source_id: nil)
                .where('source_id = ? OR source_id = ? OR source_id = ?',
                       normalized_id,
                       "<#{normalized_id}>",
                       in_reply_to_message_id)
                .first
