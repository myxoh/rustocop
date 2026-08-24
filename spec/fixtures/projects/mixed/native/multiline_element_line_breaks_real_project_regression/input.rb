class Processor
  def initialize(wildcard_address:,gitlab_host:,forwarder:,
    logger:)
    @logger = logger
  end

  def resolve(source_user, &block)
    model_relations(
      model: "Issue", source_user: source_user, reference: "author_id",
      alias_version: 1,&block
    )
  end
end

publish(
  model: "Issue",source_user: source_user,reference: "author_id",
  alias_version: 1
)

publish({
  model: "Issue", source_user: source_user,
  alias_version: 1
})

def audit(callback, &)
  subscribe(callback,
    "sql.active_record",&)
end

define_method(method,&lambda do |value|
  value
end)

receiver[
  first, second,
  third
] = value
