  def discussion_serializer
    DiscussionSerializer.new(project: project, noteable: issuable, current_user: current_user,
      note_entity: ProjectNoteEntity)
  end

  def render_conflict_response
    respond_to do |format|
      format.html do
        @conflict = true # rubocop:disable Gitlab/ModuleWithInstanceVariables
        render :edit
      end

      format.json do
        render json: {
          errors: [
            <<~HEREDOC.squish
            Someone edited this #{issuable.human_class_name} at the same time you did.
            Please refresh your browser and make sure your changes will not unintentionally remove theirs.
            HEREDOC
          ]
        }, status: :conflict
      end
    end
  end

  def authorize_destroy_issuable!
    access_denied! unless can?(current_user, :"destroy_#{issuable.to_ability_name}", issuable)
  end

  def authorize_admin_issuable!
    access_denied! unless can?(current_user, :"admin_#{resource_name}", parent)
  end
