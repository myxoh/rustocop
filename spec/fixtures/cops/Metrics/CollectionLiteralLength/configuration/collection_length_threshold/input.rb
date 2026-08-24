one = [1]
two = [1, 2]
pair = { one: 1, two: 2 }
transition [:pending] => :done
nested = [:contact, { retain_name: false, discard_invalid: false }]

def build_account
  transaction do
    @account = create_account
    @user = create_user
  end
  [@user, @account]
rescue StandardError => error
  raise error
end
