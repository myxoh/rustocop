  alias_method :reset, :reload

  class << self
    # It is strongly suggested use the `.ids` method instead.
    #
    #     User.ids # => returns all the user IDs
    #     User.where(...).ids # => returns the IDs of records matching the where clause.
    #
    alias_method :pluck_primary_key, :ids
  end

  def self.without_order
    reorder(nil)
  end

  def self.id_in(ids)
    where(id: ids)
  end

  def self.primary_key_in(values)
    where(primary_key => values)
  end

  def self.iid_in(iids)
    where(iid: iids)
  end

  def self.id_not_in(ids)
    where.not(id: ids)
  end

  def self.safe_ensure_unique(retries: 0)
    transaction(requires_new: true) do # rubocop:disable Performance/ActiveRecordSubtransactions
      yield
    end
  rescue ActiveRecord::RecordNotUnique
    if retries > 0
      retries -= 1
      retry
    end

    false
  end

  def self.safe_find_or_create_by!(*args, &block)
    safe_find_or_create_by(*args, &block).tap do |record|
      raise ActiveRecord::RecordNotFound unless record.present?

      record.validate! unless record.persisted?
    end
  end

  # Start a new transaction with a shorter-than-usual statement timeout. This is
  # currently one third of the default 15-second timeout with a 500ms buffer
  # to allow callers gracefully handling the errors to still complete within
  # the 5s target duration of a low urgency request.
  def self.with_fast_read_statement_timeout(timeout_ms = 4500)
    ::Gitlab::Database::LoadBalancing::SessionMap.current(load_balancer).fallback_to_replicas_for_ambiguous_queries do
      transaction(requires_new: true) do # rubocop:disable Performance/ActiveRecordSubtransactions
        connection.exec_query("SET LOCAL statement_timeout = #{timeout_ms}")

        yield
      end
    end
  end
