    return unless policy_class.present? && defined?(Admin.const_get(policy_class.to_s)&.const_get("Scope"))
