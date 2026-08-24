def begin!(analyzers)
  analyzers.each(&:enable!)
end

newly_enabled = begin!(analyzers)
