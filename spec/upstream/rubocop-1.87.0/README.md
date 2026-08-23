# RuboCop upstream specification snapshot

This directory started from the cop specifications and supporting test data
from RuboCop 1.87.0. Rustocop uses the retained active-cop snapshot as its
compatibility contract.

- Repository: <https://github.com/rubocop/rubocop>
- Tag: `v1.87.0`
- Commit: `e5b788dba181ad94de30cfbad661c5d6aa08a4e5`
- Imported paths: `spec/rubocop/cop`, `spec/support`, `spec/fixtures`,
  `spec/core_ext/string.rb`, `spec/spec_helper.rb`, and `config`
- License: MIT; see `LICENSE.txt` in this directory

The 83 cops in `intentionally_pending_cops.yml` have had their cop spec files
removed from the active fixture corpus. The remaining files under
`spec/rubocop/cop` are preserved upstream source. Rustocop's extractor and
compatibility runner live outside this directory so generated reports and
local adaptations cannot be confused with upstream tests.

The retained fixture contract was last verified at
`2026-08-23T13:05:46-04:00`: all 24,297 comparable executable cases and all 523 active cops
match RuboCop 1.87.0, including correction expectations. The eleven restored spec
files also contain adversarial cases derived from the 50-project audit. Cases whose assertions
depend on runtime state absent from the captured executable input are listed
with reasons in `broken_fixture_cases.yml`; LSP-only cases remain separately
excluded by the comparison runner.
