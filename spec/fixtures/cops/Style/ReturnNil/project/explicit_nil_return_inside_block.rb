def declaration
  matches.each do |match|
    return match unless match == :let
    return nil
  end
  nil
end
