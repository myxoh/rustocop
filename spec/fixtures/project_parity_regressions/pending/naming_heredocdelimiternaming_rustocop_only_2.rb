               query.where("ip_address <<= inet ?", params[:q])
