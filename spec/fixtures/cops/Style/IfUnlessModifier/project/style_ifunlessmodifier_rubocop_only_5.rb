      @lines = @body.count("\n") + (@body.end_with?("\n") || @body.empty? ? 0 : 1) if @body && !symlink?
