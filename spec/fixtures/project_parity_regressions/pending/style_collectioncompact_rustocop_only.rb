    invited_users = invites.includes(:user).reject { |invite| invite.role.nil? || invite.outside_contributor? }.map(&:user)
