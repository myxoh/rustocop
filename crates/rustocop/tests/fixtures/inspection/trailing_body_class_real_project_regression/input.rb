class PushTestService
  def test_fcm_via_hub(subscription)
    response = ChatwootHub.send_push_with_response(fcm_options(subscription))
    result(subscription, 'fcm_via_hub', :success, "HTTP #{response.code} — #{response.body}")
  rescue RestClient::ExceptionWithResponse => e
    result(subscription, 'fcm_via_hub', :failure, "HTTP #{e.response&.code} — #{e.response&.body}")
  end
end
