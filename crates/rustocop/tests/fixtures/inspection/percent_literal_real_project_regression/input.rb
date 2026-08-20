def report_progress(i, completion_percentage)
  print "\rCreating conversations: #{i + 1}/#{TOTAL_CONVERSATIONS} (#{completion_percentage}%)"
end
