  def disable_query_limiting
    # Also see https://gitlab.com/gitlab-org/gitlab/-/issues/20827
    Gitlab::QueryLimiting.disable!('https://gitlab.com/gitlab-org/gitlab/-/issues/20824')
  end

  def reports_response(report_comparison, pipeline = nil)
    if pipeline&.active?
      ::Gitlab::PollingInterval.set_header(response, interval: 3000)

      return render json: '', status: :no_content
    end

    case report_comparison[:status]
    when :parsing
      ::Gitlab::PollingInterval.set_header(response, interval: 3000)

      render json: '', status: :no_content
    when :parsed
      render json: Gitlab::Json.dump(report_comparison[:data]), status: :ok
    when :error
      render json: {
               errors: [report_comparison[:status_reason]],
               status_reason: report_comparison[:status_reason]
             },
        status: :bad_request
    else
      raise "Failed to build comparison response as comparison yielded unknown status '#{report_comparison[:status]}'"
    end
  end

  def log_merge_request_show
    return unless current_user && @merge_request

    ::Gitlab::Search::RecentMergeRequests.new(user: current_user).log_view(@merge_request)
  end

  def authorize_read_diff_head_pipeline!
    render_404 unless can?(current_user, :read_build, merge_request.diff_head_pipeline)
  end

  def show_whitespace
    current_user&.show_whitespace_in_diffs ? '0' : '1'
  end

  def endpoint_metadata_url(project, merge_request)
    params = request.query_parameters.merge(view: 'inline', diff_head: true, w: show_whitespace)

    diffs_metadata_project_json_merge_request_path(project, merge_request, 'json', params)
  end

  def endpoint_diff_batch_url(project, merge_request)
    per_page = current_user&.view_diffs_file_by_file ? '1' : DIFF_BATCH_ENDPOINT_PER_PAGE.to_s
    params = request
      .query_parameters
      .merge(view: 'inline', diff_head: true, w: show_whitespace, page: '0', per_page: per_page)
    params[:ck] = merge_request.diffs_batch_cache_key if merge_request.diffs_batch_cache_key

    diffs_batch_project_json_merge_request_path(project, merge_request, 'json', params)
  end

  def linked_file_url(project, merge_request)
    diff_by_file_hash_namespace_project_merge_request_path(
      format: 'json',
      id: merge_request.iid,
      namespace_id: project&.namespace.to_param,
      project_id: project&.path,
      file_hash: params[:file],
      diff_head: true
    )
  end

  def append_info_to_payload(payload)
    super

    return unless action_name == 'diffs' && @merge_request&.merge_request_diff.present?

    payload[:metadata] ||= {}
    payload[:metadata]['meta.diffs_files_count'] = @merge_request.merge_request_diff.files_count
  end

  def display_limit_warnings
    if @merge_request.reached_versions_limit?
      flash[:alert] = format(
        _("This merge request has reached the maximum limit of %{limit} versions and cannot be updated further. " \
          "Close this merge request and create a new one instead."), limit: Gitlab::CurrentSettings.diff_max_versions)
      return
    end

    return unless @merge_request.reached_diff_commits_limit?

    flash[:alert] = format(
      _("This merge request has reached the maximum limit of %{limit} diff commits and cannot be updated further. " \
        "Close this merge request and create a new one instead."), limit: Gitlab::CurrentSettings.diff_max_commits)
  end

  def diff_file_component(base_args)
    ::RapidDiffs::MergeRequestDiffFileComponent.new(
      **base_args.merge({
        merge_request: @merge_request,
        conflict_resolution_path: rapid_diffs_presenter.conflict_resolution_path,
        can_merge: rapid_diffs_presenter.can_merge
      })
    )
  end

  def complete_diff_path
    return project_commit_path(project, commit, format: :diff) if commit

    merge_request_path(merge_request, format: :diff)
  end

  def email_format_path
    return project_commit_path(project, commit, format: :patch) if commit

    merge_request_path(merge_request, format: :patch)
  end

  def rapid_diffs_page_enabled?
    return false unless ::Feature.enabled?(:rapid_diffs_on_mr_show, current_user, type: :beta)
    return false if params[:rapid_diffs_disabled] == 'true'
    return true if params[:rapid_diffs] == 'true'

    if ::Feature.enabled?(:rapid_diffs_default_on_mr_show, current_user)
      cookies[:rapid_diffs_enabled] != 'false'
    else
      cookies[:rapid_diffs_enabled] == 'true'
    end
  end
  strong_memoize_attr :rapid_diffs_page_enabled?
