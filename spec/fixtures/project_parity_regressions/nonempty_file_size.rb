def missing?(path)
  bad = true
  bad = false if File.size(path) != 0
  bad
end
