
    records = records.text_search(params[:query]) if params[:query].present?
    records
