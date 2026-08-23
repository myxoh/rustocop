# frozen_string_literal: true

module Rustocop
  module ProjectCorpus
    RUBOCOP_VERSION = "1.87.0"
    RUBOCOP_COMMIT = "e5b788dba181ad94de30cfbad661c5d6aa08a4e5"

    # Rows contain: local name, GitHub repository, full commit, repository license.
    BASELINE_PROJECTS = [
      ["chatwoot", "chatwoot/chatwoot", "8d93d69e8e356216e85c28de7c4240e66b8e83fa", "MIT outside enterprise/"],
      ["rubygems.org", "rubygems/rubygems.org", "3201f8831866f82eb9acd7f66287a978d0e59079", "MIT"],
      ["gitlab-ce", "gitlabhq/gitlabhq", "67a526442c20d20b6e80ebf916bd766b54018c5e", "MIT Community Edition"],
      ["rails", "rails/rails", "ba4f7369aee71f9f38d67bdbf0e8571fb372b535", "MIT"],
      ["discourse", "discourse/discourse", "cec79c60b354e37e9a119544860123b122a995e0", "GPL-2.0-or-later"],
      ["mastodon", "mastodon/mastodon", "60593f6a8de11effdcf0a0dcb40e22115ae9361a", "AGPL-3.0-or-later"],
      ["sidekiq", "sidekiq/sidekiq", "1bb4aa06e5aa178a114a5e855f9f3d5c24f6c61b", "LGPL-3.0-or-later"],
      ["devise", "heartcombo/devise", "372b295fe6f63b4af3269f5dcd51a18c0bc2016c", "MIT"],
      ["rspec-core", "rspec/rspec-core", "aec5f49485d97908183dbe790a7fefb8baaa8091", "MIT"],
      ["homebrew", "Homebrew/brew", "44d5dd835c14c1beadd5b75c49835ae391cfd86b", "BSD-2-Clause"]
    ].each(&:freeze).freeze

    EXPANSION_PROJECTS = [
      ["jekyll", "jekyll/jekyll", "74d751339d3e534aa51d5d7b0640e9bd743509e4", "MIT"],
      ["fastlane", "fastlane/fastlane", "a9a72554e1f4d6658842d4f3a7b0ca236b5c1589", "MIT"],
      ["huginn", "huginn/huginn", "9faad4aeee0d97570693377e0a281567e775abb4", "MIT"],
      ["diaspora", "diaspora/diaspora", "f96527862dc1ba2e5b95c4196efb0d1f0cc4b6e5", "AGPL-3.0"],
      ["postal", "postalserver/postal", "d038eaa8c763d3cafa797ccd6f773d53470bd336", "MIT"],
      ["forem", "forem/forem", "f354c376a7c5d1330dc40d66f150be8d1289020d", "AGPL-3.0"],
      ["openproject", "opf/openproject", "9579995645b707626b8de36fbaf33dfda6c04b9e", "GPL-3.0"],
      ["spree", "spree/spree", "bf44766bd5c7b5f39f3e26a14ceaa9a864cb5a06", "BSD-3-Clause"],
      ["solidus", "solidusio/solidus", "1f5bf5c638f5fe9fbda4d480bdb7ebcfa39a9a5e", "BSD-3-Clause"],
      ["fluentd", "fluent/fluentd", "dd45c6e18dc7be33b5e5a0f0767bf46307ff5626", "Apache-2.0"],
      ["rubocop", "rubocop/rubocop", "3a42c622171794510e03999c8d55129b73c5dc8f", "MIT"],
      ["puppet", "puppetlabs/puppet", "e227c27540975c25aa22d533a52424a9d2fc886a", "Apache-2.0"],
      ["capistrano", "capistrano/capistrano", "b54b02fa0ecdf18fad97e7ccbacd4b8d4e342c83", "MIT"],
      ["sinatra", "sinatra/sinatra", "cb22afd7902b566b6eaba6c4ea89739494a65d12", "MIT"],
      ["hanami", "hanami/hanami", "2c785981724efbf25f13acbc9d5db287011e949a", "MIT"],
      ["rack", "rack/rack", "8bf4eb078498edc9105fb04add80d89c5340e60b", "MIT"],
      ["puma", "puma/puma", "b8341dc946f0a2444e5ea6730d1967ca53c8d006", "BSD-3-Clause"],
      ["faker", "faker-ruby/faker", "cca4184947e09fdd02afb8b89d25a9c8ebc7274e", "MIT"],
      ["factory_bot", "thoughtbot/factory_bot", "18ae8b581bf55de681c8adb5f74d32787fd2157f", "MIT"],
      ["carrierwave", "carrierwaveuploader/carrierwave", "2072b120c8346cf0971f34dacc9e863c034cddfd", "MIT"],
      ["paper_trail", "paper-trail-gem/paper_trail", "098058ae472d13763fe66e6866a6a4dfc64a3eca", "MIT"],
      ["cancancan", "CanCanCommunity/cancancan", "8c1bf153a3da7b2261d6fa4a5f84eb28e2feb828", "MIT"],
      ["simple_form", "heartcombo/simple_form", "18f38aad0bdeca2ba1815043b94d96fdbbe6a325", "MIT"],
      ["ransack", "activerecord-hackery/ransack", "e82f6bab3956c5597e7debbefa218d9e94e58ceb", "MIT"],
      ["resque", "resque/resque", "bb0f097121f14ab1a77b06cbe8fc0606a8a2f2c7", "MIT"],
      ["redis-rb", "redis/redis-rb", "2ba9010b91dab9e0fde1fbae3a9aae003f8bc307", "MIT"],
      ["grape", "ruby-grape/grape", "60c1e842ceaae37adc3062600ad1579b6f8cb90e", "MIT"],
      ["dry-validation", "dry-rb/dry-validation", "e7dff1eddfa98a2bab3acd895535c29b1e0b294c", "MIT"],
      ["react_on_rails", "shakacode/react_on_rails", "e9dd9cdbf9fdcdcad028622637b9147bc95abd1c", "MIT"],
      ["searchkick", "ankane/searchkick", "93e901a75b11a25101668a616e006b158251b16e", "MIT"],
      ["pghero", "ankane/pghero", "7edb57986ffd36f9d64f0830c4ccc90a5eac46d6", "MIT"],
      ["linguist", "github-linguist/linguist", "b45dbe9b2825a43285bcd035861be91cc0a7299e", "MIT"],
      ["github-markup", "github/markup", "76e2682193828b98471b3a071edf4db0590ccacb", "MIT"],
      ["rake", "ruby/rake", "ec87311d9339f6ed9ff7143fa8e449a97ba34f1b", "MIT"],
      ["irb", "ruby/irb", "3794e99709ae114f35c3f8f227748fca6e95df04", "BSD-2-Clause"],
      ["debug", "ruby/debug", "6510cfbc7496c55ebbefa437a25c17ca58f7c5eb", "BSD-2-Clause"],
      ["psych", "ruby/psych", "9b12bb3fb679c2bc2399ffec0bed15b58ce839e8", "MIT"],
      ["net-http", "ruby/net-http", "104338732a17f30e8c06a6504a848d911eba4d00", "Ruby/BSD-2-Clause"],
      ["logger", "ruby/logger", "026eb9689c031cc7a720909f06d5b2927637fd3d", "Ruby/BSD-2-Clause"],
      ["rdoc", "ruby/rdoc", "5bd8719f1a8ab12bd180b0b0632369f9bcafd547", "Ruby"]
    ].each(&:freeze).freeze

    PROJECTS = (BASELINE_PROJECTS + EXPANSION_PROJECTS).map do |name, repository, revision, license|
      {
        "name" => name,
        "repository" => repository,
        "revision" => revision,
        "license" => license
      }.freeze
    end.freeze
  end
end
