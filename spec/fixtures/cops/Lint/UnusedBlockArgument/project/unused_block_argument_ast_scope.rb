# Minimized from chatwoot/chatwoot@8d93d69e8e356216e85c28de7c4240e66b8e83fa
# app/builders/messages/messenger/message_builder.rb.

attachments.any? { |existing| existing.external_url == url }

events.each_with_object({}) do |event, messages_by_event_id|
  messages_by_event_id[event.id] = find_message(event)
end

pairs.each_with_object({}) do |((channel_type, status), count), grouped|
  grouped[channel_type] = [status, count]
end

attachments.each { |unused| process_attachment }

lambda { |unused_lambda| process_attachment }

jobs.reduce(0) do |total, (job_efficiency, index)|
  total + (job_efficiency * index)
end

Object.new.tap do |caster|
  def caster.type_for_attribute(name)
    name
  end
end

strategies.reduce("") { |extension, strategy| extension += strategy.extension }
