    if req.path_without_extensions == '/api/v1/profile/mfa'
      req.ip if req.delete? # Throttle disable attempts
    elsif req.path_without_extensions.match?(%r{/api/v1/profile/mfa/(verify|backup_codes)})
      req.ip if req.post? # Throttle verify and backup_codes attempts
    end
