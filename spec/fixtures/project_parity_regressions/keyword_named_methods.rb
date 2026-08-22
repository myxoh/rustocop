records = records.or(personal)
visible = action_name.in?(allowed_actions)
negated = relation.not(value: nil)
