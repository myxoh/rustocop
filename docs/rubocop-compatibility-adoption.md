# Compatibility-layer project adoption

Generated from project evidence captured at `2026-08-27T07:03:56-04:00`. The
compatibility-layer consumer manifest was updated at `2026-08-26T17:30:20-04:00`.

All 10 registered consumer cops were selected against all
50 pinned projects. Selection alone is not counted as
behavioral coverage. A project is recorded as **diagnostically exercising** a
consumer only when the pinned RuboCop reference emits at least one offense for
that cop. This does not claim that every branch in the shared component ran.

- Projects exercising at least one consumer: 45/50
- Projects exercising every consumer: 1/50
- Registered consumer cops: 10

The machine-readable authority is the `compatibility_layer` section of
[`spec/compatibility_evidence/projects.json`](../spec/compatibility_evidence/projects.json).
Consumer ownership is declared in
[`crates/rustocop/rubocop-consumers.json`](../crates/rustocop/rubocop-consumers.json).
A full project evidence import refreshes both this report and the project-level
rows; `--check` rejects a changed consumer manifest without refreshed evidence.

## Registered consumers

- `Layout/SpaceAfterComma` via `lib/rubocop/cop/mixin/space_after_punctuation.rb`
- `Layout/SpaceAfterSemicolon` via `lib/rubocop/cop/mixin/space_after_punctuation.rb`
- `Style/TrailingCommaInArguments` via `lib/rubocop/cop/mixin/trailing_comma.rb`
- `Style/TrailingCommaInArrayLiteral` via `lib/rubocop/cop/mixin/trailing_comma.rb`
- `Style/TrailingCommaInHashLiteral` via `lib/rubocop/cop/mixin/trailing_comma.rb`
- `Style/HashSlice` via `lib/rubocop/cop/mixin/hash_subset.rb`
- `Style/HashExcept` via `lib/rubocop/cop/mixin/hash_subset.rb`
- `Style/Next` via `lib/rubocop/cop/mixin/min_body_length.rb`
- `Style/OptionalBooleanParameter` via `lib/rubocop/cop/mixin/allowed_methods.rb`
- `Style/NumericPredicate` via `lib/rubocop/cop/mixin/allowed_methods.rb`, `lib/rubocop/cop/mixin/allowed_pattern.rb`

## Coverage by consumer

| Cop | Shared component | Projects exercised | RuboCop diagnostics | Exact exercised projects | Mismatching projects |
| --- | --- | ---: | ---: | ---: | ---: |
| `Layout/SpaceAfterComma` | `lib/rubocop/cop/mixin/space_after_punctuation.rb` | 21/50 | 3203 | 21 | 0 |
| `Layout/SpaceAfterSemicolon` | `lib/rubocop/cop/mixin/space_after_punctuation.rb` | 3/50 | 13 | 3 | 0 |
| `Style/TrailingCommaInArguments` | `lib/rubocop/cop/mixin/trailing_comma.rb` | 16/50 | 36022 | 16 | 0 |
| `Style/TrailingCommaInArrayLiteral` | `lib/rubocop/cop/mixin/trailing_comma.rb` | 27/50 | 4752 | 27 | 0 |
| `Style/TrailingCommaInHashLiteral` | `lib/rubocop/cop/mixin/trailing_comma.rb` | 32/50 | 21428 | 32 | 0 |
| `Style/HashSlice` | `lib/rubocop/cop/mixin/hash_subset.rb` | 10/50 | 43 | 10 | 0 |
| `Style/HashExcept` | `lib/rubocop/cop/mixin/hash_subset.rb` | 14/50 | 32 | 14 | 0 |
| `Style/Next` | `lib/rubocop/cop/mixin/min_body_length.rb` | 25/50 | 319 | 25 | 0 |
| `Style/OptionalBooleanParameter` | `lib/rubocop/cop/mixin/allowed_methods.rb` | 33/50 | 837 | 33 | 0 |
| `Style/NumericPredicate` | `lib/rubocop/cop/mixin/allowed_methods.rb`<br>`lib/rubocop/cop/mixin/allowed_pattern.rb` | 39/50 | 4018 | 39 | 0 |

## Coverage by project

| Project | Repository | Consumers exercised | Exercised cops | Mismatching exercised cops |
| --- | --- | ---: | --- | --- |
| `cancancan` | `CanCanCommunity/cancancan` | 0/10 | — | — |
| `capistrano` | `capistrano/capistrano` | 0/10 | — | — |
| `carrierwave` | `carrierwaveuploader/carrierwave` | 4/10 | `Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/Next`<br>`Style/OptionalBooleanParameter` | — |
| `chatwoot` | `chatwoot/chatwoot` | 2/10 | `Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `debug` | `ruby/debug` | 7/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `devise` | `heartcombo/devise` | 4/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/OptionalBooleanParameter` | — |
| `diaspora` | `diaspora/diaspora` | 6/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashExcept`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `discourse` | `discourse/discourse` | 8/10 | `Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/HashExcept`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `dry-validation` | `dry-rb/dry-validation` | 1/10 | `Style/NumericPredicate` | — |
| `factory_bot` | `thoughtbot/factory_bot` | 3/10 | `Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `faker` | `faker-ruby/faker` | 0/10 | — | — |
| `fastlane` | `fastlane/fastlane` | 4/10 | `Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `fluentd` | `fluent/fluentd` | 8/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashExcept`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `forem` | `forem/forem` | 7/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `github-markup` | `github/markup` | 4/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral` | — |
| `gitlab-ce` | `gitlabhq/gitlabhq` | 7/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/HashExcept`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `grape` | `ruby-grape/grape` | 2/10 | `Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `hanami` | `hanami/hanami` | 0/10 | — | — |
| `homebrew` | `Homebrew/brew` | 4/10 | `Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/OptionalBooleanParameter` | — |
| `huginn` | `huginn/huginn` | 9/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/HashExcept`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `irb` | `ruby/irb` | 7/10 | `Layout/SpaceAfterSemicolon`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `jekyll` | `jekyll/jekyll` | 4/10 | `Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashExcept`<br>`Style/OptionalBooleanParameter` | — |
| `linguist` | `github-linguist/linguist` | 6/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `logger` | `ruby/logger` | 2/10 | `Style/TrailingCommaInHashLiteral`<br>`Style/NumericPredicate` | — |
| `mastodon` | `mastodon/mastodon` | 3/10 | `Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/NumericPredicate` | — |
| `net-http` | `ruby/net-http` | 6/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `openproject` | `opf/openproject` | 6/10 | `Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `paper_trail` | `paper-trail-gem/paper_trail` | 0/10 | — | — |
| `pghero` | `ankane/pghero` | 2/10 | `Style/Next`<br>`Style/NumericPredicate` | — |
| `postal` | `postalserver/postal` | 2/10 | `Style/TrailingCommaInArrayLiteral`<br>`Style/NumericPredicate` | — |
| `psych` | `ruby/psych` | 7/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/HashExcept`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `puma` | `puma/puma` | 6/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `puppet` | `puppetlabs/puppet` | 10/10 | `Layout/SpaceAfterComma`<br>`Layout/SpaceAfterSemicolon`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/HashExcept`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `rack` | `rack/rack` | 7/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `rails` | `rails/rails` | 8/10 | `Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashSlice`<br>`Style/HashExcept`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `rake` | `ruby/rake` | 5/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/NumericPredicate` | — |
| `ransack` | `activerecord-hackery/ransack` | 5/10 | `Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/NumericPredicate` | — |
| `rdoc` | `ruby/rdoc` | 7/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `react_on_rails` | `shakacode/react_on_rails` | 1/10 | `Style/HashSlice` | — |
| `redis-rb` | `redis/redis-rb` | 5/10 | `Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashExcept`<br>`Style/NumericPredicate` | — |
| `resque` | `resque/resque` | 5/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `rspec-core` | `rspec/rspec-core` | 6/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashExcept`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `rubocop` | `rubocop/rubocop` | 2/10 | `Style/HashExcept`<br>`Style/NumericPredicate` | — |
| `rubygems.org` | `rubygems/rubygems.org` | 1/10 | `Style/NumericPredicate` | — |
| `searchkick` | `ankane/searchkick` | 3/10 | `Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `sidekiq` | `sidekiq/sidekiq` | 3/10 | `Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `simple_form` | `heartcombo/simple_form` | 3/10 | `Layout/SpaceAfterComma`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `sinatra` | `sinatra/sinatra` | 7/10 | `Layout/SpaceAfterComma`<br>`Layout/SpaceAfterSemicolon`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashExcept`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `solidus` | `solidusio/solidus` | 3/10 | `Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
| `spree` | `spree/spree` | 8/10 | `Layout/SpaceAfterComma`<br>`Style/TrailingCommaInArguments`<br>`Style/TrailingCommaInArrayLiteral`<br>`Style/TrailingCommaInHashLiteral`<br>`Style/HashExcept`<br>`Style/Next`<br>`Style/OptionalBooleanParameter`<br>`Style/NumericPredicate` | — |
