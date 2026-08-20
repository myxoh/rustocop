get "/", constraints: lambda { |req|
  req.subdomain.present? && req.subdomain != "clients"
},
  to: lambda { |env| [200, {}, %w[default]] }

callback = lambda { |change_method| lambda { |record|
  touch_record(record, change_method)
}
}

let(:group) { RSpec.describe {
  example("unimplemented", pending: true) { fail }
}
}

scope :with_all_records, -> { includes(blob: {
  variant_records: { image_attachment: :blob },
})
}

allowed = items.map { |item|
  item; }
