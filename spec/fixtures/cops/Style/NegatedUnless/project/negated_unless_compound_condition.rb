return unless !object.available? || forbidden?(object)
render_error unless !user.confirmed? || user.pending_email?

return unless !object.available?
