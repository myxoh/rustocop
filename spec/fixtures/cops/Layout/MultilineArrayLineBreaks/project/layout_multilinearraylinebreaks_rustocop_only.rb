    rubygem_trusted_publisher.trusted_publisher.update!(
      repository_owner: "sigstore-conformance",
      repository_name: "extremely-dangerous-public-oidc-beacon",
      workflow_filename: "extremely-dangerous-oidc-beacon.yml"
    )

    @key = "543321"
    create(:api_key, owner: rubygem_trusted_publisher.trusted_publisher, key: @key, scopes: %i[push_rubygem])

    signing_jwt = ["", {
      aud: "sigstore",
      iat: Time.zone.now.to_i - 60,
      exp: Time.zone.now.to_i + 60,
      nbf: Time.zone.now.to_i - 60,
      iss: "sigstore-conformance",
      sub: "sigstore-conformance"
    }.to_json, ""].map { Base64.strict_encode64(it) }.join(".")

    Pusher.any_instance.stubs(:sigstore_signing_jwt).returns(signing_jwt)
    Sigstore::Signer.any_instance.stubs(:sign).returns({})
    bundle = JSON.parse(File.read(gem_file("sigstore-1.0.0.gem.sigstore.json")))

    post api_v1_rubygems_path,
         params: { "gem" => Rack::Test::UploadedFile.new(gem_file("sigstore-1.0.0.gem"), "application/octet-stream"),
                   "attestations" => JSON.dump([bundle]) },
         headers: { "CONTENT_TYPE" => "multipart/mixed", "HTTP_AUTHORIZATION" => @key }

    assert_response :success, response.body

    get info_path("sigstore")
    info_file = response.body

    assert_response :success
