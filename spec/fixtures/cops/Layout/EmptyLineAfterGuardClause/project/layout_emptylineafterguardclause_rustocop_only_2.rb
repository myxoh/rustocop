  def job_workflow_refs(jwt)
    if workflow_repository_owner.present?
      refs = []
      if (jwf_ref = jwt[:job_workflow_ref])
        expected_prefix = "#{workflow_repository}/#{workflow_slug}@"
        unless jwf_ref.start_with?(expected_prefix)
          raise OIDC::AccessPolicy::AccessError,
            "job_workflow_ref #{jwf_ref} does not match expected prefix #{expected_prefix}"
        end
        refs << jwf_ref.delete_prefix(expected_prefix)
      end
      refs << jwt[:job_workflow_sha] if jwt[:job_workflow_sha].present?
      refs.compact_blank
    else
      [jwt[:ref], jwt[:sha]].compact_blank
    end
  end
