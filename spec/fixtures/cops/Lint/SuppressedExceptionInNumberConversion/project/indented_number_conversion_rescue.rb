page = begin
  Integer(params[:page])
rescue ArgumentError, TypeError
  nil
end
