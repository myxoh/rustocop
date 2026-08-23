
  def track_user_experience_sli_by_operation_name
    ::Gitlab::Graphql::UxSliByOperationName
      .new(permitted_params[:operationName]).track { yield }
  end
