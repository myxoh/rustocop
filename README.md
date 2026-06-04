# rustocop

`rustocop` is a RuboCop-compatible gem backed by a Rust native binary.

The gem name and executable are `rustocop`. RuboCop is the compatibility target:
the specs compare selected `rustocop` output against the real `rubocop` gem so
we can grow behavior cop by cop without guessing at formatter details.

## Current slice

- Ruby gem entrypoint: `exe/rustocop`
- Native binary contract: `libexec/rustocop-native`
- Development fallback: `libexec/rustocop-ruby`
- Rust source: `crates/rustocop`
- Compatibility coverage: `Layout/TrailingWhitespace` JSON output and exit
  status for file and stdin input

## Development

Install dependencies:

```sh
bundle install
```

Run specs:

```sh
bundle exec rake spec
```

Build the native binary when Rust is installed:

```sh
bundle exec rake build:native
```

The build task copies the release binary to `libexec/rustocop-native`, which is
what `exe/rustocop` launches by default. Set `RUSTOCOP_DISABLE_NATIVE=1` to force
the Ruby fallback while developing the gem wrapper or compatibility specs.
