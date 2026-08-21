digest = OpenSSL::Digest::SHA256.new
value = OpenSSL::Digest::SHA1.digest("payload")
cipher = OpenSSL::Cipher::AES.new(128, :GCM)

error_class = OpenSSL::Cipher::CipherError
mode = :GCM
dynamic_cipher = OpenSSL::Cipher::AES.new(mode)

[digest, value, cipher, error_class, dynamic_cipher]
