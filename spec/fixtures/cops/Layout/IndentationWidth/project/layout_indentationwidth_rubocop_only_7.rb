  def self.toggle(params, current_user:)
    new(params, current_user: current_user).toggle
  end

  def initialize(params, current_user:)
    @current_user = current_user

    @params = params
    @reactable_id = params[:reactable_id]
    @reactable_type = params[:reactable_type]
    @category = params[:category] || "like"
  end

  attr_reader :category, :params, :current_user, :reactable_type, :reactable_id

  delegate :rate_limiter, to: :current_user

  def create
    destroy_contradictory_mod_reactions if %w[Article Comment].include?(reactable_type)
    return noop_result if existing_reaction

    create_new_reaction
  end

  def toggle
    destroy_contradictory_mod_reactions if %w[Article Comment].include?(reactable_type)
    return handle_existing_reaction if existing_reaction

    create_new_reaction
  end

  private

  def destroy_contradictory_mod_reactions
    return unless ReactionCategory[category].privileged?

    reactions = Reaction.contradictory_mod_reactions(
      category: category,
      reactable_id: reactable_id,
      reactable_type: reactable_type,
      user: current_user,
    )
    return if reactions.blank?

    reactions.find_each { |reaction| destroy_reaction(reaction) }
  end

  def destroy_reaction(reaction)
    destroyed_reaction = Reaction.transaction do
      locked = Reaction.lock.find_by(id: reaction.id)
      next unless locked

      locked.destroy
      locked
    end

    return unless destroyed_reaction

    sink_articles(destroyed_reaction)
    send_notifications_without_delay(destroyed_reaction)
    destroyed_reaction
  end

  def existing_reaction
  @existing_reaction ||= Reaction.where(
    user_id: current_user.id,
    reactable_id: reactable_id,
    reactable_type: reactable_type,
    category: category,
  ).first
  end

  def create_new_reaction
    reaction = build_reaction(category)
    result = result(reaction, nil)

    begin
      if reaction.save
        rate_limit_reaction_creation
        sink_articles(reaction)
        send_notifications(reaction)
        record_feed_event(reaction)
        update_last_reacted_at(reaction)
      end
    rescue ActiveRecord::RecordNotUnique
      Rails.logger.error "Reaction already exists: #{reaction.inspect}"
    end

    result.action = "create"

    if category == "readinglist" && current_user.setting.experience_level
      rate_article(reaction)
    end

    if current_user.auditable?
      Audit::Logger.log(:moderator, current_user, params.dup)
    end

    result
  end

  def build_reaction(category)
    create_params = {
      user_id: current_user.id,
      reactable_id: reactable_id,
      reactable_type: reactable_type,
      category: category
    }
    if (current_user&.any_admin? || current_user&.super_moderator?) &&
        ReactionCategory[category].negative?
      create_params[:status] = "confirmed"
    end
    Reaction.new(create_params)
  end

  def handle_existing_reaction
    locked_reaction = destroy_reaction(existing_reaction)
    log_audit(locked_reaction) if locked_reaction
    result(existing_reaction, "destroy")
  end

  def log_audit(reaction)
    return unless reaction.negative? && current_user.auditable?

    updated_params = params.dup
    updated_params[:action] = "destroy"
    Audit::Logger.log(:moderator, current_user, updated_params)
  end

