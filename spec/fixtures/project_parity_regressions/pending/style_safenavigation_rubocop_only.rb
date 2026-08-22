
      if sym == :notify_user &&
           (
             (scope.current_user.present? && scope.current_user == object.user) ||
               (object.user && object.user.bot?)
           )
        summary.delete(:can_act)
      end
