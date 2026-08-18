# RuboCop upstream specification snapshot

This directory contains the cop specifications and supporting test data from
RuboCop 1.87.0. Rustocop uses the snapshot as its compatibility contract.

- Repository: <https://github.com/rubocop/rubocop>
- Tag: `v1.87.0`
- Commit: `e5b788dba181ad94de30cfbad661c5d6aa08a4e5`
- Imported paths: `spec/rubocop/cop`, `spec/support`, `spec/fixtures`,
  `spec/core_ext/string.rb`, `spec/spec_helper.rb`, and `config`
- License: MIT; see `LICENSE.txt` in this directory

The files under `spec/rubocop/cop` are preserved as upstream source. Rustocop's
extractor and compatibility runner live outside this directory so generated
reports and local adaptations cannot be confused with upstream tests.
