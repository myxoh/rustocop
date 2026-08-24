def normalized_value(query_hash, current_val)
  if query_hash['attribute_key'] == 'phone_number'
    current_val.delete('+')
  elsif query_hash['attribute_key'] == 'country_code'
    current_val.downcase
  else
    current_val.is_a?(String) ? current_val.downcase : current_val
  end
end
