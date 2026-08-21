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

The latest complete 606-cop checkpoint was generated from Rust source
`84c8a3f842c709662f961009b8f1a74ae6fa3ea7` and native binary SHA-256
`c1a789c3c97f3a56b771430e3ceacbc77f21469923ff433de1399219ee9b1d03`.
It reported:

| Classification | Complete checkpoint |
| --- | ---: |
| Project-exact | 173 |
| Exact but dormant | 73 |
| Mismatching | 359 |
| Rust crash | 0 |
| RuboCop gate error | 1 |

The remaining RuboCop error is `Lint/RedundantCopDisableDirective`, which
RuboCop refuses to run with `--only`. The previous Rust crash was
`Style/CommentAnnotation`; its UTF-8 failure is fixed and included in the
complete checkpoint.

The current working batch fixes ten one-diagnostic mismatches. Its minimized
fixtures match RuboCop and its focused ten-project audit is exact. That result
is intentionally not folded into the table until the Rust changes are
committed and the audit records their source commit and binary SHA-256.

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
