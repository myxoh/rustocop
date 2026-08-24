        access_level = group.parent ?
          current_user&.group_members&.find_by(source_id: group.parent&.id)&.access_level :
          Gitlab::Access::OWNER
        Gitlab::Access.human_access(access_level)
        # rubocop:enable Style/MultilineTernaryOperator
