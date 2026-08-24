Operations = Struct.new(
  :archive, # whether to archive
  :remote_storage, # whether to upload
  keyword_init: true
)

Compression = Struct.new(
  :compression_cmd, # custom command
  :decompression_cmd, # custom command
  keyword_init: nil
)

Positional = Struct.new(
  :value,
  keyword_init: true,
  keyword_init: false
)

ExplicitHash = Struct.new(:value, { keyword_init: true })
