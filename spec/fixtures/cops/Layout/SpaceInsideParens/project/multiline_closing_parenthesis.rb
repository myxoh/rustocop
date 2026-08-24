def deliver
  send_mail_with_liquid(
    template,
    recipient
  )
end
