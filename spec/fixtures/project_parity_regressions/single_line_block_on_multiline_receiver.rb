def deserialize(ciphertext)
  decrypt(ciphertext)
    .yield_self { |cleartext| load(cleartext) } unless ciphertext.nil?
end
