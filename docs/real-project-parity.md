# Real-project output parity

This report is the broad real-world compatibility standard. It uses RuboCop
1.87.0 with `parser_prism`, the strict configuration in
`benchmark/project-rubocop.yml`, and ten pinned open-source projects containing
54,146 Ruby files.

An exact signature includes the relative path, cop name, severity, message,
start line and column, and end line and column. Count-only equality is not
enough. RuboCop-derived fixtures separately cover autocorrection, adversarial
cases, and configuration branches that the projects do not exercise.

## Latest realistic status

The latest complete 606-cop checkpoint was generated on 2026-08-21 from Rust
source `9e4b1d38c2ae7dacacc544b46fb2de8b4111ed8f` and native binary SHA-256
`3ea33de397f56e8ce1afc50c96761eb207ef7a1af2705ede7f7eeaec324147a2`.
It reported:

| Classification | Complete checkpoint |
| --- | ---: |
| Project-exact | 256 |
| Exact but dormant | 90 |
| Mismatching | 259 |
| Rust crash | 0 |
| RuboCop gate error | 1 |

The remaining RuboCop error is `Lint/RedundantCopDisableDirective`, which
RuboCop refuses to run with `--only`. Rustocop completed the matrix without a
crash. The 259 mismatching cops are current failures, not estimates inherited
from the older checkpoint.

The minimized project-regression corpus contains 100 cases and the
configuration-mutation corpus contains five. They preserve fixed pathological
examples, while the complete matrix catches interactions and unrepresented
syntax across the full 54,146-file corpus.

## Pinned projects

| Project | Ruby files | Revision |
| --- | ---: | --- |
| Chatwoot | 1,842 | `8d93d69e` |
| RubyGems.org | 1,337 | `3201f883` |
| GitLab CE | 30,894 | `67a52644` |
| Rails | 3,445 | `ba4f7369` |
| Discourse | 10,897 | `cec79c60` |
| Mastodon | 3,257 | `60593f6a` |
| Sidekiq | 168 | `1bb4aa06` |
| Devise | 197 | `372b295f` |
| RSpec Core | 233 | `aec5f494` |
| Homebrew | 1,876 | `44d5dd83` |

## Reproducing the complete audit

The audit requires a clean committed Rust tree. It builds the release binary,
records both the Rust commit and binary SHA-256, runs crash gates first, and
then compares complete diagnostic signatures:

```sh
bundle exec ruby script/audit_project_parity.rb \
  --from-position 606 --count 606 \
  --report tmp/project-parity/all-cops-current.json \
  --markdown tmp/project-parity/all-cops-current.md
```

Generated JSON and Markdown under `tmp/project-parity/` are intentionally
untracked. Any committed claim must include the source commit, binary digest,
corpus revisions, classification counts, and whether it came from a complete
matrix or a focused reconciliation.
