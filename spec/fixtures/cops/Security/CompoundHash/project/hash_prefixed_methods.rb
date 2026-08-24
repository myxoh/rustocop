class Parser
  def hash_start
    @depth += 1
  end

  def hash_end
    @depth -= 1
  end
end
