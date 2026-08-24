  def sync_project_namespace?
    (changes.keys & Namespaces::ProjectNamespace::SYNCED_ATTRIBUTES).any? && project_namespace.present?
  end

  def reload_project_namespace_details
    return unless (previous_changes.keys & %w[description description_html cached_markdown_version]).any? && project_namespace.namespace_details.present?

    project_namespace.namespace_details.reset
  end

  # SyncEvents are created by PG triggers (with the function `insert_projects_sync_event`)
  def schedule_sync_event_worker
    run_after_commit do
      Projects::SyncEvent.enqueue_worker
    end
  end

  def check_project_export_limit!
    return if Gitlab::CurrentSettings.current_application_settings.max_export_size == 0

    if self.statistics.export_size > Gitlab::CurrentSettings.current_application_settings.max_export_size.megabytes
      raise ExportLimitExceeded, _('The project size exceeds the export limit.')
    end
  end

  def remove_leading_spaces_on_name
    name&.lstrip!
  end

  def set_last_activity_at
    return if last_activity_at_changed?

    if new_record? || (changed & PROJECT_ACTIVITY_ATTRIBUTES).any?
      self.last_activity_at = Time.current
    elsif last_activity_at.nil?
      self.last_activity_at = created_at
    end
  end

  def set_package_registry_access_level
    return if !project_feature || project_feature.package_registry_access_level_changed?

    self.project_feature.package_registry_access_level = packages_enabled ? enabled_package_registry_access_level_by_project_visibility : ProjectFeature::DISABLED
  end

  def enabled_package_registry_access_level_by_project_visibility
    case visibility_level
    when PUBLIC
      ProjectFeature::PUBLIC
    when INTERNAL
      ProjectFeature::ENABLED
    else
      ProjectFeature::PRIVATE
    end
  end

  def runners_token_prefix
    RunnersTokenPrefixable::RUNNERS_TOKEN_PREFIX
  end

  def pool_repository_shard_matches_repository?(pool)
    pool_repository_shard = pool.shard.name

    pool_repository_shard == repository_storage
  end
