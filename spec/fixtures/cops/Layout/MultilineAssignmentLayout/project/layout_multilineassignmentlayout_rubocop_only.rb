  def drilldown_scope
    if message_metric?
      message_scope
    elsif conversation_metric?
      conversation_scope
    else
      reporting_event_scope
    end
  end

  def message_scope
    scope.messages
         .where(account_id: account.id, created_at: bucket_range)
         .public_send(MESSAGE_METRICS.fetch(metric))
         .includes(:sender, conversation: [:assignee, :contact, :inbox])
         .reorder(created_at: :desc)
  end

  def conversation_scope
    scope.conversations
         .where(account_id: account.id, created_at: bucket_range)
         .includes(:assignee, :contact, :inbox)
         .order(created_at: :desc)
  end

  def reporting_event_scope
    events = scope.reporting_events
                  .where(account_id: account.id, name: raw_event_name, created_at: bucket_range)
                  .includes(:user, :inbox, conversation: [:assignee, :contact, :inbox])
                  .order(created_at: :desc)

    if raw_count_strategy == :exclude_bot_handoffs
      events = events.where.not(conversation_id: bot_handoff_conversation_ids_subquery)
    elsif raw_count_strategy == :distinct_conversation
      events = events.where(id: distinct_conversation_event_ids(events))
    end

    events
  end

  def bot_handoff_conversation_ids_subquery
    scope.reporting_events
         .where(account_id: account.id, name: :conversation_bot_handoff, created_at: range)
         .where.not(conversation_id: nil)
         .select(:conversation_id)
  end

  def distinct_conversation_event_ids(events)
    events.reorder(nil)
          .where.not(conversation_id: nil)
          .select('MAX(reporting_events.id)')
          .group(:conversation_id)
  end

  def record_serializer(records)
    @record_serializer ||= V2::Reports::DrilldownRecordSerializer.new(
      account,
      metric,
      use_business_hours?,
      records
    )
  end

  def bucket_range
    @bucket_range ||= begin
      bucket_start = Time.zone.at(params[:bucket_timestamp].to_i).in_time_zone(timezone)
      bucket_end = bucket_end_for(bucket_start)
      requested_start = Time.zone.at(params[:since].to_i)
      requested_end = Time.zone.at(params[:until].to_i)

      [bucket_start, requested_start].max...[bucket_end, requested_end].min
    end
  end

  def bucket_end_for(bucket_start)
    {
      'hour' => bucket_start + 1.hour,
      'day' => bucket_start + 1.day,
      'week' => bucket_start + 1.week,
      'month' => bucket_start + 1.month,
      'year' => bucket_start + 1.year
    }.fetch(group_by)
  end

  def scope
    case dimension_type
    when 'account' then account
    when 'inbox' then inbox
    when 'agent' then user
    when 'label' then label
    when 'team' then team
    else
      raise ArgumentError, "Unsupported drilldown dimension type: #{dimension_type}"
    end
  end

  def inbox = @inbox ||= account.inboxes.find(params[:id])

  def user = @user ||= account.users.find(params[:id])

  def label = @label ||= account.labels.find(params[:id])

  def team = @team ||= account.teams.find(params[:id])

  def metric
    params[:metric].to_s
  end

  def report_metric
    @report_metric ||= Reports::ReportMetricRegistry.fetch(metric)
  end

  def raw_event_name
    report_metric&.raw_event_name
  end

  def raw_count_strategy
    report_metric&.raw_count_strategy
  end

  def record_type
    return 'message' if message_metric? || MESSAGE_EVENT_METRICS.include?(metric)

    'conversation'
  end

  def message_metric?
    MESSAGE_METRICS.key?(metric)
  end
