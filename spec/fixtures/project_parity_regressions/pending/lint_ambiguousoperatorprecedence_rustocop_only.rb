    entries = []
    source = @source_conversation['source'].to_h
    if self.class.source_message_importable?(source)
      entries << {
        source_id: "conversation:#{source_conversation_id}:source:#{source['id'].presence || 'initial'}",
        part: source.merge('part_type' => 'source', 'created_at' => @source_conversation['created_at'])
      }
    end
