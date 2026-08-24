def platform_app
  @platform_app = @access_token.owner if @access_token && @access_token.owner.is_a?(PlatformApp)
end
