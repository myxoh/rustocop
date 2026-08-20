query = "SELECT owners.*
  FROM owners
  ORDER BY owners.name"

command = `git diff HEAD -- #{changed_files.join(" ")}`
message = "Changed files: #{changed_files.join(" ")}"
