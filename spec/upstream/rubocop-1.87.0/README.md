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

The intentionally-pending manifest is empty, and the active fixture corpus
covers all 606 built-in cops. Files under `spec/rubocop/cop` are preserved
upstream source. Rustocop's extractor and compatibility runner live outside
this directory so generated reports and local adaptations cannot be confused
with upstream tests.

The retained fixture contract was last verified at
`2026-08-24T07:31:54-04:00`: all 28,601 comparable executable cases and all 606
active cops match RuboCop 1.87.0, including correction expectations. Restored
spec files also contain adversarial cases derived from the 50-project audit.
Cases whose assertions depend on runtime state absent from the captured
executable input are listed with reasons in `broken_fixture_cases.yml`; LSP-only
cases remain separately excluded by the comparison runner.
