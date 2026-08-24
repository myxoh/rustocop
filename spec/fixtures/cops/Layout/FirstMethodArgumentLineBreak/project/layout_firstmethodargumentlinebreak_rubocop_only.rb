    messages = messages_for(source_entries, mappings)

    Batch.new(items: source_entries.map.with_index do |source_entry, position|
      build_entry(source_entry, source_entry.fetch(:position, position), mappings, messages)
    end)
