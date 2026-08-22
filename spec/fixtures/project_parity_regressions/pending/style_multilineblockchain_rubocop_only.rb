
      keys
        .map do |theme_id, target_id, name|
          ThemeField.where(theme_id: theme_id, target_id: target_id, name: name)
        end
        .inject { |a, b| a.or(b) }
        .each(&:ensure_baked!)
        .map { |tf| [[tf.theme_id, tf.target_id, tf.name], tf.value_baked || tf.value] }
        .group_by(&:first)
