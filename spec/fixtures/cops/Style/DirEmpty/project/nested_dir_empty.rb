def delete_empty_directories(dir)
  return if Dir.children(dir).empty?

  work
end
