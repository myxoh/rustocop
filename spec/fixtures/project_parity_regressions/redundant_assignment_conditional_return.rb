def filtered(records, query)
  records = records.text_search(query) if query
  records
end
