return unless Source.supported?(import&.provider || params[:provider])
return unless user = find_user || find_verification_user
return unless direct_condition || fallback_condition
