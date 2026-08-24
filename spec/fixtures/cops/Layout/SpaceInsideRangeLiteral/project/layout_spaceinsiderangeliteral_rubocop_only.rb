        topics = Topic.where(id: (1 .. bind_params_length).to_a << 2**63)
