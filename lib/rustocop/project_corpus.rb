# frozen_string_literal: true

module Rustocop
  module ProjectCorpus
    RUBOCOP_VERSION = "1.87.0"
    RUBOCOP_COMMIT = "e5b788dba181ad94de30cfbad661c5d6aa08a4e5"

    PROJECTS = [
      {
        "name" => "chatwoot",
        "repository" => "chatwoot/chatwoot",
        "revision" => "8d93d69e8e356216e85c28de7c4240e66b8e83fa"
      },
      {
        "name" => "rubygems.org",
        "repository" => "rubygems/rubygems.org",
        "revision" => "3201f8831866f82eb9acd7f66287a978d0e59079"
      },
      {
        "name" => "gitlab-ce",
        "repository" => "gitlabhq/gitlabhq",
        "revision" => "67a526442c20d20b6e80ebf916bd766b54018c5e"
      },
      {
        "name" => "rails",
        "repository" => "rails/rails",
        "revision" => "ba4f7369aee71f9f38d67bdbf0e8571fb372b535"
      },
      {
        "name" => "discourse",
        "repository" => "discourse/discourse",
        "revision" => "cec79c60b354e37e9a119544860123b122a995e0"
      },
      {
        "name" => "mastodon",
        "repository" => "mastodon/mastodon",
        "revision" => "60593f6a8de11effdcf0a0dcb40e22115ae9361a"
      },
      {
        "name" => "sidekiq",
        "repository" => "sidekiq/sidekiq",
        "revision" => "1bb4aa06e5aa178a114a5e855f9f3d5c24f6c61b"
      },
      {
        "name" => "devise",
        "repository" => "heartcombo/devise",
        "revision" => "372b295fe6f63b4af3269f5dcd51a18c0bc2016c"
      },
      {
        "name" => "rspec-core",
        "repository" => "rspec/rspec-core",
        "revision" => "aec5f49485d97908183dbe790a7fefb8baaa8091"
      },
      {
        "name" => "homebrew",
        "repository" => "Homebrew/brew",
        "revision" => "44d5dd835c14c1beadd5b75c49835ae391cfd86b"
      }
    ].freeze
  end
end
