  MESSAGE_METRICS = {
    'incoming_messages_count' => :incoming,
    'outgoing_messages_count' => :outgoing
  }.freeze
  MESSAGE_EVENT_METRICS = %w[avg_first_response_time reply_time].freeze

  pattr_initialize :account, :params

  def self.supported_dimension_type?(type) = SUPPORTED_DIMENSION_TYPES.include?((type.presence || 'account').to_s)

  def build
    records = paginated_records.to_a
    { meta: meta, payload: records.map { |record| record_serializer(records).serialize(record) } }
  end

  private
