def endpoint(api_url, streaming)
  api_url = streaming ? (api_url + "-with-response-stream") : api_url
  api_url
end
