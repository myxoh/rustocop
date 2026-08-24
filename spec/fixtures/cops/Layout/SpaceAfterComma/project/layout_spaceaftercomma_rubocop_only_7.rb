    mult = TRUFFLE ? 10 : 80
    get = "GET /#{rand_data(10,120)} HTTP/1.1\r\n" \
      "#{"X-Test: test\r\n" * (mult * 1024)}"
