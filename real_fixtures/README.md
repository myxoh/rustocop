# Real fixtures

Each directory is an end-to-end compatibility contract:

- `input.rb` is the source presented to the linter.
- `rubocop.yml` is the complete RuboCop configuration for the example.
- `output.out` is RuboCop's `simple` formatter output for `input.rb`.
- `output.rb` is the result of running RuboCop with `--autocorrect-all`.

The initial numbered fixtures are deliberately small placeholders. Replace their
contents with anonymized real-world failures as they become available, keeping
one independently configured scenario per directory.

Regenerate expected files after changing an input or configuration:

```sh
bundle exec ruby script/update_real_fixtures.rb
```

Pass one or more directory names to update only selected fixtures:

```sh
bundle exec ruby script/update_real_fixtures.rb 01_string_literals
```

Run `bundle exec rspec spec/real_fixtures_spec.rb` to verify that every fixture
still matches RuboCop 1.87.0.
