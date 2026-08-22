    batch = begin
      with_query_timeout_retry { batch_builder.perform }
    rescue ActiveRecord::QueryCanceled
      nil
    end
