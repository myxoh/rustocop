  self.confirm_button_label = "Allowlist Email Domain"

  def fields
    field :domain, as: :text, required: true,
      help: "The domain to allowlist (e.g., privaterelay.appleid.com). Subdomains will also be allowed."
    field :notes, as: :textarea,
      help: "Optional context for this allowlist entry."
    super
  end
