def short_root
  root = "/project"
  home = ENV["HOME"]
  home && root.start_with?(home)
end
